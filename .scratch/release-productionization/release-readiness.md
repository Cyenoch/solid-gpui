# Release-readiness inventory

Generated: 2026-08-29T05:03:57Z
Decision owner: human release owner  
Current HEAD: `9d17120` (`fix(renderer): restore synchronous style validation`)

This is an evidence package, not a release approval. It records fresh command
runs, the current protocol/API surface, documentation consistency, known
boundaries, and inputs for a human version-cut decision. No artifacts were
published and no push, rebase, version change, or tag was performed.

## Executive readout

- The complete five-gate matrix passed at exit code 0 on the final tree at `9d17120`: `make ci`, `make embedded-bun`, `make host-candidate-smoke`, `make host-embedded-candidate-smoke`, and `make bun-pack-smoke` all passed in the serial refresh below.
- The current candidate remains `0.1.0` on macOS ARM. The release scripts prove
  archive consistency and CLI/runtime behavior, not display-backed GUI behavior.
  Both process and embedded candidate rehearsals passed their expected timeout,
  help, version, and commit/press checks.
- The current Unreleased inventory contains **132 top-level entries**:
  **111 Added**, **13 Fixed**, and **8 Changed**. This is materially beyond a
  patch-sized change set and provides stronger evidence for considering
  `0.2.0`. No option is selected: the human release owner retains the decision.
- `git log --oneline 96bccc1..HEAD` reports **13 commits** in this refresh
  delta. The crosswalk below records the domain evidence for every commit,
  including undo/redo, VirtualList offset commands, order-independence coverage,
  release rehearsal, rich-text runs, and the synchronous validation hotfix.
- The implementation backlog for the current 0.2.0 contract remains empty.
  The latest pass adds host-owned undo/redo, precise VirtualList offset
  persistence, styled and selectable interactive Text runs, and release-tooling
  rehearsal evidence; display-backed behavior remains a documented acceptance
  boundary.
- Remaining items are documented product/platform boundaries or human release
  decisions, not unrecorded implementation residue. Multi-click, clipboard
  text/image, select-all, word-navigation, file I/O, focus traversal/
  restoration, pointer-coordinate behavior, undo/redo, rich Text runs, and
  VirtualList offset commands are covered by the current implementation and
  tests.

`.scratch/release-productionization/v0.2.0-cut-checklist.md` is the companion
local checklist. `.scratch/release-productionization/dependency-audit.md` and
`.scratch/release-productionization/upstream-dependencies.md` are companion
dependency and upstream boundary notes. `.scratch/` is gitignored by default;
the readiness report and checklist follow the repository's force-add evidence
convention. No artifacts were published.

## 1. Capability inventory

### Protocol and wire contract

- Protocol v3 uses positional MessagePack Snapshot, Event, Patch, and Command
  frames with explicit surface/epoch/revision/request sequencing.
- The command directory is complete for codes `1..33`: focus/blur/selection,
  VirtualList scrolling, title/window commands, text and image clipboard,
  surface creation, file dialogs, notifications, static menus, keybindings,
  close-policy resolution, bounded text-file I/O, runtime font registration,
  minimize/bounds/state/activation controls.
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
  selectable Text, text-file, and clipboard-image vectors.
- Protocol references point at the split wire modules and contain zero bare
  `wire.rs` paths. `.scratch/release-productionization/upstream-dependencies.md`
  remains the three-tier tracking note for true upstream gaps, project limits
  that can be re-reviewed, and test-platform limitations.

### React renderer and retained tree

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
  non-typography fields are rejected. Selectable rich-text paragraphs now span
  those runs, and pointer-activated link runs use `onPress` with
  `accessibilityRole="link"`; per-run keyboard focus remains a documented
  boundary. The existing Text/RawText node kinds and wire remain unchanged.
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
- The core README indexes thirteen examples: counter, gallery, todo, keyboard,
  text-input, VirtualList, selectable-text, stress, focus-flow, dropdown,
  drag-reorder, multi-surface, and notes. These examples cover appearance
  bridges, onLayout, drag handlers, styled/selectable Text runs, interactive
  links, reversed selection, clipboard text/image, file persistence, and word
  navigation. `@react-gpui/dev` also provides the behavior-level
  `renderTestApp` facade.
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
the final tree at HEAD `9d17120`. The five commands ran serially and exclusively;
every command exited 0.

| Command | Exit | Real time | Counts / fresh observed evidence |
| --- | ---: | ---: | --- |
| `make ci` | 0 | **70.49 s** | Rust **199 passed / 0 failed**; Core Bun **130 pass / 0 fail / 89,328 expect() calls** across 16 files; dev Bun **20 pass / 0 fail / 50 expect() calls** across 4 files; builds, typechecks, and package consumer smoke passed. |
| `make embedded-bun` | 0 | **4.67 s** | `embedded_examples` **2 passed** (startup matrix and Fast Refresh lifecycle); `react-gpui-bun` embedded counter **1 passed**. |
| `make host-candidate-smoke` | 0 | **42.28 s** | Deterministic archive SHA-256 **`1010a5160122ed492492396de9c11e4e665f97c280cc0e2f84e33356315884bc`** matched on both archives; snapshot commit **312 bytes**; expected process timeouts **5.008 s** (error) and **5.005 s** (info); version/help passed. |
| `make host-embedded-candidate-smoke` | 0 | **26.41 s** | Release embedded binary passed; `--smoke-press` reported `sent=true, commits=1, status=None`; expected timeout **4.563 s**; version/help passed. |
| `make bun-pack-smoke` | 0 | **9.99 s** | Frozen installs, both JS/type builds and tarballs, external consumer install (**59 packages**), and `package tarball consumer smoke passed`. |

The measured sequential five-gate wall-time sum is **153.84 s**. Candidate
timeout lines are expected successful rehearsal behavior. The host archive SHA
is the process-candidate check's identical pair of SHA-256 values.

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
| Rust workspace | Fresh `make ci` passed with **199 tests / 0 failures**; `make embedded-bun` passed its embedded host startup matrix **2/2** and embedded counter **1/1**. |
| TypeScript renderer | Fresh `make ci`: **130/130 tests**, **89,328 assertions**, 0 failures; the synchronous style-validation regression from `655c71a` was caught and fixed in `9d17120` before this final matrix. |
| Development package | Fresh `make ci`: **20/20 tests**, **50 assertions**, expected malformed-source diagnostics contained and last-good tree preserved. |
| Fuzz / malformed input | Rust deterministic protocol-fuzz and TypeScript malformed-input coverage remained green in fresh `make ci`. |
| Performance smoke | Fresh Rust perf suites and TypeScript event-storm scenarios remained green in `make ci`. |
| Golden vectors | Rust golden and TypeScript protocol-golden checks passed in `make ci`; VirtualList offset, rich Text, pointer-move, and window-control vectors are covered by current protocol tests. |
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

The current section contains **132** aggregate capability entries: **111 Added**,
**13 Fixed**, and **8 Changed**. These are aggregate statements rather than one
line per commit. `git log --oneline 96bccc1..HEAD` reports **13 commits** in the
refresh delta. The crosswalk retains the earlier aggregated-domain style and
records every commit in this delta below:

| Changelog domain | Reachable evidence |
| --- | --- |
| Protocol v3, fixtures, tap, fuzz, event/property/style foundations | `65c71cc` and reachable protocol checkpoints |
| Renderer surfaces, styles, examples, TypeScript API and tests | `c8a9f18` plus subsequent renderer commits |
| Process/embedded runtime, host CLI, termination and candidate behavior | `062daf7` and host/runtime checkpoints |
| Fast Refresh and headless developer toolkit | `accabbd` |
| CI, release archives, checks, candidate scripts, fixed-SHA workflows | `3a22943` |
| Contracts, ADRs, protocol reference, initial Unreleased inventory | `52863ac` |
| File dialogs and value-tag 5 | `9fab40d` |
| Panic hook, crash reports, exit-code/stderr diagnostics | `bdabd9f` |
| Notifications, static menus, action event, command/event 20/21 vectors | `16595ec` |
| Pending-command lifecycle regression tests | `45faff7` |
| Release-readiness and version-cut preparation | `85f67a8`, `d8619d7`, `2437efa` |
| Direction and error philosophy | `0999576`, `8be3cb2`, `f135c66` |
| Cursor and TextInput placeholder/selection/geometry | `24e493f`, `a3df378`, `d4435c2` |
| Performance and protocol-family refactors | `9a00c86`, `257cff8`, `000917f` |
| Mouse selection and platform key names | `9a13dee`, `589c28b` |
| Citation/example consistency closure | `256fd5b`, `90f0e53` |
| Final changelog convergence | `223b147`, `873789e`, `f473156`, `4dd0129`, `a04c30a`, `ee0fa1b`, `7ab1da1`, `a24bd0d`, `0561ceb`, `d05171f`, `d4b865a`, `c304f29`, `68a4dd5`, `84f678e`, `ee9ad50`, `a77000f`, `436d32f`, `f183876`, `1933b8e`, `576595d`, `dd087b4`, `f8a5079`, `5ca34e1` |
| Selectable Text design, wire flag, host implementation, and review | `2ad4174`, `367f6a2`, `c5ae1ea`, `faecc81` |
| Surface creation options and RTL/base-direction review | `0d8707c`, `242862d`, `c8f4130`, `076418f` |
| Style slot gap matrix | `65ecdcf` |
| Renderer example repairs, native glyph rendering, flex/gap/scroll semantics | `145fba3`, `9efb05e`, `0b19fa2` |
| Gallery redesign, interaction isolation, hitboxes, lint, rendering notes, drag preview | `d7f2fd8`, `b17ea0e`, `97b8b9a`, `bda2e47`, `f7a115f`, `f7abd30` |
| README quick-start and native drag-target wiring | `30206ae`, `1f8361b`, `ab066ec` |
| Focus/overlay events, outside dismissal, multi-click selection, and capability notes | `9846a9a`, `ba583a5`, `66c4c74`, `1577dd6`, `d0b06c8`, `bf4262d`, `7adda08` |
| Cross-platform backend features, CI checks, and support matrix | `9cfce91`, `c25255c`, `e4e86be`, `17044d1` |
| Paint-module decomposition | `2ca99aa` |
| Focused examples, guides, and consumer headless-testing facade | `4b6d42c` |
| Transport fail-fast/typed causes, crash bridge, pending-command EOF, and registry hardening | `49b486a`, `009c795`, `0ab752c`, `66e725b`, `3c3b318` |
| Event-storm performance budgets and audit | `bf4c183`, `29e28b4` |
| Protocol version diagnostics and compatibility documentation | `df0e2c1`, `8dd8999` |
| Pointer click-count normalization | `36e9c0a` |
| Single-form protocol wire cutover | `dd98661` |
| Dev `TestApp` facade | `0c4d2a6` |
| Protocol tap aggregates and identity metadata | `c211078` |
| Reverse transition animation | `a71471f` |
| TextInput clipboard, select-all, and word navigation | `506fdea` |
| VirtualList scroll preservation and behavior contracts | `887711a` |
| Appearance-aware example theming | `06922c3` |
| Citation audit and stable Image rendering | `393e71d`, `bb0d99d` |
| Typed surface lifecycle errors and transport completion seam | `99165f2` |
| Native tooltips, close confirmation, and context-menu policy | `8656202`, `42e1d41` |
| Deterministic gate waits and embedded candidate stability | `0d48ef9` |
| Accessibility expanded/heading-level semantics and validation | `0e51899` |
| Native React tab-stop traversal and focused-unmount restoration | `2157dce`, `c2807b2`, `a87a084` |
| Root text-file commands and notes persistence example | `3ce5a8c`, `f63e783`, `eaf10ff` |
| Data interchange boundaries and clipboard-image commands | `952586b`, `18d0aca` |
| Embedded representative examples startup matrix and Fast Refresh probe | `1c369a9` |
| Pointer-coordinate wire contract, dispatch, examples, and tests | `a097f3c` |
| `33d851e` | Prior release-readiness evidence refresh carried forward as the comparison base for this update. |
| `98a4b7b` | Menu accelerator audit evidence and documented boundary. |
| `91e1042` | Gate timing baseline evidence. |
| `622394c` | TextInput caret-visibility fix and geometry evidence. |
| `2bf5f35` | Final gate timing sample evidence. |
| `33f8929` | Domain vocabulary refresh and ADR-0009–0012 decisions for close confirmation, opt-in event streams, wire tails, and host-owned input models. |
| `b61d3a3` | Host-owned TextInput undo/redo history, coalescing, and ordinary change/selection dispatch. |
| `c7850c5` | Global randomized suite-order independence sweep with zero failures. |
| `346e99b` | VirtualList precise logical-pixel `getScrollOffset()`/`scrollToOffset()` commands and persistence tests. |
| `c7ee8c6` | Detached-worktree 0.2.0 release-prep rehearsal and Makefile `bun-typecheck` → `bun-build` ordering fix. |
| `fc46d01` | Mixed raw/styled Text runs with one-level nesting and typography validation. |
| `655c71a` | Selectable rich-text paragraphs and pointer-activated link runs with `onPress`/link accessibility semantics. |
| `9d17120` | Renderer hotfix restoring eager synchronous `validateStyle` and the validation-throw contract. |

### (b) Protocol directory versus source constants

The current source has exactly **33 command constants** with values `1..33` and
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
  remains the visual Image degradation path.
- TextInput multiline IME candidate placement and all final native geometry,
  AX-tree, clipboard, menu, dialog, zoom, glyph, tooltip, pointer-positioning,
  and drag-visual checks remain display-backed where noted; these are acceptance
  questions, not inferred failures.
- `Root.zoom()` has headless command coverage, but its visual toggle effect
  remains display-backed; `pointerEvents` is intentionally not exposed because
  its partial-occlusion semantics have no native contract.

## 6. Version-cut guidance (not a decision)

The workspace remains `0.1.0`; `release-prep` can synchronize a chosen version
into Cargo and both Bun packages. The current Unreleased inventory is **132
entries (111/13/8)** across protocol, typed surface lifecycle errors, tooltips
and close policy, accessibility semantics, native keyboard focus traversal and
focused-unmount restoration, text-file persistence, clipboard text/images,
embedded representative examples, pointer coordinates, selectable and styled
Text runs with interactive links, host-owned TextInput undo/redo, precise
VirtualList offset persistence, runtime font registration, window controls,
renderer API, native runtime, Image fallback, scale-factor observations,
focus/overlay and drag input, appearance-aware examples, pointer/zoom
boundaries, keybinding registration, outbound file drag, precise TextInput
geometry and host editing, troubleshooting, and long-run stability.
The final matrix reports **130 Core Bun tests / 89,328 assertions**, **20 dev
tests / 50 assertions**, and **114+20 API locks**. These facts support
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
