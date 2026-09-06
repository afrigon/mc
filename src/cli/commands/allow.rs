use clap::Args;
use clap::Subcommand;

use crate::cli::CommandHandler;
use crate::cli::args::LockfileArgs;
use crate::cli::args::ManifestArgs;
use crate::context::McContext;
use crate::ops;
use crate::ops::players::PlayerListOptions;
use crate::ops::players::allow::AllowAddOptions;
use crate::ops::players::allow::AllowRemoveOptions;
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
pub struct AllowAddCommand {
    #[command(flatten)]
    pub manifest: ManifestArgs,

    #[command(flatten)]
    pub lockfile: LockfileArgs,

    /// Player names to allow
    #[arg(required = true, value_name = "NAME")]
    pub names: Vec<String>
}

impl CommandHandler for AllowAddCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = AllowAddOptions {
            names: self.names.clone(),
            paths: self.manifest.with_lockfile(&self.lockfile)
        };

        ops::players::allow::add(context, &options).await?;

        Ok(())
    }
}

#[derive(Args)]
pub struct AllowRemoveCommand {
    #[command(flatten)]
    pub manifest: ManifestArgs,

    #[command(flatten)]
    pub lockfile: LockfileArgs,

    /// Player names to remove from the allow list
    #[arg(required = true, value_name = "NAME")]
    pub names: Vec<String>
}

impl CommandHandler for AllowRemoveCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = AllowRemoveOptions {
            names: self.names.clone(),
            paths: self.manifest.with_lockfile(&self.lockfile)
        };

        ops::players::allow::remove(context, &options).await?;

        Ok(())
    }
}

#[derive(Args)]
pub struct AllowListCommand {
    #[command(flatten)]
    pub manifest: ManifestArgs,

    #[command(flatten)]
    pub lockfile: LockfileArgs
}

impl CommandHandler for AllowListCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = PlayerListOptions {
            paths: self.manifest.with_lockfile(&self.lockfile)
        };

        ops::players::allow::list(context, &options).await?;

        Ok(())
    }
}
