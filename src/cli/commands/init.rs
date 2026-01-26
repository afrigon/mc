use clap::Args;
use std::path::PathBuf;

use crate::{
    cli::{CommandHandler, context::CliContext},
    ops,
    utils::errors::CliResult,
};

#[derive(Args)]
pub struct InitCommand {
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Set the resulting server name, defaults to the directory name
    #[arg(long)]
    pub name: Option<String>,

    /// Automatically agree to the Minecraft EULA (https://aka.ms/MinecraftEULA)
    #[arg(long, default_value_t = false)]
    pub eula: bool,
}

impl CommandHandler for InitCommand {
    async fn handle(&self, context: &mut CliContext) -> CliResult {
        ops::init(context, self).await?;
        Ok(())
    }
}
