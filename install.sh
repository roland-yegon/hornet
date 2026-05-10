#!/bin/bash

# Hornet Installation Script
# This script builds the Hornet compiler and installs it to /usr/local/bin

set -e

OS_TYPE="$(uname)"

echo "🐝 Detecting System: $OS_TYPE"

if [ "$OS_TYPE" == "Darwin" ]; then
    INSTALL_DIR="/usr/local/bin"
elif [ "$OS_TYPE" == "Linux" ]; then
    INSTALL_DIR="/usr/local/bin"
else
    echo "❌ This script currently only supports Linux and macOS."
    echo "For Windows, please follow the manual build instructions in docs/00-installation.md"
    exit 1
fi

echo "🐝 Building Hornet Compiler..."
cargo build --release

echo "🐝 Installing to $INSTALL_DIR..."
if [ -w "$INSTALL_DIR" ]; then
    cp target/release/hornet "$INSTALL_DIR/hornet"
else
    echo "🔒 Requires sudo permissions to install to $INSTALL_DIR"
    sudo cp target/release/hornet "$INSTALL_DIR/hornet"
fi

echo "🐝 Verifying installation..."
if command -v hornet >/dev/null 2>&1; then
    echo "✅ Hornet installed successfully!"
    hornet --help | head -n 1
else
    echo "❌ Installation failed or /usr/local/bin is not in your PATH."
    echo "Please ensure /usr/local/bin is in your PATH."
fi
