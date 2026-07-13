use std::env;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::process::Stdio;
use std::time::Duration;

use anyhow::Context;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio_cron_scheduler::Job;
use tokio_cron_scheduler::JobScheduler;

use crate::context::McContext;
use crate::env::Architecture;
use crate::env::Platform;
use crate::manifest::Manifest;
use crate::minecraft::server_properties::ServerProperties;
use crate::ops;
use crate::ops::backups::BackupOptions;
use crate::ops::eula::EulaApplyOptions;
use crate::ops::init::InitDirectoriesOptions;
use crate::ops::java::JavaInstallOptions;
use crate::ops::lock::InstanceLocks;
use crate::ops::minecraft::MinecraftInstallOptions;
use crate::ops::mods::SyncModsOptions;
use crate::utils::errors::McResult;

/// How long to wait for the server to save and exit after asking it to stop
/// before forcing it down. Kept under systemd's default 90s `TimeoutStopSec` so
/// our grace window runs before systemd SIGKILLs the unit.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(85);

pub struct RunOptions {
    pub manifest_path: PathBuf,
    pub lockfile_path: PathBuf
}

fn sanitize_command(command: &Command) -> String {
    let command = command.as_std();

    let mut command_parts: Vec<String> = Vec::new();
    command_parts.push(command.get_program().to_string_lossy().into_owned());
    command_parts.extend(command.get_args().map(|a| a.to_string_lossy().into_owned()));

    command_parts
        .into_iter()
        .map(|s| {
            if s.contains(" ") || s.contains("\t") {
                format!("{:?}", s)
            } else {
                s
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Resolves when a shutdown signal arrives: Ctrl-C everywhere, plus SIGTERM on
/// Unix so `systemctl stop` triggers a graceful shutdown.
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::SignalKind;
        use tokio::signal::unix::signal;

        let mut interrupt =
            signal(SignalKind::interrupt()).expect("failed to register the SIGINT handler");
        let mut terminate =
            signal(SignalKind::terminate()).expect("failed to register the SIGTERM handler");

        tokio::select! {
            _ = interrupt.recv() => {}
            _ = terminate.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

// TODO: validate error context for all cases.
// - invalid versions
// - invalid toml format
// - missing toml file
// - etc.
/// Returns the server's exit status when it exits on its own; `None` when the
/// shutdown was requested by a signal.
pub async fn run(context: &mut McContext, options: &RunOptions) -> McResult<Option<ExitStatus>> {
    let manifest_string = tokio::fs::read_to_string(&options.manifest_path)
        .await
        .context("could not find mc.toml file")?;
    let manifest = toml::from_str::<Manifest>(&manifest_string)?;

    let path = context.cwd.clone();
    let instance_path = path.join("instance");
    let staging_path = path.join("temp");

    let init_directories_options = InitDirectoriesOptions { path: path.clone() };
    ops::init::init_directories(context, &init_directories_options).await?;

    // Take exclusive ownership of the world for as long as this server runs, so a
    // second `mc run` or a restore cannot touch it underneath us.
    let locks = InstanceLocks::new(&path);
    let mut world_lock = locks.world()?;
    let world_guard = world_lock
        .try_acquire()?
        .context("this instance is already running; only one server can run per directory")?;

    // Interrupted runs leave partial downloads behind; holding the world lock
    // guarantees nothing else is staging in there, so clear it out.
    match tokio::fs::remove_dir_all(&staging_path).await {
        Err(error) if error.kind() != ErrorKind::NotFound => return Err(error.into()),
        _ => {}
    }

    // EULA

    if !manifest.server.eula {
        anyhow::bail!(
            "the instance will not start until YOU agree to the Minecraft EULA (https://aka.ms/MinecraftEULA). you can do so by setting `eula = true` in `mc.toml`"
        );
    }

    let eula_options = EulaApplyOptions {
        accept: manifest.server.eula,
        instance_path: instance_path.clone()
    };

    ops::eula::apply(context, &eula_options).await?;

    // JAVA

    let java_directory = path.join("java");
    let java_path = java_directory.join(manifest.java.version.to_string());
    let current_platform = Platform::current();

    if !java_path.exists() {
        let java_install_options = JavaInstallOptions {
            architecture: Architecture::current(),
            platform: current_platform,
            version: manifest.java.version_descriptor(context).await?,
            java_directory,
            staging_directory: staging_path.clone()
        };

        ops::java::install(context, &java_install_options).await?;
    }

    let java_bin = match current_platform {
        Platform::Windows => "javaw.exe",
        _ => "java"
    };
    let java_bin_path = java_path.join("bin").join(java_bin);

    // MINECRAFT

    let minecraft_directory = path.join("minecraft");
    let minecraft_version = manifest.minecraft.resolved_version(context).await?;
    let minecraft_loader = manifest.minecraft.loader_descriptor(context).await?;
    let minecraft_descriptor_prefix = minecraft_loader
        .as_ref()
        .map(|l| l.to_string())
        .unwrap_or(String::from("minecraft"));
    let minecraft_descriptor = format!("{}-{}", minecraft_descriptor_prefix, minecraft_version);

    let minecraft_path = minecraft_directory
        .join(minecraft_descriptor)
        .join("server.jar");

    if !minecraft_path.exists() {
        let minecraft_install_options = MinecraftInstallOptions {
            version: minecraft_version.clone(),
            loader: minecraft_loader.clone(),
            minecraft_directory,
            staging_directory: staging_path.clone()
        };

        ops::minecraft::install(context, &minecraft_install_options).await?;
    }

    // TODO: fetch the configured version's capabilities (see `capabilities.rs`)
    // and fail fast when backups are enabled on a version too old to support RCON
    // (Capability::RemoteConsole) — without it the world cannot be flushed before
    // a backup.

    // PROPERTIES

    let mut properties = ServerProperties::default();

    properties.apply(&manifest);

    // Backups flush the world over rcon, so an enabled instance needs a password.
    // Honor an explicit one from the environment, otherwise generate a strong
    // one so backups work out of the box (enabling rcon without a password would
    // expose an unauthenticated console).
    properties.rcon_password = match env::var("MC_RCON_PASSWORD").ok() {
        Some(password) => Some(password),
        None if manifest.backups.enabled => Some(uuid::Uuid::new_v4().simple().to_string()),
        None => None
    };

    tokio::fs::write(
        instance_path.join("server.properties"),
        properties.to_string()?
    )
    .await?;

    // MODS

    let sync_options = SyncModsOptions {
        game_version: minecraft_version.clone(),
        loader: minecraft_loader.clone(),
        lockfile_path: options.lockfile_path.clone(),
        mods_path: instance_path.join("mods"),
        staging_path: staging_path.clone()
    };

    ops::mods::sync(context, &sync_options, &manifest.mods).await?;

    // PROCESS

    let mut command = tokio::process::Command::new(java_bin_path);

    command
        .args(manifest.java.args())
        .arg("-jar")
        .arg(minecraft_path.as_os_str())
        .arg("--nogui")
        .current_dir(&instance_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .kill_on_drop(true);

    let command_string = sanitize_command(&command);
    _ = context
        .shell()
        .status("Running", format!("`{}`", command_string));

    let mut child = command.spawn()?;

    // BACKUPS

    let mut scheduler = JobScheduler::new().await?;

    if manifest.backups.enabled {
        let world_path = instance_path.join(&manifest.name);
        let project_path = path.clone();
        let rcon_password = properties.rcon_password.clone();
        let storage = manifest.backups.effective_storage();
        let notifier = manifest.backups.notifier(context.http_client.clone());

        let backup_job = Job::new_async(manifest.backups.frequency, move |_, _| {
            let project_path = project_path.clone();
            let world_path = world_path.clone();
            let storage = storage.clone();
            let rcon_password = rcon_password.clone();
            let notifier = notifier.clone();

            Box::pin(async move {
                let backup_options = BackupOptions {
                    project_path,
                    world_path,
                    storage,
                    rcon_port: manifest.server.rcon_port,
                    rcon_password,
                    notifier
                };

                match ops::backups::backup(&backup_options).await {
                    Ok(_) => tracing::info!("world backup complete"),
                    Err(error) => tracing::error!("backup failed: {:?}", error)
                }
            })
        })?;

        scheduler.add(backup_job).await?;
    }

    scheduler.start().await?;

    let mut stdin = child
        .stdin
        .take()
        .context("could not attach to minecraft process")?;

    let mut exit_status = None;

    tokio::select! {
        status = child.wait() => {
            // the server exited on its own
            exit_status = Some(status?);
        }
        _ = shutdown_signal() => {
            _ = context
                .shell()
                .status("Stopping", "asking the server to save and shut down");

            stdin.write_all(b"stop\n").await?;
            stdin.flush().await?;

            // Wait for the server to flush and exit. Force it down if it hangs
            // past the grace period or a second signal arrives.
            tokio::select! {
                _ = child.wait() => {}
                _ = tokio::time::sleep(SHUTDOWN_GRACE) => {
                    _ = context
                        .shell()
                        .warn("the server did not stop in time, forcing it down");
                    let _ = child.kill().await;
                }
                _ = shutdown_signal() => {
                    _ = context
                        .shell()
                        .warn("received a second signal, forcing the server down");
                    let _ = child.kill().await;
                }
            }
        }
    };

    scheduler.shutdown().await?;

    drop(world_guard);

    Ok(exit_status)
}
