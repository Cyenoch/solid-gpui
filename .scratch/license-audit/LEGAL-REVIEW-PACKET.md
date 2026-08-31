# Legal Review Packet — v0.2.0

**Date:** 2026-08-31  
**Review basis:** repository HEAD `65a4c198c5773925463c1cf3d54f71e5ae80d177`  
**Status:** **Prepared by tooling; not legal advice; reviewer confirms or corrects.**

This packet is the reading-order consolidation for D2. Publication remains stopped
until the reviewer records a written determination for the exact versions and
artifacts below. No push, registry publication, signing, notarization, or other
external distribution is authorized by this packet.

## 1. Purpose & scope

The reviewer is asked to confirm the permitted distribution treatment and any
required notices, source offers, corresponding-source materials, or other
conditions for the three distribution objects in §2. In particular, the reviewer
should answer the six questions in §5 and identify any correction or additional
obligation.

Engineering has already completed the following boundary work; these are context,
not legal conclusions or requests to re-do the engineering investigation:

- The original GPL-3.0-or-later `zlog`, `ztracing`, and `ztracing_macro` path was
  removed from the distributed host graph by the landed Option 3′ local stub.
- The two formerly “unknown” Zed crate terms are determinable from explicit
  per-crate Apache-2.0 license files, although their Cargo manifest license fields
  remain absent.
- The candidate host archive includes a generated dependency inventory beside
  the project `LICENSE`, and the package dry-runs established the two package
  file allowlists (18 and 8 files).
- The source and artifact facts below are pinned to the evidence records; this
  packet does not treat a passing build, checksum, or inventory as legal approval.

## 2. Distribution objects

### (a) Compiled host archive

**Candidate:** `dist/react-gpui-host-0.2.0-aarch64-apple-darwin.tar.gz`  
**Archive SHA-256:**
`9b1760cbea118a4c2053c9074b84bd03f0f1efac7c3767f7964d8584192464b1`

The legal/release payloads are:

- compiled `react-gpui-host` binary;
- project `LICENSE` (Apache License 2.0 text);
- `THIRD-PARTY-NOTICES.md` (generated inventory of dependency names, versions,
  SPDX identifiers, and source provenance); and
- `SHA256SUMS` (checksums for the binary, README, `LICENSE`, and notices
  inventory).

The archive also contains its release `README.md`; the complete archive listing
and the checksum provenance are recorded in the [release artifact
manifest](../release-productionization/release-artifacts.md). The notices file
explicitly says it is an inventory, not a substitute for the third-party license
texts. The archive hash establishes content reproducibility, not publisher
identity.

### (b) npm tarballs

The intended package objects are:

| Package | Observed dry-run contents | License payload |
| --- | ---: | --- |
| `@react-gpui/core@0.2.0` | 18 files | Its own Apache-2.0 `LICENSE` only |
| `@react-gpui/dev@0.2.0` | 8 files | Its own Apache-2.0 `LICENSE` only |

The allowlists contain the package manifest, README, built JavaScript and
TypeScript declaration files, and the package's own `LICENSE`; they do not embed a
third-party license-text bundle or `THIRD-PARTY-NOTICES.md`. The package audit
records the exact 18/8 dry-run result and notes that no tarball was retained from
the dry run; the release workflow's uploaded tarballs would be the durable
artifact instances for a final artifact-level check. The dev dependency tree
contains `caniuse-lite 1.0.30001809` (CC-BY-4.0), which is addressed in Q4.

### (c) Source repository

The source repository's project-owned code is released under Apache-2.0, with the
root `LICENSE` and workspace/package metadata declaring or inheriting
`Apache-2.0`. This statement is about project-owned source; each vendored or
resolved third-party component retains its own license and obligations as listed
in the inventory. No source repository publication or push has occurred.

## 3. Resolved items (context, no action)

The GPL trio was an engineering blocker in the original host graph. Option 3′ is
now **EXECUTED**: the root Cargo patch substitutes an independently authored,
17-line `ztracing` proc-macro stub for the host graph. Its identity
`instrument` attribute discards the attribute arguments and returns the item
unchanged, matching the documented no-op behavior needed by the active host
callers. The stub has no `zlog`, `ztracing_macro`, or other dependency and copies
no Zed tracing source.

The [Option 3′ landing record](landing-option3.md) records the execution,
three-target graph absence checks, and the resulting narrowed STOP. The companion
[stub proof](spike-option3-stub.md) records the two-file composition (9-line
manifest plus 8-line implementation), the 17-line total, and the independent
Apache-2.0 authorship/license. Accordingly, the stub's license is the project's
Apache-2.0, not the removed Zed GPL licensing. The GPL trio's removal does not
resolve the independent questions in §§4–5.

## 4. Terms identified awaiting confirmation

### `gpui_shared_string` and `gpui_util`

At pinned Zed revision
`6805d952f9f3d702f760aa11b1547df8a625fa16`, both `gpui_shared_string 0.1.0`
and `gpui_util 0.1.0` have no Cargo `license` or `license-file` field. Each crate
does, however, carry a `LICENSE-APACHE` symlink to Zed's root
`LICENSE-APACHE`. The control crate `gpui` carries the same per-crate symlink and
has an explicit manifest field `license = "Apache-2.0"`. The linked files are
md5-matched (`776e07ed20b75b675553b3a113323c42`) and contain the Apache License,
Version 2.0 text.

Zed's README states that its source is licensed primarily under GPL-3.0-or-later,
“with Apache-2.0 components where marked.” The two crates are marked by those
per-crate Apache files. The terms-determinability re-check therefore identifies
the terms as Apache-2.0 while retaining the metadata finding and the need for
human confirmation.

**Question for the reviewer:** confirm whether that per-crate marking governs the
shipped `0.1.0` crate versions and the exact binary/npm/source artifacts in §2,
despite the missing Cargo manifest fields, and state any evidence or notice
condition needed.

### Other identified terms carried into the questions

The locked inventory also identifies `self_cell 1.3.0` as
`Apache-2.0 OR GPL-2.0-only`; `cbindgen 0.28.0`, `option-ext 0.2.0`, and
`dwrote 0.11.5` as MPL-2.0; `r-efi 5.3.0` and `6.0.0` as
`Apache-2.0 OR LGPL-2.1-or-later OR MIT`; and `caniuse-lite 1.0.30001809` as
CC-BY-4.0 in the dev package dependency inventory. These are not silently
cleared by the stub or by the presence of the inventory.

## 5. Open questions for counsel

Please answer each with **yes/no or a short instruction**, and identify the
artifact/version scope of the answer.

1. **Per-crate Apache marking:** For `gpui_shared_string 0.1.0` and `gpui_util
   0.1.0` at Zed revision `6805d952f9f3d702f760aa11b1547df8a625fa16`, does each
   crate's explicit per-crate `LICENSE-APACHE` marking govern the shipped crate
   versions and the §2 artifacts, notwithstanding the missing Cargo manifest
   license fields? If yes, what evidence should be retained?
2. **`self_cell`:** For `self_cell 1.3.0`'s `Apache-2.0 OR GPL-2.0-only`
   expression, does electing Apache-2.0 suffice for these distributions, and how
   should that election be evidenced?
3. **MPL-2.0:** For `cbindgen`, `option-ext`, and `dwrote`, what file-scope,
   notice, source, or other obligations apply to the compiled host binary/archive
   and any other §2 distribution object?
4. **CC-BY-4.0:** For `caniuse-lite 1.0.30001809` in the dev package dependency
   tree, is attribution required when the `@react-gpui/dev` package/tarball (or
   its dependency installation) is distributed, and where must it appear?
5. **Inventory versus texts:** Does the inventory-style
   `THIRD-PARTY-NOTICES.md`—with names, versions, SPDX identifiers, and sources but
   no embedded full license texts—satisfy Apache §4(a) attribution/notice needs
   for the host artifact, or must the artifact also carry the referenced texts?
6. **Completeness:** Are any notice, attribution, source-offer,
   corresponding-source, relinkable-object, or other obligations missing for the
   licenses and exact dependency records listed in the inventory and this packet?

## 6. Evidence index

The two spike records are one paired engineering evidence entry, yielding seven
source entries while preserving pointers to both files.

| Source entry | Pointer | One-line purpose |
| --- | --- | --- |
| Baseline audit | [2026-08-30.md](2026-08-30.md) | Initial dependency/artifact audit: GPL trio, missing metadata, dual/weak licenses, archive, and npm inventory. |
| Terms re-check | [recheck-2026-08-31.md](recheck-2026-08-31.md) | Establishes the per-crate Apache files, symlink/md5 evidence, Zed “where marked” policy, and remaining D2 boundary. |
| Feasibility | [feasibility.md](feasibility.md) | Tests feature-off, upstream-pin, fork, and legal-only paths and explains why the active engineering removal was needed. |
| Paired spikes | [spike-option3.md](spike-option3.md) and [spike-option3-stub.md](spike-option3-stub.md) | Records the larger fork proof and the narrower independent 17-line stub proof, including graph and maintenance/legal boundaries. |
| Landing record | [landing-option3.md](landing-option3.md) | Records Option 3′ execution, resulting host graph, lockfile/license-check changes, and narrowed STOP. |
| Notices inventory | [../../THIRD-PARTY-NOTICES.md](../../THIRD-PARTY-NOTICES.md) | Generated dependency inventory shipped beside the host `LICENSE`; expressly not a bundle of full third-party texts. |
| Active issue | [issues/01-gpl-and-unknown-host-licenses.md](issues/01-gpl-and-unknown-host-licenses.md) | Human-facing blocker and release-owner checklist: D2 confirmation remains required before any distribution. |
