# Richard's Development Tool

[![CI](https://github.com/rcook/devtool/actions/workflows/ci.yaml/badge.svg)][ci-workflow]
[![Release](https://github.com/rcook/devtool/actions/workflows/release.yaml/badge.svg)][release-workflow]

A CLI tool for managing Rust and Python project releases. It automates version bumping across `Cargo.toml` and `pyproject.toml` files, generates `.devtool.yaml` configuration, produces `.gitignore` entries from git status, and displays git description/version info.

## Installation

### From GitHub Releases

Download the latest binary for your platform from [Releases](https://github.com/rcook/devtool/releases).

Available targets:

- `aarch64-apple-darwin` (macOS ARM)
- `x86_64-apple-darwin` (macOS Intel)
- `x86_64-pc-windows-msvc` (Windows)
- `x86_64-unknown-linux-musl` (Linux)

### From source

```
git clone https://github.com/rcook/devtool.git
cd devtool
cargo build --release
```

The binary will be at `target/release/devtool`.

## Usage

```
devtool <command> [options]
```

### Commands

| Command | Description |
|---------|-------------|
| `bump-version [VERSION]` | Update `Cargo.toml`/`pyproject.toml` version, generate a new git tag and push. Auto-increments from the latest tag if no version is specified. |
| `gen-config` | Generate a `.devtool.yaml` configuration file |
| `gen-ignore` | Generate `.gitignore` entries from untracked and ignored files |
| `show-description` | Show git description and parsed version information |
| `scratch` | Experimental |

### Global options

| Option | Description |
|--------|-------------|
| `-d, --dir <PATH>` | Path to the git repository (auto-detected if omitted) |
| `-l, --level <LEVEL>` | Logging level filter (default: `info`) |
| `--detailed` | Enable detailed JSON logging |

### Examples

```
# Auto-increment version, commit, tag, and push
devtool bump-version

# Bump to a specific version without pushing
devtool bump-version 1.2.3 --no-push-all

# Generate a configuration file
devtool gen-config

# Show current git description
devtool show-description
```

## Contributing

See [`DEV.md`](./DEV.md) for the developer guide: workspace layout, build/test/lint workflow, CI details, and how to cut a release.

## Licence

[MIT](./LICENSE)

[ci-workflow]: https://github.com/rcook/devtool/actions/workflows/ci.yaml
[release-workflow]: https://github.com/rcook/devtool/actions/workflows/release.yaml
