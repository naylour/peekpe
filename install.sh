#!/bin/sh

set -e

OS=$(uname -s)

if [ "$OS" = "Linux" ]; then
    BIN="peekpe-linux"
elif [ "$OS" = "Darwin" ]; then
    BIN="peekpe-macos"
else
    echo "Unsupported OS"
    exit 1
fi

INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

curl -L "https://github.com/naylour/peekpe/releases/latest/download/$BIN" \
    -o "$INSTALL_DIR/peekpe"

chmod +x "$INSTALL_DIR/peekpe"

echo "Installed as peekpe"
