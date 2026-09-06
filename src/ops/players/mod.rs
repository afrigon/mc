pub mod allow;
pub mod ban;
pub mod op;

use std::path::PathBuf;

use chrono::Utc;

use crate::context::McContext;
use crate::manifest::ManifestPaths;
use crate::manifest::ManifestPlayers;
use crate::manifest::lock::Lockfile;
use crate::minecraft::MinecraftPermission;
use crate::minecraft::players;
use crate::minecraft::players::AllowEntry;
use crate::minecraft::players::BanDetails;
use crate::minecraft::players::BanEntry;
use crate::minecraft::players::IpBanEntry;
use crate::minecraft::players::OpEntry;
use crate::resolvers;
use crate::utils::errors::McResult;

/// Names are unique regardless of case, so a lookup matches whatever casing
/// the manifest holds.
pub(super) fn find_name<'a>(
    names: impl IntoIterator<Item = &'a String>,
    name: &str
) -> Option<&'a String> {
    names
        .into_iter()
        .find(|candidate| candidate.eq_ignore_ascii_case(name))
}

pub struct PlayerListOptions {
    pub paths: ManifestPaths
}

pub struct ApplyPlayersOptions {
    pub instance_path: PathBuf,
    pub lockfile_path: PathBuf,
    pub online_mode: bool,
    pub server_level: MinecraftPermission
}

/// Writes the server's player lists from the manifest, resolving any name the
/// lockfile does not know yet.
pub async fn apply(
    context: &mut McContext,
    options: &ApplyPlayersOptions,
    players: &ManifestPlayers
) -> McResult<()> {
    let mut lockfile = Lockfile::read(&options.lockfile_path).await?;
    let now = Utc::now();

    let mut allow = Vec::new();

    for name in &players.allow {
        let player =
            resolvers::players::resolve(context, &mut lockfile, name, options.online_mode).await?;

        allow.push(AllowEntry {
            uuid: player.uuid,
            name: player.name
        });
    }

    let mut ban = Vec::new();

    for (name, entry) in &players.ban {
        let player =
            resolvers::players::resolve(context, &mut lockfile, name, options.online_mode).await?;
        let details = BanDetails {
            reason: entry.reason.clone(),
            created: entry.created,
            expires: entry.expires
        };

        ban.push(BanEntry::new(player.uuid, player.name, &details, now));
    }

    let ban_ip: Vec<IpBanEntry> = players
        .ban_ip
        .iter()
        .map(|(address, entry)| {
            let details = BanDetails {
                reason: entry.reason.clone(),
                created: entry.created,
                expires: entry.expires
            };

            IpBanEntry::new(*address, &details, now)
        })
        .collect();

    let mut op = Vec::new();

    for (name, entry) in &players.op {
        let player =
            resolvers::players::resolve(context, &mut lockfile, name, options.online_mode).await?;

        op.push(OpEntry {
            uuid: player.uuid,
            name: player.name,
            level: entry.level.unwrap_or(options.server_level) as u8,
            bypasses_player_limit: entry.bypasses_player_limit
        });
    }

    if lockfile.changed() {
        lockfile.write(&options.lockfile_path).await?;
    }

    let instance_path = &options.instance_path;

    players::write_list(instance_path, players::ALLOW_FILE, &allow).await?;
    players::write_list(instance_path, players::BAN_FILE, &ban).await?;
    players::write_list(instance_path, players::BAN_IP_FILE, &ban_ip).await?;
    players::write_list(instance_path, players::OP_FILE, &op).await?;

    _ = context.shell().status(
        "Players",
        format!(
            "{} allowed, {} banned, {} operators",
            allow.len(),
            ban.len() + ban_ip.len(),
            op.len()
        )
    );

    Ok(())
}
