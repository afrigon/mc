use std::env;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::process::Stdio;
use std::sync::PoisonError;
use std::time::Duration;

use anyhow::Context;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio_cron_scheduler::Job;
use tokio_cron_scheduler::JobScheduler;
use tokio_util::sync::CancellationToken;

use crate::context::McContext;
use crate::env::Architecture;
use crate::env::Platform;
use crate::manifest::Manifest;
use crate::minecraft::log4j;
use crate::minecraft::server_properties::ManagedServerProperties;
use crate::minecraft::server_properties::ServerProperties;
use crate::ops;
use crate::ops::backups::BackupOptions;
use crate::ops::eula::EulaApplyOptions;
use crate::ops::init::InitDirectoriesOptions;
use crate::ops::java::JavaInstallOptions;
use crate::ops::lock::InstanceLocks;
use crate::ops::minecraft::MinecraftInstallOptions;
use crate::ops::mods::SyncModsOptions;
use crate::ops::notifications::ServerEvent;
use crate::utils;
use crate::utils::errors::McResult;

/// How long to wait for the server to save and exit after asking it to stop
/// before forcing it down. Kept under systemd's default 90s `TimeoutStopSec` so
/// our grace window runs before systemd SIGKILLs the unit.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(85);

/// server.properties keys whose values are sensitive.
const SECRET_PROPERTY_KEYS: [&str; 3] = [
    "management-server-secret",
    "management-server-tls-keystore-password",
    "rcon.password"
];

pub struct RunOptions {
    pub manifest_path: PathBuf,
    pub lockfile_path: PathBuf
}

// server.properties can hold plaintext secrets, so it is created readable only
// by the user running the server.
#[cfg(unix)]
async fn write_server_properties(
    context: &mut McContext,
    path: &Path,
    contents: &str,
    contains_secrets: bool
) -> McResult<()> {
    use std::os::unix::fs::PermissionsExt;

    match tokio::fs::metadata(path).await {
        Ok(metadata) => {
            let mode = metadata.permissions().mode();

            if mode & 0o077 != 0 {
                if contains_secrets {
                    anyhow::bail!(
                        "{} is accessible to other users (mode {:03o}) and holds secrets such as the rcon password; fix it with `chmod 600 {}`",
                        path.display(),
                        mode & 0o777,
                        path.display()
                    );
                }

                _ = context.shell().warn(format!(
                    "{} is accessible to other users (mode {:03o}); fix it with `chmod 600 {}`",
                    path.display(),
                    mode & 0o777,
                    path.display()
                ));
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).context("could not inspect the server.properties permissions");
        }
    }

    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .await?;

    file.write_all(contents.as_bytes()).await?;
    file.flush().await?;

    Ok(())
}

#[cfg(not(unix))]
async fn write_server_properties(
    _context: &mut McContext,
    path: &Path,
    contents: &str,
    _contains_secrets: bool
) -> McResult<()> {
    tokio::fs::write(path, contents).await?;

    Ok(())
}

fn has_jvm_property(arguments: &[String], property: &str) -> bool {
    let prefix = format!("-D{}=", property);

    arguments
        .iter()
        .any(|argument| argument.starts_with(&prefix))
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

    // A missing world alongside a `.restore.bak` can only mean a restore died
    // before putting a world in place; starting now would silently generate a
    // fresh one.
    let world_path = instance_path.join(&manifest.name);
    let aside_path = world_path.with_extension("restore.bak");

    if !tokio::fs::try_exists(&world_path).await? && tokio::fs::try_exists(&aside_path).await? {
        anyhow::bail!(
            "an interrupted restore left no world at `{}`; run `mc restore` again, or rename `{}` back to recover the previous world",
            world_path.display(),
            aside_path.display()
        );
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

    let java_directory = path.join(".java");
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

    let minecraft_directory = path.join(".minecraft");
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

    // Backups flush the world over rcon, so an enabled instance needs a password.
    // The environment password rides the managed layer; the generated fallback
    // sits in the base layer so a `rcon.password` override can replace it.
    let environment_rcon_password = env::var("MC_RCON_PASSWORD").ok();

    properties.rcon_password = match &environment_rcon_password {
        None if manifest.backups.enabled => Some(uuid::Uuid::new_v4().simple().to_string()),
        _ => None
    };

    let managed = ManagedServerProperties::from_manifest(&manifest, environment_rcon_password);
    let managed_entries = managed.to_map()?;

    let property_overrides = manifest.server.property_overrides()?;

    for key in property_overrides.keys() {
        if managed_entries.contains_key(key) {
            _ = context.shell().warn(format!(
                "the `{}` entry under [server.properties] conflicts with a value managed through the manifest and was ignored",
                key
            ));
        }
    }

    if property_overrides.contains_key("enable-rcon") {
        _ = context.shell().warn(
            "the `enable-rcon` entry under [server.properties] was ignored; rcon is enabled when a rcon password is configured"
        );
    }

    let mut property_entries = properties.to_entries(&property_overrides, &managed_entries)?;

    // rcon without a password would expose an unauthenticated console, so the
    // switch is derived from the effective password instead of set directly.
    let rcon_enabled = property_entries
        .get("rcon.password")
        .is_some_and(|password| !password.is_empty());

    property_entries.insert(String::from("enable-rcon"), rcon_enabled.to_string());

    let contains_secrets = SECRET_PROPERTY_KEYS.iter().any(|key| {
        property_entries
            .get(*key)
            .is_some_and(|value| !value.is_empty())
    });

    write_server_properties(
        context,
        &instance_path.join("server.properties"),
        &ServerProperties::entries_to_string(&property_entries)?,
        contains_secrets
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

    let server_log_level = match context.log_level {
        tracing::Level::ERROR => "error",
        tracing::Level::WARN => "warn",
        tracing::Level::INFO => "info",
        tracing::Level::DEBUG => "debug",
        tracing::Level::TRACE => "trace"
    };

    command.args(manifest.java.args());

    // Keep the server's console log level in sync with mc's verbosity, unless
    // the manifest configures the corresponding property itself.
    if !has_jvm_property(&manifest.java.jvm_arguments, "log4j.configurationFile") {
        tokio::fs::write(
            instance_path.join("log4j2.xml"),
            log4j::configuration(server_log_level)
        )
        .await?;

        command.arg("-Dlog4j.configurationFile=log4j2.xml");
    }

    if minecraft_loader.is_some()
        && !has_jvm_property(&manifest.java.jvm_arguments, "fabric.log.level")
    {
        command.arg(format!("-Dfabric.log.level={}", server_log_level));
    }

    command
        .arg("-jar")
        .arg(minecraft_path.as_os_str())
        .arg("--nogui")
        .current_dir(&instance_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .kill_on_drop(true);

    utils::process::detach_from_terminal_signals(&mut command);

    let command_string = sanitize_command(&command);
    _ = context
        .shell()
        .status("Running", format!("`{}`", command_string));

    let notifier = manifest.notifier(context);

    let mut shutdown = utils::process::ShutdownSignals::register()?;

    let mut child = command.spawn()?;

    if let Some(ref notifier) = notifier {
        notifier
            .notify_server(&manifest.name, &ServerEvent::Started)
            .await;
    }

    let rcon_password = property_entries
        .get("rcon.password")
        .filter(|password| !password.is_empty())
        .cloned();

    // Auto-save is asserted once the server accepts rcon connections, so
    // nothing — a mod, an unusual recovery state, or a differing server
    // default — can leave an instance silently not saving.
    if let Some(password) = rcon_password.clone() {
        let rcon_port = manifest.server.rcon_port;
        let shell = context.shell_handle();

        tokio::spawn(async move {
            for _ in 0..60 {
                tokio::time::sleep(Duration::from_secs(2)).await;

                if let Some(mut rcon) = ops::backups::connect_rcon(rcon_port, &password) {
                    if rcon.send_command("save-on".to_string()).is_ok() {
                        tracing::debug!("asserted auto-save at startup");
                    }

                    let _ = rcon.close();

                    return;
                }
            }

            let mut shell = shell.lock().unwrap_or_else(PoisonError::into_inner);

            _ = shell.warn("could not reach the server over rcon to assert auto-save");
        });
    }

    // BACKUPS

    let backup_cancel = CancellationToken::new();

    let mut scheduler = JobScheduler::new().await?;

    if manifest.backups.enabled {
        let world_path = instance_path.join(&manifest.name);
        let project_path = path.clone();
        let rcon_password = rcon_password.clone();
        let storage = manifest.backups.effective_storage();
        let notifier = notifier.clone();
        let shell = context.shell_handle();
        let cancel = backup_cancel.clone();

        let backup_job = Job::new_async(manifest.backups.frequency, move |_, _| {
            let project_path = project_path.clone();
            let world_path = world_path.clone();
            let storage = storage.clone();
            let rcon_password = rcon_password.clone();
            let notifier = notifier.clone();
            let shell = shell.clone();
            let cancel = cancel.clone();

            Box::pin(async move {
                let backup_options = BackupOptions {
                    project_path,
                    world_path,
                    storage,
                    rcon_port: manifest.server.rcon_port,
                    rcon_password,
                    notifier,
                    name: None,
                    shell: shell.clone(),
                    cancel
                };

                if let Err(error) = ops::backups::backup(&backup_options).await {
                    let mut shell = shell.lock().unwrap_or_else(PoisonError::into_inner);

                    _ = shell.error(format!("backup failed: {:?}", error));
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
    let mut forced_down = false;

    tokio::select! {
        status = child.wait() => {
            // the server exited on its own
            exit_status = Some(status?);
        }
        _ = shutdown.recv() => {
            // An in-flight scheduled backup restores auto-save and discards
            // its partial archive on cancellation; the server can stop
            // without waiting on it.
            backup_cancel.cancel();

            _ = context
                .shell()
                .status("Stopping", "asking the server to save and shut down");

            stdin.write_all(b"stop\n").await?;
            stdin.flush().await?;

            // Wait for the server to flush and exit. Force it down if it hangs
            // past the grace period or a second signal arrives.
            tokio::select! {
                status = child.wait() => {
                    match status {
                        Ok(status) if status.success() => {
                            _ = context
                                .shell()
                                .status("Stopped", "the server saved the world and exited cleanly");
                        }
                        Ok(status) => {
                            _ = context
                                .shell()
                                .warn(format!("the server exited with {} while stopping", status));
                        }
                        Err(error) => {
                            _ = context
                                .shell()
                                .warn(format!("could not observe the server exit: {}", error));
                        }
                    }
                }
                _ = tokio::time::sleep(SHUTDOWN_GRACE) => {
                    _ = context
                        .shell()
                        .warn("the server did not stop in time, forcing it down");
                    let _ = child.kill().await;
                    forced_down = true;
                    _ = context.shell().status("Stopped", "the server was forced down");
                }
                _ = shutdown.recv() => {
                    _ = context
                        .shell()
                        .warn("received a second signal, forcing the server down");
                    let _ = child.kill().await;
                    forced_down = true;
                    _ = context.shell().status("Stopped", "the server was forced down");
                }
            }
        }
    };

    if let Some(ref notifier) = notifier {
        let event = match exit_status {
            Some(status) if !status.success() => ServerEvent::Crashed(status),
            _ if forced_down => ServerEvent::Sigkill,
            _ => ServerEvent::Stopped
        };

        notifier.notify_server(&manifest.name, &event).await;
    }

    scheduler.shutdown().await?;

    drop(world_guard);

    Ok(exit_status)
}
