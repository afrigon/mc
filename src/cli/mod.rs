pub mod commands;
pub mod context;
pub mod globals;
pub mod styles;

use clap::{ColorChoice, Parser};

use crate::{
    cli::{commands::CliCommand, context::CliContext, globals::GlobalOptions},
    utils::errors::CliResult,
};

#[derive(Parser)]
#[command(version, about, name = "mc", styles = styles::styles(), color = ColorChoice::Auto)]
pub struct Cli {
    #[command(flatten)]
    pub globals: GlobalOptions,

    #[command(subcommand)]
    pub command: CliCommand,
}

pub trait CommandHandler {
    async fn handle(&self, context: &mut CliContext) -> CliResult;
}
