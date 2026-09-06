# ADR-0012: Host-owned input models over upstream gaps

- **Status:** Accepted
- **Date:** 2026-08-29

## Context

Solid GPUI needs native text editing and bounded selection behavior, but the
pinned GPUI input-handler surface is shaped around text replacement, selection,
marked text, paste, geometry, and text length. It has no undo/redo history
primitive for this custom element path, and it does not provide a complete
selectable-Text model. The pinned macOS path likewise leaves undo and redo
disabled without an NSTextView/NSTextField. The evidence and existing limits are
recorded in `.scratch/text-input-completeness/spec.md` and
`.scratch/text-input-completeness/issues/02-undo.md`.

Putting a second editor model in JavaScript would introduce transport latency,
controlled-value races, and duplicate UTF-16/IME rules. The host already has
the native event timing, GPUI shaping, geometry, focus, and clipboard seams. The
model is implemented in `crates/solid-gpui/src/renderer/input.rs`,
`crates/solid-gpui/src/renderer/paint/text_input.rs`,
`crates/solid-gpui/src/renderer.rs`, and the existing TextInput event contract
in `docs/protocol.md` §3.

## Decision

- Keep transient text-editing state per TextInput in `NativeInputState`: native
  text, UTF-16 selection and direction, marked range, edit sequence, length
  limit, caret-follow scroll offset, and private edit history. The host uses
  the existing GPUI input-handler seam for shaping and platform delivery while
  retaining the state needed for synchronous local semantics.
- Let the host own bounded interactions that have no upstream primitive:
  click/multi-click character, word, and line selection; Cmd/Ctrl clipboard
  editing and Select All; UTF-16-safe navigation; multiline geometry; and
  caret/IME viewport following. These operations emit the existing Change,
  Selection, Focus, and Blur Native Events rather than adding a second wire
  state model.
- Implement Text selectable rendering as a host-owned element with per-node
  selection ranges, drag anchor/head state, visual-row selection quads, focus,
  and direct host clipboard copy. Selectable Text remains visual/native state;
  it does not add JavaScript selection state or selection events to the wire.
- Keep undo and redo private to each `NativeInputState`. The history is bounded
  to 100 pre-edit snapshots; adjacent compatible typing coalesces, explicit
  paste/cut and IME commit boundaries remain separate, and undo/redo restore
  the normal text and selection state so the renderer emits the ordinary
  Change/Selection events. History is cleared by the node's lifecycle rather
  than made an application-global editor service.
- Preserve the public boundary honestly: JavaScript owns Solid rendering and
  controlled values, while the host owns native transient state. UTF-16 remains
  the external selection unit and UTF-8 remains internal storage. Features with
  no stable host primitive, including secure/password display, remain explicit
  unsupported gaps rather than fake fallbacks.

## Alternatives rejected

- **Wait for GPUI to add every missing editor primitive:** rejected because the
  current application needs bounded behavior now and the available shaping,
  geometry, focus, and clipboard seams are sufficient for a narrow host model.
- **Mirror the native editor in JavaScript:** rejected because edits, IME
  marked ranges, and selection positions would cross an asynchronous boundary
  twice, creating stale controlled state and duplicate segmentation logic.
- **Add a wire event or JS selection store for every host interaction:**
  rejected because the existing TextInput events already carry authoritative
  text and UTF-16 selection, while selectable Text can remain host-owned with
  no SolidJS consumer contract for transient selection.
- **Reuse one application-global undo stack:** rejected because edits are
  per-node native state, cross-surface isolation matters, and a global stack
  would need ownership and focus rules that this renderer does not promise.
- **Pretend unsupported capabilities work through a no-op or visual substitute:**
  rejected because secure/password behavior and other upstream gaps must be
  observable as documented limits, not silently weakened security or editing
  semantics.

## Consequences

- The host renderer carries more local state and must reconcile it on text
  changes, controlled acknowledgements, focus changes, and unmount. That cost
  buys deterministic native timing without expanding the protocol.
- Undo/redo, multi-click, selection geometry, and caret following share one
  model, so max-length, marked-text, UTF-16, and event-order invariants stay in
  one place. A future editor feature should first identify its native owner and
  whether it fits this bounded model.
- Controlled TextInput remains a negotiated host/SolidJS state: the host can
  expose an edit immediately, while SolidJS acknowledges the edit through its
  existing props and sequence fields. The host does not promise browser DOM
  editing semantics.
- Host ownership is not a license to fill upstream gaps speculatively. A new
  native capability or a wire-visible selection contract requires fresh source
  evidence and a separate decision.
