# CI/CD optimization research

## Scope and evidence

Optimize feedback time and repeated work without dropping default-feature, QuickJS,
protocol, generated-binding, package-consumer, platform, audit, or website checks.
Local caches are not evidence of hosted performance. Compare runner execution time
separately from queue time, deployment time, and aggregate runner minutes.

Baseline: commit `25f4cf1ae65ad8acdbc57ba48f7a6e785d3ba91a`.
Timings below come from GitHub's job/step API and timestamped logs, not estimates.

| Baseline job | Runner duration | Evidence |
| --- | ---: | --- |
| Rust and Bun checks | 23m 28s | [CI job](https://github.com/Cyenoch/solid-gpui/actions/runs/35210750925/job/105167414645) |
| Pages build | 6m 36s | [Pages build](https://github.com/Cyenoch/solid-gpui/actions/runs/35210750915/job/105167414183) |
| Pages deployment | 54s | [Pages deployment](https://github.com/Cyenoch/solid-gpui/actions/runs/35210750915/job/105169299686) |
| Linux host checks | 2m 41s | [Linux job](https://github.com/Cyenoch/solid-gpui/actions/runs/35210751133/job/105167415706) |
| Windows host build | 3m 47s | [Windows job](https://github.com/Cyenoch/solid-gpui/actions/runs/35210751133/job/105167415430) |
| Dependency audit | 40s | [Audit job](https://github.com/Cyenoch/solid-gpui/actions/runs/35210750859/job/105167414444) |

The macOS check step itself took 22m 40s. Within it:

| Operation | Measured duration |
| --- | ---: |
| Compile/run protocol golden example | Cargo reports 4m 20s compilation |
| Default-feature golden test | Cargo reports 7.21s compilation; 4 tests pass |
| Default-feature workspace check | Cargo reports 3m 12s |
| QuickJS all-target workspace Clippy | Cargo reports 2m 20s |
| QuickJS workspace test compilation | Cargo reports 6m 04s |
| Core Rust library test execution | Test harness reports 139.01s |
| Patched HTTP client test compilation | Cargo reports 11.00s |
| SDK native bindings exporter build | Cargo reports 3m 22s, overlapping package checks |
| Website native bindings exporter build | Cargo reports 7.36s |

Do not sum overlapping compiler/test lines as independent critical-path stages.

### Immediately preceding baseline and runner variance

While this work was in progress, an independent native DTO change was committed
as `f830c0b349f6f1939f1819a6090b9dd0aeb02f64`. Its unchanged CI configuration
completed successfully with noticeably different timings:

| Job | Runner duration | Evidence |
| --- | ---: | --- |
| Rust and Bun checks | 11m 40s | [Latest pre-optimization CI](https://github.com/Cyenoch/solid-gpui/actions/runs/35218263312/job/105191830783) |
| Pages build | 7m 48s | [Latest pre-optimization Pages](https://github.com/Cyenoch/solid-gpui/actions/runs/35218263392/job/105191830844) |
| Pages deployment | 31s | [Latest pre-optimization deployment](https://github.com/Cyenoch/solid-gpui/actions/runs/35218263392/job/105194160112) |

That CI run restored the identical 420 MiB key. The first example still compiled
for 2m 24s, default check for 1m 06s, Clippy for 1m 01s, workspace tests compiled
for 2m 55s, and the SDK exporter for 1m 47s. Use this **more recent, faster native
baseline** for headline comparisons rather than attributing all improvement from
the slower 23m 28s run to these changes. Hosted wall times are not controlled
benchmarks; record both the cache evidence and runner variability.

### Exact cache hit did not mean a complete build cache

The baseline restored the exact cache key
`v0-rust-macOS-ARM64-ci-macos-15-checks-Darwin-arm64-4b070e83-ea434542`
(440,346,943 bytes, approximately 420 MiB), then printed **412 `Compiling` lines**
for the first protocol example, including registry crates such as `libc` and
`serde`. The post step said `Cache up-to-date.` and did not update it.

Both the old main CI and lightweight Embedded Bun jobs used job ID `checks`,
runner `macos-15`, and cache purpose `ci-macos-15`. Their immutable namespace was
identical although one needed linked/test artifacts and the other only Clippy's
check graph. A failed full CI could also save a partial graph because
`cache-on-failure` was enabled. The logs prove incomplete reusable artifacts,
not which earlier job first populated the entry.

The repository cache API reported 10,481,932,967 bytes at inspection time, close
to the default 10 GiB allowance. Adding a second general-purpose compiler cache
or one full target archive per commit would increase eviction pressure.

## Primary-source findings

1. **GitHub cache entries are immutable.** An exact hit is not overwritten;
   fallback keys can restore a different input state. Use fallbacks only for
   intermediate dependencies that the compiler will revalidate, never as proof
   that a final artifact can skip compilation.
   [GitHub cache reference](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching)
2. **Cache the graph actually consumed by a job.** The pinned Rust action adds
   job/environment/dependency identity to its key and prunes local workspace
   products by default. Keep full native CI, check-only Embedded Bun, native
   platform builds, and WASM purposes separate. Only successful trusted runs
   should seed immutable entries.
   [Pinned rust-cache configuration](https://github.com/Swatinem/rust-cache/blob/6323deb102c322ba6fcbdcafc7e3dddab59af2b6/src/config.ts),
   [save behavior](https://github.com/Swatinem/rust-cache/blob/6323deb102c322ba6fcbdcafc7e3dddab59af2b6/src/save.ts),
   [documented controls](https://github.com/Swatinem/rust-cache/blob/6323deb102c322ba6fcbdcafc7e3dddab59af2b6/README.md)
3. **Cargo check, linked tests, features, and cross targets are not interchangeable.**
   Cargo stores separate build artifacts; workspace wrappers can also alter
   artifact identity. Default-feature `cargo check` must remain here because
   QuickJS Clippy enables a different consumer dependency configuration.
   [Cargo build cache](https://doc.rust-lang.org/cargo/reference/build-cache.html),
   [workspace wrapper environment](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-reads),
   [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html)
4. **Independent checks can run in separate jobs.** Move pure SDK validation to
   Linux while native macOS checks continue. Preserve a fail-closed aggregate
   status rather than letting the previous required check become narrower.
   [Job dependencies](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idneeds),
   [status functions](https://docs.github.com/en/actions/reference/workflows-and-actions/expressions#status-check-functions)
5. **Final WASM output can be reused only under full input identity.** Use an exact
   cache key over Rust/configuration/toolchain/build/exporter/embedded-asset inputs;
   no fallback keys. Frontend typechecking, Vite compilation, and browser tests
   still run on every relevant website change. Keep PRs restore-only.
   [Cache scopes and security](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching#restrictions-for-accessing-a-cache),
   [Cargo dependency information](https://doc.rust-lang.org/cargo/reference/build-cache.html#dep-info-files)
6. **Do not add package caches without measuring transfer costs.** Bun already
   has an efficient global package cache and platform-specific installation
   fast paths. The repository's earlier hosted measurement was 5s cache restore
   versus 4s uncached install; preserve uncached frozen installation here.
   [Bun package cache](https://bun.com/docs/pm/global-cache),
   [Bun installation](https://bun.com/docs/pm/cli/install)
7. **sccache is not a blanket fix.** It cannot cache Rust crates that invoke the
   system linker, and incremental compilation is incompatible with its Rust
   caching. It could help selected rlibs, but its net benefit is not established
   for this repository; first fix the demonstrated incomplete-cache namespace.
   [sccache Rust limitations](https://github.com/mozilla/sccache/blob/main/docs/Rust.md)

## Selected implementation

- Split `native-ci` and `package-ci` into independent macOS/Linux jobs. Preserve
  local `bun run ci` as their full composition with one memoized package build.
- Preserve the required status name `Rust and Bun checks`. Its aggregate accepts
  only `success` from both dependencies, including failure/skipped/cancelled cases.
- Isolate `ci-native-macos-15` from `embedded-check-macos-15`; stop saving partial
  Cargo caches from failed jobs. Leave dependency-only pruning and incremental
  settings unchanged; no source-per-commit target caches or cache deletion.
- Separate website `build:host` from `build:frontend`; local `build` still executes
  both and local `dev` still prepares bindings/WASM automatically.
- Cache generated WASM/glue, not completed test results. Guard native setup/build
  on exact hit only. Always run frontend build and the complete browser tests.
- Pack all four release-prep SDK packages using one `sdk-pack` invocation instead
  of four independent build/pack processes. Keep version-labelled uploaded
  artifacts and no publication action.

The website's existing MemoryTransport test initially loaded the native Vite
configuration and therefore compiled Rust despite never connecting to that host.
It now uses the website's universal transform with the same snapshot/navigation
assertions and shares Bun's browser Solid instance with the surrounding tests.
The entire website suite and frontend build were exercised with a PATH shim that
fails any Cargo invocation: no native build is needed on the warm Pages path.

## Verification protocol

Run `actionlint`, the workflow CLI contract checks, full local `bun run ci`,
`bun run audit`, the full website build/tests, and the unified SDK pack command.
Exercise the aggregate's success/failure/skipped/cancelled results. Verify the
WASM key includes actual tracked native build inputs but excludes ordinary docs
and website TypeScript.

For hosted measurement, first run the changed workflows to populate their new
namespaces, then rerun the same source to measure warm behavior. Report both
results; never compare a cold new namespace to a warm baseline without labeling
it. Record the native and package job durations plus their sum, and verify that
warm Pages actually skips apt/toolchain/bindgen/host compilation while frontend
and tests still pass. Record results below only after the Actions API confirms
completion.
