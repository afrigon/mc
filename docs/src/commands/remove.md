# mc remove

```text
mc remove [OPTIONS] <MOD_SLUG>...
```

Removes mods from the manifest's `mods` block, whichever group lists them. A slug that is not in
the manifest is reported and skipped.

The jars are uninstalled the next time the instance starts. A removed mod's
dependencies are not kept: anything no longer required disappears from the
lockfile and is uninstalled along with it.

## Options

- `--manifest-path <PATH>` — path to `mc.kdl`. Defaults to `./mc.kdl`.
- `--lockfile-path <PATH>` — path to `mc.lock`. Defaults to `./mc.lock`.

## Examples

```console
$ mc remove carpet
```
