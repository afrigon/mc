pub mod init;
pub mod java;
pub mod minecraft;
pub mod run;

use clap::Subcommand;

use crate::cli::commands::{
    init::InitCommand, java::JavaCommand, minecraft::MinecraftCommand, run::RunCommand,
};

#[derive(Subcommand)]
pub enum CliCommand {
    Java(JavaCommand),

    Minecraft(MinecraftCommand),

    /// Create a new mc package in an existing directory
    Init(InitCommand),

    /// Run the Minecraft server
    Run(RunCommand),
}
