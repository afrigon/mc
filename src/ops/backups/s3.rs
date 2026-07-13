use std::path::Path;

use tempfile::NamedTempFile;

use crate::context::McContext;
use crate::network;
use crate::ops::backups::BackupBackend;
use crate::services;
use crate::utils::errors::McResult;

pub struct S3BackupBackend {
    bucket: String
}

impl S3BackupBackend {
    pub fn new(bucket: String) -> S3BackupBackend {
        S3BackupBackend { bucket }
    }
}

impl BackupBackend for S3BackupBackend {
    async fn backup(&self, filename: &str, archive: NamedTempFile) -> McResult<()> {
        services::s3_api::upload(&self.bucket, filename, archive.path().to_path_buf()).await?;

        Ok(())
    }

    async fn list(&self) -> McResult<Vec<String>> {
        let mut backups = services::s3_api::list_keys(&self.bucket).await?;

        // Keys are timestamped, so a reverse lexicographic sort is newest first.
        backups.sort_by(|a, b| b.cmp(a));

        Ok(backups)
    }

    async fn restore(
        &self,
        context: &mut McContext,
        filename: &str,
        output: &Path,
        staging: &Path
    ) -> McResult<()> {
        let source = services::s3_api::artifact_source(&self.bucket, filename, None).await?;

        network::stream_artifact(&context.http_client, source, output, staging).await?;

        Ok(())
    }
}
