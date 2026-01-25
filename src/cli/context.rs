use anyhow::Context;
use std::{
    env,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use crate::utils::shell::Shell;

pub struct CliContext {
    shell: Mutex<Shell>,
    pub cwd: PathBuf,
    pub http_client: reqwest::Client,
}

impl CliContext {
    pub fn new(shell: Shell, cwd: PathBuf) -> CliContext {
        CliContext {
            shell: Mutex::new(shell),
            cwd,
            http_client: reqwest::Client::new(),
        }
    }

    pub fn default() -> anyhow::Result<CliContext> {
        let shell = Shell::new();
        let cwd = env::current_dir().context("could not get the current working directory")?;

        Ok(CliContext::new(shell, cwd))
    }

    pub fn shell(&self) -> MutexGuard<'_, Shell> {
        self.shell.lock().unwrap()
    }
}
