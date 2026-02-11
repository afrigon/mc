use anyhow::Context;
use serde::Deserialize;
use url::Url;

use crate::mods::loader::LoaderKind;
use crate::utils::errors::McResult;

#[derive(Deserialize)]
pub struct ModrinthApiVersion {
    pub id: String,
    pub dependencies: Vec<ModrinthApiDependency>,
    pub files: Vec<ModrinthApiFile>
}

#[derive(Deserialize)]
pub struct ModrinthApiFile {
    pub hashes: ModrinthApiFileHashes,
    pub url: Url,
    pub primary: bool
}

#[derive(Deserialize)]
pub struct ModrinthApiFileHashes {
    pub sha1: String
}

#[derive(Deserialize)]
pub struct ModrinthApiDependency {
    pub project_id: String,
    pub version_id: Option<String>,
    pub dependency_type: ModrinthApiDependencyKind
}

#[derive(Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ModrinthApiDependencyKind {
    Required,
    Incompatible,
    Optional
}

#[derive(Deserialize)]
pub struct ModrinthApiProject {
    pub slug: String
}

pub async fn get_project(client: &reqwest::Client, id: &String) -> McResult<ModrinthApiProject> {
    let url = Url::parse(&format!("https://api.modrinth.com/v2/project/{}", id))?;

    let project = client
        .get(url)
        .send()
        .await?
        .error_for_status()
        .context(format!("could not find modrinth project with id {}", id))?
        .json::<ModrinthApiProject>()
        .await?;

    Ok(project)
}

pub async fn get_version(client: &reqwest::Client, id: &String) -> McResult<ModrinthApiVersion> {
    let url = Url::parse(&format!("https://api.modrinth.com/v2/version/{}", id))?;

    let version = client
        .get(url)
        .send()
        .await?
        .error_for_status()
        .context(format!("could not find modrinth version with id {}", id))?
        .json::<ModrinthApiVersion>()
        .await?;

    Ok(version)
}

pub async fn get_latest_version(
    client: &reqwest::Client,
    project: &String,
    loader: LoaderKind,
    game_version: &String
) -> McResult<ModrinthApiVersion> {
    let url = Url::parse(&format!(
        "https://api.modrinth.com/v2/project/{}/version",
        project
    ))?;

    let loaders = serde_json::to_string(&vec![loader])?;
    let game_versions = serde_json::to_string(&vec![game_version])?;

    let versions = client
        .get(url)
        .query(&[
            ("loaders", loaders.as_str()),
            ("game_versions", game_versions.as_str()),
            ("include_changelog", "false")
        ])
        .send()
        .await?
        .error_for_status()
        .context(format!("could not find a suitable version of {}", project))?
        .json::<Vec<ModrinthApiVersion>>()
        .await?;

    versions
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("could not find a suitable version of {}", project))
}
