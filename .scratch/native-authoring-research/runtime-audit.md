# JavaScript event loop, GPUI, and Tokio audit

Date: 2026-09-06. Implementation and acceptance refer to the workspace at that
time. Embedded Bun used pinned source `34cbb9a40b4bd1bd767d134a7065e66c2432a676`,
`nightly-2026-07-20`, and the newly rebuilt macOS dynamic library; an older
library was not substituted for the patched build.

## Application command executor

Previously, GPUI `background_spawn` polled module Futures without a Tokio
context. The replacement `NativeExecutor` in `native/executor.rs` belongs to
the `NativeModules` registry and lazily starts a persistent Tokio multithread
runtime with timer and I/O support.

- Asynchronous factories and their entire Futures execute inside Tokio; synchronous commands execute in the blocking pool. The GPUI foreground does not call `block_on`, and neither JavaScript objects nor GPUI Entities enter Tokio.
- Calls follow JS Promise → protocol bytes → GPUI validation → Tokio → GPUI completion callback → protocol bytes → JS event loop → Promise resolve/reject. Bun microtasks execute subsequent JavaScript.
- Each Surface permits 32 requests and the registry permits 128 actual tasks. The task holds its permit, so cancelling already-running blocking work does not release capacity early.
- Root owns request Tasks. Window closure, epoch reset, and Root Drop clean them up. Tokio JoinHandles abort on drop; old-epoch results are rejected before they can affect a new request ID.
- The last owner calls `shutdown_background` to avoid waiting for threads on the GPUI foreground. Already-running blocking Rust functions follow ordinary Rust/Tokio semantics and cannot be safely killed. Detached child tasks do not automatically inherit parent cancellation.

Sources: [Tokio GUI/synchronous-code bridging](https://tokio.rs/tokio/topics/bridging)
and [JoinHandle detach and abort semantics](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html).

## Complete embedded Bun runtime boundary

The old `setInterval(0)` polling and blocking `sync_channel.send` were removed.
The embedded implementation uses the pinned Bun source's `VmHandle`, `JsPoster`,
`ManagedTask`, event-loop keepalive, and JSC termination trap directly.

| Owner                         | Responsibility and release boundary                                                                      |
| ----------------------------- | -------------------------------------------------------------------------------------------------------- |
| Process-wide Bun owner thread | Initializes JSC's main RunLoop once, preserves its identity while parked, and serializes VM sessions     |
| VM session                    | Exclusive admission shared by all Surfaces; releases admission after VM cleanup                          |
| Rust adapter State            | Owns two bounded byte queues and a C control handle until synchronous execution and all callbacks finish |
| Bun JS thread                 | Exclusively owns JS Strong roots, input dispatch, and JavaScript callback invocation                     |
| Weak VM gate / poster         | Wakes and terminates across threads; rejects and reclaims task payloads after VM closure                 |

Host input coalesces wakeups. A JavaScript turn consumes at most 64 frames or
1 MiB, then enters Bun's after-yield queue so timers and I/O can run. One valid
large frame is delivered whole before yielding immediately. Input received
during startup waits for the first data listener; no polling timer compensates
for lost wakeups.

Output follows real write/drain semantics: a frame returning false has already
been accepted, and drain is posted after the host frees capacity. Input and
output hold separate keepalives so the final backpressured output can finish
after EOF. The input queue allows 4096 frames and a byte budget equal to one
maximum frame. Output's high-water mark is 32 frames or 16 MiB; continuing to
write despite backpressure fails at the hard limit. UI sends do not wait for
a consumer to free capacity, drop frames, or reorder them.

`close_input` emits stdin end/close after accepted input and preserves final
output and asynchronous beforeExit work. Forced shutdown interrupts non-yielding
JavaScript through the JSC termination trap, then the owner tears down the VM
outside execution. Adapter Drop only requests termination; application exit
waits for complete teardown on the GPUI background executor.

Embedded `process.exit(code)` runs exit listeners, records status, and terminates
the current VM without exiting the host process. Cleanup revokes callbacks and
Strong roots before shutting down Workers, networking/timers, and thread-local
pointers. It preserves process-wide HTTP infrastructure and default runtime
options. Later sessions receive unique ScriptExecutionContext IDs, preventing
old messages from reaching a new VM. The resolver invalidates directory,
negative-cache, and configuration data between sessions while reusing stable
directory/Entry storage.

## Root causes found and fixed

1. Cross-thread `EventLoop::wakeup` produced concurrent mutable loop borrows. The fix calls the underlying thread-safe wake through the weak gate.
2. Weak memory ordering between the empty-queue check and clearing `scheduled` could lose a wakeup. Acquire/release atomic exchanges replace it; Loom retains the old protocol's counterexample and the new protocol's exploration results.
3. Recreating threads and reusing the main context ID violated JSC main-thread identity. A persistent owner and unique context IDs replace that design.
4. Main-thread exit cleanup stopped process-wide HTTP support without fully releasing the embedded VM. Dedicated embedded teardown preserves process facilities and releases session resources.
5. Process-global resolver directory and negative caches hid new files. Session boundaries invalidate them and reclaim old configuration; same-path rewrites and subsequently created entrypoints were verified.
6. Bun.serve accessed an empty global CLI Context. Canonical `write_context_no_parse` now initializes process-owned default options and logs before startup without parsing host argv.
7. Default Node conditions selected Solid's SSR exports and produced no render effect. The embedded entrypoint now uses the browser condition, matching process and Vite entrypoints.

## Acceptance

`cargo test -p solid-gpui --features embedded-bun --test host_embedded_counter -- --nocapture`
used the rebuilt real Bun/JSC and sequentially covered these cases in one process:

- Initial output, same-path source rewrites, new entrypoints, and actual Solid counter input/commit round trips.
- 512 inputs before initialization completes; rejection of a second active session; continued queue processing after recovery from a throwing listener.
- 128 consecutive outputs across write(false)/drain, with strict ordering and final-accepted-frame checks.
- EOF/end/close after queue drain, beforeExit scheduling a timer, and natural exit.
- Three consecutive Worker-message, Bun.serve, localhost-fetch, and timer sessions; shutdown with a live Worker, pending fetch, and active server.
- Termination of busy loops in entrypoints and input callbacks, plus infinite microtasks; process.exit(7) running its exit listener exactly once while the host remains alive.
- Immediate shutdown before VM publication, rejected asynchronous entrypoints, and a successful Solid counter round trip after a failed exit.

All 254 Rust workspace `--lib --tests` cases passed. Two doctests passed and one
retained its existing ignore. Strict workspace Clippy with the embedded feature
and package CI passed. The registered `bun run task embedded-check` entrypoint
was executed and passed the real VM matrix and feature Clippy. Package CI includes
generator consistency, types, runtime behavior, packed-package consumers, and
formatting. Raw records: [workspace](workspace-tests.log), [doctests](doctests.log),
[Clippy](clippy.log), [package CI](package-ci.log), and
[embedded-check](embedded-check.log). Tokio also has actual timer/TCP, capacity,
panic, cancellation, and shutdown checks.

Loom only establishes properties of the modeled atomic wakeup protocol; it is
not a memory-safety proof of all Bun/JSC. Native library builds and VM acceptance
were on macOS and do not certify other platforms or release artifacts. The
actual Gallery window passed Rust component updates, successful calls, domain
errors, subsequent recovery, and normal closure; see the
[native acceptance record](gallery-qualification.md).

Build entrypoints were also corrected: nested Bun builds explicitly clear outer
Rust/Clippy wrappers and always use the pinned nightly. CI cache keys include
embedded Rust overlays, and `embedded-check` is registered and executed.
