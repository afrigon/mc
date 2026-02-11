use std::convert::Infallible;
use std::str::FromStr;

use clap::ValueEnum;
use toml_edit::DocumentMut;
use toml_edit::Item;
use toml_edit::Table;
use toml_edit::value;

use crate::context::McContext;
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
) -> McResult<DocumentMut> {
    let game_version = services::minecraft_api::get_latest_version(&context.http_client).await?;

    let mut document = create_document_base(name, eula, &game_version);

    let mods = match preset {
        ManifestPreset::Vanilla => vec![],
        ManifestPreset::Optimized => get_optimized_modlist(),
        ManifestPreset::Technical => get_technical_modlist()
    };

    if !mods.is_empty() {
        document["minecraft"]["loader"] = value("fabric");

        for m in mods {
            let version = services::modrinth_api::get_latest_version(
                &context.http_client,
                &String::from(m),
                LoaderKind::Fabric,
                &game_version
            )
            .await?;

            document["mods"][m] = value(version.id);
        }
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

fn create_document_base(name: &str, eula: bool, game_version: &String) -> DocumentMut {
    let mut document = DocumentMut::new();
    document["name"] = value(name);
    document["description"] = value("A Minecraft Server");

    // [minecraft]
    let mut minecraft = Table::new();
    minecraft["version"] = value(game_version);
    document["minecraft"] = Item::Table(minecraft);

    // [server]
    let mut server = Table::new();
    server["gamemode"] = value("survival");
    server["difficulty"] = value("normal");
    server["hardcore"] = value(false);
    server["eula"] = value(eula);
    utils::toml::set_comment(
        &mut server,
        "eula",
        vec![
            "Setting this to true indicates YOU have read and agree to the Minecraft EULA (https://aka.ms/MinecraftEULA).",
            "This agreement is between you and Mojang/Microsoft.",
        ]
    );
    document["server"] = Item::Table(server);

    // [backups]
    document["backups"] = Item::Table(Table::new());
    document["backups"]["enabled"] = value(true);

    // [mods]
    document["mods"] = toml_edit::Item::Table(toml_edit::Table::new());

    document
}
