use std::fs::File;
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::Context;
use fd_lock::RwLock;
use fd_lock::RwLockWriteGuard;

use crate::utils::errors::McResult;

/// An advisory, exclusive, cross-process file lock. The OS releases it if the
/// holding process dies, so a crash never leaves a stale lock behind.
pub struct FileLock {
    inner: RwLock<File>
}

impl FileLock {
    pub fn new(path: &Path) -> McResult<FileLock> {
        // Write access is required to take the exclusive lock on Windows.
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .with_context(|| format!("could not open lock file `{}`", path.display()))?;

        Ok(FileLock {
            inner: RwLock::new(file)
        })
    }

    /// Try to take the exclusive lock without blocking. `Ok(None)` means another
    /// holder currently owns it.
    pub fn try_acquire(&mut self) -> McResult<Option<RwLockWriteGuard<'_, File>>> {
        match self.inner.try_write() {
            Ok(guard) => Ok(Some(guard)),
            Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e).context("could not acquire file lock")
        }
    }
}
