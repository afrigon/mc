# mc backup

```text
mc backup [OPTIONS]
```

Archives the world and stores it in the storage target configured under
`[backups.storage]` — see [Backups](../guides/backups.md). Works whether or
not scheduled backups are enabled.

The instance may be stopped or running. A running instance is reached over
its remote console so the world is flushed to disk before it is archived; if
the instance is running but the remote console is not available, the command
refuses rather than capture a world that is still being written to. Only one
backup can run at a time.

## Options

- `--manifest-path <PATH>` — path to `mc.toml`. Defaults to `./mc.toml`.

## Examples

```console
$ mc backup
```
