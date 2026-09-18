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

### More recent completed baseline and runner variance

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
- Cache every `build:host` output together: WASM/glue and the generated SDK
  component catalog, not completed test results. The final namespace is
  `pages-web-host-v2`. Guard native setup/build on exact hit only; always run
  frontend build and the complete browser tests. Save with the key computed
  before generation, so rewriting the catalog cannot change the save identity.
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

## Verification results

Measurements below were completed on 2026-09-17 and 2026-09-18 UTC. Durations
are job execution times, including setup and post steps. The CI execution window
runs from the first job's start through the aggregate job's completion; it
excludes the initial queue. Runner sums are unweighted execution seconds, not
billed cost. Reruns retain their original run ID: use the job links to identify
the measured attempt, not the workflow's original creation timestamp.

### Local and portability checks

- Full local `bun run ci`, `bun run audit`, website build/tests, and unified
  `sdk-pack` passed before the initial optimization commit. No release was
  published.
- `actionlint` v1.7.12 passed, as did the task contract tests and all 16
  success/failure/cancelled/skipped pairs for the required aggregate.
- The frontend build and all website tests passed with a PATH shim that fails
  every Cargo invocation. The final website suite passed with 8 tests and
  3,950 assertions; the final hosted warm run repeated it successfully.
- Moving SDK validation to Linux exposed three watcher fixture failures in
  [the first package job](https://github.com/Cyenoch/solid-gpui/actions/runs/35221718312/job/105203544451).
  Vite's bundled watcher suppresses duplicate `change` events within 50ms.
  The fixture helper now separates synthetic saves by 100ms, without changing
  production watcher behavior, increasing timeouts, adding retries, or removing
  assertions. All six targeted watcher tests passed locally. An isolated checkout
  passed the complete SDK gate, including 53 tooling tests, and the three website
  content/Markdown tests. Subsequent Linux package jobs passed.
- Concurrent SDK changes were preserved. Their temporary formatting failures
  were not formatted away or included in the CI repair; the isolated verification
  checkout was removed after use.
- Restore/save use the same YAML-anchored output list. The final cache includes
  both `examples/website/src/wasm` and `packages/solid-gpui/src/components.ts`.
  Caching only WASM would omit an exporter side effect and could give warm and
  cold frontend builds different catalog inputs. Independent native CI still
  verifies the committed catalog's freshness.

### Final Pages cache: identical-source comparison

Both rows use commit `fd9cd1d80fb241be6aca6ff9e4569f35f906acd8` and the final
complete-output cache layout. An artifact miss does not imply that the separate
Cargo dependency cache is empty.

| Final Pages attempt | Build | Deploy | Evidence |
| --- | ---: | ---: | --- |
| Generated-host artifact miss | 7m 29s | 22s | [Build](https://github.com/Cyenoch/solid-gpui/actions/runs/35226875078/job/105220611252), [deploy](https://github.com/Cyenoch/solid-gpui/actions/runs/35226875078/job/105223275816) |
| Exact artifact hit, same-source rerun | 35s | 9s | [Build](https://github.com/Cyenoch/solid-gpui/actions/runs/35226875078/job/105456215909), [deploy](https://github.com/Cyenoch/solid-gpui/actions/runs/35226875078/job/105456344614) |

The final warm build is **92.2% shorter than its same-source artifact miss** and
**92.5% shorter than the recent 7m 48s pre-optimization baseline**. Its execution
window including deployment and the inter-job gap was 48s. The earlier WASM-only
layout also produced a 46s warm build, but it is superseded by this complete-output
measurement.

The final log confirms an exact `pages-web-host-v2` hit of 14,249,467 bytes.
Linux native packages, nightly installation, Cargo-cache setup, wasm-bindgen
installation, and host compilation were skipped. Frontend typechecking, Vite
bundling, all eight website tests, artifact upload, deployment, and the deployed
site HTTP check passed. The cache was not resaved on an exact hit.

### Native cache and complete CI: all successful warm samples

The new native namespace's first run had no cache and took **21m 20s**:
[cold native job](https://github.com/Cyenoch/solid-gpui/actions/runs/35221718312/job/105203544255).
The native job passed and populated its cache; that workflow overall failed
because of the Linux watcher fixtures described above. Do not count it as a
successful complete CI run. A subsequent `6518544d` native run was cancelled by
a concurrent push and is excluded from timing comparisons.

The concurrent `2155fb01` SDK commit changed no Rust sources, Cargo inputs,
vendored sources, or Rust toolchain. Its two complete attempts, followed by the
Pages/documentation-only `fd9cd1d8` change, produced these successful samples:

| Source and attempt | Native job | SDK job | Aggregate | CI execution window | Runner sum | Evidence |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `2155fb01`, attempt 1 | 9m 48s | 41s | 2s | 9m 59s | 10m 31s | [Native](https://github.com/Cyenoch/solid-gpui/actions/runs/35224402456/job/105212611547), [SDK](https://github.com/Cyenoch/solid-gpui/actions/runs/35224402456/job/105212611272) |
| `2155fb01`, identical-source attempt 2 | 9m 59s | 1m 34s | 4s | 10m 12s | 11m 37s | [Native](https://github.com/Cyenoch/solid-gpui/actions/runs/35224402456/job/105216610664), [SDK](https://github.com/Cyenoch/solid-gpui/actions/runs/35224402456/job/105216611163) |
| `fd9cd1d8`, final implementation | 13m 51s | 43s | 3s | 14m 03s | 14m 37s | [Native](https://github.com/Cyenoch/solid-gpui/actions/runs/35226874881/job/105220610803), [SDK](https://github.com/Cyenoch/solid-gpui/actions/runs/35226874881/job/105220610868) |

The first and final warm native logs both restore the same complete cache and
print **59 `Compiling` lines**, versus **1,044** in the new namespace's cold run.
The first protocol example drops from 471 cold compilation events to eight in
the first warm run; the old partial-cache baseline still had 412. These are
compilation log events, not a claimed cache-hit percentage or a count of unique
crates. Local workspace/vendor compilation, linking, test execution, and native
exporters remain.

The first two execution windows are 13–14% shorter than the recent 11m 40s
baseline, but the final one is slower. The successful warm range is therefore
**9m 59s–14m 03s**, with runner sums **10m 31s–14m 37s**. The data does **not**
establish a reliable reduction in total native CI wall time or runner minutes.
The defensible gains are avoiding demonstrated redundant compilation and giving
SDK checks independent, much earlier completion, not promising a fixed native
speedup. The 1m 34s SDK sample spent 71s in the package gate itself and 11s in
Rust setup, so its variability is not solely toolchain installation.

The complete native cache is 1,510,743,075 bytes, versus 440,346,943 bytes for
the old incomplete entry. Its greater storage/transfer cost is included in the
timings above. Keep monitoring eviction pressure; no per-commit target archive,
additional general-purpose compiler cache, or manual cache deletion was added.

### Delivery boundary

Implementation commits are `313d9343` (pipeline and task graph), `6518544d`
(portable watcher fixtures), and `fd9cd1d8` (complete generated-host artifacts).
The latest implementation's CI and Pages runs passed. Existing cross-platform,
Embedded Bun, and audit checks remain in place; the initial optimized
[cross-platform](https://github.com/Cyenoch/solid-gpui/actions/runs/35221718366) and
[Embedded Bun](https://github.com/Cyenoch/solid-gpui/actions/runs/35221718417) runs
passed. The concurrent SDK commit's
[audit](https://github.com/Cyenoch/solid-gpui/actions/runs/35224401947) also passed.

English/Chinese CI and Web guides, the website README, and the changelog were
synchronized. The measurement note is not website content and changes no public
API or generated catalog. Native visual acceptance was not repeated; it was
explicitly waived and is not implied by protocol or CI results.
