# mc remove

```text
mc remove [OPTIONS] <MOD_SLUG>...
```

Removes mods from the `[mods]` table of the manifest. A slug that is not in
the manifest is reported and skipped.

The jars are uninstalled the next time the instance starts. A removed mod's
dependencies are not kept: anything no longer required disappears from the
lockfile and is uninstalled along with it.

## Options

- `--manifest-path <PATH>` — path to `mc.toml`. Defaults to `./mc.toml`.

## Examples

```console
$ mc remove carpet
```
