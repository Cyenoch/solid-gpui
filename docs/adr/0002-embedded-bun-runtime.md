# ADR-0002: Embed the Bun runtime on a dedicated thread

- **Status:** Accepted
- **Date:** 2026-08-24

## Decision

`EmbeddedBunAdapter` is a concrete `RuntimeAdapter` that owns Bun/JSC on one
runtime thread while GPUI remains on its application thread. The adapter moves
only bounded, immutable wire-protocol commit/event batches across a callback
bridge. No GPUI handle, JavaScript closure, JSC value, or callback borrow crosses
the thread boundary.

The adapter builds a pinned Bun source revision
`34cbb9a40b4bd1bd767d134a7065e66c2432a676` from `oven-sh/bun`, applies the
tracked `crates/react-gpui-bun/bun_embed.patch`, builds Bun's native graph, and
links `libbun_embed.dylib` from the graph's complete response-file inputs. The
build is cached under Cargo's target `OUT_DIR`; it does not use a checked-in or
undeclared local archive.

The patch adds a callable Bun entrypoint rather than calling `Cli::start`:
`VirtualMachine` is initialized directly, the real Bun module loader and event
loop evaluate the entry, and `teardown_embedded` destroys the VM while the JSC
API lock is held. The lock guard is consumed after teardown, so no release call
can dereference a destroyed VM. Bun returns a status to the host instead of
calling `Global::exit`.

`ProcessAdapter` remains available as the explicit process runtime. The host
selects it with `--runtime process` (the default) or the embedded runtime with
`--runtime embedded` and the `embedded-bun` Cargo feature.

Embedded polling exposes `CommitPoll::Commit`, `CommitPoll::Timeout`, and
`CommitPoll::Ended`; a temporary timeout is never represented as EOF. A
disconnected commit channel honors explicit `Shutdown`, while natural nonzero
runtime completion remains a protocol error and natural completion with any
status is visible through `RuntimeStatus`.

Fast Refresh queue submission is serialized with close under a lifecycle mutex:
shutdown closes the lifecycle before joining the watcher, so no refresh path
can enqueue after the embedded runtime has entered its terminal state.

## Runtime termination contract

`RuntimeAdapter` exposes a lifecycle status in addition to framed commit/event
transport. A `Shutdown` status means the host explicitly initiated application
shutdown and is not an error. Any `recv_commit` EOF while the adapter is not in
`Shutdown` is an unexpected runtime termination, including a clean child exit;
the host reports the exit code or signal when available, closes the GPUI
application, and exits nonzero. A framing, decode, or commit-validation error is
also terminal, is printed to stderr with its protocol context, closes the
application, and exits nonzero.

A `Failed` status denotes a retained process event-writer I/O failure and is
reported by later sends or status observations; the host fatal helper consumes
the send error when it must terminate.

Outbound Native Event and CommandResult send failures use the same fatal
contract: stderr includes the event context and non-sensitive wire identity
(event type, surface, epoch, revision, sequence, node, and listener), never
payload text; the runtime is shut down and the host exits nonzero. A send
failure observed after explicit application shutdown is ignored as a
consequence of closing the runtime.

The blocking commit reader owns no GPUI state. It sends payloads and terminal
outcomes through its bounded channel. All host callers use the same fatal
handler; GPUI surface callbacks invoke it on the foreground, and the handler
itself touches no GPUI state. Explicit application shutdown remains success
even when it closes the runtime's streams.

`ProcessAdapter` never writes a Native Event from the GPUI caller. A dedicated
`react-gpui-event-writer` thread owns `ChildStdin` and drains an ordered queue
bounded to 32 payloads and 16 MiB of queued payload bytes; a single maximum-size
protocol payload still fits an empty queue. `send_event` encodes once and
transfers ownership without waiting. Frame or byte capacity returns
`WouldBlock`; writer I/O failure is retained and invokes a child-stop callback
without holding the queue lock. Confirmed child death closes stdout so the
commit reader wakes and reports the retained `RuntimeStatus::Failed`. Shutdown
marks the adapter, kills and waits for the child to release blocked writes,
closes the queue, and joins the writer only after child exit is confirmed. A
kill/wait error returns without blocking on an unconfirmed writer join; a later
idempotent shutdown can retry and join.

## Fast Refresh

The development runtime installs React Refresh's actual runtime globals and a
Bun `onLoad` transform backed by `react-refresh/babel` plus the JSX transform.
The transform emits registration and hook-signature calls for every component
family, then wraps each family through a generated stable family registry so a
compatible nested edit keeps its hook cells while a signature change replaces
only that family's wrapper. Successful cache-busting module evaluations queue a
debounced `RefreshRuntime.performReactRefresh()` call; failed transforms/evals
do not call it. The transformed nested-module integration test proves
compatible child state, signature remount, and last-good failure behavior.
Native `--watch` smoke also proves the same process/window receives a refresh
commit and keeps the last-good native tree on malformed source.

## Consequences

- The first embedded build is larger and slower than the process path because
  Bun's Rust and JavaScriptCore/C++ graph are linked into `libbun_embed.dylib`.
- Embedded builds require a macOS JavaScriptCore toolchain and Bun's pinned
  nightly Rust toolchain. The build helper installs a pinned Ninja package in
  Cargo's target cache when Ninja is not already available.
- The embedded adapter's callback bridge applies bounded backpressure. If its
  commit queue is full or closed, Bun receives a synchronous transport error;
  shutdown drops event/refresh senders, joins the watcher, then joins the Bun
  thread deterministically.
