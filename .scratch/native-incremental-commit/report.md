# Native incremental commit delivery

## Implementation

The real host path now records decode, queue/backpressure handoff, tree mutation,
dependency collection, Extension validation, native reconciliation, instance
updates, and element construction. The native diagnostic binary uses the normal
host and disables the FPS overlay. Its Bun workload checks every measured text
operation, repeats count updates, and resizes a real window. CPU stage benchmarks
no longer reimplement the transaction or include automatic GPUI effect flushing
inside commit timing. The hardware-specific one-millisecond unit-test assertion
and the empty-store workset test were removed.

`NodeStore` keeps an undo transaction through Extension validation. Rejection or
unwinding restores the retained tree; revision and native effects are published
after acceptance. Snapshot construction no longer clones the previous store.

One `PatchChanges` result supplies journaled/deleted identities, original/final
ancestors, and typed ownership dependencies. Local validation and native
reconciliation consume that result. Deleted native entries are removed by ID;
focus observers reconcile a pending set instead of scanning the tree every draw.
Properties of an Extension and its declared native identity are distinguished
when finding child ownership dependencies.

## Baseline and CPU attribution

Base revision: `c6aec827c3e0fb44afcaa8ae28c9a338dd93a18b` (initially clean).
Machine: Mac17,8, 64 GiB RAM, 18 logical CPUs. Linked GPUI is the repository's
vendored `gpui-pre 0.3.3`, not the separate Zed reference checkout. Measurements
use the dev/test profile with optimized GPUI/Taffy dependencies. Background
applications remained running; machine load was not isolated.

The final comparison used separately compiled baseline and candidate executables
in A/B/B/A order, with compilation complete before measurements and one test
window released after each workload. Each reported median contains eight samples.

| Run | Text Patch commit p50 | Separate forced CPU draw p50 |
| --- | ---: | ---: |
| Baseline A | 5.652 ms | 67.143 ms |
| Candidate A | 0.101 ms | 65.322 ms |
| Candidate B | 0.098 ms | 65.393 ms |
| Baseline B | 5.905 ms | 67.802 ms |

These are the existing 20,000-node fixture with a single text operation. Commit
timing stops inside the GPUI update callback. The old stage imitation reported
0.165 ms while skipping full-store cloning and Extension validation; the old
outside-callback apply timer also counted an automatic draw. Both were corrected.

An initial non-isolated series showed higher candidate draw times. The benchmark
also kept earlier test windows alive; both variants now close each sample's
window before the next workload. The increase did not recur in the final matched
series. This does not establish a production frame-rate gain: rendering remains
the dominant cost in this deliberately large, unvirtualized fixture.

The work-bound regression independently proves that a local text commit visits
the same four dependency IDs with 1,000 or 20,000 unrelated nodes. A changed
Extension validates one contract among 2,000 peers.

Raw local artifacts (ignored by git):

- `artifacts/baseline-existing.log`: original, misleading stage/apply metrics.
- `artifacts/baseline-instrumentation.patch` and `profile-baseline.rs`: attribution changes used before optimization.
- `artifacts/baseline-measured.log`: corrected original-path baseline.
- `artifacts/candidate-transaction.log`: initial, non-isolated comparison.
- `artifacts/baseline-isolated.patch`: final baseline instrumentation and window cleanup.
- `artifacts/isolated-{baseline,candidate}-{a,b}.log`: final A/B/B/A comparison.
- `artifacts/js-profile.log`: combined JS signal-to-frame boundary; no IPC claim.

## Render-region decision

The temporary cached-pane experiment used the actual `SolidRoot::render_node`
path with fixed pane sizes and a shared native store. Twelve updates took
22.593 ms without cached panes and 1.394 ms with cached panes. Verification of
rendered content rejected the apparent improvement: the uncached pane rendered
`Count: 12`, while the cached pane remained at `Count: 0` even though its store
had reached 12. Reading the shared root does not supply the region notification
contract needed here.

No cached region implementation was retained. Future region ownership must
explicitly handle both commit invalidation and native-only state, including
input, scrolling, animation, asynchronous content, geometry and inherited text
style. This is the measured decision requested by step four, not a production
cache acceptance. The experiment source and output remain in
`artifacts/region-experiment.rs` and `artifacts/region-experiment.log`.

## Native verification

The production diagnostic host was built without test-support. Its exact binary
path/hash, features, runtime, monitor state and viewport sequence are recorded
in `artifacts/native-build-identity.json`; the dependency graph is in
`artifacts/native-features.txt`.

With external Bun 1.4.2, 500 static rows, and the FPS overlay off, the active
native window completed 360 verified count patches and widths
800 → 560 → 1280 → 800 at a height of 600. CUA screenshots confirmed `Count: 360`,
a native press produced `Count: 361` in the earlier run, and the final input
probe visibly changed the field to `Verified input`. The input fixture needs an
explicit height; its initial unconstrained hit area was corrected before that
verification.

The verified run's signal-to-encoded-frame p50/p95 were approximately
0.105/0.185 ms. That includes JS commit construction, encoding and microtask
scheduling, and ends before the transport call. It is neither pure Solid cost
nor input-to-present latency.

Active native frame intervals reported CPU draw p95 values from 7.135 to
18.809 ms, depending on viewport and interval. This range is not a pooled p95,
and does not establish stable 60/120 Hz or display scanout. Idle intervals and
synthetic-input timestamps are excluded from acceptance. A five-second native
CPU sample is retained in `artifacts/native-sample.txt`. Synthetic CUA scrolling
did not move content and is not accepted as a scrolling reproduction. Physical
trackpad scrolling and OS live resizing still need platform-specific acceptance.

## Verification and synchronization

- Rust workspace tests with QuickJS: 328 passed, two opt-in/previously ignored cases skipped.
- Full component/QuickJS library tests passed; focused typed-slot rollback test passed. The final retained-identity regression also passed: moving an instance out before deleting its old parent preserves its native entity and event sink.
- Strict workspace/all-target Clippy with QuickJS and frame-profile passed.
- Module dependency guards and protocol golden vectors passed; no wire schema or generated API changed.
- Solid renderer JS tests: 34 passed; tools TypeScript and changed TS formatting passed.
- Native and WebAssembly website production builds passed. All six website tests passed after prebuilding (including the retained-sidebar navigation test). The first run timed out while waiting for native compilation; its rerun was serialized after the build.
- T3 browser preview loaded the built WebGPU website, displayed the updated Signals & updates paragraph, and visibly incremented its counter from 0 to 1 without console errors. The build retains existing generated-WASM eval/chunk-size and target-specific dead-code warnings.
- Protocol and performance guides, their explicit Chinese copies, the docs index, website Signals chapter and website development README were synchronized.

The final automatic invocation completed all 360 verified operations, reported
an 800×600 logical viewport at scale factor 2, and exited successfully with code
0. Its binary hash and output are in `artifacts/native-auto-exit-identity.json`
and `artifacts/native-auto-exit.log`. The configured primary display reports
1728×1117 logical / 3456×2234 physical pixels at 120 Hz; the external display
reports 2560×1440 at 144 Hz (`artifacts/displays.json`). The workload's deliberate
30 updates/s is not a maximum-FPS measurement. The website was rebuilt after the measurement guide's final clarifications and
the host-only queue stage was excluded from non-host WASM builds. The final
content/highlighting check also passed.

Artifacts are local diagnostic evidence, not published benchmarks.
