use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use async_compression::tokio::bufread::GzipDecoder;
use async_compression::tokio::write::GzipEncoder;
use tempfile::NamedTempFile;
use tokio::io::AsyncRead;
use tokio::io::AsyncWriteExt;
use tokio_tar::ArchiveBuilder;
use walkdir::WalkDir;

use crate::crypto::checksum::LocalChecksum;
use crate::crypto::hash::Hasher;
use crate::utils::errors::McResult;

pub async fn inflate_tar_gz(
    src: PathBuf,
    exclude: &HashSet<PathBuf>,
    staging: &Path
) -> McResult<NamedTempFile> {
    let temp = tempfile::NamedTempFile::new_in(staging)?;

    let file = temp.as_file().try_clone()?;
    let async_file = tokio::fs::File::from_std(file);

    let gz = GzipEncoder::new(async_file);
    let mut tar = tokio_tar::Builder::new(gz);

    let root: PathBuf = src
        .file_name()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("world"));

    tar.append_dir(&root, &src).await?;

    for entry in WalkDir::new(&src).follow_links(false) {
        let entry = entry?;
        let path = entry.path();

        if path == src {
            continue;
        }

        let relative_path = path.strip_prefix(&src)?;

        if exclude.contains(relative_path) {
            continue;
        }

        let dst = root.join(relative_path);

        if entry.file_type().is_dir() {
            tar.append_dir(dst, &src).await?;
        } else {
            let mut f = tokio::fs::File::open(path).await?;
            tar.append_file(dst, &mut f).await?;
        }
    }

    let mut gz = tar.into_inner().await?;

    gz.shutdown().await?;

    Ok(temp)
}

pub async fn deflate_tar_gz<R: AsyncRead + Unpin>(
    mut reader: Hasher<R>,
    checksum: Option<LocalChecksum>,
    output: &Path,
    staging: &Path
) -> McResult<()> {
    let dir = tempfile::tempdir_in(staging)?;

    let buf = tokio::io::BufReader::with_capacity(256 * 1024, &mut reader);
    let gz = GzipDecoder::new(buf);
    let mut tar = ArchiveBuilder::new(gz)
        .set_allow_external_symlinks(false)
        .set_preserve_permissions(true)
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
    output: &Path,
    staging: &Path
) -> McResult<()> {
    let dir = tempfile::tempdir_in(staging)?;
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
    let extracted = dir.path().join("extracted");

    let mut archive = zip::ZipArchive::new(file)?;
    archive.extract_unwrapped_root_dir(&extracted, zip::read::root_dir_common_filter)?;

    tokio::fs::rename(extracted, output).await?;

    Ok(())
}
