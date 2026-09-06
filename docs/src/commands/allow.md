# mc allow

Manage the allow list: the players permitted to join when the allow list is
enabled, which it is by default. See the [Players](../guides/players.md)
guide for how the lists reach the server.

## mc allow add

```text
mc allow add [OPTIONS] <NAME>...
```

Adds players to the `allow` group of the manifest's `players` section. Each
`NAME` is looked up to make sure the account exists and is recorded with
the account's own casing. A player who is already allowed is skipped with a
warning; a player who is banned is refused until the ban is lifted.

When the instance is running and its remote console is reachable, the
change is applied to it immediately. Otherwise it takes effect the next
time the instance starts.

The list is still edited when `allow-list` in the `server` section is
`#false`, but a warning points out that anyone can join until it is
turned back on.

```console
$ mc allow add Notch jeb_
```

### Options

- `--manifest-path <PATH>` — path to `mc.kdl`. Defaults to `./mc.kdl`.
- `--lockfile-path <PATH>` — path to `mc.lock`. Defaults to `./mc.lock`.

## mc allow remove

```text
mc allow remove [OPTIONS] <NAME>...
```

Removes players from the allow list. Fails if a player is not on it. On a
running instance, a removed player who is online is disconnected.

```console
$ mc allow remove Notch
```

### Options

- `--manifest-path <PATH>` — path to `mc.kdl`. Defaults to `./mc.kdl`.
- `--lockfile-path <PATH>` — path to `mc.lock`. Defaults to `./mc.lock`.

## mc allow list

```text
mc allow list [OPTIONS]
```

Prints the allowed players, one per line.

### Options

- `--manifest-path <PATH>` — path to `mc.kdl`. Defaults to `./mc.kdl`.
- `--lockfile-path <PATH>` — path to `mc.lock`. Defaults to `./mc.lock`.
