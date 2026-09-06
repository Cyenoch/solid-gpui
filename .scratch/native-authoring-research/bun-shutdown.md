# Embedded Bun VM termination and destruction

Research baseline: Bun `34cbb9a40b4bd1bd767d134a7065e66c2432a676`, read from `target/debug/build/solid-gpui-bun-139d42991578c823/out/bun-source`. The checkout's `src/runtime/embedded.rs` and `VirtualMachine::teardown_embedded` are local embedding patches; the Worker/VmHandle implementation cited below is unmodified upstream at that commit. This is source research and a concrete patch design, not a claim that the embedding implementation has already passed the tests below.

## Decision

Use Bun's own `VmHandle::request_termination` and its complete Worker-style teardown sequence. Export a counted, opaque embedding control handle to the host. Keep JS and all VM destruction on the dedicated owner thread. The host may request stop from any thread; it may never call JS or destroy a VM from that thread.

A session ends only when its owner has completed native teardown and returned its completion result. Do not detach a stuck thread, abandon a live VM, free a VM under live callbacks, or invoke Bun's process exit path. `process.exit` inside an embedded VM must request that session's termination and unwind script, leaving GPUI alive.

## What upstream already provides

### Cross-thread stop with lifetime protection

- `VirtualMachine::handle(&self) -> crate::VmHandle` is public and clones a counted handle. `VmHandle` owns an `Arc<Shared>`, not the VM. Weak accesses are gated against `Closed`, and teardown waits for all active weak accesses before freeing the VM. This is the correct cross-thread capability; a raw `JSC::VM*` is not. [VirtualMachine.rs](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/VirtualMachine.rs#L857), [VmHandle.rs](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/VmHandle.rs#L300).
- `VmHandle::request_termination(&self)` at `VmHandle.rs:387` is any-thread and idempotent: `Open -> Stopping`, enter the lifetime gate, request a JSC termination trap, and wake the real Bun event loop. After `Closed`, no VM access occurs. Native-to-JS calls stop accepting script as soon as the handle leaves `Open`. [Implementation](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/VmHandle.rs#L368).
- The binding `void JSC__VM__notifyNeedTermination(JSC::VM*)` directly calls `vm->notifyNeedTermination()`. Upstream explicitly documents that VMTraps firing is concurrent-safe and must not acquire/release the API lock, including when already inside a host call. Releasing it there would run a microtask checkpoint in the middle of the host call. [bindings.cpp:5220](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/bindings/bindings.cpp#L5220).
- `VmHandle::wake(&self)` at line 400 is also any-thread and safely becomes a no-op after close. The wake agent is investigating normal bridge readiness; shutdown must use this same lifetime discipline.

**Mandatory preparation:** before publishing an embedded main VM handle to another thread, its owner must create the termination exception singleton while holding the API lock. The upstream Worker constructor does `vm.ensureTerminationException()` at `ZigGlobalObject.cpp:622`; the main VM normally postpones it because it is not remotely terminated. The existing public Rust `VM::termination_exception(&self) -> JSValue` at `VM.rs:120` reaches `ensureTerminationException`, so the embedding setup can materialize it without adding another JSC binding. [Worker initialization](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/bindings/ZigGlobalObject.cpp#L622), [VM.rs](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/VM.rs#L118).

### Destruction is a sequence, not a flag

`VirtualMachine::teardown` (current checkout lines 1794–1948; upstream line numbers are before the added 21-line `teardown_embedded`) is shared by Workers and process exit:

1. Forbid script and clear microtasks/modules. Stop WebCore active objects, including child Workers, MessagePorts, WebSockets, and listeners. Sweep cancellable pool jobs, native servers/sockets/watchers/subprocess handles, fetch/S3/build requests, socket groups, and DNS.
2. Cancel user timers and GC timers; join child workers; close this VM's SQLite databases. `VmHandle::close_and_wait` drains returning completions **without running their JS** while the heap is alive and waits for all off-thread tickets. Sweep again after native continuations release resources.
3. Release JS handles, run final collection/destructors, destroy JSC VM. Drain sockets closed by finalizers, then close timer-loop handles.
4. For Worker lifetime: fold keepalive deltas, detach socket groups, clear loop's JSC pointer, free per-thread uSockets loop, close Windows libuv loop.
5. Destroy Rust VM fields/runtime state. The Worker caller then frees the VM's console/log/raw allocation and its own arena/environment state.

[VirtualMachine teardown](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/VirtualMachine.rs#L1754), [Worker shutdown](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/web_worker.rs#L1007).

Off-thread Bun operations hold `Ticket`s. The shutdown owner cannot destroy the heap until every ticket returns; the ticket's final drop and queue posts wake the drain condition variable. The unbounded wait is an explicit upstream ownership invariant, not an excuse to leak. Cancellable operations are actively cancelled before waiting, and a debug build reports the source locations of outstanding tickets if progress stalls. [VmHandle.rs:1–29 and 433–493](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/VmHandle.rs#L433).

## Defects found in the original local embedding patch

1. `teardown_embedded` uses `Teardown::MainThreadExit`. That branch calls the process-wide `bun_http::shutdown_for_exit()`. The HTTP daemon then parks forever, and the global shutdown bit is not reset. A later VM using `fetch` cannot be supported by this branch. It also intentionally skips the complete loop close that process exit normally lets the OS perform. [HTTPThread.rs:1310–1325 and 1353–1411](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/http/HTTPThread.rs#L1310).
2. `VirtualMachine::destroy` clears fields; it does **not** free the raw VM allocation, owned log, or console. Worker `shutdown` performs these steps explicitly. The embedding patch currently does not. [destroy implementation](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/VirtualMachine.rs#L4721), [Worker raw deallocation](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/web_worker.rs#L1045).
3. `Bun__Process__exit` only distinguishes Worker from non-Worker; the current embedded main VM takes `vm.on_exit(); vm.global_exit()`, which exits the entire GPUI process. Replacing the JS-visible function alone would miss native paths. [node_process.rs:68](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/runtime/node/node_process.rs#L68).
4. `vm.load_entry_point(...).map(|_| ())` discards the returned evaluation promise. `Ok(promise)` does not mean module success: the loader explicitly returns `Ok` for a rejected promise. The owner must observe the promise's rejection/fulfillment and report a rejected entry once; otherwise startup errors can be reported as success. Worker `spin` has the correct pattern, including asynchronous top-level-await rejection. [Loader](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/VirtualMachine.rs#L2816), [Worker entry observation](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/web_worker.rs#L894).
5. A loop conditioned only on `is_event_loop_alive()` is insufficient. Once stop is requested, timers or in-flight requests may still keep the loop alive, while script no longer runs to release them. Check stop immediately after each tick/poll and enter teardown; it closes those resources. Do not call `beforeExit` after host termination. [Worker spin](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/web_worker.rs#L954).
6. The callback wrapper's zero-delay interval means closing input merely cancels the synthetic polling interval. It does not terminate arbitrary application intervals, pending requests, or busy script. Shutdown must be a native control path independent of the event mailbox.

## Smallest sound patch shape

### 1. A host-owned control capability

Inside Bun's embedding layer, create an opaque `EmbeddedControl` before VM startup with:

- a stop-request latch (for shutdown racing with VM construction),
- a lock-protected `Option<VmHandle>` (published only after initialization/termination-exception setup),
- owned/thread-safe reference counting for the ABI lifetime.

`request_stop` sets the latch, clones the published handle while holding the lock, releases the lock, and calls `handle.request_termination()`. Cloning before release avoids any raw pointer lifetime race. If there is no published VM yet, startup sees the latch and exits before running the entry. No lock is held while JSC is called or while the VM is destroyed.

At shutdown the owner unpublishes the handle, performs teardown, marks completion, and only then permits callback contexts and transport storage to be dropped. A late stop request holding an old `VmHandle` is harmless because the handle has its own close gate. This is the same reason Bun's Worker publishes a handle instead of a raw VM pointer. [Worker request and publication discipline](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/web_worker.rs#L550).

### 2. An explicit VM-local exit mode

Mark an embedded VM as host-owned independently of Node's `isMainThread` identity. In native `process.exit`, embedded mode records the requested exit code and requests the same termination trap, then unwinds to the owner's outer loop. It must never call `global_exit`.

Also audit `VirtualMachine::uncaught_exception`: two main-thread branches invoke the `process_exit` hook and then panic if it returns. Embedded mode must use the VM-local stop funnel for those branches as well. A mere edit to the normal `node_process::exit` branch would turn those exceptional paths into process aborts. Preserve three outcomes: host cancellation, script `process.exit(code)`, and unhandled entry/runtime error. [Uncaught exception branches](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/VirtualMachine.rs#L1520).

A host-forced stop runs no further user JS, including `beforeExit` and exit handlers. Natural drain may run `beforeExit`/`exit` under the existing semantics; a script-initiated exit must not permit subsequent JS after the exit call. The exit reason must survive clearing JSC's temporary termination exception during cleanup.

### 3. A VM-local teardown kind

Add `Teardown::Embedded` (or an equivalent explicit ownership classification) with Worker cleanup semantics:

- run the same stop, ticket-drain, finalization, loop-close, runtime-state destruction;
- never park the global HTTP thread or call `Global::exit`;
- reuse `WebWorker__teardownJSCVM` or rename its shared VM-only primitive; it does not require a Worker object;
- reclaim VM raw storage/log/console after teardown and clear all relevant TLS/main-VM references.

`VMHolder::set_vm(None)` alone does not clear `CACHED_GLOBAL_OBJECT`. The local embedded main mode also published `MAIN_THREAD_VM`. Make the reset an owned-VM operation inside `VirtualMachine.rs`, with an expected-pointer compare/exchange for the global slot. Reset the cached global and `IS_MAIN_THREAD_VM` before allowing another VM session to start. Never zero another live VM's global slot. [VMHolder/global slots](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/VirtualMachine.rs#L448).

Keep API-lock ownership explicit. Worker takes `JSC__VM__getAPILock` as a raw binding for the lifetime of the VM, and destroys the VM without returning through a borrowed RAII guard. Teardown must occur outside all JSC/script frames and after any owned `Strong` for the entry/callback has been dropped. Do not retain `&mut VirtualMachine` across reentrant JS or across destruction. Do not drop a Rust lock guard pointing to a destroyed `JSC::VM`. [Worker API lock](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/web_worker.rs#L657), [JSC VM destruction](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/bindings/ZigGlobalObject.cpp#L4334).

### 4. Process lifetime and repeated sessions

JSC initialization already has `std::call_once`, and the first initializing thread becomes WTF's main thread. Separate this process lifetime from per-session VM lifetime. A process-scoped dedicated Bun owner thread can initialize JSC once and create/destroy successive VM sessions without leaving WTF's main-thread identity pointing at a terminated startup thread.

The existing `Zig__GlobalObject__destructOnExit` calls `runLoop->threadWillExit()`, while `WebWorker__teardownJSCVM` only destroys the VM. On a persistent owner thread, only the actual thread exit should call the thread-exit hook; do not mark the thread dead after each session. Validate the real JSC RunLoop timer cleanup by repeated session tests. [Initialization](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/bindings/ZigGlobalObject.cpp#L285), [Distinct destructors](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/bindings/ZigGlobalObject.cpp#L4363).

An engine that permanently owns its scheduling thread is a deliberate process-scoped resource. The per-session VM, ports, timers, callbacks and queues still must all be destroyed. Returning a session-close result while its VM is merely detached does not meet this contract.

## Meaningful qualification cases

Run all cases through the actual linked embedded Bun library in one host process. A child test executable can act as an external watchdog for the test harness, but process mode is not an implementation substitute.

1. Host stop during `while (true) {}` in the initial module, in an event callback, and in a self-rearming microtask chain. Verify session completion and that subsequent VM code runs in the same surviving host.
2. Host stop while Bun is asleep with no JS work, with a long timer, with a recurring interval, and with a permanently pending top-level await. Ensure shutdown is awakened immediately by native control and does not wait for a JS timer.
3. Start pending fetch/DNS/file work, Workers and open sockets, request stop, then create a second VM that fetches successfully. This distinguishes complete VM cleanup from the broken global-HTTP-park path.
4. `process.exit(7)` from module body and callback, a throwing `uncaughtException` handler, and delayed top-level-await rejection. Verify the host survives, exit/error cause is accurate, no code after exit runs, and no stale session event is delivered.
5. Race stop with VM initialization and handle publication; repeat start/stop sequentially with retained stale control handles. Verify no use-after-free, no lost stop, callback count stays zero after teardown completion, and per-session resource growth plateaus. Use ASan/LSan where the Bun build supports them and assert live session/handle counts in the bridge.

JSC termination interrupts JS at engine safepoints. Arbitrary native addon code that neither returns nor participates in cancellation cannot be asynchronously destroyed safely in the same process; that is a native ownership fact. For the framework's own callbacks and supported Bun operations, the implementation must preserve the cancellation/wake/ticket contracts above and verify their real shutdown paths. No universal arbitrary-FFI deadline should be claimed without such cooperation.

## Implementation checkpoint

The initial patch defects above have now been addressed in the repository's `crates/solid-gpui-bun-sys/bun_embed.patch` and `embedded/lifecycle.rs`; the wake/control ABI lives in the separately owned `embedded/runtime.rs` overlay. The old full-file `src/runtime/embedded.rs` patch and old polling/eval exports were removed.

Implemented ownership details include `Teardown::Embedded`, native `process.exit`/fatal-error termination, a prepared termination singleton, raw API-lock lifetime, complete VM/loop cleanup, clearing cached globals and raw allocation ownership, and a signal-safe access gate so process signal delivery cannot retain a retiring main-VM pointer. VM initialization publishes that pointer only after all VM/loop fields exist, and cleans the initialized fields/TLS if runtime-state construction fails before JSC exists.

The lifecycle exposes `on_exit(&mut self)` separately from `finish(self)`: exit handlers run with their bridge still alive, then the bridge releases Strong roots, then the VM is destroyed. The VM stop gate suppresses user exit callbacks after a host-requested stop. Process initialization is an unconditional linkable helper with internal `Once`, because the build first links Bun's ordinary executable before linking the embedding feature.

The real linked-Bun qualification subsequently passed, including repeated HTTP/timer/Worker sessions, a pending fetch at teardown, busy entry and event callbacks, infinite microtasks, process.exit, entry rejection, immediate termination, and a new renderer roundtrip. See [the final audit](runtime-audit.md) and [actual matrix log](embedded-vm-tests.log); the larger qualification list above records research directions beyond the exact executed matrix.


## Repeated-session filesystem isolation

The real single-process matrix exposed a third-session regression: a new temporary directory and entry module created after two completed teardowns existed on disk but Bun reported "Cannot find module". Bun's `FileSystem`, directory listings and DirInfo metadata/negative results are process-global, while a fresh Resolver starts with generation zero. A file-specific cache bust would only hide this instance and leave package/tsconfig and negative directory results stale.

At the quiescent session boundary, `Resolver::begin_embedded_session` now invalidates both layers using an epoch stored beside each BSSMap index. The key-to-slot association remains, so reconstruction reuses the old DirInfo and DirEntry allocation; directory refresh also reuses same-name Entry slots. Negative results receive the new epoch on their next real lookup. Existing generation refresh still operates inside a session.

Cache validity and cache storage have separate lifetimes. Old DirInfo reference fields are reset before reclaiming owned final TSConfigJSON allocations and the PackageJSON arena, so changed package exports/tsconfig settings are parsed anew without retaining every old parse. TSConfigJSON::destroy currently logs then drops its Box; the boundary uses that existing function. The persistent owner may call this only after previous VM teardown has joined workers, closed the VmHandle ticket gate, and released all JS roots.

The parent reports the actual matrix now passes new-file creation between sessions, early input, EOF, beforeExit, handled exceptions and queue pressure. Full network/Worker/termination qualification is still ongoing; the first Bun.serve + fetch scenario exposed a separate null-reference panic being traced by the runtime owner.
