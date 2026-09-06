use anyhow::Context;
use kdl::KdlDocument;

use crate::manifest::Manifest;
use crate::manifest::ManifestPaths;
use crate::manifest::lock::Lockfile;
use crate::utils;
use crate::utils::errors::McResult;

/// An instance's manifest and lockfile, loaded together for a command that
/// edits them: the resolved manifest to read from, its document to edit
/// while preserving formatting, and the lockfile.
pub struct Workspace {
    pub paths: ManifestPaths,
    pub manifest: Manifest,
    pub document: KdlDocument,
    pub lockfile: Lockfile,
    lockfile_source: String
}

impl Workspace {
    pub async fn load(paths: &ManifestPaths) -> McResult<Workspace> {
        let manifest_string = tokio::fs::read_to_string(&paths.manifest_path)
            .await
            .context("could not find mc.kdl file")?;
        let manifest = Manifest::from_kdl_str(&manifest_string)?;
        let document = utils::kdl::parse_document(&manifest_string)?;
        let lockfile = Lockfile::read(&paths.lockfile_path).await?;
        let lockfile_source = lockfile.to_kdl_document().to_string();

        Ok(Workspace {
            paths: paths.clone(),
            manifest,
            document,
            lockfile,
            lockfile_source
        })
    }

    /// Writes the document back, and the lockfile only when it changed, so a
    /// command that never touched it leaves no lockfile behind.
    pub async fn save(&self) -> McResult<()> {
        tokio::fs::write(&self.paths.manifest_path, self.document.to_string())
            .await
            .context("could not write mc.kdl")?;

        if self.lockfile.to_kdl_document().to_string() != self.lockfile_source {
            self.lockfile.write(&self.paths.lockfile_path).await?;
        }

        Ok(())
    }
}
