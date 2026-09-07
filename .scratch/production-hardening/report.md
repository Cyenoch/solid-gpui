# Production hardening report

Date: 2026-09-07

## Changes and reasoning

| Area                | Failure or unnecessary cost                                                                                       | Result                                                                                                                                                                          |
| ------------------- | ----------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| JSX compiler        | Separate Babel integration and mismatched compiler/preset dependencies                                            | One official Solid/Oxc universal transform shared by Bun, Vite, and application bundling; TypeScript lowering and composed source maps remain explicit.                         |
| Formatting          | Direct Prettier tooling remained after the compiler migration; an unused generated-file formatter had no callers | Pinned Oxfmt replaces formatter commands, preserves existing style without import/package sorting, and removes the unused helper.                                               |
| Renderer ownership  | An unread strong node registry retained recycled rows; duplicated listener/input state had no authoritative role  | Removed the redundant state. The listener registry owns callbacks, including layout/file-drop changes and immutable text-input callback revisions.                              |
| Native transactions | A rejected compound Patch could leave created nodes or derived ancestor text behind                               | First-write transactional backups restore both primary and derived state. Sibling reindexing touches only the changed suffix and avoids cloning a vector for each index update. |
| Native validation   | Listener-only Updates could bypass interaction validation                                                         | Snapshot, Create, and merged Update validate the same interaction contract.                                                                                                     |
| Commit scheduling   | A continuously ready input could monopolize a foreground poll                                                     | Each poll yields after 16 messages or 1 MiB, preserving frame/revision/termination order. One indivisible legal frame can exceed the byte threshold.                            |
| Frame decoding      | Recursive coalesced-frame decoding and repeated concatenation could overflow the JS call/argument stack           | Iterative incremental decoding buffers only incomplete frames and returns complete incoming frames without copying.                                                             |
| Routing             | Native Link/BackButton props captured values too early                                                            | Reactive destination, disabled, tooltip, and style values survive the native component boundary.                                                                                |
| Transport ownership | Importing the core also imported process/file I/O; application cleanup depended on a cast to an optional disposer | Process transport has its own `/stdio` entry. `mountApplication` owns an explicitly created disposable transport and retains it across HMR. QuickJS uses `/embedded`.           |
| Release preparation | Already-synchronized versions skipped gates; a late failure could leave release metadata partially updated        | Every invocation checks the changelog and locked dependencies. All six release files are backed up before mutations and restored on failure or interruption.                    |
| Language            | Maintained guides and historical design evidence mixed English and Chinese                                        | English prose throughout maintained code/docs/skills and tracked historical reports; deliberate Unicode input and IME cases remain.                                             |

The compiler decision, primary sources, compatibility probe, and measured warm
transform cost are in [compiler.md](compiler.md). Its observed 5.5× improvement
applies to the compiler stage on this machine, not native rendering or startup.

## Runtime model

[ADR-0017](../../docs/adr/0017-runtime-engines.md) records the ownership decision:
Bun supports Rust-led and Bun-led applications; QuickJS is an embedded UI engine
for Rust-led applications. Rust always owns GPUI. Both engines use the same
framed protocol, surface ownership, and generated native command contracts.

QuickJS owns its VM on one dedicated worker. Only owned bytes cross threads.
Each transport direction is bounded by 4,096 frames and one maximum legal frame's
byte budget. The VM has a 256 MiB allocation limit, a 2 MiB JS stack limit, and an
8 MiB worker stack. Cancellation interrupts busy JavaScript and recursive
microtask chains; transport termination callbacks have a bounded cleanup window.
The QuickJS C interpreter is optimized in debug builds too; production bundles
select Solid’s production client runtime. Timers use a heap with logarithmic cancellation. Diagnostic file writes occur on
the worker rather than the foreground event sender.

`solid-gpui-build --runtime quickjs` bundles the Solid client graph and initializes
the explicit UI platform before application dependencies. It rejects Node/Bun
imports and unresolved dynamic imports rather than accepting browser-target
stubs. `--runtime bun` preserves Bun services. Failed builds do not overwrite the
previous output. Neither build requires Babel. The required routing primitives,
audited upstream source, and deliberate bodyless Response limit are documented in
[quickjs-platform.md](quickjs-platform.md).

## Verification

Validation environment: macOS ARM64, Bun 1.4.2, Rust 1.98.1. The working baseline
was commit `06cdeabfc541dd0bb396c27ea6c7413f897f4698`.

| Check                                                                                   | Result                                                                                                                                                                                                                                                            |
| --------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `bun run task package-ci`                                                               | Passed: 78 Bun tests, package/tool/fixture/Gallery type checks, formatting, native binding regeneration, and packed core/router consumers.                                                                                                                        |
| Packed application CLI                                                                  | Both runtime targets build from installed tarballs; a production Bun router bundle executes and preserves independent per-window navigation with shared application state.                                                                                        |
| `cargo test --workspace --locked --features solid-gpui/quickjs,solid-gpui/test-support` | Passed: 287 tests; one documentation example ignored. Includes all six real QuickJS tests and 17 native command integration tests.                                                                                                                                |
| Rust checks                                                                             | Default-feature workspace checking, strict Clippy with QuickJS/test-support enabled, and workspace rustfmt passed.                                                                                                                                                |
| Protocol generation and goldens                                                         | Generated schema/bindings checks and both Rust/TypeScript producer-consumer directions passed.                                                                                                                                                                    |
| Native Gallery audit                                                                    | All 23 routes passed at 800 and 560 pixel initial widths, light/dark themes, and retained resize/scroll stages. These are native regression measurements, not production display FPS.                                                                             |
| Advisory and notice audit                                                               | Bun checked 127 packages with no reported vulnerabilities. Cargo advisory checks include every workspace feature under the existing exception policy. Regenerated notices match exactly and include optional QuickJS dependencies and vendored source provenance. |
| Host release archive                                                                    | Release build, extracted help/version/checksums, and deterministic consecutive archives passed on macOS ARM64.                                                                                                                                                    |

The host archive is `dist/solid-gpui-host-0.2.0-aarch64-apple-darwin.tar.gz`,
SHA-256 `dc7426434443a01687a3c90dc75a524f8399e871048d73249ad376f4bf96ed8b`.
It uses the existing default host feature set; QuickJS applications compile their
host with `quickjs` enabled. This check verifies archive assembly from the same
built binary, not reproducible compilation on different machines.

Local logs are `/tmp/solid-gpui-final-{package-ci,pack-smoke,rust-test,clippy,protocol,gallery-audit,audit,host-release}.log`.
Independent reviews found and helped resolve UTF-8 stream-state, Headers
validation, source/dist graph identity, and production runtime-selection defects;
no reported supported-contract finding remains open.

## Formatter migration follow-up

The subsequent September 7 migration replaces direct Prettier tooling with Oxfmt `0.66.0`. Existing formatter tasks retain their check-only behavior, covering maintained source, runtime fixtures, and configuration. The root configuration preserves the previous style and disables import and package-field sorting. The unused generated-file formatter is removed; the local pre-commit skill now documents Oxfmt's official lint-staged integration. No hooks were installed by this migration.

The earlier compiler measurements and runtime validation above remain historical evidence for those changes. The formatter migration does not claim an additional measured speedup. Its implementation boundary and upstream sources are in [compiler.md](compiler.md#formatter-migration).

Follow-up validation passed:

- Package CI: 78 Bun tests, all TypeScript checks, native binding regeneration checks, and isolated packed consumers. The packed native consumer invokes a real Rust fixture host through Vite configuration and Oxfmt under browser conditions.
- Native export: default and browser-condition tests pass, including malformed TypeScript rejection without overwriting the previous output.
- Formatting: workspace rustfmt and Oxfmt checks pass; Oxfmt checks 120 files.
- Protocol: byte hashes of all eight generated outputs match before and after regeneration with Oxfmt. This verifies the migrated working files, not a comparison against HEAD. Protocol goldens pass four Rust tests and both TypeScript/Rust producer-consumer directions.
- Audit: Bun reports no vulnerabilities across 147 packages; Cargo advisories pass with all workspace features under the existing exception policy, and regenerated notices match exactly.

Logs are `/tmp/solid-gpui-oxfmt-{package-ci,format,protocol-codegen,protocol-golden,audit}.log`.

## Limits

- The official Solid compiler is pinned to release candidate `2.0.0-rc.6` while
  the application runtime remains Solid 1.9.15. Actual universal-renderer behavior
  is exercised; this does not claim compatibility with every Solid 2 feature.
- QuickJS runs prebundled standard ES modules. Files, networking, and domain
  services belong in Rust Native Modules. Bun retains its existing service APIs.
- The optional embedded Bun/JSC source build was not rebuilt during this pass.
  Its existing platform/toolchain requirements remain; this is not qualification
  of that complete embedding or of every host release platform.
- The repository's three existing Rust advisory exceptions remain tracked in
  `deny.toml`. The transitive `block` 0.1.6 future-incompatibility warning remains.
- Passing correctness, layout, and lifecycle checks does not prove that every
  algorithm is globally optimal. No production display FPS improvement is claimed.
