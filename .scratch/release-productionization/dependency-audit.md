# Dependency and version governance audit

Generated: 2026-08-26 (current protocol-docs HEAD `5ca34e1`)
Scope: Cargo workspace, both Bun packages, lockfiles, audit/outdated tooling,
and toolchain pins. This is a human release decision document; it does not
publish, push, rebase, or upgrade a major/toolchain pin.
- Companion evidence: `release-readiness.md`, `v0.2.0-cut-checklist.md`, and
  `upstream-dependencies.md`; their current feature and boundary statements
  agree with this dependency inventory.
- Current release facts agree with the readiness reports: **22 commands / 21
  events**, API locks **core 102 / dev 18**, and **96 Unreleased entries
  (87/4/5)**; selectable Text is a tracked feasible-bounded design, not shipped.

## Executive summary

- Cargo.lock contains **672 packages**: **643 registry**, **26 git**, and **3
  path** packages. The workspace has three path members.
- There are **13 direct Cargo dependency declarations** across the members:
  `react-gpui` has 5 runtime plus 1 dev declaration, `react-gpui-host` has 5,
  and `react-gpui-bun` has 2. The six unique external direct names are
  `futures`, `gpui`, `gpui-platform`, `rmp-serde`, `serde`, and `thiserror`;
  the remaining direct edges are workspace path crates or the duplicate dev
  `serde` edge.
- The GPUI family remains pinned to Zed revision
  `6805d952f9f3d702f760aa11b1547df8a625fa16`: `gpui = 0.2.2` and
  `gpui_platform = 0.1.0`. The pin was not moved.
- `block 0.1.6` remains in the Apple GPUI/Cocoa/Core Video/Metal dependency
  chain and still emits the known Rust future-incompatibility warning. This is
  the existing issue 02 decision, not a vulnerability conclusion.
- The only dependency refresh executed and committed is `e2c1881`:
  `@msgpack/msgpack` `3.0.0 -> 3.1.3` in both packages, Cargo lock
  `core-foundation` `0.10.0 -> 0.10.1`, and `rand` `0.8.7 -> 0.8.8`.
- `@types/react` `19.2.2 -> 19.2.18` and `bun-types` `1.1.29 -> 1.4.0` were
  attempted, but rolled back after real typecheck regressions (children,
  `onKeyDown`, and style prop typing). No incompatible upgrade was left in the
  tree.
- `cargo audit` is **not installed** (`cargo: no such command: audit`), so this
  audit makes **no vulnerability claims**. The lockfile and known
  future-incompatibility chain are recorded, but advisory coverage requires
  installing/running cargo-audit or an equivalent approved scanner.
- Current validation after the image fallback, scale-factor, keybinding, and
  outbound file-drag implementation: `make ci` exit 0, **120 Rust tests**,
  **92 core Bun tests / 54,065 assertions**, **16 dev tests / 39 assertions**;
  `make embedded-bun` exit 0 with **1 embedded test**. The latest
  `/usr/bin/time -p` wall times were `make ci` **20.35 s** and
  `make embedded-bun` **3.86 s**. The serial candidate smoke and package gates
  remain recorded in the companion readiness report.

## 1. Cargo workspace inventory

### Manifest edges

| Crate | Runtime/build direct declarations | Dev declarations |
| --- | --- | --- |
| `react-gpui` | `futures.workspace`, `gpui.workspace`, `rmp-serde.workspace`, `serde.workspace`, `thiserror.workspace` | `serde` with derive |
| `react-gpui-host` | `futures.workspace`, `gpui.workspace`, git-pinned `gpui-platform`, path `react-gpui`, optional path `react-gpui-bun` | none |
| `react-gpui-bun` | path `react-gpui`, `thiserror.workspace` | none |

Workspace declarations are exact: `futures = "0.3"`, `gpui = "0.2.2"`,
`rmp-serde = "1.3"`, `serde = "1.0"` with derive, and `thiserror = "2.0"`.
The host has an explicit `gpui-platform` git dependency at the same Zed
revision as the `gpui` patch.

### Locked direct versions and source strategy

| Direct name | Locked version | Source | Current/latest observation | Risk |
| --- | ---: | --- | --- | --- |
| `futures` | 0.3.34 | crates.io | `cargo info futures` reports 0.3.34 | Low; no update |
| `gpui` | 0.2.2 | Zed git rev `6805d952` | Pin is deliberate; no semver latest comparison is meaningful | High; rev update changes the patched GPUI graph; human-only |
| `gpui-platform` | 0.1.0 | same Zed git rev | Pin is deliberate; no independent latest comparison | High; must move with GPUI rev/features; human-only |
| `rmp-serde` | 1.3.1 | crates.io | `cargo info` reports 1.3.1 | Low; no update |
| `serde` | 1.0.229 | crates.io | `cargo info` reports 1.0.229 | Low; no update |
| `thiserror` | 2.0.20 (also transitive 1.0.69) | crates.io | `cargo info` reports 2.0.20 | Low direct; duplicate transitive major is graph context |
| `react-gpui`, `react-gpui-bun` | workspace 0.1.0 | path | local members | Low; coordinated workspace edges |

`cargo update --workspace --dry-run --verbose` reports no compatible direct
updates. Its remaining transitive drift is:

- `cocoa 0.26.0` available `0.26.1`, but the pinned `gpui_macos` requires
  `cocoa = "=0.26.0"`; `cargo update -p cocoa@0.26.0 --precise 0.26.1`
  correctly fails. Do not override the GPUI pin to force it.
- `generic-array 0.14.7` available `0.14.9`, but the locked
  `crypto-common 0.1.7` path in GPUI's graph requires `=0.14.7`; a precise
  update correctly fails. This is likewise coupled to the pinned graph.
- `core-foundation 0.10.0 -> 0.10.1` and `rand 0.8.7 -> 0.8.8` were precise
  patch updates and are included in the committed lock refresh.

### GPUI pin and future-incompatibility status

The lock graph traces `block 0.1.6` through Cocoa/Core Foundation/Core Video,
Metal, and the Zed GPUI Apple path into `react-gpui-host`. `cargo check` and
`cargo clippy` continue to emit the future-incompatibility warning. This is
tracked in `.scratch/release-productionization/issues/02-block-future-incompat.md`.
An unsigned third-party fork remains intentionally unpatched; resolution
requires an approved upstream/GPUI update or maintainer-approved fork.

## 2. Bun package inventory

### `@react-gpui/core`

- Runtime dependencies: `@msgpack/msgpack 3.1.3` and `react-reconciler 0.33.0`.
- Peer dependency: `react ^19.2.0`; development install is `react 19.2.8`.
- Dev dependencies: `@types/react 19.2.2`, `bun-types 1.1.29`, `prettier
  3.6.2`, `typescript 5.9.3`.
- `react-reconciler`, React, and `react-refresh` queried latest are already at
  current observed versions (`react-reconciler 0.33.0`, React `19.2.8`, and
  `react-refresh 0.18.0` where applicable).

### `@react-gpui/dev`

- Runtime dependencies: `@babel/core 7.28.4`,
  `@babel/plugin-transform-react-jsx 7.27.1`, `@msgpack/msgpack 3.1.3`, and
  `react-refresh 0.18.0`.
- Peer dependency: `react ^19.2.0`; development install is `react 19.2.8`.
- Dev dependencies: local `@react-gpui/core` via
  `file:../react-gpui`, `@types/react 19.2.2`,
  `@types/react-test-renderer 19.1.0`, `bun-types 1.1.29`, `prettier 3.6.2`,
  `react 19.2.8`, `react-test-renderer 19.2.8`, and `typescript 5.9.3`.
- The file link is intentional. Bun 1.4 left stale nested link metadata after
  the core package's MessagePack bump; the dev lock was aligned to the local
  core's `@msgpack/msgpack 3.1.3` metadata and then verified with
  `bun install --frozen-lockfile`.

### Bun outdated results and risk table

Fresh `bun outdated` results were run independently in both package
workspaces. `bun pm view` confirmed omitted packages that were already current.

| Package | Kind | Current | Latest | Risk / decision |
| --- | --- | ---: | ---: | --- |
| `@msgpack/msgpack` (both) | runtime | 3.1.3 | 3.1.3 | Low; upgraded from 3.0.0 and validated |
| `react-reconciler` | runtime | 0.33.0 | 0.33.0 | Low; no update |
| `react` | peer/dev | 19.2.0 range / 19.2.8 dev | 19.2.8 | Low; peer range remains React 19; future major human-only |
| `@babel/core` | runtime dev package | 7.28.4 | 8.0.1 | High; major, evaluate separately |
| `@babel/plugin-transform-react-jsx` | runtime dev package | 7.27.1 | 8.0.1 | High; major, evaluate separately |
| `react-refresh` | runtime dev package | 0.18.0 | 0.18.0 | Low; no update |
| `@types/react` (both) | dev | 19.2.2 | 19.2.18 | Medium; attempted patch caused current JSX/prop typing regressions; rolled back |
| `@types/react-test-renderer` | dev | 19.1.0 | 19.1.0 | Low; no update |
| `react-test-renderer` | dev | 19.2.8 | 19.2.8 | Low; no update |
| `bun-types` (both) | dev | 1.1.29 | 1.4.0 | Medium; attempted minor caused widespread JSX/style typing regressions; rolled back |
| `prettier` (both) | dev/tool | 3.6.2 | 3.9.6 | Medium; formatter behavior can churn source; defer to toolchain maintenance |
| `typescript` (both) | dev/tool | 5.9.3 | 7.0.2 | High; major, human-reviewed migration |
| `@react-gpui/core` | local dev file link | 0.1.0 workspace | local workspace | Low operationally; lock metadata must track the local manifest |

`bun outdated` does not provide advisory scanning. No vulnerability conclusion
is made for any Bun package.

## 3. Executed upgrades and validation

### Committed refresh

Commit `e2c1881 chore: refresh dependency patch levels` contains only the
following dependency/lock changes:

1. `@msgpack/msgpack 3.0.0 -> 3.1.3` in both package manifests and locks.
2. Cargo lock `core-foundation 0.10.0 -> 0.10.1`.
3. Cargo lock `rand 0.8.7 -> 0.8.8`.
4. Dev file-link lock metadata aligned to core's MessagePack 3.1.3.

Each attempted product dependency refresh was followed by a full `make ci`
when the source tree was stable. The final post-refresh run passed:

- `make ci`: exit 0, **20.14 s** real time; Rust **86 tests** (61 crate
  unit + 6 boundary + 1 perf + 3 process + 1 fuzz + 3 golden + 11 host),
  core Bun **82 tests / 3,954 assertions**, dev Bun **14 tests / 35
  assertions**.
- `make embedded-bun`: exit 0, **3.71 s** real time; embedded counter **1
  test passed**.
- Rust future-incompatibility warnings remain limited to the known `block
  0.1.6` issue; no new warnings were introduced by these refreshes.

### Rolled-back attempts (intentional)

- `@types/react 19.2.2 -> 19.2.18`: reverted after the real typecheck exposed
  missing `children`/`onKeyDown` props and style-prop incompatibilities across
  examples/tests. The stable 19.2.2 lock is retained.
- `bun-types 1.1.29 -> 1.4.0`: reverted after the real typecheck exposed the
  same broad JSX children/style type failures. Bun runtime remains 1.4.0,
  but the package's historical type surface is not interchangeable without a
  separate typing migration.

No major upgrade was attempted.

## 4. Audit tooling and toolchain

- `cargo audit --version` was attempted and is unavailable (`cargo` reports no
  `audit` subcommand). Install an approved `cargo-audit` version and run it in
  a controlled audit environment before publication; this report deliberately
  does not invent advisory results.
- `bun outdated` was available and run in each package. It reports freshness,
  not vulnerability advisories.
- `rust-toolchain.toml` pins Rust `1.97.1` with minimal profile, rustfmt, and
  clippy. `rustup check` reports `1.98.0` available (2026-08-18). Do not upgrade
  in this change; it changes the CI/release matrix and is a human toolchain
  decision.
- `.bun-version` pins Bun `1.4.0`. `bun upgrade --dry-run` reports that Bun
  `1.4.0` is already the latest available Bun version. No Bun toolchain change
  is needed now.

## 5. Recommendations and decision gates

### Safe/near-term maintenance

- Keep the committed MessagePack/Cargo patch refresh and retain the full CI
  evidence above.
- Add `cargo-audit` (or an approved equivalent) to a controlled security gate;
  do not infer a clean advisory report from its absence.
- Consider a dedicated toolchain window for Rust `1.98.0`, then rerun the full
  matrix, candidate smokes, and embedded gate.

### Human-only upgrade domains

- Any GPUI Zed revision update must be treated as a coupled `gpui` /
  `gpui-platform` / patched dependency graph change. It can resolve Cocoa,
  generic-array, and `block` constraints, but it may change GPUI API and the
  pinned Bun/macOS build graph. This belongs with issue 02 and the release owner.
- Babel 8 and TypeScript 7 are major upgrades; evaluate in isolated branches
  with a deliberate generated-code/type policy.
- React major changes are outside the current `^19.2.0` peer contract; do not
  infer compatibility from a patch-level React install.
- `@types/react` and `bun-types` need a coordinated renderer typing migration
  before a future refresh; the attempted versions were not drop-in compatible.
- Prettier changes should be isolated because formatter output is itself a CI
  contract; upgrade only with a reviewed formatting diff.

No release approval or major upgrade decision is made here.
