# Players

Who may join an instance, who is kept out, and who holds server commands
are all declared in the `players` section of `mc.kdl` and written to the
server when the instance starts:

```kdl
players {
    allow {
        Notch
        jeb_
    }

    ban {
        Griefer reason="stole the beacon" created="2026-09-06T14:00:00Z"
    }

    op {
        Notch level=4
    }
}
```

See [The Manifest Format](../reference/manifest.md) for every key. The
allow list is enabled by default; to open the instance to everyone, set
`allow-list #false` in the `server` section. Operators may join regardless
of the allow list.

## Editing the lists

[`mc allow`](../commands/allow.md), [`mc ban`](../commands/ban.md), and
[`mc op`](../commands/op.md) edit the section for you. Each has `add`,
`remove`, and `list` subcommands:

```console
$ mc allow add Notch jeb_
$ mc ban add Griefer --reason "stole the beacon" --for 7d
$ mc op add Notch
$ mc op list
```

Names are looked up when added, so a typo fails at the command rather
than at the next start, and the account's own casing is recorded. Names
are matched regardless of case afterwards.

## How the lists reach the server

The server keeps its lists in JSON files inside `instance/`. mc regenerates
them from the manifest on every start, so the manifest is the single
source of truth and those files are never edited by hand. Each player's
identity is resolved once and remembered in `mc.lock`, so a start never
waits on a lookup.

When the instance is running and its remote console is reachable, a
command also applies the change to it right away, so a newly banned
player is disconnected and a newly allowed one can join without a
restart. The remote console is reachable when an RCON password is
configured — see [Environment Variables](../reference/environment-variables.md).
Two kinds of change cannot be applied live and wait for the next restart:
an operator level or player limit bypass, and a ban expiry. The command
says so with a warning.

Changes made from inside the game, such as `/ban` or `/op` typed by an
operator, live only in the server's files and are replaced by the manifest
at the next start. Put them in the manifest with the matching `mc` command
to keep them.

## Offline mode

An instance with `online-mode #false` in the `properties` block of the
`server` section does not verify accounts, and the server identifies
players by name alone. mc derives the identity the server expects from the
name, so no lookup happens and any name is accepted.
