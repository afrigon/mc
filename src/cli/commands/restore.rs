use anyhow::Context;
use clap::Args;
use tokio_util::sync::CancellationToken;

use crate::cli::CommandHandler;
use crate::cli::args::ManifestArgs;
use crate::context::McContext;
use crate::manifest::Manifest;
use crate::ops;
use crate::ops::backups::ListOptions;
use crate::ops::backups::RestoreOptions;
use crate::ops::backups::UndoRestoreOptions;
use crate::utils::errors::CliResult;
use crate::utils::process::ShutdownSignals;

#[derive(Args)]
pub struct RestoreCommand {
    #[command(flatten)]
    pub manifest: ManifestArgs,

    /// Filename of the backup to restore (defaults to the most recent backup)
    #[arg(long, value_name = "BACKUP")]
    pub backup: Option<String>,

    /// List the available backups instead of restoring
    #[arg(long, conflicts_with = "backup")]
    pub list: bool,

    /// Put the world set aside by the last restore back in place, swapping it
    /// with the current world
    #[arg(long, conflicts_with_all = ["backup", "list"])]
    pub undo: bool
}

impl CommandHandler for RestoreCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let manifest_string = tokio::fs::read_to_string(&self.manifest.manifest_path)
            .await
            .context("could not find mc.kdl file")?;
        let manifest = Manifest::from_kdl_str(&manifest_string)
            .map_err(|_| anyhow::anyhow!("could not parse manifest file"))?;

        if self.list {
            let options = ListOptions {
                storage: manifest.backups.effective_storage(),
                world_name: manifest.name.clone()
            };

            ops::backups::list(context, &options).await?;

            return Ok(());
        }

        if self.undo {
            let options = UndoRestoreOptions {
                world_path: context.cwd.join("instance").join(&manifest.name),
                project_path: context.cwd.clone()
            };

            ops::backups::undo_restore(context, &options).await?;

            return Ok(());
        }

        let cancel = CancellationToken::new();
        let mut signals = ShutdownSignals::register()?;

        {
            let cancel = cancel.clone();

            tokio::spawn(async move {
                signals.recv().await;
                cancel.cancel();
            });
        }

        let options = RestoreOptions {
            storage: manifest.backups.effective_storage(),
            world_path: context.cwd.join("instance").join(&manifest.name),
            project_path: context.cwd.clone(),
            version: self.backup.clone(),
            cancel
        };

        ops::backups::restore(context, &options).await?;

        Ok(())
    }
}
