use clap::Args;

use crate::cli::CommandHandler;
use crate::cli::args::LockfileArgs;
use crate::cli::args::ManifestArgs;
use crate::context::McContext;
use crate::ops;
use crate::ops::run::RunOptions;
use crate::utils::errors::CliError;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct RunCommand {
    #[command(flatten)]
    pub manifest: ManifestArgs,

    #[command(flatten)]
    pub lockfile: LockfileArgs,

    /// Show the Minecraft server console output
    #[arg(long)]
    pub server_logs: bool,

    /// Show the tunnel agent output
    #[arg(long)]
    pub tunnel_logs: bool
}

impl CommandHandler for RunCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = RunOptions {
            paths: self.manifest.with_lockfile(&self.lockfile),
            server_logs: self.server_logs,
            tunnel_logs: self.tunnel_logs
        };

        let exit_status = ops::run::run(context, &options).await?;

        if let Some(status) = exit_status {
            if !status.success() {
                return Err(CliError::new(
                    anyhow::anyhow!("the minecraft server exited unexpectedly ({})", status),
                    status.code().unwrap_or(101)
                ));
            }
        }

        Ok(())
    }
}
