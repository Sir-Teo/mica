#!/bin/bash
set -e

echo "🔧 Building Mica for WebAssembly..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Build the WASM package
echo "📦 Compiling to WebAssembly..."
wasm-pack build --target web --out-dir docs/wasm --release

echo "✅ WASM build complete!"
echo "📁 Output: docs/wasm/"
echo ""
echo "To use in the playground, include:"
echo "  <script type=\"module\">"
echo "    import init, { tokenize, parse_ast, resolve_code, check_code, lower_code, generate_ir } from './wasm/mica.js';"
echo "    await init();"
echo "  </script>"
