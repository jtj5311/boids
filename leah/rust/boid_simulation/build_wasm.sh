#!/bin/bash
set -e

echo "Building boid simulation for WebAssembly..."

cargo build --release --target wasm32-unknown-unknown

# Optimize with wasm-strip if available
if command -v wasm-strip &> /dev/null; then
    wasm-strip target/wasm32-unknown-unknown/release/boid_simulation.wasm
fi

# Copy to Next.js public directory
TARGET_DIR="../../../../leahchilders-portfolio/web_app/public/wasm"
mkdir -p "$TARGET_DIR"
cp target/wasm32-unknown-unknown/release/boid_simulation.wasm "$TARGET_DIR/"

echo "Build complete! File copied to $TARGET_DIR"
echo "File size: $(du -h target/wasm32-unknown-unknown/release/boid_simulation.wasm | cut -f1)"
