use serde::Deserialize;
use url::Url;

use crate::crypto::checksum::ChecksumRef;
use crate::crypto::checksum::LocalChecksum;
use crate::network::artifact::ArtifactKind;
use crate::network::artifact::ArtifactSource;
use crate::utils::errors::McResult;

const LIST_URL: &'static str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MinecraftApiVersionType {
    Release,
    Snapshot,

    #[serde(rename = "old_beta")]
    Beta,

    #[serde(rename = "old_alpha")]
    Alpha
}

#[derive(Deserialize)]
pub struct MinecraftApiVersionManifest {
    pub latest: MinecraftApiVersionManifestLatest,
    pub versions: Vec<MinecraftApiVersionManifestEntry>
}

#[derive(Deserialize)]
pub struct MinecraftApiVersionManifestLatest {
    pub release: String,
    pub snapshot: String
}

#[derive(Deserialize)]
pub struct MinecraftApiVersionManifestEntry {
    pub id: String,
    pub url: Url,

    #[serde(rename = "type")]
    pub version_type: MinecraftApiVersionType
}

#[derive(Deserialize)]
pub struct MinecraftApiVersionMetadata {
    pub downloads: MinecraftApiVersionMetadataDownloads
}

#[derive(Deserialize)]
pub struct MinecraftApiVersionMetadataDownloads {
    pub client: MinecraftApiArtifactMetadata,
    pub server: MinecraftApiArtifactMetadata
}

#[derive(Deserialize)]
pub struct MinecraftApiArtifactMetadata {
    pub url: Url,
    pub sha1: String
}

pub async fn get_manifest(client: &reqwest::Client) -> McResult<MinecraftApiVersionManifest> {
    let data = client
        .get(LIST_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(data)
}

pub async fn get_latest_version(client: &reqwest::Client) -> McResult<String> {
    let data: MinecraftApiVersionManifest = client
        .get(LIST_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(data.latest.release)
}

pub async fn get_metadata(
    client: &reqwest::Client,
    url: &Url
) -> McResult<MinecraftApiVersionMetadata> {
    let data = client
        .get(url.clone())
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(data)
}

pub async fn artifact_source(
    client: &reqwest::Client,
    version: &String
) -> McResult<ArtifactSource> {
    let manifest = get_manifest(client).await?;
    let version = manifest
        .versions
        .iter()
        .find(|entry| entry.id == *version)
        .ok_or(anyhow::anyhow!("could not find minecraft version"))?;

    let metadata = get_metadata(client, &version.url).await?;
    let checksum_string = metadata.downloads.server.sha1;

    let mut checksum = [0u8; 20];
    hex::decode_to_slice(checksum_string, &mut checksum)?;

    let source = ArtifactSource {
        url: metadata.downloads.server.url,
        kind: ArtifactKind::Jar,
        checksum: Some(ChecksumRef::Local(LocalChecksum::sha1(checksum)))
    };

    Ok(source)
}
