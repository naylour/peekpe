#!/bin/sh

set -e

REPO="naylour/peekpe"

OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
    Linux) OS="linux" ;;
    Darwin) OS="macos" ;;
    *)
        echo "Неподдерживаемая операционная система"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64) ARCH="amd64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *)
        echo "Неподдерживаемая архитектура"
        exit 1
        ;;
esac

BINARY="peekpe-${OS}-${ARCH}"

URL="https://github.com/$REPO/releases/latest/download/$BINARY"

INSTALL_DIR="$HOME/.local/bin"
TARGET="$INSTALL_DIR/peekpe"

mkdir -p "$INSTALL_DIR"

echo "Downloading $BINARY..."

curl -fsSL "$URL" -o "$TARGET"

chmod +x "$TARGET"

echo ""
echo "PeekPe installed!"
echo "Binary: $TARGET"

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo ""
        echo "Add this to your shell config:"
        echo "export PATH=\"$INSTALL_DIR:\$PATH\""
        ;;
esac
