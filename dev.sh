#!/usr/bin/env bash
# Developer wrapper for common workspace tasks. Run `./dev.sh help` to see the
# list of subcommands. See DEV.md for the workflow details.

set -euo pipefail

usage() {
    cat <<'EOF'
Usage: ./dev.sh <command> [args...]

Commands:
  build                  cargo build --workspace --all-targets
  test [args...]         cargo test --workspace --all-targets [args...]
  lint                   cargo clippy --workspace --all-targets -- -D warnings
  fmt                    cargo fmt --all
  fmt-check              cargo fmt --all -- --check
  check                  fmt-check + lint + test (what CI should run)
  run [args...]          cargo run -p devtool --quiet -- [args...]
  clean                  cargo clean
  help                   show this help

Environment:
  CARGO     override the cargo binary (default: cargo)
EOF
}

CARGO="${CARGO:-cargo}"

cmd="${1:-help}"
shift || true

case "$cmd" in
    build)
        "$CARGO" build --workspace --all-targets "$@"
        ;;
    test)
        "$CARGO" test --workspace --all-targets "$@"
        ;;
    lint)
        "$CARGO" clippy --workspace --all-targets -- -D warnings
        ;;
    fmt)
        "$CARGO" fmt --all
        ;;
    fmt-check)
        "$CARGO" fmt --all -- --check
        ;;
    check)
        "$CARGO" fmt --all -- --check
        "$CARGO" clippy --workspace --all-targets -- -D warnings
        "$CARGO" test --workspace --all-targets
        ;;
    run)
        "$CARGO" run -p devtool --quiet -- "$@"
        ;;
    clean)
        "$CARGO" clean
        ;;
    help|-h|--help)
        usage
        ;;
    *)
        echo "error: unknown command '$cmd'" >&2
        usage
        exit 2
        ;;
esac
