# Skrifa migration spike

Date: 2026-08-30

## Usage inventory

The only application-owned use of `ttf-parser` was `crates/react-gpui/src/renderer/commands.rs:45-67`, in `font_family()`. `COMMAND_LOAD_FONT` reads a bounded user-provided font file, calls this helper before registration at `commands.rs:233`, then passes the original bytes to GPUI `TextSystem::add_fonts` at `commands.rs:239-249`. No other `ttf-parser` callsite exists in the workspace-owned `crates/` or `packages/` code. The pinned GPUI source has independent `ttf-parser` uses in `references/zed/crates/gpui/src/svg_renderer.rs:44-47` and declares its own dependency at `references/zed/crates/gpui/Cargo.toml:81`; those are not our parser implementation and were not changed.

## API evidence and fit

Cached `skrifa 0.44.0` source was reviewed at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/skrifa-0.44.0/`:

- `src/lib.rs:64-70` re-exports `read_fonts::FontRef` and `MetadataProvider`.
- `read-fonts 0.41.0/src/lib.rs:340-376` documents `FontRef::from_index(data, index)`, accepting a single TTF/OTF or TTC/OTC data and rejecting a nonzero index for a single face. `from_index(bytes, 0)` therefore matches the existing first-face contract and returns `Result` for malformed data without an unwrap.
- `skrifa/src/provider.rs:16-31` defines `MetadataProvider::localized_strings`.
- `skrifa/src/string.rs:33-35` aliases `StringId` to `read_fonts::types::NameId`; `font-types 0.12.4/src/name_id.rs:28-32` and `:97-103` provide `FAMILY_NAME` (name ID 1) and `TYPOGRAPHIC_FAMILY_NAME` (name ID 16).
- `skrifa/src/string.rs:50-113` provides the localized-string iterator; `:74-99` provides `english_or_first`, and `:126-160` provides character iteration and `Display`. The implementation retains the old behavior's first usable typographic-family record, then first usable family record, while applying the existing non-empty, 64-character, and no-control-character checks.
- `skrifa/README.md:7-26` lists robust OpenType reading and localized strings; `:49-52` states corrupted/malicious font files should not panic; `:61-62` forbids unsafe code.

The API is an equal-complexity clean cutover for this narrow usage: one validated constructor plus two name iterators replaces one face constructor plus one names iterator. It also keeps first-face collection handling. `skrifa 0.44.0` declares `rust-version = 1.85`, compatible with this workspace's Rust 1.97.1 toolchain; its crate edition is 2021 and is usable by this workspace's edition 2024 crates. The dependency is configured with `default-features = false, features = ["std"]`, avoiding the optional autohint-shaping feature for metadata-only parsing.

## Decision

**Migrate.** `skrifa` covers the exact loadFont family-extraction contract with no compatibility layer. Existing path regular-file and size limits remain unchanged. The `font_family()` implementation now uses `skrifa::FontRef::from_index(bytes, 0)` and `MetadataProvider::localized_strings`.

## Dependency graph delta

Before migration, `react-gpui` directly depended on `ttf-parser 0.25.1`; the lockfile also contained that same version through pinned GPUI's `fontdb -> usvg` path and through `rustybuzz`. The lockfile already contained `skrifa 0.44.0` and `read-fonts 0.41.0` through `swash` (used by GPUI's cosmic-text stack), so the migration adds no new package/version entries: it changes the `react-gpui` lock dependency edge from `ttf-parser` to `skrifa 0.44.0`.

Post-migration `cargo tree -i skrifa@0.44.0 --workspace --locked --offline` shows the direct application path:

```text
skrifa v0.44.0
└── react-gpui v0.2.0
    ├── react-gpui-bun v0.2.0
    └── react-gpui-host v0.2.0
```

`skrifa` is therefore shared with GPUI's existing `swash` stack rather than introducing a second skrifa version. `cargo tree -i ttf-parser --workspace --locked --offline` still shows only upstream-owned paths: `fontdb -> usvg -> gpui` (including GPUI platform crates) and `rustybuzz -> usvg`; the workspace-owned `react-gpui` edge is gone. The cached source sizes are approximately 1.6 MiB for skrifa 0.44.0, 4.0 MiB for read-fonts 0.41.0, and 1.3 MiB for ttf-parser 0.25.1; these are source-cache sizes, not compiled artifact sizes. The existing GPUI graph already pays for skrifa/read-fonts, so no duplicate parser stack was introduced.

## Tests and gates

Added two focused unit tests beside `font_family()`: the existing Tuffy fixture must return exactly `"Tuffy"`, and garbage bytes must return the existing `"font is malformed or unsupported"` error without panic. The serial focused run `cargo test -p react-gpui --lib renderer::commands::tests::font_family --locked --offline` passed 2 tests; the serial full run `cargo test -p react-gpui --lib --locked --offline` passed 135 tests.

All required post-migration gates passed serially: `make ci` exited 0 (Rust format/check/Clippy/workspace tests, both Bun formatter/typecheck/test/build/package-smoke stages, both Bun audits, and `cargo deny check advisories` reported `advisories ok`); `make embedded-bun` exited 0 (2 embedded host example tests and 1 embedded adapter counter test passed); `cargo fmt --all -- --check` exited 0; and `make bun-format` exited 0 with both package Prettier checks clean. The known `block 0.1.6` future-incompatibility warning remains unrelated to this migration.

## Layered advisory status

The migration eliminates the direct `react-gpui` parser edge and moves `loadFont`'s user-provided byte parsing to maintained `skrifa`. It does **not** close RUSTSEC-2026-0192 globally: `ttf-parser 0.25.1` remains in the lockfile through the pinned GPUI/fontdb/rustybuzz graph, which is upstream-owned and outside this clean cutover. `deny.toml` therefore retains the advisory ignore. The issue remains open for the transitive occurrences, with direct-dependency exposure eliminated as the resolved application-owned layer.
