# mc run

```text
mc run [OPTIONS]
```

Brings the instance in line with the manifest, then starts it. Anything
missing or out of date is installed first: the Java runtime, the Minecraft
binary, the mod loader, the mods (added, updated, and removed to match the
manifest), and the tunnel agent when a tunnel is configured. The generated
configuration files are rewritten from the manifest on every start.

Only one running server is allowed per instance directory; a second
`mc run` refuses to start.

While the instance runs, mc supervises it, and scheduled backups fire when
enabled — see [Backups](../guides/backups.md). The instance's console output
is hidden unless `--server-logs` is passed; it is always written to
`instance/logs/` by the server itself. The console is not interactive; use
the remote console (RCON) for live administration.

With a `[tunnel]` section, the tunnel agent starts beside the instance and
is restarted if it stops on its own; the public address is printed at
startup, even with `--quiet`. The agent's output is hidden unless `--tunnel-logs` is passed, and
goes to `.tunnel/playitd.log` otherwise. The first start from a terminal prints a claim link to approve in a
browser and saves the resulting secret under `.tunnel/`. Without a terminal
and without a secret, `mc run` fails with instructions rather than waiting —
see [Tunnels](../guides/tunnel.md).

## Stopping

`Ctrl-C` (or SIGTERM, e.g. from a service manager, or SIGHUP, e.g. when the
terminal that started the instance closes) asks the instance to save the
world and shut down, and waits for it to exit before returning. If the
instance hangs past a grace period, or a second signal arrives, it is forced
down immediately.

A scheduled backup caught in flight is cancelled: it discards its partial
archive and never damages stored backups — see
[Backups](../guides/backups.md).

mc does not restart a crashed instance; run it under a supervisor for that —
see [Running under systemd](../guides/systemd.md).

## Exit status

When the instance ends abnormally, `mc run` fails and propagates the
instance's exit code.

## Options

- `--manifest-path <PATH>` — path to `mc.toml`. Defaults to `./mc.toml`.
- `--lockfile-path <PATH>` — path to `mc.lock`. Defaults to `./mc.lock`.
- `--server-logs` — show the instance's console output in the terminal.
- `--tunnel-logs` — show the tunnel agent's output in the terminal.
