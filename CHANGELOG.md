# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to Semantic Versioning. No version has been released
from this work tree.

## [Unreleased]

### Added

- Added a headless large-tree snapshot/first-frame performance guard covering 1,000, 5,000, and 20,000-node trees plus a one-operation patch on a 20,000-node tree; the guard records p50/p99 scaling and isolates host stages from GPUI frame/layout work.

- Extended the flagship gallery with mixed-style rich text runs, keyboard-accessible link activation feedback, and native selectable multi-run text.

### Fixed

- Patch reconciliation now limits input, selectable-text, VirtualList, and animation maintenance to the affected node workset (including relevant ancestors), and animation style history is allocated only for styled nodes.

## [0.2.0] - 2026-08-30

- Multi-surface lifecycle now retires every closed or explicitly unmounted
  surface ID. `SurfaceHost.createRoot` rejects same-ID recreation with typed
  `SurfaceIdReusedError`, while pending commands on a closed root reject with
  typed `SurfaceClosedError`; epochs remain generation checks, not a reuse
  escape hatch. Protocol wire frames are unchanged.

- Added node-scoped VirtualList logical-pixel scroll persistence: `getScrollOffset()` (command 34, value tag 10) reads the precise native offset, while `scrollToOffset(offset)` (command 35) restores it with native clamping. Invalid, negative, or non-finite offsets are rejected; restore after the first list layout.

- Added root-only asynchronous `readTextFile(path)` and `writeTextFile(path, content)` commands (25/26). They perform bounded UTF-8 filesystem I/O off the UI executor, return file text/bytes written, and reject invalid paths, directories, oversized data, invalid UTF-8, and native I/O failures.
- Added the `notes.tsx` persistence example with Open/Save, dirty tracking, multiline editing, and close confirmation.
- Fixed multiline editing so typing no longer continues below the visible fold; the host now follows the caret (and IME marked range) with bounded vertical and horizontal offsets.
- Embedded-Bun coverage now includes a bounded gallery/text-input/virtual-list/notes startup matrix through `EmbeddedBunAdapter`, plus a Fast Refresh lifecycle probe; `make embedded-bun` runs the host target alongside the adapter test.
- Added bounded root-scoped `setClipboardImage()` and `getClipboardImage()`
  commands (27/28). The protocol carries encoded PNG, JPEG, GIF, or SVG bytes
  through binary MessagePack with a cap below the 16 MiB frame limit and
  preserves format/bytes on reads. macOS and Windows use native clipboard image
  entries; X11 and Wayland reject the unsupported path explicitly rather than
  falling back to text.
- Added root-only `Root.loadFont(path)` (`COMMAND_LOAD_FONT=29`) for bounded
  asynchronous TTF/OTF registration through GPUI's runtime text-system seam.
  The promise returns the font metadata family for use with `fontFamily`; load
  before first layout/use because GPUI caches family resolution, and late calls
  do not invalidate cached fallback choices. Repeated registration is forwarded
  to the native backend. WOFF/WOFF2 are not supported.

- Added root-only window controls: `minimizeWindow()` (command 30),
  `getWindowBounds()` (31, value tag 8), `getWindowState()` (32, value tag 9),
  and `activateWindow()` (33). Bounds are finite logical/global coordinates;
  on macOS they use the screen-relative global top-left origin. The activation
  event remains the observable state channel. The pinned headless TestWindow
  leaves minimize unimplemented and reports inactive, so visible minimize and
  activation behavior requires a display-backed host. Saved bounds can restore
  size through centered `openSurface` creation, but exact position restoration
  is unsupported because pinned GPUI exposes no public position setter.
- Extended the runnable examples with a themed window-controls panel in
  `multi-surface.tsx`, a bundled Tuffy runtime-font flow in `text-input.tsx`,
  and a README recipe for encoded clipboard image bytes.

- Added mixed-style inline `Text` runs: a paragraph can combine raw strings
  with one level of nested `Text` children. Runs share the parent font size and
  line height while overriding color, weight, style, decoration, or family;
  nested `fontSize`/`lineHeight` and non-typography fields are rejected, and
  selectable rich text remains intentionally unsupported. The wire keeps the
  existing `Text`/`RawText` node kinds.

- Added a headless rich-text performance guard covering 50- and 200-run
  paragraphs. Unchanged redraws must reuse assembled content (zero rebuilds
  across the measured frames); style-only and full-text updates rebuild once.

- Added a headless TextInput typing-performance guard covering 100-, 1,000-, and
  10,000-character single-line documents plus wrapped multiline input. The guard
  records p50/p99 timings and content/run/shape call counts; attribution records
  pinned GPUI whole-line shaping as the dominant cost for long single-line text.

### Added

- `TestApp` now provides selection, external-file-drop, and layout event
  injectors plus `drainCommands()` for assertions. Root commands remain
  available through `app.root`; imperative `VirtualListHandle` operations use
  component refs, with their emitted commands assertable through the drain
  helper.

- Added keyboard-accessible nested `Text` link runs. Listener-bearing runs now
  receive native GPUI focus handles and tab stops, emit the existing focus/blur
  events, and synthesize the existing `onPress` event for unmodified Enter
  while focused. Space remains non-activating. A visible per-run focus
  affordance remains a documented boundary.
- Added the `rich-text.tsx` example, demonstrating nested typography, keyboard-accessible link runs, and native selection/copy across styled runs.

- Added keyboard focus affordances for nested `Text` link runs: focused runs
  paint one high-contrast pixel rule per wrapped line without shifting layout.
  Listener-bearing runs activate the existing `onPress` event on unmodified
  Enter; Space remains non-activating. Cursor feedback remains scoped to the
  parent text hitbox because the pinned GPUI text interaction surface does not
  expose per-range cursor regions.
- `View` and `Pressable` now support bounded native `tooltip` text through the
  pinned GPUI tooltip path, including compatible optional tuple tails and
  tooltip-only updates.
- Opt-in `onPointerMove` support for `View` and `Pressable` streams clamped
  logical window coordinates and ordered modifier names through the existing
  pointer event family. Nodes without a handler register no native move
  listener; hover edge notifications and drag-over delivery remain separate.
- Native per-window close policy now supports asynchronous confirmation:
  `require-confirmation` emits a deduplicated `EVENT_CLOSE_REQUESTED`, and
  JavaScript resolves it with `resolveCloseRequest(requestId, allow)`.
- Context menus remain application-composed in-window overlays using
  right-pointer events, positioned overlay views, and
  `onPointerDownOutside`; no native context-menu API is added.
- Pointer down/up notifications now carry required finite non-negative logical
  window-pixel `x`/`y` coordinates in the v3 payload. The host clamps native
  positions to viewport bounds before emitting them, enabling cursor-anchored
  in-window context-menu overlays; hover/move coordinate streaming remains
  intentionally unsupported.
- Shared appearance-aware theme tokens now drive all visual React GPUI
  examples, including dark surfaces and interaction-state colors; README and
  getting-started guidance document the application-owned token-module
  pattern. This is an examples/docs change with no library API change.

- Protocol tap reporting now includes frames-by-kind, overall and patch byte
  rates, one-second frame/byte timeline buckets, byte-size histograms, and
  malformed-input counters over the existing payload-free JSONL stream.
  Host startup diagnostics and `--version` now identify `protocol=v3`; queue
  backpressure and per-commit timing remain explicitly outside the tap.

- Process-mode host coverage now selects GPUI's Linux Wayland/X11 backends and
  adds locked Ubuntu and Windows build checks; display-backed runtime validation
  and a non-macOS embedded-Bun port remain separate follow-up work.

- Added focused `focus-flow.tsx`, `dropdown.tsx`, `drag-reorder.tsx`, and
  `multi-surface.tsx` examples, a progressive getting-started walkthrough,
  core composition recipes, and a consumer headless-testing guide.
- `@react-gpui/dev` now includes a `renderTestApp` facade with
  accessibility-label/text locators and behavior-level interactions over the
  real headless dispatch path; host-owned geometry, painted output, and native
  wheel behavior remain display-backed test concerns.

- System notifications use `SHOW_NOTIFICATION=20` with `[title, body]` and a
  constrained fire-and-forget API; delivery varies by platform, host identity
  is a prerequisite, and tags, actions, and response callbacks are not exposed.
- Static menus use nested `SET_MENUS=21` wire payloads and
  `EVENT_ACTION=17` string action events, with `RootOptions.onAction`; dynamic
  enablement and keybinding registration remain intentionally unsupported.
- Generic focusable `View` and `Pressable` nodes emit native `EVENT_FOCUS` and
  `EVENT_BLUR` notifications and expose `onFocus`/`onBlur` callbacks.
- Overlay `View` nodes emit `EVENT_POINTER_DOWN_OUTSIDE=22` on capture-phase
  mouse-down outside both the rendered overlay and direct anchor subtree;
  `onPointerDownOutside` can close the overlay without an invisible scrim.
- TextInput double-click selects the UAX #29 word under the caret and
  triple-click selects the clicked logical line; multiline selection stays
  within the clicked line, and word/line selections extend by their
  granularity while dragging.
- `TextInput` now handles host-owned Cmd/Ctrl-C/X/V clipboard editing and
  Cmd/Ctrl-A selection, Option/Alt word-wise Left/Right navigation, and
  bounded Cmd/Ctrl-Z undo with Shift-Cmd-Z/Ctrl-Y redo. Typing coalesces by
  caret continuity, paste/cut/selection edits form boundaries, and IME marked
  text commits as one history entry; the wire remains unchanged. Secure/
  password display remains unsupported.
- Crash diagnostics install a host panic hook with
  `REACT_GPUI_CRASH_DIR` crash logs; TypeScript transport termination errors
  carry host exit codes and the last 50 stderr lines. README Troubleshooting
  documents reproduction and an optional APM sample.
- Transport termination now exposes a typed cause union and fails fast on
  malformed host event frames while rejecting all pending commands.
- Host panic diagnostics now print a stable crash-report path on stderr;
  TypeScript termination details retain `crashReportPath`, exit code, and the
  bounded stderr tail, and package Troubleshooting documents fresh-process
  restart.
- Protocol v3 decoders now reject mismatched renderer/host versions with a
  typed, version-bearing diagnostic; `SurfaceHost` and `RootContainer` retain
  it as a fail-fast protocol termination, and compatibility remains lockstep.
- Pending-command lifecycle regression coverage locks seven paths: surface
  close, transport-termination fan-out, unmount, every command family on a
  closed root, silent event drops, and host disposal; dangling Promises are
  treated as a zero-tolerance failure.
- The release-readiness evidence package records the fresh five-gate matrix
  (72.97 seconds), the 0.2.0 version-cut rationale, human decision items, and
  five documentation-drift corrections at
  `.scratch/release-productionization/release-readiness.md`.
- Absolute positioning extends the style tuple from 33 to 38 slots with
  `position: "relative" | "absolute"` and `left`/`top`/`right`/`bottom`
  insets. Negative finite offsets preserve GPUI semantics; relative means
  post-layout displacement and absolute means anchoring, with combined
  stretching delegated to documented Taffy behavior. There is no `zIndex`;
  paint order remains subtree order. `examples/gallery.tsx` includes overlay
  coverage.
- System appearance observation uses `EVENT_WINDOW_APPEARANCE=18` with the
  `"light" | "dark"` string enum. Vibrant variants fold to these semantic
  values in line with Zed; the existing coalesced window observer always emits
  an initial frame. `RootOptions.onAppearance` and the explicit
  `createAppearanceStore`/`useAppearance` bridge are provided; application code
  owns theme-color policy.
- Layout measurement emits `EVENT_LAYOUT=19` with `[x,y,width,height]` through
  the `MeasuredElement` prepaint seam; `on_next_frame` batches delivery,
  defers the initial frame, and applies exact deduplication/invalidation.
  `View`, `Pressable`, `Text`, and `Image` expose `onLayout`; display-backed
  numeric verification remains a release validation item.
- Dependency governance refreshed `@msgpack/msgpack` to 3.1.3 and Cargo
  `core-foundation` to 0.10.1 plus `rand` to 0.8.8. `@types/react` and
  `bun-types` upgrade attempts were rolled back after real TypeScript failures;
  `cargo-audit` is unavailable so no vulnerability conclusion is claimed.
  The GPUI pin, toolchain, and Rust 1.98 decision remain human-owned.
  The 219-line audit is at `.scratch/release-productionization/dependency-audit.md`.
- Runtime termination handling distinguishes explicit shutdown from unexpected
  EOF, non-zero process exit, protocol failure, and retained failed status; the
  host exposes these outcomes as observable CLI failures.
- Process-runtime native event delivery now uses a bounded, ordered writer
  queue with queued-byte and queued-frame limits, retained writer failures, and
  shutdown/kill/wait closure coverage.
- Rust, Bun, package-build, tarball-consumer, embedded-Bun, and host release
  checks are available through the Makefile and fixed-SHA GitHub Actions
  workflows.
- Both Bun packages now build ESM JavaScript and declaration artifacts under
  `dist/`, expose only built package exports, carry Apache-2.0 license text,
  and pass an external tarball consumer runtime/type smoke test.
- The host CLI has an early `--version` path and bounded host-option parsing
  that preserves renderer arguments after the renderer executable or `--`.
- The host release candidate creates a version-and-target named macOS ARM
  archive containing the process-runtime host, README, LICENSE, and
  SHA256SUMS. The release check proves identical SHA-256 values for two
  archives made from one staged payload.
- Renderer `StdioTransport` observes input `end`/`close`/`error` and output
  `close`/`error`, enters an idempotent `TransportTerminatedError` termination
  state, and lets `createRoot` notify via `onTransportTermination`; examples use
  `createProcessTerminationHandler` to exit non-zero.
- Native keyboard events are supported for focusable `View` nodes: the node
  tuple carries `focusable` as its tenth field, the key payload uses tag `5`,
  and the public API exposes `focusable`/`onKeyDown` with the
  `examples/keyboard.tsx` entry.
- Layout and text style fields now share the validated Style wire tuple,
  including positive `fontSize`, layout enums, colors, and `fontWeight` up to
  `BLACK=900`; `fontFamily` is supported for registered runtime families and
  falls back through GPUI when the requested family is unavailable.
- Protocol decoding has deterministic seeded fuzz coverage with 1,039 Rust
  cases and 1,034 TypeScript cases; no panic was found.
- Pointer and hover events plus `ViewHandle` focus/blur are supported: the node
  tuple has ten fields with field 10 as `focusable` (there is no node field
  11); pointer payloads use tag `6` with button/action codes, and hover uses
  the explicit null-payload edge semantics.
- The host now supports `REACT_GPUI_LOG` `off`/`error`/`info`/`debug`
  diagnostics, and `examples/gallery.tsx` demonstrates the composed layout,
  text-style, input, list, press, and transition capabilities.
- Scroll-wheel native events are supported with `EVENT_SCROLL=12`, payload tag
  `7`, `pixels`/`lines` deltas, and View-only `onScroll` dispatch.
- Performance-budget smoke coverage exercises 10,000 nodes and 1,000 Patch
  operations across six phases with a 2-second budget each; debug measured
  76.1ms at worst (26x headroom) and release measured 13.9ms. This is a
  regression guardrail, not a benchmark.
- `release-prep` synchronizes the single Cargo workspace version into both Bun
  packages, refreshes and freezes locks, and provides a manual candidate-pack
  workflow without publishing.
- Overflow style now uses the 18-slot Style wire tuple with codes `1=visible`,
  `2=hidden`, and `3=scroll`; scrolling containers expose overflow semantics
  without changing VirtualList behavior.
- Text truncation now uses the 20-slot Style wire: `lineClamp` accepts `1..100`
  and implies `overflow=hidden` only when overflow is unset, while
  `textOverflow` supports `clip` (clears ellipsis) and `ellipsis` under the
  documented priority rules.
- Error-boundary handling and `Root.setTitle` are now explicit: the
  `COMMAND_SET_TITLE` command is root-only and validates titles at no more than
  256 Unicode code points.
- The core crate is modularized into six `renderer.rs` submodules plus
  `protocol/wire.rs` while preserving the public API; the full test suite
  verifies unchanged behavior.
- `TextInput` supports `onSubmitEditing(value)` through `EVENT_SUBMIT=13`;
  new hosts carry authoritative native text (including empty text), while null
  payloads remain decode-compatible. Multiline inputs do not submit, and
  `maxLength` uses native UTF-16 truncation with symmetric
  controlled/uncontrolled behavior.
- Surface-level commands now support root-only `resize`, `zoom`,
  `toggleFullscreen`, and `openUrl` wire kinds `7`-`10`; `center` and
  `revealPath` remain intentionally unsupported.
- Module-boundary guard tests lock the six-way `renderer`/`protocol` split with
  text-level assertions against public leakage and dependency inversion,
  including a regression example.
- Keyboard focus cycling now exposes root-only `FOCUS_NEXT=11` and
  `FOCUS_PREV=12` commands over GPUI's ordered tab-stop graph; `isFocused`
  remains intentionally unexposed.
- The host candidate now has a user-view end-to-end rehearsal: it extracts the
  binary, runs an external renderer entry, verifies a Snapshot frame and info
  diagnostic, enforces the expected timeout `124`, and runs in the candidate
  workflow.
- `Image` is supported as `KIND_IMAGE=7` with payload tag `3`
  `[3,source,objectFit]`; paths are resolved as documented, async load failures
  remain silent blank images, and Image nodes reject children.
- The TypeScript renderer is split into six internal modules
  (`types`/`props`/`nodes`/`dispatch`/`root-container`/`host-config`) behind a
  179-line facade; the public API is unchanged and the full tests remain green.
- Embedded candidate rehearsal is a non-publishing exercise on the `macos-26`
  workflow; `commits>=1` is the in-process commit proof, with release-matrix
  boundaries tracked in issues 03 and 06.
- Style now spans 20 to 33 wire slots with four-way margins,
  `fontStyle`/`textDecoration`, `lineHeight`, min/max constraints, and
  `flexShrink`/`alignSelf`; `letterSpacing`, `boxShadow`, and `cursor` remain
  intentionally unsupported. Cross-language protocol golden vectors cover
  fixtures in both directions with byte-stable encoding, semantic equivalence,
  and integer/float32 acceptance for numeric slots.
- Window observation now emits `EVENT_WINDOW_RESIZE=14` and
  `EVENT_WINDOW_ACTIVATION=15` with initial first-frame values and batched
  duplicate-free logical-pixel updates; `scaleFactor` events remain
  intentionally unsupported.
- CommandResult can carry an optional fifth tagged value: root-only
  `GET_WINDOW_SIZE=13`/`GET_FOCUS=14` support `Root.getWindowSize()` and
  `ViewHandle.isFocused()`, while old no-value acknowledgements remain
  compatible.
- Protocol tap tracing is opt-in through `REACT_GPUI_TAP`, emitted per process
  path with a 64 MiB bound, non-fatal reporting, and a report script; measured
  tracing overhead is documented and payload bytes are never persisted.
- Clipboard text commands now use `CLIPBOARD_WRITE=15` and
  `CLIPBOARD_READ=16` with CommandValue tag `4` (`[4,string]`), enforce a
  1 MiB UTF-8 limit, report read failures as `success=false` with an explicit
  error, and expose `Root.setClipboardText()`/`getClipboardText()`;
  Quartz clipboard behavior remains a display-backed validation item.
- `examples/todo.tsx` now combines resize bridging, VirtualList, Image, and
  focus cycling in a practical todo flow, with three component tests.
- Keyboard accessibility now covers `Pressable` `focusable`/`onKeyDown`/
  `disabled`, Enter/Space activation through GPUI's existing `on_click` path
  without a new wire event, disabled removal from interaction and tab stops,
  and `TextInput` `onKeyDown` with IME priority preserved.
- `VirtualList` supports `emptyState` without emitting native rows for empty
  data and restores its initial range when data returns; explicit
  `createWindowSizeStore`/`useWindowSize` bridge window observations without
  implicit global state. `examples/todo.tsx` is now fully keyboard accessible.
- ADR 0003-0007 and `CONTEXT.md` now record the load-bearing domain vocabulary
  and decisions, including rejected alternatives: Surface Command, Tab-stop
  Graph, Golden Vector, Protocol Tap, and categorized Native Events.
- Image `onError` is intentionally unsupported because the loader is
  synchronous and exposes no notification hook.
- Width/height transitions use `TransitionProperty` masks `4`/`8`, preserve
  retarget/generation semantics, and document per-frame layout cost;
  transform scale/translate and borderRadius transitions remain unsupported
  because the GPUI transformation path is SVG-only and borderRadius is not
  animated.
- `docs/protocol.md` is now the authoritative 485-line protocol reference with
  source line citations, event/command indexes, evolution rules, and fixture
  walkthroughs; README keeps only the overview to avoid dual-source drift.
- Multi-window surfaces now use root-only `COMMAND_OPEN_SURFACE=17` with
  `[title,[width,height]]` and tag-1 surface-id acknowledgements, plus
  `EVENT_SURFACE_CLOSED=16`; `SurfaceHost` demultiplexes by surface, while the
  host `SurfaceRegistry` centrally routes and rejects unknown surfaces,
  terminates when the last window closes, and preserves one emitter sequence.
  Quartz multi-window behavior remains a display-backed validation item.
- `@react-gpui/dev` now provides headless `render()` test helpers for frames,
  commits, nodes, press/key/input/submit/visibleRange/commandResult,
  dispatchFrame, and constant guard tests; the pack smoke consumes the helpers
  and the README documents the headless workflow.
- File-dialog commands expose `FILE_DIALOG_OPEN=18` with
  `[title,[directories,multiple]]` and `FILE_DIALOG_SAVE=19` with one
  `defaultName` string; `files=!directories` are complementary, cancellation
  is a successful missing-value response mapped to Promise `null`, platform
  errors reject, and tag-5 results require non-empty paths. Save-dialog titles
  remain unsupported where the platform cannot apply them; the TestPlatform
  stub is headless-testable, while real dialogs remain display-backed.
- The work tree is organized into eleven reachable thematic checkpoints
  (`65c71cc` through `a1c8d95`), including the pre-existing StdioTransport
  backpressure tests; superseded checkpoint `ca9bd23` was replaced by
  reachable commit `9fab40d`.

- Drag events expose `EVENT_DRAG=20` with `[1,type]`, `[2,type]`, and
  `[3,paths]` payloads; View/Pressable gain draggable, onDragOver notification,
  typed onDrop, and onExternalFileDrop. Drag data stays on the JavaScript side
  keyed by type, the host can_drop simplification and fixed 24x24 preview are
  documented, and drag visuals remain display-backed.
- `docs/getting-started.md` adds a 365-line consumer guide covering setup, the
  architecture model, quick references, nine cookbooks, debugging pointers,
  and current boundaries; all snippets were typechecked.
- Accessibility applications now connect `Image`, `VirtualList`, and `RawText`
  to AX; explicit roles map to platform semantics, while generic and unknown
  elements accurately remain without node semantics. `checked` maps to
  `toggled`, and patch role/checked validation is complete. No upstream public
  builder exposes `disabled`, so it remains explicitly unsupported; AX-tree
  verification remains display-backed.
- Accessibility now carries optional `accessibilityExpanded` and positive
  heading-only `accessibilityLevel` tail fields through the wire to pinned
  GPUI's `aria_expanded`/`aria_level` builders; labels and descriptions remain
  independently applied. AccessKit 0.24.1's `Live` property has no pinned
  GPUI public builder/write path, so live-region announcements remain an
  explicit upstream gap; headless AX-tree inspection remains unavailable.
- Public API snapshots initially locked core's 94 and dev's 18 exports by name
  and kind. Subsequent cursor and text-alignment additions now leave the
  checked-in fixtures at core 96 and dev 18; the generator runs against built
  artifacts, and pack-smoke coverage remains extended.
- Cursor styling exposes all 19 GPUI-aligned `CursorStyle` enum values in an
  appended 39th style slot. Windows variants without an exact mapping fall back
  to the default cursor, and cursor changes are a headless no-op. API snapshot
  generation now runs against the built exports for the first time, updating
  the checked-in fixtures.
- Repository release hygiene removes the accidental `build_script_build`
  artifact and ignores it plus local `.scratch` evidence; package keywords and
  the Bun-crate description are complete, core/dev pack dry-runs verify 18/8
  files, and Cargo metadata is complete except for the repository omitted
  truthfully because no git remote is configured. `CONTRIBUTING.md` and
  package README first-screen license badges plus getting-started/protocol/
  contributing links are now present.
- The capability-gap review records multiline text input, text selection,
  `pointerEvents`, and rotation as backlog items with native-source evidence;
  current documentation makes no unsupported capability claims.
- Performance budgets now use post-30-round baselines: patch application is
  79.48 ms versus the historical 76 ms (`~1.05x`, with no regression), with
  segmented timings; the 39-slot full/null style comparison is 85.9/13.8 ms,
  demonstrating the null fast path; layout batching is 47.7 ms. TypeScript
  surface routing measures 5.8/9.2 ms for surfaces 2/8, while press, scroll,
  and drag dispatch hot paths retain explicit budgets. Workloads are measured
  at ×10 and every baseline carries a date comment.
- The Rust test suite moves the 1,922-line `tests.rs` into five domain modules
  while preserving the 64-test count at that refactor checkpoint. The 1,991-line
  wire implementation is split by message family into `mod.rs`,
  `snapshot_patch.rs`, `node.rs`, `command.rs`, and `event.rs`;
  module-boundary guardrails stay synchronized, and the pure move changes no
  public surface.
- `flexDirection` now accepts all four GPUI/Taffy values, including
  `row-reverse` and `column-reverse`, and mirrors physical layout as a
  layout-only operation. RTL/bidi text direction and caret behavior remain
  unsupported and are documented as such.
- `textAlign` supports `left`, `center`, and `right` at style slot 39, locking
  the 40-slot style tuple and its cross-language validation.
- ADR-0008 records the error philosophy across the four seams: consumer
  Error-Boundary ownership for render failures, fail-fast protocol validation,
  local blank-image degradation, and host-fatal GPUI paint panics. Resync,
  `catch_unwind`, and swallowed errors are rejected; getting-started now has the
  error-boundary table and explicitly documents shared multi-surface termination.
- TextInput placeholders render as gray, visual-only native text when the
  native value is empty, following GPUI example semantics without entering
  `value`, selection, or UTF-16 state. Selection direction now flows end to end:
  `NativeInputState` tracks it, `UTF16Selection::reversed` returns the real
  direction, 13-slot host tuples carry it, legacy 8-slot events decode with
  `reversed: false`, and TypeScript selection events expose `reversed`.
  `setSelection` retains ordered command semantics; IME candidate and click
  positioning remain documented approximate placeholders pending exact geometry.
- v0.2.0 cut preparation validates a safe `release-prep 0.2.0` dry-run, counts
  71 Unreleased entries (64/3/4), verifies archive naming follows the Cargo
  version, and records a local ignored `.scratch` checklist with five human
  decisions and a seven-step mechanical sequence.
- Single-line TextInput geometry now uses a custom `TextInputElement` that
  retains its `ShapedLine`, maps UTF-16 offsets through UTF-8 positions to x
  coordinates, caches layout per node, and invalidates the cache on edits and
  resets. Placeholder and multiline paths honestly fall back to element bounds;
  single-line IME candidate-window positioning is now precise, while multiline
  and display-backed verification remain documented boundaries.
- The pre-release consistency sweep mechanically aligns 20 events, 21 commands,
  40 style slots, and 13/8 TextInput host/event slots; finds zero project TODO
  markers; classifies all 19 production `expect` calls within ADR-0008 domains;
  corrects test-count/API-snapshot drift; and verifies every new
  `ALLOWED_PROPS` capability. It identified 76 historical `wire.rs` citations
  as P2 documentation debt, cleared by the follow-up citation sweep below.
- TextInput mouse interaction now supports single-line click-to-caret and drag
  selection with anchor/head/reversed orientation and selection highlighting.
  Shift+arrow, Home/End, and Up/Down navigation respect UTF-16 boundaries
  without splitting surrogate pairs; platform `Arrow*` and `Home`/`End` names
  are accepted. The wire contract is unchanged. The caret remains always
  visible for accessibility; double/triple-click and word-boundary semantics
  remain explicit backlog items.
- Protocol citation debt is cleared: 75 historical `wire.rs` references now map
  to the five split wire modules with zero bare `wire.rs` paths. The six-example
  index is aligned, and examples cover `onLayout`, drag handlers, `reversed`,
  `setSelection`, and `scrollToIndex`/`scrollToEnd`.
- Headless end-to-end command coverage now drives all 18 commands through a
  real `TestAppContext`/surface registry: focus/getFocus assertions, clipboard
  read/write, static menu to `EVENT_ACTION`, notification capture, and
  multi-surface registration. Eleven event paths carry A-level assertions;
  Zoom, file dialogs, and input-driving remain explicitly graded lower because
  they require platform/display semantics. The incremental command roundtrip
  suite adds **3.5 s** of coverage.
- Dialog command round trips now cover multi/single-file and directory opens
  through value tag 5, save through tag 4, cancellation as `success=true` with
  a missing value, malformed-frame routing rejection, non-UTF-8 path host error
  receipts, and safe WeakEntity callback dropping after surface close. These
  close the remaining audited paths; TestPlatform's lack of a provider-error
  injection surface is recorded as a matrix limitation.
- Headless long-run stability invariants now exercise 50,000 cycles across 10
  surfaces and 100,020 frames in 0.78 seconds: listener lifetimes stay
  constant, every command Promise settles, surface IDs remain monotonic, frame
  bounds and error counters stay clean, and press/scroll callbacks reach all
  iterations. A 60-second process soak remains an optional manual run and was
  not performed.
- TypeScript capability coverage now exercises all 21 commands at A level,
  including the tag-5 path helper and malformed-value rejection semantics;
  command receipts, surface routing, clipboard, menus/actions, dialogs, and
  window operations are asserted through the headless harness.
- The final documentation freeze check found zero drift after both golden/API
  generators, zero broken local links across 45 scanned links, and synchronized
  the 272-line readiness report with **81** Unreleased entries. External badge
  URLs were not network-HEAD checked; no publication decision is implied.
- Static menu action items now accept `[1,name,[disabled,checked]]` with legacy
  `[1,name]` compatibility; GPUI MenuItem flags map disabled and checked,
  and each static `setMenus` call fully re-sends the menu definition.
  Disabled actions do not activate natively; TestPlatform has no native menu
  structure capture, which remains an explicit test boundary.
- The 60-second process soak smoke recorded host RSS first/last/peak
  **77,504/77,584/77,584 KiB** (`+80 KiB`, `+0.10%`, slope **80 KiB/min**),
  exact tap frame matching **5,626 = 5,626**, and clean SIGTERM exit. It sent
  511 renderer command frames with zero command results because headless GPUI
  command processing is display-bound; this limitation is reported, not hidden.
  The run is leak-smoke evidence, not multi-hour soak proof.
- The DX timing audit records warm `make ci` at 9.57s, Rust format/check at
  0.12/2.19s, clean Rust check at 32.31s versus 0.47s after a one-file touch,
  cold Bun installs at 0.01/0.07s, Bun tests at 1.15s core and 0.40s dev,
  and release-host startup to first Snapshot at 45ms. A clean-cache
  `make embedded-bun` takes 137.16s versus 0.82s warm because of the pinned
  Bun/GPUI graph; this is inherent cold-build cost, not an optimization target.
  Shared Makefile install/build outputs make parallelization riskier than its
  small warm-CI benefit, so no build-time optimization was made.
- VirtualList now renders through GPUI's native variable-height list: per-node ListState with real measured committed rows and estimated placeholders outside the committed range, ListState-based scroll commands, and a zero-change wire; estimatedItemSize becomes an initial hint, the JS-side measurement path was surveyed and rejected as structurally infeasible, and placeholder flicker remains display-backed.
- VirtualList completeness now documents and tests its range-based
  `onEndReached` (overscanned/next-frame ranges, once per reach), index-based
  scroll restoration with clamping across data changes, command bounds,
  estimate convergence, and row eviction/remount state semantics. Native
  wheel-boundary behavior remains covered while empty-state transitions and
  placeholder flicker stay explicitly display-backed.
- BoxShadow and fontFamily complete the 42-slot Style wire: `boxShadow`
  supports tagged single (`tag=1`) and double (`tag=2`) shadow forms with
  finite offsets, non-negative blur/spread, RGBA colors, and inset flags;
  `fontFamily` occupies one slot and GPUI's `resolve_font` safely falls back
  through the configured stack when the requested family is unavailable. README
  stale unsupported-style wording is corrected, and
  `.scratch/release-productionization/upstream-dependencies.md` records the
  three categories of true upstream gaps, project re-review limits, and
  test-platform limits.
- Image fallback adds optional `fallbackSource` to the tag-3 four-slot host
  tuple with legacy decoding; GPUI `with_loading` and `with_fallback` cover
  both loading and failure states, while `onError` remains tracked as a true
  upstream gap.
- `scaleFactor` extends window resize events to a three-number payload while
  legacy two-number payloads normalize to `1`; resize callbacks accept a third
  argument, `WindowSize` stores expose it, and the observer reports scale-only
  changes.
- The final pre-cut audit passes all seven dimensions: 20 sampled documentation
  claims reverse-verified with zero missing implementations, all 75 commits use
  Conventional Commit subjects, four documentation drifts were fixed, and
  `.scratch/release-productionization/final-pre-cut-audit.md` records 191 lines
  of evidence.
- Multiline TextInput geometry now uses dual single-line `ShapedLine` and
  multiline `shape_text`/`WrappedLine` layout caches; cross-line selections
  compute union bounds, UTF-16 offsets map to line/local positions, and
  explicit line-height growth follows wrapped line count. IME candidate
  placement remains display-backed. Keybinding registration is tracked as a
  feasible-bounded follow-up in
  `.scratch/release-productionization/upstream-dependencies.md`.
- The consumer troubleshooting guide adds a 459-line symptom directory with
  8 entries and symptom → diagnosis → repair sections covering Snapshot/
  transport, validation, events, command lifecycle, IME, tap performance,
  crash reports, and compatibility; README Debugging points to it while
  retaining the environment quick reference.
- Keybinding registration exposes root-only `COMMAND_SET_KEYBINDINGS=22` with
  `Root.setKeybindings` full-replacement semantics, 64-entry/64-byte/64-character
  limits, `Keystroke::parse` prevalidation with atomic indexed rejection,
  per-surface retained sets rebuilt into a deterministic process-global union,
  action hits reusing the `EVENT_ACTION` active-window route, and
  context-conditional bindings documented as unsupported.
- Outbound file drag exposes `Draggable.exportFiles` (1..8 validated paths)
  through the third Drag wire slot with legacy two-slot decoding; the native
  GPUI Files resolver lazily probes directory metadata, text payloads and
  delivery-completion callbacks are intentionally absent, macOS and Wayland
  perform the gesture while X11 and Windows decline it, and the pinned
  TestWindow capture limitation remains a documented display/upstream seam
  boundary.
- Selectable Text feasibility re-review is archived with TextRun background
  support, public TextLayout geometry, and the Zed Markdown `RenderedText`
  precedent; it is reclassified from a hard upstream gap to a
  feasible-bounded host-owned design with anchor/head state, per-row selection
  quads, and host clipboard copy tracked for a future pass, and this round does
  not implement it.
- Read-only Text selection ships through `TextProps.selectable` with a
  per-node tail wire flag (Text-only validated, `UPDATE_SELECTABLE=64`
  mask): a host-owned `SelectableTextElement` composes GPUI shaping with
  per-visual-row selection quads, an IBeam cursor, focus tracking, drag
  anchor/head ranges clamped on text change, unmount cleanup, and
  host-side Cmd/Ctrl-C clipboard writes; selection is visual-only with
  zero new events or JS state, verified by display-backed
  drag/highlight/copy tests plus headless clamp/geometry coverage.
- Surface creation options extend `COMMAND_OPEN_SURFACE=17` with an optional
  third tuple `[kind, resizable, minWidth, minHeight]` while legacy
  `[title, [width, height]]` frames stay byte-identical: kind maps
  normal|floating|dialog through host WindowOptions (macOS Floating level and
  Dialog sheet, Windows dialog modal with Floating not topmost, X11/Wayland
  transient/modal), resizable and window minimum sizes are creation-time only,
  and maxSize, a generic window level, runtime setters, and centering remain
  unexposed because the pinned GPUI surface carries no such fields; A-level
  command roundtrips assert the real mapping.
- The upstream dependency review now classifies RTL text as two separate gaps
  — an explicit base-direction API and bidi-aware hit/caret geometry — after
  pinned-source evidence showed cosmic-text already performs UAX #9 reordering
  and HarfRust directional shaping for rendering, so pure RTL rendering is no
  longer listed as an upstream gap.

### Fixed
- Targeted rich-text cache invalidation now preserves assembled runs for
  unrelated patches while rebuilding changed paragraphs and final content
  ancestors, including same-patch create chains and deletes.

- Fixed nested interactive `Text` cursor scope: GPUI's native
  `InteractiveText` decision now controls the pointing hand for listener-bearing
  runs, while the paragraph's node-level cursor style is ignored so sibling
  text does not inherit it.

- Keyboard focus traversal previously failed for all React nodes because GPUI
  focusable elements were not registered as native tab stops; traversal now
  reaches React View, Pressable, TextInput, and selectable Text nodes through
  explicit native tab stops. GPUI tab-group/insertion order, wrapping,
  disabled-control skipping, and per-window isolation are preserved.
- Focused-node unmount now emits JavaScript blur synchronously and restores
  focus to the live ancestor/restore target or the first eligible stop.
- `UPDATE_FOCUSABLE` patches are now accepted for `Pressable` nodes.

- Host pointer events normalize platform `click_count=0` mouse-up values to the
  wire contract's minimum `clickCount=1`, preventing valid native pointer
  notifications from being rejected by the TypeScript decoder.

- Transition retargets now preserve the previous transition metadata when a
  changed style omits `transition`, so supported opacity/background removals
  animate back through intermediate values. Retargets still sample the current
  presentation value, restart the declared delay, and emit one completion per
  generation; numeric width/height-to-auto and transform/border-radius
  transitions remain unsupported with the documented GPUI evidence.

- CommandResult acknowledgement validation now accepts command kinds `6`-`12`,
  preserving Rust-layer replies for title and other surface commands.
- Integer-dimension resize frames were ambiguous with the untagged Animation
  `[tag,generation]` payload and could be rejected; event-type-directed
  deserialization now selects `WindowResizeWire` and accepts integer/float
  dimensions.
- EventWire visitors now propagate decode errors for type-mismatched payloads
  directly; the previous sentinel-based malformed payload relied on
  cross-variant rejection by coincidence.
- `EventWire` CommandResult validation now accepts `COMMAND_CLIPBOARD_WRITE`;
  clipboard command acknowledgements were previously dropped at the protocol
  layer. This extends the earlier kind `6`-`12` CommandResult validation fix.
- Native macOS glyph rendering now enables `gpui-platform`'s `font-kit` feature.
  A real native raster probe requires nonzero bounds and covered bitmap pixels,
  catching the `NoopTextSystem` false-green path.
- Nested non-root `flexDirection` and `gap` styles now activate GPUI flex
  layout. The compact gallery page reaches lower content through native wheel
  scrolling without horizontal overflow at 800×600 and 916×588.
- Pointer dispatch preserves the outer `EVENT_POINTER` kind while encoding
  down/up actions; `TextInput.onBlur` is exposed, and overlay dropdown
  placement uses the corrected anchored path.
- TextInput undo/redo now uses a bounded host-owned history over the existing
  native text and selection model. Undo/redo emits ordinary change and
  selection events, preserving controlled-input acknowledgement behavior;
  no protocol or GPUI API change is required.

### Changed

- Domain documentation now refreshes `CONTEXT.md` and records ADR-0009 through
  ADR-0012 for asynchronous close confirmation, opt-in high-frequency event
  streams, the single-form wire-tail doctrine, and host-owned input models.

- TypeScript encoders now force float32 values; image source/URL resource caps
  count UTF-8 bytes consistently, while `maxLength` remains UTF-16 based.
- The gallery is responsive below 1100px (single column) and at wider widths
  (two columns), with 20/12/8 spacing, a 64px header, full-width bounded
  panels, `VirtualList`, refined cards/inputs/buttons/dropdown, isolated
  hover/pressed state, and accessibility labels.
- README DX guidance now states Pressable/TextInput focus boundaries,
  `onWindowResize` React bridging, an `import.meta.url` Image example for
  packaged builds, the `(value: string) => void` `onSubmitEditing` signature,
  and the fixed-row-height VirtualList constraint.

- Protocol v3 now enforces the current tuple arities for TextInput, Image,
  Drag, WindowResize, and CommandResult; historical forms are rejected instead
  of defaulting fields. `EVENT_SUBMIT` is string-only, including empty-string
  submissions.

- The host candidate workflow runs the full ordinary `make ci` gate before
  building and uploading its unsigned, short-retention artifact.

- Internal implementation seams now isolate host command-roundtrip fixtures and
  retained-tree validation: host `test_support` moves the host main module from
  1,722 to 848 lines, tree validation splits 1,380 into 870+519 with
  synchronized module-boundary guardrails, TextInput change/selection events
  delegate to a private helper, and test coverage remains equivalent at 89→89;
  wire and public behavior are unchanged.
- The core paint path is decomposed into renderer submodules while preserving
  the public API; wire and public behavior are unchanged.
