# `paste` unmaintained advisory

Status: needs-triage  
Type: supply-chain advisory

- Advisory: [RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436)
- Affected version: `paste 1.0.15`
- Locked source: crates.io, through the pinned GPUI/Zed graph
- Scanner: `cargo deny check advisories` (`cargo-deny 0.20.2`)
- Solution: cargo-deny reports no safe upgrade; suggested alternatives are `pastey` and `with_builtin_macros`.

## Exposure

`paste` is not a direct workspace dependency. It is pulled in by GPUI's Apple/native dependency graph through `metal`, `core-video`, and related GPUI crates. It is a compile-time macro crate, not a runtime parser for renderer input. The advisory is an unmaintained-package signal rather than a reported exploitable vulnerability in this application. The dependency remains part of the upstream pinned graph and cannot be safely changed locally without replacing an upstream build dependency.

## Upgrade path

Do not add an unreviewed fork or force an alternative. Triage with the GPUI/Zed upstream revision and adopt an upstream-supported migration or an explicitly reviewed patch once available. Re-run the full Rust and embedded gates after any graph change.
