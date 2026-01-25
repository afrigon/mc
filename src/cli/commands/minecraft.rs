use clap::{Args, Subcommand, value_parser};

use crate::cli::CommandHandler;
use crate::cli::context::CliContext;
use crate::minecraft::MinecraftVersion;
use crate::ops;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct MinecraftCommand {
    #[command(subcommand)]
    pub command: MinecraftSubcommand,
}

/// Manage minecraft versions
#[derive(Subcommand)]
pub enum MinecraftSubcommand {
    Install(MinecraftInstallCommand),
    List(MinecraftListCommand),
}

#[derive(Args)]
pub struct MinecraftInstallCommand {
    #[arg(value_parser = value_parser!(MinecraftVersion))]
    pub version: MinecraftVersion,
}

impl CommandHandler for MinecraftInstallCommand {
    async fn handle(&self, context: &mut CliContext) -> CliResult {
        ops::minecraft::install(context, self).await?;
        Ok(())
    }
}

#[derive(Args)]
pub struct MinecraftListCommand {
    /// show all available versions
    #[arg(long)]
    pub all: bool,

    /// show snapshot versions
    #[arg(short, long)]
    pub snapshots: bool,

    /// show beta versions
    #[arg(short, long)]
    pub betas: bool,

    /// show alpha versions
    #[arg(short, long)]
    pub alphas: bool,
}

impl CommandHandler for MinecraftListCommand {
    async fn handle(&self, context: &mut CliContext) -> CliResult {
        ops::minecraft::list(context, self).await?;
        Ok(())
    }
}
