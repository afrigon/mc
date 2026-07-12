use std::path::PathBuf;

use anyhow::Context;
use clap::Args;

use crate::cli::CommandHandler;
use crate::context::McContext;
use crate::manifest::Manifest;
use crate::ops;
use crate::ops::backups::RestoreOptions;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct RestoreCommand {
    /// Path to mc.toml
    #[arg(
        long,
        default_value = "./mc.toml",
        hide_default_value = true,
        value_name = "PATH"
    )]
    pub manifest_path: PathBuf,

    /// Filename of the backup to restore (defaults to the most recent backup)
    #[arg(long, value_name = "BACKUP")]
    pub backup: Option<String>
}

impl CommandHandler for RestoreCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let manifest_string = tokio::fs::read_to_string(&self.manifest_path)
            .await
            .context("could not find mc.toml file")?;
        let manifest = toml::from_str::<Manifest>(&manifest_string)
            .map_err(|_| anyhow::anyhow!("could not parse manifest file"))?;

        if !manifest.backups.enabled {
            return Err(anyhow::anyhow!("backups are disabled for this instance").into());
        }

        // TODO: create a backup before restoring, if not empty

        let options = RestoreOptions {
            storage: manifest.backups.effective_storage(),
            world_path: context.cwd.join("instance").join(&manifest.name),
            project_path: context.cwd.clone(),
            version: self.backup.clone()
        };

        ops::backups::restore(context, &options).await?;

        Ok(())
    }
}
