use clap::{Args, Subcommand, value_parser};

use crate::{
    cli::{CommandHandler, context::CliContext},
    env::{Architecture, Platform},
    java::JavaDistribution,
    ops,
    utils::errors::CliResult,
};

#[derive(Args)]
pub struct JavaCommand {
    #[command(subcommand)]
    pub command: JavaSubcommand,
}

/// Manage Java versions
#[derive(Subcommand)]
pub enum JavaSubcommand {
    Install(JavaInstallCommand),
    List(JavaListCommand),
}

#[derive(Args)]
pub struct JavaInstallCommand {
    /// Version to install
    #[arg(value_parser = value_parser!(JavaDistribution))]
    pub version: JavaDistribution,

    /// Select a specific platform
    #[arg(short, long, value_parser = value_parser!(Platform))]
    pub platform: Option<Platform>,

    /// Select a specific architecture
    #[arg(short, long, value_parser = value_parser!(Architecture))]
    pub architecture: Option<Architecture>,
}

impl CommandHandler for JavaInstallCommand {
    async fn handle(&self, context: &mut CliContext) -> CliResult {
        ops::java::install(context, self).await?;
        Ok(())
    }
}

#[derive(Args)]
pub struct JavaListCommand {}

impl CommandHandler for JavaListCommand {
    async fn handle(&self, context: &mut CliContext) -> CliResult {
        ops::java::list(context, self).await?;
        Ok(())
    }
}
