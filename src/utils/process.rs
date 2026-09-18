use std::future::pending;

use anyhow::Context;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncRead;
use tokio::io::BufReader;
use tokio::process::Command;
#[cfg(unix)]
use tokio::signal::unix::Signal;
#[cfg(unix)]
use tokio::signal::unix::SignalKind;
#[cfg(unix)]
use tokio::signal::unix::signal;
#[cfg(windows)]
use tokio::signal::windows::CtrlC;

use crate::utils::errors::McResult;

#[cfg(windows)]
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

pub fn render_command(command: &Command) -> String {
    let command = command.as_std();

    let mut command_parts: Vec<String> = Vec::new();
    command_parts.push(command.get_program().to_string_lossy().into_owned());
    command_parts.extend(command.get_args().map(|a| a.to_string_lossy().into_owned()));

    command_parts
        .into_iter()
        .map(|s| {
            if s.contains(" ") || s.contains("\t") {
                format!("{:?}", s)
            } else {
                s
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Calls `handle` with every line `output` yields until it closes or fails.
/// Invalid UTF-8 is replaced rather than dropping the line.
pub async fn for_each_line<R, F>(output: R, mut handle: F)
where
    R: AsyncRead + Unpin,
    F: FnMut(&str)
{
    let mut reader = BufReader::new(output);
    let mut buffer = Vec::new();

    loop {
        buffer.clear();

        match reader.read_until(b'\n', &mut buffer).await {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }

        let line = String::from_utf8_lossy(&buffer);

        handle(line.trim_end_matches(['\n', '\r']));
    }
}

pub fn detach_from_terminal_signals(command: &mut Command) {
    #[cfg(unix)]
    command.process_group(0);

    #[cfg(windows)]
    command.creation_flags(CREATE_NEW_PROCESS_GROUP);
}

/// Shutdown requests: Ctrl-C everywhere, plus SIGTERM on Unix so
/// `systemctl stop` triggers a graceful shutdown, and SIGHUP so losing the
/// terminal does too instead of orphaning the server.
pub struct ShutdownSignals {
    #[cfg(unix)]
    interrupt: Signal,
    #[cfg(unix)]
    terminate: Signal,
    #[cfg(unix)]
    hangup: Signal,
    #[cfg(windows)]
    ctrl_c: CtrlC
}

impl ShutdownSignals {
    pub fn register() -> McResult<Self> {
        #[cfg(unix)]
        {
            Ok(Self {
                interrupt: signal(SignalKind::interrupt())
                    .context("could not register the SIGINT handler")?,
                terminate: signal(SignalKind::terminate())
                    .context("could not register the SIGTERM handler")?,
                hangup: signal(SignalKind::hangup())
                    .context("could not register the SIGHUP handler")?
            })
        }

        #[cfg(windows)]
        {
            Ok(Self {
                ctrl_c: tokio::signal::windows::ctrl_c()
                    .context("could not register the Ctrl-C handler")?
            })
        }
    }

    /// Resolves when the next shutdown signal arrives.
    pub async fn recv(&mut self) {
        #[cfg(unix)]
        tokio::select! {
            _ = received(&mut self.interrupt) => {}
            _ = received(&mut self.terminate) => {}
            _ = received(&mut self.hangup) => {}
        }

        #[cfg(windows)]
        if self.ctrl_c.recv().await.is_none() {
            pending::<()>().await;
        }
    }
}

// A closed signal stream must not masquerade as a shutdown request, so it
// pends forever instead of resolving.
#[cfg(unix)]
async fn received(signal: &mut Signal) {
    if signal.recv().await.is_none() {
        pending::<()>().await;
    }
}
