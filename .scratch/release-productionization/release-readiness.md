# Release-readiness inventory

Generated: 2026-08-27T09:23:29Z
Decision owner: human release owner  
Current HEAD: `06922c3` (`feat(examples): adopt appearance-aware theming`)

This is an evidence package, not a release approval. It records fresh command
runs, the current protocol/API surface, documentation consistency, known
boundaries, and inputs for a human version-cut decision. No artifacts were
published and no push, rebase, version change, or tag was performed.

## Executive readout

- The complete five-gate matrix passed from the final tree at exit code 0:
  `make ci`, `make embedded-bun`, `make host-candidate-smoke`,
  `make host-embedded-candidate-smoke`, and `make bun-pack-smoke`.
- The current candidate remains `0.1.0` on macOS ARM. The release scripts prove
  archive consistency and CLI/runtime behavior, not display-backed GUI behavior.
  Both process and embedded candidate rehearsals passed their expected timeout,
  help, version, and commit/press checks.
- The current Unreleased inventory contains **120 top-level entries**:
  **104 Added**, **9 Fixed**, and **7 Changed**. This is materially beyond a
  patch-sized change set and provides stronger evidence for considering
  `0.2.0`. No option is selected: the human release owner retains the decision.
- `git log --oneline 5ca34e1..HEAD` reports **51 commits** in this refresh delta,
  spanning selectable Text, surface options, glyph/layout/gallery repairs,
  focus/overlay and drag input, cross-platform runner scaffolding, transport
  and protocol hardening, TestApp/examples, event-storm budgets, animation,
  clipboard/word navigation, VirtualList restoration, and appearance-aware
  example theming.
- The implementation backlog for the current 0.2.0 contract is empty. Remaining
  items are documented product/platform boundaries or human release decisions,
  not unrecorded implementation residue: selectable Text is now a host-owned
  visual selection surface, while IME/AX/display checks, native undo/secure
  input, explicit RTL direction, and other upstream gaps remain bounded.
  Multi-click, clipboard, select-all, and word-navigation behavior are covered
  by the current implementation and tests.
`.scratch/release-productionization/v0.2.0-cut-checklist.md` is the companion
local checklist. `.scratch/release-productionization/dependency-audit.md` and
`.scratch/release-productionization/upstream-dependencies.md` are the companion
dependency and upstream boundary notes. `.scratch/` is gitignored by default;
the readiness report and checklist follow the repository's force-add evidence
convention, while the upstream note is already tracked. No artifacts were
published.

## 1. Capability inventory

### Protocol and wire contract

- Protocol v3 uses positional MessagePack Snapshot, Event, Patch, and Command
  frames with explicit surface/epoch/revision/request sequencing.
- The command directory is complete for codes `1..22`: focus/blur/selection,
  VirtualList scrolling, title/window commands, clipboard, surface creation,
  file dialogs, `SHOW_NOTIFICATION=20`, `SET_MENUS=21`, and
  `COMMAND_SET_KEYBINDINGS=22`.
- The event directory is complete for codes `1..22`: TextInput change/selection/
  focus/blur, command results, visible ranges, animation, keyboard, pointer,
  hover, scroll, submit, window lifecycle, action, appearance, layout, drag,
  notification response, and pointer-down-outside.
- Drag uses inbound `[1,type]`, `[2,type]`, and `[3,[path,...]]` notifications;
  outbound file drag uses host-properties tag 4
  `[4,dragType|null,exportFiles|null,acceptsDragOver,acceptsDrop]` with bounded
  1..8 host-local paths. The current wire has one required form; no legacy
  two-slot decoding is retained.
- Keybinding registration is root-only command 22 with full replacement,
  bounded entries/bytes/characters, atomic indexed validation, deterministic
  process-global union, and Event 17 action routing; context-conditional
  bindings remain unsupported.
- Cross-language golden fixtures are generated in both directions. Producer
  bytes and semantic decoding cover current TextInput 13-slot, Image 4-slot,
  Drag 5-slot, CommandResult 7-slot, string Submit, and three-number
  WindowResize forms, along with permitted numeric dual forms, fallback,
  outbound drag, keybinding, and selectable-text vectors.
- Protocol references now point at the five split wire modules and contain zero
  bare `wire.rs` paths. The prior 76-reference P2 citation debt is closed.
- `.scratch/release-productionization/upstream-dependencies.md` is the
  three-tier tracking note for true upstream gaps, project limits that can be
  re-reviewed, and test-platform limitations.

### React renderer and retained tree

- Host kinds are View, Text, Pressable, RawText, TextInput, VirtualList, and
  Image. Retained-tree validation covers ownership, contiguous indexes,
  revisions, listeners, host properties, accessibility, focusability, malformed
  patches, and rollback.
- VirtualList uses one persistent native GPUI variable-height `list` state per
  node: committed rows are measured at natural heights, while uncommitted rows
  use estimated placeholders. `ListState` owns scroll commands and
  reconciliation preserves the prior logical item/offset across item-count
  changes, with clamping at the new end. `estimatedItemSize` is an initial hint;
  the JS-side measurement path was surveyed and rejected as structurally
  infeasible, and placeholder flicker remains display-backed.
- Image supports an optional `fallbackSource` in its tag-3 four-slot host
  payload; GPUI `with_loading` and `with_fallback` share the visual fallback
  path, while JavaScript `onError` remains a true upstream gap.
- The public Root surface includes render/unmount, title/resize/zoom/fullscreen,
  focus traversal, URL opening, clipboard, window-size query, multi-surface
  opening, asynchronous file dialogs, notifications, static menus, and the
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
  Host-owned Cmd/Ctrl-C/X/V clipboard editing, Cmd/Ctrl-A selection, and
  Option/Alt word-wise navigation with Shift extension are supported. Placeholder
  layout falls back to element bounds, while multiline IME candidate placement
  remains display-backed.
- Mouse interaction covers click-to-caret, drag selection, anchor/head/reversed
  orientation, selection highlighting, Shift navigation, Home/End, Up/Down,
  platform Arrow key names, UAX #29 double-click word selection, and
  triple-click logical-line selection without changing the wire contract.
- Pointer, hover, scroll, keyboard, focus/blur, press, submit, window resize,
  window activation, menu action, layout, drag, and pointer-down-outside
  dispatch are covered by protocol and renderer tests. The caret remains always
  visible for accessibility; multi-click granularity is no longer backlog.
- `pointerEvents` remains intentionally unexposed; normal GPUI hitboxes permit
  basic pass-through when no listener is installed, while zoom's headless
  command path is covered but its visual effect remains display-backed.
- Read-only Text selection is implemented through `TextProps.selectable`: the
  host owns anchor/head state, paints native-shaped per-row highlights, and
  supports host-side Cmd/Ctrl-C clipboard copy without a JavaScript selection
  event or mirrored React state. The optional Text-only wire tail and native
  cursor/clipboard behavior are covered headlessly, with final visual checks
  still display-backed.
- Window resize observations carry the required
  `[width,height,scaleFactor]` form and report scale-only changes through the
  same coalesced observer path. TypeScript callbacks and `WindowSize` stores
  expose the normalized scale factor; no legacy two-number frame is accepted.

### Runtime and host delivery

- ProcessAdapter uses framed stdin/stdout with a bounded ordered event writer,
  explicit shutdown/EOF/failure states, retained failure reporting, and
  non-sensitive wire identity in fatal diagnostics. Transport termination
  exposes typed causes and malformed host event frames fail fast while
  rejecting pending commands.
- EmbeddedBunAdapter runs pinned Bun/JSC on a dedicated runtime thread with a
  bounded callback bridge and Fast Refresh lifecycle serialization.
- SurfaceHost and the GPUI SurfaceRegistry demultiplex multiple roots/windows;
  final-window teardown, unknown-surface rejection, and reentrant renderer-death
  handling are explicit. A fatal shared-runtime failure closes all registered
  surfaces.
- Host CLI options, `--version`, `--help`, process/embedded selection, renderer
  argument forwarding, and expected timeout behavior are exercised by candidate
  scripts.
- The host panic hook preserves the standard stderr hook, writes a versioned
  report under `${REACT_GPUI_CRASH_DIR:-system temp}`, and prints its stable
  crash-report path on fatal exit. TypeScript termination errors retain an
  integer host exit code, the last 50 stderr lines, typed causes, and the path
  when the transport cause supplies them.

### Developer and distribution tooling

- Both Bun packages build ESM and declaration artifacts, expose built package
  exports, carry license text, and pass an external tarball consumer smoke.
  Current API locks are **core 110 / dev 20** name-and-kind exports.
- The core README indexes twelve examples: counter, gallery, todo, keyboard,
  text-input, VirtualList, selectable-text, stress, focus-flow, dropdown,
  drag-reorder, and multi-surface. These examples cover appearance bridges,
  onLayout, drag handlers, selectable Text, reversed selection, clipboard and
  word navigation, setSelection, and scrollToIndex/scrollToEnd. `@react-gpui/dev`
  also provides the behavior-level `renderTestApp` facade.
- The troubleshooting guide provides a 467-line, eight-symptom
  symptom→diagnosis→repair entry point, while README Debugging retains the
  environment quick reference.
- Makefile gates, fixed-SHA workflows, release archive/checksum validation,
  process and embedded candidate rehearsals, release-prep synchronization,
  ADR-0008 error ownership, the consumer getting-started guide, and the
  cross-platform matrix workflow are present. The matrix is scaffolded at
  `.github/workflows/cross-platform.yml` and remains runner-only until push/PR
  execution.

## 2. Fresh five-gate evidence

All times below are fresh wall-clock `real` values from `/usr/bin/time -p` on
this tree. Every command exited 0. Expected timeout lines in candidate scripts
are part of the successful rehearsal contract, not command failures.

| Command | Exit | Real time | Fresh observed evidence |
| --- | ---: | ---: | --- |
| `make ci` | 0 | **59.25 s** | Rust: `react_gpui` **96**, frame peer 0, module-boundary **6**, perf **3**, perf-event-storm **1**, process-roundtrip **3**, protocol-fuzz **1**, protocol-golden **3**, Bun lib 0, embedded counter 0, host **12**, command-roundtrip **17**, examples-render **25**, glyph-platform **2**, doctests 0+0 = **169 passing tests**. Core Bun: **108 tests / 53,212 assertions**; dev Bun: **18 tests / 46 assertions**. Format/check/clippy/typecheck/build/package smoke all passed.
| `make embedded-bun` | 0 | **2.71 s** | Host embedded feature check passed; `react-gpui-bun` embedded counter **1 passed**; library/doc suites had 0 tests and no failures.
| `make host-candidate-smoke` | 0 | **13.05 s** | Release archive and extracted-host checks passed; identical archive SHA-256 `4c7dd6c7fdc46a2f702c2673a6f3de5cc83e91140286411d038ffe369bc43232`; README/LICENSE checks passed; snapshot commit **312 bytes** observed; process error/info rehearsals timed out at **5.013/5.015 s** as expected; version/help passed.
| `make host-embedded-candidate-smoke` | 0 | **5.68 s** | Release embedded binary passed; `--smoke-press` reported `sent=true, commits=1, status=None`; embedded rehearsal timed out at **5.013 s** as expected; version/help passed.
| `make bun-pack-smoke` | 0 | **2.63 s** | Frozen installs, JS/type builds, both tarballs, external consumer install, and `package tarball consumer smoke passed`; consumer installed **59 packages**.

The measured sequential five-gate wall-time sum was **83.32 s**. The serial
protocol golden generator completed in **0.97 s**, and the serial API surface
generator completed in **2.13 s** on its first pass and **2.06 s** on its
second pass; both generator runs left their checked-in fixtures unchanged.

## 3. Quality evidence matrix

| Area | Current evidence |
| --- | --- |
| Rust workspace | Fresh `make ci`: **169 passing tests**, 0 failures; `react_gpui` lib **96**; command-roundtrip **17**; examples-render **25**; glyph-platform **2**; module-boundary guard **6/6**; host panic-hook suite **12/12**. |
| TypeScript renderer | Fresh `make ci`: **108/108 tests**, **53,212 assertions**, 0 failures, including the 50k stress invariant and event-storm scenarios. |
| Development package | Fresh `make ci`: **18/18 tests**, **46 assertions**, expected malformed-source diagnostics contained and last-good tree preserved. |
| Fuzz / malformed input | Rust deterministic protocol-fuzz **1/1** (1,039 mutation cases) and the TypeScript malformed-input coverage remain green; no panic or uncaught decoder exception observed. |
| Performance smoke | Fresh Rust perf suites **3/3** plus perf-event-storm **1/1**. Final TypeScript hot paths: surface route 2 **6.380 ms / 2,000 events** (budget 60), route 8 **12.912 ms / 8,000** (budget 100), press **3.046 ms / 2,000** (budget 15), scroll **2.006 ms / 2,000** (budget 30), drag **1.930 ms / 2,000** (budget 35). The final event-storm suite covered 60/120/240 Hz scroll and drag paths, 10,000-node trees, bursts, no-op, layout, and visible-range cases. |
| Golden vectors | Rust golden **3/3**; focused TypeScript golden **4/4**, **275 assertions**, with current Image fallback, scale-factor, selectable-text, keybinding, and outbound-drag vectors, 0 fixture drift. |
| API surface locks | Checked-in name/kind fixtures currently lock **core 110 / dev 20** exports. |
| Protocol tap / crash diagnostics | Tap aggregate tests remain in the fresh Rust/Bun suites; host panic-hook, typed transport-cause, crash-path, and termination evidence remain green in the fresh matrix. |
| Release packaging | Process and embedded candidate scripts, deterministic archive checks, package tarball consumer smoke, and embedded package gate all passed. Checksums establish reproducibility, not publisher authenticity. |

## 4. Consistency and crosswalk audit

### (a) Current Unreleased inventory

The current section contains **120** aggregate capability entries: **104 Added**,
**9 Fixed**, and **7 Changed**. These are aggregate statements rather than one
line per commit. `git log --oneline 5ca34e1..HEAD` reports **51 commits** in the
refresh delta; the latest reachable implementation/documentation crosswalk is:

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

### (b) Protocol directory versus source constants

The current source has exactly **22 command constants** with values `1..22` and
**22 semantic event constants** with values `1..22` (pointer/key sub-action and
button constants excluded). The protocol reference has matching event/command
rows, a **42-slot** style tuple with `boxShadow`/`fontFamily` tails, and one
current wire form for TextInput (13 slots), Image (4), Drag (5), CommandResult
(7), Submit (string), and WindowResize (3). OpenSurface retains its valid
no-options two-item form plus the optional creation-options tuple; other
historical compatibility forms were removed by the cutover. The protocol
citation follow-up mapped the 75 historical references to the five split wire
files; no bare `wire.rs` path remains.
- Selectable read-only Text is in the current wire as a Text-only optional tail;
  the host owns anchor/head, geometry, highlighting, and clipboard copy without
  adding a JavaScript event or mirrored selection state.

### (c) README and consumer-guide capability audit

Root/package README claims were checked against the current Root and TypeScript
interfaces. The named Root calls (`render`, `openSurface`, `pickFiles`,
`pickSavePath`, `showNotification`, `setMenus`, `setKeybindings`, and related
close/action/window callbacks) exist. The package README and getting-started
guide now state placeholder visual-only behavior, `fallbackSource` loading/error
visuals, scale-factor resize observations, UTF-16 selection, precise
single- and multiline geometry, selectable Text, clipboard/select-all/word
navigation, multi-click, outbound file drag limits, keybinding replacement
limits, the platform support matrix, and the troubleshooting guide's symptom
workflow.
- A factual documentation drift remains in the root README Status paragraph:
  it still says Ctrl/Cmd word-boundary movement and double-/triple-click
  selection are outside the minimal interaction contract, although the current
  implementation and package README support them. This evidence refresh does
  not edit source or product documentation.

### (d) ADR alignment

ADR-0008's four failure seams remain aligned with implementation: consumer
Error-Boundary ownership for render failures, fail-fast protocol/tree rejection,
local blank-image degradation, and host-fatal GPUI paint panic. Shared-runtime
fatal errors close every registered surface. ADR-0004/0005/0006/0007 and the
new TextInput geometry/selection behavior agree with the protocol reference.

## 5. Delivery chain and checkpoint history

Latest reachable productionization checkpoints, oldest to newest in this final
refresh window:

1. `d4435c2` — precise single-line TextInput geometry with cache invalidation and
   honest multiline/placeholder fallback.
2. `90f0e53` — pre-release consistency sweep and count/slot documentation fixes.
3. `873789e` — geometry and consistency changelog convergence.
4. `9a13dee` — mouse caret placement, drag selection, highlighting, and keyboard
   navigation semantics.
5. `256fd5b` — protocol citation debt mapping and six-example coverage.
6. `589c28b` — platform Arrow/Home/End key-name follow-up tests.
7. `f473156` — input interaction and citation-debt changelog closure.
8. `4dd0129` — command roundtrip coverage and clipboard validator documentation.
9. `c4522a8` — 50k long-run stability invariants.
10. `a04c30a` — dialog and long-run coverage changelog convergence.
11. `ee0fa1b` — native GPUI variable-height VirtualList migration with per-node
    ListState, measured committed rows, estimated placeholders, and native
    ListState scroll commands.
12. `7ab1da1` — native VirtualList migration changelog entry.
13. `0561ceb` — Image `fallbackSource` and scale-factor resize observation
    support, including compatibility, hooks, observer batching, and golden
    vectors.
14. `d4b865a` — pointer-events and zoom dispositions, preserving explicit
    unsupported/display-backed boundaries.
15. `c304f29` — final pre-cut audit repairs for stale image ADR wording, exact
    BoxShadow API documentation, and Makefile phony gate declaration.
16. `68a4dd5` — pre-cut audit changelog convergence.
17. `84f678e` — precise multiline TextInput geometry with cached wrapped
    layout and UTF-16 line mapping.
18. `ee9ad50` — consumer troubleshooting guide and README debugging pointer.
19. `a77000f` — troubleshooting IME wording aligned with multiline geometry.
20. `436d32f` — multiline/troubleshooting changelog convergence.
21. `f183876` — root-only keybinding registration with Event 17 routing.
22. `1933b8e` — keybinding registration changelog entry.
23. `576595d` — constrained outbound file drag with tag-4 host properties.
24. `dd087b4` — outbound file drag changelog entry.
25. `f8a5079` — selectable Text feasibility re-review archived as a
    feasible-bounded deferred design.
26. `5ca34e1` — protocol reference synchronization for Event 21,
    Command 22, outbound drag, and precise multiline geometry.
27. `2ad4174`/`367f6a2`/`c5ae1ea`/`faecc81` — selectable Text review, wire flag,
    host selection implementation, and documentation.
28. `0d8707c`/`242862d`/`c8f4130`/`076418f` — RTL gap classification and surface
    creation options.
29. `65ecdcf` — style slot gap matrix.
30. `145fba3`/`9efb05e`/`0b19fa2` — example rendering, native glyph, flex/gap,
    and scroll repairs.
31. `d7f2fd8` through `f7abd30` — gallery redesign, interaction/state repairs,
    scene checks, documentation, and compact drag preview.
32. `30206ae`/`1f8361b`/`ab066ec` — README quick-start and drag-target wiring.
33. `9846a9a` through `7adda08` — focus/overlay events, outside dismissal,
    multi-click selection, and capability notes.
34. `9cfce91`/`c25255c`/`e4e86be`/`17044d1` — cross-platform backend features,
    CI coverage, and platform support matrix.
35. `2ca99aa` — paint-module decomposition.
36. `4b6d42c` — focused capability examples, guides, and consumer testing.
37. `49b486a`/`009c795`/`0ab752c`/`66e725b`/`3c3b318` — transport fail-fast,
    typed causes, crash-report bridge, EOF coverage, and registry hardening.
38. `bf4c183`/`29e28b4` — event-storm budgets and audit.
39. `df0e2c1`/`8dd8999` — protocol version diagnostics and compatibility docs.
40. `36e9c0a` — pointer click-count normalization.
41. `dd98661` — single-form wire cutover.
42. `0c4d2a6` — dev TestApp facade.
43. `c211078` — tap aggregates and protocol identity.
44. `a71471f` — reverse transition animation.
45. `506fdea` — TextInput clipboard, select-all, and word navigation.
46. `887711a` — VirtualList scroll preservation and behavior contracts.
47. `06922c3` — appearance-aware example theming.

Earlier readiness checkpoints remain reachable through `d8619d7`; this list is
an additive refresh, not a claim that older history was rewritten.

## 6. Known limits and human decision items

The implementation backlog for the current 0.2.0 contract is empty after the
current input/geometry/image/scale/pointer/zoom/keybinding/drag/selectable-text
review. Selectable Text is now implemented as a host-owned visual surface; the
following are explicit acceptance boundaries or human decisions, not hidden
TODOs:
- Issue 01 remains `needs-triage`: the candidate is unsigned and not notarized;
  checksums do not authenticate the publisher.
- Issue 02 remains `ready-for-human`: `block 0.1.6` emits a future-incompatibility
  warning and no unreviewed fork should be patched automatically.
- Issue 03 remains `ready-for-human`: embedded source build is separate,
  green, expensive, and outside ordinary `make ci`.
- Issue 04 remains `needs-triage`: Quartz dialogs, clipboard/menu/window
  behavior, glyph pixels, native drag visuals, and AX-tree behavior require a
  real display/hardware runner; headless evidence cannot replace it.
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
  AX-tree, clipboard, menu, dialog, zoom, glyph, and drag-visual checks remain
  display-backed where noted; these are acceptance questions, not inferred
  failures.
- A prior full-CI attempt timed out in the unrelated
  `tests::transport::process_event_writer_kills_closed_stdin_child_for_reader_eof`
  test; two immediate isolated reruns passed and no transport files changed.
  The final fresh `make ci` on this tree passed, so this is a flake observation,
  not a current gate failure.
- `Root.zoom()` has headless command coverage, but its visual toggle effect
  remains display-backed; `pointerEvents` is intentionally not exposed because
  its partial-occlusion semantics have no native contract.

## 7. Version-cut guidance (not a decision)

The workspace remains `0.1.0`; `release-prep` can synchronize a chosen version
into Cargo and both Bun packages. The current Unreleased inventory is **120
entries (104/9/7)** across protocol, selectable Text, renderer API, native
runtime, VirtualList native variable-height layout and scroll restoration,
Image fallback, scale-factor observations, focus/overlay and drag input,
appearance-aware examples, pointer/zoom boundaries, keybinding registration,
outbound file drag, precise TextInput geometry and host editing, troubleshooting,
and long-run stability. The fresh matrix reports **169 Rust tests**, **108+18
Bun tests**, **275 focused golden assertions**, and **110+20 API locks**. These
facts support considering `0.2.0` rather than a patch cut, but no version option
is selected by this inventory.

The companion checklist is
`.scratch/release-productionization/v0.2.0-cut-checklist.md`. Its five human
decisions and seven-step mechanical sequence agree with this report's gate
status. One mechanical caveat remains: the current `release-prep.sh` changelog
guard requires an exact `## [0.2.0]` heading and does not accept the dated
`## [0.2.0] - YYYY-MM-DD` form without a small guard update or temporary exact
heading. The checklist records that caveat and keeps the final decision with the
human release owner.

No option is selected by this inventory.
