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
