use std::env;
use std::process::ExitStatus;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;

use crate::services;
use crate::utils::errors::McResult;
use crate::utils::shell::Shell;

/// The notification provider. Only Discord is implemented for now.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NotificationKind {
    Discord
}

impl NotificationKind {
    pub const ALL: [NotificationKind; 1] = [NotificationKind::Discord];

    pub fn webhook_env_var(&self) -> &'static str {
        match self {
            NotificationKind::Discord => "MC_DISCORD_WEBHOOK"
        }
    }

    pub fn webhook_from_env(&self) -> Option<String> {
        env::var(self.webhook_env_var())
            .ok()
            .filter(|webhook| !webhook.is_empty())
    }
}

pub enum ServerEvent {
    Started,
    Stopped,
    Crashed(ExitStatus),
    Sigkill
}

#[derive(Clone)]
pub struct NotificationTarget {
    pub kind: NotificationKind,
    pub webhook: String
}

#[derive(Clone)]
pub struct NotifierConfiguration {
    pub targets: Vec<NotificationTarget>,
    pub on_lifecycle_event: bool,
    pub on_backup: bool,
    pub on_backup_failure: bool,
    pub on_panic: bool,
    pub on_sigkill: bool
}

#[derive(Clone)]
pub struct Notifier {
    client: reqwest::Client,
    shell: Arc<Mutex<Shell>>,
    configuration: NotifierConfiguration
}

impl Notifier {
    pub fn new(
        client: reqwest::Client,
        shell: Arc<Mutex<Shell>>,
        configuration: NotifierConfiguration
    ) -> Notifier {
        Notifier {
            client,
            shell,
            configuration
        }
    }

    pub async fn notify_backup(&self, world_name: &str, result: &McResult<()>) {
        let message = match result {
            Ok(()) if self.configuration.on_backup => {
                format!("✅ backup of `{}` completed", world_name)
            }
            Err(error) if self.configuration.on_backup_failure => {
                format!("❌ backup of `{}` failed: {}", world_name, error)
            }
            _ => return
        };

        self.send(&message).await;
    }

    pub async fn notify_server(&self, name: &str, event: &ServerEvent) {
        let message = match event {
            ServerEvent::Started if self.configuration.on_lifecycle_event => {
                format!("🟢 `{}` started", name)
            }
            ServerEvent::Stopped if self.configuration.on_lifecycle_event => {
                format!("🔴 `{}` stopped", name)
            }
            ServerEvent::Crashed(status) if self.configuration.on_panic => {
                format!("💥 `{}` crashed ({})", name, status)
            }
            ServerEvent::Sigkill if self.configuration.on_sigkill => {
                format!(
                    "⚠️ `{}` was forced down (SIGKILL) without a clean save",
                    name
                )
            }
            _ => return
        };

        self.send(&message).await;
    }

    async fn send(&self, message: &str) {
        for target in &self.configuration.targets {
            let sent = match target.kind {
                NotificationKind::Discord => {
                    services::discord_api::notify(&self.client, &target.webhook, message).await
                }
            };

            // A failed notification must never fail the operation it reports on.
            if let Err(error) = sent {
                _ = self
                    .shell
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .warn(format!("could not send notification: {:?}", error));
            }
        }
    }
}
