use std::path::PathBuf;

use clap::Args;

#[derive(Args)]
pub struct ManifestArgs {
    /// Path to mc.kdl
    #[arg(
        long,
        default_value = "./mc.kdl",
        hide_default_value = true,
        value_name = "PATH"
    )]
    pub manifest_path: PathBuf
}

#[derive(Args)]
pub struct LockfileArgs {
    /// Path to mc.lock
    #[arg(
        long,
        default_value = "./mc.lock",
        hide_default_value = true,
        value_name = "PATH"
    )]
    pub lockfile_path: PathBuf
}
