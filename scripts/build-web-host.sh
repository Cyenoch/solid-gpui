#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
bindgen="${WASM_BINDGEN:-wasm-bindgen}"
if [[ "$("$bindgen" --version)" != "wasm-bindgen 0.2.121" ]]; then
  echo "Install wasm-bindgen-cli 0.2.121 or set WASM_BINDGEN to that executable." >&2
  exit 1
fi
bun scripts/native-codegen.ts --package solid-gpui --bin solid-gpui-host --features gpui-component --out packages/solid-gpui/src/components.ts
# This nightly is required by gpui-pre-web's wasm_thread dependency.
cargo +nightly-2026-07-28 build --locked -p solid-gpui-web --target wasm32-unknown-unknown --release
"$bindgen" target/wasm32-unknown-unknown/release/solid_gpui_web.wasm --out-dir examples/website/src/wasm --target web
