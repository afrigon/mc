# mc

**mc** is a command line tool to manage and run Minecraft instances.

Describe your server in a single `mc.toml` manifest; mc takes care of the
rest:

- installs a Java runtime, the Minecraft binary, a mod loader, and mods
- manages mods with a lockfile (`mc add`, `mc remove`, `mc update`)
- runs the server with clean shutdown, ready to deploy under systemd
- takes scheduled world backups, stored locally or on S3

## Quick start

Grab a pre-built binary from the
[releases page](https://github.com/afrigon/mc/releases), then:

```console
$ mc init myserver --eula
$ cd myserver
$ mc run
```

Passing `--eula` indicates you have read and agree to the
[Minecraft EULA](https://aka.ms/MinecraftEULA).

## Documentation

Full documentation is available at
[afrigon.github.io/mc](https://afrigon.github.io/mc/): guides for mods,
backups, and systemd, plus a complete reference for every command and
manifest key.
