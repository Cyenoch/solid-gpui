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
  `crates/react-gpui/src/protocol/wire.rs`.
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
(`crates/react-gpui/src/protocol.rs:808-860`). TypeScript `FrameDecoder` accepts
fragmented input, emits every complete payload in order, accepts coalesced
frames, and resets/throws on an oversized length
(`packages/react-gpui/src/protocol.ts:294-344`).

The checked boundary cases are in `fixtures/protocol/frames.hex:3-7`:
`00000000` is an empty payload, `04000000` declares four bytes, and
`01000001` declares `16,777,217` bytes (one over the limit). A complete
maximum-size frame still needs its full payload; a header alone remains
pending/truncated as specified by the reader.

## 2. Message matrix

All protocol messages are MessagePack arrays. The first two fields are always
`protocol=3` and a message discriminator:

| Message  | Array shape                                                                            | Meaning                                                                  | Source                                   |
| -------- | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ | ---------------------------------------- |
| Snapshot | `[3, 1, surfaceId, epoch, baseRevision, revision, nodes]`                              | Complete tree bootstrap for an epoch or a complete replacement snapshot. | `protocol.rs:71-109`, `wire.rs:8-46`     |
| Event    | `[3, 2, surfaceId, epoch, revision, sequence, nodeId, listenerId, eventType, payload]` | Native semantic notification to JavaScript.                              | `protocol.rs:402-428`, `wire.rs:501-617` |
| Patch    | `[3, 3, surfaceId, epoch, baseRevision, revision, operations]`                         | Atomic incremental tree changes after a snapshot.                        | `protocol.rs:112-173`, `wire.rs:49-88`   |
| Command  | `[3, 4, surfaceId, epoch, afterRevision, requestId, nodeId, kind, payload]`            | JavaScript request to the host surface or a host node.                   | `protocol.rs:176-219`, `wire.rs:90-220`  |

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
`root.openSurface({ title?, width?, height? })`. The command is sent from that
root (`surfaceId` is the requesting root and `nodeId=1`), with omitted values
normalized to an empty title and `[0,0]` host-default dimensions. When the
CommandResult resolves with value tag `1`, its number is the new native
surface ID. The application then registers that ID and starts rendering:

```ts
const surfaceId = await root.openSurface({ title: "Inspector", width: 640, height: 480 });
const inspector = host.createRoot({ surfaceId, onClose: () => console.log("closed") });
inspector.render(<Inspector />);
```

`EVENT_SURFACE_CLOSED` is emitted before the host removes a native surface.
It has `nodeId=0`, `listenerId=0`, a null payload, and the closed
`surfaceId`; the matching root invokes `onClose` and stops accepting renders
or commands. Closing one surface does not terminate the shared transport.
When the last native window is closed, the host tears down the runtime and
terminates the process. A malformed or unknown-surface frame is rejected
instead of silently opening a replacement window.

### Shared header fields

The following table applies to the positions that occur in Snapshot, Patch,
Event, and Command. The array index is zero-based.

| Position | Type | Constraint and semantics                                                                                                                            | Source                                                                       |
| -------: | ---- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
|        0 | u32  | Must be protocol version `3`.                                                                                                                       | `protocol.rs:7-11`; `wire.rs:28-33,69-73,127-131,234-252`                    |
|        1 | u32  | Must be `1` Snapshot, `2` Event, `3` Patch, or `4` Command.                                                                                         | Same discriminator checks as above.                                          |
|        2 | u32  | Surface identity. Events/commands must match the receiving surface; tree validation uses it to reject cross-surface data.                           | `protocol.rs:77,118,180,420`; `tree.rs:878-893,914-920`                      |
|        3 | u32  | Surface epoch/generation. A new epoch can bootstrap with a base revision of zero.                                                                   | `protocol.rs:78,119,181,421`; `tree.rs:878-887`                              |
|        4 | u32  | Snapshot/Patch `baseRevision`; Event `revision`; Command `afterRevision`. Revisions are ordered u32 values.                                         | `protocol.rs:79,120,182,422`; `tree.rs:862-934`; `root-container.ts:228-235` |
|        5 | u32  | Snapshot/Patch `revision`; Event sequence; Command request ID. Event sequence is ordered by the receiver and duplicate/older sequences are ignored. | `protocol.rs:80,121,183,423`; `root-container.ts:528-538`                    |

Snapshot and Patch additionally require `revision > baseRevision`. A first
snapshot for an empty store must use `baseRevision=0`; a Patch cannot be
applied until a Snapshot exists, and a Patch base revision must equal the
retained tree's current revision (`tree.rs:862-934`).

### Snapshot and Patch bodies

| Array/index              | Type                                                                                      | Constraint and semantics                                                                                                   | Source                                                              |
| ------------------------ | ----------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| Snapshot `[6]`           | array of Node tuples                                                                      | Complete node set. Each node shape and parent/child relationship is validated by the retained tree.                        | `protocol.rs:81`; `tree.rs:937-1100`                                |
| Patch `[6]`              | array of operation tuples                                                                 | Operations are applied atomically in order; a failed patch rolls back.                                                     | `protocol.rs:122,309-326`; `wire.rs:365-418`; `tree.rs:310-326`     |
| Create operation `[0]=1` | `[1,id,parentId,index,kind,style,text,listenerId,hostProperties,accessibility,focusable]` | Same node fields as Snapshot, with operation tag `1`.                                                                      | `protocol.ts:123-135`; `wire.rs:386-399`                            |
| Update operation `[0]=2` | `[2,id,mask,style,text,listenerId,hostProperties,accessibility,focusable]`                | `mask` selects changed fields: style `1`, text `2`, listener `4`, host properties `8`, accessibility `16`, focusable `32`. | `protocol.ts:69-74,136-145`; `protocol.rs:60-65`; `wire.rs:401-412` |
| Move operation `[0]=3`   | `[3,id,parentId,index]`                                                                   | Reparents/reorders an existing node. Child indexes must remain contiguous.                                                 | `protocol.ts:147`; `wire.rs:414-415`; `tree.rs:1102-1121`           |
| Delete operation `[0]=4` | `[4,id]`                                                                                  | Deletes the node/subtree identified by `id`.                                                                               | `protocol.ts:148`; `wire.rs:417-418`                                |

### Event fields

| Position | Type                   | Constraint and semantics                                                                                                                              | Source                                                    |
| -------: | ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
|        0 | u32                    | `3`.                                                                                                                                                  | `protocol.ts:216-219`; `wire.rs:501-506`                  |
|        1 | u32                    | `2`.                                                                                                                                                  | Same sources.                                             |
|        2 | u32                    | Surface identity.                                                                                                                                     | `protocol.ts:219`; `wire.rs:506`                          |
|        3 | u32                    | Epoch.                                                                                                                                                | `protocol.ts:220`; `wire.rs:507`                          |
|        4 | u32                    | Retained-tree revision associated with the event.                                                                                                     | `protocol.ts:221`; `wire.rs:508`                          |
|        5 | u32                    | Native event sequence. The receiver rejects a sequence not greater than the last accepted sequence.                                                   | `protocol.ts:222`; `root-container.ts:528-538`            |
|        6 | u32                    | Target Host Node ID; root-level window events use synthetic root node `1`.                                                                            | `protocol.ts:223`; `protocol.rs:424`; `dispatch.ts:67-80` |
|        7 | u32                    | Listener ID; `CommandResult` uses listener `0`, while target events use the mounted listener.                                                         | `protocol.ts:224`; `protocol.rs:425,701-703`              |
|        8 | u32                    | Event type `1..20`; the payload at position 9 is validated according to this value. `EVENT_SURFACE_CLOSED` additionally requires node/listener `0/0`. | `protocol.ts:687-693`; `wire.rs:347-493`                  |
|        9 | null/string/array/bool | Event-specific payload from the directory in §3. Press/Hover/SurfaceClosed are null; Submit accepts legacy null or a string.                          | `protocol.ts:488-558`; `wire.rs:303-493`                  |

### Node tuple

A Snapshot Node and a Patch Create operation use the following ten fields. The
host validates the kind, text placement, focusability, listener eligibility,
host properties, accessibility, and parent-child shape (`tree.rs:937-1100`).

| Position | Field            | Type                  | Constraint and semantics                                                                   | Source                                      |
| -------: | ---------------- | --------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------- |
|        0 | `id`             | u32                   | Stable host identity, independent from React Fiber.                                        | `protocol.rs:222-233`; `wire.rs:373-384`    |
|        1 | `parentId`       | u32                   | Parent node identity; root node uses parent `0`.                                           | Same sources; `tree.rs:1061-1099`           |
|        2 | `index`          | u32                   | Sibling index; stored siblings must be contiguous from zero.                               | `tree.rs:1102-1121`                         |
|        3 | `kind`           | u32                   | `1=View`, `2=Text`, `3=Pressable`, `4=RawText`, `5=TextInput`, `6=VirtualList`, `7=Image`. | `tree.rs:937-951`; `protocol.ts:101-112`    |
|        4 | `style`          | 33-slot array or null | Encoded Style; see the complete slot table below.                                          | `style.ts:58-93,276-393`; `wire.rs:462-496` |
|        5 | `text`           | string or null        | Must be non-null only for RawText; all other kinds use null.                               | `tree.rs:953-959`                           |
|        6 | `listenerId`     | u32                   | Identifies a JavaScript listener table entry; zero means no listener.                      | `protocol.rs:229`; `tree.rs:967-981`        |
|        7 | `hostProperties` | tagged array or null  | Required for TextInput, VirtualList, and Image; optional on View/Pressable when drag metadata is present; forbidden for other kinds. | `wire.rs:420-448,641-645`; `tree.rs:1000-1068` |
|        8 | `accessibility`  | 7-slot array or null  | Accessibility metadata; checked requires checkbox role.                                    | `wire.rs:450-459`; `tree.rs:983-997`        |
|        9 | `focusable`      | boolean               | Current retained-tree validation permits focusability only on View and Pressable.          | `tree.rs:961-965`                           |

Images and RawText cannot contain children; RawText must be directly under
Text, and Text may contain only RawText (`tree.rs:1071-1099`).

### `hostProperties` variants

The first element is the host-property tag. The remaining positions are
variant-specific. The untagged Rust enum and TypeScript union both require the
shape to match the node kind.

| Tag | Node kind   | Shape and constraints                                                                                                                                                                                                                                                             | Source                                                                      |
| --: | ----------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
|   1 | TextInput   | `[1,value,placeholder,multiline,disabled,controlled,ackEditSeq,selectionStart,selectionEnd,markedStart,markedEnd,maxLength]`; strings may be null only where shown; sequence/selection/maxLength are u32; marked positions are both null or both numbers, and ranges are ordered. | `protocol.ts:83-99,358-395`; `wire.rs:428-442,854-887`; `tree.rs:1005-1019` |
|   2 | VirtualList | `[2,itemCount,rangeStart,rangeEnd,estimatedItemSize,overscan]`; counts/ranges/overscan are u32, `rangeStart <= rangeEnd <= itemCount`, and estimated size is finite and positive.                                                                                                 | `protocol.ts:97,396-412`; `wire.rs:444-445,871-877`; `tree.rs:1020-1031`    |
|   3 | Image       | `[3,source,objectFit]`; source is non-empty, at most 1024 UTF-8 bytes, and has no control character; object fit is `1..5`.                                                                                                                                                        | `protocol.ts:414-425`; `wire.rs:447-448,850-886`; `tree.rs:1032-1043`       |
|   4 | View/Pressable | `[4,dragType|null]`; dragType is optional for drop-only nodes and otherwise a non-empty safe string up to 128 Unicode scalars. | `protocol.ts:107-115,485-495`; `wire.rs:645,667-669,1117-1150`; `tree.rs:1005-1018` |

TextInput `maxLength` is a u32 protocol value; its text-unit meaning is
specified in §5 and [ADR-0004](adr/0004-dual-length-semantics.md).

### Accessibility tuple

| Position | Field       | Type/values     | Constraint and semantics                                                                                                 | Source                                |
| -------: | ----------- | --------------- | ------------------------------------------------------------------------------------------------------------------------ | ------------------------------------- |
|        0 | role        | u32             | `0=unspecified`, `1=generic`, `2=button`, `3=text`, `4=textbox`, `5=checkbox`, `6=heading`; values above 6 are rejected. | `props.ts:101-108`; `tree.rs:984-989` |
|        1 | label       | string or null  | Accessible label.                                                                                                        | `wire.rs:451-458`; `props.ts:118-125` |
|        2 | description | string or null  | Accessible description.                                                                                                  | Same sources.                         |
|        3 | disabled    | boolean         | Retained and validated, but not exposed in AccessKit: GPUI 0.2.2 has no public AX disabled-state builder. Pressable interaction/focus behavior still honors `disabled`. | Same sources; `renderer/paint.rs:626-628` |
|        4 | checked     | boolean or null | Non-null only with role `5=checkbox`.                                                                                    | `props.ts:113-116`; `tree.rs:991-995` |
|        5 | selected    | boolean or null | Optional selected state.                                                                                                 | `wire.rs:456-458`                     |
|        6 | value       | string or null  | Optional accessible value.                                                                                               | Same sources.                         |
The native painter applies recognized roles, labels, descriptions, checked
(`AccessKit::Toggled::True/False`), selected, values, and stable IDs to the
GPUI element. `generic` intentionally remains GPUI's role-less container and
does not produce an AccessKit node, so its other fields are not exposed.
Image, VirtualList, and RawText branches use the same accessibility helper.
The stock headless TestPlatform has no active AccessKit adapter; display-backed
desktop verification is required for a real tree inspection.

### Style tuple: all 38 slots

`style` is positional and always has 38 slots when present. `null` means the
field is unset. Color values are encoded RGBA u32 values from TypeScript
`#RRGGBB`/`#RRGGBBAA` strings. Numeric length/size fields are finite,
non-negative numbers except `fontSize`, which must be positive, `opacity`,
which must be in `0..1`, and positioning insets, which may be negative finite
pixel offsets (`style.ts:105-280`; `wire.rs:1104-1171`).

| Slot | Field           | Wire value                                         | Values/constraint                                                                                                                                                                                                                                                                                                                                      | Source                                                                      |
| ---: | --------------- | -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------- |
|    0 | width           | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | `style.ts:357-360`; `wire.rs:462-496`                                       |
|    1 | height          | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | Same sources.                                                               |
|    2 | flexDirection   | u32                                                | `0=unset`, `1=row`, `2=column`.                                                                                                                                                                                                                                                                                                                        | `style.ts:360`; `wire.rs:465`                                               |
|    3 | flexGrow        | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | `style.ts:149`; `wire.rs:466`                                               |
|    4 | padding         | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | Same sources.                                                               |
|    5 | gap             | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | Same sources.                                                               |
|    6 | backgroundColor | RGBA u32 or null                                   | Encoded `#RRGGBB`/`#RRGGBBAA`.                                                                                                                                                                                                                                                                                                                         | `style.ts:364,268-272`; `wire.rs:468`                                       |
|    7 | color           | RGBA u32 or null                                   | Encoded `#RRGGBB`/`#RRGGBBAA`.                                                                                                                                                                                                                                                                                                                         | Same sources.                                                               |
|    8 | opacity         | f32 or null                                        | Finite, `0..1`.                                                                                                                                                                                                                                                                                                                                        | `style.ts:217-219`; `wire.rs:471,935`                                       |
|    9 | transition      | `[durationMs,delayMs,easing,propertyMask]` or null | Duration/delay u32; easing `0=linear,1=easeIn,2=easeOut,3=easeInOut`; property mask bit `1=opacity`, bit `2=backgroundColor`, bit `4=width`, bit `8=height`; mask must be nonzero and contain no other bits. Width/height transitions trigger per-frame layout reflow. Generic transform scale/translate and borderRadius transitions are unsupported. | `style.ts:230-264,372`; `wire.rs:498-499,905-835,945-950`                   |
|   10 | justifyContent  | u32 or null                                        | `0=unset`, `1=flex-start`, `2=center`, `3=flex-end`, `4=space-between`, `5=space-around`, `6=space-evenly`.                                                                                                                                                                                                                                            | `style.ts:294-307`; `wire.rs:473,906-908`                                   |
|   11 | alignItems      | u32 or null                                        | `0=unset`, `1=flex-start`, `2=center`, `3=flex-end`, `4=stretch`, `5=baseline`.                                                                                                                                                                                                                                                                        | `style.ts:308-319`; `wire.rs:474,907-908`                                   |
|   12 | borderRadius    | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | `style.ts:180`; `wire.rs:475`                                               |
|   13 | borderWidth     | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | Same sources.                                                               |
|   14 | borderColor     | RGBA u32 or null                                   | Encoded color.                                                                                                                                                                                                                                                                                                                                         | `style.ts:372`; `wire.rs:477`                                               |
|   15 | fontSize        | f32 or null                                        | Finite and strictly positive.                                                                                                                                                                                                                                                                                                                          | `style.ts:182-185`; `wire.rs:478,937-938`                                   |
|   16 | fontWeight      | u32 or null                                        | `400=normal`, `500=medium`, `600=semibold`, `700=bold`, `900=heavy`.                                                                                                                                                                                                                                                                                   | `style.ts:320-331`; `wire.rs:479,940-941`                                   |
|   17 | overflow        | u32 or null                                        | `0=unset`, `1=visible`, `2=hidden`, `3=scroll`.                                                                                                                                                                                                                                                                                                        | `style.ts:375`; `wire.rs:480,910-912`                                       |
|   18 | lineClamp       | u32 or null                                        | Null or integer `1..100`.                                                                                                                                                                                                                                                                                                                              | `style.ts:192-195`; `wire.rs:481,913-915`                                   |
|   19 | textOverflow    | u32 or null                                        | `0=unset`, `1=clip`, `2=ellipsis`.                                                                                                                                                                                                                                                                                                                     | `style.ts:377`; `wire.rs:482,915-917`                                       |
|   20 | marginTop       | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | `style.ts:152-164`; `wire.rs:483`                                           |
|   21 | marginRight     | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | Same sources.                                                               |
|   22 | marginBottom    | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | Same sources.                                                               |
|   23 | marginLeft      | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | Same sources.                                                               |
|   24 | fontStyle       | u32 or null                                        | `0=normal`, `1=italic`.                                                                                                                                                                                                                                                                                                                                | `style.ts:332`; `wire.rs:487,918-920`                                       |
|   25 | textDecoration  | u32 or null                                        | `0=none`, `1=underline`, `2=lineThrough`.                                                                                                                                                                                                                                                                                                              | `style.ts:333-340`; `wire.rs:488,921-923`                                   |
|   26 | lineHeight      | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | `style.ts:152-164`; `wire.rs:489`                                           |
|   27 | minWidth        | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | Same sources.                                                               |
|   28 | maxWidth        | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | Same sources.                                                               |
|   29 | minHeight       | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | Same sources.                                                               |
|   30 | maxHeight       | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | Same sources.                                                               |
|   31 | flexShrink      | f32 or null                                        | Finite, non-negative.                                                                                                                                                                                                                                                                                                                                  | Same sources.                                                               |
|   32 | alignSelf       | u32 or null                                        | `0=unset`, `1=start`, `2=end`, `3=flex-start`, `4=flex-end`, `5=center`, `6=baseline`, `7=stretch`.                                                                                                                                                                                                                                                    | `style.ts:341-356`; `wire.rs:495,924-926`                                   |
|   33 | position        | u32                                                | `0=relative` (default), `1=absolute`; relative is a post-layout correction, absolute anchors to the closest positioned ancestor/origin and leaves no layout space.                                                                                                                                                                                     | `style.ts:1,164-173,418`; `style.rs:1227-1247`; `wire.rs:973-977,1001-1005` |
|   34 | left            | f32 or null                                        | Finite pixel offset; negative values are allowed.                                                                                                                                                                                                                                                                                                      | `style.ts:167-177,419`; `wire.rs:974,1120-1126`                             |
|   35 | top             | f32 or null                                        | Finite pixel offset; negative values are allowed.                                                                                                                                                                                                                                                                                                      | Same sources.                                                               |
|   36 | right           | f32 or null                                        | Finite pixel offset; negative values are allowed.                                                                                                                                                                                                                                                                                                      | Same sources.                                                               |
|   37 | bottom          | f32 or null                                        | Finite pixel offset; negative values are allowed.                                                                                                                                                                                                                                                                                                      | Same sources.                                                               |

Transition property lists reject duplicates and unsupported properties before
encoding (`style.ts:251-262`). The Rust side independently validates ranges,
finite values, weight, and enum codes (`wire.rs:905-953`).

GPUI exposes no `zIndex` style field. Absolute elements participate in the
normal subtree paint and hit-test order; later/deferred overlay siblings are
the practical top layer. This is why the React host does not invent a z-index
slot. Viewport-aware popovers may require a later GPUI `anchored`/`deferred`
integration rather than raw inset coordinates.

## 3. Event directory

Every event has the ten fields in the Event table above. `EVENT_PRESS`,
`EVENT_HOVER`, and `EVENT_SURFACE_CLOSED` require a null payload; SurfaceClosed
also requires `nodeId=0` and `listenerId=0`. `EVENT_SUBMIT` accepts either
legacy null or a string; the Rust constructor has both `submit` and
`submit_with_text` forms (`protocol.rs:594-636`). Window resize is an untagged
two-number array; Rust accepts integer/float32 combinations through
`WindowResizeWire`, while TypeScript accepts finite non-negative numbers
(`wire.rs:635-651`; `protocol.ts:430-445`).

| Code | Name              | Payload shape                                                                        | Validation and semantics                                                                                                                                         | Source                                                                      |
| ---: | ----------------- | ------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
|    1 | Press             | `null`                                                                               | Semantic activation notification.                                                                                                                                | `protocol.ts:430-432`; `wire.rs:584-590`                                    |
|    2 | Change            | `[1,text,start,end,markedStart,markedEnd,editSeq]`                                   | Tag `1`; text string; u32 ordered selection/edit sequence; marked pair both null or ordered.                                                                     | `protocol.ts:552-571`; `wire.rs:256-259,1064-1083`                          |
|    3 | Selection         | Same tag-1 TextInput shape                                                           | Same validation as Change.                                                                                                                                       | Same sources.                                                               |
|    4 | Focus             | Same tag-1 TextInput shape                                                           | Same validation as Change.                                                                                                                                       | Same sources.                                                               |
|    5 | Blur              | Same tag-1 TextInput shape                                                           | Same validation as Change.                                                                                                                                       | Same sources.                                                               |
|    6 | CommandResult     | `[2,requestId,command,nodeId,success,error]` or with optional tagged value at slot 6 | Tag `2`; command must be `1..21`; success bool; error string/null; value tags below. Six-field old results decode with value `None`.                             | `protocol.ts:209-217,529-566`; `wire.rs:310-338,728-735,1415-1448`          |
|    7 | VisibleRange      | `[3,start,end]`                                                                      | Tag `3`; u32 range with `start <= end`.                                                                                                                          | `protocol.ts:533-541`; `wire.rs:286-293`                                    |
|    8 | AnimationComplete | `[4,generation]`                                                                     | Tag `4`; generation u32.                                                                                                                                         | `protocol.ts:543-550`; `wire.rs:295-301`                                    |
|    9 | Key               | `[5,key,modifiers,action]`                                                           | Tag `5`; non-empty key; unique modifiers from `cmd`, `ctrl`, `alt`, `shift`, `function`; action `1=down`, `2=repeat`, `3=up`.                                    | `protocol.ts:487-496`; `wire.rs:303-305,685-687,1106-1132`                  |
|   10 | Pointer           | `[6,button,modifiers,action,clickCount]`                                             | Tag `6`; button `1=left` through `5=forward`; unique known modifiers; action `1=down`/`2=up`; clickCount > 0 u32.                                                | `protocol.ts:447-470`; `wire.rs:306-307,689,1134-1164`                      |
|   11 | Hover             | `null`                                                                               | Semantic hover change; payload must be null.                                                                                                                     | `protocol.ts:430-432`; `wire.rs:323-324,584-590`                            |
|   12 | Scroll            | `[7,deltaKind,dx,dy,x,y,modifiers]`                                                  | Tag `7`; delta kind `1=pixels`/`2=lines`; four finite numeric coordinates/deltas; unique known modifiers.                                                        | `protocol.ts:471-485`; `wire.rs:309-311,691,1178-1204`                      |
|   13 | Submit            | `null` or string                                                                     | Null is the legacy Enter notification; string carries submitted text. It is emitted for a focused single-line TextInput; multiline Enter remains text insertion. | `protocol.ts:202,430-433`; `protocol.rs:594-636`; `wire.rs:322-324,580-583` |
|   14 | WindowResize      | `[width,height]`                                                                     | Untagged two-number payload; both finite and non-negative. Root observer uses node `1`/listener `0`.                                                             | `protocol.ts:203,434-445`; `protocol.rs:639-661`; `wire.rs:312-318`         |
|   15 | WindowActivation  | `boolean`                                                                            | Untagged boolean; root observer uses node `1`/listener `0`.                                                                                                      | `protocol.ts:204,433`; `protocol.rs:663-685`; `wire.rs:319-320`             |

| 16 | SurfaceClosed | `null` | Emitted before native teardown; `nodeId=0`, `listenerId=0`, and `surfaceId` identifies the closed surface. The matching root invokes `onClose`. | `protocol.ts:34,244,604`; `protocol.rs:22,693-710`; `wire.rs:347` |
| 17 | Action | string | Root action selected from the native application menu; `nodeId=1`, `listenerId=0`, non-empty and at most 256 Unicode scalar values. | `protocol.ts:35,260,462-463,634-636`; `protocol.rs:29,438-440,713-733`; `wire.rs:298-300,376-380` |
| 18 | WindowAppearance | `"light" | "dark"` | Root-level `nodeId=1`, `listenerId=0`; vibrant GPUI variants fold to these two semantic values. Initial registration emits a value, and changes are coalesced with the existing next-frame window observation. | `protocol.ts:36,471-476,638-648`; `protocol.rs:30,439-467,740-759`; `renderer.rs:301-380`; `wire.rs:366,450-459,799-802,1570-1573` |
| 19 | Layout | `[x,y,width,height]` | Finite f32 bounds for a mounted View, Pressable, Text, or Image with `onLayout`; target node/listener identify the callback. Native measurement reports after post-layout prepaint, defers the first callback to the next frame, and deduplicates exact frames. | `protocol.ts:37,231,476-483,649-660`; `protocol.rs:31,468,762-791`; `renderer/paint.rs`; `renderer.rs:309-335`; `wire.rs:461-473,803-807,1590-1595` |
| 20 | Drag | `[1,type]`, `[2,type]`, or `[3,[path,...]]` | Node-level drag notifications. Tag `1` is drag-over, tag `2` is internal drop, and tag `3` is external file drop. Types are safe non-empty strings (up to 128 scalars); external paths are ordered strings (up to 256 paths, 4096 bytes each). `onDragOver` is notification-only; native accepts drops without a JS can-drop round trip. | `protocol.ts:38-41,241-244,485-520,687-693`; `protocol.rs:32,497-499,824-910`; `wire.rs:367-368,474-489,810-813,873-900,1641-1648` |

### CommandResult value tags

The optional seventh slot of a CommandResult is either null/absent or one of:

| Tag | Shape                | Semantics and constraints                                                                                                                                                                | Source                                                     |
| --: | -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
|   1 | `[1,number]`         | Finite numeric result. OpenSurface uses it for the newly allocated positive `surfaceId`; other commands may use a finite numeric result.                                                 | `protocol.ts:188-195,256-258`; `wire.rs:676-681,1261-1277` |
|   2 | `[2,[width,height]]` | Finite, non-negative numeric pair. Used by get-window-size.                                                                                                                              | Same sources; `commands.rs:111-121`                        |
|   3 | `[3,boolean]`        | Boolean result. Used by get-focus.                                                                                                                                                       | Same sources; `commands.rs:186-195`                        |
|   4 | `[4,string]`         | UTF-8 string up to 1 MiB. Used by clipboard-read.                                                                                                                                        | Same sources; `commands.rs:81-109`                         |
|   5 | `[5,[string,...]]`   | Non-empty selected path list. Every path must be a non-empty UTF-8 string; an empty list is invalid and cancellation uses an absent/null optional value instead. Used by FileDialogOpen. | `protocol.ts:197-204,271-287`; `wire.rs:703-719,1370-1378` |

## 4. Command directory

A Command array is `[3,4,surfaceId,epoch,afterRevision,requestId,nodeId,kind,payload]`.
`afterRevision` must equal the host retained-tree revision at dispatch; stale
commands receive an unsuccessful CommandResult (`commands.rs:25-31`).
Root-only commands require `nodeId=1`. Node commands are checked against the
kind and mounted state of `nodeId`; an unsupported node/command pair receives
an unsuccessful result. The TypeScript side rejects unsupported node command
pairs before framing (`root-container.ts:177-237`), while Rust repeats the
checks in `commands.rs:186-331`.

| Code | Name             | Node ownership                                                       | Payload                                                                                                                                 | Receipt/host semantics                                                                                                                                                                                                                                | Source                                                                        |
| ---: | ---------------- | -------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
|    1 | Focus            | Node-level; View/TextInput focus handles                             | `null`                                                                                                                                  | Focuses a mounted, enabled focus target; failure returns `success=false`.                                                                                                                                                                             | `wire.rs:197-207`; `commands.rs:197-240`                                      |
|    2 | Blur             | Node-level; View/TextInput                                           | `null`                                                                                                                                  | Blurs the target; failure returns a normal unsuccessful receipt.                                                                                                                                                                                      | Same sources.                                                                 |
|    3 | SetSelection     | Node-level; TextInput                                                | `[start,end]` u32 pair, `start <= end`                                                                                                  | Host additionally checks the range against the current UTF-16 text length and emits selection state.                                                                                                                                                  | `root-container.ts:203-209`; `wire.rs:198-205`; `commands.rs:242-264`         |
|    4 | ScrollToIndex    | Node-level; VirtualList                                              | `[index,0]` u32 pair                                                                                                                    | Uses the first value; index must be less than `itemCount`.                                                                                                                                                                                            | `nodes.ts:139-141`; `commands.rs:273-303`                                     |
|    5 | ScrollToEnd      | Node-level; VirtualList                                              | `null`                                                                                                                                  | Scrolls the mounted list to its end.                                                                                                                                                                                                                  | `wire.rs:197-207`; `commands.rs:273-321`                                      |
|    6 | SetTitle         | Root-only (`nodeId=1`)                                               | string                                                                                                                                  | Non-empty, at most 256 Unicode scalar values; sets native window title.                                                                                                                                                                               | `root-container.ts:239-265`; `wire.rs:154-160`; `commands.rs:32-41`           |
|    7 | ResizeWindow     | Root-only (`nodeId=1`)                                               | `[width,height]` u32 pair                                                                                                               | Each dimension is `1..16384` inclusive; resizes the client area.                                                                                                                                                                                      | `root-container.ts:328-339`; `wire.rs:175-184`; `commands.rs:123-140`         |
|    8 | ZoomWindow       | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                  | Toggles GPUI window zoom semantics.                                                                                                                                                                                                                   | `root-container.ts:341-347`; `commands.rs:142-149`                            |
|    9 | ToggleFullscreen | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                  | Toggles fullscreen semantics.                                                                                                                                                                                                                         | `root-container.ts:345-347`; `commands.rs:150-157`                            |
|   10 | OpenUrl          | Root-only (`nodeId=1`)                                               | string                                                                                                                                  | Non-empty HTTP/HTTPS URL, no whitespace, at most 2048 UTF-8 bytes; delegates to host URL opening.                                                                                                                                                     | `root-container.ts:349-358`; `wire.rs:111-118,161-166`; `commands.rs:159-165` |
|   11 | FocusNext        | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                  | Delegates to native tab-stop traversal.                                                                                                                                                                                                               | `root-container.ts:360-362`; `commands.rs:167-174`                            |
|   12 | FocusPrev        | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                  | Delegates to reverse native tab-stop traversal.                                                                                                                                                                                                       | `root-container.ts:364-365`; `commands.rs:175-182`                            |
|   13 | GetWindowSize    | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                  | Returns value tag `2` with logical `[width,height]`; no resize side effect.                                                                                                                                                                           | `root-container.ts:367-384`; `commands.rs:111-121`                            |
|   14 | GetFocus         | Node-level focus handle                                              | `null`                                                                                                                                  | Returns value tag `3` with the native focus boolean; missing/non-focusable handle fails.                                                                                                                                                              | `nodes.ts:130-137`; `commands.rs:186-195`                                     |
|   15 | ClipboardWrite   | Root-only (`nodeId=1`)                                               | string                                                                                                                                  | UTF-8 payload capped at 1 MiB; writes a string clipboard entry.                                                                                                                                                                                       | `root-container.ts:386-390`; `commands.rs:59-80`                              |
|   16 | ClipboardRead    | Root-only (`nodeId=1`)                                               | `null`                                                                                                                                  | Returns value tag `4`; fails when the host has no text or text exceeds 1 MiB.                                                                                                                                                                         | `root-container.ts:393-405`; `commands.rs:81-109`                             |
|   17 | OpenSurface      | Root-only (`nodeId=1`) on the requesting, already registered surface | `[title,[width,height]]` where title is at most 256 Unicode scalar values (empty allowed) and dimensions are `[0,0]` or each `1..16384` | Opens a native window and returns CommandResult value tag `1` (`[1,surfaceId]`); the host allocates the new positive u32 surface ID.                                                                                                                  | `root-container.ts:365-394`; `wire.rs:90-118,171-182`; `main.rs:193-227`      |
|   18 | FileDialogOpen   | Root-only (`nodeId=1`)                                               | `[title,[directories,multiple]]`, with both flags encoded as `0`/`1`                                                                    | Asynchronously opens the native picker with `files = !directories`, `directories`, and `multiple`. A selection completes with value tag `5`; cancellation is `success=true` with its optional value absent/null; platform failure is `success=false`. | `root-container.ts:402-425`; `wire.rs:192-200`; `commands.rs:21-88`           |
|   19 | FileDialogSave   | Root-only (`nodeId=1`)                                               | `defaultName` string (empty means no suggestion)                                                                                        | Asynchronously opens the native save picker. A selected path completes with value tag `4`; cancellation is `success=true` with its optional value absent/null; platform failure is `success=false`. GPUI's raw save API has no title/prompt option.   | `root-container.ts:428-444`; `wire.rs:201-208`; `commands.rs:89-132`          |
|   20 | ShowNotification | Root-only (`nodeId=1`)                                               | `[title,body]`, title UTF-8 ≤256 bytes and body UTF-8 ≤1024 bytes                                                                       | Fire-and-forget submission to `App::show_system_notification`; success means submitted to the platform, not delivered. Host generates its internal tag and sends no actions or response callback.                                                     | `root-container.ts:461-468`; `wire.rs:246-257`; `commands.rs:203-226`         |
|   21 | SetMenus         | Root-only (`nodeId=1`)                                               | `[[menuTitle,[item...]], ...]`; item `[0]` separator, `[1,actionName]`, or `[2,[submenuTitle,[item...]]]`                               | Replaces the application menu tree. Native action selection emits Event 17; dynamic enablement, accelerators, and keybinding registration are not part of this command.                                                                               | `root-container.ts:470-514`; `wire.rs:256-319,419-479`; `commands.rs:53-74`   |

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

SetMenus uses GPUI's typed `Action` bridge internally. The host wraps each
string action name in a host-owned action and routes the active window's
selection as Event 17. GPUI's native menu is process-wide, so the active
surface owns the event; menu updates replace the current tree. Platform
implementations differ: macOS installs an NSMenu, while Linux/Windows retain
owned menu data for their UI integrations and Web/test platforms are no-ops.

Every processed command produces a CommandResult event with the original
request ID, command, node ID, success flag, nullable error, and optional value
(`commands.rs:333-347`).

## 5. Encoding and validation conventions

### Numeric representation

TypeScript encodes with MessagePack options `{ sortKeys: false,
forceFloat32: true }` (`protocol.ts:3`). Integer values remain integer
encodings; non-integer numeric values use float32. Rust uses `f32` fields and
accepts the integer/float32 forms described by the wire enums, including mixed
forms for window dimensions (`wire.rs:635-651`). The exact byte versus semantic
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
9a030207032a0b01000e92ca44482000ca44162000
```

The decoded array is:

```text
[3, 2, 7, 3, 42, 11, 1, 0, 14, [800.5, 600.5]]
```

- It is Event version 3 on surface 7/epoch 3 at revision 42 and sequence 11.
- Node `1` and listener `0` identify the root window observer.
- Event type `14` is WindowResize; `0x92` is the untagged two-element payload,
  and each `0xca` is a float32. The semantic dimensions are `800.5 × 600.5`.

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
- The final string is the optional text-bearing form. The legacy row
  `ts-event-submit` (`ts_to_rust.hex:29`) carries the same event type with a
  null payload; both forms are intentionally accepted.

These rows demonstrate why the fixture test checks both decoded meaning and
producer bytes: the same positional contract is exercised by TypeScript and
Rust without requiring every numeric encoding choice to be byte-identical.
