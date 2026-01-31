use serde::Deserialize;

use crate::minecraft::seed::MinecraftSeed;
use crate::utils::product_descriptor::RawProductDescriptor;

#[derive(Deserialize)]
pub struct RawManifest {
    pub name: String,

    pub java: Option<RawManifestJava>,
    pub minecraft: Option<RawManifestMinecraft>,
    pub server: Option<RawManifestServer>,
    pub mods: Option<RawManifestMods>,
    pub backups: Option<RawManifestBackups>
}

#[derive(Deserialize)]
pub struct RawManifestJava {
    pub version: Option<RawProductDescriptor>
}

#[derive(Deserialize)]
pub struct RawManifestMinecraft {
    pub version: Option<String>,
    pub loader: Option<RawProductDescriptor>,
    pub seed: Option<MinecraftSeed>,
    pub eula: Option<bool>
}

#[derive(Deserialize)]
pub struct RawManifestServer {
    pub port: Option<u16>,
    pub rcon_port: Option<u16>
}

#[derive(Deserialize)]
pub struct RawManifestMods {
    pub mods: Option<Vec<RawManifestMod>>
}

#[derive(Deserialize)]
pub struct RawManifestMod {}

#[derive(Deserialize)]
pub struct RawManifestBackups {
    pub enabled: Option<bool>
}
