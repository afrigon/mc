use std::path::Path;
use std::path::PathBuf;

use crate::utils::errors::McResult;
use crate::utils::lock::FileLock;

const WORLD_LOCK: &str = "mc.world.lock";
const BACKUP_LOCK: &str = "mc.backup.lock";

/// Factory for an instance's advisory locks. The lock files live at the project
/// root, next to `mc.kdl`.
pub struct InstanceLocks {
    project_path: PathBuf
}

impl InstanceLocks {
    pub fn new(project_path: &Path) -> InstanceLocks {
        InstanceLocks {
            project_path: project_path.to_path_buf()
        }
    }

    /// Exclusive ownership of the world. Held by `mc run` for its lifetime and by
    /// `mc restore` for the duration of a restore.
    pub fn world(&self) -> McResult<FileLock> {
        FileLock::new(&self.project_path.join(WORLD_LOCK))
    }

    /// Serializes backups so only one runs at a time per instance.
    pub fn backup(&self) -> McResult<FileLock> {
        FileLock::new(&self.project_path.join(BACKUP_LOCK))
    }
}
