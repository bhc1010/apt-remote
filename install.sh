#!/usr/bin/env bash
set -e

REPO="yourusername/apt-remote"
VERSION="${VERSION:-latest}"
INSTALL_DIR="/usr/local/bin"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac

echo "📦 Installing apt-remote ($OS/$ARCH)..."

# Get latest release tag if VERSION=latest
if [ "$VERSION" = "latest" ]; then
    VERSION=$(curl -s https://api.github.com/repos/$REPO/releases/latest | grep -Po '"tag_name": "\K.*?(?=")')
fi

# Construct download URLs
DEB_URL="https://github.com/$REPO/releases/download/$VERSION/apt-remote_${VERSION}_${ARCH}.deb"
BIN_URL="https://github.com/$REPO/releases/download/$VERSION/apt-remote-${OS}-${ARCH}"

if [[ "$OS" == "linux" ]]; then
    if command -v dpkg >/dev/null 2>&1; then
        echo "📥 Downloading .deb package..."
        curl -L "$DEB_URL" -o /tmp/apt-remote.deb
        echo "📦 Installing with dpkg..."
        sudo dpkg -i /tmp/apt-remote.deb || sudo apt-get install -f -y
        rm /tmp/apt-remote.deb
        echo "✅ apt-remote installed!"
        exit 0
    else
        echo "📥 Downloading prebuilt binary..."
        curl -L "$BIN_URL" -o /tmp/apt-remote
        chmod +x /tmp/apt-remote
        sudo mv /tmp/apt-remote "$INSTALL_DIR/"
        echo "✅ apt-remote installed to $INSTALL_DIR"
        exit 0
    fi
fi

if [[ "$OS" == "darwin" ]]; then
    echo "📥 Downloading macOS binary..."
    curl -L "$BIN_URL" -o /tmp/apt-remote
    chmod +x /tmp/apt-remote
    sudo mv /tmp/apt-remote "$INSTALL_DIR/"
    echo "✅ apt-remote installed to $INSTALL_DIR"
    exit 0
fi

echo "⚠️ No prebuilt binary available for $OS/$ARCH. Installing via Cargo..."
if command -v cargo >/dev/null 2>&1; then
    cargo install apt-remote --locked
else
    echo "❌ Cargo not found. Please install Rust toolchain first."
    exit 1
fi
