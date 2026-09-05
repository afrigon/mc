# Running under systemd

`mc run` is designed to live under a process supervisor. It does not restart
the instance when it crashes; it reports the failure through its exit code
and lets the supervisor decide. On Linux, systemd is the natural fit.

## Unit file

Assuming an instance at `/srv/minecraft/myserver`, create
`/etc/systemd/system/myserver.service`:

```ini
[Unit]
Description=myserver Minecraft instance
After=network-online.target
Wants=network-online.target

[Service]
User=minecraft
WorkingDirectory=/srv/minecraft/myserver
ExecStart=/usr/local/bin/mc run --server-logs
EnvironmentFile=/etc/minecraft/myserver.env
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Then enable and start it:

```console
$ systemctl enable --now myserver
```

`WorkingDirectory` must be the instance root — the directory containing
`mc.kdl` — since mc operates relative to it.

## Secrets

Environment variables such as `MC_RCON_PASSWORD` or a notification webhook
are best kept out of the unit file, in an `EnvironmentFile` readable only by
root:

```console
$ cat /etc/minecraft/myserver.env
MC_RCON_PASSWORD=...
MC_DISCORD_WEBHOOK=...
```

See [Environment Variables](../reference/environment-variables.md) for
everything mc reads from the environment.

## Stopping and restarting

`systemctl stop` sends the service SIGTERM. mc catches it, asks the instance
to save the world and shut down, and waits for it to exit before returning —
within systemd's default stop timeout, so the unit is not killed mid-save.
The same applies to `systemctl restart` and to stops issued during a system
shutdown.

`Restart=on-failure` brings the instance back up if it crashes, while a
clean stop (including one requested from inside the game with `/stop`) stays
stopped.

## Logs

With `--server-logs`, the instance's console output goes to standard output,
which systemd captures in the journal (add `--tunnel-logs` to capture the
tunnel agent as well):

```console
$ journalctl -u myserver -f
```

mc's own log verbosity is controlled by the global `--verbose` flag; the
instance's console log level follows it.
