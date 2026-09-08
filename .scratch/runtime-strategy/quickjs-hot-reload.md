# QuickJS hot reload research

Implementation update (2026-09-07): the native frame bridge and QuickJS application reload are now implemented. The observations below record the research baseline; see [runtime strategy](../../docs/runtime-strategy.md) and [implementation validation](implementation.md) for current behavior.

Date: 2026-09-07. Status: research recommendation; no runtime implementation or performance benchmark was performed.

## Recommendation

Use external Bun/Vite development tooling to rebuild QuickJS-compatible UI code, then replace the UI application in a **fresh QuickJS Runtime** while retaining the Rust application, native windows, and Rust-owned services. Start with whole-bundle application reload and explicit state restoration. Treat a custom Vite ModuleRunner integration as a later, measured optimization, not a prerequisite for useful QuickJS reload.

This makes QuickJS development execute the same runtime and platform contract as release. It complements the default external Bun rapid-iteration path, particularly when catching unsupported APIs or testing Rust-owned application behavior. It does not make Vite a production dependency, run Vite inside QuickJS, or promise component-local signal preservation.

## Current implementation and gaps

- [`quickjs.rs`](../../crates/solid-gpui/src/runtime/quickjs.rs) reads one prebundled ES module, creates one worker-owned `Runtime`/`Context`, and evaluates it once. The worker owns the event, timer, microtask, rejection, cancellation, and teardown loop. There is no source replacement operation, module loader, or Vite connection.
- [`build.ts`](../../packages/solid-gpui/src/vite/build.ts) actually uses `Bun.build`, not Vite's build engine. It bundles dependencies into one ESM output, disables splitting, injects the explicit QuickJS platform before application imports, rejects Node/Bun imports, and currently defines `NODE_ENV` as production.
- [`dev.ts`](../../packages/solid-gpui/src/vite/dev.ts) runs Vite's runnable SSR environment inside Bun. [`index.ts`](../../packages/solid-gpui/src/vite/index.ts) externalizes Solid modules for Bun and appends an entry HMR accept boundary. This configuration cannot be reused unchanged for QuickJS.
- [`application.ts`](../../packages/solid-gpui/src/application.ts) provides useful lifecycle semantics: candidate output buffering, `captureState`, synchronous setup/render recovery, owner disposal, and epochs. Its retained session map lives on `globalThis`; it cannot survive a fresh runtime. The epoch currently derives from that map and would restart at 1 in every new VM. Its state handoff calls `structuredClone`, which the current QuickJS bootstrap/platform does not install.
- [`host/mod.rs`](../../crates/solid-gpui/src/host/mod.rs) retains adapter references and applies surface messages individually. A stable supervisor adapter and an explicit activation boundary are needed; replacing a file or a local adapter variable is insufficient. Existing per-surface validation does not by itself provide an atomic application-wide generation swap.

The locked binding is **rquickjs 0.12.2**, whose engine is QuickJS-NG. Use its actual vendored implementation and matching binding API for implementation decisions. Bellard QuickJS documentation is useful background, but its version, feature list, and startup figures are not measurements of this application. See [`Cargo.lock`](../../Cargo.lock), [rquickjs 0.12.2 source](https://docs.rs/crate/rquickjs/0.12.2/source/README.md), and [QuickJS-NG API guide](https://quickjs-ng.github.io/quickjs/developer-guide/intro/).

## What Vite supplies

Vite supports rebuild-on-change through `build.watch`/`vite build --watch`; configuration changes require restarting that watcher. This supplies bundling notifications, not native runtime replacement. [Vite build documentation](https://vite.dev/guide/build#rebuild-on-files-changes).

Vite also supports a `ModuleRunner` **inside the target runtime**, connected to a server-side environment through custom transport. HMR needs a persistent receive path; HTTP request/response alone does not supply it. The standard evaluator uses `AsyncFunction`, and external modules require separate handling. The Environment API remains release-candidate, with some experimental surfaces. [Vite runtime API](https://vite.dev/guide/api-environment-runtimes).

The existing Bun `RunnableDevEnvironment` works because its runner executes in the tooling runtime and can return live JavaScript exports there. That does not transfer Bun objects into QuickJS. A QuickJS integration needs a separate environment and serialized communication. [Vite framework API](https://vite.dev/guide/api-environment-frameworks).

### Build pipeline choice

For the first implementation, reuse the existing QuickJS `Bun.build` compiler and shared JSX transform, with Vite's development session coordinating file changes and rebuilds. Track the actual build dependencies, including dependency additions/removals; a Vite dev module graph will not automatically be populated when Bun performs the bundle. Avoid monitoring only the entry file. Explicitly separate development diagnostics from production optimization while sharing resolution, platform injection, and unsupported-import rules.

An alternative is to implement a dedicated Vite QuickJS build environment with complete bundling, no externalized Solid packages, no browser preload/client injection, and the same explicit platform. If chosen, make it the shared QuickJS build pipeline for development and release instead of maintaining two subtly different compilers. Vite build-watch is a valid direction, but not a one-flag extension of today's configuration.

## Why replace the Runtime, not merely the module

QuickJS-NG defines a Runtime as an object heap and a Context as a realm. Contexts in one Runtime can share objects; separate Runtimes cannot. Memory and interrupt controls are runtime-scoped. [QuickJS-NG runtime model](https://quickjs-ng.github.io/quickjs/developer-guide/intro/). The binding similarly restricts values to their runtime through `Context::with`. [rquickjs 0.12.2 Context](https://docs.rs/rquickjs/0.12.2/rquickjs/struct.Context.html).

Inspection of the locked `rquickjs-sys` source shows context-owned `loaded_modules` and a runtime-owned `job_list`. Therefore a new Context resets its module namespace, but should not be treated as proof that old jobs, callbacks, host handles, or heap retention have disappeared. A fresh Runtime gives a clearer lifecycle boundary. This is an engineering recommendation; no reload-cost comparison has been measured.

| Replacement unit                  | Advantage                                             | Cost / correctness issue                                                                           | Decision                                                              |
| --------------------------------- | ----------------------------------------------------- | -------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| Changed ESM with a new URL        | Small apparent edit                                   | Old dependency closures, module records, jobs, and side effects remain; unique URLs can accumulate | Reject as the initial design                                          |
| Fresh Context in existing Runtime | Fresh realm and module namespace                      | Shared heap and runtime jobs complicate isolation and cancellation                                 | Consider only if profiling justifies it                               |
| Fresh Runtime                     | Entire JS graph and job lifecycle can retire together | Bundle parse/evaluation and temporary second-VM memory                                             | Initial recommendation                                                |
| Full native process restart       | Simple implementation                                 | Loses windows and Rust service state                                                               | Explicit restart for native/config changes; not the desired UI reload |

Destroying the old VM releases JS state, but does not undo Rust I/O already performed. Host cancellation and generation ownership remain necessary.

## Proposed lifecycle and consistency contract

This section specifies new behavior to implement, not existing guarantees.

1. **Build while the active generation runs.** Assign every requested build a monotonic identifier. Publish only complete successful output, using an immutable in-memory artifact or a temporary file followed by atomic rename. Keep only the latest pending request and one candidate. Superseded candidates can never activate. Bound source size and staging memory separately from normal protocol queues.
2. **Prepare before state capture where possible.** Start the fresh VM, install the platform and bridge, and compile candidate source without running application side effects. Refactor application evaluation/mounting so preparation does not accidentally start services.
3. **Quiesce and capture at a defined boundary.** Finish the active event/microtask checkpoint, briefly pause application event dispatch, and invoke `captureState` on its owning worker. Freeze old timers/jobs for the handoff. Capturing before a long build would lose subsequent edits; keeping both applications mutating state during capture/activation creates races. Retain bounded native events during the pause with a documented overflow policy.
4. **Transfer only explicit data.** Put generation, surface identities, activation sequence, and state bytes in a host-owned handoff record. Use a bounded, runtime-independent state codec, initially an explicitly documented JSON-compatible subset with strict validation rather than silent JSON coercion. Reject cycles, functions, handles, and unsupported types. This is a deliberate new contract; do not call JSON serialization `structuredClone`. Rust business state remains in Rust. No compatibility wrappers or historical migration framework are needed.
5. **Evaluate and stage.** Initialize the candidate with its host-issued epoch and state. Hold all native output behind an activation gate. Validate complete initial snapshots and application control state before publishing. Define readiness explicitly: the first emitted frame may be a router loading snapshot, not settled route content. A valid declared loading UI can be the activation boundary, with later asynchronous failures treated as post-activation failures. If settled-route readiness is required, provide generation-scoped, explicitly read-only setup queries and a readiness deadline; do not deadlock by withholding the native replies needed to become ready. Disallow mutating commands before activation and move effects to activation/onMount. Do not make every command implicitly transactional. Apply a preparation deadline, memory limit, and bounded microtask work.
6. **Activate atomically.** On the GPUI owner thread, validate the entire affected surface set and switch the committed tree state and active generation routing in one ordered activation operation. Distinguish successful enqueue from host acceptance; the candidate needs an activation acknowledgement. Preserve native window identity. For multiple surfaces and zero-window keep-alive, transfer the application registry and activation sequence, not just a main-window snapshot. Do not advertise application-wide rollback until this coordinator exists.
7. **Retire the previous generation.** Stop its dispatch, cancel generation-scoped native requests, reject old replies, discard old queued commits/events, and run bounded cleanup on its worker. Cleanup must be unable to close the supervisor's transport or mutate the new tree. Join retired workers off the GPUI thread. Epoch, surface, and request identity checks must prevent delayed results and recycled identifiers from reaching a new application.
8. **Handle failure according to the boundary.** Build, state capture, candidate evaluation, validation, timeout, or supersession before activation leaves the old application authoritative; release the pause and report a diagnostic. After activation, `onMount`, asynchronous exceptions, or completed native I/O are not safely reversible. Report the failed active generation and permit the next successful edit to replace it; do not promise arbitrary rollback of external effects.

Queued pointer/key events belong to the tree and epoch that received them and must not be relabeled onto different nodes after activation. Host facts such as current dimensions and application activation should be resampled or transferred through explicit current-state messages. On candidate rejection, retained events can resume against the unchanged old generation.

Fresh-runtime reload preserves explicitly captured UI state plus Rust-owned state. Native input buffers, focus, scroll offsets, and component-local signals are not automatically preserved. Add stable application state ownership only where the product requires it; do not copy arbitrary native caches between epochs.

## ModuleRunner as a later option

A measured second phase could keep Vite's transforms/module graph warm while providing a QuickJS-specific environment and a bundled `vite/module-runner` bootstrap. Use a Rust-owned development control channel instead of adding ambient WebSocket/fetch/Node capabilities to the UI platform. Keep development messages separate from native UI protocol frames and bounded independently.

The installed Vite runner uses runtime evaluation, external import, module caches, and source-map hooks; its source is the concrete compatibility audit target. A prototype must reject external Node/Bun imports, provide QuickJS-appropriate import metadata, test dynamic imports/cycles/top-level await, and use host/tooling source-map mapping rather than assuming QuickJS implements V8 `Error.prepareStackTrace`. The current package declares Vite `^8.2.2`; pin the tested API/version relationship for such an integration. See [`packages/solid-gpui/package.json`](../../packages/solid-gpui/package.json) and [Vite's runner implementation](https://github.com/vitejs/vite/blob/main/packages/vite/src/module-runner/index.ts).

Runner cache invalidation is a code-loading mechanism, not a native application transaction or Solid owner disposer. A runner per candidate Runtime maintains clean failure isolation but reexecutes the application graph; a persistent runner can reduce evaluation work while inheriting harder side-effect and stale-reference problems. Decide between them with measurement and explicit failure semantics. The first prototype should reuse the supervisor/activation contract rather than replacing it.

## Implementation sequence and key acceptance tests

1. Refactor one explicit application-generation lifecycle and stable supervisor adapter. Add bounded state export/import, host-issued epochs, candidate staging, and activation acknowledgement. Validate direct bundle replacement without Vite first.
2. Add QuickJS rebuild/watch integration using one shared compiler policy, latest-wins delivery, readable source diagnostics, and explicit restart reasons for Rust/native-contract/config changes.
3. Add multi-surface/zero-window lifecycle coverage before claiming complete application parity. Measure successful reloads and recovery loops; only then prototype ModuleRunner or Context reuse when the profile identifies a meaningful benefit.

Keep the essential tests behavioral:

- A dependency edit changes the real QuickJS-rendered UI while retaining native window identity, explicit state, and a Rust service instance; rapid saves apply only the newest candidate.
- Build error, throwing setup, state codec rejection, invalid staged snapshot, infinite candidate loop, and superseded candidate retain the old interactive generation; fixing source recovers.
- Delayed old native replies, queued input, timers, and cleanup cannot affect the new epoch; resource counts return to baseline over repeated successful/failed reloads.
- Native activation rejection does not partially replace a multi-surface application; zero-window activation and close-during-reload preserve application sequencing.
- A post-activation failure is reported without claiming I/O rollback, and a subsequent valid candidate can activate.
- The same QuickJS entry and unsupported-import checks work through development reload and release bundling.

Measure edit-to-first-presented-frame, build time, VM preparation, capture pause, evaluation, native activation, retirement, peak RSS, queue depth, and retained resources. Compare the same application/data on repeated warm edits and dependency edits; report percentiles and separate errors from successful updates. The tiny engine-startup figures in upstream documentation cannot stand in for parsing the Solid graph, rendering, or native first-frame latency. Follow [`docs/performance-analysis.md`](../../docs/performance-analysis.md).
