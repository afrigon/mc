use clap::Args;
use std::path::PathBuf;

use crate::cli::CommandHandler;
use crate::cli::context::CliContext;
use crate::ops;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct RunCommand {
    /// Path to mc.toml
    #[arg(
        long,
        default_value = "./mc.toml",
        hide_default_value = true,
        value_name = "PATH"
    )]
    pub manifest_path: PathBuf,
}

impl CommandHandler for RunCommand {
    async fn handle(&self, context: &mut CliContext) -> CliResult {
        ops::run(context, self).await?;
        Ok(())
    }
}
