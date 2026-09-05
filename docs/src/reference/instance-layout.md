# Instance Layout

An instance is a directory with `mc.kdl` at its root. All mc commands run
from that directory, and everything the instance needs lives under it:

```text
myserver/
├── mc.kdl
├── mc.lock
├── .java/
├── .minecraft/
├── backups/
├── instance/
│   ├── eula.txt
│   ├── server.properties
│   ├── mods/
│   └── myserver/
└── temp/
```

Only one running server is allowed per instance directory; a second `mc run`
in the same directory refuses to start.

## `mc.kdl`

The manifest describing the instance — the only file you author. See
[The Manifest Format](./manifest.md).

## `mc.lock`

The mod lockfile, written in KDL: the fully resolved set of mods, including
required dependencies, that the instance last started with. Regenerated when
the instance starts.

## `.java/` and `.minecraft/`

Installed Java runtimes and Minecraft binaries, keyed by version, shared by
every start of this instance. Safe to delete while the instance is stopped;
whatever is missing is downloaded again on the next start.

## `backups/`

The default destination for backup archives when local storage is used. The
directory is dedicated to this instance's backups; see
[Backups](../guides/backups.md).

## `instance/`

The live working directory of the Minecraft process. Notable contents:

- `eula.txt` — generated from the `eula` key in the manifest.
- `server.properties` — generated from the manifest on every start; manual
  edits are overwritten. Use the `properties` block of the `server` section
  in `mc.kdl` instead. When the file holds secrets, such as an RCON password, mc creates
  it readable only by the owning user and refuses to start if its
  permissions have been loosened.
- `mods/` — the installed mods. This directory is fully managed: mc adds and
  removes jars to match the lockfile, so a jar placed here by hand is
  deleted on the next start. Declare URL mods in the manifest instead.
- `myserver/` — the world, named after the instance's `name`. Everything
  else Minecraft writes at runtime (logs, player data, allow list, and so on)
  also lives here and is left untouched by mc.

## `temp/`

Scratch space for in-flight downloads and archives. Cleared when the
instance starts; safe to delete while nothing is running.
