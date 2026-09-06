use clap::Args;
use clap::Subcommand;

use crate::cli::CommandHandler;
use crate::cli::commands::allow::PlayerPathArgs;
use crate::context::McContext;
use crate::minecraft::MinecraftPermission;
use crate::ops;
use crate::ops::players::OpAddOptions;
use crate::ops::players::OpRemoveOptions;
use crate::ops::players::PlayerListOptions;
use crate::utils::errors::CliError;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct OpCommand {
    #[command(subcommand)]
    pub command: OpSubcommand
}

/// Manage the server operators
#[derive(Subcommand)]
pub enum OpSubcommand {
    /// Make players operators, or change their settings
    Add(OpAddCommand),

    /// Remove operators
    Remove(OpRemoveCommand),

    /// List the operators
    List(OpListCommand)
}

#[derive(Args)]
pub struct OpAddCommand {
    #[command(flatten)]
    pub paths: PlayerPathArgs,

    /// Player names to make operators
    #[arg(required = true, value_name = "NAME")]
    pub names: Vec<String>,

    /// Permission level from 1 to 4; defaults to the server's op-permission-level
    #[arg(long, value_name = "LEVEL", value_parser = clap::value_parser!(u8).range(1..=4))]
    pub level: Option<u8>,

    /// Let the operators join even when the server is full
    #[arg(long)]
    pub bypass_player_limit: bool
}

impl CommandHandler for OpAddCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let level = self
            .level
            .map(MinecraftPermission::try_from)
            .transpose()
            .map_err(CliError::from)?;

        let options = OpAddOptions {
            names: self.names.clone(),
            level,
            bypasses_player_limit: self.bypass_player_limit,
            paths: self.paths.paths()
        };

        ops::players::op_add(context, &options).await?;

        Ok(())
    }
}

#[derive(Args)]
pub struct OpRemoveCommand {
    #[command(flatten)]
    pub paths: PlayerPathArgs,

    /// Player names to remove from the operators
    #[arg(required = true, value_name = "NAME")]
    pub names: Vec<String>
}

impl CommandHandler for OpRemoveCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = OpRemoveOptions {
            names: self.names.clone(),
            paths: self.paths.paths()
        };

        ops::players::op_remove(context, &options).await?;

        Ok(())
    }
}

#[derive(Args)]
pub struct OpListCommand {
    #[command(flatten)]
    pub paths: PlayerPathArgs
}

impl CommandHandler for OpListCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = PlayerListOptions {
            paths: self.paths.paths()
        };

        ops::players::op_list(context, &options).await?;

        Ok(())
    }
}
