use anyhow::Context;
use reqwest::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::utils::errors::McResult;

const PROFILE_URL: &str = "https://api.mojang.com/users/profiles/minecraft";

#[derive(Deserialize)]
pub struct MojangApiProfile {
    pub id: Uuid,
    pub name: String
}

/// Looks a player up by name. Returns `None` when no account has that name;
/// the returned name carries the account's canonical casing.
pub async fn get_profile(
    client: &reqwest::Client,
    name: &str
) -> McResult<Option<MojangApiProfile>> {
    let response = client
        .get(format!("{}/{}", PROFILE_URL, name))
        .send()
        .await
        .context("could not reach the Mojang profile api")?;

    if matches!(
        response.status(),
        StatusCode::NOT_FOUND | StatusCode::NO_CONTENT
    ) {
        return Ok(None);
    }

    let profile = response
        .error_for_status()
        .context("the Mojang profile api refused the lookup")?
        .json()
        .await
        .context("could not read the Mojang profile response")?;

    Ok(Some(profile))
}
