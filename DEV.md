# Developer Guide

This document describes the workflow for working on the `devtool` workspace: layout, how the crates fit together, how to build and test, how to run the lints and formatter, and what the CI-style verification step looks like.

User-facing documentation lives in [`README.md`](./README.md).

## Prerequisites

- A recent stable Rust toolchain (the workspace uses edition 2024 -- Rust 1.85 or newer).
- A Bash-compatible shell for `./dev.sh` (macOS / Linux). On Windows, run the underlying `cargo` commands directly (see the table below).

## Workspace layout

```
Cargo.toml            workspace manifest
dev.sh                developer entry point (see below)
README.md             user-facing documentation
DEV.md                this file
LICENSE               MIT licence
devtool/              main CLI binary
devtool-git/          git operations library
devtool-version/      version parsing library
```

The workspace contains three crates:

- **devtool** -- the CLI binary. Provides subcommands for version bumping, config generation, gitignore generation, and git description display.
- **devtool-git** -- a thin wrapper around the `git` CLI. Used by `devtool` to run git operations (status, describe, commit, tag, push, etc.).
- **devtool-version** -- parses and manipulates version strings in single (`1`), pair (`1.2`), and triple (`1.2.3`) formats, with optional `v` prefix.

Each crate has its own `Cargo.toml` with an independent dependency list. All crates use edition 2024.

## `./dev.sh`

A thin wrapper over the usual `cargo` invocations. Run `./dev.sh help` for the current command list.

| Command                  | What it does                                              |
|--------------------------|-----------------------------------------------------------|
| `./dev.sh build`         | `cargo build --workspace --all-targets`                   |
| `./dev.sh test`          | `cargo test --workspace --all-targets`                    |
| `./dev.sh lint`          | `cargo clippy --workspace --all-targets -- -D warnings`   |
| `./dev.sh fmt`           | `cargo fmt --all` (rewrites files in place)               |
| `./dev.sh fmt-check`     | `cargo fmt --all -- --check`                              |
| `./dev.sh check`         | `fmt-check` + `lint` + `test` -- the CI-equivalent gate  |
| `./dev.sh run <args>`    | `cargo run -p devtool --quiet -- <args>`                  |
| `./dev.sh clean`         | `cargo clean`                                             |

`dev.sh` forwards extra arguments to `cargo` where sensible, e.g. `./dev.sh test my_test_name -- --nocapture`. Set `CARGO=/path/to/cargo` to override the cargo binary.

## Day-to-day workflow

1. Edit code.
2. `./dev.sh fmt` -- format in place.
3. `./dev.sh check` -- the gate. If this is clean, your change is ready for review.

Tighter iteration:

- `./dev.sh test some_test_name` to run a specific test.
- `cargo test -p devtool-version` to scope to a single crate.
- `./dev.sh run show-description` to exercise the CLI directly.

## Lints

Each crate enables clippy's `pedantic` group at `deny` level in its `Cargo.toml`:

```toml
[lints.clippy]
missing_errors_doc = "allow"
missing_panics_doc = "allow"
pedantic = { level = "deny", priority = -1 }
```

The main `devtool` binary additionally enables `clippy::all` and `clippy::nursery` via `#![warn(...)]` in `main.rs`.

To add a local override, use `#[allow(clippy::xxx)]` on the offending item, or add the lint to the crate's `[lints.clippy]` table for crate-scoped overrides.

`./dev.sh lint` uses `-D warnings` so new warnings fail rather than silently accumulating.

## Testing

All tests live inline (`#[cfg(test)] mod tests`) in the crate they belong to. The project uses [`rstest`](https://crates.io/crates/rstest) for parameterized tests and [`tempfile`](https://crates.io/crates/tempfile) for filesystem tests that need temporary directories.

Tests run with `./dev.sh test` (everything in the workspace) or `cargo test -p <crate>` when you want to scope. Tests are hermetic -- filesystem tests use `tempfile::TempDir` -- so they can run in parallel.

The git wrapper tests in `devtool-git` create real temporary git repositories via `git init` to exercise the actual git CLI integration.

## CI and release

Two GitHub Actions workflows live under `.github/workflows/`:

| Workflow | Trigger | What it does |
|----------|---------|--------------|
| [`ci.yaml`](./.github/workflows/ci.yaml) | push/PR to `main`, nightly cron | build + test matrix on macOS (x86_64), Windows (x86_64), Linux (x86_64) |
| [`release.yaml`](./.github/workflows/release.yaml) | tag push matching `v*.*.*` | builds release binaries for aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc, x86_64-unknown-linux-musl; creates GitHub Release |

To reproduce the gate locally:

```
./dev.sh check
```

This runs `fmt-check` + `clippy` + `test` on the host platform. The full multi-OS matrix only runs in CI.

## Cutting a release

`devtool` can bump its own version:

```
./dev.sh run bump-version
```

This auto-increments the version based on the latest git tag, updates all `Cargo.toml` files in the workspace, regenerates `Cargo.lock`, commits, and creates an annotated tag. Pass `--no-push-all` to skip the push step if you want to review before pushing.

To specify an explicit version:

```
./dev.sh run bump-version 1.2.3
```

Pushing the tag triggers `release.yaml`, which builds and publishes release binaries to GitHub Releases.
