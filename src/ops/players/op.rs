use crate::context::McContext;
use crate::manifest::PlayerGroup;
use crate::manifest::document;
use crate::minecraft::MinecraftPermission;
use crate::ops::players::PlayerListOptions;
use crate::ops::players::PlayerPaths;
use crate::ops::players::find_name;
use crate::ops::players::load;
use crate::ops::players::save;
use crate::ops::server_state::ServerState;
use crate::resolvers;
use crate::utils;
use crate::utils::errors::McResult;

pub struct OpAddOptions {
    pub names: Vec<String>,
    pub level: Option<MinecraftPermission>,
    pub bypasses_player_limit: bool,
    pub paths: PlayerPaths
}

pub async fn add(context: &mut McContext, options: &OpAddOptions) -> McResult<()> {
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

    let mut live = ServerState::detect(context, workspace.manifest.server.rcon_port).await?;

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

pub async fn remove(context: &mut McContext, options: &OpRemoveOptions) -> McResult<()> {
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

    let mut live = ServerState::detect(context, workspace.manifest.server.rcon_port).await?;

    for name in removed {
        _ = context.shell().status("Deopping", &name);

        live.send(context, format!("deop {}", name));
    }

    live.close();

    Ok(())
}

pub async fn list(context: &mut McContext, options: &PlayerListOptions) -> McResult<()> {
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
