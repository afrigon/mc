use anyhow::Context;
use serde::Serialize;

use crate::utils::errors::McResult;

#[derive(Serialize)]
pub struct DiscordWebhookPayload<'a> {
    content: &'a str
}

pub async fn notify(client: &reqwest::Client, webhook: &str, message: &str) -> McResult<()> {
    client
        .post(webhook)
        .json(&DiscordWebhookPayload { content: message })
        .send()
        .await?
        .error_for_status()
        .context("could not notify discord")?;

    Ok(())
}
