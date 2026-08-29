# Release-readiness inventory

Generated: 2026-08-30T05:46:56+0800
Decision owner: human release owner  
Current HEAD: `9e53ea9` (`perf(renderer): affected-aware reconciliation and snapshot guard`)

This is an evidence package, not a release approval. It records the post-release
cycle position, fresh command runs, the current protocol/API surface,
documentation consistency, known boundaries, and inputs for the human
publication decision. Version `0.2.0` was cut and tagged locally; no artifact
has been published or pushed.

## Executive readout

- All five fresh gates passed at exit code 0 on the current tree at `9e53ea9`:
  `make ci`, `make embedded-bun`, `make host-candidate-smoke`,
  `make host-embedded-candidate-smoke`, and `make bun-pack-smoke`, run
  serially and exclusively under `/usr/bin/time -p`.
- The current candidate is `0.2.0` on macOS ARM. The release scripts prove
  archive consistency and CLI/runtime behavior, not display-backed GUI
  behavior. Both process and embedded candidate rehearsals passed their
  expected timeout, help, version, and commit/press checks.
- The current `CHANGELOG.md` Unreleased section contains exactly **3 entries**:
  **2 Added**, **1 Fixed**, and **0 Changed**. The gallery rich-text showcase is
  an Added capability; the affected-aware reconciliation is the sole Fixed
  entry; the large-tree guard is Added.
- `git log --oneline 72a520c..HEAD` reports **7 post-cut commits**. The
  crosswalk below records the local cut record, fuzz completion, gallery
  showcase, artifact/evidence manifest, upstream assessment, troubleshooting
  guide, and affected-aware renderer/performance work.
- The implementation backlog for the current 0.2.0 contract remains empty.
  Current capability evidence includes deterministic protocol fuzz coverage
  **35/35 commands and 23/23 events**, the gallery showcase, the isolated
  20,000-node snapshot/patch guard, and the troubleshooting guide.

## Current position

**v0.2.0 cut locally + tagged; publication (push/npm/signing) pending human
action.** The annotated local tag `v0.2.0` still points to release commit
`72a520c`; the post-cut renderer work is intentionally not being moved onto
that tag. The current candidate archive is therefore compared with, rather
than substituted for, the cut artifact.

`.scratch/release-productionization/v0.2.0-cut-checklist.md` is the companion
local checklist. `.scratch/release-productionization/dependency-audit.md` and
`.scratch/release-productionization/upstream-dependencies.md` are companion
dependency and upstream boundary notes; the post-cut upstream verdict is that
no released GPUI change unblocks the recorded boundaries. `.scratch/` is
gitignored by default; the readiness report and checklist follow the
repository's force-add evidence convention. No artifacts were published.

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
  `wire.rs` paths. The refreshed `.scratch/release-productionization/upstream-
  dependencies.md` records current manifest/lock direct edges, locked versions,
  and verified dependency purposes; the dead host `rmp-serde` dev edge was then
  removed by `7edad1f`.
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
  includes four physical flex directions, physical text alignment, cursor codes,
  positioning, typography, colors, overflow/truncation, margins, flex
  constraints, transition masks, `boxShadow` at slot 40, and `fontFamily` at
  slot 41. Explicit container/text base direction and bidi-aware hit/caret/IME
  semantics remain outside this contract; `letterSpacing` remains unsupported.
- TextInput placeholder text is gray visual guidance only when native text is
  empty. Single-line and multiline geometry use cached shaped/wrapped layouts
  with UTF-16 mapping; cross-line selection bounds union first/last visual rows.
  Host-owned Cmd/Ctrl-C/X/V clipboard editing, Cmd/Ctrl-A selection, Option/Alt
  word-wise navigation with Shift extension, and bounded Cmd/Ctrl-Z undo with
  Shift-Cmd-Z/Ctrl-Y redo are supported. Typing coalesces by caret continuity;
  paste/cut/selection edits and IME commits form history boundaries. The new
  typing-performance guard covers 100/1,000/10,000-character single-line and
  wrapped multiline inputs, records p50/p99 and call counts, and attributes the
  dominant long-line cost to pinned GPUI whole-line shaping. Placeholder layout
  falls back to element bounds, while multiline IME candidate placement remains
  display-backed.
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
  ADR-0008 error ownership, ADR-0013 interactive Text runs, the consumer
  getting-started guide, and the cross-platform matrix workflow are present.
  The matrix is scaffolded at `.github/workflows/cross-platform.yml` and
  remains runner-only until push/PR execution.
- The release-prep rehearsal passed in detached worktree
  `/tmp/vue-gpui-release-rehearsal`: `make release-prep VERSION=0.2.0`
  synchronized Cargo and both Bun package versions, frozen lock checks passed,
  candidate packs were produced, and the deterministic 0.2.0 host archive
  check passed. The rehearsal also caught clean-checkout `bun-typecheck`
  ordering rot; `bun-typecheck: bun-build` is now the committed prerequisite
  and the fixed CI run passed.

## 2. Fresh five-gate evidence

All times below are fresh wall-clock `real` values from `/usr/bin/time -p` on
the current post-cut tree at HEAD `9e53ea9`. The five commands ran serially and
exclusively in this order: `make ci`, `make embedded-bun`,
`make host-candidate-smoke`, `make host-embedded-candidate-smoke`, and
`make bun-pack-smoke`. Every command exited 0. Candidate timeout lines are
expected successful rehearsal behavior.

| Command | Exit | Real time | Counts / fresh observed evidence |
| --- | ---: | ---: | --- |
| `make ci` | 0 | **79.28 s** | Rust **128 passed / 0 failed**; Core Bun **130 pass / 0 fail / 90,533 expect() calls** across 16 files; dev Bun **24 pass / 0 fail / 56 expect() calls** across 4 files; formatting, checks, builds, typechecks, package consumer smoke, and TextInput typing-performance guard passed. |
| `make embedded-bun` | 0 | **3.97 s** | `embedded_examples` **2 passed** (startup matrix and Fast Refresh lifecycle); `react-gpui-bun` embedded counter **1 passed**; locked checks passed. |
| `make host-candidate-smoke` | 0 | **35.05 s** | Identical archive SHA-256 **`d3d3075697876f6a0d1055251693d5b5f5b03b9c26f969655729af7999478c3c`** matched on both archives; archive `react-gpui-host-0.2.0-aarch64-apple-darwin.tar.gz`; snapshot commit **312 bytes**; expected process timeouts **5.019 s** (error) and **5.013 s** (info); version/help passed. |
| `make host-embedded-candidate-smoke` | 0 | **19.85 s** | Release embedded binary passed; `--smoke-press` reported `sent=true, commits=2, status=None`; expected timeout **4.795 s**; version/help passed. |
| `make bun-pack-smoke` | 0 | **2.03 s** | Frozen installs, both 0.2.0 JS/type builds and tarballs, external consumer install (**59 packages**), and `package tarball consumer smoke passed`. |

The measured sequential five-gate wall-time sum for these successful runs is
**140.18 s**. The host archive SHA is the process-candidate check's identical
pair of SHA-256 values. Candidate timeout exits are expected behavior, not gate
failures.

### Quality notes

- The current Unreleased section is intentionally classified as **2 Added / 1
  Fixed / 0 Changed**. The gallery rich-text card is an Added showcase
  capability, not a defect repair; this classification was corrected after the
  previous refresh (the fourth section-placement correction in this evidence
  series).
- Deterministic protocol fuzz coverage is complete at **35/35 command seeds
  and 23/23 event seeds** in both Rust and TypeScript coverage records.
- The affected-aware reconciliation fix limits input, selectable-text,
  VirtualList, and animation maintenance to affected nodes and relevant
  ancestors; the guard covers 1,000/5,000/20,000-node snapshots and a one-op
  20,000-node patch. The isolated 20k host stage is **70.705 ms** for the
  snapshot and **0.131 ms** for the one-operation patch; 20k apply p50/p99 is
  **259.019/259.656 ms** and first-draw p50/p99 is **195.747/198.639 ms**.
- The troubleshooting guide now provides a symptom→diagnosis→repair entry
  point while retaining the README debugging quick reference.

## 3. Quality evidence matrix

| Area | Current evidence |
| --- | --- |
| Rust workspace | Fresh `make ci` passed with **128 tests / 0 failures** across the workspace (including module-boundary 6/6, perf 3/3, event-storm 1/1, protocol-fuzz 1/1, protocol-golden 3/3, process-roundtrip 3/3, command-roundtrip 27/27, examples-render 25/25, and glyph-platform 2/2); `make embedded-bun` passed its embedded host startup matrix **2/2** and embedded counter **1/1**. |
| TypeScript renderer | Fresh `make ci`: **130/130 tests**, **90,533 assertions**, 0 failures; malformed-source diagnostics remained contained and the last-good tree was preserved. |
| Development package | Fresh `make ci`: **24/24 tests**, **56 assertions**, expected malformed-source diagnostics contained and last-good tree preserved. |
| Fuzz / malformed input | Deterministic protocol fuzz coverage is complete at **35/35 command seeds and 23/23 event seeds** in both Rust and TypeScript records; fresh `make ci` remained green. |
| Performance smoke | Fresh Rust perf suites, the TextInput typing-performance guard, and TypeScript event-storm scenarios remained green in `make ci`. |
| Golden vectors | Rust golden and TypeScript protocol-golden checks passed in `make ci`; VirtualList offset, selectable rich Text, focusable Text-run, pointer-move, and window-control vectors are covered by current protocol tests. |
| API surface locks | Checked-in name/kind fixtures currently lock **core 114 / dev 20** exports. |
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

The current `CHANGELOG.md` Unreleased section contains exactly **3 aggregate
capability entries**: **2 Added**, **1 Fixed**, and **0 Changed**. The Added
entries are the headless large-tree snapshot/first-frame guard and the flagship
gallery's mixed-style rich-text/link/selectable showcase. The Fixed entry is
affected-aware input/selectable-text/VirtualList/animation reconciliation with
lazy styled-node animation history. No historical top-level bullets remain in
this freshly cut Unreleased section.

### (b) Post-cut commit crosswalk

`git log --oneline 72a520c..HEAD` reports **7 post-cut commits**:

| Commit | Reachable evidence |
| --- | --- |
| `f48f91e` | Recorded the executed local 0.2.0 cut, release commit, annotated tag, publication boundaries, and final pre-cut matrix in the companion checklist. |
| `ddb21bf` | Completed deterministic protocol fuzz seed coverage at 35/35 commands and 23/23 events in Rust and TypeScript coverage records. |
| `593ab52` | Added the flagship gallery rich-text/link/selectable showcase; its changelog entry is correctly classified under Added. |
| `38a0f2d` | Added the v0.2.0 release artifact manifest and evidence index, including the pre-cut archive SHA and tag record. |
| `ea6b67b` | Recorded the GPUI upstream drift assessment; no released upstream change unblocks the existing boundaries. |
| `c01adf5` | Added and refreshed the troubleshooting guide while retaining the README debugging quick reference. |
| `9e53ea9` | Added the affected-aware renderer reconciliation fix and large-tree snapshot/first-frame performance guard; this commit changed the host candidate binary. |

### (c) Protocol directory versus source constants
The current source has exactly **35 command constants** with values `1..35` and
**23 semantic event constants** with values `1..23` (pointer/key sub-action and
button constants excluded). The protocol reference has matching event/command
rows, a **42-slot** style tuple with `boxShadow`/`fontFamily` tails, and one
current wire form for TextInput (13 slots), Image (4), Drag (5), Pointer (7),
CommandResult (7), string Submit, and WindowResize (3). OpenSurface retains its
valid no-options form plus the optional creation-options tuple; other historical
compatibility forms were removed by the cutover.

### (d) README and consumer-guide capability audit
Root/package README claims remain aligned with the current Root and TypeScript
interfaces. They cover typed surface/transport errors, tooltip and close-policy
behavior, expanded/heading-level accessibility, native keyboard focus traversal
and focused-unmount restoration, text-file persistence, clipboard images,
embedded startup coverage, pointer press/release coordinates, runtime font
loading, and window controls. No pointer, font, or window-control work is
treated as in-flight at this committed HEAD.

### (e) ADR and upstream assessment alignment
ADR-0008's four failure seams remain aligned with implementation: consumer
Error-Boundary ownership for render failures, fail-fast protocol/tree rejection,
local blank-image degradation, and host-fatal GPUI paint panic. The GPUI drift
assessment's verdict is **no released unblocks**: released GPUI still lacks the
recorded per-run typography, explicit direction, letter spacing, secure
obscuring, live-region, runtime position-setter, and portable Linux image seams.

## 5. Known limits and human decision items

The implementation backlog for the current 0.2.0 contract remains empty after
the current input/geometry/image/scale/focus/tooltip/accessibility/file/
clipboard/keybinding/drag/selectable-text/pointer-coordinate/font/window-
control review. Selectable Text is implemented as a host-owned visual surface;
the following are explicit acceptance boundaries or human decisions, not hidden
TODOs:

- Issue 01 remains `needs-triage`: the candidate is unsigned and not notarized;
  checksums do not authenticate the publisher.
- Issue 02 remains `ready-for-human`: `block 0.1.6` emits a future-
  incompatibility warning and no unreviewed fork should be patched
  automatically.
- Issue 03 remains `ready-for-human`: embedded source build is separate,
  green, expensive, and outside ordinary `make ci`; the representative startup
  matrix is covered by that gate.
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
- Upstream gaps remain explicit: secure/password display, explicit RTL/container
  text base direction and bidi-aware hit/caret/IME semantics, `letterSpacing`,
  and JavaScript Image `onError`. Native TextInput undo/redo is host-owned and
  implemented, while `fallbackSource` remains the visual Image degradation
  path. The GPUI upstream drift verdict is **no released unblocks**.

## 6. Post-release cycle position

**v0.2.0 cut locally + tagged; publication (push/npm/signing) pending human
action.** Tag `v0.2.0` remains anchored to release commit `72a520c`; no tag move
or publication operation is part of this refresh. The post-cut candidate
archive SHA is **`d3d3075697876f6a0d1055251693d5b5f5b03b9c26f969655729af7999478c3c`**.
The cut checklist recorded **`d0dfc86e8c5b98d790d99eae0ec2ebb485e3b1a8cc28939cdafbf9a973e25ab`**;
the values differ because `9e53ea9` changed the host binary. The tag still
points at `72a520c`, honestly preserving the cut artifact record.

The new cycle currently contains exactly **2 Added / 1 Fixed / 0 Changed**
Unreleased entries: gallery showcase and large-tree guard under Added, and
affected-aware reconciliation under Fixed. The affected-aware fix's measured
20k host stage is approximately **70.705 ms** for the snapshot and **0.131 ms**
for the one-operation patch; the guard reports 20k apply p50/p99
**259.019/259.656 ms**, first-draw p50/p99 **195.747/198.639 ms**, and covers
1k/5k/20k trees. Fuzz coverage is complete at **35/35 commands and 23/23
events** in both languages. The gallery rich-text showcase, troubleshooting
guide, artifact manifest, and upstream drift assessment are all recorded in
the post-cut crosswalk above.

The companion checklist is
`.scratch/release-productionization/v0.2.0-cut-checklist.md`; its cut record
remains historical, while its post-cut refresh records the new-cycle counts,
fresh gates, archive comparison, and human publication boundaries.

No publication option is selected by this inventory; human action is required
for push, npm publication, signing/notarization, and any Rust crate release.
