use clap::Args;
use clap::Subcommand;
use clap::value_parser;

use crate::cli::CommandHandler;
use crate::context::McContext;
use crate::env::Architecture;
use crate::env::Platform;
use crate::ops;
use crate::ops::tunnel::TunnelClaimOptions;
use crate::ops::tunnel::TunnelInstallOptions;
use crate::ops::tunnel::TunnelListOptions;
use crate::resolvers::tunnel::TunnelVersionResolver;
use crate::utils::errors::CliResult;
use crate::utils::product_descriptor::RawProductDescriptor;
use crate::utils::product_descriptor::VersionResolver;

#[derive(Args)]
pub struct TunnelCommand {
    #[command(subcommand)]
    pub command: TunnelSubcommand
}

/// Manage tunnel providers
#[derive(Subcommand)]
pub enum TunnelSubcommand {
    /// Install a specific tunnel agent version
    Install(TunnelInstallCommand),

    /// List available tunnel agent versions
    List(TunnelListCommand),

    /// Link this instance's tunnel agent to a playit.gg account
    Claim(TunnelClaimCommand)
}

#[derive(Args)]
pub struct TunnelInstallCommand {
    /// Provider to install
    #[arg(value_parser = value_parser!(RawProductDescriptor), default_value = "playit")]
    pub provider: RawProductDescriptor,

    /// Select a specific platform
    #[arg(short, long, value_parser = value_parser!(Platform))]
    pub platform: Option<Platform>,

    /// Select a specific architecture
    #[arg(short, long, value_parser = value_parser!(Architecture))]
    pub architecture: Option<Architecture>
}

impl CommandHandler for TunnelInstallCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let version = TunnelVersionResolver::resolve_descriptor(context, &self.provider).await?;
        let platform = self.platform.unwrap_or_else(|| Platform::current());
        let architecture = self.architecture.unwrap_or_else(|| Architecture::current());
        let tunnel_directory = context.cwd.join(".tunnel");

        let options = TunnelInstallOptions {
            version,
            platform,
            architecture,
            tunnel_directory,
            staging_directory: context.cwd.join("temp")
        };

        ops::tunnel::install(context, &options).await?;

        Ok(())
    }
}

#[derive(Args)]
pub struct TunnelListCommand {
    /// Limit the number of results
    #[arg(long, default_value_t = 10)]
    pub limit: usize
}

impl CommandHandler for TunnelListCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = TunnelListOptions { limit: self.limit };

        ops::tunnel::list(context, &options).await?;

        Ok(())
    }
}

#[derive(Args)]
pub struct TunnelClaimCommand {
    /// Replace an existing tunnel agent secret
    #[arg(short, long)]
    pub force: bool
}

impl CommandHandler for TunnelClaimCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let tunnel_directory = context.cwd.join(".tunnel");

        let options = TunnelClaimOptions {
            secret_path: ops::tunnel::secret_path(&tunnel_directory),
            force: self.force
        };

        ops::tunnel::claim(context, &options).await?;

        Ok(())
    }
}
