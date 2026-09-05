# mc backup

```text
mc backup [OPTIONS]
```

Archives the world and stores it in the storage target configured under
the `storage` node of `backups` — see [Backups](../guides/backups.md). Works whether or
not scheduled backups are enabled.

The instance may be stopped or running. A running instance is reached over
its remote console so the world is flushed to disk before it is archived; if
the instance is running but the remote console is not available, the command
refuses rather than capture a world that is still being written to. Only one
backup can run at a time.

## Named backups

By default the archive is named after the instance and a timestamp, and is
subject to the storage's retention limit. With `--name`, the archive is
stored under the given name instead — for example
`myserver_pre-update.tar.gz` — and is kept forever: named backups never
count toward the retention limit and are never pruned. They appear in
[`mc restore --list`](restore.md) alongside automatic backups.

Names may contain ASCII letters, digits, `-` and `_`. If a backup with the
same name already exists, mc asks before overwriting it (and refuses when it
cannot ask, such as in a script).

## Cancelling

`Ctrl-C` cancels a backup in progress: the partial archive is discarded,
the instance's auto-save is re-enabled, and the command fails with a
non-zero exit status. Stored backups are never affected by a cancelled
run.

## Options

- `--name <NAME>` — store the backup under a name and keep it forever
  instead of timestamping it and rotating it out.
- `--manifest-path <PATH>` — path to `mc.kdl`. Defaults to `./mc.kdl`.

## Examples

```console
$ mc backup
$ mc backup --name pre-update
```
