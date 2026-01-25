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
}

impl CommandHandler for InitCommand {
    async fn handle(&self, context: &mut CliContext) -> CliResult {
        ops::init(context, self).await?;
        Ok(())
    }
}
