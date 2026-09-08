# Continuous integration

GitHub Actions separates development checks, dependency audits, website deployment,
and release qualification. All workflows can also be started manually.

## Automatic checks

| Workflow | Automatic trigger | Coverage |
| --- | --- | --- |
| [CI](../.github/workflows/ci.yml) | Pull requests and pushes to `main` changing source, fixtures, build configuration, or Actions | One macOS job runs `bun run ci`: generated contracts, Rust formatting/checks/Clippy/tests, package formatting/types/tests, and package installation smoke tests. |
| [Cross-platform host](../.github/workflows/cross-platform.yml) | Pull requests and pushes to `main` changing native host or renderer inputs | Linux Clippy and library fixtures; Windows workspace checks and a linked process host. |
| [Dependency audit](../.github/workflows/audit.yml) | Dependency manifests, locks, audit configuration, or notice inputs; Mondays at 03:37 UTC | Bun and Rust advisories, plus generated third-party notice verification on macOS. |
| [GitHub Pages](../.github/workflows/pages.yml) | Website, documentation, branding, SDK, Rust, or build inputs | WASM build, website types and tests; deployments only from `main`. |
| [Embedded Bun](../.github/workflows/embedded-bun.yml) | Embedded runtime, host lifecycle, embedding fixtures, or toolchain/dependency inputs | Embedded VM lifecycle tests and Clippy on macOS 26. |

Changes confined to the website's `docs/*.md` guides run the website workflow.
Agent notes and reference checkouts do not trigger builds unless a listed build
input also changes.
Path filters live in each workflow; YAML anchors keep its push and pull request
filters identical. If branch protection adds required checks, account for
[path-filtered workflows](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onpushpull_requestpull_request_targetpathspaths-ignore),
which do not report a completed check when the entire workflow is skipped.

The macOS CI job checks Rust formatting before compilation and shares package
generation and Cargo compilation between Rust and Bun checks. Linux Clippy
already checks all targets, so it does not repeat
`cargo check` or platform-independent formatting. Windows still checks the
workspace and links the host to exercise its shader compiler and linker.
Display and GPU qualification remain separate; see [distribution](distribution.md).

Browser website types and tests run in Pages after `bun run website:build` has
generated the WASM module. The SDK package gate does not require pre-existing
website build products or repeat those checks. The component example checker
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

## Release qualification

Release builds and archives run on demand:

- [Website Packages](../.github/workflows/website-packages.yml) builds and verifies
  native archives on macOS ARM64, Linux x86-64, and Windows x86-64.
- [Host Release Candidate](../.github/workflows/host-release-candidate.yml) runs
  development checks and audits, then builds and smokes the extracted process host.
- [Release Prep](../.github/workflows/release-prep.yml) synchronizes a candidate
  version, runs checks and audits, and uploads the three npm package tarballs.
- Run Embedded Bun with its `candidate` input enabled to also rehearse the
  embedded release host. The same job first runs the embedded feature checks.

These workflows upload candidates without publishing releases. Already compressed
archives are uploaded without another compression pass.

`bun run ci` is the development gate. Audits and release qualification are explicit
commands so normal development does not build unused release archives:

```sh
bun run ci
bun run audit
bun run task host-candidate-smoke
bun run task embedded-check
bun run task website-package
```

## Caching and verification

The shared [Rust setup action](../.github/actions/setup-rust/action.yml) selects
the pinned toolchain before restoring a
[Rust dependency cache](https://github.com/Swatinem/rust-cache). Cache keys include
the runner image, architecture, build purpose, installed compilers, Cargo
configuration, and dependency manifests/locks. Pages installs its pinned Web
nightly before computing the key. It does not create a new cache for every commit.

The action prunes workspace build products and incremental state before saving;
CI also disables Cargo incremental compilation. Pull requests restore caches;
pushes and manual runs can save them, including successfully compiled dependencies
from a failed check. Audit jobs cache only the registry. Candidate build caches
are isolated from development and WASM caches.

Embedded Bun uses `SOLID_GPUI_BUN_CACHE` outside Cargo's `target` directory to share
its native build graph across checks, Clippy, and release profiles. Its separate
cache requires an exact toolchain and embedding-source match. Generic Cargo cache
cleanup cannot remove that graph.

The embedded job installs Homebrew `llvm@21` before restoring build caches and
puts its binaries on `PATH`. The pinned Bun source requires LLVM 21.1; the
Apple Clang shipped with Xcode is a different toolchain. The native cache key
includes the LLVM version and workflow so compiler changes invalidate the build graph.

`cargo-deny` and `wasm-bindgen-cli` use fixed, checksum-verified prebuilt versions
through [install-action](https://github.com/taiki-e/install-action), with source
installation fallback disabled. Bun keeps setup-bun's executable cache. Its
package cache is not archived: in the
[inspected CI run](https://github.com/Cyenoch/solid-gpui/actions/runs/34180146937),
restoring it took five seconds while an uncached workspace install took four seconds.

After workflow changes, run `actionlint`, `bun test scripts/task-contract.test.ts`,
and the relevant [website checks](../examples/website/README.md). Compare cache
restore/save time, build time, and total runner minutes on subsequent GitHub runs.
Local validation cannot establish hosted runner speedups or cross-platform
release qualification.
