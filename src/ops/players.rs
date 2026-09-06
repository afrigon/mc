use std::net::IpAddr;
use std::path::PathBuf;

use anyhow::Context;
use chrono::DateTime;
use chrono::Utc;
use kdl::KdlDocument;
use kdl::KdlEntry;

use crate::context::McContext;
use crate::manifest::Manifest;
use crate::manifest::ManifestBan;
use crate::manifest::ManifestPlayers;
use crate::manifest::PlayerGroup;
use crate::manifest::document;
use crate::manifest::lock::Lockfile;
use crate::minecraft::MinecraftPermission;
use crate::minecraft::players;
use crate::minecraft::players::AllowEntry;
use crate::minecraft::players::BanDetails;
use crate::minecraft::players::BanEntry;
use crate::minecraft::players::IpBanEntry;
use crate::minecraft::players::OpEntry;
use crate::minecraft::server_properties::ServerProperties;
use crate::ops;
use crate::ops::lock::InstanceLocks;
use crate::resolvers;
use crate::utils;
use crate::utils::errors::McResult;

pub struct PlayerPaths {
    pub manifest_path: PathBuf,
    pub lockfile_path: PathBuf
}

struct Workspace {
    manifest: Manifest,
    document: KdlDocument,
    lockfile: Lockfile
}

async fn load(paths: &PlayerPaths) -> McResult<Workspace> {
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

async fn save(paths: &PlayerPaths, workspace: &Workspace) -> McResult<()> {
    tokio::fs::write(&paths.manifest_path, workspace.document.to_string()).await?;
    write_lockfile(&paths.lockfile_path, &workspace.lockfile).await
}

async fn read_lockfile(path: &PathBuf) -> McResult<Lockfile> {
    match tokio::fs::read_to_string(path).await {
        Ok(source) => Lockfile::from_kdl_str(&source).context("could not parse mc.lock"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Lockfile::default()),
        Err(error) => Err(error).context("could not read mc.lock")
    }
}

async fn write_lockfile(path: &PathBuf, lockfile: &Lockfile) -> McResult<()> {
    tokio::fs::write(path, lockfile.to_kdl_document().to_string())
        .await
        .context("could not write mc.lock")
}

/// Names are unique regardless of case, so a lookup matches whatever casing
/// the manifest holds.
fn find_name<'a>(names: impl IntoIterator<Item = &'a String>, name: &str) -> Option<&'a String> {
    names
        .into_iter()
        .find(|candidate| candidate.eq_ignore_ascii_case(name))
}

/// The running server, reached over rcon so a change applies without a
/// restart. Anything that cannot be applied live is deferred with a warning.
enum LiveServer {
    Stopped,
    Running(Option<minecraft_client_rs::Client>)
}

impl LiveServer {
    async fn detect(context: &McContext, rcon_port: u16) -> McResult<LiveServer> {
        let locks = InstanceLocks::new(&context.cwd);
        let mut world_lock = locks.world()?;

        if world_lock.try_acquire()?.is_some() {
            return Ok(LiveServer::Stopped);
        }

        let instance_path = context.cwd.join("instance");
        let rcon = ServerProperties::read_rcon_password(&instance_path)
            .await?
            .and_then(|password| ops::backups::connect_rcon(rcon_port, &password));

        Ok(LiveServer::Running(rcon))
    }

    fn send(&mut self, context: &mut McContext, command: String) {
        match self {
            LiveServer::Stopped => {}
            LiveServer::Running(Some(rcon)) => match rcon.send_command(command) {
                Ok(response) => {
                    _ = context.shell().status("Server", response.body.trim());
                }
                Err(error) => {
                    _ = context.shell().warn(format!(
                        "could not apply the change to the running server: {}; it takes effect at the next restart",
                        error
                    ));
                }
            },
            LiveServer::Running(None) => {
                _ = context.shell().warn(
                    "the server is running but rcon is unavailable; the change takes effect at the next restart"
                );
            }
        }
    }

    fn defer(&self, context: &mut McContext, what: &str) {
        if matches!(self, LiveServer::Running(_)) {
            _ = context.shell().warn(format!(
                "{} takes effect at the next restart; the running server cannot apply it",
                what
            ));
        }
    }

    fn close(self) {
        if let LiveServer::Running(Some(mut rcon)) = self {
            let _ = rcon.close();
        }
    }
}

fn ban_entries(ban: &ManifestBan) -> Vec<KdlEntry> {
    let mut entries = Vec::new();

    if let Some(reason) = &ban.reason {
        entries.push(utils::kdl::quoted_property("reason", reason));
    }

    if let Some(created) = ban.created {
        entries.push(utils::kdl::quoted_property(
            "created",
            &created.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        ));
    }

    if let Some(expires) = ban.expires {
        entries.push(utils::kdl::quoted_property(
            "expires",
            &expires.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        ));
    }

    entries
}

fn describe_ban(ban: &ManifestBan) -> String {
    let mut description = ban
        .reason
        .clone()
        .unwrap_or_else(|| String::from("no reason given"));

    if let Some(created) = ban.created {
        description.push_str(&format!(", since {}", created.format("%Y-%m-%d %H:%M UTC")));
    }

    if let Some(expires) = ban.expires {
        description.push_str(&format!(", until {}", expires.format("%Y-%m-%d %H:%M UTC")));
    }

    description
}

// ALLOW

pub struct AllowAddOptions {
    pub names: Vec<String>,
    pub paths: PlayerPaths
}

pub async fn allow_add(context: &mut McContext, options: &AllowAddOptions) -> McResult<()> {
    let mut workspace = load(&options.paths).await?;
    let online_mode = workspace.manifest.server.online_mode();
    let mut added = Vec::new();

    for name in &options.names {
        if let Some(existing) = find_name(&workspace.manifest.players.allow, name) {
            _ = context
                .shell()
                .warn(format!("`{}` is already allowed", existing));

            continue;
        }

        if let Some(banned) = find_name(workspace.manifest.players.ban.keys(), name) {
            anyhow::bail!(
                "`{}` is banned; run `mc ban remove {}` before allowing them",
                banned,
                banned
            );
        }

        let player =
            resolvers::players::resolve(context, &mut workspace.lockfile, name, online_mode)
                .await?;

        document::set_player(
            &mut workspace.document,
            PlayerGroup::Allow,
            &player.name,
            Vec::new()
        )?;
        added.push(player.name);
    }

    save(&options.paths, &workspace).await?;

    let mut live = LiveServer::detect(context, workspace.manifest.server.rcon_port).await?;

    for name in added {
        _ = context
            .shell()
            .status("Allowing", format!("{} to join", name));

        live.send(context, format!("whitelist add {}", name));
    }

    live.close();

    Ok(())
}

pub struct AllowRemoveOptions {
    pub names: Vec<String>,
    pub paths: PlayerPaths
}

pub async fn allow_remove(context: &mut McContext, options: &AllowRemoveOptions) -> McResult<()> {
    let mut workspace = load(&options.paths).await?;
    let mut removed = Vec::new();

    for name in &options.names {
        let existing = find_name(&workspace.manifest.players.allow, name)
            .ok_or_else(|| anyhow::anyhow!("`{}` is not allowed", name))?
            .clone();

        document::remove_player(&mut workspace.document, PlayerGroup::Allow, &existing);
        removed.push(existing);
    }

    save(&options.paths, &workspace).await?;

    let mut live = LiveServer::detect(context, workspace.manifest.server.rcon_port).await?;

    for name in removed {
        _ = context
            .shell()
            .status("Disallowing", format!("{} from joining", name));

        live.send(context, format!("whitelist remove {}", name));
    }

    live.close();

    Ok(())
}

pub struct PlayerListOptions {
    pub paths: PlayerPaths
}

pub async fn allow_list(context: &mut McContext, options: &PlayerListOptions) -> McResult<()> {
    let workspace = load(&options.paths).await?;
    let allow = &workspace.manifest.players.allow;

    if allow.is_empty() {
        _ = context.shell().status("Allowed", "nobody");
    }

    let mut shell = context.shell();
    let stdout = shell.out();

    for name in allow {
        writeln!(stdout, "{}", name)?;
    }

    Ok(())
}

// BAN

pub struct BanAddOptions {
    pub names: Vec<String>,
    pub addresses: Vec<IpAddr>,
    pub reason: Option<String>,
    pub expires: Option<DateTime<Utc>>,
    pub paths: PlayerPaths
}

pub async fn ban_add(context: &mut McContext, options: &BanAddOptions) -> McResult<()> {
    let mut workspace = load(&options.paths).await?;
    let online_mode = workspace.manifest.server.online_mode();
    let ban = ManifestBan {
        reason: options.reason.clone(),
        created: Some(Utc::now()),
        expires: options.expires
    };
    let mut banned_names = Vec::new();
    let mut disallowed_names = Vec::new();

    for name in &options.names {
        if let Some(existing) = find_name(workspace.manifest.players.ban.keys(), name) {
            anyhow::bail!("`{}` is already banned", existing);
        }

        let player =
            resolvers::players::resolve(context, &mut workspace.lockfile, name, online_mode)
                .await?;

        if let Some(allowed) = find_name(&workspace.manifest.players.allow, name).cloned() {
            _ = context
                .shell()
                .warn(format!("`{}` was removed from the allow list", allowed));
            document::remove_player(&mut workspace.document, PlayerGroup::Allow, &allowed);
            disallowed_names.push(allowed);
        }

        document::set_player(
            &mut workspace.document,
            PlayerGroup::Ban,
            &player.name,
            ban_entries(&ban)
        )?;
        banned_names.push(player.name);
    }

    for address in &options.addresses {
        if workspace.manifest.players.ban_ip.contains_key(address) {
            anyhow::bail!("`{}` is already banned", address);
        }

        document::set_player(
            &mut workspace.document,
            PlayerGroup::BanIp,
            &address.to_string(),
            ban_entries(&ban)
        )?;
    }

    save(&options.paths, &workspace).await?;

    let mut live = LiveServer::detect(context, workspace.manifest.server.rcon_port).await?;
    let reason = options
        .reason
        .as_ref()
        .map(|reason| format!(" {}", reason))
        .unwrap_or_default();

    for name in disallowed_names {
        live.send(context, format!("whitelist remove {}", name));
    }

    for name in banned_names {
        _ = context
            .shell()
            .status("Banning", format!("{} ({})", name, describe_ban(&ban)));

        live.send(context, format!("ban {}{}", name, reason));
    }

    for address in &options.addresses {
        _ = context
            .shell()
            .status("Banning", format!("{} ({})", address, describe_ban(&ban)));

        live.send(context, format!("ban-ip {}{}", address, reason));
    }

    if options.expires.is_some() {
        live.defer(context, "the ban expiry");
    }

    live.close();

    Ok(())
}

pub struct BanRemoveOptions {
    pub names: Vec<String>,
    pub addresses: Vec<IpAddr>,
    pub paths: PlayerPaths
}

pub async fn ban_remove(context: &mut McContext, options: &BanRemoveOptions) -> McResult<()> {
    let mut workspace = load(&options.paths).await?;
    let mut pardoned_names = Vec::new();
    let mut pardoned_addresses = Vec::new();

    for name in &options.names {
        match find_name(workspace.manifest.players.ban.keys(), name).cloned() {
            Some(existing) => {
                document::remove_player(&mut workspace.document, PlayerGroup::Ban, &existing);
                pardoned_names.push(existing);
            }
            None => {
                _ = context.shell().warn(format!("`{}` is not banned", name));
            }
        }
    }

    for address in &options.addresses {
        if workspace.manifest.players.ban_ip.contains_key(address) {
            document::remove_player(
                &mut workspace.document,
                PlayerGroup::BanIp,
                &address.to_string()
            );
            pardoned_addresses.push(*address);
        } else {
            _ = context.shell().warn(format!("`{}` is not banned", address));
        }
    }

    save(&options.paths, &workspace).await?;

    let mut live = LiveServer::detect(context, workspace.manifest.server.rcon_port).await?;

    for name in pardoned_names {
        _ = context.shell().status("Unbanning", &name);

        live.send(context, format!("pardon {}", name));
    }

    for address in pardoned_addresses {
        _ = context.shell().status("Unbanning", address);

        live.send(context, format!("pardon-ip {}", address));
    }

    live.close();

    Ok(())
}

pub async fn ban_list(context: &mut McContext, options: &PlayerListOptions) -> McResult<()> {
    let workspace = load(&options.paths).await?;
    let players = &workspace.manifest.players;

    if players.ban.is_empty() && players.ban_ip.is_empty() {
        _ = context.shell().status("Banned", "nobody");
    }

    let mut shell = context.shell();
    let stdout = shell.out();

    for (name, ban) in &players.ban {
        writeln!(stdout, "{}: {}", name, describe_ban(ban))?;
    }

    for (address, ban) in &players.ban_ip {
        writeln!(stdout, "{}: {}", address, describe_ban(ban))?;
    }

    Ok(())
}

// OP

pub struct OpAddOptions {
    pub names: Vec<String>,
    pub level: Option<MinecraftPermission>,
    pub bypasses_player_limit: bool,
    pub paths: PlayerPaths
}

pub async fn op_add(context: &mut McContext, options: &OpAddOptions) -> McResult<()> {
    let mut workspace = load(&options.paths).await?;
    let online_mode = workspace.manifest.server.online_mode();
    let server_level = workspace.manifest.server.op_permission_level();
    let live_applicable =
        options.level.is_none_or(|level| level == server_level) && !options.bypasses_player_limit;

    let mut entries = Vec::new();

    if let Some(level) = options.level {
        entries.push(utils::kdl::property("level", level as i128));
    }

    if options.bypasses_player_limit {
        entries.push(utils::kdl::property("bypasses-player-limit", true));
    }

    let mut new_ops = Vec::new();
    let mut updated_ops = Vec::new();

    for name in &options.names {
        let existing = find_name(workspace.manifest.players.op.keys(), name).cloned();

        let player_name = match existing {
            Some(existing) => {
                _ = context.shell().warn(format!(
                    "`{}` is already an operator; updating their settings",
                    existing
                ));
                updated_ops.push(existing.clone());

                existing
            }
            None => {
                let player = resolvers::players::resolve(
                    context,
                    &mut workspace.lockfile,
                    name,
                    online_mode
                )
                .await?;

                new_ops.push(player.name.clone());

                player.name
            }
        };

        document::set_player(
            &mut workspace.document,
            PlayerGroup::Op,
            &player_name,
            entries.clone()
        )?;
    }

    save(&options.paths, &workspace).await?;

    let mut live = LiveServer::detect(context, workspace.manifest.server.rcon_port).await?;

    for name in new_ops {
        _ = context.shell().status("Opping", &name);

        if live_applicable {
            live.send(context, format!("op {}", name));
        } else {
            live.defer(context, &format!("the operator level of `{}`", name));
        }
    }

    for name in updated_ops {
        _ = context
            .shell()
            .status("Updating", format!("operator {}", name));

        live.defer(context, &format!("the operator level of `{}`", name));
    }

    live.close();

    Ok(())
}

pub struct OpRemoveOptions {
    pub names: Vec<String>,
    pub paths: PlayerPaths
}

pub async fn op_remove(context: &mut McContext, options: &OpRemoveOptions) -> McResult<()> {
    let mut workspace = load(&options.paths).await?;
    let mut removed = Vec::new();

    for name in &options.names {
        match find_name(workspace.manifest.players.op.keys(), name).cloned() {
            Some(existing) => {
                document::remove_player(&mut workspace.document, PlayerGroup::Op, &existing);
                removed.push(existing);
            }
            None => {
                _ = context
                    .shell()
                    .warn(format!("`{}` is not an operator", name));
            }
        }
    }

    save(&options.paths, &workspace).await?;

    let mut live = LiveServer::detect(context, workspace.manifest.server.rcon_port).await?;

    for name in removed {
        _ = context.shell().status("Deopping", &name);

        live.send(context, format!("deop {}", name));
    }

    live.close();

    Ok(())
}

pub async fn op_list(context: &mut McContext, options: &PlayerListOptions) -> McResult<()> {
    let workspace = load(&options.paths).await?;
    let server_level = workspace.manifest.server.op_permission_level();
    let ops = &workspace.manifest.players.op;

    if ops.is_empty() {
        _ = context.shell().status("Operators", "nobody");
    }

    let mut shell = context.shell();
    let stdout = shell.out();

    for (name, op) in ops {
        let level = op.level.unwrap_or(server_level) as u8;

        if op.bypasses_player_limit {
            writeln!(
                stdout,
                "{}: level {}, bypasses the player limit",
                name, level
            )?;
        } else {
            writeln!(stdout, "{}: level {}", name, level)?;
        }
    }

    Ok(())
}

// APPLY

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
