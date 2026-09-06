# mc ban

Manage the ban list: players and addresses refused by the server. See the
[Players](../guides/players.md) guide for how the lists reach the server.

## mc ban add

```text
mc ban add [OPTIONS] <NAME>...
mc ban add [OPTIONS] --ip <ADDRESS>...
```

Bans players, or with `--ip`, addresses. Player names are looked up to make
sure the account exists and are recorded with the account's own casing. A
player who is already banned is an error. A player on the allow list is
removed from it, with a warning.

When the instance is running and its remote console is reachable, the ban
is applied to it immediately and an online player is disconnected. A ban
expiry cannot be applied live: the player stays banned until the next
restart, when the expiry takes effect. Anything not applied live is
reported with a warning.

Every player and address named in one command shares the same reason and
expiry.

```console
$ mc ban add Griefer --reason "stole the beacon"
$ mc ban add Griefer1 Griefer2 --for 7d
$ mc ban add --ip 203.0.113.7 --ip 203.0.113.8 --until 2026-12-31T00:00:00Z
```

### Options

- `--ip <ADDRESS>` — ban addresses instead of players. May be repeated.
- `--reason <TEXT>` — the reason shown to the banned player. Defaults to
  the server's own wording.
- `--until <DATE>` — lift the ban at an RFC 3339 date.
- `--for <DURATION>` — lift the ban after a duration such as `7d`, `12h`,
  or `30m`. Cannot be combined with `--until`.
- `--manifest-path <PATH>` — path to `mc.kdl`. Defaults to `./mc.kdl`.
- `--lockfile-path <PATH>` — path to `mc.lock`. Defaults to `./mc.lock`.

## mc ban remove

```text
mc ban remove [OPTIONS] <NAME>...
mc ban remove [OPTIONS] --ip <ADDRESS>...
```

Lifts bans on players, or with `--ip`, addresses. A player or address that
is not banned is skipped with a warning.

```console
$ mc ban remove Griefer1 Griefer2
$ mc ban remove --ip 203.0.113.7
```

### Options

- `--ip <ADDRESS>` — unban addresses instead of players. May be repeated.
- `--manifest-path <PATH>` — path to `mc.kdl`. Defaults to `./mc.kdl`.
- `--lockfile-path <PATH>` — path to `mc.lock`. Defaults to `./mc.lock`.

## mc ban list

```text
mc ban list [OPTIONS]
```

Prints the banned players, then the banned addresses, one per line with
the reason, when the ban was issued, and when it expires.

### Options

- `--manifest-path <PATH>` — path to `mc.kdl`. Defaults to `./mc.kdl`.
- `--lockfile-path <PATH>` — path to `mc.lock`. Defaults to `./mc.lock`.
