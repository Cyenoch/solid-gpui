# GPL linkage feasibility options

Date: 2026-08-31  
Repository source under review: workspace GPUI patch at Zed revision
`6805d952f9f3d702f760aa11b1547df8a625fa16` (2026-08-24)  
Upstream comparison: vendored checkout `origin/main` at
`1662f5f3f6497c5f80830ccdca1edfd1fc0c6c6a` (2026-08-29)

This is an engineering feasibility assessment for the publication blocker in
[`issues/01-gpl-and-unknown-host-licenses.md`](issues/01-gpl-and-unknown-host-licenses.md).
It does not select a license, approve distribution, or replace legal review.
No Rust source, Cargo manifest, lockfile, or live gallery process was changed.

## Executive verdict

There is no "feature off" escape in the pinned GPUI manifest. `ztracing`,
`gpui_shared_string`, and `gpui_util` are ordinary dependencies, not optional
or target-gated dependencies. Disabling GPUI default features therefore does
not remove them. The no-environment `ztracing` mode makes the instrumentation
attribute a no-op, but it is not a Cargo feature and does not remove the
packages from the resolved graph.

`origin/main` has not fixed this: it still declares the same three ordinary
edges and still has the two absent license fields. The GPUI direct edge and the
SVG instrumentation were added recently, on 2026-08-05, but the host's GPL
chain was already reachable through GPUI's ordinary `sum_tree` dependency after
Zed's December 2025 tracing changes. A pin bump to the observed `origin/main`
would not clear any of the three GPL crates or either unknown-license crate.

The only currently actionable engineering removal is a maintained local fork
that removes the instrumentation and its dependency edges from **both** GPUI
and `sum_tree`. Patching only GPUI is insufficient because `sum_tree` remains
an ordinary GPUI dependency and independently imports `ztracing`.

## Probe A — pinned manifest and feature reasoning

### Exact pinned GPUI lines

The source is the vendored checkout at
`/Users/jgbingzi/.cargo/git/checkouts/zed-a70e2ad075855582/6805d95`.
At the pinned revision, `crates/gpui/Cargo.toml` contains:

```text
19:[features]
20:default = ["font-kit", "wayland", "x11", "windows-manifest"]
21:test-support = [
...
46:[dependencies]
...
61:gpui_shared_string.workspace = true
...
96:gpui_util.workspace = true
...
107:ztracing.workspace = true
109:[target.'cfg(target_family = "wasm")'.dependencies]
114:[target.'cfg(target_os = "macos")'.dependencies]
```

The three relevant lines are in the ordinary `[dependencies]` table. They are
not written as `optional = true`, and they occur before the target-specific
tables. The GPUI default feature list has no tracing feature, and the manifest
has no feature that controls `ztracing`.

The two Zed crates' own pinned manifests are also ordinary dependency packages:

```text
crates/gpui_shared_string/Cargo.toml
1:[package]
2:name = "gpui_shared_string"
3:version = "0.1.0"
4:publish.workspace = true
5:edition.workspace = true
# no `license` field

crates/gpui_util/Cargo.toml
1:[package]
2:name = "gpui_util"
3:version = "0.1.0"
4:publish.workspace = true
5:edition.workspace = true
# no `license` field
```

Their dependency sections are not optional or target-gated in a way that
removes the package. `gpui_shared_string` has no manifest `license` field, and
neither does `gpui_util`.

The GPL package manifests at the same revision state:

```text
crates/ztracing/Cargo.toml:6:license = "GPL-3.0-or-later"
crates/ztracing/Cargo.toml:16:zlog.workspace = true
crates/ztracing/Cargo.toml:21:ztracing_macro.workspace = true
crates/zlog/Cargo.toml:6:license = "GPL-3.0-or-later"
crates/ztracing_macro/Cargo.toml:6:license = "GPL-3.0-or-later"
```

`zlog` and `ztracing_macro` are ordinary dependencies of `ztracing`; their
`default = []`/empty feature behavior does not make the dependency optional.

### Feature-tree result

The host manifest has `default = []`, but its `gpui`, `gpui-platform`, and
`react-gpui` dependencies are ordinary dependencies. A target-qualified
feature-tree query with GPUI defaults disabled still reported these paths
(the command was run with no dev dependencies):

```text
cargo tree --offline -p react-gpui-host \
  --target aarch64-apple-darwin --no-default-features \
  --no-dev-dependencies -i ztracing

ztracing -> gpui -> gpui_apple [build-dependencies]
  -> gpui_macos -> gpui_platform -> react-gpui-host
ztracing -> sum_tree -> gpui -> react-gpui-host
```

The corresponding queries also retained `zlog`, `ztracing_macro`,
`gpui_shared_string`, and `gpui_util`. The normal graph has both the direct
GPUI/SVG path and the `sum_tree` path; `--no-default-features` removes unrelated
GPUI defaults, not ordinary dependencies.

The no-op mode is a different question. Pinned `crates/ztracing/build.rs`
checks environment variables `ZTRACING` and `ZTRACING_WITH_MEMORY` and emits
cfgs only when those variables are present. Without them,
`ztracing_macro::instrument` returns the input item unchanged
(`crates/ztracing_macro/src/lib.rs:1-6`), so the two instrumented functions
remain available and their tracing wrapper is absent. That is useful runtime
behavior, but it is not a graph-removal switch: the `ztracing`, `zlog`, and
`ztracing_macro` packages are still resolved and compiled as ordinary
dependencies. No host rebuild was performed for this feasibility probe.

**Probe A verdict:** no Cargo feature can build this host without the three GPL
packages or the two no-license-field packages today. The engineering cost of a
true removal is not zero; it requires changing upstream-derived source or
waiting for an upstream change.

## Probe B — upstream evolution

### Current upstream state

`git show origin/main:crates/gpui/Cargo.toml` at
`1662f5f3f6497c5f80830ccdca1edfd1fc0c6c6a` still contains ordinary:

```text
gpui_shared_string.workspace = true
gpui_util.workspace = true
ztracing.workspace = true
```

The upstream `ztracing` manifest still contains ordinary `zlog.workspace` and
`ztracing_macro.workspace` edges and `license = "GPL-3.0-or-later"`; `zlog` and
`ztracing_macro` still declare the same GPL expression. Upstream
`gpui_shared_string/Cargo.toml` and `gpui_util/Cargo.toml` remain without a
`license` field. The pinned-to-main diff for these relevant manifests contains
only unrelated GPUI benchmark/stacker changes; it contains no dependency
removal, optionalization, or license-field addition.

The current upstream GPUI source still has:

```text
#[ztracing::instrument(skip_all)]
pub fn parse_svg(...)

#[ztracing::instrument(skip_all)]
pub fn render_parsed(...)
```

at `crates/gpui/src/svg_renderer.rs` (the same two attributes are present at
pinned lines 190 and 196). No upstream commit was found that replaces these
attributes with another crate or removes the relevant functions' tracing.

**Probe B current-state verdict:** `origin/main` does not clear any of the
three GPL packages or either unknown-license package. Option 2 is therefore
not presently available as a pin-bump solution.

### When the edge entered

The history separates three dates and avoids conflating a new direct edge with
the first appearance of a GPL package in the whole GPUI graph:

| Event | Commit | Date | Engineering meaning |
| --- | --- | --- | --- |
| `zlog` crate initialized | `16ad7424d66ebcf1500b2e4ecbb00a1153c0f637` | 2025-03-21 | GPL package exists in Zed; not yet evidence of this host edge by itself. |
| `ztracing` and `ztracing_macro` introduced; `sum_tree` gained ordinary ztracing usage | `b558be7ec60b265837e34d6f9b6f0ef176c20082` | 2025-12-05 | The GPUI graph could reach ztracing through its ordinary `sum_tree` dependency. |
| `ztracing` gained ordinary `zlog` dependency | `1029a8fbaf5271b6eb3e4e51f9e5cb015c52f760` | 2025-12-11 | The three-package GPL chain became reachable through that `sum_tree` path. |
| GPUI gained direct `ztracing` dependency and SVG `parse_svg`/`render_parsed` attributes | `00cba838ad4e0be4b6176438551b72b2d512e9f8` | 2026-08-05 | Direct GPUI/SVG linkage is recent, 19 days before the pinned revision. |

The pinned revision is 2026-08-24. Thus the direct GPUI edge is a recent
regression relative to this pin, but the strict host graph is not one of the
first affected graphs: `sum_tree` had already made the GPL chain reachable
since December 2025. The recent GPUI change increased direct use and added the
specific SVG instrumentation named by the blocker; it did not create the first
possible package reachability.

Since the pin, upstream has accumulated 96 commits through `origin/main`,
including broad GPUI/platform and feature changes. The existing drift
assessment records that a future GPUI update requires a full GPUI/platform graph
review rather than a version-only edit
([`.scratch/upstream-assessment/2026-08-30-gpui-drift.md`](../upstream-assessment/2026-08-30-gpui-drift.md),
“Pin-to-release delta and upgrade impact”). That review includes reconciling
changed GPUI/platform APIs and target graphs, refreshing the lock and build
matrix, and rerunning the exact artifact/license inventory. It is not justified
solely by the current GPL finding because the observed upstream state has no
GPL removal.

## Option matrix

The matrix describes engineering effects only. Every row still requires the
release owner and legal reviewer to resolve the license choices and artifact
obligations.

| Option | What engineering can establish now | Clears `zlog`/`ztracing`/`ztracing_macro`? | Clears `gpui_shared_string`/`gpui_util`? | Cost and residual obligations |
| --- | --- | --- | --- | --- |
| **1. Feature off today** | **No switch exists.** Omitting `ZTRACING` makes the proc-macro identity/no-op mode, but leaves all ordinary package edges. | **No.** All three remain in the resolved host graph. | **No.** Both ordinary GPUI dependencies remain. | Near-zero behavior change, but zero blocker progress. `self_cell`'s Apache-or-GPL choice, weak-copyleft entries, notices/source duties, and artifact re-audit remain unchanged. |
| **2. Pin bump clears it** | **Not available at current `origin/main`.** Main still has the same ordinary edges and manifests. A later fixed upstream release would require the already-scoped full GPUI/platform graph review, not a version-only edit. | **No for the currently observed bump.** Hypothetically yes only if a future upstream revision removes every active path and the post-bump graph proves it. | **No for the currently observed bump.** Main has not added fields. | High one-time review cost: changed APIs/platform crates and lockfile, all target/feature builds and callsites, then exact license inventory. Existing GPUI drift findings remain relevant; no current released fix to adopt. |
| **3. Patch-fork ztracing out** | Technically feasible, but only as a source fork. A `[patch]` substitution alone cannot erase a dependency; the fork must remove the active GPUI **and** `sum_tree` edges and their attributes. | **Yes, conditionally.** After removing every active path and confirming with target-qualified `cargo tree`/license inventory, all three disappear from this host graph. | **No.** Their separate ordinary GPUI edges and unknown manifest licenses remain and need holder/legal resolution. | Medium/high ongoing cost: maintain a local Zed fork, lock each upstream update, reapply source edits, monitor upstream changes/security fixes, update the lock, and test all targets. The fork's retained upstream copyright/license notices and the license of project-authored modifications require legal confirmation; no relicense is inferred here. `self_cell` and weak-copyleft obligations remain. |
| **4. Accept GPL terms for binaries** | No code change. Keep upstream and obtain a written legal determination for distribution and required notices/source/corresponding-source or relinkable-object materials. | **No code clearance; legal acceptance only.** The three packages remain present. | **No.** Missing license fields still require copyright-holder/legal resolution. | Lowest engineering cost, but publication remains blocked until legal accepts all findings and artifact-level obligations. `self_cell`'s dual expression and weak-copyleft review remain unchanged. |

### Exact fork scope if Option 3 is selected

A minimally scoped local Zed fork for the active macOS host graph would need to:

1. Remove `ztracing.workspace = true` from the forked `crates/gpui/Cargo.toml`.
2. Remove the two `#[ztracing::instrument(skip_all)]` attributes from
   `crates/gpui/src/svg_renderer.rs`. The functions themselves stay; this
   removes profiling wrappers, not SVG behavior.
3. Remove `ztracing.workspace = true` from the forked
   `crates/sum_tree/Cargo.toml`.
4. Remove `use ztracing::instrument` and the four instrument attributes in
   `crates/sum_tree/src/cursor.rs` (`prev`, `seek`, `seek_forward`, and
   `seek_internal`).
5. Remove `use ztracing::instrument` and the three instrument attributes in
   `crates/sum_tree/src/sum_tree.rs` (`find_exact`, `find`, and
   `find_with_prev`).
6. Remove now-unused direct `tracing` declarations in those forked manifests
   only if the fork's scoped source check confirms no other direct use; this is
   not required to remove the GPL packages themselves.
7. Point Cargo's Zed-source patch at the local fork for both `gpui` and
   `sum_tree` (a conceptual source patch is shown below), update the lockfile,
   and verify that no other active target path imports `ztracing`:

   ```toml
   [patch."https://github.com/zed-industries/zed"]
   gpui = { path = "path/to/zed-fork/crates/gpui" }
   sum_tree = { path = "path/to/zed-fork/crates/sum_tree" }
   ```

   The exact patch-source key/path must match the final fork layout and Cargo
   source resolution; this is an implementation sketch, not a manifest change
   made by this assessment.

Patching only `gpui` would leave `sum_tree -> ztracing -> zlog,
ztracing_macro`, so it would not meet the stated objective. Conversely, a local
package that merely keeps the name `ztracing` but changes its implementation
would still leave a package in the graph and would not by itself resolve the
GPL manifest finding.

The fork modification is legal-adjacent: retained Zed source licenses and
copyright notices must remain, and the project must not assign a license to
upstream-owned code. The license and notices for any project-authored fork
diff must be decided by the appropriate owner/legal reviewer.

## Engineering recommendation

Do not bump the pin solely for this blocker: the observed upstream state does
not remove the GPL chain, and the drift assessment requires a broad graph/API
review. Do not describe `ZTRACING`-off as a publication clearance.

Present Option 4 to legal first because it preserves upstream code and has zero
engineering churn; if legal rejects GPL distribution, Option 3 is the only
currently actionable technical path, subject to a deliberately owned fork and
a complete post-fork graph/license/artifact audit. This is an engineering
recommendation, not the user's or legal reviewer's decision. Regardless of the
choice, `gpui_shared_string`, `gpui_util`, `self_cell`, and the other identified
license obligations remain separate review items.

## Evidence index

- Pinned GPUI manifest: `crates/gpui/Cargo.toml:19-107` in the vendored Zed
  checkout at revision `6805d952`.
- Pinned GPL manifests: `crates/ztracing/Cargo.toml:1-29`,
  `crates/zlog/Cargo.toml:1-24`, and
  `crates/ztracing_macro/Cargo.toml:1-11`.
- Pinned unknown-license manifests:
  `crates/gpui_shared_string/Cargo.toml:1-16` and
  `crates/gpui_util/Cargo.toml:1-15`.
- Pinned source attributes:
  `crates/gpui/src/svg_renderer.rs:189-197`.
- Pinned `ztracing` no-op build/macro behavior:
  `crates/ztracing/build.rs:1-14` and
  `crates/ztracing_macro/src/lib.rs:1-6`.
- Pinned `sum_tree` active tracing imports/attributes:
  `crates/sum_tree/src/cursor.rs:1-4,214-216,406-465` and
  `crates/sum_tree/src/sum_tree.rs:12-13,399-511`.
- Upstream manifest: `git show origin/main:crates/gpui/Cargo.toml` and the same
  three GPL/two unknown-license manifests.
- Upstream history: `git log origin/main --oneline -- crates/ztracing crates/zlog
  crates/gpui/Cargo.toml`; commits are listed in the chronology above.
- Existing GPUI drift scope:
  [`.scratch/upstream-assessment/2026-08-30-gpui-drift.md`](../upstream-assessment/2026-08-30-gpui-drift.md).
