use clap::Args;

use crate::cli::CommandHandler;
use crate::cli::args::ManifestArgs;
use crate::context::McContext;
use crate::ops;
use crate::ops::mods::UpdateModsOptions;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct UpdateCommand {
    #[command(flatten)]
    pub manifest: ManifestArgs,

    /// Mods to update; updates all mods when omitted
    #[arg(value_name = "MOD_SLUG")]
    pub mods: Vec<String>
}

impl CommandHandler for UpdateCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = UpdateModsOptions {
            mods: self.mods.clone(),
            manifest_path: self.manifest.manifest_path.clone()
        };

        ops::mods::update(context, &options).await?;

        Ok(())
    }
}
