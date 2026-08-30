#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
version="${1:-}"

if [[ -z "$version" ]]; then
  printf 'usage: %s MAJOR.MINOR.PATCH\n' "$0" >&2
  exit 2
fi
if [[ "$version" == "0.0.0" || ! "$version" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
  printf 'release-prep: VERSION must be strict MAJOR.MINOR.PATCH and not 0.0.0: %s\n' "$version" >&2
  exit 2
fi

read_versions() {
  python3 - "$repo_root" <<'PY'
import json
import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1])
cargo = (root / "Cargo.toml").read_text()
section = re.search(r"(?ms)^\[workspace\.package\]\n(.*?)(?=^\[|\Z)", cargo)
if section is None:
    raise SystemExit("missing [workspace.package] section")
match = re.search(r"(?m)^version\s*=\s*\"([^\"]+)\"\s*$", section.group(1))
if match is None:
    raise SystemExit("missing workspace package version")
core = json.loads((root / "packages/react-gpui/package.json").read_text())["version"]
dev = json.loads((root / "packages/react-gpui-dev/package.json").read_text())["version"]
host_config = (root / "packages/react-gpui/src/renderer/host-config.ts").read_text()
renderer_match = re.search(r'(?m)^\s*rendererVersion:\s*"([^"]+)"\s*,?\s*$', host_config)
if renderer_match is None:
    raise SystemExit("missing rendererVersion in host config")
print(f"{match.group(1)}\t{core}\t{dev}\t{renderer_match.group(1)}")
PY
}

IFS=$'\t' read -r cargo_version core_version dev_version renderer_version <<< "$(read_versions)"
if [[ "$cargo_version" == "$version" && "$core_version" == "$version" && "$dev_version" == "$version" && "$renderer_version" == "$version" ]]; then
  printf 'release-prep: all manifests and renderer metadata already use %s; nothing to do\n' "$version"
  exit 0
fi

if ! python3 - "$repo_root/CHANGELOG.md" "$version" <<'PY'; then
import pathlib
import re
import sys

changelog = pathlib.Path(sys.argv[1])
version = sys.argv[2]
if not changelog.is_file():
    raise SystemExit("CHANGELOG.md is missing")
if re.search(rf"(?m)^## \[{re.escape(version)}\]\s*$", changelog.read_text()) is None:
    raise SystemExit(f"CHANGELOG.md has no ## [{version}] section")
PY
  printf 'release-prep: add ## [%s] to CHANGELOG.md first (move the Unreleased entries), then retry\n' "$version" >&2
  exit 1
fi

backup_dir="$(mktemp -d "${TMPDIR:-/tmp}/react-gpui-release-prep.XXXXXX")"
completed=0
restore_on_failure() {
  local status=$?
  if ((completed == 0)); then
    cp "$backup_dir/Cargo.toml" "$repo_root/Cargo.toml"
    cp "$backup_dir/core-package.json" "$repo_root/packages/react-gpui/package.json"
    cp "$backup_dir/dev-package.json" "$repo_root/packages/react-gpui-dev/package.json"
    cp "$backup_dir/host-config.ts" "$repo_root/packages/react-gpui/src/renderer/host-config.ts"
    cp "$backup_dir/Cargo.lock" "$repo_root/Cargo.lock"
    cp "$backup_dir/core-bun.lock" "$repo_root/packages/react-gpui/bun.lock"
    cp "$backup_dir/dev-bun.lock" "$repo_root/packages/react-gpui-dev/bun.lock"
  fi
  rm -rf -- "$backup_dir"
  return "$status"
}
trap restore_on_failure EXIT
cp "$repo_root/Cargo.toml" "$backup_dir/Cargo.toml"
cp "$repo_root/packages/react-gpui/package.json" "$backup_dir/core-package.json"
cp "$repo_root/packages/react-gpui-dev/package.json" "$backup_dir/dev-package.json"
cp "$repo_root/packages/react-gpui/src/renderer/host-config.ts" "$backup_dir/host-config.ts"
cp "$repo_root/Cargo.lock" "$backup_dir/Cargo.lock"
cp "$repo_root/packages/react-gpui/bun.lock" "$backup_dir/core-bun.lock"
cp "$repo_root/packages/react-gpui-dev/bun.lock" "$backup_dir/dev-bun.lock"

python3 - "$repo_root" "$version" <<'PY'
import json
import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1])
version = sys.argv[2]

cargo_path = root / "Cargo.toml"
cargo = cargo_path.read_text()
section = re.search(r"(?ms)^\[workspace\.package\]\n(.*?)(?=^\[|\Z)", cargo)
if section is None:
    raise SystemExit("missing [workspace.package] section")
section_text = section.group(1)
updated_section, count = re.subn(
    r'(?m)^(version\s*=\s*")[^\"]+("\s*)$',
    rf'\g<1>{version}\g<2>',
    section_text,
    count=1,
)
if count != 1:
    raise SystemExit("workspace package version line is not uniquely anchored")
cargo_path.write_text(cargo[:section.start(1)] + updated_section + cargo[section.end(1):])

for relative in ("packages/react-gpui/package.json", "packages/react-gpui-dev/package.json"):
    path = root / relative
    text = path.read_text()
    updated, count = re.subn(
        r'(?m)^(\s*"version"\s*:\s*")[^\"]+("\s*,?\s*)$',
        rf'\g<1>{version}\g<2>',
        text,
        count=1,
    )
    if count != 1:
        raise SystemExit(f"{relative} version line is not uniquely anchored")
    path.write_text(updated)

host_config_path = root / "packages/react-gpui/src/renderer/host-config.ts"
host_config = host_config_path.read_text()
updated_host_config, count = re.subn(
    r'(?m)^(\s*rendererVersion:\s*")[^\"]+("\s*,?\s*)$',
    rf'\g<1>{version}\g<2>',
    host_config,
    count=1,
)
if count != 1:
    raise SystemExit("rendererVersion line is not uniquely anchored")
host_config_path.write_text(updated_host_config)
PY

(
  cd "$repo_root"
  cargo update --workspace
  cargo check --workspace --locked
)
(
  cd "$repo_root/packages/react-gpui"
  bun install
  bun install --frozen-lockfile
)
(
  cd "$repo_root/packages/react-gpui-dev"
  bun install
  bun install --frozen-lockfile
)

IFS=$'\t' read -r cargo_version core_version dev_version renderer_version <<< "$(read_versions)"
if [[ "$cargo_version" != "$version" || "$core_version" != "$version" || "$dev_version" != "$version" || "$renderer_version" != "$version" ]]; then
  printf 'release-prep: version mismatch after update: %s / %s / %s / %s\n' "$cargo_version" "$core_version" "$dev_version" "$renderer_version" >&2
  exit 1
fi

printf 'release-prep: synchronized version %s\n' "$version"
printf 'release-prep diff summary:\n'
(
  cd "$repo_root"
  git diff --stat -- Cargo.toml Cargo.lock packages/react-gpui/package.json packages/react-gpui/bun.lock packages/react-gpui-dev/package.json packages/react-gpui-dev/bun.lock packages/react-gpui/src/renderer/host-config.ts
)
completed=1
