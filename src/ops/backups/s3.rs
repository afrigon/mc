use std::path::Path;

use tempfile::NamedTempFile;

use crate::context::McContext;
use crate::network;
use crate::ops::backups::BackupBackend;
use crate::services;
use crate::utils::errors::McResult;

pub struct S3BackupBackend {
    bucket: String,
    region: Option<String>,
    world_name: String
}

impl S3BackupBackend {
    pub fn new(bucket: String, region: Option<String>, world_name: String) -> S3BackupBackend {
        S3BackupBackend {
            bucket,
            region,
            world_name
        }
    }
}

impl BackupBackend for S3BackupBackend {
    async fn backup(&self, filename: &str, archive: NamedTempFile) -> McResult<()> {
        services::s3_api::upload(
            &self.bucket,
            self.region.as_deref(),
            filename,
            archive.path().to_path_buf()
        )
        .await?;

        Ok(())
    }

    async fn list(&self) -> McResult<Vec<String>> {
        let mut backups: Vec<String> =
            services::s3_api::list_keys(&self.bucket, self.region.as_deref())
                .await?
                .into_iter()
                .filter(|key| super::is_instance_backup(key, &self.world_name))
                .collect();

        // Automatic keys embed a timestamp, so a reverse lexicographic sort
        // orders them newest first; named backups land wherever their name
        // sorts.
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
        let source =
            services::s3_api::artifact_source(&self.bucket, self.region.as_deref(), filename, None)
                .await?;

        network::stream_artifact(&context.http_client, source, output, staging).await?;

        Ok(())
    }
}
