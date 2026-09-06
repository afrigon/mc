# mc update

```text
mc update [OPTIONS] [MOD_SLUG]...
```

Re-pins mods in the manifest to the latest version compatible with the
configured Minecraft version and loader. With no arguments every mod is
updated; otherwise only the named ones. Mods already at their latest
version, and mods fetched from a URL (which carry no version), are skipped.

The new versions are installed the next time the instance starts.

## Options

- `--manifest-path <PATH>` — path to `mc.kdl`. Defaults to `./mc.kdl`.
- `--lockfile-path <PATH>` — path to `mc.lock`. Defaults to `./mc.lock`.

## Examples

```console
$ mc update
$ mc update lithium carpet
```
