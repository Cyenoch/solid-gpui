# External file-drop metadata

Status: resolved
Verdict: resolved-no-gap

## Question

Should `onExternalFileDrop(paths)` include per-file size or modification time?

## Evidence

- Pinned GPUI's `ExternalPaths` is only a path collection:
  `references/zed/crates/gpui/src/interactive.rs:683-692`.
- The inbound handler maps those paths to strings without metadata at
  `crates/react-gpui/src/renderer/paint/drag.rs:147-162`.
- The event remains the bounded `[3,[path,...]]` shape at
  `crates/react-gpui/src/protocol.rs:565-567` and
  `crates/react-gpui/src/protocol/wire/event.rs:209-215`.
- The only nearby metadata query is outbound `exportFiles` directory detection
  at `crates/react-gpui/src/renderer/paint/drag.rs:49-57`; it is not inbound
  file metadata.

## Decision and action

Keep the callback path-only. The count is already available as
`paths.length`. Applications that need size, mtime, or contents can compose
received host-local paths with app-owned file persistence/path operations.
Adding host filesystem I/O and a second payload shape to a drag notification is
not a useful minimal surface.
