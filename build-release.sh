#!/usr/bin/env bash
set -euo pipefail

# Build a universal binary (Apple Silicon + Intel) for distribution

echo "Building WhatCable CLI for release..."

if ! rustup target list --installed | grep -q aarch64-apple-darwin; then
    echo "Installing aarch64-apple-darwin target..."
    rustup target add aarch64-apple-darwin
fi

# Build for Apple Silicon
echo "→ Building for aarch64-apple-darwin (Apple Silicon)..."
cargo build --release --target aarch64-apple-darwin

if ! rustup target list --installed | grep -q x86_64-apple-darwin; then
    echo "Installing x86_64-apple-darwin target..."
    rustup target add x86_64-apple-darwin
fi

# Build for Intel
echo "→ Building for x86_64-apple-darwin (Intel)..."
cargo build --release --target x86_64-apple-darwin

# Create universal binary
echo "→ Creating universal binary..."
mkdir -p target/universal/release
lipo -create \
    target/aarch64-apple-darwin/release/whatcable \
    target/x86_64-apple-darwin/release/whatcable \
    -output target/universal/release/whatcable

# Verify
echo "→ Verifying architectures..."
lipo -info target/universal/release/whatcable

# Strip debug symbols to reduce size
echo "→ Stripping debug symbols..."
strip target/universal/release/whatcable

SIZE=$(du -h target/universal/release/whatcable | cut -f1)
echo "✓ Universal binary created: target/universal/release/whatcable ($SIZE)"
echo ""
echo "Install with:"
echo "  sudo cp target/universal/release/whatcable /usr/local/bin/"
echo "  sudo chmod +x /usr/local/bin/whatcable"
