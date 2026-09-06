# mc op

Manage the operators: players granted server commands. Operators may join
regardless of the allow list. See the [Players](../guides/players.md) guide
for how the lists reach the server.

## mc op add

```text
mc op add [OPTIONS] <NAME>...
```

Makes players operators, or changes the settings of players who already
are, with a warning. Player names are looked up to make sure the account
exists and are recorded with the account's own casing.

When the instance is running and its remote console is reachable, a new
operator with the default settings is applied immediately. A custom level
or the player limit bypass cannot be applied live and takes effect at the
next restart, which is reported with a warning.

```console
$ mc op add Notch
$ mc op add jeb_ --level 2 --bypass-player-limit
```

### Options

- `--level <LEVEL>` — the permission level, from `1` to `4`. Defaults to
  the server's `op-permission-level` property, which is `4` unless set in
  the `properties` block of the `server` section.
- `--bypass-player-limit` — let the operator join when the server is full.
- `--manifest-path <PATH>` — path to `mc.kdl`. Defaults to `./mc.kdl`.
- `--lockfile-path <PATH>` — path to `mc.lock`. Defaults to `./mc.lock`.

## mc op remove

```text
mc op remove [OPTIONS] <NAME>...
```

Removes operators. A player who is not an operator is skipped with a
warning.

```console
$ mc op remove Notch
```

### Options

- `--manifest-path <PATH>` and `--lockfile-path <PATH>` — as above.

## mc op list

```text
mc op list [OPTIONS]
```

Prints the operators, one per line with their effective permission level.

### Options

- `--manifest-path <PATH>` and `--lockfile-path <PATH>` — as above.
