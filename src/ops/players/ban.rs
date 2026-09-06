use std::net::IpAddr;

use chrono::DateTime;
use chrono::Utc;
use kdl::KdlEntry;

use crate::context::McContext;
use crate::manifest::ManifestBan;
use crate::manifest::ManifestPaths;
use crate::manifest::PlayerGroup;
use crate::manifest::document;
use crate::ops::players::PlayerListOptions;
use crate::ops::players::find_name;
use crate::ops::server_state::ServerState;
use crate::ops::workspace::Workspace;
use crate::resolvers;
use crate::utils;
use crate::utils::errors::McResult;

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

pub struct BanAddOptions {
    pub names: Vec<String>,
    pub addresses: Vec<IpAddr>,
    pub reason: Option<String>,
    pub expires: Option<DateTime<Utc>>,
    pub paths: ManifestPaths
}

pub async fn add(context: &mut McContext, options: &BanAddOptions) -> McResult<()> {
    let mut workspace = Workspace::load(&options.paths).await?;
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

    workspace.save().await?;

    let mut live = ServerState::detect(context, workspace.manifest.server.rcon_port).await?;
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
    pub paths: ManifestPaths
}

pub async fn remove(context: &mut McContext, options: &BanRemoveOptions) -> McResult<()> {
    let mut workspace = Workspace::load(&options.paths).await?;
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

    workspace.save().await?;

    let mut live = ServerState::detect(context, workspace.manifest.server.rcon_port).await?;

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

pub async fn list(context: &mut McContext, options: &PlayerListOptions) -> McResult<()> {
    let workspace = Workspace::load(&options.paths).await?;
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
