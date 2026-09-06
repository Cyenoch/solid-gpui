# Solid GPUI

Solid GPUI is the boundary between SolidJS-owned application behavior and GPUI-owned native rendering.

## Language

**Surface**: A native rendering area identified by a surface ID and protocol epoch, with one validated host tree. A retired surface ID cannot be reused.

**Solid Owner Tree**: The reactive computation graph that owns component state and effects. It is not sent to Rust and is not a native tree.

**Host Node**: A renderer-created `View`, `Text`, `Pressable`, `TextInput`, `VirtualList`, `Image`, `Icon`, provider-neutral `Extension`, or raw-text node with stable protocol identity. See `packages/solid-gpui/src/renderer/types.ts` and `crates/solid-gpui/src/tree.rs`.

**Native Module**: An application-owned collection of native components and commands whose declared contract is shared by its host and JavaScript callers.

**Native Component Instance**: The native state owned by one mounted Host Node. Its identity survives property updates and ends when that node is removed, replaced, or its surface epoch changes.

**Native Contract**: The agreed component properties, event payloads, and callable methods used by JavaScript and its registered native module.

**Extension Catalog Identity**: The exact provider-neutral identity tuple `(provider ID, catalog digest, entry ID, entry version)` that names an Extension contract. An identity is valid only when the host can resolve its registered adapter.

**Extension Adapter Registry**: The host-owned catalog of adapters selected by exact Extension Catalog Identity. It gives the host provider-specific validation and rendering behavior without putting provider implementation details on the wire.

**NoExtensions**: The default empty Extension Adapter Registry. It rejects an Extension whose catalog identity has no adapter before the candidate tree is published.

**Commit Batch**: One atomic exchange produced after a completed Solid host update. Protocol v5 encodes the first update for a surface epoch as a complete Snapshot and later updates as a Patch. Host mutations are never exposed as partial wire state.

**Native Event**: A semantic notification from a native surface to the matching Solid root. It is ordered by surface, epoch, revision, and event sequence; it is not a browser event.

**Runtime Adapter**: The byte-only boundary carrying Commit Batches, Native Events, and Surface Commands. ProcessAdapter uses framed Bebop v5 stdio; EmbeddedBunAdapter owns one Bun/JSC VM on a dedicated thread.

**Surface Command**: A bounded asynchronous request from the JavaScript root or a mounted Host Node to native window, focus, clipboard, file, font, menu, notification, or list behavior.

**Host-Owned Input Model**: Native text, selection, marked text, caret geometry, scrolling, and undo history. SolidJS owns controlled values and callbacks; transient editing state stays native.

**Transactional Snapshot**: The host-facing description that is published only after validation succeeds. This follows the same useful ownership principle as GPUI Shell's script snapshot/materialization seam while retaining this project's cross-process protocol.

**Golden Vector**: A checked fixture proving producer bytes and cross-language semantic equivalence for the canonical v5 schema.

**Canonical Wire Schema**: `packages/solid-gpui/src/protocol/protocol.bop`, the single source for the Bebop v5 Envelope, body tags, command/event kinds, node fields, host properties, styles, menus, and typed command values. Checked TypeScript/Rust bindings and schema metadata are generated from it.
**Schema Digest Lock**: `packages/solid-gpui/src/protocol/schema-lock.json` pins
protocol version `5` and the SHA-256 digest of `protocol.bop`; generated
bindings are checked against that canonical schema.

**Semantic Protocol DTOs**: Rust owns typed `CommandMeta`/`CommandOperation`,
`EventMeta`/`EventPayload`, `Event::event_kind()`, and closed style enums.
Generated Bebop records remain behind the protocol adapter seam.

**Host Native State**: `NativeStateRegistry` owns native surfaces, windows,
focus, input, and host-side retained state. `CommitPump` is the bounded
foreground handoff from runtime commits to that registry.

**Surface Router**: The TypeScript `SurfaceRouter` is the sole frame decode and
surface-event routing seam. It groups one incoming byte chunk into one ordered
semantic event batch per surface.

**Host Tree**: The private TypeScript `HostTree` owns the renderer `NodeGraph`,
transaction journal, dirty-property finalization, and Snapshot/Patch production.
`CommandClient` owns request IDs, pending command results, and rejection on
surface termination. `HostKind` facts own allowed properties, child rules,
projector categories, and runtime value capabilities.

**Listener Identity**: A listener ID is resolved with the event revision to a
bounded current or previous callback generation. A prior-frame event may use
the previous generation, but never the callback currently produced by a later
revision; detached focus retains the previous identity until release.

**Bounded Wire Guard**: A schema-derived structural pass that enforces frame/message/repeated-field budgets, strict booleans and enums, UTF-8 validity, field order, known unions, terminators, and consumed lengths before generated decoding can allocate.

**Reusable Event Writer**: The process adapter's owned Event queue and writer thread. It reserves a bounded exact frame size, reuses one output buffer, and serializes directly after queueing; it does not retain a second wire protocol.

**Tap**: An opt-in metadata-only observation channel for framed traffic. Payload contents are never recorded.

## Invariants

- Solid components never receive GPUI handles.
- Rust never receives Solid owners, signals, closures, or JavaScript values.
- Only bounded immutable bytes cross a runtime thread or process boundary.
- Host property mutations finalize once per transaction from the coherent
  private prop set; failed validation leaves the last published tree intact.
- Native events dispatch only to the callback generation owned by their surface,
  epoch, listener identity, and revision; current/previous retention preserves
  valid in-flight and detached-focus events without invoking newer callbacks.
- Surface event batches preserve input order and run in one Solid transaction.
- GPUI owns layout, painting, focus, native input, and system integration.
