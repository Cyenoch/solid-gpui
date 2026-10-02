#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
bindgen="${WASM_BINDGEN:-wasm-bindgen}"
if [[ "$("$bindgen" --version)" != "wasm-bindgen 0.2.121" ]]; then
  echo "Install wasm-bindgen-cli 0.2.121 or set WASM_BINDGEN to that executable." >&2
  exit 1
fi
# This nightly is required by gpui-pre-web's wasm_thread dependency.
cargo +nightly-2026-07-28 build --locked -p solid-gpui-web --target wasm32-unknown-unknown --release
wasm_target_directory="$(cargo +nightly-2026-07-28 metadata --locked --no-deps --format-version 1 | bun -e 'const metadata = await Bun.stdin.json(); if (typeof metadata.target_directory !== "string" || !metadata.target_directory) throw new Error("Cargo target directory is missing"); process.stdout.write(metadata.target_directory);')"
"$bindgen" "$wasm_target_directory/wasm32-unknown-unknown/release/solid_gpui_web.wasm" --out-dir examples/website/src/wasm --target web
