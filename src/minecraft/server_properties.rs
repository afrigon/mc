use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::Context;
use serde::Serialize;
use url::Url;

use crate::manifest::Manifest;
use crate::minecraft::MinecraftDifficulty;
use crate::minecraft::MinecraftGamemode;
use crate::minecraft::MinecraftLevelKind;
use crate::minecraft::MinecraftPermission;
use crate::minecraft::MinecraftRegionCompression;
use crate::minecraft::seed::MinecraftSeed;
use crate::utils;
use crate::utils::csv::SeparatedList;
use crate::utils::errors::McResult;

// Mirrors the vanilla server.properties keys and defaults as of Minecraft Java
// Edition 26.3, per https://minecraft.wiki/w/Server.properties. To sync with a
// later Minecraft version: read that page's History section for every key
// added, removed, or defaulted differently after 26.3, mirror each change in
// both the struct fields (alphabetical by serialized kebab-case name) and the
// `Default` impl below, then bump the version in this comment.
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ServerProperties {
    pub accepts_transfers: bool,
    pub allow_flight: bool,
    pub broadcast_console_to_ops: bool,
    pub broadcast_rcon_to_ops: bool,
    pub bug_report_link: Option<Url>,
    pub chat_spam_threshold_seconds: usize,
    pub command_spam_threshold_seconds: usize,
    pub difficulty: MinecraftDifficulty,
    pub enable_code_of_conduct: bool,
    pub enable_jmx_monitoring: bool,
    pub enable_query: bool,
    pub enable_rcon: bool,
    pub enable_status: bool,
    pub enforce_secure_profile: bool,
    pub enforce_whitelist: bool,
    pub entity_broadcast_range_percentage: usize,
    pub force_gamemode: bool,
    pub function_permission_level: MinecraftPermission,
    pub gamemode: MinecraftGamemode,
    pub generate_structures: bool,
    pub generator_settings: String,
    pub hardcore: bool,
    pub hide_online_players: bool,
    pub initial_disabled_packs: SeparatedList<String, ','>,
    pub initial_enabled_packs: SeparatedList<String, ','>,
    pub level_name: String,
    pub level_seed: Option<MinecraftSeed>,
    pub level_type: MinecraftLevelKind,
    pub log_ips: bool,
    pub management_server_allowed_origins: SeparatedList<String, ','>,
    pub management_server_enabled: bool,
    pub management_server_host: String,
    pub management_server_port: u16,
    pub management_server_secret: Option<String>,
    pub management_server_tls_enabled: bool,
    pub management_server_tls_keystore: Option<String>,
    pub management_server_tls_keystore_password: Option<String>,
    pub max_chained_neighbor_updates: usize,
    pub max_players: i32,
    pub max_tick_time: usize,
    pub max_world_size: usize,
    pub motd: String,
    pub network_compression_threshold: usize,
    pub online_mode: bool,
    pub op_permission_level: MinecraftPermission,
    pub pause_when_empty_seconds: usize,
    pub player_idle_timeout: usize,
    pub prevent_proxy_connections: bool,

    #[serde(rename = "query.port")]
    pub query_port: u16,

    pub rate_limit: usize,

    #[serde(rename = "rcon.password")]
    pub rcon_password: Option<String>,

    #[serde(rename = "rcon.port")]
    pub rcon_port: u16,

    pub region_file_compression: Option<MinecraftRegionCompression>,
    pub require_resource_pack: bool,
    pub resource_pack: Option<String>,
    pub resource_pack_id: Option<String>,
    pub resource_pack_prompt: Option<String>,
    pub resource_pack_sha1: Option<String>,
    pub server_ip: Option<String>,
    pub server_port: u16,
    pub simulation_distance: u8,
    pub spawn_protection: usize,
    pub status_heartbeat_interval: usize,
    pub sync_chunk_writes: bool,
    pub use_native_transport: bool,
    pub view_distance: u8,
    pub white_list: bool
}

impl Default for ServerProperties {
    fn default() -> Self {
        ServerProperties {
            accepts_transfers: false,
            allow_flight: false,
            broadcast_console_to_ops: true,
            broadcast_rcon_to_ops: true,
            bug_report_link: None,
            chat_spam_threshold_seconds: 10,
            command_spam_threshold_seconds: 10,
            difficulty: MinecraftDifficulty::Normal,
            enable_code_of_conduct: false,
            enable_jmx_monitoring: false,
            enable_query: false,
            enable_rcon: false,
            enable_status: true,
            enforce_secure_profile: true,
            enforce_whitelist: true,
            entity_broadcast_range_percentage: 100,
            force_gamemode: false,
            function_permission_level: MinecraftPermission::Gamemaster,
            gamemode: MinecraftGamemode::Survival,
            generate_structures: true,
            generator_settings: String::from("{}"),
            hardcore: false,
            hide_online_players: false,
            initial_disabled_packs: vec![].into(),
            initial_enabled_packs: vec![String::from("vanilla")].into(),
            level_name: String::from("world"),
            level_seed: None,
            level_type: MinecraftLevelKind::Normal,
            log_ips: true,
            management_server_allowed_origins: vec![].into(),
            management_server_enabled: false,
            management_server_host: String::from("localhost"),
            management_server_port: 0,
            management_server_secret: None,
            management_server_tls_enabled: true,
            management_server_tls_keystore: None,
            management_server_tls_keystore_password: None,
            max_chained_neighbor_updates: 1000000,
            max_players: 20,
            max_tick_time: 60000,
            max_world_size: 29999984,
            motd: String::from("A Minecraft Server"),
            network_compression_threshold: 256,
            online_mode: true,
            op_permission_level: MinecraftPermission::Owner,
            pause_when_empty_seconds: 60,
            player_idle_timeout: 0,
            prevent_proxy_connections: false,
            query_port: 25565,
            rate_limit: 0,
            rcon_password: None,
            rcon_port: 25575,
            region_file_compression: Some(MinecraftRegionCompression::deflate),
            require_resource_pack: false,
            resource_pack: None,
            resource_pack_id: None,
            resource_pack_prompt: None,
            resource_pack_sha1: None,
            // Bind the IPv6 wildcard so the server is reachable over IPv6; on a
            // dual-stack host this also accepts IPv4-mapped connections.
            server_ip: Some(String::from("::")),
            server_port: 25565,
            simulation_distance: 16,
            spawn_protection: 0,
            status_heartbeat_interval: 0,
            sync_chunk_writes: true,
            use_native_transport: true,
            view_distance: 16,
            white_list: true
        }
    }
}

// The subset of server.properties driven by `mc.toml` and the environment.
// Serializing it yields the managed keys.
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ManagedServerProperties {
    pub difficulty: MinecraftDifficulty,
    pub enable_rcon: bool,
    pub gamemode: MinecraftGamemode,
    pub hardcore: bool,
    pub level_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub level_seed: Option<MinecraftSeed>,

    pub level_type: MinecraftLevelKind,
    pub max_players: i32,
    pub motd: String,

    #[serde(rename = "rcon.password", skip_serializing_if = "Option::is_none")]
    pub rcon_password: Option<String>,

    #[serde(rename = "rcon.port")]
    pub rcon_port: u16,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_ip: Option<String>,

    pub server_port: u16,
    pub simulation_distance: u8,
    pub view_distance: u8
}

impl ManagedServerProperties {
    pub fn from_manifest(
        manifest: &Manifest,
        rcon_password: Option<String>
    ) -> ManagedServerProperties {
        ManagedServerProperties {
            difficulty: manifest.server.difficulty,
            enable_rcon: manifest.backups.enabled,
            gamemode: manifest.server.gamemode,
            hardcore: manifest.server.hardcore,
            level_name: manifest.name.clone(),
            level_seed: manifest.server.seed.clone(),
            level_type: manifest.server.level_type,
            max_players: manifest.server.capacity.max(0),
            motd: manifest.description.clone(),
            rcon_password,
            rcon_port: manifest.server.rcon_port,
            server_ip: manifest.server.ip.clone(),
            server_port: manifest.server.port,
            simulation_distance: manifest.server.simulation_distance,
            view_distance: manifest.server.view_distance
        }
    }

    pub fn to_map(&self) -> McResult<BTreeMap<String, String>> {
        let s = serde_java_properties::to_string(self)
            .context("could not serialize the managed server properties")?;

        serde_java_properties::from_str(&s).context("could not parse the managed server properties")
    }
}

impl ServerProperties {
    /// Reads the configured rcon password from an instance's `server.properties`,
    /// the source of truth for the running server's credentials. Returns `None`
    /// if the file or the key is absent.
    pub async fn read_rcon_password(instance_path: &Path) -> McResult<Option<String>> {
        let path = instance_path.join("server.properties");

        let content = match tokio::fs::read_to_string(&path).await {
            Ok(content) => content,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error).context("could not read server.properties")
        };

        let password = content
            .lines()
            .find_map(|line| line.strip_prefix("rcon.password="))
            .map(str::to_string);

        Ok(password)
    }

    pub fn to_string(
        &self,
        overrides: &BTreeMap<String, String>,
        managed: &BTreeMap<String, String>
    ) -> McResult<String> {
        let mut s = serde_java_properties::to_string(self)
            .context("could not serialize server.properties")?;

        if !overrides.is_empty() || !managed.is_empty() {
            let mut entries: BTreeMap<String, String> = serde_java_properties::from_str(&s)
                .context("could not parse the generated server.properties")?;

            entries.extend(overrides.clone());
            entries.extend(managed.clone());

            s = serde_java_properties::to_string(&entries)
                .context("could not serialize server.properties")?;
        }

        let title = format!(
            "Minecraft server properties, Generated with {} {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        let date_string = utils::date::minecraft_date_string()?;

        Ok(format!("#{}\n#{}\n{}", title, date_string, s))
    }
}
