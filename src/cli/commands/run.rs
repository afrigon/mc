use std::path::PathBuf;

use clap::Args;

use crate::cli::CommandHandler;
use crate::context::McContext;
use crate::ops;
use crate::ops::run::RunOptions;
use crate::utils::errors::CliResult;

#[derive(Args)]
pub struct RunCommand {
    /// Path to mc.toml
    #[arg(
        long,
        default_value = "./mc.toml",
        hide_default_value = true,
        value_name = "PATH"
    )]
    pub manifest_path: PathBuf, // TODO: add ip, port and rcon port settings here

    /// TCP port to bind the server to
    #[arg(short, long, default_value_t = 25565)]
    pub port: u16,

    /// TCP port to bind the RCON service to
    #[arg(short, long, default_value_t = 25575)]
    pub rcon_port: u16 // TODO: make sure version >= Java Edition Beta 1.9 Prerelease 4 before using rcon
}

impl CommandHandler for RunCommand {
    async fn handle(&self, context: &mut McContext) -> CliResult {
        let options = RunOptions {
            manifest_path: context.cwd.join(&self.manifest_path)
        };

        ops::run::run(context, &options).await?;

        Ok(())
    }
}
