#!/bin/bash

# Build script for OHMS Agent (Internet Computer canister)
# This builds for the WebAssembly target required for IC

echo "Building OHMS Agent for Internet Computer..."

# Build for WASM target
cargo build --target wasm32-unknown-unknown

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo "📦 WASM file created at: target/wasm32-unknown-unknown/debug/ohms_agent.wasm"
else
    echo "❌ Build failed!"
    exit 1
fi
