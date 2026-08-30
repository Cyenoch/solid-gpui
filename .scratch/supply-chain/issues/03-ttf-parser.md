# `ttf-parser` unmaintained advisory

Status: needs-triage  
Type: supply-chain advisory

- Advisory: [RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192)
- Affected version: `ttf-parser 0.25.1`
- Locked source: crates.io; direct declaration in `crates/react-gpui/Cargo.toml` and also present through the GPUI graph
- Scanner: `cargo deny check advisories` (`cargo-deny 0.20.2`)
- Solution: cargo-deny reports no safe upgrade; it names `skrifa` as an actively maintained alternative.

## Exposure

Unlike the other two findings, `ttf-parser` is a direct dependency of `react-gpui`. `crates/react-gpui/src/renderer/commands.rs` calls `ttf_parser::Face::parse` in `font_family()` while handling `COMMAND_LOAD_FONT`. The command reads a user-provided font path, bounds the file to `MAX_FILE_READ_BYTES`, requires a regular file, then parses the bytes before registering the font. This makes parsing user font files a real attack surface for future parser defects, although this advisory reports unmaintained status rather than a concrete vulnerability in the locked release.

## Upgrade path

Do not force a breaking parser replacement in this audit. Evaluate an explicitly reviewed migration to `skrifa` (including API/format coverage and the GPUI registration contract), or adopt a compatible maintained release if one becomes available. Until then, retain the existing size/path validation and track the direct parser dependency for upstream maintenance. Re-run the full Rust and embedded gates after any parser migration.
