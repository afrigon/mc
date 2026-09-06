use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawManifest {
    pub name: String,
    pub description: String,

    #[serde(default)]
    pub java: RawJava,

    #[serde(default)]
    pub minecraft: RawMinecraft,

    #[serde(default)]
    pub server: RawServer,

    #[serde(default)]
    pub mods: RawMods,

    #[serde(default)]
    pub backups: RawBackups,

    #[serde(default)]
    pub notifications: RawNotifications,

    pub tunnel: Option<RawTunnel>,

    #[serde(default)]
    pub players: RawPlayers
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawJava {
    pub version: Option<String>,
    pub min_memory: Option<usize>,
    pub max_memory: Option<usize>,
    pub jvm_arguments: Option<Vec<String>>
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawMinecraft {
    pub version: Option<String>,
    pub loader: Option<String>
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawServer {
    pub gamemode: Option<String>,
    pub difficulty: Option<String>,
    pub level_type: Option<String>,
    pub hardcore: Option<bool>,
    pub allow_list: Option<bool>,
    pub online_mode: Option<bool>,
    pub hide_online_players: Option<bool>,
    pub seed: Option<RawSeed>,
    pub eula: Option<bool>,
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub rcon_port: Option<u16>,
    pub capacity: Option<i32>,
    pub view_distance: Option<u8>,
    pub simulation_distance: Option<u8>,
    pub properties: Option<BTreeMap<String, RawProperty>>
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum RawSeed {
    Numeric(i64),
    Text(String)
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum RawProperty {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Text(String),
    Nested(BTreeMap<String, RawProperty>)
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawMods {
    pub modrinth: Option<BTreeMap<String, String>>,
    pub http: Option<BTreeMap<String, Url>>
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawBackups {
    pub on: bool,
    pub frequency: Option<String>,
    pub keep: Option<usize>,
    pub local: Option<PathBuf>,
    pub s3: Option<RawS3>
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawS3 {
    #[serde(rename = "#0")]
    pub bucket: String,
    pub region: Option<String>
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawNotifications {
    pub on_lifecycle_event: Option<bool>,
    pub on_backup: Option<bool>,
    pub on_backup_failure: Option<bool>,
    pub on_panic: Option<bool>,
    pub on_sigkill: Option<bool>
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawTunnel {
    pub provider: Option<String>
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawPlayers {
    pub allow: Option<BTreeMap<String, RawAllowEntry>>,
    pub ban: Option<BTreeMap<String, RawBanEntry>>,
    pub ban_ip: Option<BTreeMap<String, RawBanEntry>>,
    pub op: Option<BTreeMap<String, RawOpEntry>>
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawAllowEntry {}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawBanEntry {
    pub reason: Option<String>,
    pub created: Option<String>,
    pub expires: Option<String>
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawOpEntry {
    pub level: Option<u8>,
    pub bypasses_player_limit: Option<bool>
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawLockfile {
    #[serde(default)]
    pub modrinth: BTreeMap<String, RawLockEntry>,

    #[serde(default)]
    pub http: BTreeMap<String, RawLockEntry>,

    #[serde(default)]
    pub players: BTreeMap<String, RawPlayerLockEntry>
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawPlayerLockEntry {
    pub uuid: Option<String>
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RawLockEntry {
    pub version: Option<String>,
    pub url: Option<Url>,
    pub hash: Option<String>
}
