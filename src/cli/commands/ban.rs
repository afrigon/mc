use std::net::IpAddr;

use chrono::DateTime;
use chrono::Utc;
use clap::Args;
use clap::Subcommand;

use crate::cli::CommandHandler;
use crate::cli::args::LockfileArgs;
use crate::cli::args::ManifestArgs;
use crate::context::McContext;
use crate::ops;
use crate::ops::players::PlayerListOptions;
use crate::ops::players::ban::BanAddOptions;
use crate::ops::players::ban::BanRemoveOptions;
use crate::utils::errors::CliError;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct BanCommand {
    #[command(subcommand)]
    pub command: BanSubcommand
}

/// Manage the ban list
#[derive(Subcommand)]
pub enum BanSubcommand {
    /// Ban players or addresses
    Add(BanAddCommand),

    /// Lift bans on players or addresses
    Remove(BanRemoveCommand),

    /// List the banned players and addresses
    List(BanListCommand)
}

#[derive(Args)]
pub struct BanAddCommand {
    #[command(flatten)]
    pub manifest: ManifestArgs,

    #[command(flatten)]
    pub lockfile: LockfileArgs,

    /// Player names to ban
    #[arg(value_name = "NAME", required_unless_present = "addresses")]
    pub names: Vec<String>,

    /// Addresses to ban instead of players
    #[arg(long = "ip", value_name = "ADDRESS")]
    pub addresses: Vec<IpAddr>,

    /// Why the ban was issued, shown to the player
    #[arg(long, value_name = "TEXT")]
    pub reason: Option<String>,

    /// Lift the ban at this RFC 3339 date, such as 2026-12-31T00:00:00Z
    #[arg(long, value_name = "DATE", conflicts_with = "duration")]
    pub until: Option<DateTime<Utc>>,

    /// Lift the ban after this long, such as 7d, 12h, or 30m
    #[arg(long = "for", value_name = "DURATION")]
    pub duration: Option<humantime::Duration>
}

impl CommandHandler for BanAddCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let expires = match (self.until, self.duration) {
            (Some(until), _) => Some(until),
            (None, Some(duration)) => {
                let duration = chrono::Duration::from_std(*duration)
                    .map_err(|_| CliError::from(anyhow::anyhow!("the ban duration is too long")))?;

                Some(Utc::now() + duration)
            }
            (None, None) => None
        };

        let options = BanAddOptions {
            names: self.names.clone(),
            addresses: self.addresses.clone(),
            reason: self.reason.clone(),
            expires,
            paths: self.manifest.with_lockfile(&self.lockfile)
        };

        ops::players::ban::add(context, &options).await?;

        Ok(())
    }
}

#[derive(Args)]
pub struct BanRemoveCommand {
    #[command(flatten)]
    pub manifest: ManifestArgs,

    #[command(flatten)]
    pub lockfile: LockfileArgs,

    /// Player names to unban
    #[arg(value_name = "NAME", required_unless_present = "addresses")]
    pub names: Vec<String>,

    /// Addresses to unban instead of players
    #[arg(long = "ip", value_name = "ADDRESS")]
    pub addresses: Vec<IpAddr>
}

impl CommandHandler for BanRemoveCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = BanRemoveOptions {
            names: self.names.clone(),
            addresses: self.addresses.clone(),
            paths: self.manifest.with_lockfile(&self.lockfile)
        };

        ops::players::ban::remove(context, &options).await?;

        Ok(())
    }
}

#[derive(Args)]
pub struct BanListCommand {
    #[command(flatten)]
    pub manifest: ManifestArgs,

    #[command(flatten)]
    pub lockfile: LockfileArgs
}

impl CommandHandler for BanListCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = PlayerListOptions {
            paths: self.manifest.with_lockfile(&self.lockfile)
        };

        ops::players::ban::list(context, &options).await?;

        Ok(())
    }
}
