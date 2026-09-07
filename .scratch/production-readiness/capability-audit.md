# Native framework capability audit

Date: 2026-09-07. Scope: the current working tree, including QuickJS `AbortSignal.any()`.
This is a source and existing-evidence audit, not a new desktop qualification run.

Solid owns composition, application state, and reactive computations; GPUI owns layout, painting, editing, focus, and native entities. Native Modules supply application services. That boundary is already explicit in [CONTEXT](../../CONTEXT.md) and [runtime ADR](../../docs/adr/0017-runtime-engines.md).

**Assessment:** component breadth is already substantial. Production work should concentrate on desktop qualification, accessibility, cancellable native operations, and application lifecycle integration before adding another large widget catalog.

## Status definitions and method

- **Implemented:** public contract and corresponding implementation exist; this does not imply every OS has been qualified.
- **Partial:** a concrete part of the contract is absent or restricted.
- **Absent from the standard SDK:** the reviewed public contract and host do not provide it; custom Rust modules may implement it.
- **Unverified:** implementation may exist, but the inspected evidence does not establish the claimed behavior.

Checked SDK exports/types, generated component contracts, Rust host/command/runtime paths, router tests, native tests, maintained guides, and existing `.scratch` investigations. Absence claims below combine API inventory, implementation inspection, and documented scope; they are not deductions from one keyword search. Linked GPUI is `gpui-pre 0.3.3` in [Cargo.lock](../../Cargo.lock), rather than the reference Zed checkout. No benchmarks or tests were rerun for this report.

## Existing foundation

| Area | Status and evidence |
| --- | --- |
| Solid rendering | **Implemented:** universal JSX, transactional Snapshot/Patch, reactive props, stable owner/node lifetimes, batched events, disposal, and HMR epochs. See [renderer](../../packages/solid-gpui/src/renderer.ts), [renderer tests](../../packages/solid-gpui/tests/renderer.test.ts), [application](../../packages/solid-gpui/src/application.ts), and [HMR guide](../../docs/hot-reload.md). |
| Routing | **Implemented:** TanStack native memory history, independent windows, nested layouts/Outlet, reactive navigation, loader errors/redirects and superseded-loader cancellation. See [router](../../packages/solid-gpui-router/src/index.ts), [router tests](../../packages/solid-gpui-router/tests/router.test.ts), and actual-VM [router fixture](../../fixtures/quickjs-router.tsx). Browser history/DOM navigation are deliberately excluded. |
| Controls and data | **Implemented:** 144 generated components/descriptors, including Editor, DataTable, Tree, choice controls, overlays, DockArea, charts, native theme and layout persistence. Virtualized native delegates and stale-request rejection already exist. See [coverage and contracts](../../docs/gpui-components.md), [generated API](../../packages/solid-gpui/src/components.ts), and [native adapters](../../crates/solid-gpui/src/components). Counting these as missing would be incorrect. |
| Editing and interaction | **Implemented:** native composition/selection/undo, controlled edit acknowledgements, keyboard actions, focus traversal/trapping, rich text, drag/drop, clipboard text/images, file dialogs, and window controls. Clipboard images explicitly reject Linux. See [bridge contracts](../../docs/rust-bridge.md), [primitive props](../../packages/solid-gpui/src/renderer/types.ts), [commands](../../crates/solid-gpui/src/renderer/commands.rs), and [host roundtrip tests](../../crates/solid-gpui/tests/host_command_roundtrip.rs). |
| Host/runtime | **Implemented:** process Bun, optional embedded Bun, embedded QuickJS, generated typed Native Modules, bounded transport/native work, lifecycle cancellation, and atomic validation. QuickJS supplies its documented UI platform, including `AbortSignal.any()`; see [cancellation evidence](cancellation.md), [native executor](../../crates/solid-gpui/src/native/executor.rs), and [runtime implementation](../../crates/solid-gpui/src/runtime). Embedded Bun remains macOS-specific. |
| Development and delivery | **Implemented:** Vite/Bun HMR, application source maps, native code generation, packed-consumer checks, protocol golden/fuzz tests, native host tests, performance tools, package checksums and portable artifacts. See [distribution](../../docs/distribution.md), [performance workflow](../../docs/performance-analysis.md), [workflows](../../.github/workflows), and [previous evidence](report.md). |

## Prioritized gaps and acceptance criteria

P0 blocks a broad production-ready claim for supported desktops. P1 is the next framework investment. P2 is product-driven expansion; it is not a prerequisite for every native application.

### P0 — Qualify the actual distributed application on each supported desktop

**Unverified / partial qualification.** Existing macOS packaged cold starts are documented. The final archived candidate's interaction smoke was not repeated; Windows/Linux packaging is wired but this session has no observed native qualification. Linux CI explicitly leaves display-backed smoke as TODO; Windows cross-platform CI builds rather than running desktop interactions. These limits are explicit in [package evidence](package-verification.md), [cross-platform CI](../../.github/workflows/cross-platform.yml), and [distribution guide](../../docs/distribution.md).

**User impact:** successful compilation does not establish first paint, usable text input, clipboard, GPU/portal behavior, or clean-machine launch.

**Acceptance:** retain one extracted artifact per advertised target; launch outside the checkout without a development toolchain; exercise first frame, input/composition, navigation, native async command, close/reopen policy, and shutdown. Record OS/architecture/display backend, artifact digest, and observed failures. Obtain successful native packaging jobs. Complete publisher signing/notarization and dependency redistribution materials for public delivery. Do not require an updater or installer to call the framework usable when a documented portable distribution is the chosen product format.

### P0 — Accessibility that survives real assistive technology use

**Partial implementation; desktop behavior unverified.** Primitive roles cover button, label, text input, checkbox, heading, and link; label/description/selected/checked/value/expanded/level are mapped. Rich native components can expose their own semantics, so primitive limitations are not proof every component is inaccessible. However, [primitive accessibility](../../crates/solid-gpui/src/renderer/paint/accessibility.rs) does not publish disabled state. The linked registry source `gpui-pre-0.3.3/src/elements/div.rs` was inspected: its public builders still have no live-region or disabled builder, while numeric values, orientation, set position, and table coordinates do exist upstream. The existing [live-region investigation](../a11y-second-pass/issues/02-live-regions.md) therefore remains a real upstream seam gap, although its reference-checkout evidence should not substitute for the linked source.

**User impact:** async completion/errors can remain unannounced; custom complex controls have insufficient semantics; a visual Gallery is not screen-reader qualification.

**Acceptance:** one small real-desktop flow per target using VoiceOver, NVDA, or Orca as appropriate: name/role/value/state, keyboard activation, focus containment and return, validation error, and async completion announcement. Fix GPUI's actual adapter seam for live/disabled semantics and then expose only supported canonical fields. Audit generated table/tree/editor controls separately. Keep the few regressions that prove semantics and identity; a blanket test per exported component is unnecessary.

### P1 — Cancel one native operation without closing its Surface

**Absent from the standard call contract; lifecycle cancellation implemented.** [NativeInvoker](../../packages/solid-gpui/src/native.ts) and [RootContainer.invokeNative](../../packages/solid-gpui/src/renderer/root-container.ts) accept module/function/bytes and return one Promise. [CommandClient](../../packages/solid-gpui/src/renderer/command-client.ts) settles on response or termination, with no per-call AbortSignal/deadline API. [Native executor documentation](../../docs/rust-bridge.md) guarantees Surface/epoch/disposal cancellation and explicitly requires cooperation for long blocking tasks. `AbortSignal.any()` is a JS signal primitive; it does not itself cancel a Rust command.

**User impact:** cancelling a search, upload, or analysis in a still-open window needs application-specific cancellation RPCs; stale operations can continue consuming bounded executor capacity.

**Acceptance:** a generated call accepts an explicit cancellation option; an already-aborted call is never admitted; abort cancels the exact admitted async task, releases capacity, and settles its Promise once; late results are discarded; adjacent calls remain unaffected. Blocking work retains admission until it exits and has a documented cooperative cancellation token. Specify timeout versus abort errors. Implement schema → generator → checked bindings, with one race/lifetime regression covering those outcomes.

### P1 — Explicit application-level activation and lifecycle hooks

**Partial.** Surface open/close, asynchronous close confirmation, active-window keybindings, external URL opening and notification responses already exist. The reviewed [root API](../../packages/solid-gpui/src/renderer/root-container.ts), [application lifecycle](../../packages/solid-gpui/src/application.ts), and [host runner](../../crates/solid-gpui/src/host/mod.rs) do not provide standard inbound deep-link/document-open, second-instance delivery, or reopen-with-no-window events. The stock runner quits after the last Surface closes; that is a valid default, but not sufficient for a tray-resident application or macOS reopen workflow. A custom HostProfile is an extension point, not a ready-made SDK contract for these cases.

**User impact:** OAuth callbacks, opening documents from the OS, and restoring an already-running application require separate host work.

**Acceptance:** define explicit host lifecycle policy and a bounded typed activation event. Deliver both cold-start and running-instance activation to the intended application/window exactly once; queue until the application is ready; reject malformed/unregistered URL schemes; define close-last-window versus quit. Qualify document associations/deep links on each advertised platform. Treat tray icons and global shortcuts as optional native modules when an application needs them, not mandatory core controls.

### P1 — Qualify Solid asynchronous composition beyond router navigation

**Unverified, not an assertion that Solid primitives are absent.** Solid itself provides resources, Suspense, ErrorBoundary, lazy components and transitions. Existing router tests prove nested-layout retention and window independence, and the real QuickJS fixture proves loader behavior. Those are not equivalent to a packaged app exercising native-module resources inside nested async/error boundaries. [Solid's official resource contract](https://docs.solidjs.com/reference/basic-reactivity/create-resource) explicitly ties resource state and errors to Suspense; the framework should demonstrate that contract under its universal renderer.

**User impact:** reusable Solid application components may fail only during pending/error/retry/unmount transitions, even when signals and route navigation work.

**Acceptance:** one consumer fixture bundled through the actual universal compiler and executed in both Bun and QuickJS: a native Promise resource enters pending/success/error, nested boundaries recover, a lazy component resolves, and unmount during pending work does not publish stale UI. Verify retained layout/input state and disposal. Clarify native JSX typing for these Solid components. Do not add DOM Portal, SSR, or Fetch emulation merely for API parity.

### P1 — Product-scale lists, input, and desktop performance evidence

**Implemented mechanisms; broader workloads unverified.** Native list/table/tree virtualization, host-owned input, bounded per-frame handoff and performance scripts exist. The [structure benchmark](report.md) proves faster host-tree work, not dropped-frame rates. Current APIs offer estimated item size and scroll commands, and the [component guide](../../docs/gpui-components.md) documents keyed data and stale requests; this should not be described as missing virtualization.

**User impact:** real documents can combine async filtering, variable-height content, IME and resizing in ways that primitive demos do not exercise.

**Acceptance:** choose one representative large data workload and one editor workload. During scroll/filter/append/reorder/resize, preserve row identity, selection/caret and useful scroll position; bound work by visible content; record input-to-present latency and frame cadence under the [measurement contract](../../docs/performance-analysis.md). Include mixed-DPI movement and target-platform IME as desktop qualification cases. Add a regression only for an actual broken invariant.

### P2 — Reusable native service and event-stream patterns

**Application-owned services are supported; a standard service library/stream abstraction is not supplied.** Networking, persistence, secure credential storage and file watching belong to app-owned Bun services or Rust Native Modules under [ADR-0017](../../docs/adr/0017-runtime-engines.md). Component event streams are implemented; typed command results are bounded JSON/bytes, rather than a general resumable service subscription.

**User impact:** applications repeatedly design lifetime, cancellation, progress and reconnect behavior for long-running services.

**Acceptance when demanded by a real app:** provide one production-shaped sample module with typed failures, lifecycle disposal and bounded/coalesced progress; demonstrate subscriber removal and slow-consumer policy. Extract a general subscription abstraction only if two applications need the same contract. Database/network implementations, permission scope and updater policy remain application decisions.

### P2 — Developer diagnostics and application scaffolding

**Partial ergonomics.** Native DivInspector, metadata-only protocol taps, logs, source maps, packaged-consumer tests and HMR already exist. [HMR](../../docs/hot-reload.md) remounts app code into a new epoch and preserves state only through the explicit capture/setup contract. The reviewed tooling does not demonstrate a unified Solid-owner → host-node → native-command inspector or a packaged runtime source-mapped crash workflow.

**Acceptance:** from a minimal external consumer, build/run/package with generated native bindings; reproduce one component error and one native-command error with actionable application source location, Surface identity and request identity. Keep payloads out of logs by default. Document explicit HMR state retention; do not promise preservation of every native editing session across reloads.

### P2 — Internationalization and richer layout/motion only where needed

**Partial public styling surface; desktop internationalization unverified.** [Style](../../packages/solid-gpui/src/style.ts) exposes flex, dimensions, native text and four transition properties (`opacity`, `backgroundColor`, `width`, `height`). It exposes neither grid nor a text-direction/logical-spacing contract nor general transforms. Native widgets have their own behavior; these omissions are not proof native text cannot shape bidirectional text. Existing Unicode fixtures establish some encoding/editing behavior, not full RTL, locale or font-fallback qualification. The [animation investigation](../animation-completeness/issues/02-gpui-animation-surface.md) records why SVG-only transforms cannot serve as generic layout/hit-test transitions.

**Acceptance when required:** a mixed RTL/LTR editor and form with correct caret/navigation, mirrored layout policy, font fallback and locale formatting; a motion case with consistent paint/hit testing and reduced-motion behavior. Add concrete native contracts instead of approximating browser CSS. WebViews, media playback, printing and full browser compatibility are optional product modules, not automatic production blockers.

## Top five priorities and ownership

1. **P0: Desktop qualification.** Framework owns reproducible native smoke and supported-platform evidence; application publisher owns signing identity and chosen distribution policy.
2. **P0: Accessibility.** Framework/GPUI own semantics and adapter fixes; applications own meaningful labels and accessible flows.
3. **P1: Per-call native cancellation.** Framework owns generated API, wire admission and task lifetime; service authors own cooperative cancellation of blocking work.
4. **P1: Application activation/lifecycle.** Framework owns typed events and configurable host policy; applications choose registered schemes, document types and routing.
5. **P1: Solid async-boundary qualification.** Framework owns compiler/runtime interoperability and cleanup/error isolation; applications own loading/error presentation. This is a qualification task until a reproducer demonstrates a defect.

## Suggested next implementation slice

Start with the per-call cancellation contract: it is a concrete framework capability gap with bounded cross-language acceptance, and it makes the recently completed JS cancellation composition useful for native work. In parallel planning, turn desktop and accessibility qualification into a short release evidence matrix. Keep widget expansion below these items unless a real application exposes a missing interaction.
