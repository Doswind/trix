#!/usr/bin/env bash
# Install the trix binary from the latest trix-tui GitHub release.
#
#   curl -fsSL https://raw.githubusercontent.com/Doswind/trix-tui/master/install.sh | sh
#
# Environment:
#   TRIX_VERSION      release version to install (default: latest)
#   TRIX_INSTALL_DIR  destination directory (default: ~/.local/bin)
set -euo pipefail

REPO="Doswind/trix-tui"
BIN="trix"
VERSION="${TRIX_VERSION:-}"
DEST="${TRIX_INSTALL_DIR:-$HOME/.local/bin}"

say() { printf '%s\n' "$*"; }
die() { printf 'trix-tui installer: %s\n' "$*" >&2; exit 1; }

command -v curl >/dev/null 2>&1 || die "curl is required"
command -v tar >/dev/null 2>&1 || die "tar is required"

case "$(uname -s)" in
    Darwin) OS="macos" ;;
    Linux) OS="linux" ;;
    *) die "unsupported operating system: $(uname -s) (macOS, Linux and WSL are supported)" ;;
esac

case "$(uname -m)" in
    arm64|aarch64) ARCH="aarch64" ;;
    x86_64|amd64) ARCH="x86_64" ;;
    *) die "unsupported architecture: $(uname -m)" ;;
esac

case "$OS/$ARCH" in
    macos/aarch64|macos/x86_64) PLATFORM="macos-universal" ;;
    linux/x86_64) PLATFORM="linux-x86_64" ;;
    linux/aarch64) PLATFORM="linux-aarch64" ;;
    *) die "no prebuilt binary for $OS/$ARCH" ;;
esac

if [ -z "$VERSION" ]; then
    say "Looking up the latest trix-tui release..."
    VERSION="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
        | sed -n 's/.*"tag_name": *"v\{0,1\}\([^"]*\)".*/\1/p' \
        | head -n 1)"
    [ -n "$VERSION" ] || die "could not determine the latest version; set TRIX_VERSION"
fi
VERSION="${VERSION#v}"

ASSET="trix-tui-${VERSION}-${PLATFORM}.tar.gz"
BASE="https://github.com/$REPO/releases/download/v${VERSION}"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

say "Downloading $ASSET..."
curl -fsSL "$BASE/$ASSET" -o "$TMP/$ASSET" || die "download failed: $BASE/$ASSET"
curl -fsSL "$BASE/SHA256SUMS" -o "$TMP/SHA256SUMS" || die "could not download SHA256SUMS"

if command -v shasum >/dev/null 2>&1; then
    HASH_CMD="shasum -a 256"
elif command -v sha256sum >/dev/null 2>&1; then
    HASH_CMD="sha256sum"
else
    die "neither shasum nor sha256sum is available; cannot verify the download"
fi

LINE="$(grep " $ASSET\$" "$TMP/SHA256SUMS" || true)"
[ -n "$LINE" ] || die "SHA256SUMS does not list $ASSET"
( cd "$TMP" && printf '%s\n' "$LINE" | $HASH_CMD -c - ) || die "checksum verification failed"

tar -xzf "$TMP/$ASSET" -C "$TMP" || die "could not extract $ASSET"
[ -f "$TMP/$BIN" ] || die "archive does not contain the $BIN binary"

mkdir -p "$DEST"
install -m 0755 "$TMP/$BIN" "$DEST/$BIN"

say "Installed $DEST/$BIN"
"$DEST/$BIN" --version
case ":$PATH:" in
    *":$DEST:"*) ;;
    *) say "Add $DEST to PATH, for example: export PATH=\"$DEST:\$PATH\"" ;;
esac
