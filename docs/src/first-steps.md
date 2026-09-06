# First Steps with mc

This walkthrough creates a new instance and runs it.

## Create an instance

Each instance lives in its own directory, with a `mc.kdl` manifest at its
root. [`mc init`](./commands/init.md) creates both:

```console
$ mc init myserver
$ cd myserver
```

The generated manifest looks like this:

```kdl
name "myserver"
description "A Minecraft Server"

minecraft {
    version "..."
    loader "fabric"
}

server {
    gamemode "survival"
    difficulty "normal"
    hardcore #false

    // Setting this to true indicates YOU have read and agree to the Minecraft EULA (https://aka.ms/MinecraftEULA).
    // This agreement is between you and Mojang/Microsoft.
    eula #false
}

backups {
    on
    frequency "0 0 * * * *"
}

mods {
    modrinth {
        lithium "..."
    }
}
```

By default `mc init` uses the `optimized` preset, which adds a mod loader and
a small set of performance mods. Pass `--preset vanilla` for an unmodded
instance. The manifest is yours to edit; every key is documented in
[The Manifest Format](./reference/manifest.md).

## Agree to the EULA

The instance will not start until you have agreed to the
[Minecraft EULA](https://aka.ms/MinecraftEULA). Once you have read it, set:

```kdl
server {
    eula #true
}
```

You can also pass `--eula` to `mc init` to do this at creation time.

## Run it

```console
$ mc run
```

On first run, mc downloads a Java runtime, the Minecraft binary, the mod
loader, and any configured mods, then starts the instance. Subsequent runs
reuse what is already installed and start immediately.

The console is attached to your terminal. Stop the instance with `Ctrl-C`:
mc asks it to save the world and waits for it to exit cleanly.

Note that the allow list is enabled by default, so players must be on it
before they can join. Add them with [`mc allow add`](./commands/allow.md).
To open the instance to everyone instead, set `allow-list #false` in the
`server` section.

## Next steps

- add or update mods with [`mc add`](./commands/add.md) and
  [`mc update`](./commands/update.md) — see [Managing Mods](./guides/mods.md)
- decide who may join and who holds commands — see
  [Players](./guides/players.md)
- configure world [Backups](./guides/backups.md)
- let players outside your network join — see [Tunnels](./guides/tunnel.md)
- deploy the instance as a service — see
  [Running under systemd](./guides/systemd.md)
