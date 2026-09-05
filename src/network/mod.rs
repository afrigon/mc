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
use crate::network::artifact::ArtifactKind;
use crate::network::artifact::ArtifactSource;
use crate::utils::archive::deflate_tar_gz;
use crate::utils::archive::deflate_zip;
use crate::utils::errors::McResult;

pub async fn stream_artifact(
    client: &reqwest::Client,
    source: ArtifactSource,
    output: &Path,
    staging: &Path
) -> McResult<()> {
    debug!("downloading from: {}", source.url);

    tokio::fs::create_dir_all(staging).await?;

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

    let result = match source.kind {
        ArtifactKind::Zip => deflate_zip(hasher, checksum, output, staging).await,
        ArtifactKind::TarGz => deflate_tar_gz(hasher, checksum, output, staging).await,
        ArtifactKind::Jar => save_file(hasher, checksum, output, staging, false).await,
        ArtifactKind::Binary => save_file(hasher, checksum, output, staging, true).await
    };

    // Empty-only removal: never deletes anything another operation is staging.
    let _ = tokio::fs::remove_dir(staging).await;

    result
}

#[cfg_attr(not(unix), allow(unused_variables))]
async fn save_file<R: AsyncRead + Unpin>(
    mut reader: Hasher<R>,
    checksum: Option<LocalChecksum>,
    output: &Path,
    staging: &Path,
    executable: bool
) -> McResult<()> {
    let dir = tempfile::tempdir_in(staging)?;
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

    #[cfg(unix)]
    if executable {
        use std::os::unix::fs::PermissionsExt;

        tokio::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o755)).await?;
    }

    tokio::fs::rename(file_path, output).await?;

    Ok(())
}
