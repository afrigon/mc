use std::path::PathBuf;

use clap::Args;

use crate::manifest::ManifestPaths;

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

impl ManifestArgs {
    pub fn with_lockfile(&self, lockfile: &LockfileArgs) -> ManifestPaths {
        ManifestPaths {
            manifest_path: self.manifest_path.clone(),
            lockfile_path: lockfile.lockfile_path.clone()
        }
    }
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
