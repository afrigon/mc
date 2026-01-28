use clap::Args;
use clap::Subcommand;
use clap::value_parser;

use crate::cli::CommandHandler;
use crate::context::McContext;
use crate::env::Architecture;
use crate::env::Platform;
use crate::ops;
use crate::ops::java::JavaInstallOptions;
use crate::ops::java::JavaListOptions;
use crate::resolvers::java::JavaVersionResolver;
use crate::utils::errors::CliResult;
use crate::utils::product_descriptor::RawProductDescriptor;
use crate::utils::product_descriptor::VersionResolver;

#[derive(Args)]
pub struct JavaCommand {
    #[command(subcommand)]
    pub command: JavaSubcommand
}

/// Manage Java versions
#[derive(Subcommand)]
pub enum JavaSubcommand {
    /// Install a specific Java version
    Install(JavaInstallCommand),

    /// List all available Java versions
    List(JavaListCommand)
}

#[derive(Args)]
pub struct JavaInstallCommand {
    /// Version to install
    #[arg(value_parser = value_parser!(RawProductDescriptor))]
    pub version: RawProductDescriptor,

    /// Select a specific platform
    #[arg(short, long, value_parser = value_parser!(Platform))]
    pub platform: Option<Platform>,

    /// Select a specific architecture
    #[arg(short, long, value_parser = value_parser!(Architecture))]
    pub architecture: Option<Architecture>
}

impl CommandHandler for JavaInstallCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let version =
            JavaVersionResolver::resolve_descriptor(context, self.version.clone()).await?;
        let platform = self.platform.unwrap_or_else(|| Platform::current());
        let architecture = self.architecture.unwrap_or_else(|| Architecture::current());
        let java_directory = context.cwd.join("java"); // TODO: fix this path

        let options = JavaInstallOptions {
            version,
            platform,
            architecture,
            java_directory
        };

        ops::java::install(context, &options).await?;

        Ok(())
    }
}

#[derive(Args)]
pub struct JavaListCommand {}

impl CommandHandler for JavaListCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = JavaListOptions {};

        ops::java::list(context, &options).await?;

        Ok(())
    }
}
