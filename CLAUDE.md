# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`mc` is a Rust CLI (edition 2024) that manages and runs a Minecraft **server**. From a single `mc.toml` manifest it resolves and installs a JDK, the Minecraft server jar, a mod loader (Fabric), and mods (Modrinth), writes `server.properties` and the EULA file, launches the server process, and handles scheduled/manual world backups (local or S3) coordinated over RCON. The architecture is closely modeled on Cargo (manifest + lockfile, `Shell` output abstraction, verbosity levels, `CliError` exit codes).

## Commands

Tools and tasks are defined in `mise.toml`: `mise run build`, `mise run test`, `mise run docs`. `rustfmt.toml` uses `unstable_features`, so formatting needs nightly rustfmt; `mise run format` runs it on the pinned nightly. Releases are tag-driven (`v*`) via `.github/workflows/deploy.yml`, cross-compiling to Linux/Windows/macOS; `.github/workflows/ci.yml` runs the tests on pull requests.

## Documentation

The mdBook under `docs/` is the user-facing documentation. Every change to commands, the manifest format, environment variables, or user-visible runtime behavior must update the affected pages under `docs/src/` in the same change. The book documents behavior, not internals: keep prose generic (instance, Minecraft binary, mod loader, mod registry) and name concrete implementations (`fabric`, `modrinth`, `graal@25`) only where a reference page lists accepted values.

## Operational model

- **One server per directory.** Each instance is a directory with `mc.toml` at its root; the process CWD is always that root, and all cwd-relative paths in the code are correct by design (the scattered "fix this path / use a data path" TODOs are won't-fix under this model).
- **Deployment target is systemd** (supervised). `mc run` is expected to receive **SIGTERM** to stop and be restarted by the supervisor on crash — so graceful shutdown must drive the Minecraft `stop` command and *wait* for the JVM to flush/exit before returning (do not let `kill_on_drop` SIGKILL it mid-save). Crash auto-restart is delegated to systemd, not handled in-process.
- **Environment variables.** `MC_RCON_PASSWORD` supplies the RCON password (written into `server.properties` and used to authenticate backup flush). Backup work additionally reads the S3 bucket / storage target from the environment. Treat env as an override layer on top of `mc.toml`; S3 auth uses the standard AWS credential chain (env / `~/.aws` / IAM role) via `aws_config::defaults`.

## Runtime layout

Everything operates relative to `context.cwd`. These are gitignored and created at runtime:
- `mc.toml` — the manifest (user-authored)
- `.java/<descriptor>/` — installed JDKs
- `.minecraft/<loader-version>/server.jar` — installed server jars
- `instance/` — the live server working directory (world, `server.properties`, `eula.txt`)
- `temp/` — scratch space for downloads/archives

## Architecture

Request flow: `main.rs` builds `McContext`, parses `Cli` (clap), then dispatches the `CliCommand` enum to a command struct's `handle()`. Command handlers are thin — they translate CLI args into an `*Options` struct and call into `ops/`, which holds the real logic.

- **`cli/`** — clap definitions. `Cli` → `CliCommand` enum (in `cli/commands/mod.rs`) → one struct per subcommand in `cli/commands/`. Every command implements the `CommandHandler` trait (`async fn handle(&self, &mut McContext)`). `globals.rs` carries `--verbose`/`--quiet`/`--color`.
- **`context.rs`** — `McContext`: holds the `Shell` (behind a `Mutex`), `cwd`, and a shared `reqwest::Client` (pre-set User-Agent). Passed by `&mut` through nearly every function.
- **`ops/`** — the operations layer (business logic): `init`, `eula`, `java`, `minecraft`, `mods`, `run`, `backups/`. `ops/run.rs` is the orchestrator — read it first to understand the full pipeline (init dirs → check EULA → install Java → install Minecraft+loader → sync mods → write `server.properties` → spawn the Java subprocess alongside a `tokio-cron-scheduler` for backups).
- **`manifest/`** — `Manifest` deserializes `mc.toml` (serde). `lock.rs` is the mod lockfile. `presets.rs` generates manifests for the `vanilla`/`optimized`/`technical` presets using `toml_edit` (preserves formatting/comments).
- **`resolvers/`** — turn version aliases (e.g. `latest`, unspecified) into concrete validated versions by hitting APIs. All implement the `VersionResolver` trait.
- **`services/`** — stateless HTTP API clients, one file per upstream: `minecraft_api`, `fabric_api`, `modrinth_api`, `corretto_api`/`graal_api` (JDK distributions), `s3_api`, `discord_api`.
- **`network/`** — `stream_artifact` downloads with streaming + checksum verification, inflating tar.gz/zip on the fly. `artifact.rs` models an `ArtifactSource` and its checksum.
- **`crypto/`**, **`env/`** — hashing (sha1/sha2/md5) and `Platform`/`Architecture` detection.
- **`java/`, `minecraft/`, `mods/`** — domain models: JDK versions/vendors, server properties, EULA, seed, difficulty/gamemode, loader kinds, mod service kinds.
- **`utils/`** — `shell.rs` (Cargo-style status output), `errors.rs`, `archive.rs`, `product_descriptor.rs`, toml helpers, `verbosity.rs`.
- **`capabilities.rs`** — maps a Minecraft version to a `Capability` set (e.g. whether RCON exists), used to gate features like backup save-flush.

## Key conventions

- **Product descriptors.** The `product@version` string is the core identifier throughout (`graal@21`, `fabric`, mod versions). `RawProductDescriptor` is the parsed-but-unresolved form (version optional); `ProductDescriptor<P, V>` is the resolved, validated form produced by a `VersionResolver`. Manifest fields deserialize directly into `RawProductDescriptor`.
- **Errors.** `McResult<T>` = `anyhow::Result<T>` is used everywhere internally; add context with `.context(...)`. At the CLI boundary, errors become `CliError` (carries an exit code; default 101). Wrap a "this should never happen" error with `utils::errors::internal(...)` / `InternalError` — `exit_with_error` detects it and prints the bug-report footer.
- **Traits to extend behavior.** New API client → add to `services/`. New version source → implement `VersionResolver`. New backup destination → implement `BackupBackend` (`ops/backups/`, currently `local` + `s3`). New JDK distribution → implement `JavaProvider`.
- **Backups** coordinate with the running server over RCON (`save-off` → `save-all flush` → archive → `save-on`), then hand the archive to a `BackupBackend` (`local` or `s3`). Backend dispatch uses enum matching rather than `Box<dyn>` because the trait's `async fn` methods make it `dyn`-incompatible.
- **Output channels.** User-facing messages go through `Shell` (via `McContext::shell()` or a cloned `shell_handle()` in `'static` closures); `tracing` is reserved for `debug!`/`trace!` diagnostics.
- **Formatting** is enforced by `rustfmt.toml`: `imports_granularity = "Item"` (one `use` per item) and `group_imports = "StdExternalCrate"`. Match the existing one-import-per-line style.

## Panic safety

- Never run code that can panic: no `unwrap`, `expect`, indexing without bounds checks, or similar.
- Always handle errors safely and propagate them with `?`/`.context(...)`.
- Always unwrap `Option`/`Result` through safe combinators or pattern matching (e.g. recover a poisoned lock with `unwrap_or_else(PoisonError::into_inner)`).
