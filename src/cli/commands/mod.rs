pub mod add;
pub mod backup;
pub mod init;
pub mod java;
pub mod minecraft;
pub mod remove;
pub mod restore;
pub mod run;
pub mod update;

use clap::Subcommand;

use crate::cli::commands::add::AddCommand;
use crate::cli::commands::backup::BackupCommand;
use crate::cli::commands::init::InitCommand;
use crate::cli::commands::java::JavaCommand;
use crate::cli::commands::minecraft::MinecraftCommand;
use crate::cli::commands::remove::RemoveCommand;
use crate::cli::commands::restore::RestoreCommand;
use crate::cli::commands::run::RunCommand;
use crate::cli::commands::update::UpdateCommand;

#[derive(Subcommand)]
pub enum CliCommand {
    Java(JavaCommand),

    Minecraft(MinecraftCommand),

    /// Create a new mc package in an existing directory
    Init(InitCommand),

    /// Run the Minecraft instance
    Run(RunCommand),

    /// Add mods to a manifest file
    Add(AddCommand),

    /// Remove mods from a manifest file
    Remove(RemoveCommand),

    /// Update all mods to their latest version for the configured Minecraft version
    Update(UpdateCommand),

    /// Start a backup of the world files
    Backup(BackupCommand),

    /// Restore a backup
    Restore(RestoreCommand)
}

// TODO: use refs + lifetime in the option structs to avoid cloning the cli args.
