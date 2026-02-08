use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;
use url::Url;

use crate::minecraft::seed::MinecraftSeed;
use crate::mods::service::ModServiceKind;
use crate::utils::product_descriptor::RawProductDescriptor;

#[derive(Deserialize)]
pub struct RawManifest {
    pub name: String,

    pub java: Option<RawManifestJava>,
    pub minecraft: Option<RawManifestMinecraft>,
    pub server: Option<RawManifestServer>,
    pub mods: Option<HashMap<String, RawManifestMod>>,
    pub backups: Option<RawManifestBackups>
}

#[derive(Deserialize)]
pub struct RawManifestJava {
    pub version: Option<RawProductDescriptor>
}

#[derive(Deserialize)]
pub struct RawManifestMinecraft {
    pub version: Option<String>,
    pub loader: Option<RawProductDescriptor>
}

#[derive(Deserialize)]
pub struct RawManifestServer {
    pub seed: Option<MinecraftSeed>,
    pub eula: Option<bool>,
    pub port: Option<u16>,
    pub rcon_port: Option<u16>
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum RawManifestMod {
    Version(String),
    Detailed(RawDetailedManifestMod),
    Remote(Url)
}

#[derive(Deserialize)]
pub struct RawDetailedManifestMod {
    pub version: String,
    pub service: Option<ModServiceKind>
}

#[derive(Deserialize)]
pub struct RawManifestBackups {
    pub enabled: Option<bool>
}
