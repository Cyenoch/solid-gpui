# Runtime transport research

Implementation update (2026-09-07): the native frame bridge and QuickJS application reload are now implemented. The observations below record the research baseline; see [runtime strategy](../../docs/runtime-strategy.md) and [implementation validation](implementation.md) for current behavior.

Date: 2026-09-07. Status: research and proposed direction, not an implemented transport redesign.

## Recommendation

Keep one generated native contract, binary codec, atomic commit model, and asynchronous ownership boundary across all engines. Keep framed stdio for external Bun. Refactor the two embedded engines toward one explicit native frame bridge with bounded admission, pressure notification, and termination semantics; stdio emulation should cease to be the embedded renderer's public transport. Do not introduce shared mutable JS/GPUI objects. Consider ownership transfer for large byte buffers only after measuring copy cost and designing memory accounting.

These recommendations are project-specific deductions from the implementation and primary sources below. No three-engine benchmark was run, and no claim of measured speedup follows from this research.

## Current implementation

| Boundary             | External Bun                                                      | Embedded Bun                                                                                | QuickJS                                                                 |
| -------------------- | ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| JS-facing adapter    | `StdioTransport`                                                  | `StdioTransport` over the embedded bootstrap                                                | `EmbeddedTransport` over `__solidGpuiHost`                              |
| Physical crossing    | Child stdin/stdout pipes; reader/writer paths                     | Owned byte queues and a native C ABI; no OS pipe for renderer traffic                       | Owned byte queues and worker-local native functions                     |
| Encoding             | Four-byte little-endian frame length plus bounded binary envelope | Same frame/codec contract; host commit queue stores payload                                 | Same contract; submit validates a complete frame and queues its payload |
| JS output pressure   | Stream `write(false)` plus bounded TS pending bytes               | Native high-water mark, accepted `write(false)`, real drain, hard overflow; plus TS staging | Submit succeeds or throws; no soft watermark/drain notification         |
| Host-to-JS pressure  | Bounded event queue, no capacity wait on UI thread                | Bounded event queue, no capacity wait on UI thread                                          | Bounded event queue, no capacity wait on UI thread                      |
| Cross-thread objects | Owned Rust event DTOs internally; bytes at process boundary       | Owned bytes; no JS values or GPUI handles                                                   | Owned bytes; every JS value belongs to the worker                       |

Evidence: [transport interface](../../packages/solid-gpui/src/transport.ts), [stdio implementation](../../packages/solid-gpui/src/stdio.ts), [process queues](../../crates/solid-gpui/src/transport.rs), [embedded Bun adapter](../../crates/solid-gpui/src/runtime/embedded.rs), [QuickJS adapter](../../crates/solid-gpui/src/runtime/quickjs.rs), [embedded JS adapter](../../packages/solid-gpui/src/embedded.ts), [ADR-0002](../../docs/adr/0002-embedded-bun-runtime.md).

The implementations are semantically aligned but do not have identical pressure policies. Embedded Bun commits have a 32-frame soft watermark and 64-frame hard bound, plus byte thresholds. QuickJS bounds each queue by 4,096 frames and `MAX_FRAME_LENGTH + 4` bytes. These are implementation parameters, not performance targets. The TS stdio staging queue bounds bytes; its overflow branch throws a `RangeError` directly, whereas native QuickJS overflow also marks the connection failed. A shared bridge should specify terminal behavior explicitly instead of preserving accidental differences. [Bun queue](../../crates/solid-gpui/src/runtime/embedded.rs), [QuickJS queue](../../crates/solid-gpui/src/runtime/quickjs.rs), [TS enqueue](../../packages/solid-gpui/src/stdio.ts).

QuickJS deliberately copies submitted JS bytes into an owned Rust payload outside the shared lock. Incoming events are copied into VM-allocated typed arrays so retained event buffers count toward the VM limit. Replacing those copies with externally backed storage would change ownership and accounting, not just allocation speed. [QuickJS submit and dispatch](../../crates/solid-gpui/src/runtime/quickjs.rs).

## What primary sources establish

**Backpressure is distinct from rejection.** Node streams specify that `write(false)` has accepted the chunk and asks the producer to stop until `drain`. Continuing to write can grow memory; the watermark is not itself a hard limit. This supports the current Embedded Bun accepted-under-pressure semantics, but does not prove it is sufficient for the renderer's end-to-end production rate. [Node stream documentation](https://nodejs.org/api/stream.html#buffering), [writable.write](https://nodejs.org/api/stream.html#writablewritechunk-encoding-callback).

**A JS heap is not a cross-thread application state store.** QuickJS documents that one runtime cannot execute concurrently and separate runtimes cannot exchange JS objects. rquickjs serializes runtime access; enabling parallel access is not concurrent execution of the same heap. [QuickJS-NG C API](https://quickjs-ng.github.io/quickjs/developer-guide/intro/#runtime-and-contexts), [rquickjs](https://docs.rs/rquickjs/0.12.2/rquickjs/).

**Do not overstate JavaScriptCore restrictions.** Apple's public Objective-C JavaScriptCore API permits calls from different threads but serializes access to one VM. That does not make this repository's Bun event loop, internal JSC handles, or GPUI entities safe to use on arbitrary threads. The repository's stronger worker ownership rule follows its embedding implementation. [Apple JSVirtualMachine](https://developer.apple.com/documentation/javascriptcore/jsvirtualmachine), [local Bun ownership decision](../../docs/adr/0002-embedded-bun-runtime.md).

**Direct native interoperation is possible without sharing arbitrary JS objects.** React Native uses JSI to associate JS fibers with native shadow nodes; its native shadow tree is immutable and host-view mutation remains on the UI thread. Adopting that approach here would require a new native object lifetime and rendering design, not replacing a byte queue with pointers. [React Native rendering pipeline](https://reactnative.dev/architecture/render-pipeline), [threading model](https://reactnative.dev/architecture/threading-model).

**Owned external buffers differ from shared mutable buffers.** rquickjs exposes both copying and owned-source ArrayBuffer APIs; its documentation requires exclusive mutable access for mutable backing storage and distinguishes immutable shared sources. Such APIs can avoid a copy, but lifetime, mutation, and finalization requirements remain. Inspection of the installed rquickjs-core 0.12.2 source confirms `new_copy`, `from_source`, and `from_source_immutable`; the latter rejects JS writes and supports immutable shared backing. This confirms API availability, not the accounting or performance of a proposed integration. [ArrayBuffer API](https://docs.rs/rquickjs/0.12.2/rquickjs/struct.ArrayBuffer.html), [pinned dependency](../../Cargo.lock).

**An explicit command bridge is a viable architecture.** Tauri uses asynchronous messages and typed commands, and provides raw binary responses/channels for appropriate data flows. This supports separating ordinary commands from bulk data; it is not evidence that Tauri's exact transport or JSON representation is optimal for GPUI. [Tauri IPC](https://v2.tauri.app/concept/inter-process-communication/), [Rust commands and raw data](https://v2.tauri.app/develop/calling-rust/).

## Proposed design

### One semantic contract, two physical transports

Use the existing generated Native Modules and renderer protocol as the authoritative application boundary. External Bun keeps a pipe adapter; both embedded engines expose a small native bridge for complete frames, subscriptions, pressure, and termination. Bun can retain ordinary logging and runtime services independently of this renderer bridge. Do not add an embedded-only direct command API that bypasses generated validation, Surface identity, revisions, cancellation, or tracing.

This is a replacement design, not another compatibility wrapper around embedded stdio. Update both embedded bootstraps and their entry points together when implemented. Keep the same encoded payload initially so engine changes preserve protocol replay and meaningful development/production parity. A stream needs framing; an embedded message queue already knows its message length, so an internal frame-header bypass is possible but should not split the semantic schema or become a priority without evidence.

### Pressure is part of the renderer scheduler

Define three admission outcomes: accepted, accepted-under-pressure, and terminal rejection. A pressure signal must never cause the accepted frame to be retried. Resume at a documented low-water condition. Keep byte and frame hard caps, including staging buffers and any retained external storage. Do not stack unaccounted queues in the renderer, TS adapter, native bridge, and host.

The current synchronous `Transport.submit(): void` hides recoverable pressure from the renderer. A redesign should expose pressure to its flush scheduler and bound retained changes, rather than merely adding another queue to `EmbeddedTransport`. Preserve atomic commits and sequence order. Coalesce replaceable state before it becomes a protocol revision only when the event/commit semantics permit it; never discard arbitrary encoded patches, clicks, command replies, or cancellation messages.

The UI thread must not wait for queue capacity or execute a synchronous VM callback. Small lock sections are still contention points; avoid calling a serializer, JS, allocator-heavy copy, file logger, or destructor while holding a shared queue lock. Drain work needs a frame/byte/time budget and a real wakeup, with no periodic polling. Existing Bun delivery already yields after bounded batches. [Bun runtime](../../crates/solid-gpui-bun-sys/embedded/runtime.rs), [commit pump](../../crates/solid-gpui/src/host/commit_pump.rs).

Retain a shutdown/control wake path that works even when data queues are full. If cancellation needs separate reserved admission, define ordering against its request explicitly; priority must not let cancel overtake request creation. Epoch retirement must reject stale completions and release subscriptions/resources. Native commands already have a bounded executor and cancellation protocol; extend those rather than inventing another RPC system. Already-running blocking Rust closures are not forcibly cancelled. [Native executor](../../crates/solid-gpui/src/native/module.rs), [renderer cancellation](../../packages/solid-gpui/src/renderer/root-container.ts).

### Object ownership and bulk data

Keep GPUI entities on the UI thread and JS values on the VM thread. For reusable Rust resources, prefer opaque typed identifiers validated by owner/session/generation; define explicit release and host cleanup on Surface/session retirement. JS wrappers can provide convenient methods while commands retain transport-independent semantics. A wrapper is not shared object identity.

For large immutable assets, first avoid repeatedly sending the same bytes: keep the resource in Rust and send a descriptor or handle. If copying remains significant, prototype exclusive ownership transfer or an immutable externally backed buffer, with host-wide accounting independent of VM allocator limits. A lease must have bounded total bytes, exactly-once release, cancellation/teardown cleanup, and no JS mutation of memory concurrently read by Rust. GC must not be the sole mechanism limiting scarce native resources.

Do not start with SharedArrayBuffer, raw pointers, or a lock-free ring. These introduce synchronization, publication ordering, wraparound, detachment, and shutdown obligations; they do not remove serialization of rich JS values. Bun's FFI pointer/ArrayBuffer facilities likewise do not provide a thread-safe GPUI object model. [Bun FFI](https://bun.com/docs/runtime/ffi).

## Alternatives and decision gates

| Alternative                                                   | Potential benefit                                                          | Decision                                                                                                                         |
| ------------------------------------------------------------- | -------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Binary framed stdio                                           | Simple child lifecycle, existing development support                       | Retain for external Bun; reserve stdout for protocol and stderr for logs                                                         |
| Unified embedded native frame bridge                          | Clear API, consistent pressure semantics, removes renderer stdio emulation | Recommended next transport refactor                                                                                              |
| Decode JS DTOs directly into owned Rust messages on VM thread | Avoid binary encode/decode inside one process                              | Prototype only if codec cost dominates; preserve generated schema and validation; crossing JS properties may itself be expensive |
| Immutable native objects exposed through JSI-like wrappers    | Avoid repeated object serialization                                        | Architectural fork requiring ownership/renderer redesign; not justified by current measurements                                  |
| Shared memory / zero-copy bulk path                           | Lower large-payload copy cost                                              | Narrow measured workload only, with explicit leases and accounting                                                               |
| Unix sockets, named pipes, WebSocket                          | Independent endpoint lifecycle or remote tooling                           | No demonstrated advantage for a locally spawned child; use a separate dev-control channel only where needed                      |

## Meaningful verification and performance plan

This is a plan, not executed evidence. Follow the [repository measurement workflow](../../docs/performance-analysis.md). Pin the same UI bundle, Rust revision, release profile, machine, window geometry, workload, and monitor setting. Run serially after builds finish; limit individual measurements to five minutes. Account for the Bun child as well as the host when reporting process-mode memory.

Use three workloads: small event-to-patch interaction, a large atomic snapshot followed by small patches, and sustained replaceable updates with a deliberately slow consumer. Record codec time, copy bytes/allocations, queue residence p50/p95/p99, peak queued/retained bytes, wakeups, UI-thread lock time, throughput, and native input-to-present separately. Include pure native scrolling as a counterexample: it should not improve solely because the JS transport changes. Compare the current bridge with the unified bridge before attempting a codec or external-buffer variant. Do not infer FPS from queue throughput.

Keep only invariant tests that can catch material failures:

1. Slow-consumer burst: exact accepted ordering, accepted-under-pressure frame delivered once, real resume, hard memory bound, and responsive host shutdown.
2. Retirement with pending native work: stale responses cannot affect a replacement epoch; cancellation and blocking-task limits match documented behavior.
3. Malformed/oversized frame and transport failure: one explicit terminal outcome, no partially applied commit, no hidden unlimited staging.
4. If ownership transfer is implemented: mutation/detachment behavior, finalization exactly once, retained-byte cap, and runtime destruction while buffers remain leased.

The result should choose the smallest design that measurably improves the target workload while preserving order, bounded work, and engine parity. No numerical speedup is established by the architecture alone.
