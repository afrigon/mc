pub mod artifact;

use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use url::Url;

pub async fn stream_file(
    client: &reqwest::Client,
    url: &Url,
    output: &Path,
    cache_directory: &Path
) -> anyhow::Result<()> {
    let filename = output.file_name().expect("output must be a file");
    let extension = output.extension().unwrap_or_default();

    let cache = cache_directory
        .join("downloads")
        .with_file_name(filename)
        .with_extension(extension);

    if let Some(parent) = output.parent() {
        let _ = fs::create_dir_all(&parent).await?;
    }

    if let Some(cache_parent) = cache.parent() {
        let _ = fs::create_dir_all(&cache_parent).await?;
    }

    let mut f = fs::File::create(&cache).await?;
    let mut r = client.get(url.clone()).send().await?.error_for_status()?;

    while let Some(chunk) = r.chunk().await? {
        f.write_all(&chunk).await?;
    }

    f.flush().await?;

    fs::rename(&cache, output).await?;

    Ok(())
}
