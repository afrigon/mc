# mc restore

```text
mc restore [OPTIONS]
```

Replaces the world with one restored from a backup. The instance must be
stopped.

The world being replaced is moved aside rather than deleted; if the restore
fails, it is put back and the command reports the failure. One set-aside
world is kept until the next restore replaces it.

## Options

- `--list` — list the available backups instead of restoring. Shows both
  automatic (timestamped) and named backups; `(latest)` marks the newest
  automatic one.
- `--backup <BACKUP>` — the backup to restore, by the filename shown by
  `--list`. Defaults to the most recent automatic backup; named backups are
  only restored when passed explicitly.
- `--manifest-path <PATH>` — path to `mc.toml`. Defaults to `./mc.toml`.

## Examples

```console
$ mc restore --list
myserver_pre-update.tar.gz
myserver_2026-07-15_15-00-00.tar.gz (latest)
myserver_2026-07-14_15-00-00.tar.gz
$ mc restore
$ mc restore --backup myserver_pre-update.tar.gz
```
