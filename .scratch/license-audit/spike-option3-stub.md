# Option 3′ stub patch proof

Date: 2026-08-31
Purpose: time-boxed engineering proof of the smaller Option 3 variant.
Starting repository HEAD: `d8608e2` (`docs: readiness refresh`)
Pinned Zed revision: `6805d952f9f3d702f760aa11b1547df8a625fa16`
Experiment worktree: `/tmp/stub-spike` (throwaway; removed after the proof)

This is an engineering result, not a license decision. It does not clear the
separate `gpui_shared_string`/`gpui_util` missing-license findings, decide
whether any license option is acceptable, or replace legal review.

## Executive result

**PROVEN for the current macOS host graph:** Cargo accepts a local path patch
for the git-sourced `ztracing` package under the exact source key
`https://github.com/zed-industries/zed`. The local replacement is a two-file,
17-line, independently authored Apache-2.0 proc-macro crate. It exports
`ztracing::instrument` as an identity attribute and has no `zlog`,
`ztracing_macro`, or other dependency. The host check, release build, version
smoke, and requested React GPUI library tests all passed.

The proof removes the three GPL package records (`zlog`, `ztracing`, and
`ztracing_macro`) from the resolved target host graph and Cargo.lock. The
remaining `ztracing` package name is our local proc-macro at
`/private/tmp/stub-spike/ztracing-stub`, not Zed source. This is narrower than a
whole-Zed-workspace replacement: runtime users in unrelated Zed crates remain
outside the host graph and would require additional APIs if later pulled in.
`gpui_shared_string` and `gpui_util` remain ordinary Zed dependencies without
manifest license fields, so the release STOP remains active.

## Usage-exhaustiveness scan

The pre-patch graph was captured with:

```text
cargo tree -p react-gpui-host --target aarch64-apple-darwin --no-dev-dependencies
```

It contained 22 distinct pinned Zed package identities:
`collections`, `derive_refineable`, `gpui`, `gpui_apple`, `gpui_macos`,
`gpui_macros`, `gpui_platform`, `gpui_shared_string`, `gpui_util`,
`gpui_web`, `gpui_wgpu`, `gpui_windows`, `http_client`, `media`, `perf`,
`refineable`, `scheduler`, `sum_tree`, and `util_macros` (with target-irrelevant
platform packages also present in Cargo metadata). The full tree, rather than
only `gpui` and `sum_tree`, was used to identify which workspace members could
reach tracing. The source scan then covered the entire pinned checkout at
`/Users/jgbingzi/.cargo/git/checkouts/zed-a70e2ad075855582/6805d95`, excluding
only the `ztracing` and `ztracing_macro` implementation directories when
classifying consumers, for all four forms requested:

```text
ztracing::
use ztracing
zlog::
ztracing_macro::
```

### Current host graph

Only `gpui` and `sum_tree` consume `ztracing` in the target normal/build graph:

- `gpui/src/svg_renderer.rs:190,196`: two
  `#[ztracing::instrument(skip_all)]` attributes.
- `sum_tree/src/cursor.rs:4`: `use ztracing::instrument`; its four
  instrumented methods use the imported attribute.
- `sum_tree/src/sum_tree.rs:13`: `use ztracing::instrument`; its three
  instrumented methods use the imported attribute.
- `sum_tree/src/sum_tree.rs:1401`: `zlog::init_test()` exists only in
  sum_tree test code and is not in the target `--no-dev-dependencies` graph.
- No direct `ztracing_macro::` consumer was found.
- `gpui_apple`, `gpui_macos`, `gpui_platform`, and the other pinned Zed
  packages in the full host tree have no `ztracing` runtime callsites.

**Runtime result for the active host graph: zero.** There are no
`ztracing::debug_span!`, `info_span!`, `trace_span!`, `warn_span!`,
`error_span!`, `span!`, `event!`, or `ztracing::init()` calls in any active
host-graph package. The stub therefore needs only the attribute entry point;
it intentionally does not invent a runtime API.

### Whole-checkout runtime boundary

The exhaustive checkout scan did find runtime APIs in Zed crates that are not
in this host graph. They are listed here so this proof does not imply that the
stub can replace ztracing for an arbitrary Zed workspace:

| Runtime call | Pinned checkout locations |
| --- | --- |
| `ztracing::debug_span!` | `editor/src/display_map/block_map.rs:931,997,1141`; `editor/src/git/blame.rs:635`; `git/src/blame.rs:90`; `language/src/language_registry.rs:725` |
| `ztracing::info_span!` | `git_ui/src/diff_multibuffer.rs:667` |
| `ztracing::init()` | `zed/src/main.rs:309` |
| `zlog::init`, `init_output_stderr`, `init_output_stdout`, `init_output_file`, `scoped!`, `time!`, and logging macros (`trace!`, `debug!`, `info!`, `warn!`, `error!`) | Out-of-graph production/test consumers including `docs_preprocessor`, `language`, `onboarding`, `project` (including `lsp_store`), `watch`, `zed`, and `zlog_settings`; numerous additional test-only `init_test()` sites occur throughout the checkout. |

Those crates are not dependencies of `react-gpui-host` in the captured full
normal/build graph. If a future host feature or dependency introduces any of
them, this two-file stub is insufficient and the variant must either add an
independently authored compatible API or be rejected; no such API was hidden
or suppressed in this proof.

## Upstream no-op semantics

At the pinned upstream `crates/ztracing/build.rs:4-13`, `ZTRACING` and
`ZTRACING_WITH_MEMORY` are the only environment variables that enable cfgs;
without them, neither ztracing cfg is emitted. The upstream
`crates/ztracing/src/lib.rs:8-9` no-op branch re-exports
`ztracing_macro::instrument`. The pinned
`crates/ztracing_macro/src/lib.rs:1-6` is exact identity passthrough:

```rust
#[proc_macro_attribute]
pub fn instrument(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}
```

The local implementation independently reproduces this documented no-op
behavior: it discards the attribute arguments and returns the item token
stream unchanged. It does not copy upstream source. Because consumers write
`#[ztracing::instrument(...)]`, the stub is itself a proc-macro crate exporting
`instrument` at the `ztracing` root; this is the same consumer-facing path as
the upstream `ztracing` library's re-export, without retaining a
`ztracing_macro` dependency. No regular-library symbols are provided because
usage exhaustiveness found no active runtime calls.

## Stub and patch

The experiment created only these stub files:

| File | Lines | Contents |
| --- | ---: | --- |
| `ztracing-stub/Cargo.toml` | 9 | Package `ztracing` v0.1.0, authors `React GPUI contributors`, `Apache-2.0`, `proc-macro = true`. |
| `ztracing-stub/src/lib.rs` | 8 | Independent identity `#[proc_macro_attribute] instrument`. |
| **Total** | **17** | No upstream code; no dependencies. |

The only experimental manifest addition was:

```toml
[patch."https://github.com/zed-industries/zed"]
ztracing = { path = "ztracing-stub" }
```

Cargo accepted the URL-without-query key and applied the patch to the git
source package. The query-bearing key was not used. The experiment made no
source edits to GPUI or sum_tree; their existing attributes compile against
the local proc-macro unchanged.

## Graph and lock evidence

Before patching, the inverse trace showed the upstream package and both active
paths (normal GPUI and the `gpui_apple` build path):

```text
ztracing v0.1.0 (https://github.com/zed-industries/zed?...)
├── gpui ...
│   └── gpui_apple [build-dependencies] -> gpui_macos -> gpui_platform
└── sum_tree ... -> gpui
```

After patching:

```text
$ cargo tree -p react-gpui-host --target aarch64-apple-darwin \
    --no-dev-dependencies -i ztracing
ztracing v0.1.0 (proc-macro) (/private/tmp/stub-spike/ztracing-stub)
├── gpui v0.2.2 (https://github.com/zed-industries/zed?...)
│   └── ...
└── sum_tree v0.1.0 (https://github.com/zed-industries/zed?...)
    └── gpui ...
```

The exact final inverse commands produced these package-ID-not-found results:

```text
$ cargo tree -p react-gpui-host --target aarch64-apple-darwin \
    --no-dev-dependencies -i zlog
error: package ID specification `zlog` did not match any packages

$ cargo tree -p react-gpui-host --target aarch64-apple-darwin \
    --no-dev-dependencies -i ztracing_macro
error: package ID specification `ztracing_macro` did not match any packages
```

These exit 101 because Cargo could not find the requested package IDs; they
are the absence proof, not hidden empty trees. The `ztracing` inverse trace
resolved to the local path shown above. In `Cargo.lock`, the only remaining
record is:

```toml
[[package]]
name = "ztracing"
version = "0.1.0"
```

It has no `source` and no dependency list. There is no `name = "zlog"` or
`name = "ztracing_macro"` package record. The former registry tracing support
records pulled only by upstream ztracing (`tracing-subscriber` and its
transitives) were also removed where no longer reachable.

## Build and smoke evidence

All commands ran in `/tmp/stub-spike`; no live gallery process was touched.

| Command | Result |
| --- | --- |
| `cargo check -p react-gpui-host` | **passed** (32.86 s). Cargo reported only the existing future-incompatibility warning for `block v0.1.6`. |
| `cargo build -p react-gpui-host --release` | **passed** (65.45 s), with the same non-fatal future-incompatibility warning. |
| `./target/release/react-gpui-host --version` | **passed**: `react-gpui-host 0.2.0 protocol=v3`. |
| `cargo test -p react-gpui --lib` | **passed**: 135 tests, 1 suite, 18.41 s. |

The library tests exercise the existing renderer/rich/SVG paths while all
upstream instrument attributes are compiled through the local identity macro.

## Stub versus two-crate fork

The prior fork proof recorded 181 changed lines in two copied Zed crates (81
additions and 100 deletions), including 69 GPUI manifest declarations and 10
sum_tree declarations translated out of workspace inheritance, plus three
source files with 11 instrumentation deletions. Its host build and smoke test
passed, but the proof exceeded its nominal time box and left a medium/high
ongoing upstream-sync burden.

| Dimension | Option 3′ local stub | Prior two-crate fork |
| --- | --- | --- |
| Change surface | 2 new files / 17 lines; one source patch and lock regeneration in the experiment; no GPUI/sum_tree source copies | Two copied Zed crates, five changed source/manifest files, 181 changed lines, standalone workspace-declaration translation |
| Runtime behavior | Preserves every existing attribute and exactly matches upstream no-op identity semantics; no runtime API invented | Deletes the active attributes/imports and their dependency edges, so the same code runs without instrumentation syntax at those sites |
| Maintenance | Re-audit host graph/API use when pins/features change; no tracing implementation to sync; reject or extend independently if runtime calls enter the graph | Rebase/reapply source edits, preserve upstream notices, reconcile manifests and all transitive pins, monitor API/security changes, refresh lock and target matrix on each Zed update |
| Legal posture | The replacement distributes zero Zed tracing code: it is independent project-authored Apache-2.0 code. Other Zed crates and the two absent-license fields remain separate obligations | Distributes modified Zed Apache-licensed GPUI/sum_tree source and must retain upstream notices while obtaining a legal treatment for project-authored fork changes |

**Engineering comparison:** for this host graph, the stub is the better
Option 3 variant: it is dramatically smaller, avoids an upstream-code fork and
ongoing fork synchronization, and has the stronger narrow legal posture of
distributing zero Zed code in the substituted package. This is not a legal
conclusion or a release-owner decision. The GPL trio's technical graph removal
does not clear `gpui_shared_string`, `gpui_util`, `self_cell`, weak-copyleft
entries, notices, or artifact-level review obligations.

## Limits and cleanup

- This is a target-qualified host proof for `aarch64-apple-darwin`; it is not a
  claim that arbitrary Zed workspace members can build against the stub.
- The entire pinned checkout was scanned, and out-of-graph runtime users are
  explicitly listed above. The current active host graph has zero runtime
  ztracing calls.
- The local stub, root patch, and generated lockfile existed only in the
  throwaway worktree. Only this note and the two-line issue pointer are copied
  into the main tree.
- `/tmp/stub-spike` was removed after the main-tree documentation commit.
