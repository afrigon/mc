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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_uuid_matches_the_server() {
        // UUID.nameUUIDFromBytes("OfflinePlayer:Notch".getBytes(UTF_8))
        assert_eq!(
            offline_uuid("Notch").to_string(),
            "b50ad385-829d-3141-a216-7e7d7539ba7f"
        );
    }

    #[test]
    fn ban_entries_fill_the_server_defaults() -> McResult<()> {
        let now = DateTime::parse_from_rfc3339("2026-09-06T14:00:00Z")?.with_timezone(&Utc);
        let details = BanDetails {
            reason: None,
            created: None,
            expires: None
        };
        let entry = BanEntry::new(Uuid::nil(), String::from("Griefer"), &details, now);

        assert_eq!(entry.created, "2026-09-06 14:00:00 +0000");
        assert_eq!(entry.expires, "forever");
        assert_eq!(entry.reason, "Banned by an operator.");
        assert_eq!(entry.source, "mc");

        Ok(())
    }

    #[test]
    fn op_entries_use_the_server_field_names() -> McResult<()> {
        let entry = OpEntry {
            uuid: Uuid::nil(),
            name: String::from("Notch"),
            level: 4,
            bypasses_player_limit: true
        };

        assert_eq!(
            serde_json::to_string(&entry)?,
            r#"{"uuid":"00000000-0000-0000-0000-000000000000","name":"Notch","level":4,"bypassesPlayerLimit":true}"#
        );

        Ok(())
    }
}
