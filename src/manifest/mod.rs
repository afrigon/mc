pub mod lock;
pub mod raw;

use std::collections::HashMap;
use std::path::PathBuf;

use url::Url;

use crate::context::McContext;
use crate::java::JavaDescriptor;
use crate::java::JavaVendor;
use crate::java::JavaVersion;
use crate::manifest::raw::RawManifest;
use crate::manifest::raw::RawManifestMod;
use crate::minecraft::MinecraftDifficulty;
use crate::minecraft::MinecraftGamemode;
use crate::minecraft::seed::MinecraftSeed;
use crate::mods::loader::LoaderKind;
use crate::mods::service::ModServiceKind;
use crate::resolvers::java::JavaVersionResolver;
use crate::resolvers::loader::LoaderVersionResolver;
use crate::resolvers::minecraft::MinecraftVersionResolver;
use crate::services;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::ProductDescriptor;
use crate::utils::product_descriptor::VersionResolver;

pub struct Manifest {
    pub name: String,
    pub java: ManifestJava,
    pub minecraft: ManifestMinecraft,
    pub server: ManifestServer,
    pub mods: HashMap<String, ManifestMod>,
    pub backups: ManifestBackups
}

pub enum ManifestMod {
    Detailed(DetailedManifestMod),
    Remote(Url)
}

pub struct DetailedManifestMod {
    pub version: String,
    pub service: ModServiceKind
}

pub struct ManifestJava {
    pub version: JavaDescriptor
}

pub struct ManifestMinecraft {
    pub version: String,
    pub loader: Option<ProductDescriptor<LoaderKind>>
}

pub struct ManifestServer {
    pub gamemode: MinecraftGamemode,
    pub difficulty: MinecraftDifficulty,
    pub hardcore: bool,
    pub seed: MinecraftSeed,
    pub eula: bool,
    pub ip: Option<String>,
    pub port: u16,
    pub rcon_port: u16
}

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
            mods: HashMap::new(),
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

        if let Some(ref mods) = raw.mods {
            for (k, v) in mods {
                let m = match v {
                    RawManifestMod::Version(version) => {
                        ManifestMod::Detailed(DetailedManifestMod {
                            version: version.clone(),
                            service: ModServiceKind::Modrinth
                        })
                    }
                    RawManifestMod::Detailed(detailed) => {
                        ManifestMod::Detailed(DetailedManifestMod {
                            version: detailed.version.clone(),
                            service: detailed.service.unwrap_or_default()
                        })
                    }
                    RawManifestMod::Remote(url) => ManifestMod::Remote(url.clone())
                };

                self.mods.insert(k.clone(), m);
            }
        }

        if let Some(ref backups) = raw.backups {
            if let Some(enabled) = backups.enabled {
                self.backups.enabled = enabled
            }
        }

        Ok(())
    }
}

// TODO: add an apply test
