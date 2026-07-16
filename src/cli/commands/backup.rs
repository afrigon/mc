use std::path::PathBuf;

use anyhow::Context;
use clap::Args;

use crate::cli::CommandHandler;
use crate::context::McContext;
use crate::manifest::Manifest;
use crate::minecraft::server_properties::ServerProperties;
use crate::ops;
use crate::ops::backups::BackupOptions;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct BackupCommand {
    /// Path to mc.toml
    #[arg(
        long,
        default_value = "./mc.toml",
        hide_default_value = true,
        value_name = "PATH"
    )]
    pub manifest_path: PathBuf
}

impl CommandHandler for BackupCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let manifest_string = tokio::fs::read_to_string(&self.manifest_path)
            .await
            .context("could not find mc.toml file")?;
        let manifest = toml::from_str::<Manifest>(&manifest_string)
            .map_err(|_| anyhow::anyhow!("could not parse manifest file"))?;

        let instance_path = context.cwd.join("instance");
        let rcon_password = ServerProperties::read_rcon_password(&instance_path).await?;

        let options = BackupOptions {
            rcon_port: manifest.server.rcon_port,
            rcon_password,
            storage: manifest.backups.effective_storage(),
            world_path: instance_path.join(&manifest.name),
            project_path: context.cwd.clone(),
            notifier: manifest.notifier(context)
        };

        ops::backups::backup(&options).await?;

        Ok(())
    }
}
