use anyhow::Context;
use serde::Serialize;
use tracing::field::debug;
use url::Url;

use crate::exit_with_error;
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

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ServerProperties {
    accepts_transfers: bool,
    allow_flight: bool,
    broadcast_console_to_ops: bool,
    broadcast_rcon_to_ops: bool,
    bug_report_link: Option<Url>,
    difficulty: MinecraftDifficulty,
    enable_code_of_conduct: bool,
    enable_jmx_monitoring: bool,
    enable_query: bool,
    enable_rcon: bool,
    enable_status: bool,
    enforce_secure_profile: bool,
    enforce_whitelist: bool,
    entity_broadcast_range_percentage: usize,
    force_gamemode: bool,
    function_permission_level: MinecraftPermission,
    gamemode: MinecraftGamemode,
    generate_structures: bool,
    generator_settings: String,
    hardcore: bool,
    hide_online_players: bool,
    initial_disabled_packs: SeparatedList<String, ','>,
    initial_enabled_packs: SeparatedList<String, ','>,
    level_name: String,
    level_seed: Option<MinecraftSeed>,
    level_type: MinecraftLevelKind,
    log_ips: bool,
    management_server_allowed_origins: SeparatedList<String, ','>,
    management_server_enabled: bool,
    management_server_host: String,
    management_server_port: u16,
    management_server_secret: Option<String>,
    management_server_tls_enabled: bool,
    management_server_tls_keystore: Option<String>,
    management_server_tls_keystore_password: Option<String>,
    max_chained_neighbor_updates: usize,
    max_players: i32,
    max_tick_time: usize,
    max_world_size: usize,
    motd: String,
    network_compression_threshold: usize,
    online_mode: bool,
    op_permission_level: MinecraftPermission,
    pause_when_empty_seconds: usize,
    player_idle_timeout: usize,
    prevent_proxy_connections: bool,

    #[serde(rename = "query.port")]
    query_port: u16,

    rate_limit: usize,

    #[serde(rename = "rcon.password")]
    rcon_password: Option<String>,

    #[serde(rename = "rcon.port")]
    rcon_port: u16,

    region_file_compression: Option<MinecraftRegionCompression>,
    require_resource_pack: bool,
    resource_pack: Option<String>,
    resource_pack_id: Option<String>,
    resource_pack_prompt: Option<String>,
    resource_pack_sha1: Option<String>,
    server_ip: Option<String>,
    server_port: u16,
    simulation_distance: u8,
    spawn_protection: usize,
    status_heartbeat_interval: usize,
    sync_chunk_writes: bool,
    use_native_transport: bool,
    view_distance: u8,
    white_list: bool
}

impl Default for ServerProperties {
    fn default() -> Self {
        ServerProperties {
            accepts_transfers: false,
            allow_flight: false,
            broadcast_console_to_ops: true,
            broadcast_rcon_to_ops: true,
            bug_report_link: None,
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
            server_ip: None,
            server_port: 25565,
            simulation_distance: 16,
            spawn_protection: 0,
            status_heartbeat_interval: 0,
            sync_chunk_writes: true,
            use_native_transport: true,
            view_distance: 16,
            white_list: false
        }
    }
}

impl ServerProperties {
    pub fn apply(&mut self, manifest: &Manifest) {
        self.level_name = manifest.name.clone();
        self.motd = manifest.description.clone();
        self.enable_rcon = manifest.backups.enabled;
        self.rcon_port = manifest.server.rcon_port;
        self.server_port = manifest.server.port;
        self.server_ip = manifest.server.ip.clone();
        self.gamemode = manifest.server.gamemode;
        self.difficulty = manifest.server.difficulty;
        self.hardcore = manifest.server.hardcore;
        self.max_players = manifest.server.capacity.max(0);
        self.level_type = manifest.server.level_type;
        self.level_seed = manifest.server.seed.clone();
        self.view_distance = manifest.server.view_distance;
        self.simulation_distance = manifest.server.simulation_distance;
    }

    pub fn to_string(&self) -> McResult<String> {
        let s = serde_java_properties::to_string(self)
            .context("could not serialize server.properties")?;

        let title = format!(
            "Minecraft server properties, Generated with {} {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        let date_string = utils::date::minecraft_date_string()?;

        Ok(format!("#{}\n#{}\n{}", title, date_string, s))
    }
}
