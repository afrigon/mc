pub mod allow;
pub mod ban;
pub mod op;

use std::path::PathBuf;

use anyhow::Context;
use chrono::Utc;
use kdl::KdlDocument;

use crate::context::McContext;
use crate::manifest::Manifest;
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
use crate::utils;
use crate::utils::errors::McResult;

pub struct PlayerPaths {
    pub manifest_path: PathBuf,
    pub lockfile_path: PathBuf
}

pub(super) struct Workspace {
    pub(super) manifest: Manifest,
    pub(super) document: KdlDocument,
    pub(super) lockfile: Lockfile
}

pub(super) async fn load(paths: &PlayerPaths) -> McResult<Workspace> {
    let manifest_string = tokio::fs::read_to_string(&paths.manifest_path)
        .await
        .context("could not find mc.kdl file")?;
    let manifest = Manifest::from_kdl_str(&manifest_string)?;
    let document = utils::kdl::parse_document(&manifest_string)?;
    let lockfile = read_lockfile(&paths.lockfile_path).await?;

    Ok(Workspace {
        manifest,
        document,
        lockfile
    })
}

pub(super) async fn save(paths: &PlayerPaths, workspace: &Workspace) -> McResult<()> {
    tokio::fs::write(&paths.manifest_path, workspace.document.to_string()).await?;
    write_lockfile(&paths.lockfile_path, &workspace.lockfile).await
}

pub(super) async fn read_lockfile(path: &PathBuf) -> McResult<Lockfile> {
    match tokio::fs::read_to_string(path).await {
        Ok(source) => Lockfile::from_kdl_str(&source).context("could not parse mc.lock"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Lockfile::default()),
        Err(error) => Err(error).context("could not read mc.lock")
    }
}

pub(super) async fn write_lockfile(path: &PathBuf, lockfile: &Lockfile) -> McResult<()> {
    tokio::fs::write(path, lockfile.to_kdl_document().to_string())
        .await
        .context("could not write mc.lock")
}

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
    pub paths: PlayerPaths
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
    let mut lockfile = read_lockfile(&options.lockfile_path).await?;
    let locked_before = lockfile.players.clone();
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

    if lockfile.players != locked_before {
        write_lockfile(&options.lockfile_path, &lockfile).await?;
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
