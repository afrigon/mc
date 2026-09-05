use std::io::ErrorKind;
use std::path::Path;

use anyhow::Context;
use tokio::io::AsyncWriteExt;

use crate::context::McContext;
use crate::utils::errors::McResult;

// Files holding plaintext secrets are created readable only by the user
// running the server.
#[cfg(unix)]
pub async fn write_private_file(
    context: &mut McContext,
    path: &Path,
    contents: &str,
    contains_secrets: bool
) -> McResult<()> {
    use std::os::unix::fs::PermissionsExt;

    match tokio::fs::metadata(path).await {
        Ok(metadata) => {
            let mode = metadata.permissions().mode();

            if mode & 0o077 != 0 {
                if contains_secrets {
                    anyhow::bail!(
                        "{} is accessible to other users (mode {:03o}) and holds secrets; fix it with `chmod 600 {}`",
                        path.display(),
                        mode & 0o777,
                        path.display()
                    );
                }

                _ = context.shell().warn(format!(
                    "{} is accessible to other users (mode {:03o}); fix it with `chmod 600 {}`",
                    path.display(),
                    mode & 0o777,
                    path.display()
                ));
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("could not inspect the {} permissions", path.display()));
        }
    }

    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .await?;

    file.write_all(contents.as_bytes()).await?;
    file.flush().await?;

    Ok(())
}

#[cfg(not(unix))]
pub async fn write_private_file(
    _context: &mut McContext,
    path: &Path,
    contents: &str,
    _contains_secrets: bool
) -> McResult<()> {
    tokio::fs::write(path, contents).await?;

    Ok(())
}
