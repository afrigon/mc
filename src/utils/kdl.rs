use std::fmt::Write;

use kdl::KdlDocument;
use kdl::KdlEntry;
use kdl::KdlEntryFormat;
use kdl::KdlNode;
use kdl::KdlNodeFormat;
use kdl::KdlValue;
use serde::de::DeserializeOwned;

use crate::utils::errors::McResult;

const INDENT: &str = "    ";

/// Nodes that may stand alone, without a value.
const FLAG_NODES: &[&str] = &["on", "tunnel"];

/// Nodes that take several values.
const LIST_NODES: &[&str] = &["jvm-arguments"];

/// Blocks whose children are named after players, carry properties only,
/// and may stand alone.
const PLAYER_GROUPS: &[&[&str]] = &[
    &["players", "allow"],
    &["players", "ban"],
    &["players", "ban-ip"],
    &["players", "op"]
];

pub fn parse_document(source: &str) -> McResult<KdlDocument> {
    KdlDocument::parse_v2(source).map_err(|error| {
        let mut message = String::from("could not parse the KDL document");

        for diagnostic in &error.diagnostics {
            let (line, column) = line_and_column(source, diagnostic.span.offset());
            let detail = diagnostic.message.as_deref().unwrap_or("unexpected input");

            _ = write!(message, "\n  line {}, column {}: {}", line, column, detail);

            if let Some(help) = &diagnostic.help {
                _ = write!(message, " ({})", help);
            }
        }

        anyhow::anyhow!(message)
    })
}

pub fn deserialize<T: DeserializeOwned>(source: &str) -> McResult<T> {
    kdl::de::from_str(source).map_err(|error| {
        let message = error
            .to_string()
            .replace("field `#0`", "the bucket value")
            .replace("`#0`", "the bucket value");

        match error.span() {
            Some(span) => {
                let (line, column) = line_and_column(source, span.offset());

                anyhow::anyhow!("line {}, column {}: {}", line, column, message)
            }
            None => anyhow::anyhow!(message)
        }
    })
}

pub struct Validated {
    pub bare_tunnel: bool
}

/// The serde adapter silently drops arguments a struct does not declare and
/// reads a bare node as `true`, so shape mistakes are caught here first,
/// with positions.
pub fn validate(source: &str, document: &KdlDocument) -> McResult<Validated> {
    validate_nodes(source, document, &mut Vec::new())?;

    let bare_tunnel = document
        .get("tunnel")
        .is_some_and(|node| node.entries().is_empty() && node.children().is_none());

    Ok(Validated { bare_tunnel })
}

fn validate_nodes<'a>(
    source: &str,
    document: &'a KdlDocument,
    path: &mut Vec<&'a str>
) -> McResult<()> {
    let mut seen: Vec<&str> = Vec::new();
    let player_entry = PLAYER_GROUPS.contains(&path.as_slice());

    for node in document.nodes() {
        let name = node.name().value();
        let (line, column) = line_and_column(source, node.span().offset());

        if seen.contains(&name) {
            anyhow::bail!(
                "line {}, column {}: the `{}` node appears more than once",
                line,
                column,
                name
            );
        }

        seen.push(name);

        let arguments = node
            .entries()
            .iter()
            .filter(|entry| entry.name().is_none())
            .count();

        match node.children() {
            Some(children) => {
                if arguments > 0 {
                    anyhow::bail!(
                        "line {}, column {}: `{}` cannot have both values and a block",
                        line,
                        column,
                        name
                    );
                }

                path.push(name);
                validate_nodes(source, children, path)?;
                path.pop();
            }
            None if player_entry => {
                if arguments > 0 {
                    anyhow::bail!(
                        "line {}, column {}: `{}` takes properties only, such as `reason=\"...\"`",
                        line,
                        column,
                        name
                    );
                }
            }
            None => {
                if node.entries().is_empty() && !FLAG_NODES.contains(&name) {
                    anyhow::bail!(
                        "line {}, column {}: `{}` is empty; give it a value or a block",
                        line,
                        column,
                        name
                    );
                }

                if arguments > 1 && !LIST_NODES.contains(&name) {
                    anyhow::bail!(
                        "line {}, column {}: `{}` takes a single value",
                        line,
                        column,
                        name
                    );
                }
            }
        }
    }

    Ok(())
}

fn line_and_column(source: &str, offset: usize) -> (usize, usize) {
    let consumed: String = source.chars().take(offset).collect();
    let line = consumed.matches('\n').count() + 1;
    let column = consumed
        .rsplit('\n')
        .next()
        .unwrap_or_default()
        .chars()
        .count()
        + 1;

    (line, column)
}

pub fn node(name: &str, depth: usize) -> KdlNode {
    let mut node = KdlNode::new(name);

    node.set_format(KdlNodeFormat {
        leading: INDENT.repeat(depth),
        before_children: String::from(" "),
        terminator: String::from("\n"),
        ..KdlNodeFormat::default()
    });

    node
}

pub fn leaf(name: &str, value: impl Into<KdlEntry>, depth: usize) -> KdlNode {
    let mut node = node(name, depth);

    node.push(value);

    node
}

/// A string argument that is always written quoted. The crate writes any
/// string that happens to be a valid identifier bare, which reads poorly for
/// values such as version ids.
pub fn quoted(value: &str) -> KdlEntry {
    let mut entry = KdlEntry::new(value);

    entry.set_format(KdlEntryFormat {
        leading: String::from(" "),
        value_repr: quote(value),
        ..KdlEntryFormat::default()
    });

    entry
}

pub fn quoted_property(key: &str, value: &str) -> KdlEntry {
    let mut entry = KdlEntry::new_prop(key, value);

    entry.set_format(KdlEntryFormat {
        leading: String::from(" "),
        value_repr: quote(value),
        ..KdlEntryFormat::default()
    });

    entry
}

/// A non-string property, rendered the way the crate prints the value.
pub fn property(key: &str, value: impl Into<KdlValue>) -> KdlEntry {
    let value = value.into();
    let value_repr = value.to_string();
    let mut entry = KdlEntry::new_prop(key, value);

    entry.set_format(KdlEntryFormat {
        leading: String::from(" "),
        value_repr,
        ..KdlEntryFormat::default()
    });

    entry
}

fn quote(value: &str) -> String {
    let mut output = String::from("\"");

    for character in value.chars() {
        match character {
            '\\' | '"' => {
                output.push('\\');
                output.push(character);
            }
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            _ => output.push(character)
        }
    }

    output.push('"');
    output
}

pub fn set_comment(node: &mut KdlNode, lines: &[&str]) {
    let indent = node
        .format()
        .map(|format| {
            format
                .leading
                .rsplit('\n')
                .next()
                .unwrap_or_default()
                .to_owned()
        })
        .unwrap_or_default();

    let mut leading = String::from("\n");

    for line in lines {
        _ = writeln!(leading, "{}// {}", indent, line);
    }

    leading.push_str(&indent);

    set_leading(node, leading);
}

pub fn add_blank_line_before(node: &mut KdlNode) {
    let leading = node
        .format()
        .map(|format| format.leading.clone())
        .unwrap_or_default();

    set_leading(node, format!("\n{}", leading));
}

fn set_leading(node: &mut KdlNode, leading: String) {
    match node.format_mut() {
        Some(format) => format.leading = leading,
        None => node.set_format(KdlNodeFormat {
            leading,
            before_children: String::from(" "),
            terminator: String::from("\n"),
            ..KdlNodeFormat::default()
        })
    }
}

/// The indentation used by the nodes already in a block, so that a node
/// appended to a user-authored block matches its neighbours.
pub fn indentation_of(document: &KdlDocument, depth: usize) -> String {
    document
        .nodes()
        .iter()
        .filter_map(KdlNode::format)
        .map(|format| format.leading.rsplit('\n').next().unwrap_or_default())
        .find(|indent| !indent.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| INDENT.repeat(depth))
}
