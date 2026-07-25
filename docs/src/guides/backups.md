# Backups

mc can archive the instance's world on a schedule while it runs, and restore
any archive later. Backups are coordinated with the running instance so the
world is flushed to disk before it is captured — archives are always
consistent, even under load.

## Scheduled backups

```toml
[backups]
enabled = true
frequency = "0 0 * * * *"
```

With `enabled = true`, backups fire on the `frequency` schedule while the
instance runs. `frequency` is a cron expression with six fields — seconds,
minutes, hours, day of month, month, day of week. The example above backs up
at the start of every hour.

Coordination with a running instance happens over RCON, Minecraft's remote
console protocol. RCON is enabled whenever an RCON password is configured;
when backups are enabled and no password is set, mc generates one at startup
so backups work out of the box. See
[`MC_RCON_PASSWORD`](../reference/environment-variables.md) to set the
password yourself.

## Manual backups

A backup can be taken at any time with
[`mc backup`](../commands/backup.md), even when scheduled backups are
disabled — `enabled` only controls the schedule. It works against a stopped
instance, and against a running one as long as the instance was started with
an RCON password configured (always the case when backups are enabled). When
the instance is running but cannot be reached, mc refuses to back up rather
than capture a world that is still being written to.

## Interruptions

Backups are safe to interrupt. `Ctrl-C` cancels a backup in progress: the
partial archive is discarded and the instance's auto-save is re-enabled
before the error is reported — the same recovery runs when a backup fails
on its own. Stopping an instance while a scheduled backup is running
cancels the backup the same way.

A backup killed with no chance to clean up (power loss, SIGKILL) cannot
re-enable auto-save. As a last line of defense, mc turns auto-save on
whenever an instance starts, as soon as it accepts remote console
connections.

## Storage

Archives go to a storage target configured under `[backups.storage]`. Two
types are supported.

**Local** (the default) stores archives in a directory and keeps only the
`keep` most recent automatic ones:

```toml
[backups.storage]
type = "local"
path = "backups"
keep = 20
```

Archives appear in the directory atomically: an interrupted backup never
leaves a partial archive under a backup name or damages the backup it was
about to replace.

**S3** uploads archives to a bucket:

```toml
[backups.storage]
type = "s3"
bucket = "my-minecraft-backups"
```

Credentials come from the standard AWS credential chain (environment,
`~/.aws`, or an IAM role). The bucket can also be supplied with the
`MC_BACKUPS_S3_BUCKET` environment variable instead of the manifest. mc does
not prune S3 backups; use a bucket lifecycle rule to expire old archives.

Give each instance its own bucket or directory. mc treats a file in the
storage target as one of this instance's backups when its name starts with
the instance name and ends with `.tar.gz` (for example `myserver_*.tar.gz`);
anything else is ignored — never listed and never deleted.

## Keeping a backup forever

Retention only applies to automatic backups — the timestamped archives
created by the schedule or a plain `mc backup`. A backup taken with
[`mc backup --name`](../commands/backup.md) is stored as
`{instance}_{name}.tar.gz`, shows up in `mc restore --list`, and is never
pruned, no matter the `keep` limit.

Renaming an archive by hand works too: any file in the storage target named
like `myserver_important.tar.gz` — the instance name, an underscore, and a
label that is not a timestamp — is treated as a named backup: listed,
restorable, and exempt from pruning.

On S3, mc never deletes anything, and a bucket lifecycle rule cannot tell a
named backup from an automatic one; scope the rule (for example by key
prefix) if named backups must outlive it.

## Notifications

mc reports backup results — along with other instance events — to a webhook
when one is configured through the environment (`MC_DISCORD_WEBHOOK` for
Discord). The `[notifications]` section of the manifest selects which
events are sent; see
[The Manifest Format](../reference/manifest.md#notifications). A failed
notification never fails the backup itself.

## Restoring

List the available backups, then restore one:

```console
$ mc restore --list
myserver_2026-07-15_15-00-00.tar.gz (latest)
myserver_2026-07-14_15-00-00.tar.gz
$ mc restore --backup myserver_2026-07-14_15-00-00.tar.gz
```

Without `--backup`, the most recent backup is restored. The instance must be
stopped to restore. The world being replaced is set aside rather than
deleted, and is put back if the restore fails. See
[`mc restore`](../commands/restore.md).
