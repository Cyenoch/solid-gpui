#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
package_name="solid-gpui-host"

metadata="$(
  cd "$repo_root"
  cargo metadata --format-version 1 --no-deps |
    python3 -c 'import json, sys; package = next(p for p in json.load(sys.stdin)["packages"] if p["name"] == "solid-gpui"); print("solid-gpui-host\t" + package["version"])'
)"
IFS=$'\t' read -r metadata_name version <<< "$metadata"
if [[ "$metadata_name" != "$package_name" ]]; then
  printf 'unexpected package metadata name: %s\n' "$metadata_name" >&2
  exit 1
fi

target="$(
  cd "$repo_root"
  rustc -vV | python3 -c 'import sys; print(next(line.split(": ", 1)[1] for line in sys.stdin if line.startswith("host: ")))'
)"
bundle_name="${package_name}-${version}-${target}"
dist_dir="$repo_root/dist"
archive="$dist_dir/${bundle_name}.tar.gz"

stage_parent=""
stage_dir=""
extract_dir=""

cleanup() {
  if [[ -n "$stage_parent" && -d "$stage_parent" ]]; then
    rm -rf -- "$stage_parent"
  fi
  if [[ -n "$extract_dir" && -d "$extract_dir" ]]; then
    rm -rf -- "$extract_dir"
  fi
}
trap cleanup EXIT

fail() {
  printf 'host release check failed: %s\n' "$1" >&2
  exit 1
}

build_release_binary() {
  (
    cd "$repo_root"
    cargo build -p solid-gpui --bin "$package_name" --release --locked --target "$target"
  )
}

prepare_stage() {
  local binary
  stage_parent="$(mktemp -d "${TMPDIR:-/tmp}/solid-gpui-host-release.XXXXXX")"
  stage_dir="$stage_parent/$bundle_name"
  mkdir -p "$stage_dir" "$dist_dir"

  build_release_binary
  binary="$repo_root/target/$target/release/$package_name"
  [[ -x "$binary" ]] || fail "release binary is missing or not executable"

  cp "$binary" "$stage_dir/$package_name"
  cp "$repo_root/README.md" "$stage_dir/README.md"
  cp "$repo_root/LICENSE" "$stage_dir/LICENSE"
  [[ -f "$repo_root/THIRD-PARTY-NOTICES.md" ]] || fail "third-party notices inventory is missing"
  cp "$repo_root/THIRD-PARTY-NOTICES.md" "$stage_dir/THIRD-PARTY-NOTICES.md"
  if command -v xattr >/dev/null 2>&1; then
    xattr -rc "$stage_dir"
  fi
  (
    cd "$stage_dir"
    shasum -a 256 "$package_name" README.md LICENSE THIRD-PARTY-NOTICES.md > SHA256SUMS
  )
  python3 - "$stage_dir" "$package_name" <<'PY'
import os
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
for relative in (sys.argv[2], "README.md", "LICENSE", "THIRD-PARTY-NOTICES.md", "SHA256SUMS", "."):
    os.utime(root / relative, (0, 0), follow_symlinks=False)
PY
}

make_archive() {
  local destination="$1"
  local raw="$stage_parent/archive.tar"
  rm -f -- "$destination" "$raw"
  COPYFILE_DISABLE=1 tar \
    --format=ustar \
    --no-recursion \
    --uid=0 \
    --gid=0 \
    --uname=root \
    --gname=root \
    -cf "$raw" \
    -C "$stage_parent" \
    "$bundle_name/" \
    "$bundle_name/$package_name" \
    "$bundle_name/README.md" \
    "$bundle_name/LICENSE" \
    "$bundle_name/THIRD-PARTY-NOTICES.md" \
    "$bundle_name/SHA256SUMS"
  gzip -n -c "$raw" > "$destination"
  rm -f -- "$raw"
}

sha256_file() {
  local result
  result="$(shasum -a 256 "$1")"
  printf '%s' "${result%% *}"
}

print_bundle_details() {
  printf 'host release bundle: %s\n' "$archive"
  printf 'host release files:\n'
  tar -tzf "$archive"
  printf 'host release checksums:\n'
  cat "$stage_dir/SHA256SUMS"
}

bundle() {
  prepare_stage
  rm -f -- "$archive"
  make_archive "$archive"
  print_bundle_details
  rm -rf -- "$stage_parent"
  stage_parent=""
  stage_dir=""
}

check() {
  local list entry extracted help_output version_output second_archive first_hash second_hash
  prepare_stage
  rm -f -- "$archive"
  make_archive "$archive"
  second_archive="$stage_parent/${bundle_name}.second.tar.gz"
  make_archive "$second_archive"
  first_hash="$(sha256_file "$archive")"
  second_hash="$(sha256_file "$second_archive")"
  printf 'host release archive SHA256: %s\n' "$first_hash"
  printf 'host release second archive SHA256: %s\n' "$second_hash"
  [[ "$first_hash" == "$second_hash" ]] || fail "stage-to-archive SHA-256 changed between consecutive archives"

  extract_dir="$(mktemp -d "${TMPDIR:-/tmp}/solid-gpui-host-check.XXXXXX")"
  list="$extract_dir/archive.list"
  tar -xzf "$archive" -C "$extract_dir"
  tar -tzf "$archive" > "$list"

  while IFS= read -r entry; do
    case "$entry" in
      "$bundle_name/"|"$bundle_name/$package_name"|"$bundle_name/README.md"|"$bundle_name/LICENSE"|"$bundle_name/THIRD-PARTY-NOTICES.md"|"$bundle_name/SHA256SUMS")
        ;;
      *)
        fail "unexpected archive entry: $entry"
        ;;
    esac
  done < "$list"

  extracted="$extract_dir/$bundle_name"
  [[ -x "$extracted/$package_name" ]] || fail "extracted host is not executable"
  (
    cd "$extracted"
    shasum -a 256 -c SHA256SUMS
  )

  help_output="$(cd "$extracted" && ./$package_name --help)"
  case "$help_output" in
    *"Usage:"*"--version"*) ;;
    *) fail "extracted host help output is incomplete" ;;
  esac
  version_output="$(cd "$extracted" && ./$package_name --version)"
  [[ "$version_output" == "$package_name $version protocol=v5" ]] ||
    fail "unexpected version output: $version_output"

  printf 'host release check passed: %s\n' "$archive"
  rm -rf -- "$extract_dir" "$stage_parent"
  extract_dir=""
  stage_parent=""
  stage_dir=""
}

case "${1:-}" in
  bundle) bundle ;;
  check) check ;;
  *)
    printf 'usage: %s bundle|check\n' "$0" >&2
    exit 2
    ;;
esac
