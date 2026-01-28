use serde::Deserialize;

use crate::utils::product_descriptor::RawProductDescriptor;

#[derive(Deserialize)]
pub struct Manifest {
    name: String,

    pub java: Option<ManifestJava>,
    pub minecraft: Option<ManifestMinecraft>,
    pub server: Option<ManifestServer>,
    pub mods: Option<ManifestMods>,
    pub backups: Option<ManifestBackups>
}

#[derive(Deserialize)]
pub struct ManifestJava {
    pub version: Option<RawProductDescriptor>
}

#[derive(Deserialize)]
pub struct ManifestMinecraft {}

#[derive(Deserialize)]
pub struct ManifestServer {
    seed: Option<usize>,
    port: Option<usize>
}

#[derive(Deserialize)]
pub struct ManifestMods {
    pub mods: Option<Vec<ManifestMod>>
}

#[derive(Deserialize)]
pub struct ManifestMod {}

#[derive(Deserialize)]
pub struct ManifestBackups {
    pub enabled: Option<bool>
}
