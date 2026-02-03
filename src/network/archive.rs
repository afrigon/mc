use std::fs;
use std::path::Path;
use std::path::PathBuf;

use async_compression::tokio::bufread::GzipDecoder;
use tokio::io::AsyncRead;
use tokio::io::AsyncWriteExt;
use tokio_tar::ArchiveBuilder;

use crate::crypto::checksum::LocalChecksum;
use crate::crypto::hash::Hasher;
use crate::utils::errors::McResult;

pub async fn deflate_tar_gz<R: AsyncRead + Unpin>(
    mut reader: Hasher<R>,
    checksum: Option<LocalChecksum>,
    output: &Path
) -> McResult<()> {
    let dir = tempfile::tempdir()?;

    let buf = tokio::io::BufReader::with_capacity(256 * 1024, &mut reader);
    let gz = GzipDecoder::new(buf);
    let mut tar = ArchiveBuilder::new(gz)
        .set_allow_external_symlinks(false)
        .set_preserve_permissions(false)
        .set_preserve_mtime(false)
        .set_unpack_xattrs(false)
        .set_overwrite(false)
        .build();

    tar.unpack(dir.path()).await?;

    if let Some(checksum) = checksum {
        if reader.hash().as_ref() != checksum.hash() {
            anyhow::bail!("checksum does not match")
        }
    }

    let mut rd = tokio::fs::read_dir(dir.path()).await?;
    let mut candidate: Option<PathBuf> = None;

    while let Some(entry) = rd.next_entry().await? {
        if entry.metadata().await?.is_dir() {
            if candidate != None {
                break;
            }

            candidate = Some(entry.path());
        }
    }

    let source = if let Some(candidate) = candidate {
        candidate
    } else {
        dir.path().to_path_buf()
    };

    tokio::fs::rename(source, output).await?;

    Ok(())
}

pub async fn deflate_zip<R: AsyncRead + Unpin>(
    mut reader: Hasher<R>,
    checksum: Option<LocalChecksum>,
    output: &Path
) -> McResult<()> {
    let dir = tempfile::tempdir()?;
    let archive_path = dir.path().join("archive.zip.partial");
    let async_file = tokio::fs::File::create(&archive_path).await?;

    let mut writer = tokio::io::BufWriter::with_capacity(256 * 1024, async_file);

    tokio::io::copy(&mut reader, &mut writer).await?;
    writer.flush().await?;

    if let Some(checksum) = checksum {
        if reader.hash().as_ref() != checksum.hash() {
            anyhow::bail!("checksum does not match")
        }
    }

    let file = fs::File::open(&archive_path)?;

    let mut archive = zip::ZipArchive::new(file)?;
    archive.extract_unwrapped_root_dir(output, zip::read::root_dir_common_filter)?;

    Ok(())
}
