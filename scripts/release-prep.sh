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
match = re.search(r'(?m)^version\s*=\s*"([^"]+)"\s*$', section.group(1))
if match is None:
    raise SystemExit("missing workspace package version")
package_versions = []
router_core_peer = None
for relative in ("packages/solid-gpui/package.json", "packages/solid-gpui-router/package.json"):
    package = json.loads((root / relative).read_text())
    package_versions.append(package["version"])
    if relative.endswith("solid-gpui-router/package.json"):
        peer_dependencies = package.get("peerDependencies")
        if not isinstance(peer_dependencies, dict):
            raise SystemExit("router package is missing peerDependencies")
        router_core_peer = peer_dependencies.get("@solid-gpui/core")
        if not isinstance(router_core_peer, str):
            raise SystemExit("router package is missing @solid-gpui/core peer dependency")
print("\t".join((match.group(1), *package_versions, router_core_peer)))
PY
}

IFS=$'\t' read -r cargo_version core_version router_version router_core_peer <<< "$(read_versions)"
if [[ "$cargo_version" == "$version" && "$core_version" == "$version" && "$router_version" == "$version" && "$router_core_peer" == "^$version" ]]; then
  printf 'release-prep: all manifests already use %s; nothing to do\n' "$version"
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
  printf 'release-prep: add ## [%s] to CHANGELOG.md first, then retry\n' "$version" >&2
  exit 1
fi

backup_dir="$(mktemp -d "${TMPDIR:-/tmp}/solid-gpui-release-prep.XXXXXX")"
completed=0
restore_on_failure() {
  local status=$?
  if ((completed == 0)); then
    cp "$backup_dir/Cargo.toml" "$repo_root/Cargo.toml"
    cp "$backup_dir/core-package.json" "$repo_root/packages/solid-gpui/package.json"
    cp "$backup_dir/router-package.json" "$repo_root/packages/solid-gpui-router/package.json"
    cp "$backup_dir/Cargo.lock" "$repo_root/Cargo.lock"
    cp "$backup_dir/bun.lock" "$repo_root/bun.lock"
    cp "$backup_dir/THIRD-PARTY-NOTICES.md" "$repo_root/THIRD-PARTY-NOTICES.md"
  fi
  rm -rf -- "$backup_dir"
  return "$status"
}
trap restore_on_failure EXIT
cp "$repo_root/Cargo.toml" "$backup_dir/Cargo.toml"
cp "$repo_root/packages/solid-gpui/package.json" "$backup_dir/core-package.json"
cp "$repo_root/packages/solid-gpui-router/package.json" "$backup_dir/router-package.json"
cp "$repo_root/Cargo.lock" "$backup_dir/Cargo.lock"
cp "$repo_root/bun.lock" "$backup_dir/bun.lock"
cp "$repo_root/THIRD-PARTY-NOTICES.md" "$backup_dir/THIRD-PARTY-NOTICES.md"

python3 - "$repo_root" "$version" <<'PY'
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
updated_section, count = re.subn(
    r'(?m)^(version\s*=\s*")[^"]+("\s*)$',
    rf'\g<1>{version}\g<2>',
    section.group(1),
    count=1,
)
if count != 1:
    raise SystemExit("workspace package version line is not uniquely anchored")
cargo_path.write_text(cargo[:section.start(1)] + updated_section + cargo[section.end(1):])

for relative, label in (
    ("packages/solid-gpui/package.json", "core"),
    ("packages/solid-gpui-router/package.json", "router"),
):
    package_path = root / relative
    package = package_path.read_text()
    updated_package, count = re.subn(
        r'(?m)^(\s*"version"\s*:\s*")[^"]+("\s*,?\s*)$',
        rf'\g<1>{version}\g<2>',
        package,
        count=1,
    )
    if count != 1:
        raise SystemExit(f"{label} package version line is not uniquely anchored")
    if label == "router":
        updated_package, peer_count = re.subn(
            r'(?m)^(\s*"@solid-gpui/core"\s*:\s*")[^"]+("\s*,?\s*)$',
            rf'\g<1>^{version}\g<2>',
            updated_package,
            count=1,
        )
        if peer_count != 1:
            raise SystemExit("router @solid-gpui/core peer dependency is not uniquely anchored")
    package_path.write_text(updated_package)
PY

(
  cd "$repo_root"
  cargo update --workspace
  cargo check --workspace --locked
)
(
  cd "$repo_root"
  bun install
  bun install --frozen-lockfile
)
(
  cd "$repo_root"
  bash scripts/third-party-notices.sh
)

IFS=$'\t' read -r cargo_version core_version router_version router_core_peer <<< "$(read_versions)"
if [[ "$cargo_version" != "$version" || "$core_version" != "$version" || "$router_version" != "$version" || "$router_core_peer" != "^$version" ]]; then
  printf 'release-prep: version mismatch after update: cargo=%s core=%s router=%s router-core-peer=%s\n' "$cargo_version" "$core_version" "$router_version" "$router_core_peer" >&2
  exit 1
fi

printf 'release-prep: synchronized version %s\n' "$version"
printf 'release-prep diff summary:\n'
(
  cd "$repo_root"
  git diff --stat -- Cargo.toml Cargo.lock bun.lock THIRD-PARTY-NOTICES.md packages/solid-gpui/package.json packages/solid-gpui-router/package.json
)
completed=1
