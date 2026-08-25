#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/react-gpui-pack-smoke.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT

core_archive="$tmp_dir/react-gpui-core.tgz"
dev_archive="$tmp_dir/react-gpui-dev.tgz"
consumer_dir="$tmp_dir/consumer"

pack() {
  local package_dir="$1"
  local archive="$2"
  (
    cd "$package_dir"
    bun pm pack --dry-run --quiet
    bun pm pack --filename "$archive" --quiet
  )
}

check_archive() {
  local archive="$1"
  local list="$tmp_dir/$(basename "$archive").list"
  local has_js=0
  local has_dts=0
  local has_license=0
  local has_readme=0

  tar -tzf "$archive" > "$list"
  while IFS= read -r entry; do
    case "$entry" in
      package/package.json|package/README.md|package/LICENSE|package/dist/*)
        ;;
      *)
        printf 'unexpected file in %s: %s\n' "$archive" "$entry" >&2
        exit 1
        ;;
    esac
    case "$entry" in
      package/README.md) has_readme=1 ;;
      package/LICENSE) has_license=1 ;;
      package/dist/index.js) has_js=1 ;;
      package/dist/index.d.ts) has_dts=1 ;;
    esac
  done < "$list"

  if ! tar -xOf "$archive" package/LICENSE | cmp -s - "$repo_root/LICENSE"; then
    printf 'license drift in %s\n' "$archive" >&2
    exit 1
  fi
  if ((has_js == 0 || has_dts == 0 || has_license == 0 || has_readme == 0)); then
    printf 'missing required package files in %s\n' "$archive" >&2
    exit 1
  fi
}

pack "$repo_root/packages/react-gpui" "$core_archive"
pack "$repo_root/packages/react-gpui-dev" "$dev_archive"
check_archive "$core_archive"
check_archive "$dev_archive"

mkdir -p "$consumer_dir"
cat > "$consumer_dir/package.json" <<EOF
{
  "name": "react-gpui-pack-consumer",
  "private": true,
  "type": "module",
  "dependencies": {
    "@react-gpui/core": "file:$core_archive",
    "@react-gpui/dev": "file:$dev_archive",
    "react": "19.2.8"
  },
  "devDependencies": {
    "@types/react": "19.2.2",
    "bun-types": "1.1.29",
    "typescript": "5.9.3"
  }
}
EOF

cat > "$consumer_dir/runtime.ts" <<'EOF'
import React from "react";
import { Pressable, StyleSheet, Text, View } from "@react-gpui/core";
import { render, transformRefreshSource } from "@react-gpui/dev";

const styles = StyleSheet.create({ root: { flexDirection: "column" } });
if (styles.root?.flexDirection !== "column" || View === undefined || Text === undefined) {
  throw new Error("consumer could not import the built core API");
}

let pressed = false;
const testView = render(
  React.createElement(
    Pressable,
    { accessibilityLabel: "pack smoke", onPress: () => (pressed = true) },
    React.createElement(Text, null, "pack smoke"),
  ),
);
const button = testView.node("Pressable", (node) => node.accessibility?.[1] === "pack smoke");
testView.press(button);
if (!pressed || testView.frames.length !== 1 || testView.commits().length !== 1) {
  throw new Error("consumer could not execute the packed headless testing API");
}
testView.unmount();

const transformed = await transformRefreshSource(
  "import React, { useState } from 'react'; export default function Consumer() { useState(0); return null; }",
  "/tmp/react-gpui-pack-consumer.tsx",
);
if (!transformed.includes("__reactGpuiFamily")) {
  throw new Error("consumer could not execute the built Fast Refresh transform");
}
EOF

cat > "$consumer_dir/types.tsx" <<'EOF'
import type { Root, TextProps } from "@react-gpui/core";
import type { RefreshRoot, RenderResult, TestNodeHandle } from "@react-gpui/dev";

const props: TextProps = { children: "from a tarball" };
declare const root: Root;
declare const refreshRoot: RefreshRoot;
declare const testResult: RenderResult;
declare const testNode: TestNodeHandle;
void [props, root, refreshRoot, testResult, testNode];
EOF

(
  cd "$consumer_dir"
  bun install --no-progress
  bun run runtime.ts
  bunx --no-install tsc --noEmit --strict --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler --jsx react-jsx --types bun-types,react runtime.ts types.tsx
)

printf 'package tarball consumer smoke passed\n'
