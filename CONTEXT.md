# React GPUI

React GPUI is a bridge between React-owned behavior and GPUI-owned native rendering. This context defines the vocabulary for the shared surface, tree, commit, event, and runtime boundaries.

## Language

**Surface**:
A native rendering area identified by a surface identity and generation, with one host tree presented to GPUI.
_Avoid_: Window, canvas, DOM root

**Host Node**:
A React-rendered node in the native host tree, identified independently from React Fiber and carrying a host kind, parent relationship, and optional native-facing data.
_Avoid_: DOM element, widget object, Fiber

**Commit Batch**:
An atomic versioned exchange produced once per completed React commit; protocol v3 uses a Snapshot for bootstrap and a Patch for later commits, including tagged TextInput and VirtualList host properties, accessibility data, and native animation style metadata.
_Avoid_: Full snapshot, mutation stream, per-field update

**Native Event**:
A semantic notification from the native surface to JavaScript, grouped by interaction input, surface lifecycle, list or animation progress, and command acknowledgement. It carries ordered protocol data for a mounted Host Node or native driver rather than representing a browser event lifecycle.
_Avoid_: Browser event, DOM event, cancellable event

**Runtime Adapter**:
The boundary that carries Commit Batches and Native Events between the JavaScript renderer and the native host, independent of how the JavaScript runtime is hosted.
_Avoid_: Snapshot-only transport, FFI callback, renderer backend, widget bridge

**Surface Command**:
A request from JavaScript to the native surface, addressed either to the root surface for window/focus/clipboard operations or to a Host Node, with an optional typed value returned through CommandResult.
_Avoid_: Native Event, RPC method, GPUI callback

**Tab-stop Graph**:
The native ordered set and traversal relationships of focusable interactive nodes, including the host's disabled semantics; `Root.focusNext()` and `Root.focusPrev()` traverse this graph.
_Avoid_: DOM focus tree, browser tab order, focus callback

**Golden Vector**:
A checked protocol fixture row or collection that asserts producer byte stability, cross-language semantic equivalence, and permitted numeric representations.
_Avoid_: Screenshot, example payload, visual snapshot

**Tap**:
An opt-in metadata-only observation channel for framed protocol traffic, recording timing, direction, peer, kind, and size without recording payload contents.
_Avoid_: Transport, payload logger, application event stream
