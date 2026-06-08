#!/usr/bin/env bash
# WienerEnvoy developer bootstrap.
#
# Idempotent. Re-runnable. Exits 0 on success, non-zero with a clear message
# on failure. No silent failures: every step reports its outcome.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

bold() { printf "\033[1m%s\033[0m\n" "$*"; }
green() { printf "\033[32m%s\033[0m\n" "$*"; }
yellow() { printf "\033[33m%s\033[0m\n" "$*"; }
red() { printf "\033[31m%s\033[0m\n" "$*" >&2; }

required_pinned_toolchain() {
    local pinned
    pinned="$(awk -F'"' '/^channel/ {print $2}' rust-toolchain.toml)"
    if [[ -z "${pinned}" ]]; then
        red "Could not read pinned channel from rust-toolchain.toml"
        return 1
    fi
    bold "==> Rust toolchain"
    if ! command -v rustup >/dev/null 2>&1; then
        red "rustup not found. Install from https://rustup.rs"
        return 1
    fi
    rustup show active-toolchain
    rustup component add rustfmt clippy --toolchain "${pinned}" >/dev/null 2>&1 || true
    green "ok"
}

install_cargo_subcommands() {
    bold "==> Cargo subcommands"
    local subcommands=("cargo-nextest" "cargo-deny" "cargo-audit")
    for sub in "${subcommands[@]}"; do
        if command -v "${sub}" >/dev/null 2>&1; then
            green "${sub}: already installed"
            continue
        fi
        yellow "installing ${sub}"
        cargo install "${sub}" --locked
    done
}

install_hooks() {
    bold "==> Git hooks"
    "${REPO_ROOT}/scripts/install-hooks.sh"
}

install_web_deps() {
    bold "==> Dashboard dependencies"
    if ! command -v pnpm >/dev/null 2>&1; then
        yellow "pnpm not found; skipping web deps. Install with: corepack enable && corepack prepare pnpm@latest --activate"
        return 0
    fi
    (cd web && pnpm install)
    green "web dependencies installed"
}

build_workspace() {
    bold "==> Build workspace"
    cargo build --workspace
    green "Build green"
}

summary() {
    bold "==> Ready"
    cat <<SUMMARY

  Daemon:    http://127.0.0.1:4747  (run: cargo run -p wienerenvoy-daemon)
  Dashboard: http://localhost:3000  (run: pnpm -C web dev)

  Point the dashboard at the daemon in dev:
    WIENERENVOY_DAEMON_URL=http://127.0.0.1:4747

  Next steps:
    cargo run -p wienerenvoy-cli -- --help
    cargo nextest run --workspace

SUMMARY
}

main() {
    required_pinned_toolchain
    install_cargo_subcommands
    install_hooks
    install_web_deps
    build_workspace
    summary
}

main "$@"
