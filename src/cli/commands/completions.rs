use anyhow::Context;
use clap::Args;
use clap::CommandFactory;
use clap_complete::Generator;
use clap_complete::Shell;

use crate::cli::Cli;
use crate::cli::CommandHandler;
use crate::context::McContext;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct CompletionsCommand {
    /// Shell to generate a completion script for
    #[arg(value_enum, value_name = "SHELL")]
    pub shell: Shell
}

impl CommandHandler for CompletionsCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let mut command = Cli::command();
        command.set_bin_name("mc");
        command.build();

        let mut shell = context.shell();

        self.shell
            .try_generate(&command, shell.out())
            .context("could not write the completion script")?;

        Ok(())
    }
}
