#!/bin/bash
set -e

echo "🚀 Building Fast Browser Search Native App..."

# Check dependencies
if ! command -v bun &> /dev/null; then
    echo "Error: bun is not installed"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed"
    exit 1
fi

# Build frontend
echo "📦 Building Frontend..."
cd frontend
bun install
bun run build
cd ..

# Build Tauri app
echo "🦀 Building Tauri App..."

# Use cargo tauri (preferred)
if command -v cargo-tauri &> /dev/null; then
    echo "Using cargo tauri build..."
    cargo tauri build
elif command -v bunx &> /dev/null; then
    echo "Using bunx tauri build..."
    bunx tauri build
else
    echo "Error: neither cargo-tauri nor bunx available to run tauri build"
    echo "Please install tauri-cli: cargo install tauri-cli"
    exit 1
fi

echo "✅ Build Complete!"
echo "App bundle should be in src-tauri/target/release/bundle/macos/"

