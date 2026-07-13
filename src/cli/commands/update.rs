use std::path::PathBuf;

use clap::Args;

use crate::cli::CommandHandler;
use crate::context::McContext;
use crate::ops;
use crate::ops::mods::UpdateModsOptions;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct UpdateCommand {
    /// Path to mc.toml
    #[arg(
        long,
        default_value = "./mc.toml",
        hide_default_value = true,
        value_name = "PATH"
    )]
    pub manifest_path: PathBuf
}

impl CommandHandler for UpdateCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = UpdateModsOptions {
            manifest_path: self.manifest_path.clone()
        };

        ops::mods::update(context, &options).await?;

        Ok(())
    }
}
