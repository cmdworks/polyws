#!/usr/bin/env bash
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# polyws installer — works via:
#   curl -fsSL https://raw.githubusercontent.com/cmdworks/polyws/main/install.sh | bash
#
# Options (env vars):
#   polyws_VERSION   — tag to install (default: latest)
#   polyws_INSTALL   — install directory (default: /usr/local/bin)
#   polyws_NAME      — binary name to install as (default: polyws)
#                   e.g.  polyws_NAME=poly   → installs as `poly`
#                         polyws_NAME=polyws → installs as `polyws`
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
set -euo pipefail

REPO="cmdworks/polyws"
INSTALL_DIR="${polyws_INSTALL:-/usr/local/bin}"
BINARY_NAME="${polyws_NAME:-}"

# ── Colours ────────────────────────────────────────────────
GREEN='\033[0;32m'
AMBER='\033[0;33m'
RED='\033[0;31m'
DIM='\033[2m'
RESET='\033[0m'

info()  { printf "${GREEN}▸${RESET} %s\n" "$*"; }
warn()  { printf "${AMBER}▸${RESET} %s\n" "$*"; }
fail()  { printf "${RED}✘${RESET} %s\n" "$*" >&2; exit 1; }

# ── Detect OS & architecture ──────────────────────────────
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)  os="unknown-linux-gnu" ;;
        Darwin*) os="apple-darwin" ;;
        MINGW*|MSYS*|CYGWIN*) fail "On Windows, download the .zip from GitHub Releases or build with: cargo build --release --target x86_64-pc-windows-msvc" ;;
        *)       fail "Unsupported OS: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64" ;;
        arm64|aarch64)  arch="aarch64" ;;
        *)              fail "Unsupported architecture: $(uname -m)" ;;
    esac

    echo "${arch}-${os}"
}

# ── Resolve version tag ───────────────────────────────────
resolve_version() {
    if [ -n "${polyws_VERSION:-}" ]; then
        echo "$polyws_VERSION"
        return
    fi
    # Fetch the latest release tag from GitHub API
    local tag
    tag=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | python3 -c "import sys,json; print(json.load(sys.stdin)['tag_name'])" 2>/dev/null || true)
    if [ -z "$tag" ]; then
        fail "Could not determine latest release. Set polyws_VERSION=vX.Y.Z manually."
    fi
    echo "$tag"
}

# ── Download & install ────────────────────────────────────
main() {
    echo ""
    printf "${GREEN}  ┌─────────────────────────────────────┐${RESET}\n"
    printf "${GREEN}  │       polyws  —  installer          │${RESET}\n"
    printf "${GREEN}  └─────────────────────────────────────┘${RESET}\n"
    echo ""

    local platform version archive_name url tmpdir

    # ── Ask for binary name if not set ───────────────────
    if [ -z "${BINARY_NAME}" ]; then
        # When piped from curl, stdin is the script itself — can't read interactively.
        # Detect pipe mode: if stdin is not a terminal, skip the prompt.
        if [ -t 0 ]; then
            printf "${GREEN}▸${RESET} Install as (default: polyws): "
            read -r name_input </dev/tty
            BINARY_NAME="${name_input:-polyws}"
        else
            BINARY_NAME="polyws"
        fi
    fi

    # Sanitise: allow only alphanumeric + dash + underscore
    BINARY_NAME="$(printf '%s' "$BINARY_NAME" | tr -cd '[:alnum:]_-')"
    if [ -z "${BINARY_NAME}" ]; then
        BINARY_NAME="polyws"
    fi

    platform=$(detect_platform)
    info "Detected platform: ${platform}"

    version=$(resolve_version)
    info "Installing version: ${version}"

    archive_name="polyws-${platform}.tar.gz"
    url="https://github.com/${REPO}/releases/download/${version}/${archive_name}"

    info "Downloading ${url}"

    tmpdir=$(mktemp -d)
    trap 'rm -rf "${tmpdir:-}"' EXIT

    if ! curl -fSL --progress-bar -o "${tmpdir}/${archive_name}" "$url"; then
        echo ""
        warn "Pre-built binary not found for ${platform} ${version}."
        echo ""
        info "Falling back to building from source…"
        build_from_source "$tmpdir"
        return
    fi

    info "Extracting…"
    tar -xzf "${tmpdir}/${archive_name}" -C "$tmpdir"

    install_binary "${tmpdir}/polyws" "${BINARY_NAME}"
}

# ── Fallback: build from source ──────────────────────────
build_from_source() {
    local tmpdir="$1"

    if ! command -v cargo &>/dev/null; then
        fail "cargo not found. Install Rust first: https://rustup.rs"
    fi

    info "Cloning ${REPO}…"
    git clone --depth 1 "https://github.com/${REPO}.git" "${tmpdir}/polyws-src"

    info "Building release binary (this may take a minute)…"
    cd "${tmpdir}/polyws-src"
    cargo build --release

    install_binary "${tmpdir}/polyws-src/target/release/polyws" "${BINARY_NAME}"
}

# ── Copy binary to install dir ───────────────────────────
install_binary() {
    local bin="$1"
    local name="${2:-polyws}"
    local dest="${INSTALL_DIR}/${name}"

    if [ ! -f "$bin" ]; then
        fail "Binary not found at ${bin}"
    fi

    chmod +x "$bin"

    # If install dir is not writable, use sudo
    if [ -w "$INSTALL_DIR" ]; then
        cp "$bin" "$dest"
    else
        info "Requesting sudo to install to ${dest}…"
        sudo cp "$bin" "$dest"
        sudo chmod +x "$dest"
    fi

    echo ""
    printf "${GREEN}  ✓ ${name} installed to ${dest}${RESET}\n"
    echo ""
    printf "${DIM}  Quick start:${RESET}\n"
    printf "${DIM}    mkdir my-project && cd my-project${RESET}\n"
    printf "${DIM}    ${name} init${RESET}\n"
    printf "${DIM}    ${name} add core git@github.com:org/core.git${RESET}\n"
    printf "${DIM}    ${name} bootstrap${RESET}\n"
    printf "${DIM}    ${name}              # launch TUI${RESET}\n"
    echo ""

    if command -v "$dest" &>/dev/null || [ -x "$dest" ]; then
        info "Version: $("$dest" --version 2>/dev/null || echo 'installed')"
    fi
}

main
