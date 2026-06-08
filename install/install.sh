#!/usr/bin/env bash
# WienerEnvoy installer.
#
# Installs the daemon as a root LaunchDaemon so it can manage power without sudo.
# Build the binaries first, then run this with sudo:
#
#   cargo build --release -p wienerenvoy-daemon -p wienerenvoy-cli
#   sudo ./install/install.sh
#
# Or run it from an unpacked release tarball (binaries and dashboard alongside).

set -euo pipefail

LABEL="io.wienerlabs.wienerenvoy"
PREFIX="/usr/local/bin"
SUPPORT="/Library/Application Support/WienerEnvoy"
LOGS="/Library/Logs/WienerEnvoy"
PLIST="/Library/LaunchDaemons/${LABEL}.plist"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

bold() { printf "\033[1m%s\033[0m\n" "$*"; }
green() { printf "\033[32m%s\033[0m\n" "$*"; }
yellow() { printf "\033[33m%s\033[0m\n" "$*"; }
die() {
    printf "\033[31merror: %s\033[0m\n" "$*" >&2
    exit 1
}

[[ "$(uname)" == "Darwin" ]] || die "WienerEnvoy is macOS only"
[[ "${EUID}" -eq 0 ]] || die "run with sudo: sudo ./install/install.sh"

command -v tailscale >/dev/null 2>&1 ||
    yellow "tailscale not found on PATH; the daemon will bind loopback only until a tailnet address exists"

# Locate binaries: prefer a workspace build, fall back to a release tarball.
DAEMON_BIN="${REPO_ROOT}/target/release/wienerenvoy-daemon"
CLI_BIN="${REPO_ROOT}/target/release/wenvoy"
[[ -x "${DAEMON_BIN}" ]] || DAEMON_BIN="${SCRIPT_DIR}/wienerenvoy-daemon"
[[ -x "${CLI_BIN}" ]] || CLI_BIN="${SCRIPT_DIR}/wenvoy"
[[ -x "${DAEMON_BIN}" ]] ||
    die "daemon binary not found. Build it first: cargo build --release -p wienerenvoy-daemon -p wienerenvoy-cli"
[[ -x "${CLI_BIN}" ]] ||
    die "wenvoy binary not found. Build it first: cargo build --release -p wienerenvoy-cli"

bold "==> Installing binaries to ${PREFIX}"
install -m 0755 "${DAEMON_BIN}" "${PREFIX}/wienerenvoy-daemon"
install -m 0755 "${CLI_BIN}" "${PREFIX}/wenvoy"

bold "==> Creating ${SUPPORT} and ${LOGS}"
mkdir -p "${SUPPORT}" "${LOGS}"

if [[ ! -f "${SUPPORT}/config.toml" ]]; then
    bold "==> Writing default config"
    cat >"${SUPPORT}/config.toml" <<'CONFIG'
[server]
http_port = 4747

[power]
default_machine_action = "sleep"
shutdown_requires_confirm = true
CONFIG
    chmod 0644 "${SUPPORT}/config.toml"
fi

# Dashboard: copy the static export if we have one.
DASHBOARD_SRC=""
[[ -d "${REPO_ROOT}/web/out" ]] && DASHBOARD_SRC="${REPO_ROOT}/web/out"
[[ -z "${DASHBOARD_SRC}" && -d "${SCRIPT_DIR}/dashboard" ]] && DASHBOARD_SRC="${SCRIPT_DIR}/dashboard"
if [[ -n "${DASHBOARD_SRC}" ]]; then
    bold "==> Installing dashboard"
    rm -rf "${SUPPORT}/dashboard"
    cp -R "${DASHBOARD_SRC}" "${SUPPORT}/dashboard"
else
    yellow "no dashboard build found; the daemon will serve a placeholder. Build it with: pnpm -C web build"
fi

bold "==> Enabling Wake-on-LAN (so a shutdown machine can be woken)"
pmset -a womp 1 2>/dev/null || yellow "could not enable Wake-on-LAN"

bold "==> Installing LaunchDaemon"
install -m 0644 "${REPO_ROOT}/infra/macos/${LABEL}.plist" "${PLIST}"
chown root:wheel "${PLIST}"

# (Re)load the service.
launchctl bootout "system/${LABEL}" 2>/dev/null || true
launchctl bootstrap system "${PLIST}"
launchctl enable "system/${LABEL}"

green ""
green "WienerEnvoy installed and running on http://127.0.0.1:4747 (and your tailnet)."
green "Get the dashboard token:  sudo wenvoy token show"
green "Check status:             wenvoy status"
