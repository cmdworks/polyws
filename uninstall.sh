#!/usr/bin/env bash
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# polyws uninstaller
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/cmdworks/polyws/main/uninstall.sh | bash
#
# Options (env vars):
#   polyws_NAME    — binary name to remove (default: polyws)
#                 e.g.  polyws_NAME=poly   removes `poly`  from PATH
#   polyws_INSTALL — directory to remove from (default: auto-detect from PATH)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
set -euo pipefail

GREEN='\033[0;32m'
AMBER='\033[0;33m'
RED='\033[0;31m'
DIM='\033[2m'
RESET='\033[0m'

info() { printf "${GREEN}▸${RESET} %s\n" "$*"; }
warn() { printf "${AMBER}▸${RESET} %s\n" "$*"; }
fail() { printf "${RED}✘${RESET} %s\n" "$*" >&2; exit 1; }

BINARY_NAME="${polyws_NAME:-polyws}"

echo ""
printf "${RED}  ┌─────────────────────────────────────┐${RESET}\n"
printf "${RED}  │       polyws  —  uninstaller        │${RESET}\n"
printf "${RED}  └─────────────────────────────────────┘${RESET}\n"
echo ""

# ── Resolve the binary path ───────────────────────────────
if [ -n "${polyws_INSTALL:-}" ]; then
    BIN_PATH="${polyws_INSTALL}/${BINARY_NAME}"
else
    BIN_PATH="$(command -v "$BINARY_NAME" 2>/dev/null || true)"
fi

if [ -z "$BIN_PATH" ] || [ ! -f "$BIN_PATH" ]; then
    warn "${BINARY_NAME} is not installed (not found in PATH)."
    exit 0
fi

info "Found: ${BIN_PATH}"

# ── Confirm ───────────────────────────────────────────────
if [ -t 0 ]; then
    printf "${AMBER}▸${RESET} Remove ${BIN_PATH}? [y/N]: "
    read -r answer </dev/tty
    case "$answer" in
        [yY]|[yY][eE][sS]) ;;
        *) info "Aborted."; exit 0 ;;
    esac
fi

# ── Remove ────────────────────────────────────────────────
if [ -w "$(dirname "$BIN_PATH")" ]; then
    rm -f "$BIN_PATH"
else
    info "Requesting sudo to remove ${BIN_PATH}…"
    sudo rm -f "$BIN_PATH"
fi

echo ""
printf "${GREEN}  ✓ ${BINARY_NAME} removed from ${BIN_PATH}${RESET}\n"
echo ""
printf "${DIM}  Workspace config files (.polyws, .polyws/) are not deleted.${RESET}\n"
printf "${DIM}  Remove them manually if no longer needed.${RESET}\n"
echo ""
