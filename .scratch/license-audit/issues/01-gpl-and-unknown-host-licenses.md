# Host distribution license blocker

Status: ready-for-human (Option 3′ EXECUTED; STOP narrowed; terms determinable, metadata missing)
Type: legal-release blocker  
Label: `ready-for-human`  
Severity: blocker  

## Decision required

**STOP (narrowed).** Option 3′ now removes `zlog`, `ztracing`, and
`ztracing_macro` from the resolved `react-gpui-host` graph. A release owner
and legal reviewer must still confirm that the explicit Apache-2.0 license
files for `gpui_shared_string` and `gpui_util` govern the exact versions and
distributed artifacts, complete the missing Cargo metadata path, and confirm
`self_cell`'s Apache option before any push, registry publication, signing,
notarization, or other external distribution. This remains a legal and
artifact decision, not an engineering waiver. The terms-determinability
finding is downgraded; the publication STOP remains.
See the execution record in [`../landing-option3.md`](../landing-option3.md).


## Option 3′ execution status (2026-08-31)

**EXECUTED.** The conservative Option 3′ local patch is landed in the main
tree: an independently authored Apache-2.0 `ztracing` proc-macro stub replaces
the upstream package for the host graph. It exports the identity `instrument`
attribute, removes the GPL package records from the lockfile, and preserves
the upstream documented no-op instrumentation behavior. It distributes no Zed
tracing implementation code and does not select Option 4.

**STOP (narrowed).** The remaining release review covers the identified
Apache-2.0 terms for `gpui_shared_string` and `gpui_util` (their manifest
license metadata is still missing), plus confirmation that this marking governs
the exact distribution artifacts and the Apache option for `self_cell`'s dual
expression. The issue remains `ready-for-human`; this engineering change is
not legal review and does not authorize publication. Weak-copyleft, notice,
and artifact obligations remain review items as documented.
## Evidence

### Pre-Option 3′ evidence

At repository HEAD `c329b63d38b3c444d37b3dcfb2d37a024b43eca5`, the default
`react-gpui-host` build graph reaches these pinned Zed crates:

- `zlog 0.1.0` — `GPL-3.0-or-later`
- `ztracing 0.1.0` — `GPL-3.0-or-later`
- `ztracing_macro 0.1.0` — `GPL-3.0-or-later`
- `gpui_shared_string 0.1.0` — no manifest `license` field; its per-crate `LICENSE-APACHE` symlink identifies Apache-2.0 text
- `gpui_util 0.1.0` — no manifest `license` field; its per-crate `LICENSE-APACHE` symlink identifies Apache-2.0 text

The pinned Zed manifest at revision
`6805d952f9f3d702f760aa11b1547df8a625fa16` declares `zlog`, `ztracing`, and
`ztracing_macro` as GPL-3.0-or-later. `ztracing` has ordinary dependencies on
`zlog` and `ztracing_macro`; its platform-specific dependencies do not gate
those edges away for native builds. GPUI itself declares `Apache-2.0`, but that
does not change the licenses of its dependencies.

The two no-metadata crates each carry `LICENSE-APACHE`, a symlink to the Zed
root `LICENSE-APACHE`. Its exact text identifies the `Apache License, Version
2.0` and grants rights to distribute the work in source or object form. The
control crate `gpui` carries the same symlink and identical file contents.
This makes the terms determinable while leaving the Cargo metadata finding and
human/legal artifact review in place.

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
host artifacts include them. They are not substitutes for completing the
metadata, legal, and artifact review for the two Apache-marked crates.

## Required release-owner checklist

- [ ] Obtain written legal/rights-holder confirmation that the Apache-2.0 terms
  identified by the two per-crate `LICENSE-APACHE` files govern the exact
  versions and artifacts, and record any required notices or other obligations.
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
`zlog` and `ztracing_macro` through `ztracing`. The two crates also remain
ordinary GPUI dependencies; their per-crate `LICENSE-APACHE` files identify
Apache-2.0 text, but the Cargo manifest fields remain absent. There is
therefore no near-zero "feature off today" clearance for the metadata finding.

### Probe B: no upstream pin-bump clearance

Vendored fetched `origin/main` at `1662f5f3f6497c5f80830ccdca1edfd1fc0c6c6a`
still has ordinary `gpui_shared_string`, `gpui_util`, and `ztracing`
dependencies in GPUI, and still declares GPL-3.0-or-later for `zlog`,
`ztracing`, and `ztracing_macro`. Both target manifests still lack a
`license` field even though their per-crate `LICENSE-APACHE` files identify
Apache-2.0 text; the current GPUI SVG attributes remain. The direct GPUI edge
and `parse_svg`/`render_parsed` instrumentation entered in
`00cba838ad4e0be4b6176438551b72b2d512e9f8` (2026-08-05), but the GPL chain was
already reachable through `sum_tree` after the December 2025 tracing commits
(`b558be7ec60b265837e34d6f9b6f0ef176c20082` and
`1029a8fbaf5271b6eb3e4e51f9e5cb015c52f760`). A bump to observed main clears
none of the GPL findings and does not repair the metadata. Any future fixed
upstream release requires the full GPUI/platform graph review already scoped
by the drift assessment, not a version-only edit.

### Decision matrix

| Option | Engineering result | GPL trio | Two crates with missing metadata | Residual work |
| --- | --- | --- | --- | --- |
| Feature off today | No Cargo switch; `ZTRACING` no-op still resolves packages | Not cleared | Metadata not cleared; terms identified | Legal review, `self_cell`, weak-copyleft and notices unchanged |
| Pin bump clears it | Not available at observed main; future fix needs full graph/API/lock/artifact review | Not cleared now | Metadata not cleared now; terms identified | Re-audit exact targets and artifacts if upstream later fixes it |
| Patch-fork ztracing out | Feasible only by forking and editing both GPUI and `sum_tree`; remove their ztracing deps/imports/attributes, patch both packages, and maintain the fork | Conditionally cleared after target-qualified graph/license proof | Metadata not cleared; terms identified | Fork maintenance, rebasing/security updates, retained upstream notices, and legal treatment of fork modifications |
| Accept GPL terms for binaries | No engineering change. The local Option 3′ graph has removed the GPL trio; upstream terms remain relevant to any unpatched upstream use. | Remains present upstream; local graph cleared | Metadata not cleared; terms identified | Written confirmation of Apache marking plus required notices/source/corresponding-source or relinkable-object materials |

For the fork option, patching GPUI alone is insufficient because
`sum_tree -> ztracing -> zlog,ztracing_macro` remains active. The exact source
changes and conceptual Cargo patch are listed in `feasibility.md`. The
fork-modification license is legal-adjacent and is not inferred here.

Engineering recommendation: do not bump solely for this blocker. Preserve
upstream and present the GPL-acceptance route to legal first; if legal rejects
GPL distribution, evaluate the maintained two-crate fork as the technical last
resort. The two missing manifest license fields (with Apache-2.0 terms
identified by their per-crate files), `self_cell`'s dual expression, and other
weak-copyleft findings remain independent obligations under every option.

## Comments

- 2026-08-31: Created from the evidence-backed license audit. No dependency,
  Makefile, `deny.toml`, or publication command was changed.
- 2026-08-31: Option 3 fork proof: [`../spike-option3.md`](../spike-option3.md) — proven narrowly; all three GPL package IDs gone in the target no-dev graph, host check/release/smoke passed, but standalone translation and ongoing maintenance are medium/high cost.
- The proof does not clear the two missing manifest license fields (their terms are identified by explicit per-crate Apache-2.0 files) or make a legal distribution decision; the STOP and release-owner checklist remain active.
- 2026-08-31: Option 3′ stub patch proof: [`../spike-option3-stub.md`](../spike-option3-stub.md) — local Apache-2.0 identity proc-macro patch removes the three GPL package records from the target host graph; host check/release/version/test gates passed.
- Engineering verdict: the stub is the smaller Option 3 variant, but the separate missing metadata for `gpui_shared_string`/`gpui_util` and legal STOP remain active; the decision is still for the release owner and legal reviewer.
- 2026-08-31: Option 3′ executed in the main tree; the GPL trio is absent from all three target-qualified host graphs and the STOP is narrowed to Apache-marked crates whose manifest metadata is missing, plus `self_cell` option confirmation.
- 2026-08-31: Terms-determinability recheck recorded in [`../recheck-2026-08-31.md`](../recheck-2026-08-31.md): both target crates carry explicit Apache-2.0 license files; the blocker is metadata/legal/artifact confirmation, not unknowable terms.
