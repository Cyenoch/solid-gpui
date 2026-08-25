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
A semantic notification from the native surface to JavaScript, currently Press, TextInput, VisibleRange, AnimationComplete, or CommandResult, carrying ordered protocol data for a mounted Host Node or native driver.
_Avoid_: Browser event, DOM event, cancellable event

**Runtime Adapter**:
The boundary that carries Commit Batches and Native Events between the JavaScript renderer and the native host, independent of how the JavaScript runtime is hosted.
_Avoid_: Snapshot-only transport, FFI callback, renderer backend, widget bridge
