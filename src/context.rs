use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::MutexGuard;

use anyhow::Context;

use crate::utils::shell::Shell;

pub struct McContext {
    shell: Mutex<Shell>,
    pub cwd: PathBuf,
    pub http_client: reqwest::Client
}

impl McContext {
    pub fn new(shell: Shell, cwd: PathBuf) -> McContext {
        McContext {
            shell: Mutex::new(shell),
            cwd,
            http_client: reqwest::Client::new()
        }
    }

    pub fn default() -> anyhow::Result<McContext> {
        let shell = Shell::new();

        let cwd = env::current_dir().context("could not get the current working directory")?;

        Ok(McContext::new(shell, cwd))
    }

    pub fn shell(&self) -> MutexGuard<'_, Shell> {
        self.shell.lock().unwrap()
    }
}
