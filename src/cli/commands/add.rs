use std::path::PathBuf;

use clap::Args;

use crate::cli::CommandHandler;
use crate::context::McContext;
use crate::ops;
use crate::ops::mods::AddModsOptions;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct AddCommand {
    /// Path to mc.toml
    #[arg(
        long,
        default_value = "./mc.toml",
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
    pub lockfile_path: PathBuf,

    /// Reference to a mod to add
    #[arg(required = true, value_name = "MOD_ID")]
    pub mods: Vec<String>
}

impl CommandHandler for AddCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = AddModsOptions {
            mods: self.mods.clone(),
            manifest_path: self.manifest_path.clone(),
            lockfile_path: self.lockfile_path.clone()
        };

        ops::mods::add(context, &options).await?;

        Ok(())
    }
}
