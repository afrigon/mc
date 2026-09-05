# mc

**mc** is a command line tool to manage and run Minecraft instances.

Describe your server in a single `mc.toml` manifest; mc takes care of the
rest:

- installs a Java runtime, the Minecraft binary, a mod loader, and mods
- manages mods with a lockfile (`mc add`, `mc remove`, `mc update`)
- runs the server with clean shutdown
- takes scheduled world backups, stored locally or on S3

## Quick start

Install with [mise](https://mise.jdx.dev):

```sh
mise use -g github:afrigon/mc
```

or grab a pre-built binary from the
[releases page](https://github.com/afrigon/mc/releases). Then:

```sh
mc init myserver --eula
cd myserver
mc run
```

Passing `--eula` indicates you have read and agree to the
[Minecraft EULA](https://aka.ms/MinecraftEULA).

## Development

Tools are pinned with [mise](https://mise.jdx.dev); `mise install` sets up
the toolchain, then:

```sh
mise run build     # release build
mise run test      # test suite
mise run format    # rustfmt, on the nightly the config requires
mise run docs      # build the documentation book
```

## Documentation

Full documentation is available at
[afrigon.github.io/mc](https://afrigon.github.io/mc/): guides for mods,
backups, and systemd, plus a complete reference for every command and
manifest key.
