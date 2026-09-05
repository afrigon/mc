pub mod document;
pub mod lock;
pub mod presets;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::env;

use anyhow::Context;
use kdl::KdlDocument;
use kdl::KdlNode;
use kdl::KdlValue;
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
use crate::utils;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::ProductDescriptor;
use crate::utils::product_descriptor::RawProductDescriptor;
use crate::utils::product_descriptor::VersionResolver;

pub trait FromKdlNode: Sized {
    fn from_kdl_node(node: &KdlNode) -> McResult<Self>;
}

pub struct Manifest {
    pub name: String,
    pub description: String,
    pub java: ManifestJava,
    pub minecraft: ManifestMinecraft,
    pub server: ManifestServer,
    pub mods: HashMap<String, ManifestMod>,
    pub backups: ManifestBackups,
    pub notifications: ManifestNotifications
}

impl Manifest {
    pub fn from_kdl_str(source: &str) -> McResult<Manifest> {
        let document = utils::kdl::parse_document(source)?;

        utils::kdl::check_children(
            &document,
            "the manifest",
            &[
                "name",
                "description",
                "java",
                "minecraft",
                "server",
                "mods",
                "backups",
                "notifications"
            ]
        )?;

        Ok(Manifest {
            name: utils::kdl::string_argument(utils::kdl::required_child(&document, "name")?)?
                .to_owned(),
            description: utils::kdl::string_argument(utils::kdl::required_child(
                &document,
                "description"
            )?)?
            .to_owned(),
            java: section(&document, "java")?,
            minecraft: section(&document, "minecraft")?,
            server: section(&document, "server")?,
            mods: mods_section(&document)?,
            backups: section(&document, "backups")?,
            notifications: section(&document, "notifications")?
        })
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

fn section<T: FromKdlNode + Default>(document: &KdlDocument, name: &str) -> McResult<T> {
    document
        .get(name)
        .map(T::from_kdl_node)
        .transpose()
        .with_context(|| format!("invalid `{}` section", name))
        .map(Option::unwrap_or_default)
}

fn mods_section(document: &KdlDocument) -> McResult<HashMap<String, ManifestMod>> {
    let mut mods = HashMap::new();

    let Some(node) = document.get("mods") else {
        return Ok(mods);
    };

    utils::kdl::check_properties(node, &[])?;

    if !utils::kdl::arguments(node).is_empty() {
        anyhow::bail!("the `mods` node does not take values, list mods as children");
    }

    for entry in node.iter_children() {
        let name = entry.name().value();

        if mods.contains_key(name) {
            anyhow::bail!("the mod `{}` is listed more than once", name);
        }

        let m = ManifestMod::from_kdl_node(entry)
            .with_context(|| format!("invalid mod entry `{}`", name))?;

        mods.insert(name.to_owned(), m);
    }

    Ok(mods)
}

/// Children of a section node, once the section itself has been checked to
/// carry nothing but children.
fn children<'a>(node: &'a KdlNode, allowed: &[&str]) -> McResult<Option<&'a KdlDocument>> {
    let name = node.name().value();

    utils::kdl::check_properties(node, &[])?;

    if !utils::kdl::arguments(node).is_empty() {
        anyhow::bail!(
            "the `{}` node does not take values, use a block instead",
            name
        );
    }

    match node.children() {
        Some(children) => {
            utils::kdl::check_children(children, &format!("the `{}` section", name), allowed)?;

            Ok(Some(children))
        }
        None => Ok(None)
    }
}

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

impl FromKdlNode for ManifestMod {
    fn from_kdl_node(node: &KdlNode) -> McResult<Self> {
        utils::kdl::check_properties(node, &["service", "url"])?;

        if node.children().is_some() {
            anyhow::bail!("a mod entry does not take children");
        }

        let url = utils::kdl::string_property(node, "url")?;
        let service = utils::kdl::string_property(node, "service")?;

        match (utils::kdl::arguments(node).as_slice(), url) {
            ([], Some(url)) => {
                if service.is_some() {
                    anyhow::bail!("a mod fetched from a url cannot name a service");
                }

                Ok(ManifestMod::Remote {
                    url: Url::parse(url).context("invalid mod url")?
                })
            }
            ([version], None) => {
                let version = version
                    .as_string()
                    .ok_or_else(|| anyhow::anyhow!("the mod version must be a string"))?
                    .to_owned();

                match service {
                    Some(service) => Ok(ManifestMod::Detailed {
                        version,
                        service: service.parse()?
                    }),
                    None => Ok(ManifestMod::Version(version))
                }
            }
            ([_], Some(_)) => anyhow::bail!("a mod takes either a version or a url, not both"),
            ([], None) => anyhow::bail!("a mod requires a version or a url"),
            _ => anyhow::bail!("a mod takes a single version")
        }
    }
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

impl FromKdlNode for ManifestJava {
    fn from_kdl_node(node: &KdlNode) -> McResult<Self> {
        let mut java = ManifestJava::default();

        let Some(children) = children(
            node,
            &["version", "min-memory", "max-memory", "jvm-arguments"]
        )?
        else {
            return Ok(java);
        };

        if let Some(version) = children.get("version") {
            java.version = utils::kdl::parse_argument(version)?;
        }

        if let Some(min_memory) = children.get("min-memory") {
            java.min_memory = utils::kdl::integer_argument(min_memory)?;
        }

        if let Some(max_memory) = children.get("max-memory") {
            java.max_memory = utils::kdl::integer_argument(max_memory)?;
        }

        if let Some(jvm_arguments) = children.get("jvm-arguments") {
            java.jvm_arguments = utils::kdl::string_arguments(jvm_arguments)?;
        }

        Ok(java)
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

impl FromKdlNode for ManifestMinecraft {
    fn from_kdl_node(node: &KdlNode) -> McResult<Self> {
        let mut minecraft = ManifestMinecraft::default();

        let Some(children) = children(node, &["version", "loader"])? else {
            return Ok(minecraft);
        };

        if let Some(version) = children.get("version") {
            minecraft.version = Some(utils::kdl::string_argument(version)?.to_owned());
        }

        if let Some(loader) = children.get("loader") {
            minecraft.loader = Some(utils::kdl::parse_argument(loader)?);
        }

        Ok(minecraft)
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

impl FromKdlNode for ManifestNotifications {
    fn from_kdl_node(node: &KdlNode) -> McResult<Self> {
        let mut notifications = ManifestNotifications::default();

        let Some(children) = children(
            node,
            &[
                "on-lifecycle-event",
                "on-backup",
                "on-backup-failure",
                "on-panic",
                "on-sigkill"
            ]
        )?
        else {
            return Ok(notifications);
        };

        if let Some(value) = children.get("on-lifecycle-event") {
            notifications.on_lifecycle_event = utils::kdl::bool_argument(value)?;
        }

        if let Some(value) = children.get("on-backup") {
            notifications.on_backup = utils::kdl::bool_argument(value)?;
        }

        if let Some(value) = children.get("on-backup-failure") {
            notifications.on_backup_failure = utils::kdl::bool_argument(value)?;
        }

        if let Some(value) = children.get("on-panic") {
            notifications.on_panic = utils::kdl::bool_argument(value)?;
        }

        if let Some(value) = children.get("on-sigkill") {
            notifications.on_sigkill = utils::kdl::bool_argument(value)?;
        }

        Ok(notifications)
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

impl FromKdlNode for ManifestServer {
    fn from_kdl_node(node: &KdlNode) -> McResult<Self> {
        let mut server = ManifestServer::default();

        let Some(children) = children(
            node,
            &[
                "gamemode",
                "difficulty",
                "level-type",
                "hardcore",
                "seed",
                "eula",
                "ip",
                "port",
                "rcon-port",
                "capacity",
                "view-distance",
                "simulation-distance",
                "properties"
            ]
        )?
        else {
            return Ok(server);
        };

        if let Some(value) = children.get("gamemode") {
            server.gamemode = utils::kdl::parse_argument(value)?;
        }

        if let Some(value) = children.get("difficulty") {
            server.difficulty = utils::kdl::parse_argument(value)?;
        }

        if let Some(value) = children.get("level-type") {
            server.level_type = utils::kdl::parse_argument(value)?;
        }

        if let Some(value) = children.get("hardcore") {
            server.hardcore = utils::kdl::bool_argument(value)?;
        }

        if let Some(value) = children.get("seed") {
            server.seed = Some(seed(value)?);
        }

        if let Some(value) = children.get("eula") {
            server.eula = utils::kdl::bool_argument(value)?;
        }

        if let Some(value) = children.get("ip") {
            server.ip = Some(utils::kdl::string_argument(value)?.to_owned());
        }

        if let Some(value) = children.get("port") {
            server.port = utils::kdl::integer_argument(value)?;
        }

        if let Some(value) = children.get("rcon-port") {
            server.rcon_port = utils::kdl::integer_argument(value)?;
        }

        if let Some(value) = children.get("capacity") {
            server.capacity = utils::kdl::integer_argument(value)?;
        }

        if let Some(value) = children.get("view-distance") {
            server.view_distance = utils::kdl::integer_argument(value)?;
        }

        if let Some(value) = children.get("simulation-distance") {
            server.simulation_distance = utils::kdl::integer_argument(value)?;
        }

        if let Some(properties) = children.get("properties") {
            utils::kdl::check_properties(properties, &[])?;

            if !utils::kdl::arguments(properties).is_empty() {
                anyhow::bail!("the `properties` node does not take values, use a block instead");
            }

            if let Some(entries) = properties.children() {
                flatten_properties(None, entries, &mut server.properties)?;
            }
        }

        Ok(server)
    }
}

fn seed(node: &KdlNode) -> McResult<MinecraftSeed> {
    match utils::kdl::argument(node)? {
        KdlValue::Integer(integer) => Ok(MinecraftSeed::Numeric(
            i64::try_from(*integer).context("the seed is out of range")?
        )),
        KdlValue::String(text) => Ok(MinecraftSeed::Text(text.clone())),
        _ => anyhow::bail!("the seed must be an integer or a string")
    }
}

// Nested blocks flatten back into dot-joined server.properties keys, so
// `rcon { port 25575 }` and `"rcon.port" 25575` are the same override.
fn flatten_properties(
    prefix: Option<&str>,
    document: &KdlDocument,
    output: &mut BTreeMap<String, String>
) -> McResult<()> {
    for node in document.nodes() {
        let key = match prefix {
            Some(prefix) => format!("{}.{}", prefix, node.name().value()),
            None => node.name().value().to_owned()
        };

        utils::kdl::check_properties(node, &[])?;

        match (utils::kdl::arguments(node).as_slice(), node.children()) {
            ([], Some(children)) => flatten_properties(Some(&key), children, output)?,
            ([value], None) => {
                let value = utils::kdl::scalar_to_string(value).ok_or_else(|| {
                    anyhow::anyhow!(
                        "the server property `{}` must be a string, integer, float, or boolean",
                        key
                    )
                })?;

                if output.insert(key.clone(), value).is_some() {
                    anyhow::bail!("the server property `{}` is set more than once", key);
                }
            }
            _ => anyhow::bail!(
                "the server property `{}` must have either a single value or a block",
                key
            )
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

impl FromKdlNode for ManifestBackups {
    fn from_kdl_node(node: &KdlNode) -> McResult<Self> {
        let mut backups = ManifestBackups::default();

        let Some(children) = children(node, &["enabled", "frequency", "storage"])? else {
            return Ok(backups);
        };

        if let Some(enabled) = children.get("enabled") {
            backups.enabled = utils::kdl::bool_argument(enabled)?;
        }

        if let Some(frequency) = children.get("frequency") {
            backups.frequency = utils::kdl::string_argument(frequency)?.to_owned();
        }

        if let Some(storage) = children.get("storage") {
            backups.storage = BackupStorage::from_kdl_node(storage)?;
        }

        Ok(backups)
    }
}
