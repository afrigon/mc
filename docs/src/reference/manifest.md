# The Manifest Format

The `mc.kdl` manifest at the root of an instance describes everything about
it. It is written in [KDL](https://kdl.dev). Every section except the two
top-level keys is optional; omitted keys take the defaults listed below.
Unknown keys, repeated keys, and stray values are errors.

```kdl
name "myserver"
description "A Minecraft Server"

java {
    version "graal@25"
    min-memory 4096
    max-memory 4096
    jvm-arguments "-Djava.net.preferIPv6Addresses=true" "-XX:+AlwaysPreTouch" "-Djdk.graal.TuneInlinerExploration=1"
}

minecraft {
    version "latest"
    loader "fabric"
}

server {
    gamemode "survival"
    difficulty "normal"
    level-type "minecraft:normal"
    hardcore #false
    allow-list #true
    port 25565
    rcon-port 25575
    capacity 20
    view-distance 16
    simulation-distance 16
    eula #true

    properties {
        spawn-protection 0
    }
}

mods {
    modrinth {
        lithium "..."
    }
}

backups {
    on
    frequency "0 0 * * * *"
    keep 20
    local "backups"
}

notifications {
    on-lifecycle-event #true
    on-panic #true
    on-sigkill #true
    on-backup #true
    on-backup-failure #true
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
opt-in: when it is absent, no tunnel agent is installed or started. A bare
`tunnel` node enables it with the defaults.

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
- `allow-list` — whether players must be on the allow list before they can
  join. Defaults to `#true`. (`white-list`) Enabling it also sets
  `enforce-whitelist`, so a player removed from the list is kicked at once;
  disabling it clears both. `enforce-whitelist` is not managed, so an entry
  for it in `properties` overrides that half on its own.
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
        spawn-protection 16
        enforce-whitelist #false
        "query.port" 25565
    }
}
```

Values may be strings, integers, floats, or booleans. Keys containing a dot
must be quoted, as above; a nested block spells the same key, so
`rcon { broadcast "yes" }` sets `rcon.broadcast`.

Keys managed by mc are rejected: an entry for a key that a `server` field
or the top-level `name` and `description` already drive is an error naming
the field to use instead. `enable-rcon` is rejected too — RCON is enabled
exactly when an RCON password is configured (see
[Environment Variables](./environment-variables.md)).

Every key not written here takes the vanilla default as of Minecraft 26.3,
listed on the [`server.properties`](https://minecraft.wiki/w/Server.properties)
wiki page, with three exceptions:

- `enforce-whitelist` — follows `allow-list`, so `#true` by default.
- `server-ip` — `"::"`, all addresses, IPv6 included; driven by `ip`.
- `spawn-protection` — `0`.

## `mods`

The mods to install, grouped by where they come from. Inside a group, each
node is named after the mod and carries where to fetch it:

```kdl
mods {
    // mods from the registry, pinned to a version identifier
    modrinth {
        lithium "..."
        carpet "..."
    }

    // jars fetched from a URL, for mods not on the registry
    http {
        my-mod "https://example.com/my-mod.jar"
    }
}
```

`modrinth` maps a mod's identifier (slug) on the registry to the version
identifier to install. `http` maps a name of your choosing to the URL of a
jar. A name may appear in only one group; it becomes the jar's filename.

[`mc add`](../commands/add.md), [`mc remove`](../commands/remove.md), and
[`mc update`](../commands/update.md) edit the `modrinth` group for you and
pin compatible versions. Required dependencies are resolved automatically
when the instance starts and recorded in the `mc.lock` lockfile — see
[Managing Mods](../guides/mods.md).

## `backups`

Scheduled world backups, taken while the instance runs.

- `on` — a bare flag: when present, backups run on the `frequency` schedule
  while the instance runs. Comment it out or remove it to disable the
  schedule. Manual [`mc backup`](../commands/backup.md) works regardless.
- `frequency` — a cron expression with six fields: seconds, minutes, hours,
  day of month, month, day of week. Defaults to `"0 0 * * * *"` (hourly).
- `keep` — the number of most-recent automatic archives to keep in local
  storage; older ones are pruned. Defaults to `20`. S3 storage is never
  pruned.
- `local` — the directory archives are stored in. This is the default
  storage, under `backups`, when neither `local` nor `s3` is written.
- `s3` — the bucket archives are uploaded to, with an optional `region`
  property. Credentials come from the standard AWS credential chain, and
  `MC_BACKUPS_S3_BUCKET` overrides the bucket. Without `region`, the region
  also comes from the credential chain.

Write at most one of `local` and `s3`.

```kdl
backups {
    on
    frequency "0 0 * * * *"
    keep 20
    local "/mnt/data/mc"
}
```

```kdl
backups {
    on
    s3 "my-minecraft-backups" region="us-east-1"
}
```

See the [Backups](../guides/backups.md) guide for the full picture.

## `players`

Who may join, who is kept out, and who holds server commands. Each group
lists players as nodes named after them; a name that starts with a digit
must be quoted. Options ride on the node as properties.

```kdl
players {
    allow {
        Notch
        "123abc"
    }

    ban {
        Griefer reason="stole the beacon" created="2026-09-06T14:00:00Z" expires="2026-10-01T00:00:00Z"
    }

    ban-ip {
        "203.0.113.7" reason="bot traffic"
    }

    op {
        Notch level=4 bypasses-player-limit=#true
        jeb_
    }
}
```

- `allow` — players permitted to join while `allow-list` in `server` is
  `#true`, which it is by default. Entries take no properties.
- `ban` — players refused by the server. `reason` is shown to the player
  and defaults to the server's own wording; `created` records when the ban
  was issued; `expires` lifts the ban at that time. Both dates are RFC 3339
  timestamps. A player cannot be in both `allow` and `ban`.
- `ban-ip` — addresses refused by the server, with the same properties as
  `ban`. Node names are IP addresses and must be quoted.
- `op` — operators. `level` is the permission level from `1` to `4` and
  defaults to the server's `op-permission-level` property;
  `bypasses-player-limit` lets the operator join when the server is full
  and defaults to `#false`. Operators may join regardless of the allow
  list.

[`mc allow`](../commands/allow.md), [`mc ban`](../commands/ban.md), and
[`mc op`](../commands/op.md) edit these groups for you and look names up as
they go. The lists are written to the instance on every start — see
[Players](../guides/players.md).

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
