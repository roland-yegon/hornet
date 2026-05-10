#!/bin/bash

# Hornet Installation Script
# This script builds the Hornet compiler and installs it to /usr/local/bin

set -e

echo "🐝 Building Hornet Compiler..."
cargo build --release

echo "🐝 Installing to /usr/local/bin..."
# Using sudo if necessary, but we'll try to copy directly if we have permissions
if [ -w /usr/local/bin ]; then
    cp target/release/hornet /usr/local/bin/hornet
else
    echo "Requires sudo permissions to install to /usr/local/bin"
    sudo cp target/release/hornet /usr/local/bin/hornet
fi

echo "🐝 Verifying installation..."
if command -v hornet >/dev/null 2>&1; then
    echo "✅ Hornet installed successfully!"
    hornet --help | head -n 1
else
    echo "❌ Installation failed or /usr/local/bin is not in your PATH."
    echo "Please ensure /usr/local/bin is in your PATH."
fi
