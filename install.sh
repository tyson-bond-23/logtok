#!/usr/bin/env bash
set -euo pipefail

echo "Building and installing logtok..."

if ! command -v cargo &>/dev/null; then
    echo "Error: Rust is not installed. Install it from https://rustup.rs"
    exit 1
fi

cargo install --path .

echo ""
echo "Done! logtok is now available globally."
echo "Try: logtok --help"
