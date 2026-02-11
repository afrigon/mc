pub mod lock;
pub mod presets;

use std::collections::HashMap;

use serde::Deserialize;
use url::Url;

use crate::context::McContext;
use crate::java::JavaDescriptor;
use crate::minecraft::MinecraftDifficulty;
use crate::minecraft::MinecraftGamemode;
use crate::minecraft::MinecraftLevelKind;
use crate::minecraft::seed::MinecraftSeed;
use crate::mods::loader::LoaderKind;
use crate::mods::service::ModServiceKind;
use crate::resolvers::java::JavaVersionResolver;
use crate::resolvers::loader::LoaderVersionResolver;
use crate::resolvers::minecraft::MinecraftVersionResolver;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::ProductDescriptor;
use crate::utils::product_descriptor::RawProductDescriptor;
use crate::utils::product_descriptor::VersionResolver;

#[derive(Deserialize)]
pub struct Manifest {
    pub name: String,
    pub description: String,

    #[serde(default)]
    pub java: ManifestJava,

    #[serde(default)]
    pub minecraft: ManifestMinecraft,

    #[serde(default)]
    pub server: ManifestServer,

    #[serde(default)]
    pub mods: HashMap<String, ManifestMod>,

    #[serde(default)]
    pub backups: ManifestBackups
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ManifestMod {
    Version(String),
    Detailed {
        version: String,
        service: ModServiceKind
    },
    Remote {
        url: Url
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub struct ManifestJava {
    pub version: RawProductDescriptor
}

impl ManifestJava {
    pub async fn version_descriptor(&self, context: &McContext) -> McResult<JavaDescriptor> {
        JavaVersionResolver::resolve_descriptor(context, &self.version).await
    }
}

impl Default for ManifestJava {
    fn default() -> Self {
        ManifestJava {
            version: RawProductDescriptor {
                product: String::from("graal"),
                version: Some(String::from("25"))
            }
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct ManifestMinecraft {
    pub version: Option<String>,
    pub loader: Option<RawProductDescriptor>
}

impl ManifestMinecraft {
    pub async fn resolved_version(&self, context: &McContext) -> McResult<String> {
        MinecraftVersionResolver::resolve(context, self.version.as_deref()).await
    }

    pub async fn loader_descriptor(
        &self,
        context: &McContext
    ) -> McResult<Option<ProductDescriptor<LoaderKind>>> {
        if let Some(ref loader) = self.loader {
            Ok(Some(
                LoaderVersionResolver::resolve_descriptor(context, &loader).await?
            ))
        } else {
            Ok(None)
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub struct ManifestServer {
    pub gamemode: MinecraftGamemode,
    pub difficulty: MinecraftDifficulty,
    pub level_type: MinecraftLevelKind,
    pub hardcore: bool,
    pub seed: Option<MinecraftSeed>,
    pub eula: bool,
    pub ip: Option<String>,
    pub port: u16,
    pub rcon_port: u16,
    pub capacity: i32,
    pub view_distance: u8,
    pub simulation_distance: u8
}

impl Default for ManifestServer {
    fn default() -> Self {
        ManifestServer {
            gamemode: MinecraftGamemode::Survival,
            difficulty: MinecraftDifficulty::Normal,
            level_type: MinecraftLevelKind::Normal,
            hardcore: false,
            seed: None,
            eula: false,
            ip: None,
            port: 25565,
            rcon_port: 25575,
            capacity: 20,
            view_distance: 16,
            simulation_distance: 16
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct ManifestBackups {
    pub enabled: bool
}
