pub mod lock;
pub mod presets;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::env;

use serde::Deserialize;
use toml::Table;
use toml::Value;
use url::Url;

use crate::context::McContext;
use crate::java::JavaDescriptor;
use crate::minecraft::MinecraftDifficulty;
use crate::minecraft::MinecraftGamemode;
use crate::minecraft::MinecraftLevelKind;
use crate::minecraft::seed::MinecraftSeed;
use crate::mods::loader::LoaderKind;
use crate::mods::service::ModServiceKind;
use crate::ops::backups::BackupStorage;
use crate::ops::notifications::NotificationKind;
use crate::ops::notifications::NotificationTarget;
use crate::ops::notifications::Notifier;
use crate::ops::notifications::NotifierConfiguration;
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
    pub backups: ManifestBackups,

    #[serde(default)]
    pub notifications: ManifestNotifications
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
    pub version: RawProductDescriptor,
    pub min_memory: usize,
    pub max_memory: usize,
    pub jvm_arguments: Vec<String>
}

impl ManifestJava {
    pub async fn version_descriptor(&self, context: &McContext) -> McResult<JavaDescriptor> {
        JavaVersionResolver::resolve_descriptor(context, &self.version).await
    }

    pub fn args(&self) -> Vec<String> {
        let mut arguments = vec![];

        arguments.push(format!("-Xms{}m", self.min_memory));
        arguments.push(format!("-Xmx{}m", self.max_memory));

        for argument in &self.jvm_arguments {
            arguments.push(argument.clone());
        }

        arguments
    }
}

impl Default for ManifestJava {
    fn default() -> Self {
        ManifestJava {
            version: RawProductDescriptor {
                product: String::from("graal"),
                version: Some(String::from("25"))
            },
            min_memory: 4096,
            max_memory: 4096,
            jvm_arguments: vec![
                String::from("-Djava.net.preferIPv6Addresses=true"),
                String::from("-XX:+AlwaysPreTouch"),
                String::from("-Djdk.graal.TuneInlinerExploration=1"),
            ]
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

impl Manifest {
    /// Providers are enabled by their webhook environment variable alone; the
    /// secret is intentionally never read from `mc.toml`.
    pub fn notifier(&self, context: &McContext) -> Option<Notifier> {
        let targets: Vec<NotificationTarget> = NotificationKind::ALL
            .into_iter()
            .filter_map(|kind| {
                kind.webhook_from_env()
                    .map(|webhook| NotificationTarget { kind, webhook })
            })
            .collect();

        if targets.is_empty() {
            return None;
        }

        let configuration = NotifierConfiguration {
            targets,
            on_lifecycle_event: self.notifications.on_lifecycle_event,
            on_backup: self.notifications.on_backup,
            on_backup_failure: self.notifications.on_backup_failure,
            on_panic: self.notifications.on_panic
        };

        Some(Notifier::new(
            context.http_client.clone(),
            context.shell_handle(),
            configuration
        ))
    }
}

#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct ManifestNotifications {
    pub on_lifecycle_event: bool,
    pub on_backup: bool,
    pub on_backup_failure: bool,
    pub on_panic: bool
}

impl Default for ManifestNotifications {
    fn default() -> Self {
        ManifestNotifications {
            on_lifecycle_event: true,
            on_backup: true,
            on_backup_failure: true,
            on_panic: true
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
    pub simulation_distance: u8,
    pub properties: Table
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
            simulation_distance: 16,
            properties: Table::new()
        }
    }
}

impl ManifestServer {
    pub fn property_overrides(&self) -> McResult<BTreeMap<String, String>> {
        let mut overrides = BTreeMap::new();

        flatten_properties(None, &self.properties, &mut overrides)?;

        Ok(overrides)
    }
}

// TOML parses a dotted key like `rcon.port` as nested tables, so nested tables
// flatten back into dot-joined server.properties keys.
fn flatten_properties(
    prefix: Option<&str>,
    table: &Table,
    output: &mut BTreeMap<String, String>
) -> McResult<()> {
    for (key, value) in table {
        let key = match prefix {
            Some(prefix) => format!("{}.{}", prefix, key),
            None => key.clone()
        };

        match value {
            Value::Table(inner) => flatten_properties(Some(&key), inner, output)?,
            Value::String(value) => {
                output.insert(key, value.clone());
            }
            Value::Integer(value) => {
                output.insert(key, value.to_string());
            }
            Value::Float(value) => {
                output.insert(key, value.to_string());
            }
            Value::Boolean(value) => {
                output.insert(key, value.to_string());
            }
            _ => anyhow::bail!(
                "the server property `{}` must be a string, integer, float, or boolean",
                key
            )
        }
    }

    Ok(())
}

#[derive(Deserialize)]
#[serde(default)]
pub struct ManifestBackups {
    pub enabled: bool,
    pub frequency: String,
    pub storage: BackupStorage
}

impl ManifestBackups {
    /// The storage target to use. When storage is configured as S3,
    /// `MC_BACKUPS_S3_BUCKET` overrides the bucket without editing `mc.toml`;
    /// other storage types ignore it.
    pub fn effective_storage(&self) -> BackupStorage {
        match &self.storage {
            BackupStorage::S3 { bucket } => BackupStorage::S3 {
                bucket: env::var("MC_BACKUPS_S3_BUCKET")
                    .ok()
                    .filter(|bucket| !bucket.is_empty())
                    .or_else(|| bucket.clone())
            },
            local => local.clone()
        }
    }
}

impl Default for ManifestBackups {
    fn default() -> Self {
        ManifestBackups {
            enabled: false,
            frequency: "0 0 * * * *".into(), // every hour (sec min hour day month weekday)
            storage: BackupStorage::Local {
                path: "backups".into(),
                keep: 20
            }
        }
    }
}
