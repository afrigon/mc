use std::net::IpAddr;
use std::path::Path;

use anyhow::Context;
use chrono::DateTime;
use chrono::Utc;
use md5::Digest;
use md5::Md5;
use serde::Serialize;
use uuid::Uuid;

use crate::utils::errors::McResult;

pub const ALLOW_FILE: &str = "whitelist.json";
pub const BAN_FILE: &str = "banned-players.json";
pub const BAN_IP_FILE: &str = "banned-ips.json";
pub const OP_FILE: &str = "ops.json";

const BAN_SOURCE: &str = "mc";
const DEFAULT_BAN_REASON: &str = "Banned by an operator.";
const NEVER_EXPIRES: &str = "forever";
const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

/// Accepts every name the server does: legacy accounts predate the current
/// rules, so only what cannot be a name at all is rejected.
pub fn validate_name(name: &str) -> McResult<()> {
    if name.is_empty() {
        anyhow::bail!("a player name cannot be empty");
    }

    if name.chars().any(|c| c.is_whitespace() || c.is_control()) {
        anyhow::bail!(
            "`{}` is not a player name; names contain no spaces or control characters",
            name
        );
    }

    Ok(())
}

/// The identity an offline-mode server gives a player: a version 3 UUID
/// over `OfflinePlayer:<name>` with no namespace, as `UUID.nameUUIDFromBytes`
/// computes it.
pub fn offline_uuid(name: &str) -> Uuid {
    let mut bytes: [u8; 16] = Md5::digest(format!("OfflinePlayer:{}", name)).into();

    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    Uuid::from_bytes(bytes)
}

pub fn format_date(date: DateTime<Utc>) -> String {
    date.format(DATE_FORMAT).to_string()
}

#[derive(Serialize)]
pub struct AllowEntry {
    pub uuid: Uuid,
    pub name: String
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpEntry {
    pub uuid: Uuid,
    pub name: String,
    pub level: u8,
    pub bypasses_player_limit: bool
}

#[derive(Serialize)]
pub struct BanEntry {
    pub uuid: Uuid,
    pub name: String,
    pub created: String,
    pub source: String,
    pub expires: String,
    pub reason: String
}

#[derive(Serialize)]
pub struct IpBanEntry {
    pub ip: IpAddr,
    pub created: String,
    pub source: String,
    pub expires: String,
    pub reason: String
}

pub struct BanDetails {
    pub reason: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub expires: Option<DateTime<Utc>>
}

impl BanDetails {
    fn created(&self, now: DateTime<Utc>) -> String {
        format_date(self.created.unwrap_or(now))
    }

    fn expires(&self) -> String {
        self.expires
            .map(format_date)
            .unwrap_or_else(|| String::from(NEVER_EXPIRES))
    }

    fn reason(&self) -> String {
        self.reason
            .clone()
            .unwrap_or_else(|| String::from(DEFAULT_BAN_REASON))
    }
}

impl BanEntry {
    pub fn new(uuid: Uuid, name: String, details: &BanDetails, now: DateTime<Utc>) -> BanEntry {
        BanEntry {
            uuid,
            name,
            created: details.created(now),
            source: String::from(BAN_SOURCE),
            expires: details.expires(),
            reason: details.reason()
        }
    }
}

impl IpBanEntry {
    pub fn new(ip: IpAddr, details: &BanDetails, now: DateTime<Utc>) -> IpBanEntry {
        IpBanEntry {
            ip,
            created: details.created(now),
            source: String::from(BAN_SOURCE),
            expires: details.expires(),
            reason: details.reason()
        }
    }
}

pub async fn write_list<T: Serialize>(
    instance_path: &Path,
    file: &str,
    entries: &[T]
) -> McResult<()> {
    let path = instance_path.join(file);
    let content = serde_json::to_string_pretty(entries)
        .with_context(|| format!("could not serialize {}", file))?;

    tokio::fs::write(&path, content)
        .await
        .with_context(|| format!("could not write {}", path.display()))
}
