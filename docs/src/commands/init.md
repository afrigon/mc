# mc init

```text
mc init [OPTIONS] [PATH]
```

Creates a new instance: a `mc.toml` manifest and the supporting directories.
`PATH` defaults to the current directory and is created if it does not
exist. The command refuses to run where a `mc.toml` already exists.

## Options

- `--name <NAME>` — the instance name. Defaults to the directory name.
- `--eula` — record your agreement to the
  [Minecraft EULA](https://aka.ms/MinecraftEULA) in the generated manifest.
  Without it, the manifest is created with `eula = false` and the instance
  will not start until you edit it.
- `--preset <PRESET>` — the shape of the generated manifest:
    - `vanilla` — no mod loader and no mods.
    - `optimized` — a mod loader plus performance mods. This is the default.
    - `technical` — a mod loader plus performance mods and tools for
      technical play.

Presets pin the latest Minecraft release and compatible mod versions at the
time the command runs; the generated manifest is a starting point to edit,
not a fixed template.

## Examples

```console
$ mc init myserver
$ mc init --preset technical --eula
$ mc init myserver --name "smp" --preset vanilla
```
