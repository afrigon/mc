# Installation

## mise

[mise](https://mise.jdx.dev) installs the latest release binary and puts it
on your `PATH`:

```console
$ mise use -g github:afrigon/mc
```

## Pre-built binaries

Every release publishes pre-built binaries on the
[GitHub releases page](https://github.com/afrigon/mc/releases) for:

- Linux (`x86_64`, `aarch64`)
- macOS (`x86_64`, `aarch64`)
- Windows (`x86_64`)

Download the archive for your platform, extract it, and place the `mc` binary
somewhere on your `PATH`:

```console
$ tar -xzf mc-linux-x86_64.tar.gz
$ install -m 755 mc /usr/local/bin/mc
$ mc --version
```

## Building from source

mc is written in Rust and builds with a recent stable toolchain:

```console
$ git clone https://github.com/afrigon/mc
$ cd mc
$ cargo build --release
```

The binary is produced at `target/release/mc`.
