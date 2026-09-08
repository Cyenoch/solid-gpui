# Solid GPUI protocol v5

This is the implemented wire contract between the TypeScript renderer and the
Rust/GPUI host. Protocol v5 is a lockstep clean cutover to Bebop. The canonical
wire schema is
[`packages/solid-gpui/src/protocol/protocol.bop`](../packages/solid-gpui/src/protocol/protocol.bop);
checked generated bindings are under the TypeScript and Rust protocol seams.
`packages/solid-gpui/src/protocol/schema-lock.json` pins the schema SHA-256
digest (`e0fcd0e7b6c78ce18dce79c5d5d54be7e6fe27a0c717d4b88132ca13f0c2e3b3`)
for protocol version `5`; codegen checks fail on drift.
Normal package and Rust builds consume those checked files and do not invoke
`bebopc`. Regenerate and check them with:

```sh
bun run task protocol-codegen
bun run task protocol-codegen-check
```

The semantic DTOs in
`packages/solid-gpui/src/protocol/types.ts` and
`crates/solid-gpui/src/protocol.rs` are the public protocol interface. Generated
Bebop types stay inside the codec adapters. Domain validation remains
handwritten; generated decoding is only structural parsing.

Rust semantic messages are typed at the protocol boundary: `Command` contains
`CommandMeta` and `CommandOperation`; `Event` contains `EventMeta` and
`EventPayload`, whose `event_kind()` identifies the payload family. Style
optionals use closed enum types, and semantic absence is represented by
omission/`None`, not a zero-valued compatibility sentinel.

## 1. Frame and envelope

A transport frame is:

```text
[u32 little-endian payload length][Bebop Envelope bytes]
```

The four-byte prefix counts only the Bebop payload. The maximum payload is
`16 * 1024 * 1024` bytes. Rust `read_frame` and `write_frame` retain the
existing exact-read, truncation, size-bound, and flush behavior. TypeScript
`FrameDecoder` retains fragmented and coalesced input behavior.

The root Bebop record is:

```text
Envelope {
  protocolVersion: u32 = 5,
  body: Body,
}
```

`Body` uses the stable message tags `1=Snapshot`, `2=Event`, `3=Patch`, and
`4=Command`. Every decoder requires `protocolVersion` to be present and equal
to `5`, requires one known body union, rejects trailing bytes, and never
attempts a legacy decode. There is no version negotiation, dual decoder, or
permissive fallback; peers must use the same v5 contract.

Bebop messages are length-delimited records with monotonically ordered field
IDs and a terminating field ID `0`; arrays carry a bounded u32 item count;
strings and byte arrays carry bounded u32 byte lengths. Before the generated
reader runs, the schema-derived guard in
`crates/solid-gpui/src/protocol/guard.rs` and
`packages/solid-gpui/src/protocol/bebop-guard.ts` enforces:

- the frame, message, string, byte-array, repeated-item, total-item, and
  recursion budgets;
- strict `bool` values (`0` or `1`) and declared enum ranges;
- fatal UTF-8 decoding;
- no duplicate or out-of-order field IDs and no unknown fields;
- known union discriminators only, exact message/union terminators, consumed
  lengths, and no trailing bytes; and
- per-field resource budgets before generated repeated/string/bytes decoding
  can allocate.

The Rust guard executes the checked static plan in
`crates/solid-gpui/src/protocol/generated/schema_guard.rs`; it does not parse
JSON schema metadata on the first decode.

After structural decoding, the existing semantic validators enforce node
ownership, surface/epoch/revision rules, resource limits, style constraints,
command ownership, event ownership, and publication invariants. A malformed
Snapshot or Patch is never published to the retained tree.

For an Extension, the exact catalog identity is the tuple
`(providerId, catalogDigest, entryId, entryVersion)`. The host must resolve that
identity to an adapter and validate the candidate's Extension properties and
children before publication. An unknown identity is rejected transactionally;
the default `NoExtensions` registry therefore rejects every Extension rather
than publishing an unrenderable node.

## 2. Snapshot and Patch

`Snapshot` contains `surfaceId`, `epoch`, `baseRevision`, `revision`, and the
complete `nodes` array. `Patch` contains the same revision header and ordered
`operations`. Snapshot/Patch revisions must satisfy the existing retained-tree
rules: a first snapshot has base revision zero, later patches match the current
revision, and `revision > baseRevision`.

`Node` carries every current field: `id`, `parentId`, `index`, `kind`, all 42
Style slots, optional RawText `text`, `listenerId`, optional tagged
`HostProperties`, optional `AccessibilityProperties`, `focusable`,
`selectable`, `tooltip`, and `acceptsPointerMove`. `NodeKind` is
`1=View`, `2=Text`, `3=Pressable`, `4=RawText`, `5=TextInput`,
`6=VirtualList`, `7=Image`, `8=Extension`, and `9=Icon`.

`HostProperties` contains the complete variants:

- TextInput: value, placeholder, multiline, disabled, controlled, edit
  sequence, selection range, marked range, max length, and reversed selection;
- VirtualList: item count, visible range, estimated item size, and overscan;
- Image: source, object-fit code, and fallback source; and
- Drag: drag type, exported files, accepts-drag-over, and accepts-drop.

- Extension: a 16-byte `providerId`, 32-byte `catalogDigest`, nonzero
  `entryId` and `entryVersion`, sorted unique nonzero field IDs, and sorted
  unique nonzero event IDs. Fields use `bool`, `i32`, `u32`, finite `f32`,
  bounded text, or bounded bytes values. There are at most 256 fields and
  event IDs; text and bytes are each capped at 1 MiB per value and in
  aggregate.

Accessibility carries role, label, description, disabled, checked, selected,
value, expanded, and heading level. Style contains width/height, flex and
alignment enums, spacing, colors, opacity, transition, borders, typography,
positioning, cursor, text alignment, up to two BoxShadow values, and font
family. The migration fields add per-edge padding, border widths/colors,
per-corner radii, percentage width/height, wrapping, and a bounded two-stop
linear gradient. Percent and pixel dimensions are mutually exclusive. Gradient
angles are 0..=360 degrees, stops are strictly increasing in 0..=1, and colors
are RGBA u32 values. `flexWrap` uses 0=no-wrap, 1=wrap, 2=wrap-reverse; omission
retains the native default. Edge/corner values override their shorthands,
including zero. See [native migration](native-migration.md) for the public API.
Numeric layout values use f32 on the wire; IDs, enum codes, masks, and
counts use u32. Semantic validators enforce finite/non-negative ranges and the
existing enum and ownership rules.

For optional enum style fields, semantic absence is represented only by
`None`/omission. In particular, `flexDirection` codes are `1..=4` and
`textAlign` codes are `1..=3`; `Some(0)`/wire `0` is invalid rather than an
alternate unset representation.

Patch operations are `1=Create`, `2=Update`, `3=Move`, and `4=Delete`.
`PatchUpdate` preserves mask presence exactly:

| Bit | Field                   |
| --: | ----------------------- |
|   1 | style                   |
|   2 | text                    |
|   4 | listener                |
|   8 | host properties         |
|  16 | accessibility           |
|  32 | focusable               |
|  64 | selectable              |
| 128 | tooltip                 |
| 256 | pointer-move capability |

An Update with the style bit set carries either `style` (set) or the empty
`clearStyle` marker (clear); with the bit unset, both are omitted (unchanged).

`focusable` and `selectable` follow the same exact presence rule: each field is
omitted when its update bit is clear and is present (including explicit
`false`) when its bit is set. Decoders reject either field when its bit is
clear and materialize an omitted field as the semantic `false` placeholder.
Masked optional fields distinguish an omitted clear from an unchanged field.
Explicit `false` and zero values are retained when present, and optional tails
remain semantic absence rather than a legacy compatibility path. Create carries
the complete Node. Move reparents/reorders an existing node, and Delete removes
its subtree. Existing tree validation and atomic rollback remain authoritative.

## 3. Commands and command values

A Command carries `surfaceId`, `epoch`, `afterRevision`, `requestId`, `nodeId`,
its numeric `kind`, and a typed `CommandPayload` union. The host admits commands
only when their surface, epoch, and `afterRevision` match the current tree.
The window entry executes each command before consuming the next message, so a
later Patch cannot make an already admitted command stale. Native asynchronous
services may complete later and return their own request's result. Layout queries
and focus traversal use GPUI's currently drawn frame; a tree commit alone does
not imply that a new frame has been painted.

The semantic command kind remains the following complete set:

| Code | Kind                 | Payload / ownership                                            |
| ---: | -------------------- | -------------------------------------------------------------- |
|    1 | Focus                | null; focusable node                                           |
|    2 | Blur                 | null; focusable node                                           |
|    3 | SetSelection         | u32 start/end; TextInput                                       |
|    4 | ScrollToIndex        | u32 index/alignment; VirtualList                               |
|    5 | ScrollToEnd          | null; VirtualList                                              |
|    6 | SetTitle             | text; root                                                     |
|    7 | ResizeWindow         | u32 width/height; root                                         |
|    8 | ZoomWindow           | null; root                                                     |
|    9 | ToggleFullscreen     | null; root                                                     |
|   10 | OpenUrl              | text; root                                                     |
|   11 | FocusNext            | null; root                                                     |
|   12 | FocusPrev            | null; root                                                     |
|   13 | GetWindowSize        | null; root                                                     |
|   14 | GetFocus             | null; focusable node                                           |
|   15 | ClipboardWrite       | text; root                                                     |
|   16 | ClipboardRead        | null; root                                                     |
|   17 | OpenSurface          | title, u32 width/height, optional creation options; root       |
|   18 | FileDialogOpen       | title, directories, multiple; root                             |
|   19 | FileDialogSave       | default name text; root                                        |
|   20 | ShowNotification     | title, body, optional action list; root                        |
|   21 | SetMenus             | complete menu tree; root                                       |
|   22 | SetKeybindings       | complete binding list; root                                    |
|   23 | SetClosePolicy       | `allow` or `require-confirmation`; root                        |
|   24 | ResolveCloseRequest  | request ID and allow bool; root                                |
|   25 | ReadTextFile         | absolute path text; root                                       |
|   26 | WriteTextFile        | absolute path and content; root                                |
|   27 | ClipboardWriteImage  | format code and bounded bytes; root                            |
|   28 | ClipboardReadImage   | null; root                                                     |
|   29 | LoadFont             | absolute path text; root                                       |
|   30 | MinimizeWindow       | null; root                                                     |
|   31 | GetWindowBounds      | null; root                                                     |
|   32 | GetWindowState       | null; root                                                     |
|   33 | ActivateWindow       | null; root                                                     |
|   34 | GetScrollOffset      | null; VirtualList                                              |
|   35 | ScrollToOffset       | finite non-negative f32; VirtualList                           |
|   36 | InvokeNative         | catalog identity, function ID, bounded args; root or component |
|   37 | CancelNative         | original invocation request ID; root                           |
|   38 | ConfigureApplication | lifecycle policy and acknowledged sequence; application scope  |
|   39 | OpenPopup            | anchor node, content size, placement, gap; owner root          |
|   40 | ClosePopup           | original OpenPopup request ID; owner root                      |

The generated payload union uses typed records for u32 pairs, f32 values, text,
string pairs, OpenSurface, FileDialogOpen, notifications, menus, keybindings,
clipboard images, close resolution, native invocation/cancellation, application
configuration, and popup creation/cancellation. The decoder checks that the payload
variant agrees with the command kind and that root/node ownership is valid.

`OpenPopup` requires a mounted owner anchor (node ID at least 2), positive content
size at most 16384 per axis, placement 0–11, and finite gap 0–1024. Its successful
result carries a fresh child Surface ID. `ClosePopup` names the original opening
request within the owner Surface and epoch, so cancellation works before a child
ID exists. Repeated cancellation is idempotent. See [System popovers](system-popover.md).

Command results are Events carrying request ID, command code, node ID, success,
optional error text, and an optional `CommandValue`. Value tags remain:
`1=number`, `2=pair`, `3=boolean`, `4=text`, `5=paths`, `6=file text`,
`7=image`, `8=bounds`, `9=window state`, and `10=scroll offset`. File, image,
clipboard, path, menu, notification, and keybinding limits are checked before
publication to JavaScript.

Valid in-flight messages for a retired Surface are discarded without publishing
state or terminating the application. A late initial Snapshot receives a matching
SurfaceClosed event so its newly registered root can dispose itself. This does
not admit never-allocated IDs or revive retired IDs.

## 4. Events

Every Event carries surface, epoch, revision, sequence, node, listener, numeric
`eventType`, and an optional typed `EventPayload`. Event codes are:

| Code | Kind                 | Payload                                             |
| ---: | -------------------- | --------------------------------------------------- |
|    1 | Press                | null                                                |
|    2 | Change               | TextInput data                                      |
|    3 | Selection            | TextInput data                                      |
|    4 | Focus                | null or TextInput data                              |
|    5 | Blur                 | null or TextInput data                              |
|    6 | CommandResult        | Command result                                      |
|    7 | VisibleRange         | u32 start/end                                       |
|    8 | AnimationComplete    | u32 generation                                      |
|    9 | Key                  | key, modifiers, action down/repeat/up               |
|   10 | Pointer              | pointer down/up or pointer-move payload             |
|   11 | Hover                | null                                                |
|   12 | Scroll               | delta kind, f32 deltas/position, modifiers          |
|   13 | Submit               | text                                                |
|   14 | WindowResize         | f32 width/height/scale                              |
|   15 | WindowActivation     | active bool                                         |
|   16 | SurfaceClosed        | null; node/listener `0/0`                           |
|   17 | Action               | action text; root/listener `1/0`                    |
|   18 | WindowAppearance     | light/dark; root/listener `1/0`                     |
|   19 | Layout               | f32 x/y/width/height                                |
|   20 | Drag                 | drag-over, drag-drop, or external paths             |
|   21 | NotificationResponse | tag and optional action ID; root/listener `1/0`     |
|   22 | PointerDownOutside   | f32 x/y                                             |
|   23 | CloseRequested       | request ID; root/listener `1/0`                     |
|   24 | Extension            | event ID and sorted Extension fields; node/listener |

Focus and Blur intentionally retain dual forms: View/Pressable focus observers
use an omitted payload, while TextInput focus/blur carries TextInput data.
Pointer down/up and pointer-move are typed variants under Event 10. Drag-over,
drag-drop, and external-file-drop are typed variants under Event 20. Modifier
lists contain unique members of `cmd`, `ctrl`, `alt`, `shift`, and `function`.
Coordinates are finite and non-negative where the existing semantic contract
requires it; layout and scroll offsets preserve their existing f32 semantics.

The host keeps event sequence ordering and the renderer keeps per-surface
surface/epoch/revision checks. Listener IDs are revision-owned callback
identities: current and previous generations are retained only long enough for
valid in-flight events and detached focus, so a stale revision cannot call a
newer callback. For Event 24, dispatch first resolves the listener generation
for the event revision, then requires an attached Extension node with the
matching node ID and an event ID listed in that node's subscription list. Only
that subscribed callback receives `{eventId, fields, target}`; unknown or
unsubscribed event IDs are ignored. SurfaceClosed is emitted before host
removal. Root-level action, appearance, notification, and close-request events
retain their existing node/listener ownership. No new wire event is fabricated
for host-owned text selection or editing state.

## 5. Runtime ownership seams

The Rust host keeps native surfaces, windows, focus, input, and retained state
in `NativeStateRegistry`. `CommitPump` is the bounded handoff that receives
runtime commit payloads and applies them on the foreground owner of that
registry. The registry validates and publishes only complete Snapshot/Patch
state.

On the TypeScript side, `SurfaceRouter` is the only frame decoder and event
router. It groups one incoming chunk into ordered semantic event batches per
surface. `HostTree` owns the private `NodeGraph`, transaction journal, dirty
property finalization, and Snapshot/Patch production; `CommandClient` owns
request IDs, pending results, and termination rejection. The `HostKind` facts
module owns allowed properties, child rules, projector categories, and runtime
value capabilities. These modules do not expose transport internals or
generated protocol records to Solid components.

## 6. Conformance and cutover

`bun run task protocol-golden-check` regenerates representative v5 Snapshot,
Patch, all 40 Command kinds, all Event payload forms (including both focus/blur
forms), malformed cases, and frame boundaries, then fails if committed fixtures
drift. TypeScript authors `ts_to_rust.hex`; Rust independently constructs the
same representative semantic families and authors `rust_to_ts.hex`. Rust
decodes and re-encodes every TypeScript row, while TypeScript structurally and
semantically decodes every Rust row and verifies the canonical bytes. Both
languages execute the declared `invalid.hex` and `frames.hex` expectations.
Semantic conversion and validation errors include bounded `body`, operation,
node, and field paths; the predecode guard intentionally retains generic bounded
structural errors.

A reusable framed Event writer owns one bounded buffer per Rust output worker,
serializes in place, writes the four-byte length and payload directly, and
flushes stream chunks without a per-event payload Vec plus frame copy. The
TypeScript adapter uses the reusable BebopView for generated writes and copies
only the returned transport frame that the caller owns. The generated runtime
is never exposed to renderer callers.
