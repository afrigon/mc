use std::convert::Infallible;
use std::str::FromStr;

use clap::ValueEnum;
use kdl::KdlDocument;

use crate::context::McContext;
use crate::manifest::document;
use crate::mods::loader::LoaderKind;
use crate::services;
use crate::utils;
use crate::utils::errors::McResult;

#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
pub enum ManifestPreset {
    Vanilla,
    Optimized,
    Technical
}

impl FromStr for ManifestPreset {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "vanilla" | "default" => ManifestPreset::Vanilla,
            "tech" | "technical" => ManifestPreset::Technical,
            _ => ManifestPreset::Optimized
        })
    }
}

pub async fn create_document(
    context: &mut McContext,
    preset: ManifestPreset,
    name: &str,
    eula: bool
) -> McResult<KdlDocument> {
    let game_version = services::minecraft_api::get_latest_version(&context.http_client).await?;

    let mut document =
        create_document_base(name, eula, &game_version, preset != ManifestPreset::Vanilla);

    let mods = match preset {
        ManifestPreset::Vanilla => vec![],
        ManifestPreset::Optimized => get_optimized_modlist(),
        ManifestPreset::Technical => get_technical_modlist()
    };

    for m in mods {
        let version = services::modrinth_api::get_latest_version(
            &context.http_client,
            &String::from(m),
            LoaderKind::Fabric,
            &game_version
        )
        .await?;

        document::set_mod_version(&mut document, m, &version.id)?;
    }

    Ok(document)
}

fn get_optimized_modlist() -> Vec<&'static str> {
    vec!["lithium"]
}

fn get_technical_modlist() -> Vec<&'static str> {
    vec![
        "lithium",
        "servux",
        "carpet",
        "carpet-tis-addition",
        "stackable-shulkers-fix",
        "spark",
        "chunk-debug",
    ]
}

pub(super) fn create_document_base(
    name: &str,
    eula: bool,
    game_version: &str,
    with_loader: bool
) -> KdlDocument {
    let mut document = KdlDocument::new();
    let nodes = document.nodes_mut();

    nodes.push(utils::kdl::leaf("name", utils::kdl::quoted(name), 0));
    nodes.push(utils::kdl::leaf(
        "description",
        utils::kdl::quoted("A Minecraft Server"),
        0
    ));

    let mut minecraft = utils::kdl::node("minecraft", 0);
    utils::kdl::add_blank_line_before(&mut minecraft);
    minecraft
        .ensure_children()
        .nodes_mut()
        .push(utils::kdl::leaf(
            "version",
            utils::kdl::quoted(game_version),
            1
        ));

    if with_loader {
        minecraft
            .ensure_children()
            .nodes_mut()
            .push(utils::kdl::leaf("loader", utils::kdl::quoted("fabric"), 1));
    }

    nodes.push(minecraft);

    let mut server = utils::kdl::node("server", 0);
    utils::kdl::add_blank_line_before(&mut server);

    let mut eula_node = utils::kdl::leaf("eula", eula, 1);
    utils::kdl::set_comment(
        &mut eula_node,
        &[
            "Setting this to true indicates YOU have read and agree to the Minecraft EULA (https://aka.ms/MinecraftEULA).",
            "This agreement is between you and Mojang/Microsoft."
        ]
    );

    server.ensure_children().nodes_mut().extend([
        utils::kdl::leaf("gamemode", utils::kdl::quoted("survival"), 1),
        utils::kdl::leaf("difficulty", utils::kdl::quoted("normal"), 1),
        utils::kdl::leaf("hardcore", false, 1),
        eula_node
    ]);
    nodes.push(server);

    let mut backups = utils::kdl::node("backups", 0);
    utils::kdl::add_blank_line_before(&mut backups);
    backups.ensure_children().nodes_mut().extend([
        utils::kdl::leaf("enabled", true, 1),
        utils::kdl::leaf("frequency", utils::kdl::quoted("0 0 * * * *"), 1)
    ]);
    nodes.push(backups);

    document
}
