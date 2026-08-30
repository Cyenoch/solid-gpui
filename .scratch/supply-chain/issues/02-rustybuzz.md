# `rustybuzz` unmaintained advisory

Status: needs-triage  
Type: supply-chain advisory

- Advisory: [RUSTSEC-2026-0206](https://rustsec.org/advisories/RUSTSEC-2026-0206)
- Affected version: `rustybuzz 0.20.1`
- Locked source: crates.io, through `usvg`/`resvg` in the pinned GPUI/Zed graph
- Scanner: `cargo deny check advisories` (`cargo-deny 0.20.2`)
- Solution: cargo-deny reports no safe upgrade; it names `harfrust` as an actively maintained alternative.

## Exposure

`rustybuzz` is not a direct workspace dependency. It is pulled into GPUI's SVG/font shaping path through `usvg` and `resvg`. Exposure is therefore in the upstream GPUI rendering path, not a direct application dependency or a package we can safely bump independently. The advisory is an unmaintained-package signal; cargo-deny reports no specific vulnerability or compatible patch release.

## Upgrade path

Do not force a breaking or graph-local replacement. Coordinate an update of the pinned GPUI/Zed revision that adopts a maintained shaping dependency, or conduct a separately reviewed upstream migration to `harfrust`. Re-run the full Rust and embedded gates after any graph change.
