# mc minecraft

Manage Minecraft versions and mod loaders.

## mc minecraft list

```text
mc minecraft list [OPTIONS]
```

Lists Minecraft versions, most recent first. By default only the ten most
recent releases are shown, with the latest release and latest snapshot
tagged.

### Options

- `--all` — show every version instead of the ten most recent.
- `-s`, `--snapshots` — include snapshot versions.
- `-b`, `--betas` — include beta versions.
- `-a`, `--alphas` — include alpha versions.

## mc minecraft list-loaders

```text
mc minecraft list-loaders [OPTIONS]
```

Lists mod loader versions compatible with a Minecraft version, most recent
first.

### Options

- `-l`, `--loader <LOADER>` — the loader to list versions for. Defaults to
  `fabric`.
- `-m`, `--minecraft-version <VERSION>` — the Minecraft version to list
  loader versions for. Defaults to `latest`.
- `--limit <LIMIT>` — the number of results. Defaults to `10`.

## mc minecraft install

```text
mc minecraft install [OPTIONS] [VERSION]
```

Downloads a Minecraft binary into the instance's `.minecraft` directory.
`VERSION` accepts the same values as `[minecraft] version` in the manifest
(`latest`, `latest-snapshot`, or an exact version id) and defaults to
`latest`. Fails if that version is already installed.

[`mc run`](./run.md) installs the configured version automatically; this
command exists to provision one ahead of time.

### Options

- `-l`, `--loader <LOADER>` — install the binary for a mod loader instead
  of the vanilla one, as a `name` or `name@version` descriptor (for example
  `fabric`).

### Examples

```console
$ mc minecraft install
$ mc minecraft install --loader fabric
```
