use std::path::PathBuf;

use clap::Args;

use crate::cli::CommandHandler;
use crate::context::McContext;
use crate::ops;
use crate::ops::run::RunOptions;
use crate::utils::errors::CliError;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct RunCommand {
    /// Path to mc.kdl
    #[arg(
        long,
        default_value = "./mc.kdl",
        hide_default_value = true,
        value_name = "PATH"
    )]
    pub manifest_path: PathBuf,

    /// Path to mc.lock
    #[arg(
        long,
        default_value = "./mc.lock",
        hide_default_value = true,
        value_name = "PATH"
    )]
    pub lockfile_path: PathBuf
}

impl CommandHandler for RunCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = RunOptions {
            manifest_path: self.manifest_path.clone(),
            lockfile_path: self.lockfile_path.clone()
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
