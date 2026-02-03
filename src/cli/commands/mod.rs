pub mod init;
pub mod java;
pub mod minecraft;
pub mod run;

use clap::Subcommand;

use crate::cli::commands::init::InitCommand;
use crate::cli::commands::java::JavaCommand;
use crate::cli::commands::minecraft::MinecraftCommand;
use crate::cli::commands::run::RunCommand;

#[derive(Subcommand)]
pub enum CliCommand {
    Java(JavaCommand),

    Minecraft(MinecraftCommand),

    /// Create a new mc package in an existing directory
    Init(InitCommand),

    /// Run the Minecraft instance
    Run(RunCommand)
}
