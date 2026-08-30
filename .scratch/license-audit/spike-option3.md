# Option 3 fork proof spike

Date: 2026-08-31  
Purpose: time-boxed engineering feasibility proof for the GPL-linkage blocker.  
Starting repository HEAD: `02a640e` (`docs: gpl linkage feasibility`)  
Pinned Zed revision: `6805d952f9f3d702f760aa11b1547df8a625fa16`  
Worktree: `/tmp/gpl-spike` (throwaway; removed after the proof)

This is an engineering result, not a license decision. It proves the requested
macOS target graph and host build path; it does not clear the two separate
`gpui_shared_string`/`gpui_util` missing-license findings, select a license for
fork modifications, or replace legal review.

## Method

1. Created a detached worktree with `git worktree add /tmp/gpl-spike HEAD`.
2. Copied only the pinned `crates/gpui` and `crates/sum_tree` directories from
the locally cached Zed checkout into `/tmp/gpl-spike/zed-fork/crates/`, retaining
the upstream `LICENSE-APACHE` files.
3. Rewrote the two copied manifests so they no longer depend on Zed's root
workspace inheritance. Registry dependencies were pinned to the exact version
already resolved in this repository's `Cargo.lock`; Zed workspace-member path
dependencies were represented as explicit git dependencies at the pinned Zed
URL/revision/package. Member-local features and `optional` flags were retained.
The full declaration table is below.
4. Applied the exact source scope from `feasibility.md`: remove the two GPUI
SVG `ztracing` attributes; remove the `ztracing` import and four attributes in
`sum_tree/src/cursor.rs`; remove the `ztracing` import and three attributes in
`sum_tree/src/sum_tree.rs`; remove `ztracing` from both fork manifests.
5. Verified the lockfile GPUI source and used the exact patch source key
`https://github.com/zed-industries/zed` (the key is the URL without the
`?rev=...` query):

```toml
[patch."https://github.com/zed-industries/zed"]
gpui = { path = "zed-fork/crates/gpui" }
sum_tree = { path = "zed-fork/crates/sum_tree" }
```

The first trial used the query-bearing key and Cargo warned that the patch was
unused. Replacing it with the exact source key above made both local patches
resolve. The root `gpui` workspace dependency was also changed from the
crates.io shorthand to the pinned Zed git source so the patch has the same
source identity as the host's `gpui-platform` dependency.

## Scope and diff size

The copied source trees are otherwise unchanged. Relative to the pinned Zed
checkout, the fork diff was:

| File | Added | Removed | Change |
| --- | ---: | ---: | --- |
| `gpui/Cargo.toml` | 70 | 74 | workspace inheritance translated; `ztracing` edge removed |
| `gpui/src/svg_renderer.rs` | 0 | 2 | two attributes removed |
| `sum_tree/Cargo.toml` | 11 | 15 | workspace inheritance translated; `ztracing` edge removed |
| `sum_tree/src/cursor.rs` | 0 | 5 | import plus four attributes removed |
| `sum_tree/src/sum_tree.rs` | 0 | 4 | import plus three attributes removed |
| **fork total** | **81** | **100** | **181 changed lines** |

The throwaway worktree's host integration changes were additionally:

| File | Added | Removed | Reason |
| --- | ---: | ---: | --- |
| root `Cargo.toml` | 4 | 3 | exact Zed source identity and two local patch entries |
| `Cargo.lock` | 0 | 90 | local GPUI/sum_tree source entries and the now-unused ztracing chain were regenerated |

The source-only fork scope is exactly 11 deletions: 2 in GPUI SVG, 5 in
`sum_tree/cursor.rs`, and 4 in `sum_tree/sum_tree.rs`. No function or behavior
was otherwise changed. `tracing` remains in both manifests because the
feasibility report scoped only the `ztracing` removal and the source check did
not establish that all direct `tracing` uses were removable. `sum_tree`'s
`zlog` dev-dependency was also left unchanged: the requested proof is the
explicit `--no-dev-dependencies` host graph.

Exact source diff:

```diff
--- pinned/crates/gpui/src/svg_renderer.rs
+++ fork/crates/gpui/src/svg_renderer.rs
@@ -187,13 +187,11 @@ impl SvgRenderer {
     }
 
     /// Parses SVG data into a [`ParsedSvg`] that can be rasterized at any scale.
-    #[ztracing::instrument(skip_all)]
     pub fn parse_svg(&self, bytes: &[u8]) -> Result<ParsedSvg, usvg::Error> {
         usvg::Tree::from_data(bytes, &self.usvg_options).map(ParsedSvg)
     }
 
     /// Rasterizes a previously parsed SVG into an image buffer.
-    #[ztracing::instrument(skip_all)]
     pub fn render_parsed(
         &self,
         svg: &ParsedSvg,
--- pinned/crates/sum_tree/src/cursor.rs
+++ fork/crates/sum_tree/src/cursor.rs
@@ -1,7 +1,6 @@
 use super::*;
 use heapless::Vec as ArrayVec;
 use std::{cmp::Ordering, mem, sync::Arc};
-use ztracing::instrument;
 
 #[derive(Clone)]
 struct StackEntry<'a, T: Item, D> {
@@ -212,7 +211,6 @@ where
     }
 
     #[track_caller]
-    #[instrument(skip_all)]
     pub fn prev(&mut self) {
         self.search_backward(|_| true)
     }
@@ -404,7 +402,6 @@ where
 {
     /// Returns whether we found the item you were seeking for.
     #[track_caller]
-    #[instrument(skip_all)]
     pub fn seek<Target>(&mut self, pos: &Target, bias: Bias) -> bool
     where
         Target: SeekTarget<'a, T::Summary, D>,
@@ -419,7 +416,6 @@ where
     ///
     /// If we did not seek before, use seek instead in that case.
     #[track_caller]
-    #[instrument(skip_all)]
     pub fn seek_forward<Target>(&mut self, pos: &Target, bias: Bias) -> bool
     where
         Target: SeekTarget<'a, T::Summary, D>,
@@ -461,7 +457,6 @@ where
 
     /// Returns whether we found the item you were seeking for.
     #[track_caller]
-    #[instrument(skip_all)]
     fn seek_internal(
         &mut self,
         target: &dyn SeekTarget<'a, T::Summary, D>,
--- pinned/crates/sum_tree/src/sum_tree.rs
+++ fork/crates/sum_tree/src/sum_tree.rs
@@ -10,7 +10,6 @@ use std::marker::PhantomData;
 use std::mem;
 use std::{cmp::Ordering, fmt, iter::FromIterator, sync::Arc};
 pub use tree_map::{MapSeekTarget, TreeMap, TreeSet};
-use ztracing::instrument;
 
 #[cfg(test)]
 pub const TREE_BASE: usize = 2;
@@ -396,7 +395,6 @@ impl<T: Item> SumTree<T> {
     /// A more efficient version of `Cursor::new()` + `Cursor::seek()` + `Cursor::item()`.
     ///
     /// Only returns the item that exactly has the target match.
-    #[instrument(skip_all)]
     pub fn find_exact<'a, 'slf, D, Target>(
         &'slf self,
         cx: <T::Summary as Summary>::Context<'a>,
@@ -422,7 +420,6 @@ impl<T: Item> SumTree<T> {
     }
 
     /// A more efficient version of `Cursor::new()` + `Cursor::seek()` + `Cursor::item()`
-    #[instrument(skip_all)]
     pub fn find<'a, 'slf, D, Target>(
         &'slf self,
         cx: <T::Summary as Summary>::Context<'a>,
@@ -507,7 +504,6 @@ impl<T: Item> SumTree<T> {
     }
 
     /// A more efficient version of `Cursor::new()` + `Cursor::seek()` + `Cursor::item()`
-    #[instrument(skip_all)]
     pub fn find_with_prev<'a, 'slf, D, Target>(
         &'slf self,
         cx: <T::Summary as Summary>::Context<'a>,
```

The standalone manifest edits are represented exactly by the following
translation table. Entries with multiple declarations appear once with the
number of declarations shown; the same explicit value was used for each
repeated target/dev form unless noted.

## Manifest translation table

The pinned GPUI manifest contained **69 workspace declarations / 60 unique
names** (the `ztracing` declaration is included in that count but removed in
the fork). The pinned sum_tree manifest contained **10 declarations / 9 unique
names** (the `ztracing` declaration is included in that count but removed in
the
fork). The count is above the report's rough “~10” estimate for GPUI, but only
two manifests needed translation; copying Zed's 264-member workspace would
have been substantially larger and less bounded. Translation was therefore
still the cheaper proof route.

### GPUI

| Name | Declarations | Explicit standalone value |
| --- | ---: | --- |
| `accesskit` | 1 | `version = "=0.24.1", features = ["enumn"]` |
| `anyhow` | 1 | `version = "=1.0.104"` |
| `async-task` | 1 | `version = "=4.7.1"` |
| `backtrace` | 2 | `version = "=0.3.76"`; dependency form preserves `optional = true` |
| `bitflags` | 1 | `version = "=2.13.1"` |
| `collections` | 2 | `git = Zed, rev = pinned, package = "collections", version = "0.1.0"`; dev form adds `test-support` |
| `criterion` | 1 | `version = "0.5", features = ["html_reports"], optional = true` |
| `ctor` | 1 | `version = "=1.0.13"` |
| `derive_more` | 1 | `version = "=2.1.1"` plus the pinned add/deref/display/from/mul/not feature list |
| `etagere` | 1 | `version = "=0.2.15"` |
| `futures` | 1 | `version = "=0.3.34"` |
| `futures-concurrency` | 1 | `version = "=7.7.1"` |
| `gpui_macros` | 1 | pinned Zed git source/package |
| `gpui_shared_string` | 1 | pinned Zed git source/package |
| `http_client` | 2 | pinned Zed git source/package; target dev form adds `test-support` |
| `image` | 1 | `version = "=0.25.10", default-features = false` plus pinned image feature list |
| `inventory` | 1 | `version = "=0.3.24"` |
| `itertools` | 1 | `version = "=0.14.0"` |
| `log` | 2 | `version = "=0.4.34", features = ["kv_unstable_serde", "serde"]` |
| `parking_lot` | 1 | `version = "=0.12.5"` |
| `postage` | 1 | `version = "=0.5.0", features = ["futures-traits"]` |
| `proptest` | 3 | pinned proptest git rev, `features = ["attr-macro"]`; dependency form preserves `optional = true` |
| `chrono` | 1 | `version = "=0.4.45", features = ["serde"]` |
| `profiling` | 1 | `version = "=1.0.18"` |
| `rand` | 2 | `version = "=0.9.5"` |
| `raw-window-handle` | 1 | `version = "=0.6.2"` |
| `regex` | 1 | `version = "=1.13.1"` |
| `refineable` | 1 | pinned Zed git source/package |
| `scheduler` | 2 | pinned Zed git source/package; dev form adds `test-support` |
| `resvg` | 1 | `version = "=0.46.0", default-features = false` plus pinned text/system-font/raster feature list |
| `usvg` | 1 | `version = "=0.46.0", default-features = false` |
| `util_macros` | 1 | pinned Zed git source/package |
| `schemars` | 1 | `version = "=1.2.2", features = ["indexmap2"]` |
| `serde` | 1 | `version = "=1.0.229", features = ["derive", "rc"]` |
| `serde_json` | 1 | `version = "=1.0.151", features = ["preserve_order", "raw_value"]` |
| `slotmap` | 1 | `version = "=1.1.1"` |
| `smallvec` | 1 | `version = "=1.15.2", features = ["union", "const_new"]` |
| `async-channel` | 1 | `version = "=2.5.0"` |
| `stacksafe` | 1 | `version = "=1.0.3"` |
| `strum` | 1 | `version = "=0.27.2", features = ["derive"]` |
| `sum_tree` | 1 | pinned Zed git source/package; root patch substitutes the local fork |
| `thiserror` | 1 | `version = "=2.0.20"` |
| `tracing` | 1 | `version = "=0.1.44"` |
| `gpui_util` | 1 | pinned Zed git source/package |
| `hdrhistogram` | 1 | `version = "7"`, optional (no resolved package because feature is off) |
| `pollster` | 1 | `version = "=0.4.0"` |
| `url` | 1 | `version = "=2.5.8"` |
| `uuid` | 2 | `version = "=1.25.0", features = ["v4", "v5", "v7", "serde"]`; wasm form adds `js` |
| `web-time` | 1 | `version = "=1.1.0"` |
| `heapless` | 1 | `version = "=0.9.3"` |
| `ztracing` | 1 | **removed**, not translated into the fork manifest |
| `core-video` | 1 | `version = "=0.5.2", features = ["metal"]` |
| `objc2` | 1 | `version = "=0.6.4"`, optional |
| `scap` | 1 | pinned `zed-scap` git source/rev/package, default features off, optional |
| `windows` | 1 | `version = "=0.61.3", features = ["Win32_Foundation", "Win32_System_Power"]` |
| `env_logger` | 1 | `version = "0.11"` (dev-only) |
| `gpui_platform` | 1 | pinned Zed git source/package, default features off, `font-kit/wayland/x11` |
| `unicode-segmentation` | 1 | `version = "=1.13.3"` |
| `reqwest_client` | 1 | pinned Zed git source/package, `test-support` (dev-only) |
| `wasm-bindgen` | 1 | `version = "=0.2.127"` (wasm dev-only) |

Names already written as explicit declarations in the upstream manifest (for
example `num_cpus`, `parking`, `ttf-parser`, `seahash`, `taffy`, `waker-fn`,
`lyon`, `pin-project`, `spin`, `objc2-metal`, `getrandom`, `font-kit`,
`embed-resource`, and `bindgen`) were not part of the workspace-translation
count and were retained.

### sum_tree

| Name | Declarations | Explicit standalone value |
| --- | ---: | --- |
| `heapless` | 1 | `version = "=0.9.3"` |
| `rayon` | 1 | `version = "=1.12.0"` |
| `log` | 1 | `version = "=0.4.34", features = ["kv_unstable_serde", "serde"]` |
| `ztracing` | 1 | **removed**, not translated into the fork manifest |
| `tracing` | 1 | `version = "=0.1.44"` |
| `proptest` | 2 | pinned proptest git rev, `features = ["attr-macro"]`; dependency form preserves `optional = true` |
| `ctor` | 1 | `version = "=1.0.13"` (dev-only) |
| `rand` | 1 | `version = "=0.9.5"` (dev-only) |
| `zlog` | 1 | pinned Zed git source/package (dev-only; retained outside the no-dev proof scope) |

The translation itself was mechanical and bounded, but the actual count makes
clear that a maintained fork must own a sizeable manifest surface, not only the
11 source deletions.

## Graph-clear evidence

After correcting the patch key and regenerating `Cargo.lock`, the local lock
entries for `gpui` and `sum_tree` had no `source` field (they were local paths),
`gpui` had no `ztracing` dependency, `sum_tree` had no `ztracing` dependency,
and the lockfile no longer contained `ztracing`, `zlog`, or
`ztracing_macro` package records in the target no-dev resolution.

The three required commands were run exactly against the target-qualified host
without dev dependencies:

```text
$ cargo tree -p react-gpui-host --target aarch64-apple-darwin --no-dev-dependencies -i ztracing
warning: the --no-dev-dependencies flag has changed to -e=no-dev
error: package ID specification `ztracing` did not match any packages
help: a package with a similar name exists: `tracing`

$ cargo tree -p react-gpui-host --target aarch64-apple-darwin --no-dev-dependencies -i zlog
warning: the --no-dev-dependencies flag has changed to -e=no-dev
error: package ID specification `zlog` did not match any packages
help: a package with a similar name exists: `log`

$ cargo tree -p react-gpui-host --target aarch64-apple-darwin --no-dev-dependencies -i ztracing_macro
warning: the --no-dev-dependencies flag has changed to -e=no-dev
error: package ID specification `ztracing_macro` did not match any packages
```

These are Cargo's package-ID-not-found results, not an empty tree hidden by a
shell filter. Verdict: **3/3 requested GPL package IDs are gone from the
resolved target host graph**. This is the requested conditional graph result;
it does not claim that a test/dev graph is GPL-free while `sum_tree` retains its
upstream `zlog` dev-dependency.

## Build and smoke evidence

| Command | Result | Wall time |
| --- | --- | ---: |
| `cargo check -p react-gpui-host` | passed; only the existing `rust_analyzer` unexpected-cfg and future-incompatibility warnings | 35.09 s |
| `cargo build -p react-gpui-host --release` | passed; same non-fatal warnings | 89.76 s |
| `./target/release/react-gpui-host --version` | `react-gpui-host 0.2.0 protocol=v3` | 1.01 s |

No live gallery process was touched. No host source, package manifest, or
release configuration in the main tree was changed.

## Effort and maintained-fork cost

The worktree was created at approximately `04:19:07 +0800`; the recorded
end-of-proof clock was approximately `05:11:42 +0800`, about **52m35s**. This
exceeded the nominal 45-minute box, largely because the first patch-key trial
was invalid and the standalone rewrite/build needed correction and resolution.
The proof nevertheless completed all requested gates. The overrun is itself a
cost signal: the fork is buildable, but not a one-line Cargo patch.

Observed one-time proof effort:

- copy and scoped source edits: small; exactly two manifests and three source
  files;
- manifest rewrite: 69 GPUI declarations and 10 sum_tree declarations, with
  60 and 9 unique names respectively; approximately the first 14 minutes of
the spike before build troubleshooting;
- patch/lock correction: one failed query-bearing source key, then successful
  local resolution using the base URL key;
- validation: approximately 35 seconds check plus 90 seconds release build.

A maintained fork should be budgeted as **medium/high ongoing cost**, not as a
single patch: preserve upstream notices and copyright, decide the legal status
of project-authored fork changes, reapply the source and manifest edits on each
Zed update, reconcile all transitive workspace-member revisions, update and
review the lockfile, repeat target/feature/test graph audits, rebuild all
release targets, and monitor upstream security/API changes. A reasonable
engineering estimate is a dedicated owner for an initial one- to two-day
cutover/review, followed by recurring maintenance on each upstream pin/update;
the exact cadence depends on Zed release frequency. This is an engineering
estimate, not a staffing commitment.

The fork also leaves `gpui_shared_string` and `gpui_util` as ordinary upstream
members without manifest `license` fields. Removing the GPL trio therefore does
not by itself make publication legally clear.

## Verdict

**PROVEN, conditionally and narrowly:** a two-crate source fork of GPUI and
`sum_tree`, with the exact `ztracing` edges/imports/attributes removed and the
manifests made standalone, builds `react-gpui-host` for the target and clears
all three requested GPL package IDs from its `--no-dev-dependencies` resolved
graph. The release binary also passes the `--version` smoke test.

**Cost:** medium/high ongoing fork maintenance; the nominal proof box was
exceeded. Option 3 is technically real, but it is not a low-cost or
maintenance-free publication switch. The two unknown-license crates and all
other license/artifact obligations remain for the release owner and legal
reviewer. Option 4 remains the lower-engineering-churn route if legal accepts
the upstream terms.
