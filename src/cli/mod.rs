pub mod commands;
pub mod globals;
pub mod styles;

use clap::ColorChoice;
use clap::Parser;

use crate::cli::commands::CliCommand;
use crate::cli::globals::GlobalOptions;
use crate::context::McContext;
use crate::utils::errors::CliResult;

#[derive(Parser)]
#[command(version, about, name = "mc", styles = styles::styles(), color = ColorChoice::Auto)]
pub struct Cli {
    #[command(flatten)]
    pub globals: GlobalOptions,

    #[command(subcommand)]
    pub command: CliCommand
}

pub trait CommandHandler {
    async fn handle(&self, context: &mut McContext) -> CliResult;
}
