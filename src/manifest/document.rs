use kdl::KdlDocument;
use kdl::KdlNode;
use kdl::KdlNodeFormat;

use crate::utils;
use crate::utils::errors::McResult;
use crate::utils::errors::internal;

/// The children of the `mods` block, creating the block after the last node
/// when the manifest has none yet.
pub fn ensure_mods_block(document: &mut KdlDocument) -> McResult<&mut KdlDocument> {
    if document.get("mods").is_none() {
        let mut mods = utils::kdl::node("mods", 0);

        utils::kdl::add_blank_line_before(&mut mods);
        mods.ensure_children();
        document.nodes_mut().push(mods);
    }

    document
        .get_mut("mods")
        .map(KdlNode::ensure_children)
        .ok_or_else(|| internal("the mods block vanished right after it was created"))
}

/// Pins `slug` to `version`, keeping any other property of an existing entry.
pub fn set_mod_version(document: &mut KdlDocument, slug: &str, version: &str) -> McResult<()> {
    let mods = ensure_mods_block(document)?;
    let indentation = utils::kdl::indentation_of(mods);

    match mods.get_mut(slug) {
        Some(existing) => {
            let entries = existing.entries_mut();

            entries.retain(|entry| entry.name().map(|name| name.value()) != Some("url"));

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
            mods.nodes_mut().push(node);
        }
    }

    Ok(())
}

pub fn remove_mod(document: &mut KdlDocument, slug: &str) -> bool {
    let Some(mods) = document
        .get_mut("mods")
        .and_then(|node| node.children_mut().as_mut())
    else {
        return false;
    };

    let before = mods.nodes().len();

    mods.nodes_mut().retain(|node| node.name().value() != slug);

    mods.nodes().len() != before
}
