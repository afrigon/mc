use clap::Args;
use clap::Subcommand;
use clap::value_parser;

use crate::cli::CommandHandler;
use crate::context::McContext;
use crate::minecraft::loader::LoaderKind;
use crate::ops;
use crate::ops::minecraft::MinecraftInstallOptions;
use crate::ops::minecraft::MinecraftListLoadersOptions;
use crate::ops::minecraft::MinecraftListOptions;
use crate::resolvers::loader::LoaderVersionResolver;
use crate::resolvers::minecraft::MinecraftVersionResolver;
use crate::utils::errors::CliResult;
use crate::utils::product_descriptor::RawProductDescriptor;
use crate::utils::product_descriptor::VersionResolver;

#[derive(Args)]
pub struct MinecraftCommand {
    #[command(subcommand)]
    pub command: MinecraftSubcommand
}

/// Manage minecraft versions
#[derive(Subcommand)]
pub enum MinecraftSubcommand {
    /// Install a specific Minecraft version
    Install(MinecraftInstallCommand),

    /// List all available Minecraft version
    List(MinecraftListCommand),

    /// List all available Minecraft loader versions
    ListLoaders(MinecraftListLoadersCommand)
}

#[derive(Args)]
pub struct MinecraftInstallCommand {
    /// Specify a minecraft version, defaults to latest
    #[arg(default_value = "latest")]
    pub version: String,

    /// Specify a mod loader, defaults to vanilla
    #[arg(short, long)]
    pub loader: Option<RawProductDescriptor>
}

impl CommandHandler for MinecraftInstallCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let version =
            MinecraftVersionResolver::resolve(context, Some(self.version.clone())).await?;

        let loader = match self.loader.clone() {
            Some(l) => LoaderVersionResolver::resolve_descriptor(context, l)
                .await
                .ok(),
            None => None
        };

        let minecraft_directory = context.cwd.join("minecraft"); // TODO: fix this path, use data path by default, also review other paths for cwd != project dir

        let options = MinecraftInstallOptions {
            version,
            loader,
            minecraft_directory
        };

        ops::minecraft::install(context, &options).await?;

        Ok(())
    }
}

#[derive(Args)]
pub struct MinecraftListCommand {
    /// show all available versions
    #[arg(long)]
    pub all: bool,

    /// show snapshot versions
    #[arg(short, long)]
    pub snapshots: bool,

    /// show beta versions
    #[arg(short, long)]
    pub betas: bool,

    /// show alpha versions
    #[arg(short, long)]
    pub alphas: bool
}

impl CommandHandler for MinecraftListCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = MinecraftListOptions {
            all: self.all,
            snapshots: self.snapshots,
            betas: self.betas,
            alphas: self.alphas
        };

        ops::minecraft::list(context, &options).await?;

        Ok(())
    }
}

#[derive(Args)]
pub struct MinecraftListLoadersCommand {
    /// List versions for a specific loader
    #[arg(short, long, value_parser = value_parser!(LoaderKind))]
    pub loader: LoaderKind,

    /// List loader versions for a specific game version
    #[arg(short, long, default_value = "latest")]
    pub minecraft_version: String,

    /// Limit the number of results
    #[arg(long, default_value_t = 10)]
    pub limit: usize
}

impl CommandHandler for MinecraftListLoadersCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let minecraft_version =
            MinecraftVersionResolver::resolve(context, Some(self.minecraft_version.clone()))
                .await?;

        let options = MinecraftListLoadersOptions {
            loader: self.loader,
            minecraft_version,
            limit: self.limit
        };

        ops::minecraft::list_loaders(context, &options).await?;

        Ok(())
    }
}
