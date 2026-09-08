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
core_peers = []
for relative in ("packages/solid-gpui/package.json", "packages/solid-gpui-router/package.json", "packages/solid-gpui-shiki/package.json"):
    package = json.loads((root / relative).read_text())
    package_versions.append(package["version"])
    if relative != "packages/solid-gpui/package.json":
        peer_dependencies = package.get("peerDependencies")
        if not isinstance(peer_dependencies, dict):
            raise SystemExit(f"{relative} is missing peerDependencies")
        core_peer = peer_dependencies.get("@solid-gpui/core")
        if not isinstance(core_peer, str):
            raise SystemExit(f"{relative} is missing @solid-gpui/core peer dependency")
        core_peers.append(core_peer)
print("\t".join((match.group(1), *package_versions, *core_peers)))
PY
}

versions="$(read_versions)"
IFS=$'\t' read -r cargo_version core_version router_version shiki_version router_core_peer shiki_core_peer <<< "$versions"
update_versions=1
if [[ "$cargo_version" == "$version" && "$core_version" == "$version" && "$router_version" == "$version" && "$shiki_version" == "$version" && "$router_core_peer" == "^$version" && "$shiki_core_peer" == "^$version" ]]; then
  update_versions=0
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
release_files=(
  Cargo.toml
  packages/solid-gpui/package.json
  packages/solid-gpui-router/package.json
  packages/solid-gpui-shiki/package.json
  Cargo.lock
  bun.lock
  THIRD-PARTY-NOTICES.md
)
# Finish the backup before arming rollback; a missing input must never cause a
# partially populated backup to overwrite the workspace.
for file in "${release_files[@]}"; do
  mkdir -p "$backup_dir/$(dirname "$file")"
  if ! cp "$repo_root/$file" "$backup_dir/$file"; then
    rm -rf -- "$backup_dir"
    exit 1
  fi
done
completed=0
restore_on_failure() {
  local status=$?
  if ((completed == 0)); then
    for file in "${release_files[@]}"; do
      cp "$backup_dir/$file" "$repo_root/$file"
    done
  fi
  rm -rf -- "$backup_dir"
  return "$status"
}
trap restore_on_failure EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

if ((update_versions)); then
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
    ("packages/solid-gpui-shiki/package.json", "shiki"),
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
    if label != "core":
        updated_package, peer_count = re.subn(
            r'(?m)^(\s*"@solid-gpui/core"\s*:\s*")[^"]+("\s*,?\s*)$',
            rf'\g<1>^{version}\g<2>',
            updated_package,
            count=1,
        )
        if peer_count != 1:
            raise SystemExit(f"{label} @solid-gpui/core peer dependency is not uniquely anchored")
    package_path.write_text(updated_package)
PY
fi

(
  cd "$repo_root"
  if ((update_versions)); then
    cargo update --workspace
  fi
  # Re-running an already synchronized version still validates its release gates.
  cargo check --workspace --locked
)
(
  cd "$repo_root"
  if ((update_versions)); then
    bun install
  fi
  bun install --frozen-lockfile
)
(
  cd "$repo_root"
  bash scripts/third-party-notices.sh
)

versions="$(read_versions)"
IFS=$'\t' read -r cargo_version core_version router_version shiki_version router_core_peer shiki_core_peer <<< "$versions"
if [[ "$cargo_version" != "$version" || "$core_version" != "$version" || "$router_version" != "$version" || "$shiki_version" != "$version" || "$router_core_peer" != "^$version" || "$shiki_core_peer" != "^$version" ]]; then
  printf 'release-prep: version mismatch after update: cargo=%s core=%s router=%s shiki=%s router-core-peer=%s shiki-core-peer=%s\n' "$cargo_version" "$core_version" "$router_version" "$shiki_version" "$router_core_peer" "$shiki_core_peer" >&2
  exit 1
fi

printf 'release-prep: synchronized version %s\n' "$version"
printf 'release-prep diff summary:\n'
(
  cd "$repo_root"
  git diff --stat -- Cargo.toml Cargo.lock bun.lock THIRD-PARTY-NOTICES.md packages/solid-gpui/package.json packages/solid-gpui-router/package.json
)
completed=1
