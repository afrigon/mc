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
    keep: usize,
    world_name: String
}

impl LocalBackupBackend {
    pub fn new(directory: PathBuf, keep: usize, world_name: String) -> LocalBackupBackend {
        LocalBackupBackend {
            directory,
            keep,
            world_name
        }
    }

    /// Delete the oldest automatic backups beyond the retention limit. Named
    /// backups are kept forever and do not count toward the limit.
    async fn prune(&self) -> McResult<()> {
        let backups = self.list().await?;

        let automatic = backups
            .into_iter()
            .filter(|filename| super::is_automatic_backup(filename, &self.world_name));

        for filename in automatic.skip(self.keep) {
            tokio::fs::remove_file(self.directory.join(filename)).await?;
        }

        Ok(())
    }
}

impl BackupBackend for LocalBackupBackend {
    async fn backup(&self, filename: &str, archive: NamedTempFile) -> McResult<()> {
        tokio::fs::create_dir_all(&self.directory).await?;

        let target = self.directory.join(filename);

        // `persist` is a rename, which fails when the backup directory is on a
        // different filesystem than the staging area. In that case, copy into
        // a temp file next to the target and rename, so an interrupted copy
        // can never leave a partial or truncated backup behind.
        if let Err(error) = archive.persist(&target) {
            let staged = NamedTempFile::new_in(&self.directory)?;

            tokio::fs::copy(error.file.path(), staged.path()).await?;

            staged.persist(&target)?;
        }

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
                if super::is_instance_backup(filename, &self.world_name) {
                    backups.push(filename.to_string());
                }
            }
        }

        // Automatic filenames embed a timestamp, so a reverse lexicographic
        // sort orders them newest first; named backups land wherever their
        // name sorts.
        backups.sort_by(|a, b| b.cmp(a));

        Ok(backups)
    }

    async fn restore(
        &self,
        _context: &mut McContext,
        filename: &str,
        output: &Path,
        staging: &Path
    ) -> McResult<()> {
        let path = self.directory.join(filename);

        let file = tokio::fs::File::open(&path)
            .await
            .with_context(|| format!("could not open backup `{}`", path.to_string_lossy()))?;

        let reader = Hasher::new(file, ChecksumAlgorithm::md5);

        deflate_tar_gz(reader, None, output, staging).await?;

        Ok(())
    }
}
