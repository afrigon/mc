use kdl::KdlDocument;
use kdl::KdlDocumentFormat;
use kdl::KdlEntry;
use kdl::KdlNode;
use kdl::KdlNodeFormat;

use crate::manifest::PlayerGroup;
use crate::utils;
use crate::utils::errors::McResult;
use crate::utils::errors::internal;

const GROUPS: &[&str] = &["modrinth", "http"];

/// The children of the block named `name` inside `parent`, creating the
/// block after the last node when it does not exist yet.
fn ensure_block<'a>(
    parent: &'a mut KdlDocument,
    name: &str,
    depth: usize
) -> McResult<&'a mut KdlDocument> {
    if parent.get(name).is_none() {
        let mut block = utils::kdl::node(name, depth);

        if depth == 0 || !parent.nodes().is_empty() {
            utils::kdl::add_blank_line_before(&mut block);
        }

        block.ensure_children();
        parent.nodes_mut().push(block);
    }

    parent
        .get_mut(name)
        .map(KdlNode::ensure_children)
        .ok_or_else(|| {
            internal(format!(
                "the `{}` block vanished right after it was created",
                name
            ))
        })
}

/// Pins the modrinth mod `slug` to `version`, keeping the rest of an existing
/// entry's line intact.
pub fn set_mod_version(document: &mut KdlDocument, slug: &str, version: &str) -> McResult<()> {
    let mods = ensure_block(document, "mods", 0)?;

    for group in GROUPS.iter().filter(|group| **group != "modrinth") {
        if mods
            .get(group)
            .and_then(KdlNode::children)
            .is_some_and(|children| children.get(slug).is_some())
        {
            anyhow::bail!("the mod `{}` is already listed under `{}`", slug, group);
        }
    }

    let modrinth = ensure_block(mods, "modrinth", 1)?;
    let indentation = utils::kdl::indentation_of(modrinth, 2);

    match modrinth.get_mut(slug) {
        Some(existing) => {
            let entries = existing.entries_mut();
            let position = entries.iter().position(|entry| entry.name().is_none());

            match position.and_then(|index| entries.get_mut(index)) {
                Some(argument) => *argument = utils::kdl::quoted(version),
                None => entries.insert(0, utils::kdl::quoted(version))
            }
        }
        None => {
            let mut node = KdlNode::new(slug);

            node.set_format(KdlNodeFormat {
                leading: indentation,
                terminator: String::from("\n"),
                ..KdlNodeFormat::default()
            });
            node.push(utils::kdl::quoted(version));
            modrinth.nodes_mut().push(node);
        }
    }

    Ok(())
}

/// Removes `slug` from whichever source group lists it.
pub fn remove_mod(document: &mut KdlDocument, slug: &str) -> bool {
    let Some(mods) = document
        .get_mut("mods")
        .and_then(|node| node.children_mut().as_mut())
    else {
        return false;
    };

    let mut removed = false;

    for group in mods.nodes_mut() {
        if let Some(children) = group.children_mut() {
            removed |= remove_child(children, slug);
        }
    }

    removed
}

// The newline after `{` belongs to the first child's leading trivia, so
// removing that child hands the line break to whichever child comes first
// now, and removing the last one keeps the block from collapsing.
fn remove_child(children: &mut KdlDocument, name: &str) -> bool {
    let position = children
        .nodes()
        .iter()
        .position(|node| node.name().value() == name);

    let Some(position) = position else {
        return false;
    };

    let removed = children.nodes_mut().remove(position);

    match children.nodes_mut().first_mut() {
        None => keep_block_open(children),
        Some(first) if position == 0 => {
            let line_break = removed
                .format()
                .and_then(|format| format.leading.split_inclusive('\n').next())
                .filter(|prefix| prefix.ends_with('\n'))
                .unwrap_or_default();

            if let Some(format) = first.format_mut() {
                format.leading = format!("{}{}", line_break, format.leading);
            }
        }
        Some(_) => {}
    }

    true
}

// The newline after `{` belongs to the first child's leading trivia, so
// removing the last child would collapse the block onto one line.
fn keep_block_open(children: &mut KdlDocument) {
    let trailing = children
        .format()
        .map(|format| format.trailing.clone())
        .unwrap_or_default();

    children.set_format(KdlDocumentFormat {
        leading: String::from("\n"),
        trailing
    });
}

/// Writes `name` into the player `group` with exactly `entries` as its
/// properties, replacing the properties of an existing entry in place.
pub fn set_player(
    document: &mut KdlDocument,
    group: PlayerGroup,
    name: &str,
    entries: Vec<KdlEntry>
) -> McResult<()> {
    let players = ensure_block(document, "players", 0)?;
    let group = ensure_block(players, group.node_name(), 1)?;
    let indentation = utils::kdl::indentation_of(group, 2);

    match group.get_mut(name) {
        Some(existing) => {
            existing.entries_mut().clear();

            for entry in entries {
                existing.push(entry);
            }
        }
        None => {
            let mut node = KdlNode::new(name);

            node.set_format(KdlNodeFormat {
                leading: indentation,
                terminator: String::from("\n"),
                ..KdlNodeFormat::default()
            });

            for entry in entries {
                node.push(entry);
            }

            group.nodes_mut().push(node);
        }
    }

    Ok(())
}

/// Removes `name` from the player `group`.
pub fn remove_player(document: &mut KdlDocument, group: PlayerGroup, name: &str) -> bool {
    let Some(children) = document
        .get_mut("players")
        .and_then(|node| node.children_mut().as_mut())
        .and_then(|players| players.get_mut(group.node_name()))
        .and_then(|node| node.children_mut().as_mut())
    else {
        return false;
    };

    remove_child(children, name)
}
