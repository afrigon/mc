use kdl::KdlDocument;
use kdl::KdlDocumentFormat;
use kdl::KdlNode;
use kdl::KdlNodeFormat;

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

        if depth == 0 {
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
            let before = children.nodes().len();

            children
                .nodes_mut()
                .retain(|node| node.name().value() != slug);

            removed |= children.nodes().len() != before;

            if children.nodes().is_empty() {
                keep_block_open(children);
            }
        }
    }

    removed
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
