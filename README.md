# React GPUI

React GPUI is a first vertical slice that renders a React tree into a GPUI surface. React and Bun own Fiber, hooks, context, fragments, and JavaScript closures; Rust and GPUI own validated native tree state and drawing.

## Status

The V3 path is working end to end: the React custom renderer emits an immutable Snapshot bootstrap followed by incremental Patches, the Rust host validates and applies them, native press/TextInput/VirtualList/animation events return to JavaScript listener callbacks, and the host can select either `ProcessAdapter` or the in-process `EmbeddedBunAdapter`. Embedded Bun builds the pinned Bun/JSC graph from `crates/react-gpui-bun/bun_embed.patch`; Fast Refresh lives in `packages/react-gpui-dev`.

## Architecture

```text
React components and hooks
          │
          ▼
@react-gpui/core (Bun/TypeScript)
  Fiber commit → one framed MessagePack Commit Batch
          │ stdout commits / stdin events
RuntimeAdapter (`ProcessAdapter` or `EmbeddedBunAdapter`)
          │
          ▼
ReactRoot (Rust) → decode + validate → NodeStore
          │
          ▼
GPUI main thread → render one validated surface tree
          │
          └── native press → framed event → RuntimeAdapter → JS callback
```

A commit reader performs blocking process I/O away from the GPUI foreground executor, then applies each complete Commit Batch on the GPUI side. GPUI rebuilds ephemeral elements from the retained `NodeStore`; native callbacks send events through the same adapter.

## Quick start

From the repository root:

```sh
cargo run -p react-gpui-host -- --runtime process bun run packages/react-gpui/examples/counter.tsx

# Embedded Bun/JSC (builds the pinned source graph under target/)
cargo run -p react-gpui-host --features embedded-bun -- --runtime embedded packages/react-gpui/examples/counter.tsx

```

For opt-in Fast Refresh while developing an embedded entry, add `--watch`.

## Workspace map

- `crates/react-gpui-bun/` — bounded in-process Bun/JSC adapter and pinned source patch/build.
- `packages/react-gpui-dev/` — Babel React Refresh transform, runtime globals, family refresh, and last-good failure handling.
- `crates/react-gpui-host/` — executable host with explicit ProcessAdapter/EmbeddedBunAdapter selection.
- `packages/react-gpui/` — TypeScript package `@react-gpui/core`, custom React renderer, transports, tests, and counter example.
- `crates/react-gpui/` — Rust protocol, runtime adapter seam, snapshot validation, retained node store, and GPUI rendering entity.
- `references/zed/` — checked-in GPUI reference source used by the workspace.

## V3 protocol and ownership invariants

- Every message is a four-byte little-endian payload length followed by MessagePack bytes; frames are bounded at 16 MiB.
- The first commit for a surface epoch is a Snapshot `[3,1,surfaceId,epoch,baseRevision,revision,nodes]`; every later completed React commit is one Patch `[3,3,surfaceId,epoch,baseRevision,revision,operations]`. Both are immutable atomic Commit Batches, and each completed React commit produces exactly one transport submission.
- Patch operations are compact positional Create, Update, Move/Reorder, and Delete records. Functions remain listener IDs and are never serialized.
- Each node is `[id,parentId,index,kind,style,text,listenerId,hostProperties,accessibility]`. `hostProperties` is `null`, TextInput `[1,value,placeholder,multiline,disabled,controlled,ackEditSeq,selStart,selEnd,markedStart,markedEnd]`, or VirtualList `[2,itemCount,rangeStart,rangeEnd,estimatedItemSize,overscan]`; node kind must match the tag. Node `1` is one permanent synthetic `View` root with `parentId=0` and `index=0`; user host IDs start at `2`.
- Revisions are strictly increasing `u32` values, and `baseRevision` must match the last accepted Snapshot or Patch. Surface and epoch identify the native surface generation; a new epoch starts with a Snapshot bootstrap.
- V3 kinds are `View=1`, `Text=2`, `Pressable=3`, `RawText=4`, `TextInput=5`, and `VirtualList=6`. Text-tree shape, host properties, accessibility values, transitions, and styles are validated on both sides of the adapter seam.
- Events are exactly `[3,2,surface,epoch,revision,sequence,node,listener,eventType,payload|null]`. Press uses `null`; TextInput payloads are tag `1`, CommandResult tag `2`, VisibleRange tag `3`, and AnimationComplete tag `4`. Stale, detached, duplicate, and out-of-order events are ignored.
- `VirtualList<T>` keeps `data`, `itemKey`, and `renderItem` in JavaScript, commits only the bounded visible range, and exposes Promise-returning `scrollToIndex`/`scrollToEnd` refs. Native GPUI owns a persistent fixed-height `uniform_list` scroll handle per node.
- React/Bun own JavaScript behavior and closures. GPUI owns main-thread/native state, including retained list handles and animation sampling. Runtime adapters transport versioned Commit Batches and events; animation frames never produce JavaScript commits.

## Development commands

Rust workspace checks and tests:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo run -p react-gpui-host -- --help
```

TypeScript package checks and tests:

```sh
cd packages/react-gpui
bun install --frozen-lockfile
bunx tsc --noEmit
bun test
bun run examples/counter.tsx
```

V3 supports the documented native host kinds, protocol-v3 press/TextInput/VirtualList/animation notifications, transitions, and accessibility fields. It does not provide synchronous native cancellation, arbitrary native widgets, or a browser/DOM compatibility layer. `ProcessAdapter` remains the default for fast iteration; `EmbeddedBunAdapter` is available with `--features embedded-bun` and is built from the pinned Bun source graph. Fast Refresh failures keep the last-good native tree visible and print an actionable stderr diagnostic.
