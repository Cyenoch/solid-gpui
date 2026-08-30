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

## Engineering feasibility (2026-08-31)

The read-only feasibility probe is recorded in
[`../feasibility.md`](../feasibility.md). Status remains `ready-for-human`;
this section does not make a legal decision or remove the STOP.

### Probe A: no feature-off path

At pinned Zed revision `6805d952f9f3d702f760aa11b1547df8a625fa16`,
`crates/gpui/Cargo.toml:61` (`gpui_shared_string.workspace = true`), `:96`
(`gpui_util.workspace = true`), and `:107` (`ztracing.workspace = true`) are
ordinary entries in `[dependencies]`, not optional or target-gated entries.
The host's `default = []` and a target-qualified `--no-default-features`
feature-tree query still retain `ztracing` through `gpui` and `sum_tree`, plus
`zlog` and `ztracing_macro` through `ztracing`. The two no-license-field crates
also remain ordinary GPUI dependencies. `ZTRACING` only selects the crate's
no-op instrumentation implementation; it does not remove any package from the
resolved graph. There is therefore no near-zero "feature off today" clearance.

### Probe B: no upstream pin-bump clearance

Vendored `origin/main` at `1662f5f3f6497c5f80830ccdca1edfd1fc0c6c6a` still has
ordinary `gpui_shared_string`, `gpui_util`, and `ztracing` dependencies in
GPUI, and still declares GPL-3.0-or-later for `zlog`, `ztracing`, and
`ztracing_macro`. Both `gpui_shared_string` and `gpui_util` still lack a
manifest `license` field; the current GPUI SVG attributes remain. The direct
GPUI edge and `parse_svg`/`render_parsed` instrumentation entered in
`00cba838ad4e0be4b6176438551b72b2d512e9f8` (2026-08-05), but the GPL chain was
already reachable through `sum_tree` after the December 2025 tracing commits
(`b558be7ec60b265837e34d6f9b6f0ef176c20082` and
`1029a8fbaf5271b6eb3e4e51f9e5cb015c52f760`). A bump to observed main clears
none of the five findings. Any future fixed upstream release requires the
full GPUI/platform graph review already scoped by the drift assessment, not a
version-only edit.

### Decision matrix

| Option | Engineering result | GPL trio | Two no-license crates | Residual work |
| --- | --- | --- | --- | --- |
| Feature off today | No Cargo switch; `ZTRACING` no-op still resolves packages | Not cleared | Not cleared | Legal review, `self_cell`, weak-copyleft and notices unchanged |
| Pin bump clears it | Not available at observed main; future fix needs full graph/API/lock/artifact review | Not cleared now | Not cleared now | Re-audit exact targets and artifacts if upstream later fixes it |
| Patch-fork ztracing out | Feasible only by forking and editing both GPUI and `sum_tree`; remove their ztracing deps/imports/attributes, patch both packages, and maintain the fork | Conditionally cleared after target-qualified graph/license proof | Not cleared | Fork maintenance, rebasing/security updates, retained upstream notices, and legal treatment of fork modifications |
| Accept GPL terms for binaries | No engineering change | Remains present; legal acceptance only | Not cleared | Written determination plus required notices/source/corresponding-source or relinkable-object materials |

For the fork option, patching GPUI alone is insufficient because
`sum_tree -> ztracing -> zlog,ztracing_macro` remains active. The exact source
changes and conceptual Cargo patch are listed in `feasibility.md`. The
fork-modification license is legal-adjacent and is not inferred here.

Engineering recommendation: do not bump solely for this blocker. Preserve
upstream and present the GPL-acceptance route to legal first; if legal rejects
GPL distribution, evaluate the maintained two-crate fork as the technical last
resort. The two missing license fields, `self_cell`'s dual expression, and
other weak-copyleft findings remain independent obligations under every option.

## Comments

- 2026-08-31: Created from the evidence-backed license audit. No dependency,
  Makefile, `deny.toml`, or publication command was changed.
- 2026-08-31: Option 3 fork proof: [`../spike-option3.md`](../spike-option3.md) — proven narrowly; all three GPL package IDs gone in the target no-dev graph, host check/release/smoke passed, but standalone translation and ongoing maintenance are medium/high cost.
- The proof does not clear `gpui_shared_string`/`gpui_util` missing license fields or make a legal distribution decision; the STOP and release-owner checklist remain active.
