use std::path::PathBuf;

use clap::Args;

use crate::cli::CommandHandler;
use crate::context::McContext;
use crate::ops;
use crate::ops::init::InitOptions;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct InitCommand {
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Set the resulting server name, defaults to the directory name
    #[arg(long)]
    pub name: Option<String>,

    /// Automatically agree to the Minecraft EULA (https://aka.ms/MinecraftEULA)
    #[arg(long, default_value_t = false)]
    pub eula: bool
}

impl CommandHandler for InitCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = InitOptions {
            path: context.cwd.join(&self.path),
            name: self.name.clone(),
            eula: self.eula
        };

        ops::init::init(context, &options).await?;

        Ok(())
    }
}
