# ADR-0002: Embed Bun on a dedicated runtime thread

- **Status:** Accepted
- **Date:** 2026-08-24
- **Updated:** 2026-09-06

## Context

Process mode is simple and portable but requires a separate Bun process. A packaged macOS application may instead host Bun/JSC in-process. JavaScriptCore values and GPUI handles are thread-affine and must not cross between the runtime thread and GPUI's main thread.

## Decision

`solid-gpui::runtime::embedded` owns a process-scoped Bun thread and admits one
VM session at a time. JSC initializes its main RunLoop on this thread, so the
thread survives session teardown. Every application Surface shares the active
session. A new session is admitted only after the previous VM is destroyed.
Only bounded owned protocol bytes cross between Bun and GPUI.

The build script fetches Bun revision
`34cbb9a40b4bd1bd767d134a7065e66c2432a676`, applies
`crates/solid-gpui-bun-sys/bun_embed.patch` and the `embedded/` source overlays,
builds the native graph using `nightly-2026-07-20`, and links the generated
library. `SOLID_GPUI_BUN_CACHE` can select a reusable build directory; it does
not bypass revision verification, patch application, or rebuilding.

The C ABI owns a control handle with create, run, wake, terminate, and destroy
operations. Cross-thread control holds Bun's weak `VmHandle` and `JsPoster`,
never a JS value or a mutable event-loop reference. A coalesced `ManagedTask`
delivers queued input on the JS thread; each turn processes at most 64 frames
or 1 MiB, then yields through Bun's event loop. One larger legal frame is
delivered atomically. An input keepalive holds the loop open without a polling
timer, and input queued during module loading waits for a data listener.

The bootstrap exposes an explicit `EmbeddedTransport` native frame bridge.
It does not replace `process.stdin` or `process.stdout`. A commit write
returns true when writable, false when the frame was accepted under pressure,
and throws when closed or over its hard bound. Removing queued commits posts
a real drain notification; an output keepalive retains a pending drain even
after input EOF. Host input never waits for queue capacity. Its byte and frame
caps produce an explicit transport error on exhaustion.

`close_input` drains accepted input before the bridge input termination callback and permits final
output and normal beforeExit work. `request_shutdown` requests JSC termination
and wakes the native loop without waiting. `shutdown` additionally waits for
teardown on a background executor. Fatal host paths request termination before
exiting; they never join the VM on the UI thread. Adapter Drop only requests
termination. Termination interrupts busy JS and microtask loops;
`process.exit` records the VM exit code without exiting the native host.
After JS frames unwind, teardown revokes delivery, releases Strong roots,
joins VM work, closes VM resources, and clears VM/thread-local pointers.
Process-owned services remain available to the next session. Resolver cache
generations are invalidated at the quiescent session boundary, so changed or
new source files are observed without retaining obsolete configuration data.

The entry is evaluated once per session with the browser export condition
required by Solid's universal renderer. Development reload remains the Vite
protocol; embedding does not add another watch or refresh protocol.

`ProcessAdapter` remains the default and owns child stdio on dedicated reader/writer paths. Ordered queues are bounded by payload count and bytes; overflow and transport failure are fatal rather than silently dropping revisions or events.

## Consequences

- Bun embedding is macOS-only until another platform has a proven Bun/JSC embedding path. [ADR-0017](0017-runtime-engines.md) adds a separate QuickJS UI runtime without changing this Bun decision.
- Runtime shutdown must release JavaScript values before destroying the VM.
- GPUI handles, Solid owners, signals, and closures never cross runtime channels.
- Both adapters implement the same Snapshot/Patch/Event/Command contract.
- Actual VM tests cover queued startup input, handled listener failure,
  write/drain ordering, EOF/beforeExit, HTTP/timers across repeated sessions,
  busy-loop and microtask termination, process.exit, rejected entries, and
  subsequent renderer roundtrips (`tests/host_embedded_counter.rs`).
