# mc run

```text
mc run [OPTIONS]
```

Brings the instance in line with the manifest, then starts it. Anything
missing or out of date is installed first: the Java runtime, the Minecraft
binary, the mod loader, and the mods (added, updated, and removed to match
the manifest). The generated configuration files are rewritten from the
manifest on every start.

Only one running server is allowed per instance directory; a second
`mc run` refuses to start.

While the instance runs, mc supervises it: the console output is attached to
the terminal, and scheduled backups fire when enabled — see
[Backups](../guides/backups.md). The console is not interactive; use the
remote console (RCON) for live administration.

## Stopping

`Ctrl-C` (or SIGTERM, e.g. from a service manager, or SIGHUP, e.g. when the
terminal that started the instance closes) asks the instance to save the
world and shut down, and waits for it to exit before returning. If the
instance hangs past a grace period, or a second signal arrives, it is forced
down immediately.

mc does not restart a crashed instance; run it under a supervisor for that —
see [Running under systemd](../guides/systemd.md).

## Exit status

When the instance ends abnormally, `mc run` fails and propagates the
instance's exit code.

## Options

- `--manifest-path <PATH>` — path to `mc.toml`. Defaults to `./mc.toml`.
- `--lockfile-path <PATH>` — path to `mc.lock`. Defaults to `./mc.lock`.
