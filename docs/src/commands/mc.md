# mc

```text
mc [OPTIONS] <COMMAND>
```

The `mc` binary is a collection of subcommands; run one of the commands
documented in this chapter. Every command operates on the instance in the
current working directory — the directory containing `mc.toml`.

## Global options

These options are accepted by every command.

- `-v`, `--verbose` — more detailed output. Repeat for more: `-v` prints
  informational messages, `-vv` debug, `-vvv` trace. The instance's console
  log level follows this setting.
- `-q`, `--quiet` — do not print mc log messages.
- `--color <WHEN>` — control colored output: `auto` (the default), `always`,
  or `never`.
- `-h`, `--help` — print help.
- `-V`, `--version` — print the mc version.

## Exit status

`mc` exits with `0` on success and a non-zero code on failure.
[`mc run`](./run.md) propagates the instance's own exit code when the
instance ends abnormally.
