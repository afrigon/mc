# mc tunnel

Manage the tunnel agent that makes an instance reachable from outside the
local network. See the [Tunnels](../guides/tunnel.md) guide for the full
picture.

## mc tunnel list

```text
mc tunnel list [OPTIONS]
```

Lists the tunnel agent versions mc can install, most recent first. The one
marked `(latest)` is what `[tunnel] provider` resolves to when its version
half is omitted or set to `latest`.

```console
$ mc tunnel list
1.0.10 (latest)
1.0.9
...
```

### Options

- `--limit <LIMIT>` — the number of results. Defaults to `10`.

## mc tunnel install

```text
mc tunnel install [OPTIONS] [PROVIDER]
```

Downloads a tunnel agent into the instance's `.tunnel` directory. `PROVIDER`
accepts the same values as `[tunnel] provider` in the manifest, a `name` or
`name@version` descriptor, and defaults to `playit`. Fails if that version is
already installed.

[`mc run`](./run.md) installs the configured agent automatically; this
command exists to provision one ahead of time.

### Options

- `-p`, `--platform <PLATFORM>` — install for a specific operating system
  instead of the current one.
- `-a`, `--architecture <ARCHITECTURE>` — install for a specific CPU
  architecture instead of the current one.

### Examples

```console
$ mc tunnel install
$ mc tunnel install playit@1.0.9
```

## mc tunnel claim

```text
mc tunnel claim [OPTIONS]
```

Links the instance's tunnel agent to an account with the tunnel provider.
The command prints a link; opening it in a browser while signed in to the
provider approves the agent, after which the agent secret is saved to
`.tunnel/playit.toml`, readable only by the owning user. The link expires
after a few minutes.

[`mc run`](./run.md) performs the same claim on its own when the instance
starts from a terminal without a secret file, so this command is only needed
to claim ahead of time, for example before deploying under a service manager,
or to replace an existing secret.

### Options

- `-f`, `--force` — replace an existing secret with a fresh claim. The
  previous agent stays registered with the provider until it is removed
  there.

### Examples

```console
$ mc tunnel claim
    Claiming tunnel agent, approve it at https://playit.gg/claim/... with your playit.gg account
     Claimed tunnel agent secret saved to .tunnel/playit.toml
```
