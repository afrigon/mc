# Managing Mods

Mods are declared in the `mods` block of `mc.kdl` and installed when the
instance starts. Each node names a mod by its identifier on the mod registry
and pins the version to install:

```kdl
minecraft {
    version "..."
    loader "fabric"
}

mods {
    lithium "..."
}
```

A mod loader must be configured under `minecraft` for mods to be installed;
without one, the `mods` block is ignored. See
[The Manifest Format](../reference/manifest.md) for every supported form of a
mod entry, including mods fetched from a direct URL.

## Adding and removing mods

[`mc add`](../commands/add.md) and [`mc remove`](../commands/remove.md) edit
the `mods` block for you. `mc add` looks the mod up on the registry and pins
the latest version compatible with the configured Minecraft version and
loader:

```console
$ mc add sodium lithium
$ mc remove sodium
```

## Updating mods

[`mc update`](../commands/update.md) re-pins mods to the latest compatible
version — every mod in the manifest, or only the ones you name:

```console
$ mc update
$ mc update lithium
```

Mods fetched from a direct URL have no version to compare and are skipped.

## How mods are installed

Changes to `mods` take effect the next time the instance starts. On
startup, mc resolves each entry — including its required dependencies — and
records the result in the `mc.lock` lockfile. It then downloads any mod that
is missing and deletes any mod that is no longer in the lockfile.

Two consequences of this are worth knowing:

- You do not need to declare a mod's dependencies; they are resolved and
  installed automatically.
- The instance's `mods` directory is fully managed by mc. A jar placed there
  by hand is removed on the next start. To install a mod that is not on the
  registry, declare it in the manifest with a `url` property instead.
