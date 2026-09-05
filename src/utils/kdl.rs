use std::fmt::Write;
use std::str::FromStr;

use anyhow::Context;
use kdl::KdlDocument;
use kdl::KdlEntry;
use kdl::KdlEntryFormat;
use kdl::KdlNode;
use kdl::KdlNodeFormat;
use kdl::KdlValue;

use crate::utils::errors::McResult;

const INDENT: &str = "    ";

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

pub fn required_child<'a>(document: &'a KdlDocument, name: &str) -> McResult<&'a KdlNode> {
    document
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("the `{}` node is required", name))
}

/// Rejects unknown and repeated node names so that a misspelled key fails
/// loudly instead of silently falling back to its default.
pub fn check_children(document: &KdlDocument, scope: &str, allowed: &[&str]) -> McResult<()> {
    let mut seen: Vec<&str> = Vec::new();

    for node in document.nodes() {
        let name = node.name().value();

        if !allowed.contains(&name) {
            anyhow::bail!("unknown node `{}` in {}", name, scope);
        }

        if seen.contains(&name) {
            anyhow::bail!("the `{}` node appears more than once in {}", name, scope);
        }

        seen.push(name);
    }

    Ok(())
}

pub fn check_properties(node: &KdlNode, allowed: &[&str]) -> McResult<()> {
    for entry in node.entries() {
        if let Some(key) = entry.name()
            && !allowed.contains(&key.value())
        {
            anyhow::bail!(
                "unknown property `{}` on the `{}` node",
                key.value(),
                node.name().value()
            );
        }
    }

    Ok(())
}

pub fn arguments(node: &KdlNode) -> Vec<&KdlValue> {
    node.entries()
        .iter()
        .filter(|entry| entry.name().is_none())
        .map(KdlEntry::value)
        .collect()
}

/// The single value of a leaf node such as `port 25565`. Properties and
/// children are rejected because a leaf node has no use for them.
pub fn argument(node: &KdlNode) -> McResult<&KdlValue> {
    let name = node.name().value();

    check_properties(node, &[])?;

    if node.children().is_some() {
        anyhow::bail!("the `{}` node does not take children", name);
    }

    match arguments(node).as_slice() {
        [value] => Ok(value),
        [] => anyhow::bail!("the `{}` node requires a value", name),
        _ => anyhow::bail!("the `{}` node takes a single value", name)
    }
}

pub fn string_argument(node: &KdlNode) -> McResult<&str> {
    string_value(argument(node)?, node.name().value())
}

pub fn integer_argument<T: TryFrom<i128>>(node: &KdlNode) -> McResult<T> {
    integer_value(argument(node)?, node.name().value())
}

pub fn bool_argument(node: &KdlNode) -> McResult<bool> {
    argument(node)?.as_bool().ok_or_else(|| {
        anyhow::anyhow!("the `{}` node must be #true or #false", node.name().value())
    })
}

pub fn parse_argument<T>(node: &KdlNode) -> McResult<T>
where
    T: FromStr,
    T::Err: Into<anyhow::Error>
{
    string_argument(node)?
        .parse()
        .map_err(Into::into)
        .with_context(|| format!("invalid value for the `{}` node", node.name().value()))
}

pub fn string_arguments(node: &KdlNode) -> McResult<Vec<String>> {
    let name = node.name().value();

    check_properties(node, &[])?;

    arguments(node)
        .into_iter()
        .map(|value| string_value(value, name).map(str::to_owned))
        .collect()
}

pub fn string_property<'a>(node: &'a KdlNode, key: &str) -> McResult<Option<&'a str>> {
    node.get(key)
        .map(|value| string_value(value, key))
        .transpose()
}

pub fn integer_property<T: TryFrom<i128>>(node: &KdlNode, key: &str) -> McResult<Option<T>> {
    node.get(key)
        .map(|value| integer_value(value, key))
        .transpose()
}

fn string_value<'a>(value: &'a KdlValue, name: &str) -> McResult<&'a str> {
    value
        .as_string()
        .ok_or_else(|| anyhow::anyhow!("`{}` must be a string", name))
}

fn integer_value<T: TryFrom<i128>>(value: &KdlValue, name: &str) -> McResult<T> {
    value
        .as_integer()
        .and_then(|integer| T::try_from(integer).ok())
        .ok_or_else(|| anyhow::anyhow!("`{}` must be an integer in range", name))
}

pub fn scalar_to_string(value: &KdlValue) -> Option<String> {
    match value {
        KdlValue::String(string) => Some(string.clone()),
        KdlValue::Integer(integer) => Some(integer.to_string()),
        KdlValue::Float(float) => Some(float.to_string()),
        KdlValue::Bool(boolean) => Some(boolean.to_string()),
        KdlValue::Null => None
    }
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
pub fn indentation_of(document: &KdlDocument) -> String {
    document
        .nodes()
        .iter()
        .filter_map(KdlNode::format)
        .map(|format| format.leading.rsplit('\n').next().unwrap_or_default())
        .find(|indent| !indent.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| String::from(INDENT))
}
