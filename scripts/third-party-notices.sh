#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if (( $# > 1 )); then
  printf 'usage: %s [output-path]\n' "$0" >&2
  exit 2
fi
output_path="${1:-$repo_root/THIRD-PARTY-NOTICES.md}"
if [[ "$output_path" != /* ]]; then
  output_path="$repo_root/$output_path"
fi

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/solid-gpui-third-party-notices.XXXXXX")"
trap 'rm -rf -- "$work_dir"' EXIT

(
  cd "$repo_root"
  cargo deny --all-features list --format json --layout crate > "$work_dir/cargo-deny.json"
  cargo metadata --locked --all-features --format-version 1 > "$work_dir/cargo-metadata.json"
)
(
  cd "$repo_root"
  bun pm licenses --all > "$work_dir/bun-workspace.txt"
)

rustc_version="$(cd "$repo_root" && rustc -vV)"
target="$(python3 -c 'import sys; print(next(line.split(": ", 1)[1] for line in sys.stdin if line.startswith("host: ")))' <<< "$rustc_version")"
if [[ -n "${SOURCE_DATE_EPOCH:-}" ]]; then
  generated_date="$(date -u -r "$SOURCE_DATE_EPOCH" +%Y-%m-%d)"
else
  generated_date="$(git -C "$repo_root" log -1 --format=%cs -- Cargo.lock Cargo.toml bun.lock package.json packages/solid-gpui/package.json packages/solid-gpui-router/package.json scripts/tasks.ts scripts/third-party-notices.sh)"
fi
[[ -n "$generated_date" ]] || { printf 'unable to determine a stable generation date\n' >&2; exit 1; }

mkdir -p "$(dirname "$output_path")"
python3 - "$repo_root" "$output_path" "$work_dir/cargo-deny.json" "$work_dir/cargo-metadata.json" "$work_dir/bun-workspace.txt" "$target" "$generated_date" <<'PY'
import datetime as dt
import json
import os
import re
import sys
import tempfile
import pathlib
from collections import defaultdict

(
    repo_root,
    output_path,
    cargo_licenses_path,
    cargo_metadata_path,
    bun_workspace_path,
    target,
    generated_date,
) = sys.argv[1:]

root = pathlib.Path(repo_root)
output = pathlib.Path(output_path)

if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", generated_date):
    raise SystemExit(f"invalid generation date: {generated_date!r}")
try:
    dt.date.fromisoformat(generated_date)
except ValueError as exc:
    raise SystemExit(f"invalid generation date: {generated_date!r}") from exc

metadata = json.loads(pathlib.Path(cargo_metadata_path).read_text(encoding="utf-8"))
packages = metadata.get("packages")
if not isinstance(packages, list):
    raise SystemExit("cargo metadata did not contain a packages array")

# cargo-deny supplies resolved SPDX identifiers, including license-file
# resolutions that Cargo metadata cannot classify by itself. Its JSON output
# preserves unlicensed entries without relying on the TSV table's column count.
cargo_licenses = json.loads(pathlib.Path(cargo_licenses_path).read_text(encoding="utf-8"))
if not isinstance(cargo_licenses, dict) or not cargo_licenses:
    raise SystemExit("cargo deny returned no crate licenses")

metadata_by_name_version = {}
for package in packages:
    name = package.get("name")
    version = package.get("version")
    if not isinstance(name, str) or not isinstance(version, str):
        raise SystemExit("cargo metadata package is missing name/version")
    key = (name, version)
    if key in metadata_by_name_version:
        raise SystemExit(f"duplicate cargo metadata package key: {name}@{version}")
    metadata_by_name_version[key] = package

rust_third_party = defaultdict(list)
rust_project_owned = []
seen_rust = set()
for package_id, license_info in cargo_licenses.items():
    try:
        name, version, _source = package_id.split(" ", 2)
    except ValueError as exc:
        raise SystemExit(f"invalid cargo package reference: {package_id!r}") from exc
    package_ref = f"{name}@{version}"
    package = metadata_by_name_version.get((name, version))
    if package is None:
        raise SystemExit(f"cargo metadata is missing {package_ref}")

    source = package.get("source")
    if source is None:
        manifest = pathlib.Path(package["manifest_path"]).resolve()
        vendored_root = root / "vendor" / "gpui-component"
        if manifest.is_relative_to(vendored_root):
            source_kind = "vendored (https://github.com/longbridge/gpui-component @ 928c3eb776a3d733d9b771f7dea27a6a79242ced; local patches)"
            source_detail = "vendored"
        elif manifest.is_relative_to(root / "vendor"):
            provenance = package.get("metadata", {}).get("solid-gpui-vendor", {}).get("source")
            if not isinstance(provenance, str) or not provenance.startswith("https://"):
                raise SystemExit(f"vendored package requires source provenance: {package_ref}")
            source_kind = f"vendored ({provenance}; local patches)"
            source_detail = "vendored"
        else:
            source_kind = "local (project-owned)"
            source_detail = "local"
    elif source.startswith("registry+"):
        source_kind = "registry (crates.io)"
        source_detail = "registry"
    elif source.startswith("git+"):
        source_kind = f"git ({source})"
        source_detail = "git"
    else:
        source_kind = f"other ({source})"
        source_detail = "other"

    licenses = license_info.get("licenses") if isinstance(license_info, dict) else None
    if not isinstance(licenses, list) or not licenses or not all(isinstance(item, str) and item for item in licenses):
        raise SystemExit(f"cargo deny returned no SPDX license for {package_ref}")
    licenses = sorted(set(licenses))
    identity = (name, version, source_kind, tuple(licenses))
    if identity in seen_rust:
        raise SystemExit(f"duplicate cargo deny package row: {package_ref}")
    seen_rust.add(identity)
    record = {
        "name": name,
        "version": version,
        "licenses": licenses,
        "source": source_kind,
        "source_detail": source_detail,
    }
    if source_detail == "local":
        rust_project_owned.append(record)
    else:
        for license_id in licenses:
            rust_third_party[license_id].append(record)

for records in rust_third_party.values():
    records.sort(key=lambda item: (item["name"].casefold(), item["name"], item["version"], item["source"]))
rust_project_owned.sort(key=lambda item: (item["name"].casefold(), item["name"], item["version"]))

package_versions = {}
package_licenses = {}
for relative in ("packages/solid-gpui/package.json", "packages/solid-gpui-router/package.json", "packages/solid-gpui-shiki/package.json"):
    package_json = json.loads((root / relative).read_text(encoding="utf-8"))
    package_versions[package_json["name"]] = package_json["version"]
    package_licenses[package_json["name"]] = package_json["license"]

bun_group = re.compile(r"^(?P<license>.+) \(\d+\)$")
bun_entry = re.compile(r"^[├└]── (?P<package>.+)$")

def parse_bun(path, package_label):
    grouped = defaultdict(list)
    current_license = None
    for raw_line in pathlib.Path(path).read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or re.fullmatch(r"bun pm licenses v[^ ]+ \([0-9a-f]+\)", line):
            continue
        if re.fullmatch(r"\d+ packages across \d+ licenses \(checked \d+ packages in bun\.lock\) \[[0-9.]+ms\]", line):
            continue
        group_match = bun_group.match(line)
        if group_match:
            current_license = group_match.group("license")
            continue
        entry_match = bun_entry.match(line)
        if entry_match is None:
            raise SystemExit(f"unrecognized Bun license output line in {path}: {raw_line!r}")
        if current_license is None:
            raise SystemExit(f"Bun package appeared before a license group in {path}")
        token = entry_match.group("package")
        is_dev = token.endswith(" (dev)")
        if is_dev:
            token = token[: -len(" (dev)")]
        try:
            name, version = token.rsplit("@", 1)
        except ValueError as exc:
            raise SystemExit(f"invalid Bun package reference: {token!r}") from exc
        if version.startswith(("../", "./")):
            source = "local (project-owned)"
            version = package_versions.get(name, version)
        else:
            source = "registry (npm)"
        grouped[current_license].append(
            {
                "package": package_label,
                "name": name,
                "version": version,
                "license": current_license,
                "source": source,
                "scope": "dev" if is_dev else "runtime",
            }
        )
    for records in grouped.values():
        records.sort(key=lambda item: (item["name"].casefold(), item["name"], item["version"], item["scope"]))
    return grouped

bun_inventories = {
    "Solid GPUI workspace": parse_bun(bun_workspace_path, "Solid GPUI workspace"),
}


def markdown_cell(value):
    return str(value).replace("|", "\\|").replace("\n", " ")


def rust_table(records):
    lines = [
        "| Crate | Version | License (SPDX) | Source |",
        "| --- | --- | --- | --- |",
    ]
    for record in records:
        lines.append(
            "| "
            + " | ".join(
                markdown_cell(value)
                for value in (record["name"], record["version"], record["license"], record["source"])
            )
            + " |"
        )
    return lines


def bun_table(records):
    lines = [
        "| Package | Version | License (SPDX) | Source | Scope |",
        "| --- | --- | --- | --- | --- |",
    ]
    for record in records:
        lines.append(
            "| "
            + " | ".join(
                markdown_cell(value)
                for value in (
                    record["name"],
                    record["version"],
                    record["license"],
                    record["source"],
                    record["scope"],
                )
            )
            + " |"
        )
    return lines

host_package = next((p for p in packages if p.get("name") == "solid-gpui"), None)
if host_package is None:
    raise SystemExit("cargo metadata did not contain solid-gpui")
host_archive = f"solid-gpui-host-{host_package['version']}-{target}.tar.gz"
project_license = host_package["license"]

third_party_rust_packages = {
    (record["name"], record["version"])
    for records in rust_third_party.values()
    for record in records
}

lines = [
    "# Third-Party Notices",
    "",
    f"This inventory accompanies the host release archive `{host_archive}` and the companion Bun packages from this checkout.",
    "It records each resolved dependency's name, version, SPDX license identifier, and source provenance for the generated release artifacts.",
    "The inventory is generated from the resolved Cargo graph with all workspace features and `bun pm licenses --all` output; it is not a substitute for the license texts.",
    f"The archive embeds the project-owned {project_license} text as `LICENSE`. The independently authored local `ztracing` stub and each npm package carry their own `LICENSE`.",
    "Full third-party license texts are intentionally not copied into this inventory; they remain available from the referenced registry or git source. This keeps the artifact an inventory rather than a large license-text bundle.",
    "",
    f"**Generated:** {generated_date}",
    "**Generation command:** `bun run task third-party-notices`",
    "",
    "The host archive keeps this single inventory next to `LICENSE`. The npm tarball remains lean and carries only its own package `LICENSE`; the JavaScript dependency inventory stays in the repository and release archive.",
    "License groups below repeat a package when its declared expression contains multiple SPDX identifiers. Totals count package records within that group.",
    "",
    "## Rust dependencies",
    "",
    f"The current Cargo inventory contains {len(third_party_rust_packages)} third-party packages and {sum(len(records) for records in rust_third_party.values())} package-license records; local workspace records are listed separately below.",
    "",
]

for license_id in sorted(rust_third_party, key=lambda value: (value.casefold(), value)):
    records = [dict(record, license=license_id) for record in rust_third_party[license_id]]
    lines.extend([f"### {license_id}", "", *rust_table(records), "", f"**Total {license_id}: {len(records)} package records.**", ""])

lines.extend(
    [
        "## Project-owned Rust crates",
        "",
        "These local Cargo records are project-owned rather than third-party dependencies. The `ztracing` row is the independently authored stub used by the host graph.",
        "",
        *rust_table([dict(record, license=record["licenses"][0]) for record in rust_project_owned]),
        "",
        f"**Total project-owned Rust crates: {len(rust_project_owned)}.**",
        "",
        "## Bun dependencies",
        "",
        "This inventory is the direct output of `bun pm licenses --all` for the root Bun workspace. Entries marked `dev` are development-only.",
        "",
    ]
)

for package_label in ("Solid GPUI workspace",):
    grouped = bun_inventories[package_label]
    lines.extend([f"### `{package_label}`", ""])
    for license_id in sorted(grouped, key=lambda value: (value.casefold(), value)):
        records = grouped[license_id]
        lines.extend([f"#### {license_id}", "", *bun_table(records), "", f"**Total {license_id} ({package_label}): {len(records)} package records.**", ""])

lines.extend(
    [
        "## Bundled fonts",
        "",
        "Maple Mono v7.9 (standard TTF, without CN or Nerd Font additions) is distributed under SIL OFL 1.1. Regular, italic, bold and bold italic are bundled in `crates/solid-gpui/fonts`; the complete license is in `MapleMono-OFL.txt`.",
        "Source: https://github.com/subframe7536/maple-font/releases/tag/v7.9",
        "",
        "## Vendored JavaScript",
        "",
        "`packages/solid-gpui/src/vite/quickjs-abort.ts` adapts the AbortController/AbortSignal implementation from Vercel's `@edge-runtime/primitives` 6.0.0 under MIT (copyright 2024 Vercel, Inc.).",
        "Source: https://github.com/vercel/edge-runtime/blob/440c123a37284d6a852ce453af810ad484ecfc01/packages/primitives/src/primitives/abort-controller.js",
        "The source file retains the complete MIT notice and describes local changes. It is shipped with the core package's build tools and included in QuickJS application bundles.",
        "",
        "`packages/solid-gpui/src/vite/quickjs-headers.js` adapts `fetch-headers` 3.0.1 under MIT (copyright 2021 Jimmy Wärting).",
        "Source: https://github.com/jimmywarting/fetch-headers/blob/66d63ac7a67d3b9c863cd80b872f2ea999cbc7a7/headers.js",
        "The source retains the complete MIT notice. Local changes correct header validation/normalization, preserve values after rejected mutations, and remove Node inspection support.",
        "",
        "## Project-owned Bun packages",
        "",
        "These package manifests declare their project-owned licenses and are not third-party dependencies. Their npm tarballs include their own `LICENSE`; the shared JavaScript dependency inventory remains in this release artifact.",
        "",
        "| Package | Version | License (SPDX) | Source |",
        "| --- | --- | --- | --- |",
    ]
)
for name in sorted(package_versions, key=lambda value: (value.casefold(), value)):
    lines.append(f"| {name} | {package_versions[name]} | {package_licenses[name]} | local (project-owned) |")
lines.extend(["", f"**Total project-owned Bun packages: {len(package_versions)}.**", ""])

content = "\n".join(lines)
if not content.endswith("\n"):
    content += "\n"

output.parent.mkdir(parents=True, exist_ok=True)
with tempfile.NamedTemporaryFile("w", encoding="utf-8", dir=output.parent, prefix=f".{output.name}.", delete=False) as handle:
    handle.write(content)
    temporary = pathlib.Path(handle.name)
os.replace(temporary, output)

bun_records = sum(len(records) for grouped in bun_inventories.values() for records in grouped.values())
print(f"third-party notices: {len(third_party_rust_packages)} Rust third-party packages, {bun_records} Bun dependency records -> {output}")
for license_id in sorted(rust_third_party, key=lambda value: (value.casefold(), value)):
    print(f"Rust {license_id}: {len(rust_third_party[license_id])}")
for package_label, grouped in bun_inventories.items():
    for license_id in sorted(grouped, key=lambda value: (value.casefold(), value)):
        print(f"Bun {package_label} {license_id}: {len(grouped[license_id])}")
PY
