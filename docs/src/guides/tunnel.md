# Tunnels

An instance normally accepts players only from its own network; letting
friends in from elsewhere means forwarding a port on the router and sharing
a public address. A tunnel removes both steps: a small agent runs beside the
instance, keeps an outbound connection to a relay, and the relay hands out a
public hostname that reaches the instance through that connection. Nothing
on the network needs configuring, and the hostname is what you share.

mc supports [playit.gg](https://playit.gg) as the tunnel provider. It is free
for a friend group and needs an account, which is created the first time an
agent is claimed.

## Enabling a tunnel

```kdl
tunnel {
    provider "playit"
}
```

Adding the section is enough; see
[`tunnel`](../reference/manifest.md#tunnel) for its keys.

## First start

Start the instance from a terminal:

```console
$ mc run
    Claiming tunnel agent, approve it at https://playit.gg/claim/... with your playit.gg account
     Claimed tunnel agent secret saved to .tunnel/playit.toml
    Creating a Minecraft tunnel for port 25565
      Tunnel players can join at quiet-fox.joinmc.link
```

Opening the link in a browser, signed in to the provider, approves the agent
for that account. mc then stores the agent secret in `.tunnel/playit.toml`,
readable only by the owning user, creates a Minecraft tunnel for the
instance's server port when the agent has none, and prints the public
address. Players add that address to their server list as they would any
other; no port is needed.

Later starts skip the claim and reuse the tunnel. The address is stable for
as long as the tunnel exists with the provider. When notifications are
enabled, the "started" message includes it.

Deleting `.tunnel/playit.toml`, or running
[`mc tunnel claim --force`](../commands/tunnel.md), links the instance to a
new agent on the next start.

## While the instance runs

The agent runs for as long as the instance does and is restarted by mc if it
stops on its own. Its output is written to `.tunnel/playitd.log`, or shown in
the terminal when the instance is started with `mc run --tunnel-logs`.
Stopping the instance stops the agent.

Traffic between players and the instance flows through the provider's relay,
so the address only works while the instance is running.

## Under a service manager

The claim needs a browser, and a service has no terminal to print the link
to. When no secret file exists and the instance is not started from a
terminal, `mc run` fails with instructions instead of waiting. Claim the
agent once beforehand, either by running `mc run` from a terminal or with
[`mc tunnel claim`](../commands/tunnel.md), then deploy as described in
[Running under systemd](./systemd.md). The secret file travels with the
instance directory.

## Managing the tunnel

The tunnel itself, its address, and the agent are visible in the provider's
dashboard, where a custom hostname or region can be assigned. When mc cannot
create the tunnel, for example because the account has reached the
provider's limits, it prints a warning with a link to the dashboard and keeps
running; a tunnel created there for the instance's server port is picked up
on the next start.
