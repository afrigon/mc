pub mod archive;
pub mod artifact;

use std::io::Write;
use std::path::Path;

use anyhow::Context;
use digest::Digest;
use digest::DynDigest;
use md5::Md5;
use sha1::Sha1;
use sha2::Sha256;
use url::Url;

use crate::crypto::checksum::LocalChecksum;
use crate::network::artifact::ArtifactSource;
use crate::utils::errors::McResult;

pub async fn stream_and_deflate(
    client: &reqwest::Client,
    source: ArtifactSource,
    output: &Path
) -> McResult<()> {
    Ok(())
}

pub async fn stream_artifact(
    client: &reqwest::Client,
    source: ArtifactSource,
    output: &Path
) -> McResult<()> {
    let checksum = source
        .checksum(client)
        .await
        .context("Failed to get checksum for artifact")?;

    let mut hasher: Box<dyn DynDigest> = match checksum {
        LocalChecksum::md5(_) => Box::new(Md5::new()),
        LocalChecksum::sha1(_) => Box::new(Sha1::new()),
        LocalChecksum::sha256(_) => Box::new(Sha256::new())
    };

    let mut f = tempfile::tempfile()?;
    let mut r = client.get(source.url).send().await?.error_for_status()?;

    while let Some(chunk) = r.chunk().await? {
        hasher.update(&chunk);
        f.write_all(&chunk)?;
    }

    let hash = hasher.finalize();
    let valid = match checksum {
        LocalChecksum::md5(real_hash) => real_hash == *hash,
        LocalChecksum::sha1(real_hash) => real_hash == *hash,
        LocalChecksum::sha256(real_hash) => real_hash == *hash
    };

    if !valid {
        // f.

        anyhow::bail!("hash does not match")
    }

    f.flush()?;
    // f.

    // tokio::fs::rename(&f, output).await?;

    Ok(())
}

pub async fn stream_file(
    client: &reqwest::Client,
    url: &Url,
    output: &Path,
    cache_directory: &Path
) -> McResult<()> {
    let filename = output.file_name().expect("output must be a file");
    let extension = output.extension().unwrap_or_default();

    let cache = cache_directory
        .join("downloads")
        .with_file_name(filename)
        .with_extension(extension);

    if let Some(parent) = output.parent() {
        let _ = tokio::fs::create_dir_all(&parent).await?;
    }

    if let Some(cache_parent) = cache.parent() {
        let _ = tokio::fs::create_dir_all(&cache_parent).await?;
    }

    let mut f = tokio::fs::File::create(&cache).await?;
    let mut r = client.get(url.clone()).send().await?.error_for_status()?;

    while let Some(chunk) = r.chunk().await? {
        // f.write_all(&chunk).await?;
    }

    // f.flush().await?;
    tokio::fs::rename(&cache, output).await?;

    Ok(())
}
