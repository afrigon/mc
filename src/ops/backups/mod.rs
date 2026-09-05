pub mod local;
pub mod s3;

use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;

use anyhow::Context;
use chrono::NaiveDateTime;
use kdl::KdlNode;
use minecraft_client_rs::Client;
use tempfile::NamedTempFile;
use tokio_util::sync::CancellationToken;

use crate::context::McContext;
use crate::manifest::FromKdlNode;
use crate::ops::backups::local::LocalBackupBackend;
use crate::ops::backups::s3::S3BackupBackend;
use crate::ops::lock::InstanceLocks;
use crate::ops::notifications::Notifier;
use crate::utils;
use crate::utils::errors::McResult;
use crate::utils::shell::Shell;

pub trait BackupBackend {
    /// Store `archive` under `filename`.
    async fn backup(&self, filename: &str, archive: NamedTempFile) -> McResult<()>;

    /// List the filenames of every backup belonging to this instance
    /// (`{world}_*.tar.gz`), newest first. Unrelated files in the storage
    /// location (bucket or directory) are ignored.
    async fn list(&self) -> McResult<Vec<String>>;

    /// Download `filename` and extract it into `output`. `output` must not
    /// already exist; the caller is responsible for clearing it first.
    /// `staging` is a scratch directory on the same filesystem as `output`.
    async fn restore(
        &self,
        context: &mut McContext,
        filename: &str,
        output: &Path,
        staging: &Path
    ) -> McResult<()>;
}

/// Concrete backend dispatch. `BackupBackend` has `async fn` methods and is
/// therefore not `dyn`-compatible, so we enum-dispatch over the variants
/// instead of using `Box<dyn BackupBackend>`.
pub enum Backend {
    S3(S3BackupBackend),
    Local(LocalBackupBackend)
}

impl Backend {
    pub fn from_storage(storage: &BackupStorage, world_name: &str) -> McResult<Backend> {
        let backend = match storage {
            BackupStorage::S3 { bucket } => {
                let bucket = bucket.clone().context(
                    "no S3 bucket is configured; set `bucket` on the `storage` node of `backups` in mc.kdl or the MC_BACKUPS_S3_BUCKET environment variable"
                )?;

                Backend::S3(S3BackupBackend::new(bucket, world_name.to_string()))
            }
            BackupStorage::Local { path, keep } => {
                if *keep == 0 {
                    anyhow::bail!(
                        "`keep` must be at least 1; set a positive number of backups to retain, or remove it to use the default"
                    );
                }

                Backend::Local(LocalBackupBackend::new(
                    path.clone(),
                    *keep,
                    world_name.to_string()
                ))
            }
        };

        Ok(backend)
    }
}

impl BackupBackend for Backend {
    async fn backup(&self, filename: &str, archive: NamedTempFile) -> McResult<()> {
        match self {
            Backend::S3(backend) => backend.backup(filename, archive).await,
            Backend::Local(backend) => backend.backup(filename, archive).await
        }
    }

    async fn list(&self) -> McResult<Vec<String>> {
        match self {
            Backend::S3(backend) => backend.list().await,
            Backend::Local(backend) => backend.list().await
        }
    }

    async fn restore(
        &self,
        context: &mut McContext,
        filename: &str,
        output: &Path,
        staging: &Path
    ) -> McResult<()> {
        match self {
            Backend::S3(backend) => backend.restore(context, filename, output, staging).await,
            Backend::Local(backend) => backend.restore(context, filename, output, staging).await
        }
    }
}

/// A backup belonging to this instance: `{world}_{anything}.tar.gz`. Both
/// automatic backups and named ones (created with `--name` or by manually
/// renaming a file to this shape) qualify; anything else in the storage
/// location is invisible to mc and never touched.
fn is_instance_backup(filename: &str, world_name: &str) -> bool {
    filename
        .strip_prefix(world_name)
        .and_then(|rest| rest.strip_prefix('_'))
        .is_some_and(|rest| rest.ends_with(".tar.gz"))
}

/// An automatic backup: `{world}_{timestamp}.tar.gz`. Only these are subject
/// to retention pruning and eligible as the implicit "latest" restore target.
fn is_automatic_backup(filename: &str, world_name: &str) -> bool {
    filename
        .strip_prefix(world_name)
        .and_then(|rest| rest.strip_prefix('_'))
        .and_then(|rest| rest.strip_suffix(".tar.gz"))
        .is_some_and(|timestamp| {
            NaiveDateTime::parse_from_str(timestamp, utils::date::FILENAME_DATE_FORMAT).is_ok()
        })
}

fn validate_backup_name(name: &str) -> McResult<()> {
    let valid = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

    if !valid {
        anyhow::bail!("backup names may only contain ASCII letters, digits, `-` and `_`");
    }

    if NaiveDateTime::parse_from_str(name, utils::date::FILENAME_DATE_FORMAT).is_ok() {
        anyhow::bail!(
            "backup name `{}` would be mistaken for an automatic backup timestamp; pick a different name",
            name
        );
    }

    Ok(())
}

#[derive(Clone)]
pub enum BackupStorage {
    S3 {
        /// May be omitted in `mc.kdl` and supplied via `MC_BACKUPS_S3_BUCKET`.
        bucket: Option<String>
    },
    Local {
        path: PathBuf,

        /// Keep only this many most-recent backups, pruning older ones.
        keep: usize
    }
}

impl FromKdlNode for BackupStorage {
    fn from_kdl_node(node: &KdlNode) -> McResult<Self> {
        if node.children().is_some() {
            anyhow::bail!("the `storage` node does not take children");
        }

        let kind = match utils::kdl::arguments(node).as_slice() {
            [kind] => kind
                .as_string()
                .ok_or_else(|| anyhow::anyhow!("the storage kind must be a string"))?,
            _ => anyhow::bail!("the `storage` node takes a single kind, `local` or `s3`")
        };

        match kind {
            "local" => {
                utils::kdl::check_properties(node, &["path", "keep"])?;

                Ok(BackupStorage::Local {
                    path: utils::kdl::string_property(node, "path")?
                        .map(PathBuf::from)
                        .ok_or_else(|| anyhow::anyhow!("local storage requires a `path`"))?,
                    keep: utils::kdl::integer_property(node, "keep")?.unwrap_or(20)
                })
            }
            "s3" => {
                utils::kdl::check_properties(node, &["bucket"])?;

                Ok(BackupStorage::S3 {
                    bucket: utils::kdl::string_property(node, "bucket")?.map(str::to_owned)
                })
            }
            _ => anyhow::bail!("unknown storage kind `{}`, expected `local` or `s3`", kind)
        }
    }
}

pub struct BackupOptions {
    pub project_path: PathBuf,
    pub world_path: PathBuf,
    pub storage: BackupStorage,
    pub rcon_port: u16,
    pub rcon_password: Option<String>,
    pub notifier: Option<Notifier>,
    pub name: Option<String>,
    pub shell: Arc<Mutex<Shell>>,
    pub cancel: CancellationToken
}

pub async fn backup(options: &BackupOptions) -> McResult<()> {
    let result = run_backup(options).await;

    if let Some(notifier) = &options.notifier {
        let world_name = options
            .world_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("world");

        notifier.notify_backup(world_name, &result).await;
    }

    result
}

async fn run_backup(options: &BackupOptions) -> McResult<()> {
    let locks = InstanceLocks::new(&options.project_path);

    // Only one backup may run at a time for an instance.
    let mut backup_lock = locks.backup()?;
    let backup_guard = backup_lock
        .try_acquire()?
        .context("a backup is already in progress for this instance")?;

    let world_name = options
        .world_path
        .file_name()
        .and_then(|name| name.to_str())
        .context("could not determine world name from path")?;

    let backend = Backend::from_storage(&options.storage, world_name)?;

    if !tokio::fs::try_exists(&options.world_path).await? {
        anyhow::bail!(
            "there is no world to back up yet at `{}`",
            options.world_path.display()
        );
    }

    let filename = match &options.name {
        Some(name) => {
            validate_backup_name(name)?;

            let filename = format!("{}_{}.tar.gz", world_name, name);

            if backend.list().await?.contains(&filename) {
                let question = format!(
                    "a backup named `{}` already exists, overwrite it?",
                    filename
                );
                let overwrite = {
                    let mut shell = options.shell.lock().unwrap_or_else(PoisonError::into_inner);

                    utils::prompt::confirm(&mut shell, &question)?
                };

                if !overwrite {
                    anyhow::bail!("a backup named `{}` already exists", filename);
                }
            }

            filename
        }
        None => format!(
            "{}_{}.tar.gz",
            world_name,
            utils::date::filename_date_string()
        )
    };

    {
        let mut shell = options.shell.lock().unwrap_or_else(PoisonError::into_inner);

        _ = shell.status("Archiving", format!("world to `{}`", filename));
    }

    let mut world_lock = locks.world()?;

    // TODO: gate rcon use on the instance's Minecraft version capabilities
    // (Capability::RemoteConsole); versions older than RCON cannot be flushed.
    //
    // A successful rcon connection means the server is live (the password is the
    // one it is running with).
    let mut rcon = options
        .rcon_password
        .as_deref()
        .and_then(|password| connect_rcon(options.rcon_port, password));

    // A running server is flushed over rcon. A server at rest exposes no rcon, so
    // we take the world lock ourselves to keep it stopped while we read the world
    // into the archive. If that lock is already held while rcon is unavailable,
    // the server is up but rcon is misconfigured, so we refuse rather than
    // capture a torn snapshot.
    let world_guard = if let Some(rcon) = &mut rcon {
        rcon.send_command("save-off".to_string())
            .map_err(|_| anyhow::anyhow!("could not disable auto-save over rcon"))?;

        None
    } else {
        Some(world_lock.try_acquire()?.context(
            "the server appears to be running but rcon is unavailable; set MC_RCON_PASSWORD and enable rcon, or stop the server before backing up"
        )?)
    };

    let staging = options.project_path.join("temp");

    // Auto-save is disabled from here until the save-on below: the outcome is
    // collected instead of propagated so it is always re-enabled first.
    let archive_result = archive_world(options, &mut rcon, &staging).await;

    if archive_result.is_err() && options.cancel.is_cancelled() {
        let mut shell = options.shell.lock().unwrap_or_else(PoisonError::into_inner);

        _ = shell.status("Cancelling", "discarding the partial archive");
    }

    if let Some(rcon) = &mut rcon {
        let restored = rcon.send_command("save-on".to_string());
        let _ = rcon.close();

        if restored.is_err() {
            if archive_result.is_ok() {
                anyhow::bail!("could not re-enable auto-save over rcon");
            }

            let mut shell = options.shell.lock().unwrap_or_else(PoisonError::into_inner);

            _ = shell.warn("could not re-enable auto-save over rcon");
        }
    }

    // The world is fully archived; release it so the server can start again.
    drop(world_guard);

    let archive = archive_result?;

    backend.backup(&filename, archive).await?;

    {
        let mut shell = options.shell.lock().unwrap_or_else(PoisonError::into_inner);

        _ = shell.status("Finished", "archiving world");
    }

    let _ = tokio::fs::remove_dir(&staging).await;

    // Keep backups serialized until the upload finishes.
    drop(backup_guard);

    Ok(())
}

/// The span while auto-save is disabled: flush the world, then archive it,
/// racing cancellation. The caller collects the outcome instead of `?`-ing so
/// `save-on` is never skipped.
async fn archive_world(
    options: &BackupOptions,
    rcon: &mut Option<Client>,
    staging: &Path
) -> McResult<NamedTempFile> {
    if let Some(rcon) = rcon {
        rcon.send_command("save-all flush".to_string())
            .map_err(|_| anyhow::anyhow!("could not flush the world to disk over rcon"))?;
    }

    tokio::fs::create_dir_all(staging).await?;

    let exclude = HashSet::from([PathBuf::from("session.lock")]);

    tokio::select! {
        archive = utils::archive::inflate_tar_gz(options.world_path.clone(), &exclude, staging) => {
            archive
        }
        _ = options.cancel.cancelled() => Err(anyhow::anyhow!("the backup was cancelled"))
    }
}

pub struct ListOptions {
    pub storage: BackupStorage,
    pub world_name: String
}

pub async fn list(context: &mut McContext, options: &ListOptions) -> McResult<()> {
    let backend = Backend::from_storage(&options.storage, &options.world_name)?;
    let backups = backend.list().await?;

    if backups.is_empty() {
        _ = context.shell().warn("no backups were found");

        return Ok(());
    }

    // The implicit restore target is the newest automatic backup, which is not
    // necessarily the first entry once named backups are in the sort.
    let latest = backups
        .iter()
        .position(|backup| is_automatic_backup(backup, &options.world_name));

    let mut shell = context.shell();
    let stdout = shell.out();

    for (i, backup) in backups.iter().enumerate() {
        if Some(i) == latest {
            writeln!(stdout, "{} (latest)", backup)?;
        } else {
            writeln!(stdout, "{}", backup)?;
        }
    }

    Ok(())
}

pub struct RestoreOptions {
    pub project_path: PathBuf,
    pub world_path: PathBuf,
    pub storage: BackupStorage,
    pub version: Option<String>,
    pub cancel: CancellationToken
}

pub async fn restore(context: &mut McContext, options: &RestoreOptions) -> McResult<()> {
    let locks = InstanceLocks::new(&options.project_path);

    // Restoring needs exclusive ownership of the world: the server must be
    // stopped (it holds the world lock while running) and no other restore may be
    // running.
    let mut world_lock = locks.world()?;
    let world_guard = world_lock.try_acquire()?.context(
        "cannot restore while the server is running or another operation owns the world; stop the server first"
    )?;

    // Keep backups from running against the world we are about to replace.
    let mut backup_lock = locks.backup()?;
    let backup_guard = backup_lock
        .try_acquire()?
        .context("a backup is in progress; wait for it to finish before restoring")?;

    let world_name = options
        .world_path
        .file_name()
        .and_then(|name| name.to_str())
        .context("could not determine world name from path")?;

    let backend = Backend::from_storage(&options.storage, world_name)?;

    let backups = backend.list().await?;

    if backups.is_empty() {
        anyhow::bail!("no backups were found to restore");
    }

    let filename = match &options.version {
        Some(version) => backups
            .iter()
            .find(|name| name.as_str() == version)
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "backup `{}` was not found, available backups are:\n  {}",
                    version,
                    backups.join("\n  ")
                )
            })?,
        // `list` is sorted newest first, so the first automatic entry is the
        // latest one. Named backups are only ever restored explicitly.
        None => backups
            .iter()
            .find(|name| is_automatic_backup(name, world_name))
            .cloned()
            .context(
                "no automatic backups were found to restore; pass --backup <filename> to restore a named backup"
            )?
    };

    _ = context
        .shell()
        .status("Restoring", format!("world from `{}`", filename));

    // Move the current world aside rather than deleting it so a failed restore
    // can be rolled back, and so the extraction target does not already exist.
    // The name is static, so each restore replaces the aside left by the
    // previous one and at most one is kept around.
    let aside = if tokio::fs::try_exists(&options.world_path).await? {
        let aside = options.world_path.with_extension("restore.bak");

        if tokio::fs::try_exists(&aside).await? {
            tokio::fs::remove_dir_all(&aside).await?;
        }

        tokio::fs::rename(&options.world_path, &aside).await?;

        Some(aside)
    } else {
        None
    };

    let staging = options.project_path.join("temp");
    tokio::fs::create_dir_all(&staging).await?;

    // Cancellation surfaces as an error so the rollback below runs and the
    // partially extracted world is never left in place.
    let restore_result = tokio::select! {
        result = backend.restore(context, &filename, &options.world_path, &staging) => result,
        _ = options.cancel.cancelled() => Err(anyhow::anyhow!("the restore was cancelled"))
    };

    let result = match restore_result {
        Ok(()) => {
            _ = context.shell().status("Finished", "world restored");

            Ok(())
        }
        Err(error) => {
            // Roll the previous world back into place so a failed restore is not
            // destructive.
            if let Some(aside) = aside {
                let _ = tokio::fs::remove_dir_all(&options.world_path).await;

                tokio::fs::rename(&aside, &options.world_path)
                    .await
                    .context(
                        "the restore failed and the original world could not be rolled back"
                    )?;
            }

            Err(error).context("failed to restore the backup; the original world was kept")
        }
    };

    let _ = tokio::fs::remove_dir(&staging).await;

    drop(backup_guard);
    drop(world_guard);

    result
}

pub struct UndoRestoreOptions {
    pub project_path: PathBuf,
    pub world_path: PathBuf
}

/// Puts the set-aside world from the last restore back in place. When a world
/// is currently present the two are swapped, so undoing twice returns to the
/// starting state and nothing is ever deleted.
pub async fn undo_restore(context: &mut McContext, options: &UndoRestoreOptions) -> McResult<()> {
    let locks = InstanceLocks::new(&options.project_path);

    let mut world_lock = locks.world()?;
    let world_guard = world_lock.try_acquire()?.context(
        "cannot undo a restore while the server is running or another operation owns the world; stop the server first"
    )?;

    let mut backup_lock = locks.backup()?;
    let backup_guard = backup_lock
        .try_acquire()?
        .context("a backup is in progress; wait for it to finish before undoing a restore")?;

    let aside = options.world_path.with_extension("restore.bak");

    if !tokio::fs::try_exists(&aside).await? {
        anyhow::bail!(
            "there is no set-aside world at `{}` to put back",
            aside.display()
        );
    }

    if tokio::fs::try_exists(&options.world_path).await? {
        let swap = options.world_path.with_extension("restore.swap");

        if tokio::fs::try_exists(&swap).await? {
            tokio::fs::remove_dir_all(&swap).await?;
        }

        tokio::fs::rename(&options.world_path, &swap).await?;
        tokio::fs::rename(&aside, &options.world_path).await?;
        tokio::fs::rename(&swap, &aside).await?;

        _ = context.shell().status(
            "Finished",
            "swapped the world with the set-aside copy; undo again to swap back"
        );
    } else {
        tokio::fs::rename(&aside, &options.world_path).await?;

        _ = context
            .shell()
            .status("Finished", "moved the set-aside world back into place");
    }

    drop(backup_guard);
    drop(world_guard);

    Ok(())
}

pub fn connect_rcon(port: u16, password: &str) -> Option<minecraft_client_rs::Client> {
    let rcon_address = format!("127.0.0.1:{}", port);

    let mut client = minecraft_client_rs::Client::new(rcon_address).ok()?;

    if client.authenticate(password.to_string()).is_err() {
        let _ = client.close();

        None
    } else {
        Some(client)
    }
}
