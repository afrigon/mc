use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use serde::Deserialize;
use url::Url;

use crate::mods::loader::LoaderKind;
use crate::network::artifact::ArtifactKind;
use crate::network::artifact::ArtifactSource;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::ProductDescriptor;

#[derive(Deserialize)]
struct FabricApiVersion {
    loader: FabricApiLoaderVersion
}

#[derive(Deserialize)]
pub struct FabricApiLoaderVersion {
    pub version: String
}

#[derive(Deserialize)]
pub struct FabricApiInstallerVersion {
    pub version: String
}

pub async fn artifact_source(
    client: &reqwest::Client,
    loader: &ProductDescriptor<LoaderKind>,
    version: &String
) -> McResult<ArtifactSource> {
    let installer = get_latest_installer(client).await?.version;

    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}/{}/{}/server/jar",
        version, loader.version, installer
    );

    let source = ArtifactSource {
        url: Url::parse(&url)?,
        kind: ArtifactKind::Jar,
        checksum: None
    };

    Ok(source)
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

#[derive(Deserialize)]
pub struct FabricModMetadata {
    pub id: String,
    pub version: String
}

pub async fn parse_jar_metadata(path: PathBuf) -> McResult<FabricModMetadata> {
    let file = fs::File::open(&path)?;

    let mut archive = zip::ZipArchive::new(file)?;

    let fabric_file = archive.by_name("fabric.mod.json")?;

    let metadata = serde_json::from_reader(fabric_file)?;

    Ok(metadata)
}
