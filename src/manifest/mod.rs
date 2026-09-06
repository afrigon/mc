pub mod document;
pub mod lock;
pub mod presets;
mod raw;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::env;
use std::net::IpAddr;
use std::path::PathBuf;

use anyhow::Context;
use chrono::DateTime;
use chrono::Utc;
use url::Url;

use crate::context::McContext;
use crate::java::JavaDescriptor;
use crate::manifest::raw::RawBackups;
use crate::manifest::raw::RawBanEntry;
use crate::manifest::raw::RawJava;
use crate::manifest::raw::RawManifest;
use crate::manifest::raw::RawMinecraft;
use crate::manifest::raw::RawMods;
use crate::manifest::raw::RawNotifications;
use crate::manifest::raw::RawOpEntry;
use crate::manifest::raw::RawPlayers;
use crate::manifest::raw::RawProperty;
use crate::manifest::raw::RawSeed;
use crate::manifest::raw::RawServer;
use crate::manifest::raw::RawTunnel;
use crate::minecraft::MinecraftDifficulty;
use crate::minecraft::MinecraftGamemode;
use crate::minecraft::MinecraftLevelKind;
use crate::minecraft::MinecraftPermission;
use crate::minecraft::players;
use crate::minecraft::seed::MinecraftSeed;
use crate::mods::loader::LoaderKind;
use crate::ops::backups::BackupStorage;
use crate::ops::notifications::NotificationKind;
use crate::ops::notifications::NotificationTarget;
use crate::ops::notifications::Notifier;
use crate::ops::notifications::NotifierConfiguration;
use crate::resolvers::java::JavaVersionResolver;
use crate::resolvers::loader::LoaderVersionResolver;
use crate::resolvers::minecraft::MinecraftVersionResolver;
use crate::resolvers::tunnel::TunnelVersionResolver;
use crate::tunnel::TunnelDescriptor;
use crate::utils;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::ProductDescriptor;
use crate::utils::product_descriptor::RawProductDescriptor;
use crate::utils::product_descriptor::VersionResolver;

pub struct Manifest {
    pub name: String,
    pub description: String,
    pub java: ManifestJava,
    pub minecraft: ManifestMinecraft,
    pub server: ManifestServer,
    pub mods: HashMap<String, ManifestMod>,
    pub backups: ManifestBackups,
    pub notifications: ManifestNotifications,
    pub tunnel: Option<ManifestTunnel>,
    pub players: ManifestPlayers
}

impl Manifest {
    pub fn from_kdl_str(source: &str) -> McResult<Manifest> {
        let document = utils::kdl::parse_document(source)?;
        let validated = utils::kdl::validate(source, &document)?;
        let raw: RawManifest = utils::kdl::deserialize(source)?;

        raw.resolve(validated.bare_tunnel)
    }

    /// Providers are enabled by their webhook environment variable alone; the
    /// secret is intentionally never read from `mc.kdl`.
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
            on_panic: self.notifications.on_panic,
            on_sigkill: self.notifications.on_sigkill
        };

        Some(Notifier::new(
            context.http_client.clone(),
            context.shell_handle(),
            configuration
        ))
    }
}

impl RawManifest {
    fn resolve(self, bare_tunnel: bool) -> McResult<Manifest> {
        let tunnel = if bare_tunnel {
            Some(ManifestTunnel::default())
        } else {
            self.tunnel
                .map(RawTunnel::resolve)
                .transpose()
                .context("invalid `tunnel` section")?
        };

        Ok(Manifest {
            name: self.name,
            description: self.description,
            java: self.java.resolve().context("invalid `java` section")?,
            minecraft: self
                .minecraft
                .resolve()
                .context("invalid `minecraft` section")?,
            server: self.server.resolve().context("invalid `server` section")?,
            mods: self.mods.resolve().context("invalid `mods` section")?,
            backups: self
                .backups
                .resolve()
                .context("invalid `backups` section")?,
            notifications: self.notifications.resolve(),
            tunnel,
            players: self
                .players
                .resolve()
                .context("invalid `players` section")?
        })
    }
}

fn parse_or<T>(value: Option<String>, default: T) -> McResult<T>
where
    T: std::str::FromStr,
    T::Err: Into<anyhow::Error>
{
    match value {
        Some(value) => value.parse().map_err(Into::into),
        None => Ok(default)
    }
}

pub enum ManifestMod {
    Modrinth(String),
    Http(Url)
}

impl RawMods {
    fn resolve(self) -> McResult<HashMap<String, ManifestMod>> {
        let mut mods = HashMap::new();

        for (name, version) in self.modrinth.unwrap_or_default() {
            insert_mod(&mut mods, name, ManifestMod::Modrinth(version))?;
        }

        for (name, url) in self.http.unwrap_or_default() {
            insert_mod(&mut mods, name, ManifestMod::Http(url))?;
        }

        Ok(mods)
    }
}

fn insert_mod(
    mods: &mut HashMap<String, ManifestMod>,
    name: String,
    m: ManifestMod
) -> McResult<()> {
    if mods.contains_key(&name) {
        anyhow::bail!("the mod `{}` is listed under more than one source", name);
    }

    mods.insert(name, m);

    Ok(())
}

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

impl RawJava {
    fn resolve(self) -> McResult<ManifestJava> {
        let defaults = ManifestJava::default();

        Ok(ManifestJava {
            version: parse_or(self.version, defaults.version)?,
            min_memory: self.min_memory.unwrap_or(defaults.min_memory),
            max_memory: self.max_memory.unwrap_or(defaults.max_memory),
            jvm_arguments: self.jvm_arguments.unwrap_or(defaults.jvm_arguments)
        })
    }
}

pub struct ManifestTunnel {
    pub provider: RawProductDescriptor
}

impl ManifestTunnel {
    pub async fn provider_descriptor(&self, context: &McContext) -> McResult<TunnelDescriptor> {
        TunnelVersionResolver::resolve_descriptor(context, &self.provider).await
    }
}

impl Default for ManifestTunnel {
    fn default() -> Self {
        ManifestTunnel {
            provider: RawProductDescriptor {
                product: String::from("playit"),
                version: None
            }
        }
    }
}

impl RawTunnel {
    fn resolve(self) -> McResult<ManifestTunnel> {
        let defaults = ManifestTunnel::default();

        Ok(ManifestTunnel {
            provider: parse_or(self.provider, defaults.provider)?
        })
    }
}

#[derive(Default)]
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

impl RawMinecraft {
    fn resolve(self) -> McResult<ManifestMinecraft> {
        Ok(ManifestMinecraft {
            version: self.version,
            loader: self.loader.map(|loader| loader.parse()).transpose()?
        })
    }
}

#[derive(Clone)]
pub struct ManifestNotifications {
    pub on_lifecycle_event: bool,
    pub on_backup: bool,
    pub on_backup_failure: bool,
    pub on_panic: bool,
    pub on_sigkill: bool
}

impl Default for ManifestNotifications {
    fn default() -> Self {
        ManifestNotifications {
            on_lifecycle_event: true,
            on_backup: true,
            on_backup_failure: true,
            on_panic: true,
            on_sigkill: true
        }
    }
}

impl RawNotifications {
    fn resolve(self) -> ManifestNotifications {
        let defaults = ManifestNotifications::default();

        ManifestNotifications {
            on_lifecycle_event: self
                .on_lifecycle_event
                .unwrap_or(defaults.on_lifecycle_event),
            on_backup: self.on_backup.unwrap_or(defaults.on_backup),
            on_backup_failure: self.on_backup_failure.unwrap_or(defaults.on_backup_failure),
            on_panic: self.on_panic.unwrap_or(defaults.on_panic),
            on_sigkill: self.on_sigkill.unwrap_or(defaults.on_sigkill)
        }
    }
}

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
    pub properties: BTreeMap<String, String>
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
            properties: BTreeMap::new()
        }
    }
}

impl ManifestServer {
    pub fn property_overrides(&self) -> McResult<BTreeMap<String, String>> {
        Ok(self.properties.clone())
    }
}

impl RawServer {
    fn resolve(self) -> McResult<ManifestServer> {
        let defaults = ManifestServer::default();
        let mut properties = BTreeMap::new();

        if let Some(raw_properties) = self.properties {
            flatten_properties(None, raw_properties, &mut properties)?;
        }

        Ok(ManifestServer {
            gamemode: parse_or(self.gamemode, defaults.gamemode)?,
            difficulty: parse_or(self.difficulty, defaults.difficulty)?,
            level_type: parse_or(self.level_type, defaults.level_type)?,
            hardcore: self.hardcore.unwrap_or(defaults.hardcore),
            seed: self.seed.map(|seed| match seed {
                RawSeed::Numeric(seed) => MinecraftSeed::Numeric(seed),
                RawSeed::Text(seed) => MinecraftSeed::Text(seed)
            }),
            eula: self.eula.unwrap_or(defaults.eula),
            ip: self.ip,
            port: self.port.unwrap_or(defaults.port),
            rcon_port: self.rcon_port.unwrap_or(defaults.rcon_port),
            capacity: self.capacity.unwrap_or(defaults.capacity),
            view_distance: self.view_distance.unwrap_or(defaults.view_distance),
            simulation_distance: self
                .simulation_distance
                .unwrap_or(defaults.simulation_distance),
            properties
        })
    }
}

// Nested blocks flatten back into dot-joined server.properties keys, so
// `rcon { port 25575 }` and `"rcon.port" 25575` are the same override.
fn flatten_properties(
    prefix: Option<&str>,
    properties: BTreeMap<String, RawProperty>,
    output: &mut BTreeMap<String, String>
) -> McResult<()> {
    for (key, value) in properties {
        let key = match prefix {
            Some(prefix) => format!("{}.{}", prefix, key),
            None => key
        };

        let value = match value {
            RawProperty::Nested(inner) => {
                flatten_properties(Some(&key), inner, output)?;

                continue;
            }
            RawProperty::Bool(value) => value.to_string(),
            RawProperty::Integer(value) => value.to_string(),
            RawProperty::Float(value) => value.to_string(),
            RawProperty::Text(value) => value
        };

        if output.insert(key.clone(), value).is_some() {
            anyhow::bail!("the server property `{}` is set more than once", key);
        }
    }

    Ok(())
}

pub struct ManifestBackups {
    pub enabled: bool,
    pub frequency: String,
    pub storage: BackupStorage
}

impl ManifestBackups {
    /// The storage target to use. When storage is configured as S3,
    /// `MC_BACKUPS_S3_BUCKET` overrides the bucket without editing `mc.kdl`;
    /// other storage types ignore it.
    pub fn effective_storage(&self) -> BackupStorage {
        match &self.storage {
            BackupStorage::S3 { bucket, region } => BackupStorage::S3 {
                bucket: env::var("MC_BACKUPS_S3_BUCKET")
                    .ok()
                    .filter(|bucket| !bucket.is_empty())
                    .unwrap_or_else(|| bucket.clone()),
                region: region.clone()
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

impl RawBackups {
    fn resolve(self) -> McResult<ManifestBackups> {
        let defaults = ManifestBackups::default();
        let keep = self.keep.unwrap_or(20);

        let storage = match (self.local, self.s3) {
            (Some(_), Some(_)) => {
                anyhow::bail!("backups can use either `local` or `s3` storage, not both")
            }
            (Some(path), None) => BackupStorage::Local { path, keep },
            (None, Some(s3)) => BackupStorage::S3 {
                bucket: s3.bucket,
                region: s3.region
            },
            (None, None) => BackupStorage::Local {
                path: PathBuf::from("backups"),
                keep
            }
        };

        Ok(ManifestBackups {
            enabled: self.on,
            frequency: self.frequency.unwrap_or(defaults.frequency),
            storage
        })
    }
}

#[derive(Default)]
pub struct ManifestPlayers {
    pub allow: BTreeSet<String>,
    pub ban: BTreeMap<String, ManifestBan>,
    pub ban_ip: BTreeMap<IpAddr, ManifestBan>,
    pub op: BTreeMap<String, ManifestOp>
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestBan {
    pub reason: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub expires: Option<DateTime<Utc>>
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManifestOp {
    pub level: Option<MinecraftPermission>,
    pub bypasses_player_limit: bool
}

impl RawPlayers {
    fn resolve(self) -> McResult<ManifestPlayers> {
        let mut allow = BTreeSet::new();

        for name in self.allow.unwrap_or_default().into_keys() {
            players::validate_name(&name)?;
            allow.insert(name);
        }

        let mut ban = BTreeMap::new();

        for (name, entry) in self.ban.unwrap_or_default() {
            players::validate_name(&name)?;

            if allow.contains(&name) {
                anyhow::bail!("the player `{}` is both allowed and banned", name);
            }

            ban.insert(name, entry.resolve()?);
        }

        let mut ban_ip = BTreeMap::new();

        for (address, entry) in self.ban_ip.unwrap_or_default() {
            let address: IpAddr = address
                .parse()
                .with_context(|| format!("`{}` is not an ip address", address))?;

            ban_ip.insert(address, entry.resolve()?);
        }

        let mut op = BTreeMap::new();

        for (name, entry) in self.op.unwrap_or_default() {
            players::validate_name(&name)?;
            op.insert(name, entry.resolve()?);
        }

        Ok(ManifestPlayers {
            allow,
            ban,
            ban_ip,
            op
        })
    }
}

impl RawBanEntry {
    fn resolve(self) -> McResult<ManifestBan> {
        Ok(ManifestBan {
            reason: self.reason,
            created: parse_timestamp(self.created, "created")?,
            expires: parse_timestamp(self.expires, "expires")?
        })
    }
}

fn parse_timestamp(value: Option<String>, key: &str) -> McResult<Option<DateTime<Utc>>> {
    value
        .map(|value| {
            DateTime::parse_from_rfc3339(&value)
                .map(|timestamp| timestamp.with_timezone(&Utc))
                .with_context(|| {
                    format!(
                        "`{}` must be an RFC 3339 timestamp such as `2026-01-31T00:00:00Z`",
                        key
                    )
                })
        })
        .transpose()
}

impl RawOpEntry {
    fn resolve(self) -> McResult<ManifestOp> {
        let level = self
            .level
            .map(|level| {
                MinecraftPermission::try_from(level)
                    .ok()
                    .filter(|permission| *permission != MinecraftPermission::All)
                    .ok_or_else(|| anyhow::anyhow!("`level` must be between 1 and 4"))
            })
            .transpose()?;

        Ok(ManifestOp {
            level,
            bypasses_player_limit: self.bypasses_player_limit.unwrap_or(false)
        })
    }
}
