# Protocol wire cutover

Status: resolved

## Scope

Protocol v3 now has one supported positional shape for each historical dual-form
host-property and event payload. This is a clean cutover: decoders reject old
arities and no default-filling compatibility path remains. The frame prefix and
message discriminators are unchanged.

## Current forms

The current TypeScript encoders and Rust wire constructors establish the forms
that remain supported:

- TextInput host properties: 13 slots, ending in `selectionReversed`.
- Image host properties: 4 slots, including `fallbackSource`.
- Drag host properties: 5 slots, including both callback booleans.
- TextInput events (Change, Selection, Focus, Blur): 8 slots, ending in
  `reversed`.
- WindowResize events: 3 numeric slots, `[width, height, scaleFactor]`.
- CommandResult events: 7 slots, with a nullable typed-value slot.
- Submit events: a string payload; the empty string represents an empty
  submission.

These forms are encoded by `crates/react-gpui/src/protocol/wire/node.rs`,
`crates/react-gpui/src/protocol/wire/event.rs`,
`packages/react-gpui/src/protocol.ts`, and the renderer submit path in
`packages/react-gpui/src/renderer/dispatch.ts`.

## Decisions and evidence

1. Historical TextInput, Image, Drag, WindowResize, CommandResult, and null
   Submit payload forms are removed from both decoders and validators. Existing
   current optional forms are not collapsed: open-surface, notification, and
   menu commands may still omit optional fields because current encoders emit
   those shapes intentionally (`packages/react-gpui/src/protocol.ts` and
   `crates/react-gpui/src/protocol/wire/command.rs`).
2. Submit is unified on `Event::submit(..., text)` in Rust. The input renderer
   passes native text, and TypeScript dispatch requires a string; no null-to-empty
   conversion remains. This preserves the authoritative empty-string semantic
   while removing the historical dual form.
3. The protocol golden generators remove null-submit rows and retain the
   string-submit row (`scripts/protocol-golden.ts` and
   `crates/react-gpui/examples/protocol_golden.rs`). Goldens are regenerated so
   both producers lock the same current contract.
4. Tests that asserted acceptance or defaulting of historical forms are deleted.
   Current round trips and malformed-payload checks remain in the Rust and
   TypeScript protocol suites; fuzz seeds remain valid current v3 payloads.
5. The accessibility wire field remains current data, but its comment no longer
   calls it a compatibility field: pinned GPUI has no public aria-disabled
   builder (`crates/react-gpui/src/renderer/paint/accessibility.rs`).

## Rejected alternatives

- Keeping old arities behind permissive deserializers: rejected because this
  would retain two protocol designs and hide mismatched host/renderer builds.
- Translating null Submit to `""`: rejected because null is the historical wire
  form; current submit semantics already define a string, including empty.
- Collapsing optional command tails: rejected because current TypeScript command
  encoders intentionally omit optional values such as actions or menus.
