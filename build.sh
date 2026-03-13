#!/usr/bin/env bash
set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_DIR"

WASM_OUT="target/wasm32-unknown-unknown/release/space_elevator.wasm"
WEB_PKG="web/pkg"

echo "🔨 Building WASM (release)..."
cargo build --target wasm32-unknown-unknown --release

echo "🔗 Running wasm-bindgen..."
wasm-bindgen \
  "$WASM_OUT" \
  --out-dir "$WEB_PKG" \
  --target web \
  --no-typescript

echo "📦 Optimising with wasm-opt (if available)..."
if command -v wasm-opt &>/dev/null; then
  wasm-opt -Oz \
    "$WEB_PKG/space_elevator_bg.wasm" \
    -o "$WEB_PKG/space_elevator_bg.wasm"
  echo "   wasm-opt: done"
else
  echo "   wasm-opt not found – skipping (install binaryen to enable)"
fi

echo ""
echo "✅ Build complete!  Serve the web/ directory, e.g.:"
echo "   python3 -m http.server 8080 --directory web"
echo "   then open http://localhost:8080"
