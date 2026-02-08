use std::env;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::MutexGuard;

use anyhow::Context;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::header::USER_AGENT;

use crate::utils::errors::McResult;
use crate::utils::shell::Shell;

pub struct McContext {
    shell: Mutex<Shell>,
    pub cwd: PathBuf,
    pub http_client: reqwest::Client
}

impl McContext {
    pub fn new(shell: Shell, cwd: PathBuf) -> McResult<McContext> {
        let mut headers = HeaderMap::new();
        let user_agent = format!(
            "afrigon/{}/{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        headers.insert(USER_AGENT, HeaderValue::from_str(&user_agent)?);

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(McContext {
            shell: Mutex::new(shell),
            cwd,
            http_client
        })
    }

    pub fn default() -> McResult<McContext> {
        let shell = Shell::new();

        let cwd = env::current_dir().context("could not get the current working directory")?;

        Ok(McContext::new(shell, cwd)?)
    }

    pub fn shell(&self) -> MutexGuard<'_, Shell> {
        self.shell.lock().unwrap()
    }
}
