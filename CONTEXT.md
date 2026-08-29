# React GPUI

React GPUI is a bridge between React-owned behavior and GPUI-owned native rendering. This context defines the vocabulary for the shared surface, tree, commit, event, and runtime boundaries.

## Language

**Surface**:
A native rendering area identified by a surface identity and generation, with one host tree presented to GPUI. See `packages/react-gpui/src/surface-host.ts` and `docs/protocol.md` §2.
_Avoid_: Window, canvas, DOM root

**Surface Retirement**:
A closed or explicitly unmounted surface ID is permanently unavailable; changing its epoch cannot revive or reuse it. See `packages/react-gpui/src/surface-host.ts` and `.scratch/surface-lifecycle/spec.md`.
_Avoid_: epoch reuse, reopen

**Host Node**:
A React-rendered node in the native host tree, identified independently from React Fiber and carrying a host kind, parent relationship, and optional native-facing data. See `crates/react-gpui/src/tree.rs` and `docs/protocol.md` §2.
_Avoid_: DOM element, widget object, Fiber

**Rich Text**:
A Text paragraph composed of raw strings and one level of nested Text runs, with shared paragraph presentation and content that may span run boundaries. See [ADR-0013](docs/adr/0013-interactive-text-runs.md).
_Avoid_: separate inline elements, nested paragraphs

**Text Run**:
A direct nested Text fragment within Rich Text that contributes a contiguous part of the paragraph and may carry run-level presentation or press behavior. See [ADR-0013](docs/adr/0013-interactive-text-runs.md).
_Avoid_: inline element, nested paragraph

**Clickable Range**:
A contiguous UTF-8 byte interval in flattened Rich Text associated with a listener-bearing Text Run and eligible for pointer activation and range cursor feedback. The parent InteractiveText hitbox maps pointer positions to this interval. See [ADR-0013](docs/adr/0013-interactive-text-runs.md).
_Avoid_: glyph hitbox, DOM range

**Focus Affordance**:
A visible cue that identifies the currently focused interactive Text Run across its wrapped-line segments. See [ADR-0013](docs/adr/0013-interactive-text-runs.md).
_Avoid_: selection highlight, hover decoration

**Commit Batch**:
An atomic versioned exchange produced once per completed React commit; protocol v3 uses a Snapshot for bootstrap and a Patch for later commits, including tagged TextInput and VirtualList host properties, accessibility data, and native animation style metadata. See `docs/protocol.md` §2.
_Avoid_: Full snapshot, mutation stream, per-field update

**Wire Tail Slots**:
A current optional positional suffix, with placeholders where needed for
unambiguous decoding; historical arities are not part of the current contract.
See `crates/react-gpui/src/protocol/wire/node.rs`,
`crates/react-gpui/src/protocol/wire/snapshot_patch.rs`, and [ADR-0011](docs/adr/0011-single-form-wire-optional-tails.md).
_Avoid_: legacy decode, dual-form protocol, compatibility tail

**Native Event**:
A semantic notification from the native surface to JavaScript, grouped by interaction input, surface lifecycle, list or animation progress, and command acknowledgement. It carries ordered protocol data for a mounted Host Node or native driver rather than representing a browser event lifecycle. See `docs/protocol.md` §3.
_Avoid_: Browser event, DOM event, cancellable event

**Pointer Coordinates**:
Required finite non-negative logical window pixels on pointer down/up events, clamped to the native viewport before crossing the wire. See `crates/react-gpui/src/renderer/events.rs` and `docs/protocol.md` §3.
_Avoid_: device pixels, optional click position, hover position

**Pointer-Move Capability**:
An opt-in View or Pressable listener capability that registers the native move stream only while `onPointerMove` exists and emits the tagged pointer-move payload. See `packages/react-gpui/src/renderer/nodes.ts`, `crates/react-gpui/src/renderer/paint/mod.rs`, and [ADR-0010](docs/adr/0010-opt-in-high-frequency-event-streams.md).
_Avoid_: always-on mouse stream, hover edge event

**Drag Capability Bits**:
Independent `acceptsDragOver` and `acceptsDrop` host properties that select target notifications; the draggable source and optional exported files remain separate capabilities. See `packages/react-gpui/src/renderer/props.ts` and `crates/react-gpui/src/renderer/paint/drag.rs`.
_Avoid_: drag source implies drop target, native can-drop round trip

**Overlay Outside-Dismissal Capture**:
Capture-phase mouse-down detection that emits an outside event only when the point is outside both an overlay and its direct anchor subtree. See `crates/react-gpui/src/renderer/paint/overlay.rs` and `docs/protocol.md` §3.
_Avoid_: bubble-phase dismissal, invisible scrim, native popup menu

**Runtime Adapter**:
The boundary that carries Commit Batches and Native Events between the
JavaScript renderer and the native host, independent of how the JavaScript
runtime is hosted. See `crates/react-gpui/src/transport.rs` and
`docs/adr/0002-embedded-bun-runtime.md`.
_Avoid_: Snapshot-only transport, FFI callback, renderer backend, widget bridge

**JSC Single-VM Isolation**:
One embedded JavaScriptCore VM is owned by one runtime thread; only bounded
immutable protocol bytes cross its callback bridge, never GPUI handles,
JavaScript values, or closures. See `crates/react-gpui-bun/src/lib.rs`,
`crates/react-gpui-bun/bun_embed.patch`, and [ADR-0002](docs/adr/0002-embedded-bun-runtime.md).
_Avoid_: shared VM, cross-thread JSC value, GPUI handle in JavaScript

**Surface Command**:
A request from JavaScript to the native surface, addressed either to the root surface for window, focus, clipboard, file, or font operations or to a Host Node, with an optional typed value returned through CommandResult. See `packages/react-gpui/src/renderer/root-container.ts` and `docs/protocol.md` §4.
_Avoid_: Native Event, RPC method, GPUI callback

**Runtime Resource Commands**:
Root-only bounded asynchronous file, clipboard, and font operations whose I/O and registration are owned by the host rather than the renderer runtime. See `packages/react-gpui/src/renderer/root-container.ts`, `crates/react-gpui/src/renderer/commands.rs`, and `docs/protocol.md` §4.
_Avoid_: unbounded filesystem bridge, renderer-owned native resource, synchronous I/O

**Close Policy**:
Per-Surface host-held `allow` or `require-confirmation` state consulted by the native close callback before window retirement. See `crates/react-gpui-host/src/main.rs` and [ADR-0009](docs/adr/0009-async-close-confirmation.md).
_Avoid_: synchronous JavaScript cancellation, global close policy

**Close Request/Resolve**:
The one-in-flight root Native Event and root Surface Command pair used when a Close Policy requires JavaScript confirmation across the runtime boundary. See `packages/react-gpui/src/renderer/root-container.ts`, `crates/react-gpui-host/src/main.rs`, and `docs/protocol.md` §§3–4.
_Avoid_: blocking native callback, duplicate close event, `preventDefault()`

**Tab-stop Graph**:
The native ordered set and traversal relationships of focusable interactive nodes, including the host's disabled semantics; `Root.focusNext()` and `Root.focusPrev()` traverse this graph. See `crates/react-gpui/src/renderer/input.rs` and `crates/react-gpui/src/renderer/paint/mod.rs`.
_Avoid_: DOM focus tree, browser tab order, focus callback

**Tab-stop Capability**:
The explicit `FocusHandle.tab_stop(true)` capability required for an eligible focus handle to participate in the Tab-stop Graph; `.focusable()` alone is not traversal registration. See `crates/react-gpui/src/renderer/input.rs` and [ADR-0005](docs/adr/0005-keyboard-activation-reuses-press-semantics.md).
_Avoid_: focusable prop as traversal guarantee, `isFocused` polling

**Host-Owned Input Model**:
A bounded native model for text, UTF-16 selection, marked text, visual selection, and edit transitions used where pinned GPUI has no matching React primitive. See `crates/react-gpui/src/renderer/input.rs`, `crates/react-gpui/src/renderer/paint/text_input.rs`, and [ADR-0012](docs/adr/0012-host-owned-input-models.md).
_Avoid_: JavaScript shadow editor, fake upstream primitive, wire-owned caret state

**Host-Owned Undo History**:
The private per-TextInput bounded undo/redo stacks in `NativeInputState`, with typing coalescing and composition boundaries, emitting the normal change/selection contract. See `crates/react-gpui/src/renderer/input.rs`, `crates/react-gpui/src/renderer/paint/text_input.rs`, and [ADR-0012](docs/adr/0012-host-owned-input-models.md).
_Avoid_: app-global undo, JavaScript-only history, unbounded snapshots

**Caret Follow/Scroll Offset**:
A host-computed bounded per-input viewport offset that keeps the caret or IME marked range visible and shifts text, selection, and caret together without wire state. See `crates/react-gpui/src/renderer/input.rs`, `crates/react-gpui/src/renderer/paint/text_input.rs`, and `.scratch/caret-follow/reproduction.md`.
_Avoid_: JavaScript scroll state, absolute caret painting, new viewport wire field

**Font Registration Timing Invariant**:
Runtime font registration must precede the family's first layout/use because GPUI caches successful and failed family resolution; late registration does not invalidate that cache. See `crates/react-gpui/src/renderer/commands.rs`, `.scratch/font-loading/spec.md`, and `docs/protocol.md` §4.
_Avoid_: late fallback invalidation, WOFF promise, implicit preload

**Golden Vector**:
A checked protocol fixture row or collection that asserts producer byte stability, cross-language semantic equivalence, and permitted numeric representations. See `fixtures/protocol/`, `scripts/protocol-golden.ts`, and [ADR-0003](docs/adr/0003-cross-language-golden-vector-contract.md).
_Avoid_: Screenshot, example payload, visual snapshot

**Tap**:
An opt-in metadata-only observation channel for framed protocol traffic, recording timing, direction, peer, kind, and size without recording payload contents. See `packages/react-gpui/src/protocol-tap.ts` and `crates/react-gpui/src/protocol_tap.rs`.
_Avoid_: Transport, payload logger, application event stream
