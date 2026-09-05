use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::time::Duration;
use std::time::Instant;

use anyhow::Context;
use serde::Deserialize;
use serde::Serialize;
use tokio::process::Child;
use tokio::process::Command;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::context::McContext;
use crate::env::Architecture;
use crate::env::Platform;
use crate::network;
use crate::services::playit_api;
use crate::services::playit_api::CLAIM_URL;
use crate::services::playit_api::MINECRAFT_JAVA_TUNNEL_TYPE;
use crate::services::playit_api::PlayitApi;
use crate::services::playit_api::PlayitApiClaimStatus;
use crate::services::playit_api::PlayitApiCreateTunnel;
use crate::services::tunnel_provider::TunnelProvider;
use crate::tunnel::TunnelDescriptor;
use crate::tunnel::TunnelProviderKind;
use crate::utils;
use crate::utils::errors::McResult;
use crate::utils::shell::Shell;

pub const SECRET_FILE_NAME: &'static str = "playit.toml";
pub const LOG_FILE_NAME: &'static str = "playitd.log";

const CLAIM_POLL_INTERVAL: Duration = Duration::from_secs(2);
const CLAIM_TIMEOUT: Duration = Duration::from_secs(600);
const AGENT_RESTART_DELAY: Duration = Duration::from_secs(5);
const AGENT_STOP_GRACE: Duration = Duration::from_secs(5);
const TUNNEL_ALLOCATION_POLL_INTERVAL: Duration = Duration::from_secs(3);
const TUNNEL_ALLOCATION_TIMEOUT: Duration = Duration::from_secs(15);

pub fn agent_path(
    tunnel_directory: &Path,
    version: &TunnelDescriptor,
    platform: Platform
) -> PathBuf {
    let binary = match version.product {
        TunnelProviderKind::Playit => PlayitApi::agent_binary_name(platform)
    };

    tunnel_directory.join(version.to_string()).join(binary)
}

pub fn secret_path(tunnel_directory: &Path) -> PathBuf {
    tunnel_directory.join(SECRET_FILE_NAME)
}

pub fn socket_path(name: &str) -> String {
    #[cfg(windows)]
    {
        format!(r"\\.\pipe\mc-playitd-{}", name)
    }

    #[cfg(not(windows))]
    {
        let _ = name;

        String::from("playitd.sock")
    }
}

pub struct TunnelInstallOptions {
    pub version: TunnelDescriptor,
    pub platform: Platform,
    pub architecture: Architecture,
    pub tunnel_directory: PathBuf,
    pub staging_directory: PathBuf
}

pub async fn install(context: &mut McContext, options: &TunnelInstallOptions) -> McResult<()> {
    let name = options.version.to_string();
    let path = agent_path(
        &options.tunnel_directory,
        &options.version,
        options.platform
    );

    if path.exists() {
        anyhow::bail!("{} is already installed", name);
    }

    _ = context.shell().status("Installing", &name);

    // Create only the parent: the final rename out of staging is what creates
    // `path`, so its existence proves a completed install.
    let parent = path
        .parent()
        .context("the tunnel agent path has no parent directory")?;

    tokio::fs::create_dir_all(parent).await?;

    let source = match options.version.product {
        TunnelProviderKind::Playit => {
            PlayitApi::agent_source(
                &context.http_client,
                &options.version.version,
                options.platform,
                options.architecture
            )
            .await?
        }
    };

    network::stream_artifact(
        &context.http_client,
        source,
        &path,
        &options.staging_directory
    )
    .await
}

pub struct TunnelListOptions {
    pub limit: usize
}

pub async fn list(context: &mut McContext, options: &TunnelListOptions) -> McResult<()> {
    let versions = PlayitApi::versions(&context.http_client).await?;

    let mut shell = context.shell();
    let stdout = shell.out();

    for (index, version) in versions.iter().take(options.limit).enumerate() {
        if index == 0 {
            writeln!(stdout, "{} (latest)", version)?
        } else {
            writeln!(stdout, "{}", version)?
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct SecretFile {
    secret_key: String
}

pub async fn read_secret(path: &Path) -> McResult<String> {
    let contents = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("could not read {}", path.display()))?;
    let file: SecretFile = toml::from_str(&contents).with_context(|| {
        format!(
            "could not parse the tunnel agent secret in {}",
            path.display()
        )
    })?;

    Ok(file.secret_key)
}

fn claim_code() -> String {
    hex::encode(&uuid::Uuid::new_v4().as_bytes()[..5])
}

pub struct TunnelClaimOptions {
    pub secret_path: PathBuf,
    pub force: bool
}

pub async fn claim(context: &mut McContext, options: &TunnelClaimOptions) -> McResult<()> {
    if !options.force && tokio::fs::try_exists(&options.secret_path).await? {
        anyhow::bail!(
            "a tunnel agent is already claimed at {}; pass --force to replace it",
            options.secret_path.display()
        );
    }

    let code = claim_code();

    _ = context.shell().status(
        "Claiming",
        format!(
            "tunnel agent, approve it at {}/{} with your playit.gg account",
            CLAIM_URL, code
        )
    );

    let deadline = Instant::now() + CLAIM_TIMEOUT;

    loop {
        match playit_api::claim_setup(&context.http_client, &code).await? {
            PlayitApiClaimStatus::UserAccepted => break,
            PlayitApiClaimStatus::UserRejected => {
                anyhow::bail!("the tunnel agent claim was rejected in the browser")
            }
            PlayitApiClaimStatus::WaitingForUserVisit | PlayitApiClaimStatus::WaitingForUser => {}
        }

        if Instant::now() > deadline {
            anyhow::bail!("timed out waiting for the tunnel agent claim to be approved");
        }

        tokio::time::sleep(CLAIM_POLL_INTERVAL).await;
    }

    let secret = loop {
        if let Some(secret) = playit_api::claim_exchange(&context.http_client, &code).await? {
            break secret;
        }

        if Instant::now() > deadline {
            anyhow::bail!("timed out waiting for the tunnel agent secret");
        }

        tokio::time::sleep(CLAIM_POLL_INTERVAL).await;
    };

    if let Some(parent) = options.secret_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let contents = toml::to_string(&SecretFile { secret_key: secret })?;

    utils::private_file::write_private_file(context, &options.secret_path, &contents, true).await?;

    _ = context.shell().status(
        "Claimed",
        format!(
            "tunnel agent secret saved to {}",
            options.secret_path.display()
        )
    );

    Ok(())
}

pub struct TunnelEnsureOptions {
    pub secret: String,
    pub server_port: u16,
    pub name: String
}

/// Returns the public address of the instance's Minecraft tunnel, creating
/// the tunnel when the agent has none; `None` while playit is still
/// allocating it.
pub async fn ensure(
    context: &mut McContext,
    options: &TunnelEnsureOptions
) -> McResult<Option<String>> {
    let is_minecraft =
        |tunnel_type: &Option<String>| tunnel_type.as_deref() == Some(MINECRAFT_JAVA_TUNNEL_TYPE);

    let run_data = playit_api::get_run_data(&context.http_client, &options.secret).await?;

    if let Some(tunnel) = run_data.tunnels.iter().find(|tunnel| {
        is_minecraft(&tunnel.tunnel_type) && tunnel.local_port == options.server_port
    }) {
        return Ok(Some(tunnel.address()));
    }

    if run_data
        .pending
        .iter()
        .any(|tunnel| is_minecraft(&tunnel.tunnel_type))
    {
        return Ok(None);
    }

    _ = context.shell().status(
        "Creating",
        format!("a Minecraft tunnel for port {}", options.server_port)
    );

    let request = PlayitApiCreateTunnel::minecraft_java(
        options.name.clone(),
        run_data.agent_id,
        options.server_port
    );

    playit_api::create_tunnel(&context.http_client, &options.secret, &request).await?;

    // A fresh tunnel is reported as pending until playit allocates it, which
    // usually takes a few seconds.
    let deadline = Instant::now() + TUNNEL_ALLOCATION_TIMEOUT;

    loop {
        tokio::time::sleep(TUNNEL_ALLOCATION_POLL_INTERVAL).await;

        let run_data = playit_api::get_run_data(&context.http_client, &options.secret).await?;

        if let Some(tunnel) = run_data.tunnels.iter().find(|tunnel| {
            is_minecraft(&tunnel.tunnel_type) && tunnel.local_port == options.server_port
        }) {
            return Ok(Some(tunnel.address()));
        }

        if Instant::now() > deadline {
            return Ok(None);
        }
    }
}

pub struct TunnelAgentOptions {
    pub agent_path: PathBuf,
    pub work_directory: PathBuf,
    pub socket_path: String,
    pub log_level: &'static str,
    pub logs: bool
}

fn agent_command(options: &TunnelAgentOptions) -> Command {
    let mut command = Command::new(&options.agent_path);

    command
        .arg("--secret-path")
        .arg(SECRET_FILE_NAME)
        .arg("--socket-path")
        .arg(&options.socket_path)
        .env("PLAYIT_LOG", options.log_level)
        .current_dir(&options.work_directory)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .kill_on_drop(true);

    // The agent logs to stderr unless given a log file, so hidden output is
    // kept on disk rather than dropped.
    if !options.logs {
        command.arg("--log-path").arg(LOG_FILE_NAME);
    }

    utils::process::detach_from_terminal_signals(&mut command);

    command
}

// The agent shuts down cleanly on SIGINT, telling the relay it is going
// away; a plain kill leaves the tunnel looking online until its session
// expires.
async fn stop_agent(child: &mut Child) {
    #[cfg(unix)]
    {
        let interrupted = child
            .id()
            .map(|pid| unsafe { libc::kill(pid as libc::pid_t, libc::SIGINT) } == 0)
            .unwrap_or(false);

        if interrupted {
            tokio::select! {
                _ = child.wait() => return,
                _ = tokio::time::sleep(AGENT_STOP_GRACE) => {}
            }
        }
    }

    let _ = child.kill().await;
}

fn warn(shell: &Arc<Mutex<Shell>>, message: String) {
    _ = shell
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .warn(message);
}

/// Keeps the tunnel agent running until cancelled, restarting it whenever it
/// exits on its own.
pub fn supervise(
    shell: Arc<Mutex<Shell>>,
    options: TunnelAgentOptions,
    cancel: CancellationToken
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut command = agent_command(&options);

        _ = shell
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .status("Tunnel", "starting the agent");

        loop {
            let mut child = match command.spawn() {
                Ok(child) => child,
                Err(error) => {
                    warn(
                        &shell,
                        format!("could not start the tunnel agent: {}", error)
                    );
                    return;
                }
            };

            tokio::select! {
                _ = cancel.cancelled() => {
                    stop_agent(&mut child).await;
                    return;
                }
                status = child.wait() => {
                    let status = match status {
                        Ok(status) => status.to_string(),
                        Err(error) => error.to_string()
                    };

                    warn(
                        &shell,
                        format!(
                            "the tunnel agent exited ({}), restarting in {}s",
                            status,
                            AGENT_RESTART_DELAY.as_secs()
                        )
                    );
                }
            }

            tokio::select! {
                _ = cancel.cancelled() => return,
                _ = tokio::time::sleep(AGENT_RESTART_DELAY) => {}
            }
        }
    })
}
