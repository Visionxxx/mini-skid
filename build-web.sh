#!/bin/bash
set -e

echo "Bygger WASM..."
cargo build --release --target wasm32-unknown-unknown

echo "Kopierer til web/..."
cp target/wasm32-unknown-unknown/release/mini_skid.wasm web/

SIZE=$(du -h web/mini_skid.wasm | cut -f1)
echo "Ferdig! WASM: $SIZE"
echo ""
echo "Start lokal server:"
echo "  cd web && python3 -m http.server 8080"
echo "  Apne http://localhost:8080"
