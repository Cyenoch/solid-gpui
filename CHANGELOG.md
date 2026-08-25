# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to Semantic Versioning. No version has been released
from this work tree.

## [Unreleased]

### Added

- System notifications use `SHOW_NOTIFICATION=20` with `[title, body]` and a
  constrained fire-and-forget API; delivery varies by platform, host identity
  is a prerequisite, and tags, actions, and response callbacks are not exposed.
- Static menus use nested `SET_MENUS=21` wire payloads and
  `EVENT_ACTION=17` string action events, with `RootOptions.onAction`; dynamic
  enablement and keybinding registration remain intentionally unsupported.
- Crash diagnostics install a host panic hook with
  `REACT_GPUI_CRASH_DIR` crash logs; TypeScript transport termination errors
  carry host exit codes and the last 50 stderr lines. README Troubleshooting
  documents reproduction and an optional APM sample.
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
  `BLACK=900`; `fontFamily` remains intentionally unsupported and is documented
  as such.
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

### Fixed

- CommandResult acknowledgement validation now accepts command kinds `6`-`12`,
  preserving Rust-layer replies for title and other surface commands.
- Integer-dimension resize frames were ambiguous with the untagged Animation
  `[tag,generation]` payload and could be rejected; event-type-directed
  deserialization now selects `WindowResizeWire` and accepts integer/float
  dimensions.
- EventWire visitors now propagate decode errors for type-mismatched payloads
  directly; the previous sentinel-based malformed payload relied on
  cross-variant rejection by coincidence.

### Changed

- TypeScript encoders now force float32 values; image source/URL resource caps
  count UTF-8 bytes consistently, while `maxLength` remains UTF-16 based.
- README DX guidance now states Pressable/TextInput focus boundaries,
  `onWindowResize` React bridging, an `import.meta.url` Image example for
  packaged builds, the `(value: string) => void` `onSubmitEditing` signature,
  and the fixed-row-height VirtualList constraint.

- `TextInput.onSubmitEditing` is now a breaking `(value: string) => void`
  callback: `EVENT_SUBMIT` carries native text as a string, with an empty
  string for the null-frame compatibility fallback; the host remains
  authoritative and golden vectors lock both forms.
- The host candidate workflow runs the full ordinary `make ci` gate before
  building and uploading its unsigned, short-retention artifact.

