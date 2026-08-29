# Release-readiness inventory

Generated: 2026-08-30T00:36:08+0800
Decision owner: human release owner  
Current HEAD: `00f46a0` (`style: complete paint module boundary conformance`)

This is an evidence package, not a release approval. It records fresh command
runs, the current protocol/API surface, documentation consistency, known
boundaries, and inputs for a human version-cut decision. No artifacts were
published and no push, rebase, version change, or tag was performed.

## Executive readout

- The complete five-gate matrix passed at exit code 0 on the final tree at `00f46a0`: `make ci`, `make embedded-bun`, `make host-candidate-smoke`, `make host-embedded-candidate-smoke`, and `make bun-pack-smoke` all passed in the serial refresh below.
- The current candidate remains `0.1.0` on macOS ARM. The release scripts prove
  archive consistency and CLI/runtime behavior, not display-backed GUI behavior.
  Both process and embedded candidate rehearsals passed their expected timeout,
  help, version, and commit/press checks.
- The current Unreleased inventory contains **136 sectioned entries**:
  **114 Added**, **14 Fixed**, and **8 Changed**. This is materially beyond a
  patch-sized change set and provides stronger evidence for considering
  `0.2.0`. No option is selected: the human release owner retains the decision.
- `git log --oneline a5a91f6..HEAD` reports **9 commits** in this refresh
  delta: bounded rich run reassembly, deferred workstream issue notes, native
  window-title notes behavior, TestApp event/command facade coverage, targeted
  rich-text cache invalidation, run-scoped interactive cursor behavior, and
  three focused module/clippy boundary fixes. The crosswalk below records every
  commit in this delta.
- The implementation backlog for the current 0.2.0 contract remains empty.
  The latest pass adds host-owned undo/redo, precise VirtualList offset
  persistence, styled and selectable interactive Text runs, keyboard-accessible
  links with a visible focus affordance, targeted rich-text cache invalidation,
  run-scoped cursor behavior, the rich-text showcase, the TestApp facade drain
  and event injectors, native notes titles, and release-tooling rehearsal
  evidence; display-backed behavior remains a documented acceptance boundary.

`.scratch/release-productionization/v0.2.0-cut-checklist.md` is the companion
local checklist. `.scratch/release-productionization/dependency-audit.md` and
`.scratch/release-productionization/upstream-dependencies.md` are companion
dependency and upstream boundary notes. `.scratch/` is gitignored by default;
the readiness report and checklist follow the repository's force-add evidence
convention. No artifacts were published.

## 1. Capability inventory

### Protocol and wire contract

- The latest pass adds host-owned undo/redo, precise VirtualList offset
  persistence, styled and selectable interactive Text runs, keyboard-accessible
  links with a visible focus affordance, targeted rich-text cache invalidation,
  run-scoped cursor behavior, the rich-text showcase, the TestApp facade drain
  and event injectors, native notes titles, and release-tooling rehearsal
  evidence; display-backed behavior remains a documented acceptance boundary.
- The command directory is complete for codes `1..35`: focus/blur/selection,
  VirtualList scrolling and precise offset persistence, title/window commands,
  text and image clipboard, surface creation, file dialogs, notifications,
  static menus, keybindings, close-policy resolution, bounded text-file I/O,
  runtime font registration, minimize/bounds/state/activation controls.
  keyboard, pointer, hover, scroll, submit, window lifecycle, action,
  appearance, layout, drag, notification response, pointer-down-outside, and
  close-requested.
- Pointer press/release payloads use the strict seven-field form
  `[6,button,modifiers,action,clickCount,x,y]`; coordinates are finite,
  non-negative logical window pixels, while hover/move coordinates remain
  unexposed. Legacy five-field pointer payloads are rejected.
- Drag uses inbound `[1,type]`, `[2,type]`, and `[3,[path,...]]` notifications;
  outbound file drag uses host-properties tag 4
  `[4,dragType|null,exportFiles|null,acceptsDragOver,acceptsDrop]` with bounded
  1..8 host-local paths. The current wire has one required form; no legacy
  two-slot decoding is retained.
- Keybinding registration is root-only command 22 with full replacement,
  bounded entries/bytes/characters, atomic indexed validation, deterministic
  process-global union, and Event 17 action routing; context-conditional
  bindings remain unsupported.
- Text-file commands 25/26 are root-only asynchronous bounded UTF-8 reads and
  writes. Invalid paths, directories, oversized data, invalid UTF-8, and native
  I/O failures reject explicitly rather than blocking the UI executor.
- Clipboard-image commands 27/28 are root-only and carry bounded PNG, JPEG,
  GIF, or SVG bytes through binary MessagePack. macOS and Windows use native
  image entries; X11 and Wayland reject the unsupported path explicitly rather
  than falling back to text.
- Cross-language golden fixtures are generated in both directions. Producer
  bytes and semantic decoding cover current TextInput 13-slot, Image 4-slot,
  Drag 5-slot, strict Pointer 7-slot, CommandResult 7-slot, string Submit, and
  three-number WindowResize forms, plus fallback, outbound drag, keybinding,
  selectable Text, focusable Text-run, text-file, and clipboard-image vectors.

- Protocol references point at the split wire modules and contain zero bare
  `wire.rs` paths. `.scratch/release-productionization/upstream-dependencies.md`
  remains the three-tier tracking note for true upstream gaps, project limits
  that can be re-reviewed, and test-platform limitations.

- Host kinds are View, Text, Pressable, RawText, TextInput, VirtualList, and
  Image. Retained-tree validation covers ownership, contiguous indexes,
  revisions, listeners, host properties, accessibility, focusability, malformed
  patches, and rollback.
- VirtualList uses one persistent native GPUI variable-height `list` state per
  node: committed rows are measured at natural heights, while uncommitted rows
  use estimated placeholders. `ListState` owns scroll commands; the explicit
  `getScrollOffset()`/`scrollToOffset(offset)` commands preserve precise
  logical-pixel offsets across reconciliation and clamp restores at the new
  content end. `estimatedItemSize` is an initial hint; placeholder flicker
  remains display-backed.
- Image supports an optional `fallbackSource` in its tag-3 four-slot host
  payload; GPUI `with_loading` and `with_fallback` share the visual fallback
  path, stable element IDs prevent state collisions across mounted images,
  SVG byte sources load through the same pinned GPUI image element, and
  JavaScript `onError` remains a true upstream gap.
- Text supports one level of nested styled runs mixed with raw strings. Runs
  share the parent font size and line height while overriding color, weight,
  style, decoration, or family; nested `fontSize`/`lineHeight` and other
  non-typography fields are rejected. Selectable rich-text paragraphs span
  those runs, and pointer-activated link runs use `onPress` with
  `accessibilityRole="link"`; listener-bearing links are keyboard-accessible
  through native focus handles and tab stops, with existing focus/blur events,
  unmodified Enter activation, and a high-contrast per-line focus affordance.
  Space remains non-activating. The run-scoped pointing cursor is applied only
  to listener-bearing rich-text runs, so sibling/non-interactive text retains
  its node-level cursor style. The existing Text/RawText node kinds and wire
  remain unchanged.

- The public Root surface includes render/unmount, title/resize/zoom/fullscreen,
  focus traversal, URL opening, text and image clipboard, window-size query,
  multi-surface opening, asynchronous file dialogs and text-file I/O,
  notifications, static menus, keybindings, close policy/resolution, and the
  `onAction`, close, window-observation, and transport-termination callbacks.
- Styles use a validated **42-slot** positional tuple. The current style domain
  includes four physical flex directions, physical text alignment, cursor
  codes, positioning, typography, colors, overflow/truncation, margins, flex
  constraints, transition masks, `boxShadow` at slot 40, and `fontFamily` at
  slot 41. Explicit container/text base direction and bidi-aware hit/caret/IME
  semantics remain outside this contract; `letterSpacing` remains unsupported.
- TextInput placeholder text is gray visual guidance only when native text is
  empty. Single-line and multiline geometry use cached shaped/wrapped layouts
  with UTF-16 mapping; cross-line selection bounds union first/last visual rows.
  Host-owned Cmd/Ctrl-C/X/V clipboard editing, Cmd/Ctrl-A selection, Option/Alt
  word-wise navigation with Shift extension, and bounded Cmd/Ctrl-Z undo with
  Shift-Cmd-Z/Ctrl-Y redo are supported. Typing coalesces by caret continuity;
  paste/cut/selection edits and IME commits form history boundaries. Placeholder
  layout falls back to element bounds, while multiline IME candidate placement
  remains display-backed.
- Mouse interaction covers click-to-caret, drag selection, anchor/head/reversed
  orientation, selection highlighting, Shift navigation, Home/End, Up/Down,
  platform Arrow key names, UAX #29 double-click word selection, and
  triple-click logical-line selection without changing the wire contract.
- Pointer, hover, scroll, keyboard, focus/blur, press, submit, window resize,
  window activation, menu action, layout, drag, pointer-down-outside,
  close-requested, accessibility, and pointer-coordinate dispatch are covered
  by protocol and renderer tests. Pointer coordinates are attached to native
  down/up events only; the public event exposes logical `x`/`y` values.
- Focus traversal reaches React View, Pressable, TextInput, and selectable Text
  nodes through explicit native tab stops. Focused-node unmount emits JavaScript
  blur synchronously and restores focus to the live ancestor/restore target or
  first eligible stop. Disabled controls are skipped and per-window isolation
  is preserved.
- `pointerEvents` remains intentionally unexposed; normal GPUI hitboxes permit
  basic pass-through when no listener is installed, while zoom's headless
  command path is covered but its visual effect remains display-backed.
- Read-only Text selection is implemented through `TextProps.selectable`: the
  host owns anchor/head state, paints native-shaped per-row highlights, and
  supports host-side Cmd/Ctrl-C clipboard copy without a JavaScript selection
  event or mirrored React state. The optional Text-only wire tail and native
  cursor/clipboard behavior are covered headlessly, with final visual checks
  still display-backed.
- Window resize observations carry the required `[width,height,scaleFactor]`
  form and report scale-only changes through the same coalesced observer path.
  TypeScript callbacks and `WindowSize` stores expose the normalized scale
  factor; no legacy two-number frame is accepted.

### Runtime and host delivery

- ProcessAdapter uses framed stdin/stdout with a bounded ordered event writer,
  explicit shutdown/EOF/failure states, retained failure reporting, and
  non-sensitive wire identity in fatal diagnostics. Transport termination
  exposes typed causes and malformed host event frames fail fast while
  rejecting pending commands.
- EmbeddedBunAdapter runs pinned Bun/JSC on a dedicated runtime thread with a
  bounded callback bridge and Fast Refresh lifecycle serialization. The
  representative matrix covers gallery, text-input, virtual-list, and notes
  startup snapshots, plus a Fast Refresh lifecycle probe.
- SurfaceHost and the GPUI SurfaceRegistry demultiplex multiple roots/windows;
  final-window teardown, unknown-surface rejection, and reentrant
  renderer-death handling are explicit. A fatal shared-runtime failure closes
  all registered surfaces.
- Host CLI options, `--version`, `--help`, process/embedded selection, renderer
  argument forwarding, and expected timeout behavior are exercised by
  candidate scripts.
- The host panic hook preserves the standard stderr hook, writes a versioned
  report under `${REACT_GPUI_CRASH_DIR:-system temp}`, and prints its stable
  crash-report path on fatal exit. TypeScript termination errors retain an
  integer host exit code, the last 50 stderr lines, typed causes, and the path
  when the transport cause supplies them.

### Developer and distribution tooling

- Both Bun packages build ESM and declaration artifacts, expose built package
  exports, carry license text, and pass an external tarball consumer smoke.
  Current API locks are **core 114 / dev 20** name-and-kind exports.
- The core README indexes fourteen examples: counter, gallery, todo, keyboard,
  text-input, VirtualList, selectable-text, rich-text, stress, focus-flow,
  dropdown, drag-reorder, multi-surface, and notes. These examples cover
  appearance bridges, onLayout, drag handlers, styled/selectable Text runs,
  interactive links, reversed selection, clipboard text/image, file
  persistence, and word navigation. `@react-gpui/dev` also provides the
  behavior-level `renderTestApp` facade.

- The troubleshooting guide provides a symptom→diagnosis→repair entry point,
  while README Debugging retains the environment quick reference.
- Makefile gates, fixed-SHA workflows, release archive/checksum validation,
  process and embedded candidate rehearsals, release-prep synchronization,
  ADR-0008 error ownership, the consumer getting-started guide, and the
  cross-platform matrix workflow are present. The matrix is scaffolded at
  `.github/workflows/cross-platform.yml` and remains runner-only until
  push/PR execution.
- The release-prep rehearsal passed in detached worktree `/tmp/vue-gpui-release-rehearsal`:
  `make release-prep VERSION=0.2.0` synchronized Cargo and both Bun package
  versions, frozen lock checks passed, candidate packs were produced, and the
  deterministic 0.2.0 host archive check passed. The rehearsal also caught
  clean-checkout `bun-typecheck` ordering rot; `bun-typecheck: bun-build` is
  now the committed prerequisite and the fixed CI run passed.

## 2. Fresh five-gate evidence

All times below are fresh wall-clock `real` values from `/usr/bin/time -p` on
the final tree at HEAD `00f46a0`. The five commands ran serially and exclusively;
every command exited 0. The initial `make host-embedded-candidate-smoke` run
exceeded the tool's 300-second foreground limit without producing a result; the
gate was rerun successfully, and that retry is the recorded passing evidence.

| Command | Exit | Real time | Counts / fresh observed evidence |
| --- | ---: | ---: | --- |
| `make ci` | 0 | **67.33 s** | Rust **124 passed / 0 failed** (including module-boundary 6/6, perf 3/3, event-storm 1/1, protocol-fuzz 1/1, protocol-golden 3/3, process-roundtrip 3/3, command-roundtrip 27/27, examples-render 25/25, glyph-platform 2/2); Core Bun **130 pass / 0 fail / 89,335 expect() calls** across 16 files; dev Bun **24 pass / 0 fail / 56 expect() calls** across 4 files; builds, typechecks, and package consumer smoke passed. |
| `make embedded-bun` | 0 | **1.98 s** | `embedded_examples` **2 passed** (startup matrix and Fast Refresh lifecycle); `react-gpui-bun` embedded counter **1 passed**. |
| `make host-candidate-smoke` | 0 | **36.18 s** | Deterministic archive SHA-256 **`668083b77ff406f46cdc04ec94c77eca490e048f974f55e115694305ced85064`** matched on both archives; snapshot commit **312 bytes**; expected process timeouts **5.008 s** (error) and **5.016 s** (info); version/help passed. |
| `make host-embedded-candidate-smoke` | 0 | **5.23 s** (retry) | Release embedded binary passed; `--smoke-press` reported `sent=true, commits=2, status=None`; expected timeout **4.795 s**; version/help passed. Initial run exceeded the tool's 300-second foreground limit; retry passed. |
| `make bun-pack-smoke` | 0 | **4.44 s** | Frozen installs, both JS/type builds and tarballs, external consumer install (**59 packages**), and `package tarball consumer smoke passed`. |

The measured sequential five-gate wall-time sum for the successful recorded
runs is **115.16 s**. The initial host-embedded candidate tool timeout is not
included in that sum. Candidate timeout lines are expected successful rehearsal
behavior. The host archive SHA is the process-candidate check's identical pair
of SHA-256 values.

### Quality note: regression detection

During this refresh, commit `655c71a` temporarily regressed the synchronous
validation contract by removing eager `validateStyle` from renderer props.
The renderer suite caught the regression; same-round hotfix `9d17120` restored
the eager validation, and the post-fix focused renderer suite reported **73/73
tests in 95 ms**. The passing gates therefore provide evidence that this matrix
catches real regressions, not only build or packaging failures.
## 3. Quality evidence matrix

| Area | Current evidence |
| --- | --- |
| Rust workspace | Fresh `make ci` passed with **124 tests / 0 failures** across the workspace (including module-boundary 6/6, perf 3/3, event-storm 1/1, protocol-fuzz 1/1, protocol-golden 3/3, process-roundtrip 3/3, command-roundtrip 27/27, examples-render 25/25, and glyph-platform 2/2); `make embedded-bun` passed its embedded host startup matrix **2/2** and embedded counter **1/1**. |
| TypeScript renderer | Fresh `make ci`: **130/130 tests**, **89,335 assertions**, 0 failures; the synchronous style-validation regression from `655c71a` was caught and fixed in `9d17120` before this final matrix. |
| Development package | Fresh `make ci`: **24/24 tests**, **56 assertions**, expected malformed-source diagnostics contained and last-good tree preserved. |
| Fuzz / malformed input | Rust deterministic protocol-fuzz and TypeScript malformed-input coverage remained green in fresh `make ci`. |
| Performance smoke | Fresh Rust perf suites and TypeScript event-storm scenarios remained green in `make ci`. |
| Golden vectors | Rust golden and TypeScript protocol-golden checks passed in `make ci`; VirtualList offset, selectable rich Text, focusable Text-run, pointer-move, and window-control vectors are covered by current protocol tests. |
| Order independence | Commit `c7850c5` recorded 10 randomized core Bun runs (128 tests each), randomized dev Bun, parallel/single-thread Rust modes, and repeated host example smoke with **0 failures**; this is retained as suite-order evidence. |
| API surface locks | Checked-in name/kind fixtures currently lock **core 114 / dev 20** exports. |
| Protocol tap / crash diagnostics | Typed transport-cause, crash-path, tap, and termination suites passed in `make ci`. |
| Release packaging | Process and embedded candidate scripts, deterministic archive checks, package tarball consumer smoke, and embedded package gate all passed. Checksums establish reproducibility, not publisher authenticity. |

## 3.1 Citation audit evidence

The prior citation audit covered `docs/protocol.md`, `docs/getting-started.md`,
`docs/troubleshooting.md`, the root `README.md`, and both package READMEs.
Occurrence-based guide/README sampling checked **90** explicit repository
references before the independent corrections (**20 getting-started, 43
troubleshooting, 27 root README**): **3 wrong, 3 fixed, 0 remaining**. The
three fixes are in commit `393e71d`. `docs/protocol.md` was then checked for
**290 citation occurrences and 480 numeric range segments** against its audit
tree with 0 out of range. This refresh records the current protocol and API
claims above; it does not claim a new citation audit.

## 4. Consistency and crosswalk audit

### (a) Current Unreleased inventory

The current section contains **136** aggregate capability entries: **114 Added**,
**14 Fixed**, and **8 Changed**. These are aggregate statements rather than one
line per commit. `git log --oneline a5a91f6..HEAD` reports **9 commits** in this
refresh delta. The crosswalk retains the earlier aggregated-domain style and
records every commit in this delta below:

| Commit | Reachable evidence |
| --- | --- |
| `598448a` | Bounded rich-run reassembly and performance guard evidence. |
| `348a3d0` | Deferred workstream issue notes for run cursor and rich-text cache work. |
| `475f6ad` | Native window title in the notes example and protocol documentation corrections. |
| `f024b5c` | TestApp selection, external-file-drop, layout injectors, and `drainCommands()` facade coverage and doctrine. |
| `704beff` | Targeted rich-text cache invalidation for changed ancestors, creates, moves, and deletes. |
| `17a28dc` | Run-scoped pointing cursor for interactive rich-text runs, with README/example/changelog coverage. |
| `56bd91b` | Clippy cleanups for the run-scoped cursor implementation. |
| `c9e212a` | Paint module-boundary conformance for the cursor helper. |
| `00f46a0` | Complete paint module-boundary conformance, including test-only imports. |
| `346e99b` | VirtualList precise logical-pixel `getScrollOffset()`/`scrollToOffset()` commands and persistence tests. |
| `c7ee8c6` | Detached-worktree 0.2.0 release-prep rehearsal and Makefile `bun-typecheck` → `bun-build` ordering fix. |
| `fc46d01` | Mixed raw/styled Text runs with one-level nesting and typography validation. |
| `655c71a` | Selectable rich-text paragraphs and pointer-activated link runs with `onPress`/link accessibility semantics. |
| `9d17120` | Renderer hotfix restoring eager synchronous `validateStyle` and the validation-throw contract. |
| `30f6dd4` | Keyboard-accessible nested Text link runs with native focus handles/tab stops, focus/blur routing, and unmodified Enter activation. |
| `8ce3246` | Cross-language protocol golden vectors and checks for focusable Text runs. |
| `54807ce` | Protocol reference correction documenting Text focusability and Enter/Space behavior. |
| `cf52b68` | High-contrast per-line focus affordance for focused interactive Text runs. |
| `35e0150` | Rustfmt-only cleanup of the protocol golden example; no behavior or changelog entry. |
| `9e885e9` | Rich-text showcase example, registration in host example rendering, and README/CHANGELOG coverage; amended landing commit places both new user-facing bullets under Added. |

### (b) Protocol directory versus source constants

The current source has exactly **35 command constants** with values `1..35` and
**23 semantic event constants** with values `1..23` (pointer/key sub-action and
button constants excluded). The protocol reference has matching event/command
rows, a **42-slot** style tuple with `boxShadow`/`fontFamily` tails, and one
current wire form for TextInput (13 slots), Image (4), Drag (5), Pointer (7),
CommandResult (7), string Submit, and WindowResize (3). OpenSurface retains its
valid no-options form plus the optional creation-options tuple; other historical
compatibility forms were removed by the cutover.

### (c) README and consumer-guide capability audit

Root/package README claims were checked against the current Root and TypeScript
interfaces. They cover typed surface/transport errors, tooltip and close-policy
behavior, expanded/heading-level accessibility, native keyboard focus traversal
and focused-unmount restoration, text-file persistence, clipboard images,
embedded startup coverage, pointer press/release coordinates, runtime font
loading, and window controls. No pointer, font, or window-control work is treated
as in-flight at this committed HEAD.

### (d) ADR alignment

ADR-0008's four failure seams remain aligned with implementation: consumer
Error-Boundary ownership for render failures, fail-fast protocol/tree rejection,
local blank-image degradation, and host-fatal GPUI paint panic. Shared-runtime
fatal errors close every registered surface. The typed surface errors,
close-policy, file/clipboard commands, pointer-coordinate contract, runtime
fonts, and window controls agree with the protocol and consumer documentation.

## 5. Known limits and human decision items

The implementation backlog for the current 0.2.0 contract is empty after the
current input/geometry/image/scale/focus/tooltip/accessibility/file/clipboard/
keybinding/drag/selectable-text/pointer-coordinate/font/window-control review.
Selectable Text is implemented as a host-owned visual surface; the following
are explicit acceptance boundaries or human decisions, not hidden TODOs:

- Issue 01 remains `needs-triage`: the candidate is unsigned and not notarized;
  checksums do not authenticate the publisher.
- Issue 02 remains `ready-for-human`: `block 0.1.6` emits a future-incompatibility
  warning and no unreviewed fork should be patched automatically.
- Issue 03 remains `ready-for-human`: embedded source build is separate, green,
  expensive, and outside ordinary `make ci`; the representative startup matrix
  is now covered by that gate.
- Issue 04 remains `needs-triage`: Quartz dialogs, clipboard/menu/window
  behavior, glyph pixels, native drag visuals, tooltip appearance, pointer
  positioning, and AX-tree behavior require a real display/hardware runner;
  headless evidence cannot replace it.
- Issue 05 remains `needs-triage`: malformed-input coverage is implemented and
  green; long-term fuzz maintenance ownership is human-owned.
- Issue 06 remains `needs-triage`: the candidate is macOS ARM. The Linux/Windows
  backend, lock graph, SDK, and display matrix workflow is scaffolded at
  `.github/workflows/cross-platform.yml`, but runner validation is not claimed
  until its push/PR jobs execute; local macOS cross-checks are not runner proof.
- Upstream gaps remain explicit: native TextInput undo/redo, secure/password
  display, explicit RTL/container text base direction and bidi-aware hit/caret/
  IME semantics, `letterSpacing`, and JavaScript Image `onError`. `fallbackSource`
  remains the visual Image degradation path. Targeted rich-text cache invalidation
  is now implemented and the deferred rich-cache issue is resolved; unrelated
  patches preserve assembled runs while changed paragraphs and final ancestors
  rebuild. The run-scoped pointing cursor is likewise implemented for
  listener-bearing rich-text runs; no full-cache-clear or parent-cursor behavior
  remains as a known tradeoff.

## 6. Version-cut guidance (not a decision)

The workspace remains `0.1.0`; `release-prep` can synchronize a chosen version
into Cargo and both Bun packages. The current Unreleased inventory is **136
entries (114/14/8)** across protocol, typed surface lifecycle errors, tooltips
and close policy, accessibility semantics, native keyboard focus traversal and
focused-unmount restoration, text-file persistence, clipboard text/images,
embedded representative examples, pointer coordinates, selectable and styled
Text runs with keyboard-accessible interactive links and focus affordances,
targeted rich-text cache invalidation, run-scoped interactive cursors,
rich-text examples, host-owned TextInput undo/redo, precise VirtualList offset
persistence, runtime font registration, window controls, renderer API, native
runtime, Image fallback, scale-factor observations, focus/overlay and drag
input, appearance-aware examples, pointer/zoom boundaries, keybinding
registration, outbound file drag, precise TextInput geometry and host editing,
the TestApp facade event injectors and command drain, native notes titles,
troubleshooting, and long-run stability.
The final matrix reports **130 Core Bun tests / 89,335 assertions**, **24 dev
tests / 56 assertions**, and **114+20 API locks**. These facts support
considering `0.2.0` rather than a patch cut, but no version option is selected
by this inventory.

The companion checklist is
`.scratch/release-productionization/v0.2.0-cut-checklist.md`. Its five human
decisions and seven-step mechanical sequence agree with this report's gate
status. One mechanical caveat remains: `release-prep.sh` requires an exact
`## [0.2.0]` heading and does not accept the dated
`## [0.2.0] - YYYY-MM-DD` form without a small guard update or temporary exact
heading. The checklist records that caveat and keeps the final decision with
the human release owner.

No option is selected by this inventory.
