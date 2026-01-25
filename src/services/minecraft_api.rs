use serde::Deserialize;
use url::Url;

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
    pub id: String,
    pub downloads: MinecraftApiVersionMetadataDownloads
}

#[derive(Deserialize)]
pub struct MinecraftApiVersionMetadataDownloads {
    pub client: MinecraftApiArtifactMetadata,
    pub client_mappings: MinecraftApiArtifactMetadata,
    pub server: MinecraftApiArtifactMetadata,
    pub server_mappings: MinecraftApiArtifactMetadata
}

#[derive(Deserialize)]
pub struct MinecraftApiArtifactMetadata {
    pub url: Url,
    pub size: usize,
    pub sha1: String
}

pub struct MinecraftApiService {}

impl MinecraftApiService {
    pub async fn get_manifest(
        client: &reqwest::Client
    ) -> Result<MinecraftApiVersionManifest, reqwest::Error> {
        client
            .get(LIST_URL)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn get_metadata(
        client: &reqwest::Client,
        url: &Url
    ) -> Result<MinecraftApiVersionMetadata, reqwest::Error> {
        client
            .get(url.clone())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    // TODO: move this outside of this service
    pub async fn download_version(client: &reqwest::Client, url: &Url) -> anyhow::Result<()> {
        // let cache = &Context::current().directories.cache;

        // let filename = url
        //     .path_segments()
        //     .and_then(|s| s.last())
        //     .filter(|s| !s.is_empty())
        //     .expect("url must end with a file name");

        // let output = &Context::current().directories.data
        //     .join(filename);

        // network::stream_file(client, url, output, cache).await?;

        // TODO: add checksum validation

        Ok(())
    }
}
