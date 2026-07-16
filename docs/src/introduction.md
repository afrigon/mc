# Introduction

**mc** is a command line tool to manage and run Minecraft instances.

An instance is described by a single manifest file, `mc.toml`. From that one
file, `mc run` takes care of everything needed to bring the instance up:

- installs a Java runtime
- installs Minecraft for the configured version
- installs a mod loader, when one is configured
- downloads mods and resolves their dependencies into a lockfile
- writes the configuration files Minecraft expects
- launches the instance and supervises it until it stops
- takes scheduled world backups to local storage or S3

If you have used a package manager, the workflow will feel familiar: a
declarative manifest, a generated lockfile, and a small set of commands that
operate on them.

```console
$ mc init myserver --eula
$ cd myserver
$ mc run
```

## Sections

**[Getting Started](./getting-started.md)**

Install mc and start your first instance.

**[Guides](./guides.md)**

Task-oriented guides: managing mods, configuring backups, and deploying an
instance as a systemd service.

**[Reference](./reference.md)**

The `mc.toml` manifest format, the environment variables mc reads, and the
on-disk layout of an instance.

**[Commands](./commands.md)**

Detailed documentation for every `mc` command.
