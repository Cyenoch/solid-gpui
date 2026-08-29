# Window title follow-up

## Verdict

The original brief's premise was incorrect: runtime window-title control is already implemented as command 6 (`SetTitle`) across the protocol, TypeScript Root API, host dispatch, and golden/fuzz coverage. Do not add command 36, a second wire form, or any compatibility alias. This workstream is intentionally collapsed to documenting the actual command range and exercising the existing API from `notes.tsx`.

## Pinned GPUI evidence

The vendored Zed checkout is pinned at `6805d952f9f3d702f760aa11b1547df8a625fa16`.

- `references/zed/crates/gpui/src/window.rs:2582-2585` exposes `Window::set_window_title(&mut self, title: &str)`, forwarding to the platform and updating the accessibility window label.
- `references/zed/crates/gpui/src/platform.rs:842-846` exposes the platform `set_title(&mut self, title: &str)` seam.
- `references/zed/crates/gpui/src/platform/test/window.rs:302-304` implements the same seam in the pinned headless test window, so the title mutation is testable without a display.
- The repository's existing command path is `crates/react-gpui/src/protocol.rs:56` (`COMMAND_SET_TITLE = 6`), `packages/react-gpui/src/renderer.ts:147,236-238`, `packages/react-gpui/src/renderer/root-container.ts:375-401`, and `crates/react-gpui/src/renderer/commands.rs:424-433`.

No public pinned GPUI runtime window/dock `badge` or `subtitle` setter was found in the searched `gpui` Window/platform/App sources. The unrelated element-level `aria_description`/subtitle wording is not native window metadata. Badge/subtitle wiring is therefore skipped for this round.

## Count correction

The command-result allowlists include the VirtualList offset commands: `COMMAND_GET_SCROLL_OFFSET = 34` and `COMMAND_SCROLL_TO_OFFSET = 35` (`crates/react-gpui/src/protocol.rs:54-55`; `crates/react-gpui/src/protocol/wire/event.rs:61-100`; TypeScript validation at `packages/react-gpui/src/protocol.ts:845-893`). The protocol documentation's `1..33` wording was stale, so `docs/protocol.md` now says `1..35` in both the CommandResult row and command-directory introduction. The count-bearing `.scratch/surface-coverage/coverage.txt` row now records 35/35 and names commands 34–35. `docs/getting-started.md` and the root README have no stale numeric command-range wording to change. The user-owned `.scratch/release-productionization/**` notes were not touched.

## Example

`packages/react-gpui/examples/notes.tsx` now calls the existing `root.setTitle` from the effect that already tracks `path` and `dirty`, using the basename (or `Untitled note`) plus ` — ●` while dirty. This keeps the example's native title synchronized with the document state without changing protocol plumbing.
