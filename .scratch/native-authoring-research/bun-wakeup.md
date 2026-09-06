# Embedded Bun: cross-thread delivery without polling

Pinned source: `oven-sh/bun@34cbb9a40b4bd1bd767d134a7065e66c2432a676`.
Read on 2026-09-06 from the build cache and verified against the same GitHub commit.
`git status` confirms the wakeup/queue/handle files cited below are unchanged upstream files; the existing `src/runtime/embedded.rs` is our patch, not an upstream API.

## Decision

Use Bun's **`JsPoster` + `ConcurrentTask` + `ManagedTask::new_owned`** as the host-to-JS door. Keep the VM/JSC on its dedicated thread; drive Bun's real `tick` and `auto_tick_active` there. Tokio owns its own reactor and sends owned byte messages through bounded queues. Do not enter JS from Tokio or GPUI, and do not pump either reactor from the other's callback.

This is an implementation primitive already used by Bun, not a stable external embedding ABI. Expose our small, versioned C ABI around it in the pinned patch; no new runtime dependency/package is needed.

## Exact primitives and guarantees

| Purpose | API / owner | Evidence |
| --- | --- | --- |
| Capture JS-loop destination | `VirtualMachine::js_poster(&self) -> bun_event_loop::JsPoster`, on JS thread, captures current loop kind | [VmHandle.rs:869–873](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/VmHandle.rs#L869-L873) |
| Deliver from any thread | `JsPoster::post(&self, NonNull<ConcurrentTask>) -> Posted`; `JsPoster: Send + Sync + Clone` | [AnyEventLoop.rs:537–595](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/event_loop/AnyEventLoop.rs#L537-L595) |
| Queue and wake | `VmHandle::post(kind, task)` uses an active-access gate; `Shared::deliver` pushes the MPSC queue and wakes the loop, or the teardown condition variable while draining | [VmHandle.rs:171–187, 325–342](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/VmHandle.rs#L171-L187) |
| Task payload | `ManagedTask::new_owned<T>(*mut T, fn(*mut T) -> JsResult<()>) -> Task`; wrap using `ConcurrentTask::create(Task)` | [ManagedTask.rs:19–88](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/event_loop/ManagedTask.rs#L19-L88), [ConcurrentTask.rs:267–285](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/event_loop/ConcurrentTask.rs#L267-L285) |
| Refusal | `Posted::Refused(task)` returns ownership; `ConcurrentTask::release_refused(task)` frees both the carrier and a ManagedTask, including its owned context | [ConcurrentTask.rs:314–348](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/event_loop/ConcurrentTask.rs#L314-L348) |
| Unrun queued task | `Taskable::release_unrun`; the VM releases queued tasks before destroying the JSC heap | [ConcurrentTask.rs:135–163](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/event_loop/ConcurrentTask.rs#L135-L163), [event_loop.rs:775–848](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/event_loop.rs#L775-L848) |
| JS callback root | `Strong::create(JSValue, global)`, `get`, `Drop`; explicitly `!Send + !Sync`, drop on JS thread while heap lives | [Strong.rs:1–59](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/Strong.rs#L1-L59) |
| Fair continuation | `EventLoop::enqueue_task_after_yield(Task)`, promoted only before the next I/O/timer turn | [event_loop.rs:70–75, 862–882](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/event_loop.rs#L862-L882) |
| Native-call checkpoint | A queued task may use `JSValue::call` and return `JsResult`; task dispatch folds the exception and drains nextTick/microtasks after each task. Calls outside that queue require `EventLoop::run_callback`. | [dispatch.rs:599–632](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/runtime/dispatch.rs#L599-L632), [event_loop.rs:405–450](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/event_loop.rs#L405-L450) |

Two ownership traps matter:

- `ManagedTask::new_owned` cleans the context **when never executed**. On execution, the callback itself must reconstruct/drop the owned box (`bun_core::heap::take(ptr)`), including its error path. The wrapper only frees the ManagedTask. [ManagedTask implementation](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/event_loop/ManagedTask.rs#L26-L44).
- A permanent host bridge must hold a weak `JsPoster`/`VmHandle`, **not a `Ticket`**. Tickets are counted in-flight VM work and shutdown waits for them. Holding one in an object released after shutdown deadlocks teardown. The producer's payload therefore contains only host-owned bytes/Arc state; no JSC references. [VmHandle lifetime contract](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/VmHandle.rs#L1-L31).

## Minimal patch shape

1. On VM thread initialization, create the poster and a separate opaque host signal handle. Register the JS event delivery function through a host function and retain it in JS-thread state with `Strong`. The cross-thread handle contains only `JsPoster` and thread-safe signal state.
2. Transfer the signal handle to the host in a ready callback. A host queue push completes before `signal.notify()`. Events arriving before ready stay in the bounded queue; ready immediately signals once if queued data exists. Handle shutdown-before-ready as a terminal state.
3. `notify` coalesces outstanding notification tasks using an atomic scheduled bit. It posts `ConcurrentTask::create(ManagedTask::new_owned(Box<Arc<SignalState>>, deliver))`. Handle `Refused` immediately with `release_refused`.
4. The task executes only on the Bun thread, retrieves the registered JS callback, releases every TLS/RefCell/Mutex borrow **before** calling it, and delivers owned frames. Do not hold a host lock or Rust `&mut` state across JS execution: JS may synchronously send a commit, unsubscribe, close the transport, or run a nested loop.
5. Bound each delivery turn by frames **and** bytes (for example 64 frames / 1 MiB; tune from measurement). For simpler exact microtask semantics, make one task deliver one frame, and use a yielded continuation when more remain. If batching multiple frames into one callback, document that the batch is one JS task and ensure its callback has a finite budget.
6. If more remain, keep scheduled=true and use `enqueue_task_after_yield`, not ordinary `enqueue_task`: the latter drains until empty and self-enqueue can starve timers and I/O. The normal concurrent refill cap of eight does not rescue an infinitely self-refilling ordinary task queue. [tick and queue loops](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/event_loop.rs#L649-L720), [ordinary drain](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/runtime/dispatch.rs#L611-L632).
7. Close the lost-wakeup race when the queue becomes empty: either clear scheduled and recheck queue under its own synchronization, or compare a producer generation captured **before draining** after clearing scheduled. A plain `read empty; scheduled=false` loses a producer notification in between. Producers increment generation only after committing the frame to the queue. Continue through a CAS that elects exactly one scheduler.
8. Keep one explicit event-loop keep-alive while the host input is subscribed, using `ref_keep_alive`/`unref_keep_alive` on JS thread. This replaces the accidental keep-alive currently provided by `setInterval(0)`. Wake notification does not itself preserve VM liveness. Balance the ref exactly once at input close before exiting. [EventLoop keep-alive](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/event_loop.rs#L1073-L1087).
9. Drop the JS callback Strong and clear host callback TLS before `teardown_embedded`. The opaque poster may outlive VM shutdown safely, but the C ABI caller must still keep its handle allocation alive for each call; free the handle only after producers have stopped/joined or use explicit retain/release.

### Tiny task example (inside Bun patch)

```rust
fn post_signal(poster: &JsPoster, state: Arc<SignalState>) -> bool {
    let payload = bun_core::heap::into_raw(Box::new(state));
    let task = ManagedTask::new_owned(payload, deliver_signal);
    match poster.post(ConcurrentTask::create(task)) {
        Posted::Queued => true,
        Posted::Refused(task) => {
            unsafe { ConcurrentTask::release_refused(task) };
            false
        }
    }
}

fn deliver_signal(payload: *mut Arc<SignalState>) -> JsResult<()> {
    let state = unsafe { bun_core::heap::take(payload) };
    // Only now, on JS thread, consult thread-local JS callback and IO table.
    // Retrieve callback value, release borrow, then call; return its JsResult.
    deliver_bounded_turn(&state)
}
```

Imports in this pinned tree: `bun_event_loop::{JsPoster, Posted}`, `bun_event_loop::ConcurrentTask::ConcurrentTask`, `bun_event_loop::ManagedTask::ManagedTask`. Bun's `bun_jsc` also reexports these module namespaces. Example is a design skeleton, not a compiled patch.

## One upstream aliasing problem to fix in the patch

`JsPoster`'s queue/shutdown gate is a suitable ownership mechanism, but its wake implementation currently passes through `EventLoop::wakeup`, which on POSIX calls `vm_ref().platform_loop_opt() -> Option<&mut PlatformEventLoop>` and then `Loop::wakeup(&mut self)`. Its own comment acknowledges unchanged aliasing soundness. The platform event-loop thread can already be parked in a call holding that same loop. [event_loop.rs:1043–1062](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/event_loop.rs#L1043-L1062), [VirtualMachine.rs platform accessor](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/jsc/VirtualMachine.rs#L1046).

Use the already exported raw `bun_uws::us_wakeup_loop(loop_ptr)` from this one wake path, without materializing `&mut Loop` off-thread. Bun explicitly exports it for that reason and calls the double-`&mut` pattern UB. Preserve the `VmHandle` gate; using the raw wake alone would remove lifetime protection. On POSIX obtain the fixed loop pointer through raw per-field access; on Windows use its stored `uws_loop` pointer, never the sender thread's TLS `Loop::get()`. [raw wake contract](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/src/uws_sys/Loop.rs#L603-L610).

The C function uses release-atomic pending-wakeup publication followed by the platform async wake primitive. The poll consumes the pending count with acquire ordering before deciding whether to sleep. This is the existing event-driven mechanism; a timer polling bridge is unnecessary. [loop.c:164–169](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/packages/bun-usockets/src/loop.c#L164-L169), [epoll_kqueue.c:496–533](https://github.com/oven-sh/bun/blob/34cbb9a40b4bd1bd767d134a7065e66c2432a676/packages/bun-usockets/src/eventing/epoll_kqueue.c#L496-L533).

## Backpressure and acceptance criteria

The Bun concurrent task queue is unbounded; one task per host packet simply moves unbounded growth into Bun. Keep the data in a bounded host queue and coalesce the notification. Once credit is exhausted, suspend the Tokio producer asynchronously; never block the GPUI or JS thread waiting for queue space. The JS→host synchronous callback must distinguish accepted-with-backpressure from closed/rejected; Node stream `write(false)` means data was accepted and producers wait for `drain`, not that a frame may be discarded.

The replacement should prove:

- An idle bridge has no repeated drain calls or timer ticks; a delayed native event wakes JS and resolves its Promise.
- JS timers, Promise continuations, `fetch`, and host replies progress while both directions are continuously busy.
- Accepted frames preserve order/count through saturation; queue frame/byte ceilings hold; producers unblock on close.
- Close before ready, close while idle, close with queued delivery, close during callback, late post after VM destruction, and repeated startup/shutdown all reclaim payloads and terminate.
- The empty-drain/producer race is exercised deterministically, not only with sleep-based stress.
- A long-lived host handle does not count as a Bun Ticket; teardown completes after outstanding Bun I/O settles/cancels.

These are correctness gates for implementing the bridge. This source research alone is not proof that the existing embedded implementation satisfies them.

## Implementation audit: lost wakeup ordering

The first overlay used a release store to clear `scheduled`, followed by an acquire read of `flags`. That is insufficient across the two atomics: a producer may publish flags but still observe the old scheduled=true, while the consumer fails to observe the new flags. The overlay now clears with `scheduled.swap(false, AcqRel)`. If a producer's scheduled RMW preceded that clear, its release synchronizes with the consumer's acquire; otherwise the producer observes false and schedules delivery itself.

The bounded Loom 0.7.2 model in `wakeup-loom/src/lib.rs` finds the old no-task/pending-frame execution and accepts the revised ordering across its explored executions. Reproduce with `cargo test --manifest-path .scratch/native-authoring-research/wakeup-loom/Cargo.toml -- --nocapture`: 2 tests pass, including the old version's expected counterexample. This models the atomic notification handoff only; actual VM, queues and teardown remain covered by the native integration test. [Loom primary documentation](https://docs.rs/loom/0.7.2/loom/).

EOF is now a half-close of input. Output stays writable for final responses and exit listeners. A separate keep-alive is held from the first `write(false)` until its drain notification, so a final backpressured response cannot be cut off by natural exit after input EOF.

## Implementation audit: runtime process options

The real local HTTP fixture exposed a separate startup requirement. `Bun.serve` creates its request timeout-warning handler through `NewServer::should_add_timeout_handler_for_warning`, which reads `cli::Command::get().debug.silent`. The embedded path had never published `GLOBAL_CLI_CTX`; its first local request therefore panicked at `src/runtime/cli/mod.rs:822` with `null reference produced`. The same pinned ordinary Bun CLI completed the loopback `Bun.serve`/`fetch` fixture, and a temporary Rust panic hook identified the exact source line.

The patch now initializes process options once through the canonical `write_context_no_parse` constructor before JSC/VM creation. It reuses CLI's process-owned static Log and stores the process start time. `ContextData` owns its default option containers and contains no session entry or environment pointer; its log must live for the process because Workers, HTTP/HTML services and console formatting can read the context across VM lifetimes. Host argv is never interpreted as Bun CLI arguments. Per-VM logs remain independently owned and reclaimed by session teardown.

The full real-VM matrix passed after this fix (`1 passed`, 0.86 s): changed entry contents across sessions, reactive counter commits, queued input followed by EOF, asynchronous `beforeExit`, 512 events sent before input activation, a handled input exception, output pressure/drain, three local HTTP/fetch/timer sessions and their teardown, termination of busy JS and endless microtasks, `process.exit(7)` with one exit callback, stop before publication, rejected asynchronous entry, and counter restart. The matrix uses the actual linked Bun VM and C ABI. Temporary diagnostics were then removed, and allocation failure while constructing an input Buffer was folded into the same unread-input rescheduling path as listener errors. The rebuilt final source passed the same matrix with no diagnostic hook or startup markers (`1 passed`, 0.89 s).
