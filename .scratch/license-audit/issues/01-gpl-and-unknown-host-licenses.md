# Host distribution license blocker

Status: ready-for-human  
Type: legal-release blocker  
Label: `ready-for-human`  
Severity: blocker  

## Decision required

**STOP.** A release owner and legal reviewer must decide whether the current
license set may be distributed before any push, registry publication, signing,
notarization, or other external distribution. This is a legal decision, not an
engineering waiver. Do not resolve this ticket by silently allow-listing the
findings or by changing dependencies without a separate approved plan.

## Evidence

At repository HEAD `c329b63d38b3c444d37b3dcfb2d37a024b43eca5`, the default
`react-gpui-host` build graph reaches these pinned Zed crates:

- `zlog 0.1.0` — `GPL-3.0-or-later`
- `ztracing 0.1.0` — `GPL-3.0-or-later`
- `ztracing_macro 0.1.0` — `GPL-3.0-or-later`
- `gpui_shared_string 0.1.0` — no `license` field in its manifest
- `gpui_util 0.1.0` — no `license` field in its manifest

The pinned Zed manifest at revision
`6805d952f9f3d702f760aa11b1547df8a625fa16` declares `zlog`, `ztracing`, and
`ztracing_macro` as GPL-3.0-or-later. `ztracing` has ordinary dependencies on
`zlog` and `ztracing_macro`; its platform-specific dependencies do not gate
those edges away for native builds. GPUI itself declares `Apache-2.0`, but that
does not change the licenses of its dependencies.

The following target-qualified, no-dev-dependency traces prove reachability to
the host's normal default graph:

```text
cargo tree -p react-gpui-host --target aarch64-apple-darwin --no-dev-dependencies -i zlog
zlog -> ztracing -> gpui -> gpui_apple [build-dependencies] -> gpui_macos
  -> gpui_platform -> react-gpui-host
zlog -> ztracing -> gpui -> react-gpui -> react-gpui-host

cargo tree -p react-gpui-host --target aarch64-apple-darwin --no-dev-dependencies -i ztracing
ztracing -> gpui -> gpui_apple [build-dependencies] -> gpui_macos
  -> gpui_platform -> react-gpui-host
ztracing -> gpui -> react-gpui -> react-gpui-host

cargo tree -p react-gpui-host --target aarch64-apple-darwin --no-dev-dependencies -i ztracing_macro
ztracing_macro -> ztracing -> gpui -> gpui_apple [build-dependencies]
  -> gpui_macos -> gpui_platform -> react-gpui-host
ztracing_macro -> ztracing -> gpui -> react-gpui -> react-gpui-host
```

The Linux default graph also reaches `self_cell 1.3.0` through
`cosmic-text -> gpui_wgpu -> gpui_linux -> gpui_platform -> react-gpui-host`.
Its registry manifest is `Apache-2.0 OR GPL-2.0-only`, with both
`LICENSE-APACHE` and `LICENSE-GPLv2` present. The Apache option must be
verified and selected for distribution by the legal reviewer; it is not a
GPL finding by itself.

Other recognized weak-copyleft entries are present in the dependency graph:
`cbindgen`, `dwrote`, and `option-ext` are `MPL-2.0`; `r-efi 5.3.0` and
`r-efi 6.0.0` declare `Apache-2.0 OR LGPL-2.1-or-later OR MIT`. Their
file-scope/linking and notice/source obligations need a legal review if the
host artifacts include them. They are not substitutes for resolving the GPL
and unknown-license blocker.

## Required release-owner checklist

- [ ] Obtain a written legal determination for the GPL-3.0-or-later crates and
  the two missing manifest license fields.
- [ ] Confirm whether the legal determination selects a permitted upstream
  license option (including `self_cell`'s Apache option) and records any
  required notices, source offers, or corresponding-source obligations.
- [ ] Re-run the complete license inventory against the exact source revision,
  host binary/archive, and each npm tarball that will be distributed.
- [ ] Do not remove the STOP warning until the determination and artifact-level
  evidence are attached to this issue.

## Comments

- 2026-08-31: Created from the evidence-backed license audit. No dependency,
  Makefile, `deny.toml`, or publication command was changed.
