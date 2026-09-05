# mc add

```text
mc add [OPTIONS] <MOD_SLUG>...
```

Adds mods to the `mods` block of the manifest. Each `MOD_SLUG` is the
mod's identifier on the mod registry. The latest version compatible with the
configured Minecraft version and loader is looked up and pinned; the command
fails if a mod cannot be found for that combination.

A mod loader must be configured under `minecraft` before mods can be
added.

Changes take effect the next time the instance starts, which also installs
any required dependencies — see [Managing Mods](../guides/mods.md).

## Options

- `--manifest-path <PATH>` — path to `mc.kdl`. Defaults to `./mc.kdl`.

## Examples

```console
$ mc add lithium
$ mc add carpet servux
```
