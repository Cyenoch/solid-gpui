# Continuous integration

GitHub Actions separates development checks, dependency audits, website deployment,
and release qualification. All workflows can also be started manually.

## Automatic checks

| Workflow                                                       | Automatic trigger                                                                             | Coverage                                                                                                                                   |
| -------------------------------------------------------------- | --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| [CI](../.github/workflows/ci.yml)                              | Pull requests and pushes to `main` changing source, fixtures, build configuration, or Actions | Independent macOS native check/test lanes and Linux SDK package checks; `Rust and Bun checks` requires every lane to succeed. |
| [Cross-platform host](../.github/workflows/cross-platform.yml) | Pull requests and pushes to `main` changing native host or renderer inputs                    | Linux Clippy and library fixtures; Windows workspace checks and a linked process host.                                                     |
| [Dependency audit](../.github/workflows/audit.yml)             | Dependency manifests, locks, audit configuration, or notice inputs; Mondays at 03:37 UTC      | Bun and Rust advisories, plus generated third-party notice verification on macOS.                                                          |
| [GitHub Pages](../.github/workflows/pages.yml)                 | Website, documentation, branding, SDK, Rust, or build inputs                                  | WASM build, website types and tests; deployments only from `main`.                                                                         |
| [Embedded Bun](../.github/workflows/embedded-bun.yml)          | Embedded runtime, host lifecycle, embedding fixtures, or toolchain/dependency inputs          | Rust integration Clippy across all targets and embedding overlay formatting on macOS 15; no Bun compilation.                               |

Changes confined to the website's `docs/*.md` guides run the website workflow.
Agent notes and reference checkouts do not trigger builds unless a listed build
input also changes.
Path filters live in each workflow; YAML anchors keep its push and pull request
filters identical. If branch protection adds required checks, account for
[path-filtered workflows](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onpushpull_requestpull_request_targetpathspaths-ignore),
which do not report a completed check when the entire workflow is skipped.

The macOS matrix has two independent lanes. `native-check-ci` owns Rust formatting,
generated protocol checks, default-feature workspace compilation, strict QuickJS
Clippy, and native binding consistency. `native-test-ci` owns cross-language
goldens and all workspace runtime tests, including the patched HTTP client.
Neither lane cancels its sibling on failure. The Linux `package-ci` job owns
package formatting, types, tests, and packed-consumer smoke checks; its small
native-tooling fixtures do not compile GPUI. The `Rust and Bun checks` aggregate
runs even after failures or skips and accepts only successful native matrix and
package results. Keep that status in branch protection.

Local `native-ci` composes both native lanes sequentially; `bun run ci` also runs
the package gate and shares the memoized package build. Separate task processes
must not build packages concurrently in one checkout: their `dist` cleanup is
not coordinated. Hosted lanes use separate workspaces and cache purposes.

Rust-hosted QuickJS VM tests compile their TSX through
`fixtures/vite.quickjs-test.config.ts`, with no external native host. They retain
real Vite compilation and real QuickJS execution without nested Cargo builds.
The managed `fixtures/vite.config.ts` still builds/exports the host for development
and the Bun launch integration. Native preparation/export behavior remains covered
by the tooling tests and native binding gate; no runtime assertions are skipped.

Watcher fixtures use `scripts/watch-fixture.ts` for distinct application saves.
Vite's bundled watcher coalesces `change` events within 50ms; fast Linux inotify
delivery can otherwise merge a test's next edit even after HMR has completed.
The helper separates synthetic saves by 100ms. It does not change application
watcher settings, retry failed checks, increase test timeouts, or remove assertions.

The macOS default-feature `cargo check` is intentional: QuickJS Clippy enables a
different dependency feature set and cannot validate consumers that must compile
without QuickJS. Linux Clippy already checks all targets with the default feature
set, so that job does not repeat `cargo check` or platform-independent formatting.
Windows still checks the workspace and links the host. Display and GPU
qualification remain separate; see [distribution](distribution.md).

Pages restores independent exact-input caches for WASM and generated SDK bindings;
only the producer whose cache misses runs. It always runs `build:frontend` and browser tests.
The SDK package gate does not require website build products or repeat those
checks. The component example checker
resolves types from the website's tsconfig, so it also works when invoked from
the repository root with Bun's isolated dependency layout.

Linux portal dependencies explicitly select Ashpd's `async-io` backend to match
GPUI. Enabling Ashpd's default Tokio backend at the same time fails compilation.
The host HTTP adapter and browser asset downloader use upstream Reqwest, keeping
the unmaintained `rustls-pemfile` dependency out of the root lockfile. macOS CI and
Linux library checks also run the patched HTTP adapter's local-server tests for
redirect policies, streamed bodies, timeouts, and proxy configuration.

The audit regenerates `THIRD-PARTY-NOTICES.md` from Cargo metadata,
`cargo-deny`'s JSON license inventory, and Bun's license inventory. Regenerate it
with `bun run task third-party-notices` after changing dependencies. Every local
workspace package must declare its license, normally with `license.workspace = true`.
Vendored GPUI crates must also declare their upstream package URL in
`package.metadata.solid-gpui-vendor.source` in `Cargo.toml`; a prose provenance
record alone does not populate the generated inventory.

The inventory has no generation timestamp: identical resolved dependencies and
release metadata must produce identical notices before and after a commit.
Changes to commit dates or unrelated task definitions must not invalidate it.
Run `bun run audit` after regenerating; package tests alone do not verify the
notice inventory.

## Release qualification

Release builds and archives run on demand:

- [Website Packages](../.github/workflows/website-packages.yml) builds and verifies
  native archives on macOS ARM64, Linux x86-64, and Windows x86-64.
- [Host Release Candidate](../.github/workflows/host-release-candidate.yml) runs
  development checks and audits, then builds and smokes the extracted process host.
- [Release Prep](../.github/workflows/release-prep.yml) synchronizes a candidate
  version, runs checks and audits, then uses one `sdk-pack` invocation to build
  and upload the core, Vite, router, and Shiki tarballs. The version-labelled
  artifact contains `solid-gpui-{core,vite,router,shiki}.tgz`.
- Run Embedded Bun with its `candidate` input enabled to also rehearse the
  embedded release host. After the lightweight checks, a separate macOS 26 job
  builds Bun and runs the real VM lifecycle tests before the release smoke.

These workflows upload candidates without publishing releases. Already compressed
archives are uploaded without another compression pass.

`bun run ci` is the development gate. Audits and release qualification are explicit
commands so normal development does not build unused release archives:

```sh
bun run ci
bun run audit
bun run task host-candidate-smoke
bun run task embedded-check
bun run task embedded-test # Builds Bun; requires the macOS 26 SDK and LLVM 21.1.
bun run task website-package
```

`embedded-check` sets `SOLID_GPUI_BUN_CHECK_ONLY=1` only for Clippy. The sys crate
then exposes the real Rust FFI declarations without building or linking Bun.
No replacement symbols or VM mocks are supplied. This checks host-side types and
lints, including test targets; it cannot verify ABI compatibility with Bun,
patch applicability, native linking, or VM behavior. The overlay formatting check
parses those Rust sources but does not type-check them against Bun. Use
`embedded-test` and the manual candidate workflow for those runtime guarantees.
Do not set the check-only variable for executable builds.

## Caching and verification

The shared [Rust setup action](../.github/actions/setup-rust/action.yml) selects
the pinned toolchain before restoring a
[Rust dependency cache](https://github.com/Swatinem/rust-cache). Cache keys include
the runner image, architecture, build purpose, installed compilers, Cargo
configuration, and dependency manifests/locks. Whenever either Pages artifact
misses, Pages installs the pinned Web nightly before computing the Cargo key,
including bindings-only misses, keeping the installed-toolchain identity stable.
Cargo dependency caches do not create a new entry for every source commit.

The action prunes local workspace/vendor build products and incremental state
before saving; CI also disables Cargo incremental compilation. PRs restore
caches; only successful jobs on pushes and manual runs can save them. Exact cache hits
are immutable: a failed or check-only build must not seed the cache used by full
compile/test jobs. Native check, native test, and Embedded Bun therefore have
separate cache purposes even on macOS 15. Audit jobs cache only the registry.
Candidate build caches are isolated from development and WASM caches.

Pages caches `examples/website/src/wasm` and generated
`packages/solid-gpui/src/components.ts` separately, with no fallback restore keys.
The WASM key includes the complete Rust, Cargo, toolchain, embedded-asset and
WASM build-script inputs, but excludes Bun locks and TypeScript exporter sources.
The bindings key adds Bun/package/formatter/exporter inputs and the committed
catalog. A compiler-only update therefore cannot invalidate unchanged WASM.
Native system packages are needed only for bindings; wasm-bindgen only for WASM.
Both hits skip Rust setup entirely. Frontend compilation, type checking and tests
always run; only successful non-PR runs save artifact caches. Keep both input lists
in `pages.yml` current; see [Web deployment](web.md#github-pages).

The lightweight embedded job skips SDK package builds, native binding generation,
LLVM installation, and native graph downloads. Its isolated Cargo cache contains
the `embedded-bun` Clippy graph, not the native CI test/link graph.

The manual embedded candidate uses `SOLID_GPUI_BUN_CACHE` outside Cargo's `target` directory to share
its native build graph across checks, Clippy, and release profiles. Its separate
cache requires an exact toolchain and embedding-source match. Generic Cargo cache
cleanup cannot remove that graph.

The candidate job installs Homebrew `llvm@21` before restoring build caches and
puts its binaries on `PATH`. The pinned Bun source requires LLVM 21.1; the
Apple Clang shipped with Xcode is a different toolchain. The native cache key
includes the LLVM version and workflow so compiler changes invalidate the build graph.

`cargo-deny` and `wasm-bindgen-cli` use fixed, checksum-verified prebuilt versions
through [install-action](https://github.com/taiki-e/install-action), with source
installation fallback disabled. Bun keeps setup-bun's executable cache. Its
package cache is not archived: in the
[inspected CI run](https://github.com/Cyenoch/solid-gpui/actions/runs/34180146937),
restoring it took five seconds while an uncached workspace install took four seconds.

Do not equate a cache hit with avoided compilation. The measured baseline restored
an exact 420 MiB macOS entry but still compiled 412 crates for the first protocol
example alone. Compare compiler logs and cache size as well as hit rate. Prefer
separate complete dependency graphs over adding another compiler cache or caching
every commit's `target` tree; the repository already approached the default cache
storage limit during the baseline measurement.

See the [optimization research](../.scratch/ci-optimization/research.md) for source
links, measured baseline, retained coverage, and hosted verification results.

After workflow changes, run `actionlint`, `bun test scripts/task-contract.test.ts`,
and the relevant [website checks](../examples/website/README.md). Compare cache
restore/save time, build time, and total runner minutes on subsequent GitHub runs.
Local validation cannot establish hosted runner speedups or cross-platform
release qualification.
