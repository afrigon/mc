# mc java

Manage Java runtimes.

## mc java list

```text
mc java list
```

Lists the runtimes mc can install, as `vendor@version` descriptors. The one
marked `(recommended)` is the default used when the manifest does not
configure `version` under `java`.

```console
$ mc java list
graal@25 (recommended)
graal@21
...
```

## mc java install

```text
mc java install [OPTIONS] <VERSION>
```

Downloads a runtime into the instance's `.java` directory. `VERSION` is a
descriptor from `mc java list`; the version half may be omitted to take the
vendor's recommended version. Fails if the runtime is already installed.

[`mc run`](./run.md) installs the configured runtime automatically; this
command exists to provision one ahead of time.

### Options

- `-p`, `--platform <PLATFORM>` — install for a specific operating system
  instead of the current one.
- `-a`, `--architecture <ARCHITECTURE>` — install for a specific CPU
  architecture instead of the current one.

### Examples

```console
$ mc java install graal@25
$ mc java install corretto@21
```
