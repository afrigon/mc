use std::io;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use tempfile::NamedTempFile;

use crate::context::McContext;
use crate::crypto::checksum::ChecksumAlgorithm;
use crate::crypto::hash::Hasher;
use crate::ops::backups::BackupBackend;
use crate::utils::archive::deflate_tar_gz;
use crate::utils::errors::McResult;

pub struct LocalBackupBackend {
    directory: PathBuf,
    keep: usize
}

impl LocalBackupBackend {
    pub fn new(directory: PathBuf, keep: usize) -> LocalBackupBackend {
        LocalBackupBackend { directory, keep }
    }

    /// Delete the oldest backups beyond the retention limit.
    async fn prune(&self) -> McResult<()> {
        let backups = self.list().await?;

        for filename in backups.into_iter().skip(self.keep) {
            tokio::fs::remove_file(self.directory.join(filename)).await?;
        }

        Ok(())
    }
}

impl BackupBackend for LocalBackupBackend {
    async fn backup(&self, filename: &str, archive: NamedTempFile) -> McResult<()> {
        tokio::fs::create_dir_all(&self.directory).await?;

        _ = archive.persist(self.directory.join(filename))?;

        self.prune().await?;

        Ok(())
    }

    async fn list(&self) -> McResult<Vec<String>> {
        let mut rd = match tokio::fs::read_dir(&self.directory).await {
            Ok(rd) => rd,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e.into())
        };

        let mut backups = Vec::new();

        while let Some(entry) = rd.next_entry().await? {
            let filename = entry.file_name();

            if let Some(filename) = filename.to_str() {
                if filename.ends_with(".tar.gz") {
                    backups.push(filename.to_string());
                }
            }
        }

        // Filenames are timestamped, so a reverse lexicographic sort is newest first.
        backups.sort_by(|a, b| b.cmp(a));

        Ok(backups)
    }

    async fn restore(
        &self,
        _context: &mut McContext,
        filename: &str,
        output: &Path
    ) -> McResult<()> {
        let path = self.directory.join(filename);

        let file = tokio::fs::File::open(&path)
            .await
            .with_context(|| format!("could not open backup `{}`", path.to_string_lossy()))?;

        let reader = Hasher::new(file, ChecksumAlgorithm::md5);

        deflate_tar_gz(reader, None, output).await?;

        Ok(())
    }
}
