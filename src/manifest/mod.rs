pub mod raw;

use crate::context::McContext;
use crate::java::JavaDescriptor;
use crate::java::JavaVendor;
use crate::java::JavaVersion;
use crate::manifest::raw::RawManifest;
use crate::minecraft::loader::LoaderKind;
use crate::minecraft::seed::MinecraftSeed;
use crate::resolvers::java::JavaVersionResolver;
use crate::resolvers::loader::LoaderVersionResolver;
use crate::resolvers::minecraft::MinecraftVersionResolver;
use crate::services;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::ProductDescriptor;
use crate::utils::product_descriptor::VersionResolver;

pub struct Manifest {
    name: String,

    pub java: ManifestJava,
    pub minecraft: ManifestMinecraft,
    pub server: ManifestServer,
    pub mods: ManifestMods,
    pub backups: ManifestBackups
}

pub struct ManifestJava {
    pub version: JavaDescriptor
}

pub struct ManifestMinecraft {
    pub version: String,
    pub loader: Option<ProductDescriptor<LoaderKind>>
}

pub struct ManifestServer {
    pub seed: MinecraftSeed,
    pub eula: bool,
    pub port: u16,
    pub rcon_port: u16
}

pub struct ManifestMods {
    pub mods: Vec<ManifestMod>
}

pub struct ManifestMod {}

pub struct ManifestBackups {
    pub enabled: bool
}

impl Manifest {
    pub async fn default(context: &McContext) -> McResult<Manifest> {
        let minecraft_latest =
            services::minecraft_api::get_latest_version(&context.http_client).await?;

        Ok(Manifest {
            name: String::from("server"),
            java: ManifestJava {
                version: JavaDescriptor {
                    product: JavaVendor::graal,
                    version: JavaVersion::Java25
                }
            },
            minecraft: ManifestMinecraft {
                version: minecraft_latest,
                loader: None
            },
            server: ManifestServer {
                seed: MinecraftSeed::random(),
                eula: false,
                port: 25565,
                rcon_port: 25575
            },
            mods: ManifestMods { mods: vec![] },
            backups: ManifestBackups { enabled: true }
        })
    }

    pub async fn apply(&mut self, context: &McContext, raw: &RawManifest) -> McResult<()> {
        self.name = raw.name.clone();

        if let Some(ref java) = raw.java {
            if let Some(ref version) = java.version {
                let descriptor =
                    JavaVersionResolver::resolve_descriptor(context, version.clone()).await?;

                self.java.version = descriptor
            }
        }

        if let Some(ref minecraft) = raw.minecraft {
            let version =
                MinecraftVersionResolver::resolve(context, minecraft.version.clone()).await;

            if let Ok(ref version) = version {
                self.minecraft.version = version.clone()
            }

            if let Some(ref loader) = minecraft.loader {
                let descriptor =
                    LoaderVersionResolver::resolve_descriptor(context, loader.clone()).await?;
                self.minecraft.loader = Some(descriptor)
            }
        }

        if let Some(ref server) = raw.server {
            if let Some(ref seed) = server.seed {
                self.server.seed = seed.clone()
            }

            if let Some(eula) = server.eula {
                self.server.eula = eula
            }

            if let Some(port) = server.port {
                self.server.port = port
            }

            if let Some(rcon_port) = server.rcon_port {
                self.server.rcon_port = rcon_port
            }
        }

        if let Some(ref mods) = raw.mods {}

        if let Some(ref backups) = raw.backups {
            if let Some(enabled) = backups.enabled {
                self.backups.enabled = enabled
            }
        }

        Ok(())
    }
}

// TODO: add an apply test
