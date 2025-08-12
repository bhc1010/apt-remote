#!/usr/bin/env bash
set -e

REPO="bhc1010/apt-remote"

OS="$(uname -s)"
echo "🔍 Detected OS: $OS"

echo "📡 Fetching latest release info..."
LATEST_URL=$(curl -s https://api.github.com/repos/$REPO/releases/latest)

if [[ "$OS" == "Linux" ]]; then
    echo "📦 Downloading latest .deb package..."
    FILE_URL=$(echo "$LATEST_URL" | grep "browser_download_url" | grep ".deb" | cut -d '"' -f 4)

    if [ -z "$FILE_URL" ]; then
        echo "❌ Could not find .deb file in latest release."
        exit 1
    fi

    TMP_FILE=$(mktemp)
    curl -L "$FILE_URL" -o "$TMP_FILE"

    echo "📦 Installing .deb package..."
    sudo apt install -y "$TMP_FILE"

    rm "$TMP_FILE"
    echo "✅ apt-remote installed successfully!"

elif [[ "$OS" == "Darwin" ]]; then
    echo "🍎 Downloading latest .pkg package..."
    FILE_URL=$(echo "$LATEST_URL" | grep "browser_download_url" | grep ".pkg" | cut -d '"' -f 4)

    if [ -z "$FILE_URL" ]; then
        echo "❌ Could not find .pkg file in latest release."
        exit 1
    fi

    TMP_FILE=$(mktemp)
    curl -L "$FILE_URL" -o "$TMP_FILE"

    echo "📦 Installing .pkg package..."
    sudo installer -pkg "$TMP_FILE" -target /

    rm "$TMP_FILE"
    echo "✅ apt-remote installed successfully!"

else
    echo "❌ Unsupported OS: $OS"
    echo "This script currently supports Linux (.deb) and macOS (.pkg) only."
    exit 1
fi
