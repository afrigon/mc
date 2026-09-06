use clap::Args;

use crate::cli::CommandHandler;
use crate::cli::args::ManifestArgs;
use crate::context::McContext;
use crate::ops;
use crate::ops::mods::AddModsOptions;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct AddCommand {
    #[command(flatten)]
    pub manifest: ManifestArgs,

    /// Reference to a mod to add
    #[arg(required = true, value_name = "MOD_SLUG")]
    pub mods: Vec<String>
}

impl CommandHandler for AddCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = AddModsOptions {
            mods: self.mods.clone(),
            manifest_path: self.manifest.manifest_path.clone()
        };

        ops::mods::add(context, &options).await?;

        Ok(())
    }
}
