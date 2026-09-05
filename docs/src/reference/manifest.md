# The Manifest Format

The `mc.kdl` manifest at the root of an instance describes everything about
it. It is written in [KDL](https://kdl.dev). Every section except the two
top-level keys is optional; omitted keys take the defaults listed below.
Unknown or repeated keys are an error.

```kdl
name "myserver"
description "A Minecraft Server"

java {
    version "graal@25"
    min-memory 4096
    max-memory 4096
}

minecraft {
    version "..."
    loader "fabric"
}

server {
    gamemode "survival"
    difficulty "normal"
    eula #true

    properties {
        white-list #false
    }
}

mods {
    lithium "..."
}

backups {
    enabled #true
    frequency "0 0 * * * *"
    storage "local" path="backups" keep=20
}
```

## `name` (required)

The instance name. It names the world directory inside the instance and is
used as the world's `level-name`. Must be usable as a directory name.

## `description` (required)

A short description, used as the message of the day (`motd`) shown in the
multiplayer list.

## `java`

The Java runtime used to launch the instance.

- `version` — the runtime to install and use, as a `vendor@version`
  descriptor. Defaults to `"graal@25"`. Run
  [`mc java list`](../commands/java.md) to see the available runtimes; the
  one marked `(recommended)` is the default.
- `min-memory` — initial heap size in megabytes. Defaults to `4096`.
- `max-memory` — maximum heap size in megabytes. Defaults to `4096`.
- `jvm-arguments` — extra arguments passed to the JVM. Defaults to a small
  set of tuned flags; setting this key replaces the defaults entirely.

```kdl
java {
    version "graal@25"
    min-memory 8192
    max-memory 8192
    jvm-arguments "-XX:+AlwaysPreTouch"
}
```

## `minecraft`

The Minecraft version and mod loader.

- `version` — the Minecraft version to run. Defaults to `"latest"`, the
  latest release. `"latest-snapshot"` selects the latest snapshot, and any
  exact version id (see [`mc minecraft list`](../commands/minecraft.md))
  pins that version. Aliases are resolved every start, so an instance on
  `"latest"` upgrades itself when a new release comes out — pin an exact
  version if that is not what you want.
- `loader` — the mod loader, as a `name` or `name@version` descriptor
  (for example `"fabric"`, which resolves to the latest loader version for
  the configured Minecraft version). When omitted, the instance runs without
  a loader and the `mods` block is ignored. Run
  [`mc minecraft list-loaders`](../commands/minecraft.md) to see loader
  versions.

## `tunnel`

Exposes the instance to players outside the local network through a tunnel
provider; see the [Tunnels](../guides/tunnel.md) guide. The section is
opt-in: when it is absent, no tunnel agent is installed or started.

- `provider` — the tunnel provider, as a `name` or `name@version`
  descriptor. `playit` is the accepted provider. Defaults to `"playit"`,
  which resolves to the latest agent version on every start; pin an exact
  version, such as `"playit@1.0.10"`, to hold it. Run
  [`mc tunnel list`](../commands/tunnel.md) to see the available versions.

```kdl
tunnel {
    provider "playit"
}
```

## `server`

Settings mc manages for the server. Each maps to a `server.properties` key,
listed in parentheses; because these are managed here, they cannot be set
through `properties`.

- `gamemode` — `"survival"`, `"creative"`, `"adventure"`, or `"spectator"`.
  Defaults to `"survival"`. (`gamemode`)
- `difficulty` — `"peaceful"`, `"easy"`, `"normal"`, or `"hard"`. Defaults
  to `"normal"`. (`difficulty`)
- `level-type` — `"minecraft:normal"`, `"minecraft:flat"`,
  `"minecraft:large_biomes"`, `"minecraft:amplified"`, or
  `"minecraft:single_biome_surface"`. Defaults to `"minecraft:normal"`.
  (`level-type`)
- `hardcore` — defaults to `#false`. (`hardcore`)
- `seed` — the world seed, as an integer or a string. Random when omitted.
  (`level-seed`)
- `eula` — indicates that YOU have read and agree to the
  [Minecraft EULA](https://aka.ms/MinecraftEULA). The instance refuses to
  start until this is `#true`. Defaults to `#false`.
- `ip` — the address to bind. When omitted, the server binds all addresses,
  IPv6 included. (`server-ip`)
- `port` — the game port. Defaults to `25565`. (`server-port`)
- `rcon-port` — the remote console port. Defaults to `25575`. (`rcon.port`)
- `capacity` — the maximum number of players. Defaults to `20`.
  (`max-players`)
- `view-distance` — in chunks. Defaults to `16`. (`view-distance`)
- `simulation-distance` — in chunks. Defaults to `16`.
  (`simulation-distance`)

### `properties`

A block inside `server` holding overrides for any other
[`server.properties`](https://minecraft.wiki/w/Server.properties) key. mc
generates the file on every start — hand edits do not survive — so this
block is the way to reach settings that have no `server` field:

```kdl
server {
    properties {
        white-list #false
        spawn-protection 16
        "query.port" 25565
    }
}
```

Values may be strings, integers, floats, or booleans. Keys containing a dot
must be quoted, as above; a nested block spells the same key, so
`rcon { broadcast "yes" }` sets `rcon.broadcast`.

Keys managed by mc take precedence: an entry that conflicts with a value
derived from the manifest or the environment is ignored with a warning.
`enable-rcon` is always ignored — RCON is enabled exactly when an RCON
password is configured (see
[Environment Variables](./environment-variables.md)).

Note two defaults that differ from a vanilla server: the allow list is
enabled (`white-list`), and the server binds all addresses, IPv6 included
(`server-ip`).

## `mods`

The mods to install, one node per mod, named by the mod's identifier (slug)
on the mod registry. Three forms are supported:

```kdl
mods {
    // a version identifier on the registry
    lithium "..."

    // the same, naming the registry
    carpet "..." service="modrinth"

    // a jar fetched from a URL, for mods not on the registry
    my-mod url="https://example.com/my-mod.jar"
}
```

`service` names the registry hosting the mod and defaults to `"modrinth"`.

[`mc add`](../commands/add.md), [`mc remove`](../commands/remove.md), and
[`mc update`](../commands/update.md) edit this block for you and pin
compatible versions. Required dependencies are resolved automatically when
the instance starts and recorded in the `mc.lock` lockfile — see
[Managing Mods](../guides/mods.md).

## `backups`

Scheduled world backups, taken while the instance runs.

- `enabled` — schedule backups while the instance runs. Defaults to
  `#false`. Manual [`mc backup`](../commands/backup.md) works regardless.
- `frequency` — a cron expression with six fields: seconds, minutes, hours,
  day of month, month, day of week. Defaults to `"0 0 * * * *"` (hourly).
- `storage` — where archives are stored; see below. When the node is
  omitted, archives go to local storage under `backups`, keeping the 20 most
  recent.

### `storage`

Where archives are stored. The first value selects the backend:

```kdl
backups {
    // path is required once the node is written; keep shown at its default
    storage "local" path="backups" keep=20
}
```

```kdl
backups {
    storage "s3" bucket="my-minecraft-backups"
}
```

For `local`, `path` is required and `keep` is the number of most-recent
automatic archives to keep. For `s3`, the bucket may also come from the
`MC_BACKUPS_S3_BUCKET` environment variable, and credentials come from the
standard AWS credential chain. The default is local storage.

See the [Backups](../guides/backups.md) guide for the full picture.

## `notifications`

Webhook notifications about the instance. A provider is activated by
setting its webhook environment variable (`MC_DISCORD_WEBHOOK` for
Discord) — the URL is a secret and is never read from the manifest, and
without one no notifications are sent. This block selects which events are
reported; every key defaults to `#true`:

- `on-lifecycle-event` — the instance started or stopped. When a tunnel is
  configured, the started message includes the address to join at.
- `on-panic` — the instance crashed.
- `on-sigkill` — the instance was forced down without a clean save, because
  it did not stop within the grace period or a second stop signal arrived.
- `on-backup` — a backup completed.
- `on-backup-failure` — a backup failed.

```kdl
notifications {
    on-lifecycle-event #false
    on-backup #false
}
```
