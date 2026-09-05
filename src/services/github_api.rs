use anyhow::Context;
use serde::Deserialize;
use url::Url;

use crate::utils::errors::McResult;

const API_URL: &'static str = "https://api.github.com";

#[derive(Deserialize)]
pub struct GithubApiRelease {
    pub tag_name: String,
    pub prerelease: bool,
    pub draft: bool,
    pub assets: Vec<GithubApiAsset>
}

#[derive(Deserialize)]
pub struct GithubApiAsset {
    pub name: String,
    pub browser_download_url: Url,
    pub digest: Option<String>
}

pub async fn get_releases(
    client: &reqwest::Client,
    owner: &str,
    repository: &str
) -> McResult<Vec<GithubApiRelease>> {
    client
        .get(format!(
            "{}/repos/{}/{}/releases",
            API_URL, owner, repository
        ))
        .query(&[("per_page", "100")])
        .send()
        .await
        .context("could not send HTTP request")?
        .error_for_status()
        .with_context(|| format!("could not list the releases of {}/{}", owner, repository))?
        .json()
        .await
        .context("could not parse the GitHub releases response")
}

pub async fn get_release(
    client: &reqwest::Client,
    owner: &str,
    repository: &str,
    tag: &str
) -> McResult<GithubApiRelease> {
    client
        .get(format!(
            "{}/repos/{}/{}/releases/tags/{}",
            API_URL, owner, repository, tag
        ))
        .send()
        .await
        .context("could not send HTTP request")?
        .error_for_status()
        .with_context(|| format!("could not find release {} of {}/{}", tag, owner, repository))?
        .json()
        .await
        .context("could not parse the GitHub release response")
}
