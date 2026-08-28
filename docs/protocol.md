# React GPUI protocol v3

This is the protocol reference for contributors changing either side of the
React/Bun renderer ↔ Rust/GPUI boundary. It describes the implemented wire
contract, not a proposed protocol.

The field layouts and validation below are cross-checked against:

- TypeScript constants, tuple types, encoders, frame decoder, and event
  validation in `packages/react-gpui/src/protocol.ts`.
- Rust public protocol types and constants in
  `crates/react-gpui/src/protocol.rs`.
- Rust MessagePack tuple definitions and decode validation in
  `crates/react-gpui/src/protocol/wire/` (`mod.rs`, `snapshot_patch.rs`,
  `node.rs`, `command.rs`, and `event.rs`).
- Retained-tree revision/node validation in `crates/react-gpui/src/tree.rs`,
  and command ownership/side effects in
  `crates/react-gpui/src/renderer/commands.rs`.
- Cross-language fixtures in `fixtures/protocol/` and their assertions in
  `packages/react-gpui/tests/protocol-golden.test.ts`.

When changing a field, update both protocol implementations and regenerate the
fixtures with `make protocol-golden-generate`. The three-layer fixture contract
is recorded in [ADR-0003](adr/0003-cross-language-golden-vector-contract.md).

## 1. Frame layer

A frame is:

```text
[u32 little-endian payload length][payload MessagePack bytes]
```

The four-byte prefix counts payload bytes only. `MAX_FRAME_LENGTH` and
`MAX_FRAME_SIZE` are both `16 * 1024 * 1024` bytes. Rust `write_frame` rejects
larger payloads, writes the prefix and payload, and flushes; Rust `read_frame`
reads exactly one frame and treats EOF before a new header as clean end of
stream. EOF after a partial header or payload is a truncation error
(`crates/react-gpui/src/protocol.rs:1216-1269`). TypeScript `FrameDecoder` accepts
fragmented input, emits every complete payload in order, accepts coalesced
frames, and resets/throws on an oversized length
(`packages/react-gpui/src/protocol.ts:392-442`).

The checked boundary cases are in `fixtures/protocol/frames.hex:3-7`:
`00000000` is an empty payload, `04000000` declares four bytes, and
`01000001` declares `16,777,217` bytes (one over the limit). A complete
maximum-size frame still needs its full payload; a header alone remains
pending/truncated as specified by the reader.

## 2. Message matrix

All protocol messages are MessagePack arrays. The first two fields are always
`protocol=3` and a message discriminator:

| Message  | Array shape                                                                            | Meaning                                                                  | Source                                                |
| -------- | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ | ----------------------------------------------------- |
| Snapshot | `[3, 1, surfaceId, epoch, baseRevision, revision, nodes]`                              | Complete tree bootstrap for an epoch or a complete replacement snapshot. | `protocol.rs:71-109`, `wire/snapshot_patch.rs:9-48`   |
| Event    | `[3, 2, surfaceId, epoch, revision, sequence, nodeId, listenerId, eventType, payload]` | Native semantic notification to JavaScript.                              | `protocol.rs:402-428`, `wire/event.rs:9-182`          |
| Patch    | `[3, 3, surfaceId, epoch, baseRevision, revision, operations]`                         | Atomic incremental tree changes after a snapshot.                        | `protocol.rs:112-173`, `wire/snapshot_patch.rs:50-89` |
| Command  | `[3, 4, surfaceId, epoch, afterRevision, requestId, nodeId, kind, payload]`            | JavaScript request to the host surface or a host node.                   | `protocol.rs:176-219`, `wire/command.rs:6-76,87-249`  |

### Version compatibility

Protocol v3 is a lockstep boundary: every decoder accepts only its own
`PROTOCOL_VERSION` and rejects a valid frame from another version before
validating the message body. Rust reports both versions and the upgrade remedy
(`renderer speaks protocol vN; this host binary speaks protocol v3 — update the
host binary / pin @react-gpui/core to a v3 release`) through
`ProtocolError::UnsupportedProtocol`
(`crates/react-gpui/src/protocol.rs:1184-1187`). The TypeScript event decoder
reports the symmetric host/renderer diagnostic through
`ProtocolVersionMismatchError` (`packages/react-gpui/src/protocol.ts:12-22,
766-778`), and `SurfaceHost`/`RootContainer` preserve it as a typed
`TransportTerminatedError` with `{ kind: "protocol", detail }`. Update the host
binary and `@react-gpui/core` together, or pin the renderer to the host's
protocol release; there is no version negotiation or dual-version support.

### Multi-surface routing and lifecycle

The host keeps one `RuntimeAdapter` reader and a registry keyed by `surfaceId`.
The first native window is surface `1`; opening another window never changes
the routing shape of existing messages. Every Snapshot, Patch, and Command
must name an already registered surface. Unknown surface IDs are rejected;
the host never creates a window implicitly from an otherwise valid frame.

The TypeScript `createSurfaceHost(transport)` API owns the shared reader and
demultiplexes each decoded Event to the root registered for its `surfaceId`.
Each root has its own retained revision and event-sequence gate, while all
roots share the one transport. The compatibility `createRoot(transport)` API
still creates one root with the same command/event behavior.

To open a native surface, call the requesting root's
`root.openSurface({ title?, width?, height?, kind?, resizable?, minSize? })`.
The command is sent from that root (`surfaceId` is the requesting root and
`nodeId=1`). Omitted dimensions normalize to `[0,0]` host-default dimensions,
and omitted creation options preserve the GPUI defaults. For an option-bearing
request, the payload appends a fourth-slot options tuple:
`[title,[width,height],[kind,resizable,minWidth,minHeight]]`. `kind` is
`0=normal`, `1=floating` (above the owning app window where the platform
supports that relationship), or `2=dialog` (modal where supported);
`resizable` is a creation-time boolean; and `minWidth`/`minHeight` are positive
integer pixels. The old two-item payload remains byte-compatible. `maxSize`,
runtime window-level changes, and a centered toggle are intentionally absent:
GPUI has no corresponding portable API, and this host already centers its
initial and OpenSurface windows.

When the CommandResult resolves with value tag `1`, its number is the new
native surface ID. The application then registers that ID and starts rendering:

```ts
const surfaceId = await root.openSurface({
  title: "Inspector",
  width: 640,
  height: 480,
  kind: "floating",
  resizable: false,
  minSize: [320, 240],
});
const inspector = host.createRoot({ surfaceId, onClose: () => console.log("closed") });
inspector.render(<Inspector />);
```

`EVENT_SURFACE_CLOSED` is emitted before the host removes a native surface.
It has `nodeId=0`, `listenerId=0`, a null payload, and the closed
`surfaceId`; the matching root invokes `onClose`, rejects its pending commands
with `SurfaceClosedError` (including the closed `surfaceId`), and stops
accepting renders or commands. Closing one surface does not terminate the
shared transport. When the last native window is closed, the host tears down
the runtime and terminates the process. A malformed or unknown-surface frame is
rejected instead of silently opening a replacement window.

The host allocates surface IDs monotonically and never reuses an ID after a
native close or an explicit root unmount. `SurfaceHost.createRoot({ surfaceId
})` therefore rejects a retired ID with `SurfaceIdReusedError`, even when the
caller supplies a different `epoch`; callers must use a fresh host-allocated
ID. The `epoch` remains the renderer/native generation carried by every frame
for that surface and is checked against the root before dispatch. It is not a
permission to revive a retired ID and does not make same-ID recreation safe.

### Shared header fields

The following table applies to the positions that occur in Snapshot, Patch,
Event, and Command. The array index is zero-based.

| Position | Type | Constraint and semantics                                                                                                                            | Source                                                                                                             |
| -------: | ---- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
|        0 | u32  | Must be protocol version `3`.                                                                                                                       | `protocol.rs:7-11`; `wire/snapshot_patch.rs:29-34,70-75`; `wire/command.rs:94-99`; `wire/event.rs:183-195,295-317` |
|        1 | u32  | Must be `1` Snapshot, `2` Event, `3` Patch, or `4` Command.                                                                                         | Same discriminator checks as above.                                                                                |
|        2 | u32  | Surface identity. Events/commands must match the receiving surface; tree validation uses it to reject cross-surface data.                           | `protocol.rs:77,118,180,420`; `tree/validation.rs:22-30,58-64`                                                     |
|        3 | u32  | Surface epoch/generation. A new epoch can bootstrap with a base revision of zero.                                                                   | `protocol.rs:78,119,181,421`; `tree/validation.rs:13-31`                                                           |
|        4 | u32  | Snapshot/Patch `baseRevision`; Event `revision`; Command `afterRevision`. Revisions are ordered u32 values.                                         | `protocol.rs:79,120,182,422`; `tree/validation.rs:3-45,48-78`; `root-container.ts:228-235`                         |
|        5 | u32  | Snapshot/Patch `revision`; Event sequence; Command request ID. Event sequence is ordered by the receiver and duplicate/older sequences are ignored. | `protocol.rs:80,121,183,423`; `root-container.ts:528-538`                                                          |

Snapshot and Patch additionally require `revision > baseRevision`. A first
snapshot for an empty store must use `baseRevision=0`; a Patch cannot be
applied until a Snapshot exists, and a Patch base revision must equal the
retained tree's current revision (`tree/validation.rs:48-78`).

### Snapshot and Patch bodies

| Array/index              | Type                                                                                                           | Constraint and semantics                                                                                                                                                                                                                          | Source                                                                                               |
| ------------------------ | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| Snapshot `[6]`           | array of Node tuples                                                                                           | Complete node set. Each node has ten required fields and may carry optional `selectable`, `tooltip`, and pointer-move capability tails; shape and parent/child relationship are validated by the retained tree.                                                          | `protocol.rs:81`; `tree.rs:215-303; tree/validation.rs:81-229,231-273`                               |
| Patch `[6]`              | array of operation tuples                                                                                      | Operations are applied atomically in order; a failed patch rolls back.                                                                                                                                                                            | `protocol.rs:122,309-326`; `wire/snapshot_patch.rs:63-89,182-245`; `tree.rs:315-329,332-389,860-881` |
| Create operation `[0]=1` | `[1,id,parentId,index,kind,style,text,listenerId,hostProperties,accessibility,focusable,selectable?,tooltip?,acceptsPointerMove?]` | Same node fields as Snapshot. Optional tails are omitted when absent; a tooltip includes the selectable placeholder, and `acceptsPointerMove=true` is emitted only for an eligible View/Pressable listener. | `protocol.ts:123-136`; `wire/snapshot_patch.rs:90-116,136-153`                               |
| Update operation `[0]=2` | `[2,id,mask,style,text,listenerId,hostProperties,accessibility,focusable,selectable?,tooltip?,acceptsPointerMove?]`                | `mask` selects changed fields: style `1`, text `2`, listener `4`, host properties `8`, accessibility `16`, focusable `32`, selectable `64`, tooltip `128`, pointer move `256`; pointer capability updates append the optional tail and may clear it with `false`. | `protocol.ts:69-74,137-147`; `protocol.rs:60-65`; `wire/snapshot_patch.rs:90-130,155-176`       |
| Move operation `[0]=3`   | `[3,id,parentId,index]`                                                                                        | Reparents/reorders an existing node. Child indexes must remain contiguous.                                                                                                                                                                        | `protocol.ts:147`; `wire/snapshot_patch.rs:96-100,130-131,172-179,227-236`; `tree.rs:619-709`        |
| Delete operation `[0]=4` | `[4,id]`                                                                                                       | Deletes the node/subtree identified by `id`.                                                                                                                                                                                                      | `protocol.ts:148`; `wire/snapshot_patch.rs:96-100,133-134,237-243`                                   |

### Event fields

| Position | Type                   | Constraint and semantics                                                                                                                                                                                                                                                                            | Source                                                    |
| -------: | ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
|        0 | u32                    | `3`.                                                                                                                                                                                                                                                                                                | `protocol.ts:216-219`; `wire/event.rs:183-195,295-317`    |
|        1 | u32                    | `2`.                                                                                                                                                                                                                                                                                                | Same sources.                                             |
|        2 | u32                    | Surface identity.                                                                                                                                                                                                                                                                                   | `protocol.ts:219`; `wire/event.rs:183-195,295-317`        |
|        3 | u32                    | Epoch.                                                                                                                                                                                                                                                                                              | `protocol.ts:220`; `wire/event.rs:183-195,295-317`        |
|        4 | u32                    | Retained-tree revision associated with the event.                                                                                                                                                                                                                                                   | `protocol.ts:221`; `wire/event.rs:183-195,295-317`        |
|        5 | u32                    | Native event sequence. The receiver rejects a sequence not greater than the last accepted sequence.                                                                                                                                                                                                 | `protocol.ts:222`; `root-container.ts:528-538`            |
|        6 | u32                    | Target Host Node ID; root-level window events use synthetic root node `1`.                                                                                                                                                                                                                          | `protocol.ts:223`; `protocol.rs:424`; `dispatch.ts:67-80` |
|        7 | u32                    | Listener ID; `CommandResult` and `CloseRequested` use listener `0`, while target events use the mounted listener.                                                                                                                                                                                   | `protocol.ts:224`; `protocol.rs:425,701-703`              |
|        8 | u32                    | Event type `1..23`; the payload at position 9 is validated according to this value. `EVENT_SURFACE_CLOSED` additionally requires node/listener `0/0`; `EVENT_CLOSE_REQUESTED` requires node `1`/listener `0`; generic View/Pressable Focus/Blur uses a null payload and positive node/listener IDs. | `protocol.ts:687-693`; `wire/event.rs:20-171`             |
|        9 | null/string/array/bool | Event-specific payload from the directory in §3. Press/Hover/SurfaceClosed are null; Submit carries a string; CloseRequested carries `[9,requestId]`.                                                                                                                                               | `protocol.ts:488-558`; `wire/event.rs:45-171`             |
`EVENT_POINTER` uses a tagged payload family: tag `6` is pointer down/up and
tag `10` is pointer move. Pointer move is `[10,x,y,modifiers]`, where `x` and
`y` are finite, non-negative logical window pixels and modifiers are unique
members of `cmd`, `ctrl`, `alt`, `shift`, and `function`. The native painter
clamps coordinates to the viewport before encoding. The capability is opt-in:
only `View` and `Pressable` nodes with `onPointerMove` receive native move
registration and frames. Button state is not serialized. Hover remains an
edge notification and active drag delivery remains on the separate drag path.

### Node tuple

A Snapshot Node and a Patch Create operation use the ten required fields below,
followed by optional `selectable` and `tooltip` tails. The tooltip is only
legal for `View` and `Pressable`, is non-empty, contains no control
characters, and is at most 256 UTF-8 bytes. When present, the renderer includes
the selectable placeholder before the tooltip so old ten-field and
selectable-only tuples remain unambiguous. The host validates the kind, text
placement, focusability, listener eligibility, host properties, accessibility,
tooltip, and parent-child shape (`tree/validation.rs:81-229,231-273`). For a Text node with
`selectable=true`, the renderer consumes the flag with host-owned drag
selection, per-visual-row highlights, and direct clipboard copy; no selection
event is sent over this protocol.

| Position | Field            | Type                  | Constraint and semantics                                                                                                                                                                           | Source                                                                           |
| -------: | ---------------- | --------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
|        0 | `id`             | u32                   | Stable host identity, independent from React Fiber.                                                                                                                                                | `protocol.rs:222-233`; `wire/node.rs:3-15,121-160`                               |
|        1 | `parentId`       | u32                   | Parent node identity; root node uses parent `0`.                                                                                                                                                   | Same sources; `tree.rs:227-265`                                                  |
|        2 | `index`          | u32                   | Sibling index; stored siblings must be contiguous from zero.                                                                                                                                       | `tree.rs:296-303`; `tree/validation.rs:275-298`                                  |
|        3 | `kind`           | u32                   | `1=View`, `2=Text`, `3=Pressable`, `4=RawText`, `5=TextInput`, `6=VirtualList`, `7=Image`.                                                                                                         | `tree/validation.rs:81-95`; `protocol.ts:101-112`                                |
|        4 | `style`          | 42-slot array or null | Encoded Style; see the complete slot table below.                                                                                                                                                  | `style.ts:88-148,485-632`; `wire/node.rs:86-128,221-317`                         |
|        5 | `text`           | string or null        | Must be non-null only for RawText; all other kinds use null.                                                                                                                                       | `tree/validation.rs:97-104`                                                      |
|        6 | `listenerId`     | u32                   | Identifies a JavaScript listener table entry; zero means no listener.                                                                                                                              | `protocol.rs:229`; `tree/validation.rs:117-135`                                  |
|        7 | `hostProperties` | tagged array or null  | Required for TextInput, VirtualList, and Image; optional on View/Pressable when drag metadata is present; forbidden for other kinds.                                                               | `wire/node.rs:8-74,27-74,254-380`; `tree/validation.rs:133-135,158-229`          |
|        8 | `accessibility`  | 7-slot array or null  | Accessibility metadata; checked requires checkbox role.                                                                                                                                            | `wire/node.rs:8-74,254-380`; `tree/validation.rs:137-156`                        |
|        9 | `focusable`      | boolean               | Current retained-tree validation permits focusability only on View and Pressable.                                                                                                                  | `tree/validation.rs:105-109`                                                     |
|       10 | `selectable`     | optional boolean      | Omitted when false; only Text nodes may set it true.                                                                                                                                               | `protocol.ts:123-147`; `wire/node.rs:3-16,153-178`; `tree/validation.rs:111-115` |
|       11 | `tooltip`        | optional string       | Only View and Pressable; non-empty, no control characters, at most 256 UTF-8 bytes. Omitted when absent. If present, position 10 is always emitted as a selectable placeholder (`false` is valid). | `protocol.ts:123-147`; `wire/node.rs:3-16,149-166`; `tree.rs:584-628`            |

Images and RawText cannot contain children; RawText must be directly under
Text, and Text may contain only RawText (`tree/validation.rs:238-272`).

### `hostProperties` variants

The first element is the host-property tag. The remaining positions are
variant-specific. The untagged Rust enum and TypeScript union both require the
shape to match the node kind.

| Tag | Node kind      | Shape and constraints                                                                                                                                                                                                                                                                                                        | Source                                                                                                                                                                                                        |
| --: | -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
|   1 | TextInput      | `[1,value,placeholder,multiline,disabled,controlled,ackEditSeq,selectionStart,selectionEnd,markedStart,markedEnd,maxLength,selectionReversed]`                                                                                                                                                                               | Strings may be null only where shown; sequence/selection/maxLength are u32; marked positions are both null or both numbers and ranges are ordered; `selectionReversed` preserves the UTF-16 head orientation. | `protocol.ts:94-108,426-466`; `wire/node.rs:27-61,292-326,423-508`; `tree/validation.rs:176-189`                                                                                                                                                                                                                                                                                                                                                                                  |
|   2 | VirtualList    | `[2,itemCount,rangeStart,rangeEnd,estimatedItemSize,overscan]`; counts/ranges/overscan are u32, `rangeStart <= rangeEnd <= itemCount`, and estimated size is finite and positive. `estimatedItemSize` is the initial size hint for unmeasured or not-yet-committed rows; measured rows use their natural GPUI `list` height. | `protocol.ts:97,396-412`; `wire/node.rs:59-60,331-348,506-529`; `renderer/paint/virtual_list.rs:16-107`; `renderer.rs`                                                                                        |
|   3 | Image          | `[3,source,objectFit,fallbackSource                                                                                                                                                                                                                                                                                          | null]`                                                                                                                                                                                                        | Source and optional fallback are non-empty host-local path strings of at most 1024 UTF-8 bytes with no control characters; `PathBuf` forces local-file semantics, so `file://` and `http(s)://` strings are path names, not URL fetches. Object fit is `1=Fill`, `2=Contain`, `3=Cover`, `4=ScaleDown`, `5=None`. GPUI renders `fallbackSource` only after a load error; a loading replacement is also configured and appears after 200ms when the image has a stable element ID. | `protocol.ts:111-117,485-500`; `wire/node.rs:61-63,308-310,412-425,556-569`; `renderer/paint/image.rs:26-65`; pinned `references/zed/crates/gpui/src/elements/img.rs:277-425`                                                                                                                                                                                                                                                                                                                                                                                                                                                |
|   4 | View/Pressable | `[4,dragType                                                                                                                                                                                                                                                                                                                 | null,exportFiles                                                                                                                                                                                              | null,acceptsDragOver,acceptsDrop]`                                                                                                                                                                                                                                                                                                                                                                                                                                                | `dragType` is optional for drop targets; `acceptsDragOver` and `acceptsDrop` describe the independent JavaScript callbacks and may be true without a draggable source. Optional `exportFiles` contains 1..8 non-empty host-local paths of at most 1024 UTF-8 bytes with no control characters. Internal Event 20 notifications are emitted only for the advertised callbacks; outbound files are offered to the platform when the drag leaves the viewport and have no JavaScript completion event. macOS and Wayland Linux provide native starts; X11 and Windows retain the platform default that declines outbound drags. | `protocol.ts:116-124,521-545`; `wire/node.rs:64-72,372-385,449-483`; `renderer/paint/drag.rs:49-165` |

Image rendering is owned by pinned GPUI's `img()` element. The host passes a
`PathBuf`, so every source is a local filesystem path: relative paths are
resolved against the host process current working directory, while absolute
paths are portable across launch directories only when the application ships
the asset and derives that path itself. `file://`, `http://`, and `https://`
strings are not fetched; they are literal path names and normally fail to
load. Missing, unreadable, oversized, or non-image bytes produce an image load
error; the host keeps the retained tree and uses `fallbackSource` when one is
provided, otherwise the image paints blank. There is no source-extension or
MIME preflight in the React layer, and no new fallback layer is inserted.

`objectFit` maps to pinned GPUI exactly:

| Code/name         | Native behavior                                                                                                                                                       |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `1` / `fill`      | Stretch the decoded image to the element bounds.                                                                                                                      |
| `2` / `contain`   | Preserve aspect ratio, fit inside the element, and center the image.                                                                                                  |
| `3` / `cover`     | Preserve aspect ratio, cover the element, and center the oversized image; GPUI clips the painted tile to the element bounds regardless of the React `overflow` style. |
| `4` / `scaleDown` | If either intrinsic dimension exceeds the element, behave like `contain`; otherwise retain intrinsic size and center it.                                              |
| `5` / `none`      | Retain intrinsic size at the element's top-left origin; it is not CSS's centered object positioning.                                                                  |

The element's layout size comes from style and intrinsic image dimensions. With
both `width` and `height` omitted, a successfully decoded image contributes
its intrinsic dimensions (it does not collapse to zero). With exactly one
definite pixel dimension, GPUI derives the other from the intrinsic aspect
ratio. Percentage/flex/auto constraints do not trigger this cross-dimension
derivation; give both dimensions when a deterministic slot is required. Before
the image load completes, intrinsic size is unavailable, so an unsized image
can initially occupy zero space and then grow when decoding completes; provide
explicit dimensions to reserve layout space.

SVG bytes are supported by `img()` when format sniffing falls through to the
SVG renderer. The renderer rasterizes at a 2x quality scale and enforces its
own 8192-pixel render cap; SVG source paths therefore work, but the React
protocol does not expose GPUI's separate `svg()` asset resolver, MIME checks,
or SVG-specific controls.

When `fallbackSource` is set, the renderer configures both GPUI's error
replacement and delayed loading replacement. The latter appears only after
GPUI's 200ms loading delay and requires the stable image element ID installed
by the native painter; the error replacement appears when the primary loader
returns an error. The two callbacks share the same visual fallback path in
this protocol, so applications needing distinct loading and error artwork
cannot distinguish those states. Load errors are intentionally contained and
are not sent as JavaScript events: pinned GPUI's asynchronous loader returns an
error to `Img`, but `Img` consumes it for replacement/blank painting and
exposes no completion/error callback or protocol event. `onError` remains
unsupported.

TextInput `maxLength` is a u32 protocol value; its text-unit meaning is
specified in §5 and [ADR-0004](adr/0004-dual-length-semantics.md).

`TextInput` editing commands are host-owned and do not add protocol events:
Cmd/Ctrl-C copies the ordered UTF-16 selection, Cmd/Ctrl-X copies then removes
it, Cmd/Ctrl-V inserts text from the native clipboard, and Cmd/Ctrl-A selects
the full value. Option/Alt-Left/Right moves by the existing Unicode word
segmentation helper, with Shift extending the selection. Enter submits only a
focused single-line input; multiline Enter remains text insertion. The pinned
GPUI custom input-handler surface has no undo/redo history primitive, so
Cmd/Ctrl-Z and Shift-Cmd/Shift-Ctrl-Z remain an upstream gap. Secure/password
display is likewise unsupported because neither this host tuple nor pinned
GPUI exposes a password-obscuring primitive. Implementation evidence:
`renderer/paint/text_input.rs:394-456` and
`renderer/input.rs:1087-1249`.

### Accessibility tuple

The accessibility value is a seven-field tuple with optional tail fields:
`[role,label,description,disabled,checked,selected,value,expanded?,level?]`.
The tail follows the same append-only optional-slot convention as tooltip;
`level` is emitted only when present, and an `expanded` placeholder is retained
before it when needed. Old seven-field tuples remain valid.

| Position | Field       | Type/values               | Constraint and semantics                                                                                                                                                | Source                                                                                                                                                                 |
| -------: | ----------- | ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
|        0 | role        | u32                       | `0=unspecified`, `1=generic`, `2=button`, `3=text`, `4=textbox`, `5=checkbox`, `6=heading`; values above 6 are rejected.                                                | `props.ts:101-108`; `tree/validation.rs:137-156`                                                                                                                       |
|        1 | label       | string or null            | Accessible label; applied independently from description.                                                                                                               | `wire/node.rs:66-74,631-652`; `props.ts:142-168`; `renderer/paint/accessibility.rs:33-38`                                                                              |
|        2 | description | string or null            | Supplementary accessible description, applied independently from label.                                                                                                 | Same sources.                                                                                                                                                          |
|        3 | disabled    | boolean                   | Retained and validated, but not exposed in AccessKit: GPUI 0.2.2 has no public AX disabled-state builder. Pressable interaction/focus behavior still honors `disabled`. | Same sources; `renderer/paint/accessibility.rs:59-61`                                                                                                                  |
|        4 | checked     | boolean or null           | Non-null only with role `5=checkbox`; maps to AccessKit `Toggled::True/False`.                                                                                          | `props.ts:149-164`; `tree/validation.rs:144-155`                                                                                                                       |
|        5 | selected    | boolean or null           | Optional selected state.                                                                                                                                                | `wire/node.rs:51-61,631-652`                                                                                                                                           |
|        6 | value       | string or null            | Optional accessible value.                                                                                                                                              | Same sources.                                                                                                                                                          |
|        7 | expanded    | boolean or null, optional | Optional popup/disclosure state; maps to GPUI's public `aria_expanded` builder.                                                                                         | `props.ts:145-168`; `wire/node.rs:51-61,631-652`; `renderer/paint/accessibility.rs:53-54`; pinned `references/zed/crates/gpui/src/elements/div.rs:1339-1343,3408-3410` |
|        8 | level       | u32 or null, optional     | Positive value requiring role `6=heading`; maps to GPUI's public `aria_level` builder.                                                                                  | `props.ts:147-168`; `tree/validation.rs:156-170`; `renderer/paint/accessibility.rs:55-57`; pinned `references/zed/crates/gpui/src/elements/div.rs:1396-1400,3435-3437` |

The native painter applies recognized roles, labels, descriptions, checked
(`AccessKit::Toggled::True/False`), selected, values, expanded state, heading
levels, and stable IDs to the GPUI element. `generic` intentionally remains
GPUI's role-less container and does not produce an AccessKit node, so its other
fields are not exposed. Image, VirtualList, and RawText branches use the same
accessibility helper. AccessKit 0.24.1 has a `Live` property, but pinned GPUI
has no public `aria_live`/live-region builder or write path; live-region
announcements are therefore an upstream gap and no live field is carried.
The stock headless TestPlatform has no active AccessKit adapter; display-backed
desktop verification is required for a real tree inspection.

### Style tuple: all 42 slots

`style` is positional and always has 42 slots when present. `null` means the
field is unset. Color values are encoded RGBA u32 values from TypeScript
`#RRGGBB`/`#RRGGBBAA` strings. Numeric length/size fields are finite,
non-negative numbers except `fontSize`, which must be positive, `opacity`,
which must be in `0..1`, and positioning insets, which may be negative finite
pixel offsets (`style.ts:175-430`; `wire/node.rs:468-528`).
All producers and decoders use the complete 42-slot style form; the final two
slots carry `boxShadow` and `fontFamily`.

|                                                                         Slot | Field           | Wire value                                         | Values/constraint                                                                                                                                                                                                                                                                                                                                                                                    | Source                                                                                                                                   |
| ---------------------------------------------------------------------------: | --------------- | -------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
|                                                                            0 | width           | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | `style.ts:357-360`; `wire/node.rs:75-117,162-206`                                                                                        |
|                                                                            1 | height          | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | Same sources.                                                                                                                            |
|                                                                            2 | flexDirection   | u32                                                | `0=unset`, `1=row`, `2=column`, `3=row-reverse`, `4=column-reverse`.                                                                                                                                                                                                                                                                                                                                 | `style.ts:1-2,481`; `wire/node.rs:78-82,382-384`                                                                                         |
|                                                                            3 | flexGrow        | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | `style.ts:149`; `wire/node.rs:75-117,162-206`                                                                                            |
|                                                                            4 | padding         | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | Same sources.                                                                                                                            |
|                                                                            5 | gap             | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | Same sources.                                                                                                                            |
|                                                                            6 | backgroundColor | RGBA u32 or null                                   | Encoded `#RRGGBB`/`#RRGGBBAA`.                                                                                                                                                                                                                                                                                                                                                                       | `style.ts:364,268-272`; `wire/node.rs:75-117,162-206`                                                                                    |
|                                                                            7 | color           | RGBA u32 or null                                   | Encoded `#RRGGBB`/`#RRGGBBAA`.                                                                                                                                                                                                                                                                                                                                                                       | Same sources.                                                                                                                            |
|                                                                            8 | opacity         | f32 or null                                        | Finite, `0..1`.                                                                                                                                                                                                                                                                                                                                                                                      | `style.ts:217-219`; `wire/node.rs:75-117,162-206,381-440`                                                                                |
|                                                                            9 | transition      | `[durationMs,delayMs,easing,propertyMask]` or null | Duration/delay u32; easing `0=linear,1=easeIn,2=easeOut,3=easeInOut`; property mask bit `1=opacity`, bit `2=backgroundColor`, bit `4=width`, bit `8=height`; mask must be nonzero and contain no other bits. Width/height transitions trigger per-frame layout reflow. Generic transform scale/translate and borderRadius transitions are unsupported.                                               | `style.ts:230-264,372`; `wire/node.rs:75-120,162-206,256-279,381-440`                                                                    |
|                                                                           10 | justifyContent  | u32 or null                                        | `0=unset`, `1=flex-start`, `2=center`, `3=flex-end`, `4=space-between`, `5=space-around`, `6=space-evenly`.                                                                                                                                                                                                                                                                                          | `style.ts:294-307`; `wire/node.rs:75-117,162-206,381-440`                                                                                |
|                                                                           11 | alignItems      | u32 or null                                        | `0=unset`, `1=flex-start`, `2=center`, `3=flex-end`, `4=stretch`, `5=baseline`.                                                                                                                                                                                                                                                                                                                      | `style.ts:308-319`; `wire/node.rs:75-117,162-206,381-440`                                                                                |
|                                                                           12 | borderRadius    | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | `style.ts:180`; `wire/node.rs:75-117,162-206`                                                                                            |
|                                                                           13 | borderWidth     | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | Same sources.                                                                                                                            |
|                                                                           14 | borderColor     | RGBA u32 or null                                   | Encoded color.                                                                                                                                                                                                                                                                                                                                                                                       | `style.ts:372`; `wire/node.rs:75-117,162-206`                                                                                            |
|                                                                           15 | fontSize        | f32 or null                                        | Finite and strictly positive.                                                                                                                                                                                                                                                                                                                                                                        | `style.ts:182-185`; `wire/node.rs:75-117,162-206,381-440`                                                                                |
|                                                                           16 | fontWeight      | u32 or null                                        | `400=normal`, `500=medium`, `600=semibold`, `700=bold`, `900=heavy`.                                                                                                                                                                                                                                                                                                                                 | `style.ts:320-331`; `wire/node.rs:75-117,162-206,381-440`                                                                                |
|                                                                           17 | overflow        | u32 or null                                        | `0=unset`, `1=visible`, `2=hidden`, `3=scroll`.                                                                                                                                                                                                                                                                                                                                                      | `style.ts:375`; `wire/node.rs:75-117,162-206,381-440`                                                                                    |
|                                                                           18 | lineClamp       | u32 or null                                        | Null or integer `1..100`.                                                                                                                                                                                                                                                                                                                                                                            | `style.ts:192-195`; `wire/node.rs:75-117,162-206,381-440`                                                                                |
|                                                                           19 | textOverflow    | u32 or null                                        | `0=unset`, `1=clip`, `2=ellipsis`.                                                                                                                                                                                                                                                                                                                                                                   | `style.ts:377`; `wire/node.rs:75-117,162-206,381-440`                                                                                    |
|                                                                           20 | marginTop       | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | `style.ts:152-164`; `wire/node.rs:75-117,162-206`                                                                                        |
|                                                                           21 | marginRight     | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | Same sources.                                                                                                                            |
|                                                                           22 | marginBottom    | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | Same sources.                                                                                                                            |
|                                                                           23 | marginLeft      | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | Same sources.                                                                                                                            |
|                                                                           24 | fontStyle       | u32 or null                                        | `0=normal`, `1=italic`.                                                                                                                                                                                                                                                                                                                                                                              | `style.ts:332`; `wire/node.rs:75-117,162-206,381-440`                                                                                    |
|                                                                           25 | textDecoration  | u32 or null                                        | `0=none`, `1=underline`, `2=lineThrough`.                                                                                                                                                                                                                                                                                                                                                            | `style.ts:333-340`; `wire/node.rs:75-117,162-206,381-440`                                                                                |
|                                                                           26 | lineHeight      | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | `style.ts:152-164`; `wire/node.rs:75-117,162-206`                                                                                        |
|                                                                           27 | minWidth        | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | Same sources.                                                                                                                            |
|                                                                           28 | maxWidth        | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | Same sources.                                                                                                                            |
|                                                                           29 | minHeight       | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | Same sources.                                                                                                                            |
|                                                                           30 | maxHeight       | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | Same sources.                                                                                                                            |
|                                                                           31 | flexShrink      | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                                                                | Same sources.                                                                                                                            |
|                                                                           32 | alignSelf       | u32 or null                                        | `0=unset`, `1=start`, `2=end`, `3=flex-start`, `4=flex-end`, `5=center`, `6=baseline`, `7=stretch`.                                                                                                                                                                                                                                                                                                  | `style.ts:341-356`; `wire/node.rs:75-117,162-206,381-440`                                                                                |
|                                                                           33 | position        | u32                                                | `0=relative` (default), `1=absolute`, `2=overlay`; relative is a post-layout correction, absolute anchors to the closest positioned ancestor/origin and leaves no layout space, and overlay uses a local `left`/`top` anchor with deferred priority and viewport fit for dropdowns/popovers. Overlay is not a general portal or z-index and rejects `right`/`bottom`.                                | `style.ts:1,159-171,512`; `tree/validation.rs:354-365`; `wire/node.rs:76-235`                                                            |
|                                                                           34 | left            | f32 or null                                        | Finite pixel offset; negative values are allowed. For `overlay`, omitted `left` defaults to `0` and this offset is supplied to the local anchor.                                                                                                                                                                                                                                                     | `style.ts:167-177,419`; `wire/node.rs:75-117,162-206,381-440`                                                                            |
|                                                                           35 | top             | f32 or null                                        | Finite pixel offset; negative values are allowed. For `overlay`, omitted `top` defaults to `0` and this offset is supplied to the local anchor.                                                                                                                                                                                                                                                      | Same sources.                                                                                                                            |
|                                                                           36 | right           | f32 or null                                        | Finite pixel offset; negative values are allowed for `relative`/`absolute`; must be null for `overlay`.                                                                                                                                                                                                                                                                                              | `style.ts:276-277`; `tree/validation.rs:360-364`; `wire/node.rs:495-496`                                                                 |
|                                                                           37 | bottom          | f32 or null                                        | Finite pixel offset; negative values are allowed for `relative`/`absolute`; must be null for `overlay`.                                                                                                                                                                                                                                                                                              | `style.ts:276-277`; `tree/validation.rs:360-364`; `wire/node.rs:495-496`                                                                 |
|                                                                           38 | cursor          | u32 or null                                        | Cursor code: `0=default`, `1=text`, `2=pointer`, `3=grab`, `4=grabbing`, `5=not-allowed`, `6=context-menu`, `7=crosshair`, `8=vertical-text`, `9=alias`, `10=copy`, `11=no-drop`, `12=move`, `13=ew-resize`, `14=ns-resize`, `15=nesw-resize`, `16=nwse-resize`, `17=col-resize`, `18=row-resize`. Windows may fall back to Arrow for unsupported variants; headless backends do not render cursors. | `style.ts:1-21,127-165,342-384,515-516`; `protocol.rs:353-394`; `wire/node.rs:75-117,162-206,381-440`; `renderer/paint/style.rs:174-194` |
|                                                                           39 | textAlign       | u32                                                | `0=unset`, `1=left`, `2=center`, `3=right`; physical alignment only, not logical RTL start/end.                                                                                                                                                                                                                                                                                                      | `style.ts:2,56-57,306-308,528`; `protocol.rs:394`; `wire/node.rs:59-99,144-233,340-399`; `renderer/paint/style.rs:225-231`               |
|                                                                           40 | boxShadow       | `[tag,payload]` or null                            | `tag=1` payload `[offsetX,offsetY,blurRadius,spreadRadius,rgba,inset01]`; `tag=2` payload is a two-element array of those payloads. Offsets are finite f32 values; blur/spread are finite non-negative f32 values; at most two shadows.                                                                                                                                                              | `style.ts:29-38,200-256,483-496`; `wire/node.rs:76-128,174-219,445-466,468-528`; `renderer/paint/style.rs:154-166`                       |
|                                                                           41 | fontFamily      | string or null                                     | Non-empty, at most 64 Unicode characters, no control characters. GPUI resolves unavailable primary families through its configured fallback stack.                                                                                                                                                                                                                                                   | `style.ts:247-255,428-430`; `wire/node.rs:445-448,512-513`; `renderer/paint/style.rs:199-203`                                            |
| `row-reverse` and `column-reverse` are physical flex-axis mirrors only. This |
| protocol does not expose a container direction or text base-direction field; |
| Unicode bidi shaping remains platform behavior and explicit RTL layout/caret |
|                     semantics are unsupported until GPUI exposes those APIs. |

Transition property lists reject duplicates and unsupported properties before
encoding (`style.ts:251-262`). The Rust side independently validates ranges,
finite values, weight, and enum codes (`wire/node.rs:381-440`).

### Transition semantics

The transition tuple is applied natively on frame ticks; it does not create
per-frame JavaScript commits. `easing` defaults to `easeInOut` when omitted,
and `delayMs` is held before the easing curve begins. A retarget samples the
current presentation value, starts a new generation, and restarts the declared
delay from that retarget. Completion is emitted once for that generation.

When a changed style omits `transition`, the previous transition metadata is
used for a changed supported property so opacity and background-color
removals animate back instead of snapping. Opacity's unset target is `1.0`;
an unset background target is the transparent variant of the previous color.
Width and height animate only between two numeric values: an unset
(`auto`) endpoint has no numeric protocol target and changes immediately.
Background colors use straight RGBA/sRGB-channel interpolation. `borderRadius`
and generic `transform` scale/translate remain unsupported because GPUI has no
generic transition primitive for them: its public `Transformation` path is
SVG-only and explicitly does not affect layout or hit testing.

GPUI exposes no `zIndex` style field. Absolute elements participate in the
normal subtree paint and hit-test order. Explicit `overlay` elements are
deferred above normal siblings and use GPUI's local anchored placement to fit
inside the window viewport; use them for dropdowns/popovers, not as a general
portal or z-index substitute.

## 3. Event directory

Every event has the ten fields in the Event table above. `EVENT_PRESS`,
`EVENT_HOVER`, and `EVENT_SURFACE_CLOSED` require a null payload; SurfaceClosed
also requires `nodeId=0` and `listenerId=0`. `EVENT_SUBMIT` carries a string.
Window resize uses the three-number payload
`[width,height,scaleFactor]`; Rust accepts integer/float32 combinations through
`WindowResizeWire`, while TypeScript accepts finite non-negative dimensions and
a positive scale factor (`wire/event.rs`; `protocol.ts`). Focus and Blur retain
the tag-1 TextInput payload, while View and Pressable focus observers emit the
same event codes with a null payload. `EVENT_CLOSE_REQUESTED` is root-scoped,
uses `nodeId=1`/`listenerId=0`, and carries `[9,requestId]` while an asynchronous
close decision is pending.

|                                                                              Code | Name                 | Payload shape                                               | Validation and semantics                                                                                                                                                                                                                                                                                                                                    | Source                                                                                                                                                                                                                     |
| --------------------------------------------------------------------------------: | -------------------- | ----------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
|                                                                                 1 | Press                | `null`                                                      | Semantic activation notification.                                                                                                                                                                                                                                                                                                                           | `protocol.ts:430-432`; `wire/event.rs:20-45,283-289`                                                                                                                                                                       |
|                                                                                 2 | Change               | `[1,text,start,end,markedStart,markedEnd,editSeq,reversed]` | Tag `1`; text string; u32 ordered selection/edit sequence; marked pair both null or ordered; `reversed` is the UTF-16 head orientation.                                                                                                                                                                                                                     | `protocol.ts:230-240,668-691`; `wire/event.rs:362-483`                                                                                                                                                                     |
|                                                                                 3 | Selection            | Same tag-1 TextInput shape                                  | Same validation as Change.                                                                                                                                                                                                                                                                                                                                  | Same sources.                                                                                                                                                                                                              |
|                                                                                 4 | Focus                | Same tag-1 TextInput shape                                  | Same validation as Change.                                                                                                                                                                                                                                                                                                                                  | Same sources.                                                                                                                                                                                                              |
|                                                                                 5 | Blur                 | Same tag-1 TextInput shape                                  | Same validation as Change.                                                                                                                                                                                                                                                                                                                                  | Same sources.                                                                                                                                                                                                              |
|                                                                                 6 | CommandResult        | `[2,requestId,command,nodeId,success,error,value            | null]`                                                                                                                                                                                                                                                                                                                                                      | Tag `2`; command must be one of the implemented command codes `1..29`; success bool; error string/null; value is null when the command has no typed return value, otherwise it uses the tags below.                        | `protocol.ts:209-217,529-5…
|                                                                                 7 | VisibleRange         | `[3,start,end]`                                             | Tag `3`; u32 range with `start <= end`.                                                                                                                                                                                                                                                                                                                     | `protocol.ts:533-541`; `wire/event.rs:81-89,244-250,395-395`                                                                                                                                                               |
|                                                                                 8 | AnimationComplete    | `[4,generation]`                                            | Tag `4`; generation u32.                                                                                                                                                                                                                                                                                                                                    | `protocol.ts:543-550`; `wire/event.rs:90-97,250-252,399-399`                                                                                                                                                               |
|                                                                                 9 | Key                  | `[5,key,modifiers,action]`                                  | Tag `5`; non-empty key; unique modifiers from `cmd`, `ctrl`, `alt`, `shift`, `function`; action `1=down`, `2=repeat`, `3=up`.                                                                                                                                                                                                                     | `protocol.ts:487-496`; `wire/event.rs:98-100,253-254,397,492-529`                                                                                                                                                          |
|                                                                                10 | Pointer              | `[6,button,modifiers,action,clickCount,x,y]`                | Tag `6`; button `1=left` through `5=forward`; unique known modifiers; action `1=down`/`2=up`; clickCount > 0 u32; `x`/`y` are required finite non-negative logical window pixels. The host clamps native coordinates to the viewport before encoding.                                                                                                                                 | `protocol.ts:323,723-754`; `wire/event.rs:468,550-600`; `renderer/paint/mod.rs:286-320`                                                                                                                                    |
|                                                                                10 | PointerMove          | `[10,x,y,modifiers]`                                       | Tag `10` in the existing Pointer event family; finite non-negative logical window pixels clamped to the viewport by the host, plus unique known modifiers. Emitted only for `View`/`Pressable` nodes with `onPointerMove`; no button state is serialized. Hover edge notifications remain `null` and drag-over remains a separate event path. | `protocol.ts:330,730-780`; `wire/event.rs:411-414,477,610-638`; `renderer/paint/mod.rs:338-363` |
|                                                                                11 | Hover                | `null`                                                      | Semantic hover change; payload must be null.                                                                                                                                                                                                                                                                                                      | `protocol.ts:430-432`; `wire/event.rs:20-45,283-289`                                                                                                                                                                       |
|                                                                                12 | Scroll               | `[7,deltaKind,dx,dy,x,y,modifiers]`                         | Tag `7`; delta kind `1=pixels`/`2=lines`; four finite numeric coordinates/deltas; unique known modifiers.                                                                                                                                                                                                                                                   | `protocol.ts:471-485`; `wire/event.rs:104-106,255-256,403,575-614`                                                                                                                                                         |
|                                                                                13 | Submit               | string                                                      | The submitted text, including the empty string. It is emitted for a focused single-line TextInput; multiline Enter remains text insertion.                                                                                                                                                                                                        | `protocol.ts:202,430-433`; `protocol.rs:594-636`; `wire/event.rs:165-168,271-274`                                                                                                                                          |
|                                                                                14 | WindowResize         | `[width,height,scaleFactor]`                                | Width/height are finite non-negative logical pixels; `scaleFactor` is finite and positive. Root observer uses node `1`/listener `0`; scale-factor-only changes are reported as resize observations.                                                                                                                                                         | `protocol.ts:243,562-574`; `protocol.rs:506-509,765-817`; `wire/event.rs:109-125,372-404`; `renderer.rs:408-471`                                                                                                           |
|                                                                                15 | WindowActivation     | `boolean`                                                   | Untagged boolean; root observer uses node `1`/listener `0`.                                                                                                                                                                                                                                                                                                 | `protocol.ts:204,433`; `protocol.rs:663-685`; `wire/event.rs:114-116,259-262`                                                                                                                                              |
|                                                                                16 | SurfaceClosed        | `null`                                                      | Emitted before native teardown; `nodeId=0`, `listenerId=0`, and `surfaceId` identifies the closed surface. The matching root invokes `onClose`.                                                                                                                                                                                                             | `protocol.ts:34,244,604`; `protocol.rs:22,693-710`; `wire/event.rs:166-168,283-289`                                                                                                                                        |
|                                                                                17 | Action               | string                                                      | Root action selected from the native application menu; `nodeId=1`, `listenerId=0`, non-empty and at most 256 Unicode scalar values.                                                                                                                                                                                                                         | `protocol.ts:35,260,462-463,634-636`; `protocol.rs:29,438-440,713-733`; `wire/event.rs:117-124,275-278`                                                                                                                    |
|                                                                                18 | WindowAppearance     | `"light"                                                    | "dark"`                                                                                                                                                                                                                                                                                                                                                     | Root-level `nodeId=1`, `listenerId=0`; vibrant GPUI variants fold to these two semantic values. Initial registration emits a value, and changes are coalesced with the existing next-frame window observation.             | `protocol.ts:36,471-476,638-648`; `protocol.rs:30,439-467,740-759`; `renderer.rs:301-380`; `wire/event.rs:125-134,279-282` |
|                                                                                19 | Layout               | `[x,y,width,height]`                                        | Finite f32 bounds for a mounted View, Pressable, Text, or Image with `onLayout`; target node/listener identify the callback. Native measurement reports after post-layout prepaint, defers the first callback to the next frame, and deduplicates exact frames.                                                                                             | `protocol.ts:37,231,476-483,649-660`; `protocol.rs:31,468,762-791`; `renderer/paint/mod.rs:122-136`; `renderer.rs:309-335`; `wire/event.rs:135-146,263-266`                                                                |
|                                                                                20 | Drag                 | `[1,type]`, `[2,type]`, or `[3,[path,...]]`                 | Node-level drag notifications. Tag `1` is drag-over, tag `2` is internal drop, and tag `3` is external file drop. Types are safe non-empty strings (up to 128 scalars); external paths are ordered strings (up to 256 paths, 4096 bytes each). `onDragOver` is notification-only; native accepts drops without a JS can-drop round trip.                    | `protocol.ts:38-41,241-244,485-520,687-693`; `protocol.rs:32,497-499,824-910`; `wire/event.rs:147-164,267-270,338-340`                                                                                                     |
|                                                                                21 | NotificationResponse | `[tag,actionId                                              | null]`                                                                                                                                                                                                                                                                                                                                                      | Root-level (`nodeId=1`, `listenerId=0`) response to a system notification body or action. The host-generated tag is non-empty; action IDs are nullable and bounded. Unknown/closed-surface tags are discarded by the host. | `protocol.ts:38,244-264,506-552,708-735`; `protocol.rs`; `renderer.rs`; `host/main.rs`                                     |
|                                                                                22 | PointerDownOutside   | `[8,x,y]`                                                   | Capture-phase mouse-down notification for a rendered `position="overlay"` View. `x`/`y` are finite logical window coordinates. It is emitted only when the point is outside the overlay bounds and outside its direct anchor subtree; the listener/node IDs identify the overlay callback. Escape dismissal remains a JS key handler.                       | `protocol.ts`; `protocol.rs`; `wire/event.rs`; `renderer/paint/overlay.rs:164-194`                                                                                                                                         |
|                                                                                23 | CloseRequested       | `[9,requestId]`                                             | Root-level (`nodeId=1`, `listenerId=0`) asynchronous close request. The host emits at most one request while a native close decision is pending; JavaScript resolves it with `ResolveCloseRequest`.                                                                                                                                                         | `protocol.ts`; `protocol.rs`; `wire/event.rs:183-187`; `host/main.rs`                                                                                                                                                      |
|        Single-line TextInput geometry is backed by its shaped native text layout: |
|      `bounds_for_range` maps UTF-16 selection offsets through the shaped line and |
|   `character_index_for_point` localizes the point before mapping its x coordinate |
|  back to UTF-16. Multiline input uses `shape_text` and cached `WrappedLine` rows; |
|      line positions map through UTF-16 line starts, cross-line ranges union their |
| first/last visual-row bounds, and explicit empty/trailing-newline lines are kept. |
| IME candidate placement remains display-backed and requires desktop verification. |

### CommandResult value tags

The optional seventh slot of a CommandResult is either null/absent or one of:

| Tag | Shape                  | Semantics and constraints                                                                                                                                                                | Source                                        |
| --: | ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- |
|   1 | `[1,number]`           | Finite numeric result. OpenSurface uses it for the newly allocated positive `surfaceId`; write-text-file uses it for UTF-8 bytes written.                                                | `protocol.ts`; `wire/event.rs`                |
|   2 | `[2,[width,height]]`   | Finite, non-negative numeric pair. Used by get-window-size.                                                                                                                              | Same sources; `commands.rs`                   |
|   3 | `[3,boolean]`          | Boolean result. Used by get-focus.                                                                                                                                                       | Same sources; `commands.rs`                   |
|   4 | `[4,string]`           | UTF-8 string up to 1 MiB. Used by ClipboardRead and LoadFont (the latter returns the font metadata family name).                                                                            | `protocol.ts`; `wire/event.rs`                |
|   5 | `[5,[string,...]]`     | Non-empty selected path list. Every path must be a non-empty UTF-8 string; an empty list is invalid and cancellation uses an absent/null optional value instead. Used by FileDialogOpen. | `protocol.ts`; `wire/event.rs`                |
|   6 | `[6,string]`           | UTF-8 text returned by ReadTextFile, bounded to `MAX_FILE_READ_BYTES = MAX_FRAME_SIZE - 1024`.                                                                                           | `protocol.ts`; `wire/event.rs`; `commands.rs` |
|   7 | `[7,[formatCode,bin]]` | Encoded clipboard image bytes. `formatCode` is `1=png`, `2=jpeg`, `3=gif`, or `4=svg`; bytes are non-empty and at most `MAX_CLIPBOARD_IMAGE_BYTES = MAX_FRAME_SIZE - 1024`.              | `protocol.ts`; `wire/event.rs`; `commands.rs` |
|   8 | `[8,[x,y,width,height]]` | Finite logical/global window bounds. Width and height are non-negative; macOS coordinates use the global screen-relative top-left origin.                                               | `protocol.ts`; `wire/event.rs`; `commands.rs` |
|   9 | `[9,[fullscreen,maximized]]` | Boolean fullscreen and maximized state in that order.                                                                                                                               | `protocol.ts`; `wire/event.rs`; `commands.rs` |

## 4. Command directory

A Command array is `[3,4,surfaceId,epoch,afterRevision,requestId,nodeId,kind,payload]`.
`afterRevision` must equal the host retained-tree revision at dispatch; stale
commands receive an unsuccessful CommandResult (`commands.rs:25-31`).
Root-only commands require `nodeId=1`. Node commands are checked against the
kind and mounted state of `nodeId`; an unsupported node/command pair receives
an unsuccessful result. The TypeScript side rejects unsupported node command
pairs before framing (`root-container.ts:177-237`), while Rust repeats the
checks in `commands.rs:186-331`.

| Code | Name                | Node ownership                                                       | Payload                                                                                                                                                                                                             | Receipt/host semantics                                                                                                                                                                                                                                                                                                                                                    | Source                                                                                       |
| ---: | ------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
|    1 | Focus               | Node-level; View/TextInput focus handles                             | `null`                                                                                                                                                                                                              | Focuses a mounted, enabled focus target; failure returns `success=false`.                                                                                                                                                                                                                                                                                                 | `wire/command.rs:211-222,325-336`; `commands.rs:197-240`                                     |
|    2 | Blur                | Node-level; View/TextInput                                           | `null`                                                                                                                                                                                                              | Blurs the target; failure returns a normal unsuccessful receipt.                                                                                                                                                                                                                                                                                                          | Same sources.                                                                                |
|    3 | SetSelection        | Node-level; TextInput                                                | `[start,end]` u32 pair, `start <= end`                                                                                                                                                                              | Host additionally checks the range against the current UTF-16 text length and emits selection state.                                                                                                                                                                                                                                                                      | `root-container.ts:203-209`; `wire/command.rs:211-222,325-336`; `commands.rs:242-264`        |
|    4 | ScrollToIndex       | Node-level; VirtualList                                              | `[index,0]` u32 pair                                                                                                                                                                                                | Uses the first value; index must be less than `itemCount`.                                                                                                                                                                                                                                                                                                                | `nodes.ts:139-141`; `commands.rs:273-303`                                                    |
|    5 | ScrollToEnd         | Node-level; VirtualList                                              | `null`                                                                                                                                                                                                              | Scrolls the mounted list to its end.                                                                                                                                                                                                                                                                                                                                      | `wire/command.rs:197-222,325-336`; `commands.rs:273-321`                                     |
|    6 | SetTitle            | Root-only (`nodeId=1`)                                               | string                                                                                                                                                                                                              | Non-empty, at most 256 Unicode scalar values; sets native window title.                                                                                                                                                                                                                                                                                                   | `root-container.ts:239-265`; `wire/command.rs:132-138,325-336`; `commands.rs:32-41`          |
|    7 | ResizeWindow        | Root-only (`nodeId=1`)                                               | `[width,height]` u32 pair                                                                                                                                                                                           | Each dimension is `1..16384` inclusive; resizes the client area.                                                                                                                                                                                                                                                                                                          | `root-container.ts:328-339`; `wire/command.rs:187-196,325-336`; `commands.rs:123-140`        |
|    8 | ZoomWindow          | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                                                                                              | Toggles GPUI window zoom semantics.                                                                                                                                                                                                                                                                                                                                       | `root-container.ts:341-347`; `commands.rs:142-149`                                           |
|    9 | ToggleFullscreen    | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                                                                                              | Toggles fullscreen semantics.                                                                                                                                                                                                                                                                                                                                             | `root-container.ts:345-347`; `commands.rs:150-157`                                           |
|   10 | OpenUrl             | Root-only (`nodeId=1`)                                               | string                                                                                                                                                                                                              | Non-empty HTTP/HTTPS URL, no whitespace, at most 2048 UTF-8 bytes; delegates to host URL opening.                                                                                                                                                                                                                                                                         | `root-container.ts:349-358`; `wire/command.rs:173-178,325-336`; `commands.rs:159-165`        |
|   11 | FocusNext           | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                                                                                              | Delegates to native tab-stop traversal.                                                                                                                                                                                                                                                                                                                                   | `root-container.ts:360-362`; `commands.rs:167-174`                                           |
|   12 | FocusPrev           | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                                                                                              | Delegates to reverse native tab-stop traversal.                                                                                                                                                                                                                                                                                                                           | `root-container.ts:364-365`; `commands.rs:175-182`                                           |
|   13 | GetWindowSize       | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                                                                                              | Returns value tag `2` with logical `[width,height]`; no resize side effect.                                                                                                                                                                                                                                                                                               | `root-container.ts:367-384`; `commands.rs:111-121`                                           |
|   14 | GetFocus            | Node-level focus handle                                              | `null`                                                                                                                                                                                                              | Returns value tag `3` with the native focus boolean; missing/non-focusable handle fails.                                                                                                                                                                                                                                                                                  | `nodes.ts:130-137`; `commands.rs:186-195`                                                    |
|   15 | ClipboardWrite      | Root-only (`nodeId=1`)                                               | string                                                                                                                                                                                                              | UTF-8 payload capped at 1 MiB; writes a string clipboard entry.                                                                                                                                                                                                                                                                                                           | `root-container.ts:386-390`; `commands.rs:59-80`                                             |
|   16 | ClipboardRead       | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                                                                                              | Returns value tag `4`; fails when the host has no text or text exceeds 1 MiB.                                                                                                                                                                                                                                                                                             | `root-container.ts:393-405`; `commands.rs:81-109`                                            |
|   17 | OpenSurface         | Root-only (`nodeId=1`) on the requesting, already registered surface | `[title,[width,height]]` or `[title,[width,height],[kind,resizable,minWidth,minHeight]]`; title at most 256 Unicode scalar values (empty allowed), dimensions `[0,0]` or each `1..16384`, options slots are `kind=0 | 1                                                                                                                                                                                                                                                                                                                                                                         | 2                                                                                            | null`, `resizable=boolean | null`, and paired positive `minWidth`/`minHeight`through`16384` | Opens a native window and returns CommandResult value tag `1` (`[1,surfaceId]`). The optional tail maps only creation-time GPUI fields: `Normal`, `Floating`, or `Dialog`, resizability, and minimum size. Popup, max size, runtime level/resizable changes, and a center switch are not exposed; old two-item payloads remain compatible. | `root-container.ts:403-461`; `wire/command.rs:6-59,216-239,397-449`; `main.rs:192-258` |
|   18 | FileDialogOpen      | Root-only (`nodeId=1`)                                               | `[title,[directories,multiple]]`, with both flags encoded as `0`/`1`                                                                                                                                                | Asynchronously opens the native picker with `files = !directories`, `directories`, and `multiple`. A selection completes with value tag `5`; cancellation is `success=true` with its optional value absent/null; platform failure is `success=false`.                                                                                                                     | `root-container.ts:402-425`; `wire/command.rs:150-156,325-336`; `commands.rs:21-88`          |
|   19 | FileDialogSave      | Root-only (`nodeId=1`)                                               | `defaultName` string (empty means no suggestion)                                                                                                                                                                    | Asynchronously opens the native save picker. A selected path completes with value tag `4`; cancellation is `success=true` with its optional value absent/null; platform failure is `success=false`. GPUI's raw save API has no title/prompt option.                                                                                                                       | `root-container.ts:428-444`; `wire/command.rs:157-162,325-336`; `commands.rs:89-132`         |
|   20 | ShowNotification    | Root-only (`nodeId=1`)                                               | `[title,body]` or `[title,body,[[actionId,label],...]]`; title UTF-8 ≤256 bytes, body UTF-8 ≤1024 bytes, at most 3 actions with IDs ≤64 and labels ≤256 UTF-8 bytes                                                 | Submits a tagged native notification. Action/body responses emit Event 21 with the host-generated tag; action ID is null for body activation. Delivery is platform best effort.                                                                                                                                                                                           | `root-container.ts:467-501`; `wire/command.rs:29-48,172-188,282-304`; `commands.rs:173-209`  |
|   21 | SetMenus            | Root-only (`nodeId=1`)                                               | `[[menuTitle,[item...]], ...]`; item `[0]` separator, `[1,actionName]` or `[1,actionName,[disabled,checked]]` (boolean flags), or `[2,[submenuTitle,[item...]]]`                                                    | Replaces the application menu tree. Omitted action flags default to `false`; state changes re-send the complete definition. Native action selection emits Event 17; disabled actions are unavailable to native activation and checked actions use GPUI's toggled indicator.                                                                                               | `root-container.ts:476-514`; `wire/command.rs:169-172,223-235,293-344`; `commands.rs:23-44`  |
|   22 | SetKeybindings      | Root-only (`nodeId=1`)                                               | `[[keystrokes,actionName], ...]`, at most 64 bindings; each keystrokes string is at most 64 UTF-8 bytes and each action name is 1..64 Unicode characters                                                            | Full-replaces this surface's binding set. The host validates every chord with GPUI `Keystroke::parse`, then clears and rebuilds the process-global union of all live surface sets atomically; an invalid chord returns `success=false` naming the entry and leaves the previous sets installed. A matched action uses Event 17 and routes to the active window's surface. | `root-container.ts:548-579`; `wire/command.rs:6-129,207-211,291-318`; `host/main.rs:237-335` |
|   24 | ResolveCloseRequest | Root-only (`nodeId=1`)                                               | `[requestId,allowCode]`, where `allowCode` is `0` or `1`                                                                                                                                                            | Resolves the pending Event 23 request. `0` leaves the window open; `1` removes it through the host close path. Unknown, stale, or already-resolved IDs are acknowledged/ignored without closing another request.                                                                                                                                                          | `root-container.ts`; `wire/command.rs:126-135,313-317`; `host/main.rs`                  |
|   25 | ReadTextFile        | Root-only (`nodeId=1`)                                               | absolute non-empty path string, no control characters, UTF-8 ≤1024 bytes                                                                                                                                            | Asynchronously reads a regular file on the background executor. Returns value tag `6`; rejects missing paths, directories, oversized files, and invalid UTF-8.                                                                                                                                                                                                            | `root-container.ts`; `wire/command.rs`; `renderer/commands.rs`                               |
|   26 | WriteTextFile       | Root-only (`nodeId=1`)                                               | `[absolutePath,content]`; path as above and content UTF-8 ≤`MAX_FILE_WRITE_BYTES = MAX_FRAME_SIZE - 1024` bytes                                                                                                     | Asynchronously replaces/creates the file using host filesystem semantics and returns value tag `1` with UTF-8 bytes written. Filesystem failures reject; symlinks follow ordinary OS semantics.                                                                                                                                                                           | `root-container.ts`; `wire/command.rs`; `renderer/commands.rs`                               |
|   27 | ClipboardWriteImage | Root-only (`nodeId=1`)                                               | `[formatCode,bin]`, with format `1=png`, `2=jpeg`, `3=gif`, or `4=svg`; non-empty binary bytes capped at `MAX_CLIPBOARD_IMAGE_BYTES = MAX_FRAME_SIZE - 1024`                                                        | Writes an encoded image through GPUI's native clipboard image entry. macOS and Windows support this path; X11 and Wayland return `platform-unsupported` rather than writing text.                                                                                                                                                                                         | `root-container.ts`; `wire/command.rs`; `renderer/commands.rs`                               |
|   28 | ClipboardReadImage  | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                                                                                              | Returns value tag `7` with the original encoded format and bytes; empty/non-image clipboard contents resolve as `null` in TypeScript. macOS and Windows support this path; X11 and Wayland return `platform-unsupported`.                                                                                                                                                 | `root-container.ts`; `wire/event.rs`; `renderer/commands.rs`                                 |
|   29 | LoadFont            | Root-only (`nodeId=1`)                                               | absolute non-empty regular-file path string, no control characters, UTF-8 ≤1024 bytes                                                                                                                               | Asynchronously reads a bounded font file, parses the first face's metadata family, and registers its bytes through `TextSystem::add_fonts`. Success returns value tag `4` with that family name. TTF/OTF are supported; WOFF/WOFF2 are not. The command does not invalidate GPUI's cached family resolution, so load before first layout/use; repeated calls are forwarded to the native backend. | `root-container.ts`; `wire/command.rs`; `renderer/commands.rs` |
|   30 | MinimizeWindow      | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                                                                                              | Requests native window minimization/miniaturization. The command is acknowledged after dispatch; the pinned headless TestWindow leaves this native operation unimplemented, so display-backed verification is required for the visible effect.                                                                                                                    | `root-container.ts`; `wire/command.rs`; `renderer/commands.rs`; `gpui::Window::minimize_window` |
|   31 | GetWindowBounds     | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                                                                                              | Returns value tag `8` with finite logical/global `[x,y,width,height]`; coordinates are screen-relative global top-left bounds on macOS, and width/height are non-negative. No position setter is exposed by pinned GPUI.                                                                                                   | `root-container.ts`; `wire/event.rs`; `renderer/commands.rs`; `gpui::Window::bounds`       |
|   32 | GetWindowState      | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                                                                                              | Returns value tag `9` as `[fullscreen,maximized]` from native state queries. `isFullscreen` and `isMaximized` are observations, not setters.                                                                                                                                            | `root-container.ts`; `wire/event.rs`; `renderer/commands.rs`; `gpui::Window`              |
|   33 | ActivateWindow      | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                                                                                              | Requests native activation/foreground focus. The pinned headless TestWindow reports `is_active=false`, so activation is proven by routing and requires display-backed observation for the visible effect; activation events remain the state channel.                                                                  | `root-container.ts`; `wire/command.rs`; `renderer/commands.rs`; `gpui::Window::activate_window` |

### Surface creation option platform matrix

The options are creation-time only; no command changes an existing native
window's level, resizability, or size constraints. The host maps `kind` to the
corresponding GPUI `WindowKind` and does not expose `WindowKind::PopUp`.

| Option/kind      | macOS                                   | Windows                                                                | X11                                                       | Wayland                                             | Web/test                                                                                             |
| ---------------- | --------------------------------------- | ---------------------------------------------------------------------- | --------------------------------------------------------- | --------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `normal`         | Normal level                            | Normal window                                                          | Normal window                                             | Normal toplevel                                     | Normal/default behavior                                                                              |
| `floating`       | `NSFloatingWindowLevel`                 | Floating is not topmost; parent/transient semantics are platform-owned | Transient to the owning window                            | Parent-linked toplevel                              | Popup/floating/dialog creation is rejected by the web adapter; test adapters may ignore native level |
| `dialog`         | Sheet/modal dialog when an owner exists | Modal dialog disables the active parent                                | Dialog/transient and modal hints                          | Parent-linked; modal extension is optional          | Popup/floating/dialog creation is rejected by the web adapter                                        |
| `resizable`      | Native resizable style mask             | Native resize styles                                                   | Adapter does not consume this flag in the pinned revision | Adapter does not expose a portable resizable toggle | Adapter-defined/no native effect                                                                     |
| `minSize: [w,h]` | Content minimum                         | `WM_GETMINMAXINFO` minimum                                             | WM size hint minimum                                      | `xdg_toplevel.set_min_size`                         | Adapter-defined/no native effect                                                                     |

`maxSize`, a generic always-on-top/window-level value, and runtime setters are
not represented because the pinned GPUI `WindowOptions`/`PlatformWindow`
contracts provide no portable fields or mutators. `WindowKind::floating` is
therefore intentionally documented as above-parent, not global always-on-top.
The first host window remains a host-owned centered `800×600` window; it is
created before JavaScript starts and has no `Root.openSurface` negotiation.

FileDialogOpen and FileDialogSave are the asynchronous exceptions to the
otherwise immediate command path. The host starts the GPUI foreground picker,
continues processing the render loop and other commands, and emits the
CommandResult only from the receiver completion task. A cancelled picker is a
normal successful completion with the optional value omitted; a platform
error rejects the JavaScript promise.

SystemNotification delivery is platform best effort: macOS, Linux, and
Windows provide native implementations, while Web and visual-test platforms
inherit no-op defaults. Windows AppUserModel identity belongs to host packaging
and is not configured by the React wire. Headless coverage uses TestPlatform;
success only means the host submitted the notification.
ShowNotification actions are optional and capped at three bounded
`[actionId,label]` pairs. The host generates each notification tag and registers
one process-wide response callback, routing `[tag,actionId|null]` to the
originating root; body activation uses a null action ID. Responses for closed
surfaces are discarded.

SetMenus uses GPUI's typed `Action` bridge internally. The host wraps each
string action name in a host-owned action, applies the action's checked and
disabled flags, and routes the active window's selection as Event 17. GPUI's
native menu is process-wide, so the active surface owns the event; menu updates
replace the current tree and JavaScript state changes must re-send the complete
definition. Platform implementations differ: macOS installs an NSMenu, while
Linux/Windows retain owned menu data for their UI integrations and Web/test
platforms are no-ops.

SetKeybindings is intentionally separate from menu definitions. Its
`keystrokes` string is one or more GPUI keystrokes separated by ASCII
whitespace; each chord has optional modifiers followed by a key, with
components joined by `-`. Examples are `cmd-shift-p` and
`ctrl-k ctrl-1`. GPUI accepts the modifier names `ctrl`, `alt`, `shift`, `fn`,
`secondary`, `cmd`/`super`/`win`; an optional `->key_char` suffix is reserved
for GPUI test-event syntax. Context predicates are not included in v1, so
bindings are global to the host keymap and only the active window's surface
receives the resulting Event 17. Rebuild order is ascending surface ID followed
by declaration order, matching GPUI's later-binding precedence for conflicts.

Every processed command produces a CommandResult event with the original
request ID, command, node ID, success flag, nullable error, and optional value
(`commands.rs:333-347`).

## 5. Encoding and validation conventions

### Numeric representation

TypeScript encodes with MessagePack options `{ sortKeys: false,
forceFloat32: true }` (`protocol.ts:3`). Integer values remain integer
encodings; non-integer numeric values use float32. Rust uses `f32` fields and
accepts the integer/float32 forms described by the wire enums, including mixed
forms for window dimensions (`wire/event.rs:318-360`). The exact byte versus semantic
contract is [ADR-0003](adr/0003-cross-language-golden-vector-contract.md); do
not require two independent MessagePack libraries to choose identical bytes
for every integral float value.

### Length units

Resource-like strings are bounded in UTF-8 bytes: Image sources are at most
1024 bytes, URLs at most 2048 bytes, and clipboard text at most 1 MiB.
TextInput `maxLength` is a UTF-16 code-unit limit because it governs JavaScript
selection/edit positions. The rationale and boundary consequences are in
[ADR-0004](adr/0004-dual-length-semantics.md).

### String constraints

- Image paths: non-empty, no control characters, maximum 1024 UTF-8 bytes;
  relative paths resolve from the host process working directory.
- URLs: non-empty `http://` or `https://`, no whitespace, maximum 2048 UTF-8
  bytes, and a non-empty host suffix.
- Window titles: `SetTitle` requires non-empty text; `OpenSurface` permits an empty title. Both allow at most 256 Unicode scalar values.
- File-dialog open titles and save default names: at most 256 Unicode scalar values; save has no title option because GPUI's raw save picker exposes only a directory and suggested name.
- File-dialog result paths: non-empty UTF-8 strings; open selections must contain at least one path.
- Clipboard strings and CommandResult text values: maximum 1 MiB UTF-8 bytes.
- TextInput text itself is a string; selection and marked ranges are u32
  UTF-16 positions and must be ordered.

The TypeScript and Rust validators independently enforce these boundaries;
resource byte semantics and TextInput UTF-16 semantics must not be collapsed
into one generic character counter.

## 6. Evolution rules

1. **Keep both implementations synchronized.** A protocol change updates the
   TypeScript tuple/types/validation and Rust public types/wire conversion and
   validation in the same change. Update both directions of golden fixtures and
   invalid cases when applicable.
2. **Prefer compatible tail additions.** CommandResult's optional value is an
   example: append a tagged optional field so a six-field result remains valid;
   do not silently reorder or reinterpret existing positions. Any incompatible
   change needs an explicit protocol-version decision.
3. **Regenerate, do not hand-edit golden output.** The supported command is:

   ```sh
   make protocol-golden-generate
   ```

   It runs the Rust `protocol_golden` example and
   `scripts/protocol-golden.ts`, writing `fixtures/protocol/rust_to_ts.hex`,
   `ts_to_rust.hex`, `invalid.hex`, and `frames.hex` (`Makefile:36-39`).

4. **Preserve the three-layer fixture contract.** Golden tests lock each
   producer's own bytes, cross-direction semantic equality, and permitted
   numeric dual forms as defined by [ADR-0003](adr/0003-cross-language-golden-vector-contract.md).
5. **Add a focused invalid vector for new rejection rules.** Rust and
   TypeScript must agree whether malformed tags, ranges, enum values, trailing
   bytes, or oversized frames are rejected; the existing invalid and frame
   fixtures are checked by `protocol-golden.test.ts:84-126`.
6. **Preserve surface ownership.** New surface-scoped messages must carry the
   requesting or owning `surfaceId`; do not introduce a global `0/0` control
   frame. Unknown surface IDs remain hard errors, and a new window is created
   only by the validated OpenSurface command. Event sequence allocation and
   dispatch remain per surface, including host acknowledgements and close
   notifications.
7. **Keep asynchronous command completion explicit.** Dialog commands must
   emit their acknowledgement from the picker completion task, not when the
   request is accepted. Cancellation is a successful result with no optional
   value; new value tags must reject empty selections rather than overloading
   cancellation.

## 7. Fixture walkthroughs

The following rows are from `fixtures/protocol/ts_to_rust.hex`; the matching
Rust rows in `rust_to_ts.hex` are used by the golden test
(`protocol-golden.test.ts:64-82`). The hex shown is the MessagePack payload,
not the four-byte frame prefix.

### Command: set title

Fixture `ts-command-title` (`ts_to_rust.hex:16`):

```text
99030407032a6a0106b1476f6c64656e20f09f9880207469746c65
```

`0x99` is a nine-element array. Decoding the positions gives:

```text
[3, 4, 7, 3, 42, 106, 1, 6, "Golden 😀 title"]
```

- `3` = protocol version, `4` = Command.
- `surfaceId=7`, `epoch=3`, `afterRevision=42`, `requestId=106`.
- `nodeId=1` identifies the root surface; `kind=6` is SetTitle.
- The final MessagePack string is the title. This row exercises Unicode text
  and the root-only command shape.

### Event: window resize

Fixture `ts-event-window-resize` (`ts_to_rust.hex:34`):

```text
9a030207032a0b01000e93ca44482000ca4416200002
```

The decoded array is:

```text
[3, 2, 7, 3, 42, 11, 1, 0, 14, [800.5, 600.5, 2]]
```

- Event type `14` is WindowResize; node `1` and listener `0` identify the root observer.
- `0x93` is the current untagged three-element payload. The semantic dimensions
  are `800.5 × 600.5` logical pixels and the display scale factor is `2`.

### Event: submit with text

Fixture `ts-event-submit-text` (`ts_to_rust.hex:30`):

```text
9a030207032a1005090dae7375626d69747465642074657874
```

The decoded array is:

```text
[3, 2, 7, 3, 42, 16, 5, 9, 13, "submitted text"]
```

- Event type `13` is Submit, targeted at TextInput node `5` with listener `9`.
- The final string carries the submitted text; an empty string represents an
  empty submission.

### Event: pointer with coordinates

The pointer payload is always seven fields:

```text
[6, button, modifiers, action, clickCount, x, y]
```

`x` and `y` are logical window pixels. GPUI supplies a position on every
mouse-down and mouse-up; the host clamps it to the viewport before encoding.
Both decoders require finite, non-negative coordinates, and there is no
legacy five-field/default-coordinate form. The event is intentionally sparse:
coordinates are carried for press/release, not hover or pointer-move events.

These rows demonstrate why the fixture test checks both decoded meaning and
producer bytes: the same positional contract is exercised by TypeScript and
Rust without requiring every numeric encoding choice to be byte-identical.
