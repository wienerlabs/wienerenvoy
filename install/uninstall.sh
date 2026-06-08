#!/usr/bin/env bash
# WienerEnvoy uninstaller. Run with sudo.
#
#   sudo ./install/uninstall.sh

set -euo pipefail

LABEL="io.wienerlabs.wienerenvoy"
PREFIX="/usr/local/bin"
SUPPORT="/Library/Application Support/WienerEnvoy"
PLIST="/Library/LaunchDaemons/${LABEL}.plist"

bold() { printf "\033[1m%s\033[0m\n" "$*"; }
green() { printf "\033[32m%s\033[0m\n" "$*"; }
die() {
    printf "\033[31merror: %s\033[0m\n" "$*" >&2
    exit 1
}

[[ "$(uname)" == "Darwin" ]] || die "WienerEnvoy is macOS only"
[[ "${EUID}" -eq 0 ]] || die "run with sudo: sudo ./install/uninstall.sh"

bold "==> Stopping and removing the LaunchDaemon"
launchctl bootout "system/${LABEL}" 2>/dev/null || true
rm -f "${PLIST}"

bold "==> Removing binaries"
rm -f "${PREFIX}/wienerenvoy-daemon" "${PREFIX}/wenvoy"

# The config and token hold state; ask before deleting.
if [[ -d "${SUPPORT}" ]]; then
    read -r -p "Remove config and token at ${SUPPORT}? [y/N] " answer
    if [[ "${answer}" == "y" || "${answer}" == "Y" ]]; then
        rm -rf "${SUPPORT}"
        green "removed ${SUPPORT}"
    else
        green "kept ${SUPPORT}"
    fi
fi

green "WienerEnvoy uninstalled. Wake-on-LAN was left enabled; disable with: sudo pmset -a womp 0"
