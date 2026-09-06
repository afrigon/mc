use std::path::PathBuf;

use clap::Args;
use clap::Subcommand;

use crate::cli::CommandHandler;
use crate::context::McContext;
use crate::ops;
use crate::ops::players::AllowAddOptions;
use crate::ops::players::AllowRemoveOptions;
use crate::ops::players::PlayerListOptions;
use crate::ops::players::PlayerPaths;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct AllowCommand {
    #[command(subcommand)]
    pub command: AllowSubcommand
}

/// Manage the allow list
#[derive(Subcommand)]
pub enum AllowSubcommand {
    /// Allow players to join
    Add(AllowAddCommand),

    /// Stop allowing players to join
    Remove(AllowRemoveCommand),

    /// List the allowed players
    List(AllowListCommand)
}

#[derive(Args)]
pub struct PlayerPathArgs {
    /// Path to mc.kdl
    #[arg(
        long,
        default_value = "./mc.kdl",
        hide_default_value = true,
        value_name = "PATH"
    )]
    pub manifest_path: PathBuf,

    /// Path to mc.lock
    #[arg(
        long,
        default_value = "./mc.lock",
        hide_default_value = true,
        value_name = "PATH"
    )]
    pub lockfile_path: PathBuf
}

impl PlayerPathArgs {
    pub fn paths(&self) -> PlayerPaths {
        PlayerPaths {
            manifest_path: self.manifest_path.clone(),
            lockfile_path: self.lockfile_path.clone()
        }
    }
}

#[derive(Args)]
pub struct AllowAddCommand {
    #[command(flatten)]
    pub paths: PlayerPathArgs,

    /// Player names to allow
    #[arg(required = true, value_name = "NAME")]
    pub names: Vec<String>
}

impl CommandHandler for AllowAddCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = AllowAddOptions {
            names: self.names.clone(),
            paths: self.paths.paths()
        };

        ops::players::allow_add(context, &options).await?;

        Ok(())
    }
}

#[derive(Args)]
pub struct AllowRemoveCommand {
    #[command(flatten)]
    pub paths: PlayerPathArgs,

    /// Player names to remove from the allow list
    #[arg(required = true, value_name = "NAME")]
    pub names: Vec<String>
}

impl CommandHandler for AllowRemoveCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = AllowRemoveOptions {
            names: self.names.clone(),
            paths: self.paths.paths()
        };

        ops::players::allow_remove(context, &options).await?;

        Ok(())
    }
}

#[derive(Args)]
pub struct AllowListCommand {
    #[command(flatten)]
    pub paths: PlayerPathArgs
}

impl CommandHandler for AllowListCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = PlayerListOptions {
            paths: self.paths.paths()
        };

        ops::players::allow_list(context, &options).await?;

        Ok(())
    }
}
