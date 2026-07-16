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

## Storage

Archives go to a storage target configured under `[backups.storage]`. Two
types are supported.

**Local** (the default) stores archives in a directory and keeps only the
most recent ones:

```toml
[backups.storage]
type = "local"
path = "backups"
keep = 20
```

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

Give each instance its own bucket or directory: mc treats everything in the
storage target as a backup of this instance.

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
