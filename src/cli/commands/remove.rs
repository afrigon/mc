use clap::Args;

use crate::cli::CommandHandler;
use crate::cli::args::LockfileArgs;
use crate::cli::args::ManifestArgs;
use crate::context::McContext;
use crate::ops;
use crate::ops::mods::RemoveModsOptions;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct RemoveCommand {
    #[command(flatten)]
    pub manifest: ManifestArgs,

    #[command(flatten)]
    pub lockfile: LockfileArgs,

    /// Reference to a mod to remove
    #[arg(required = true, value_name = "MOD_SLUG")]
    pub mods: Vec<String>
}

impl CommandHandler for RemoveCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = RemoveModsOptions {
            mods: self.mods.clone(),
            paths: self.manifest.with_lockfile(&self.lockfile)
        };

        ops::mods::remove(context, &options).await?;

        Ok(())
    }
}
