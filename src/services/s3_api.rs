use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use aws_config::BehaviorVersion;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use url::Url;

use crate::network::artifact::ArtifactKind;
use crate::network::artifact::ArtifactSource;
use crate::utils::errors::McResult;

pub async fn upload(bucket: &str, key: &str, path: PathBuf) -> McResult<()> {
    let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
    let s3_client = aws_sdk_s3::Client::new(&config);

    let body = ByteStream::from_path(&path)
        .await
        .context("could not find file to upload to s3")?;

    s3_client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body)
        .send()
        .await
        .context("could not upload object to s3")?;

    Ok(())
}

pub async fn artifact_source(
    bucket: &str,
    key: &str,
    version: Option<&str>
) -> McResult<ArtifactSource> {
    let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
    let s3_client = aws_sdk_s3::Client::new(&config);

    let presigning_config = PresigningConfig::expires_in(Duration::from_mins(1))?;

    let r = match version {
        Some(version) => s3_client
            .get_object()
            .bucket(bucket)
            .key(key)
            .version_id(version)
            .presigned(presigning_config)
            .await
            .context("could not create a signed s3 url")?,
        None => s3_client
            .get_object()
            .bucket(bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .context("could not create a signed s3 url")?
    };

    let source = ArtifactSource {
        url: Url::parse(r.uri())?,
        kind: ArtifactKind::TarGz,
        checksum: None
    };

    Ok(source)
}

/// List every object key in the (backup-dedicated) bucket, paginating until
/// exhausted.
pub async fn list_keys(bucket: &str) -> McResult<Vec<String>> {
    let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
    let s3_client = aws_sdk_s3::Client::new(&config);

    let mut keys = Vec::new();
    let mut continuation_token = None;

    loop {
        let mut request = s3_client.list_objects_v2().bucket(bucket);

        if let Some(token) = continuation_token {
            request = request.continuation_token(token);
        }

        let response = request
            .send()
            .await
            .context("could not list backups from s3")?;

        for object in response.contents() {
            if let Some(key) = object.key() {
                keys.push(key.to_string());
            }
        }

        if response.is_truncated() == Some(true) {
            continuation_token = response.next_continuation_token().map(str::to_string);
        } else {
            break;
        }
    }

    Ok(keys)
}
