pub mod archive;
pub mod artifact;

use std::io;
use std::path::Path;

use anyhow::Context;
use futures_util::StreamExt;
use tokio::io::AsyncRead;
use tokio::io::AsyncWriteExt;
use tokio_util::io::StreamReader;
use tracing::debug;

use crate::crypto::checksum::ChecksumAlgorithm;
use crate::crypto::checksum::LocalChecksum;
use crate::crypto::hash::Hasher;
use crate::network::archive::deflate_tar_gz;
use crate::network::archive::deflate_zip;
use crate::network::artifact::ArtifactKind;
use crate::network::artifact::ArtifactSource;
use crate::utils::errors::McResult;

pub async fn stream_artifact(
    client: &reqwest::Client,
    source: ArtifactSource,
    output: &Path
) -> McResult<()> {
    debug!("downloading from: {}", source.url);

    let checksum = source
        .checksum(client)
        .await
        .context("Failed to get checksum for artifact")?;

    let r = client.get(source.url).send().await?.error_for_status()?;

    let stream = r
        .bytes_stream()
        .map(|s| s.map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
    let reader = StreamReader::new(stream);

    // TODO: clean this up to avoid hashing files when checksum is None.
    let hasher = Hasher::new(
        reader,
        checksum
            .clone()
            .map(|c| c.algorithm())
            .unwrap_or(ChecksumAlgorithm::md5)
    );

    match source.kind {
        ArtifactKind::Zip => {
            deflate_zip(hasher, checksum, output).await?;
        }
        ArtifactKind::TarGz => {
            deflate_tar_gz(hasher, checksum, output).await?;
        }
        _ => {
            save_file(hasher, checksum, output).await?;
        }
    };

    Ok(())
}

pub async fn save_file<R: AsyncRead + Unpin>(
    mut reader: Hasher<R>,
    checksum: Option<LocalChecksum>,
    output: &Path
) -> McResult<()> {
    let dir = tempfile::tempdir()?;
    let file_path = dir.path().join("file.partial");
    let async_file = tokio::fs::File::create(&file_path).await?;

    let mut writer = tokio::io::BufWriter::with_capacity(256 * 1024, async_file);

    tokio::io::copy(&mut reader, &mut writer).await?;
    writer.flush().await?;

    if let Some(checksum) = checksum {
        if reader.hash().as_ref() != checksum.hash() {
            anyhow::bail!("checksum does not match")
        }
    }

    tokio::fs::rename(file_path, output).await?;

    Ok(())
}
