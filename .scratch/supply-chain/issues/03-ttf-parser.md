# `ttf-parser` unmaintained advisory

Status: direct-dep exposure eliminated; transitive occurrences remain
Type: supply-chain advisory

- Advisory: [RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192)
- Affected version: `ttf-parser 0.25.1`
- Locked source: crates.io; `ttf-parser 0.25.1` remains in the pinned GPUI graph, while the direct declaration in `crates/react-gpui/Cargo.toml` has been replaced
- Scanner: `cargo deny check advisories` (`cargo-deny 0.20.2`)
- Solution: cargo-deny reports no safe upgrade; it names `skrifa` as an actively maintained alternative.

## Exposure

`ttf-parser` was previously a direct dependency of `react-gpui`; `crates/react-gpui/src/renderer/commands.rs` called `ttf_parser::Face::parse` in `font_family()` while handling `COMMAND_LOAD_FONT`. The command still reads a user-provided font path, bounds the file to `MAX_FILE_READ_BYTES`, requires a regular file, and parses the bytes before registering the font, but the application-owned parse now uses maintained `skrifa::FontRef::from_index(bytes, 0)` and its localized name APIs. The direct user-byte parser exposure is eliminated from this repository-owned crate.

The same locked `ttf-parser 0.25.1` remains transitively present through pinned GPUI's `fontdb -> usvg` and `rustybuzz -> usvg` paths. Those occurrences are upstream-owned (`references/zed`) and cannot be safely removed by changing this crate's direct dependency. The advisory remains ignored in `deny.toml` for that transitive graph.

## Upgrade path

The reviewed migration is complete for the direct application-owned usage. Keep the issue open for the remaining upstream-owned transitive occurrences; remove the `deny.toml` ignore only after the pinned GPUI/Zed graph no longer contains the advisory crate. Existing path and size validation remains in place. Full Rust and embedded gates are required after this parser migration.

## Answer

**Verdict: migrate the direct usage; keep the advisory open for transitive upstream occurrences.** Cached `skrifa 0.44.0` exposes the exact required API: `FontRef::from_index(bytes, 0)` validates single fonts and collections while returning an error for malformed input, and `MetadataProvider::localized_strings` accepts `StringId::TYPOGRAPHIC_FAMILY_NAME` then `StringId::FAMILY_NAME`. `LocalizedString` has `Display`/character iteration. Its declared Rust 1.85 MSRV is compatible with the workspace's Rust 1.97.1 toolchain.

`react-gpui` now depends on `skrifa` with `default-features = false, features = ["std"]`; no compatibility shim remains. The existing Tuffy fixture assertion remains exactly `"Tuffy"`, and a garbage-byte test covers parser rejection without panic. The lockfile edge changed from direct `ttf-parser` to `skrifa 0.44.0`; `skrifa` and `read-fonts` were already present through GPUI's `swash` stack, so no new package/version entries were added.

Post-migration graph evidence:

- `cargo tree -i skrifa@0.44.0 --workspace --locked --offline`: `skrifa -> react-gpui` (and its host/Bun consumers), with the same version already used by `swash` in the GPUI stack.
- `cargo tree -i ttf-parser --workspace --locked --offline`: only `fontdb -> usvg -> gpui` and `rustybuzz -> usvg` upstream paths remain; no `react-gpui` direct edge.

Resolved application-owned layer: `loadFont` user-byte parsing moved to maintained `skrifa`. Open upstream layer: transitive `ttf-parser` remains in the lockfile, so this issue is not closed and the deny ignore stays.

## Comments

- 2026-08-30: Migration spike completed; see `.scratch/skrifa-spike/notes.md` for API evidence, graph delta, source-cache size measurements, and gate results.
- 2026-08-30: Serial verification passed: `cargo test -p react-gpui --lib --locked --offline` (135 tests), `cargo test -p react-gpui --lib renderer::commands::tests::font_family --locked --offline` (2 tests), `make ci` (including `cargo deny check advisories`, `advisories ok`), `make embedded-bun` (2 embedded host tests + 1 adapter test), `cargo fmt --all -- --check`, and `make bun-format`. Direct `react-gpui` exposure is resolved; transitive GPUI-owned occurrences keep this issue open and the deny ignore retained.
