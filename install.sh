#!/usr/bin/env bash
# TermLaps installer — Arch, Debian, Fedora and their derivatives.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/kronos-AUR/TermLaps/main/install.sh | bash
#   ./install.sh [--prefix DIR]
#
# The script installs the system dependencies, installs Rust via rustup if
# cargo is missing, builds the project in release mode and installs the
# binary plus a desktop entry. Default prefix: ~/.local (or /usr/local when
# run as root). If the current directory is not the repository, it is cloned
# into ${TMPDIR:-/tmp}/TermLaps first.

set -euo pipefail

APP="termlaps"
REPO_URL="https://github.com/kronos-AUR/TermLaps.git"
DESKTOP_ID="com.github.termlaps.Terminal"

PREFIX="$HOME/.local"
[ "$(id -u)" -eq 0 ] && PREFIX="/usr/local"

usage() {
    cat <<EOF
Usage: $0 [--prefix DIR]

  --prefix DIR   Installation prefix (default: ~/.local, /usr/local as root)
  -h, --help     Show this help
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix) PREFIX="${2:?--prefix needs a value}"; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage; exit 2 ;;
    esac
done

case "$PREFIX" in /*) ;; *) echo "--prefix must be an absolute path" >&2; exit 2 ;; esac

if [ "$(id -u)" -eq 0 ]; then SUDO=""; else SUDO="sudo"; fi

# --- Detect the distribution -------------------------------------------------
ID=""; ID_LIKE=""
[ -r /etc/os-release ] && . /etc/os-release
IDS=" ${ID:-} ${ID_LIKE:-} "
PM=""
case "$IDS" in
    *arch*)                     PM="arch"   ;;
    *debian*|*ubuntu*|*mint*)   PM="debian" ;;
    *fedora*|*rhel*|*centos*)   PM="fedora" ;;
esac

if [ -z "$PM" ]; then
    echo "Unsupported distribution (ID=${ID:-unknown})." >&2
    echo "Supported: Arch (+derivatives), Debian (+derivatives), Fedora (+derivatives)." >&2
    echo "Install the dependencies manually, see README.md." >&2
    exit 1
fi
echo "==> Detected package family: $PM (ID=${ID:-unknown})"

# --- System dependencies ------------------------------------------------------
case "$PM" in
    arch)
        $SUDO pacman -S --needed --noconfirm base-devel pkgconf git gtk4 libadwaita vte4
        ;;
    debian)
        $SUDO apt-get update
        DEBIAN_FRONTEND=noninteractive $SUDO apt-get install -y \
            build-essential pkg-config git libgtk-4-dev libadwaita-1-dev libvte-2.91-gtk4-dev
        ;;
    fedora)
        $SUDO dnf install -y gcc pkgconf-pkg-config git gtk4-devel libadwaita-devel vte291-gtk4-devel
        ;;
esac

# --- Rust toolchain -----------------------------------------------------------
if ! command -v cargo >/dev/null 2>&1; then
    echo "==> Installing Rust via rustup"
    curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain stable
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
fi
echo "==> Using $(cargo --version)"

# --- Get the sources ----------------------------------------------------------
SRC_DIR="$(pwd)"
if [ ! -f "$SRC_DIR/Cargo.toml" ]; then
    SRC_DIR="${TMPDIR:-/tmp}/TermLaps"
    echo "==> Cloning $REPO_URL into $SRC_DIR"
    git clone --depth 1 "$REPO_URL" "$SRC_DIR"
fi
cd "$SRC_DIR"

# --- Build --------------------------------------------------------------------
echo "==> Building $APP (release)"
cargo build --release

# --- Install ------------------------------------------------------------------
if [ -w "$PREFIX" ] || [ -w "$(dirname "$PREFIX")" ]; then INS=""; else INS="$SUDO"; fi

echo "==> Installing to $PREFIX"
$INS install -d "$PREFIX/bin" "$PREFIX/share/applications"
$INS install -m 755 target/release/"$APP" "$PREFIX/bin/$APP"

TMP_DESKTOP="$(mktemp)"
cat > "$TMP_DESKTOP" <<EOF
[Desktop Entry]
Name=TermLaps
Comment=Modern terminal for Linux with smart Ctrl+C
Exec=$PREFIX/bin/$APP
Icon=utilities-terminal
Type=Application
Categories=System;TerminalEmulator;
EOF
$INS install -m 644 "$TMP_DESKTOP" "$PREFIX/share/applications/$DESKTOP_ID.desktop"
rm -f "$TMP_DESKTOP"

command -v update-desktop-database >/dev/null 2>&1 && \
    $INS update-desktop-database "$PREFIX/share/applications" 2>/dev/null || true

echo
echo "TermLaps installed: $PREFIX/bin/$APP"
case ":$PATH:" in
    *":$PREFIX/bin:"*) ;;
    *) echo "Note: $PREFIX/bin is not in your PATH — add it to launch '$APP' from anywhere." ;;
esac
echo "Run it now with: $PREFIX/bin/$APP"
