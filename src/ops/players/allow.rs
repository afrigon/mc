use crate::context::McContext;
use crate::manifest::Manifest;
use crate::manifest::PlayerGroup;
use crate::manifest::document;
use crate::ops::players::PlayerListOptions;
use crate::ops::players::PlayerPaths;
use crate::ops::players::find_name;
use crate::ops::players::load;
use crate::ops::players::save;
use crate::ops::server_state::ServerState;
use crate::resolvers;
use crate::utils::errors::McResult;

fn warn_when_allow_list_is_off(context: &mut McContext, manifest: &Manifest) {
    if !manifest.server.allow_list {
        _ = context.shell().warn(
            "the allow list is off, so anyone can join; set `allow-list #true` in the `server` section of `mc.kdl` to enforce it"
        );
    }
}

pub struct AllowAddOptions {
    pub names: Vec<String>,
    pub paths: PlayerPaths
}

pub async fn add(context: &mut McContext, options: &AllowAddOptions) -> McResult<()> {
    let mut workspace = load(&options.paths).await?;
    let online_mode = workspace.manifest.server.online_mode();
    let mut added = Vec::new();

    warn_when_allow_list_is_off(context, &workspace.manifest);

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

    let mut live = ServerState::detect(context, workspace.manifest.server.rcon_port).await?;

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

pub async fn remove(context: &mut McContext, options: &AllowRemoveOptions) -> McResult<()> {
    let mut workspace = load(&options.paths).await?;
    let mut removed = Vec::new();

    warn_when_allow_list_is_off(context, &workspace.manifest);

    for name in &options.names {
        let existing = find_name(&workspace.manifest.players.allow, name)
            .ok_or_else(|| anyhow::anyhow!("`{}` is not allowed", name))?
            .clone();

        document::remove_player(&mut workspace.document, PlayerGroup::Allow, &existing);
        removed.push(existing);
    }

    save(&options.paths, &workspace).await?;

    let mut live = ServerState::detect(context, workspace.manifest.server.rcon_port).await?;

    for name in removed {
        _ = context
            .shell()
            .status("Disallowing", format!("{} from joining", name));

        live.send(context, format!("whitelist remove {}", name));
    }

    live.close();

    Ok(())
}

pub async fn list(context: &mut McContext, options: &PlayerListOptions) -> McResult<()> {
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
