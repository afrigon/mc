mod cli;
mod context;
mod crypto;
mod env;
mod java;
mod manifest;
mod minecraft;
mod mods;
mod network;
mod ops;
mod resolvers;
mod services;
mod utils;

use std::process::exit;

use clap::Parser;
use tracing::debug;

use crate::cli::Cli;
use crate::cli::CommandHandler;
use crate::cli::commands::CliCommand;
use crate::cli::commands::java::JavaSubcommand;
use crate::cli::commands::minecraft::MinecraftSubcommand;
use crate::context::McContext;
use crate::utils::errors::CliError;
use crate::utils::errors::CliResult;
use crate::utils::errors::InternalError;
use crate::utils::shell::Shell;
use crate::utils::verbosity::Verbosity;

#[tokio::main]
async fn main() {
    let mut context = match McContext::default() {
        Ok(context) => context,
        Err(e) => {
            let mut shell = Shell::new();

            exit_with_error(e.into(), &mut shell)
        }
    };

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => exit_with_error(e.into(), &mut context.shell())
    };

    let verbose = cli.globals.verbose;
    let quiet = cli.globals.quiet;

    let verbosity = if quiet {
        Verbosity::Quiet
    } else {
        match verbose {
            0 => Verbosity::Regular,
            1 => Verbosity::Verbose,
            _ => Verbosity::VeryVerbose
        }
    };
    context.shell().set_verbosity(verbosity);

    let color_choice = cli.globals.color;
    context.shell().set_color_choice(color_choice);

    // TODO: add lockfile checks here to make sure we're not trying to edit / backup the server while its running... or various other edge cases.
    match run(&cli, &mut context).await {
        Err(e) => exit_with_error(e, &mut context.shell()),
        Ok(()) => {}
    };
}

async fn run(cli: &Cli, context: &mut McContext) -> CliResult {
    match &cli.command {
        CliCommand::Init(command) => command.handle(context).await,
        CliCommand::Run(command) => command.handle(context).await,
        CliCommand::Minecraft(command) => match &command.command {
            MinecraftSubcommand::Install(command) => command.handle(context).await,
            MinecraftSubcommand::List(command) => command.handle(context).await,
            MinecraftSubcommand::ListLoaders(command) => command.handle(context).await
        },
        CliCommand::Java(command) => match &command.command {
            JavaSubcommand::Install(command) => command.handle(context).await,
            JavaSubcommand::List(command) => command.handle(context).await
        }
    }
}

fn exit_with_error(error: CliError, shell: &mut Shell) -> ! {
    debug!("exit_with_error; error={:?}", error);

    if let Some(ref err) = error.error {
        if let Some(clap_err) = err.downcast_ref::<clap::Error>() {
            let exit_code = if clap_err.use_stderr() { 1 } else { 0 };
            let _ = clap_err.print();

            exit(exit_code)
        }
    }

    let CliError { error, exit_code } = error;

    if let Some(error) = error {
        for (i, error) in error.chain().enumerate() {
            if i == 0 {
                drop(shell.error(&error));
            } else {
                let lines: String = error
                    .to_string()
                    .lines()
                    .map(|line| {
                        if line.is_empty() {
                            String::from("\n")
                        } else {
                            format!("  {}\n", line)
                        }
                    })
                    .collect();

                drop(writeln!(shell.err(), "\nCaused by:"));
                drop(writeln!(shell.err(), "{}", lines));
            }
        }

        if error
            .chain()
            .any(|e| e.downcast_ref::<InternalError>().is_some())
        {
            drop(shell.note("this is an unexpected mc internal error"));

            drop(
                shell.note("you can submit bug reports at: https://github.com/afrigon/mc/issues/")
            );

            drop(shell.note(format!(
                "{} {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            )));
        }
    }

    exit(exit_code)
}
