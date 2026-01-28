use anyhow::Context;
use serde::Deserialize;

use crate::utils::errors::McResult;

#[derive(Deserialize)]
struct FabricApiVersion {
    loader: FabricApiLoaderVersion
}

#[derive(Deserialize)]
pub struct FabricApiLoaderVersion {
    pub version: String,
    pub stable: bool
}

#[derive(Deserialize)]
pub struct FabricApiInstallerVersion {
    pub version: String,
    pub stable: bool
}

pub async fn get_versions(client: &reqwest::Client) -> McResult<Vec<FabricApiLoaderVersion>> {
    client
        .get("https://meta.fabricmc.net/v2/versions/loader")
        .send()
        .await
        .context("could not send HTTP request")?
        .error_for_status()
        .context("could not get fabric versions")?
        .json::<Vec<FabricApiLoaderVersion>>()
        .await
        .context("could not parse json from fabric versions")
}

pub async fn get_versions_for_game(
    client: &reqwest::Client,
    minecraft_version: &String
) -> McResult<Vec<FabricApiLoaderVersion>> {
    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}",
        minecraft_version
    );

    let versions = client
        .get(url)
        .send()
        .await
        .context("could not send HTTP request")?
        .error_for_status()
        .context("could not get fabric versions")?
        .json::<Vec<FabricApiVersion>>()
        .await
        .context("could not parse json from fabric versions")?
        .into_iter()
        .map(|v| v.loader)
        .collect::<Vec<FabricApiLoaderVersion>>();

    Ok(versions)
}

pub async fn get_installer_versions(
    client: &reqwest::Client
) -> McResult<Vec<FabricApiInstallerVersion>> {
    client
        .get("https://meta.fabricmc.net/v2/versions/installer")
        .send()
        .await
        .context("could not send HTTP request")?
        .error_for_status()
        .context("could not get fabric installer versions")?
        .json::<Vec<FabricApiInstallerVersion>>()
        .await
        .context("could not parse json from fabric installer versions")
}

pub async fn get_latest_installer(client: &reqwest::Client) -> McResult<FabricApiInstallerVersion> {
    client
        .get("https://meta.fabricmc.net/v2/versions/installer")
        .send()
        .await
        .context("could not send HTTP request")?
        .error_for_status()
        .context("could not get fabric installer versions")?
        .json::<Vec<FabricApiInstallerVersion>>()
        .await
        .context("could not parse json from fabric installer versions")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("could not find a fabric installer version"))
}
