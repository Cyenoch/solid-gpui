# Solid GPUI alternatives: primary-source comparison

Research date: **2026-10-01**. Scope: `heyhuynhgiabuu/solid-gpui`, `lxsmnsyc/solid-gpui`, and our current `Cyenoch/solid-gpui` checkout. This is a source and evidence assessment, **not a performance shootout**. No dependency installation, compilation, test execution, application launch, or downloaded script execution was performed.

## Executive assessment

- **Our strongest decisions are the ownership and correctness boundaries:** Solid owns composition, native code owns synchronous editing and rendering, commits are transactional, callbacks are revision-aware, runtime queues have bounds, and native modules have explicit generated contracts. Both alternatives validate the basic Solid-universal-renderer plus GPUI-host split, but neither inspected host provides our complete publication/event/extension model. [O1] [O2] [O3] [O4] [O5] [H1] [L1]
- **`lxsmnsyc` has the cleanest small application surface:** a direct `render`, an independently exported compiler, CSS-like style normalization, reactive recorded canvas, native input geometry, and platform-binary package plumbing. Its simpler implementation is easier to understand, but its singleton session, permissive operation application, unbounded transport queue, and current-handler event routing are weaker foundations for our requirements. [L1] [L2] [L3] [L4] [L5] [L6] [L7] [L8]
- **`heyhuynhgiabuu` is more evidence-conscious than a README-only reading suggests:** correlated apply errors, explicit poison/recovery policy, Node/Bun supervision, cross-language fixtures, real-window tests, headless lifecycle measurements, and hosted GUI jobs exist. However, native editing is materially less complete than ours or `lxsmnsyc`'s, lists retain all children, and some advertised utility/style mappings do not survive the actual Rust mapper. [H2] [H3] [H4] [H5] [H6] [H7] [H8] [H9] [H36] [H37]
- **Our clearest mistake is making the simplest consumer pay for the advanced integration architecture.** A trivial app currently inherits SDK checkout, unpublished Cargo-crate, workspace patch/profile, generated-binding, and runtime/tooling requirements. A prebuilt default host and a standalone scaffold would improve adoption without weakening the protocol or replacing the custom native-module path. Our registry package is publicly available; this is a setup-cost criticism, not a claim that npm publication is missing. [O6] [O7] [L8] [H10] [M5]
- **Adopt ergonomics and evidence practices, not their state/protocol shortcuts.** Priorities: simple consumer path; compiler-only export; optional style normalization; a bounded recorded-paint native component if actual consumers need it. Keep transactional commits, explicit native ownership, controlled-edit acknowledgements, multi-surface routing, and data-identity-aware virtualization. [O2] [O3] [O4] [O8] [O9] [L2] [L3] [L5]

## 1. Pinned identities, versions, and observation dates

### Repository metadata

The following GitHub API observations were made on **2026-10-01**. `pushed_at` is repository metadata, not necessarily the default branch's commit time. Neither remote repository is marked archived or disabled; both are marked `fork: false`, which does **not** establish independent authorship of every source file. [M1] [M2]

| Project | Inspected default-branch HEAD | Commit time (UTC) | Default branch | Last push (UTC) | Archived | Source license |
| --- | --- | --- | --- | --- | --- | --- |
| `heyhuynhgiabuu/solid-gpui` | `05f84e9f17cd8360b9af92f615a884527fa3a986` | 2026-09-11 09:24:23 | `main` | 2026-09-11 09:24:29 | No | Apache-2.0; MIT-derived Markdown subsystem has attribution [H11] [H12] |
| `lxsmnsyc/solid-gpui` | `196aa6edc779cb39f37a3ade4517ed197ad58813` | 2026-08-24 14:55:02 | `main` | 2026-08-24 14:55:05 | No | MIT [L8] [L9] |
| Our checkout, `Cyenoch/solid-gpui` | `e9bc4389c5394e5de1fecb6758d2d06e48877fdb` | 2026-09-29 10:23:21 | Local comparison baseline, not a fresh remote HEAD claim | Not queried here | Not queried here | MIT project; vendored GPUI retains Apache-2.0 [O7] [O10] |

Remote projects were cloned shallowly, without submodule initialization, into unique new directories:

```text
/private/var/folders/sl/42r5kc756mj96rlxkglrpr900000gn/T/opencode/solid-alternatives-20261001.VvobOF/heyhuynhgiabuu
/private/var/folders/sl/42r5kc756mj96rlxkglrpr900000gn/T/opencode/solid-alternatives-20261001.VvobOF/lxsmnsyc
```

They were left in place. No existing files were removed. The local checkout was clean at the initial identity check; concurrent research notes are outside the source baseline.

### Dependency and delivery identities

| Boundary | `heyhuynhgiabuu` | `lxsmnsyc` | Ours |
| --- | --- | --- | --- |
| Package/crate version | `0.1.0` [H11] [H13] | `0.1.0` [L8] [L9] | `0.5.2` [O7] [O11] |
| Solid runtime | `2.0.0-rc.7`, exact runtime/universal pins [H13] [H14] | `2.0.0-rc.1` in lockfile/dev setup; wider prerelease peer/dependency ranges [L8] [L10] | `1.9.15` workspace; core peer `^1.9.10` [O11] [O12] |
| JSX compiler | Babel `@solidjs/babel-plugin 2.0.0-rc.7`, universal mode [H14] [H15] | `@dom-expressions/compiler 0.50.0-next.43`, universal mode [L2] [L10] | Official `@solidjs/compiler 2.0.0-rc.9`, followed by Oxc type erasure/source-map composition [O13] [O14] |
| GPUI | Git Zed `35aab214c2df2aae8c4c173965aad0520b6823de`, lockfile package `0.2.2`; Cargo manifests do not specify `rev` [H16] [H17] | Git Zed `d9ad6aff67e47de43abb270d22de75dd950f1b48`, explicit `rev` [L9] | Vendored/patched `gpui-pre 0.3.7`; manifest describes upstream snapshot `zed@1a28cff` [O7] [O10] |
| Runtime topology | Node/Bun application spawns a one-window Rust helper [H3] [H18] | Node/Bun application spawns one Rust host; singleton renderer session [L4] [L6] | Runtime-independent byte seam; process Bun, worker-owned QuickJS, optional Embedded Bun [O15] [O17] [O42] |
| Wire | NDJSON tagged mutations, version `1`, apply acknowledgements [H3] [H40] | NDJSON positional op arrays, hand-maintained parser; shown messages have no protocol-version/surface/epoch header [L11] | Framed Bebop v6, schema digest lock, generated bindings and structural guard [O18] [O19] |

**Availability check, not inferred from workflows:** GitHub's Releases API returned no releases for either alternative on the research date. The public npm registry returned 404 for `solid-gpui`, `@solid-gpui/solid`, and `@solid-gpui/client`; the latter two are documented as restricted/private, so 404 is **not evidence that an authorized user cannot install them**. `@solid-gpui/core` returned 200, `latest: 0.5.2`, publication time `2026-09-29T10:28:33.505Z`. We did not download or execute any tarball. [M3] [M4] [M5] [M8] [M9] [M10] [H10]

## 2. Architecture comparison at the actual seams

| Concern | `heyhuynhgiabuu` | `lxsmnsyc` | Our relevant implementation |
| --- | --- | --- | --- |
| Solid integration | Per-renderer `createRenderer`; shadow bookkeeping and manually pumped `flushSolid`/ack rounds; JSX delegates through a module-global suite [H1] [H19] [H24] | Module-global `createRenderer` plus singleton `Session`; microtask batching and Solid flush after each event [L1] [L6] | Module-global universal primitives resolve a root-owned `HostTree`; dirty props finalize once per transaction [O1] [O2] [O20] |
| Publication | Mutations applied one by one; a later failure leaves earlier mutations applied; client poisons further flushes [H2] [H4] [H24] | One parsed batch applied operation by operation to the live tree; missing-node operations often no-op; one root notify [L7] [L12] | JS host-graph journal, Snapshot/Patch revision chain, Rust rollback journal and extension validation before publication [O2] [O3] [O21] |
| Foreground/native ownership | GPUI main thread owns `HostView`; IO thread waits for a reply per line; input/focus/list/cache maps retained per node [H35] [H22] | GPUI main thread owns tree/root; stdin reader uses unbounded channel; native input is its own entity [L7] [L12] [L13] | `NativeStateRegistry` plus bounded `CommitPump`; RuntimeAdapter carries owned bytes, not JS/native objects [O4] [O15] [O43] |
| Callback identity | Lookup by element ID and event name against the current JS registry [H18] | Lookup by node ID/event name against current `node.handlers` [L6] | Surface + epoch + ordered sequence + revision; current/previous callback generations [O5] [O23] [O24] |
| Editing | UTF-16 text/selection/marked state; no candidate bounds/hit testing in `InputHandler`; displayed caret at end; controlled push resets selection [H6] [H20] [H21] | Shaped/wrapped text, native selection painting, grapheme motion, IME bounds and hit testing; equal-value echo is ignored [L13] [L14] | Native editing/history/scroll/geometry, grapheme tests, controlled edit-sequence acknowledgement and composition guard [O8] [O25] [O26] |
| Virtualization | Retain every Solid/native child; GPUI List builds visible items [H9] [H22] | Logical count + committed row window, range feedback, placeholders, ListState/UniformList handles [L15] | Only row window owns Solid/native nodes; revision-chained replacement spans preserve other measured rows and scroll anchor [O9] [O27] [O44] |
| Extensibility | Closed helper command match; no general native module/component registry found in inspected source [H35] [H32] [H18] | Closed native command match; transport injection and compiler export, but no general native component registry found [L2] [L16] | NativeModule + NativeView/element contracts, generated SDK, adapter registry with exact catalog identity [O16] [O28] [O29] |
| Development | Bun preload compiles TSX; `--hot` example updates the same handle/window, disposing/remounting Solid [H15] [H18] [H23] | Vite plugin compiles/configures export conditions; examples build then launch Node; no persistent-host HMR implementation found in inspected sources [L17] [L18] | Managed Vite environment, Rust rebuild/session replacement, staged candidate commits, explicit captured state, epoch handoff [O14] [O30] [O31] |

All three have incremental **host mutations**, but this does not mean every native paint is restricted to the mutated node. Our `SolidRoot::render`, `lxsmnsyc`'s `Root::render`, and `heyhuynhgiabuu`'s `HostView::render` all enter their retained root and construct GPUI elements. Native entities, lists, and caches have more specific paths; do not equate whole-root entry with painting every logical list item or with a measured bottleneck. [O32] [L27] [H5]

## 3. `heyhuynhgiabuu/solid-gpui`

### What it does well

1. **A small, inspectable application/transport split.** `render`/`mount` bind renderer mutations to `sendBatch`, route events, and expose window/dialog/shell helpers. The client supervises a helper process, waits for stdio to drain on `close`, correlates requests, rejects duplicate in-flight sequence IDs, and rejects pending work when the process closes. Those are implemented policies, not just diagram arrows. The standard mounted JSX path is nevertheless tied to one global suite. [H3] [H18] [H19]
2. **Honest failure semantics.** Window-mode apply returns how many mutations succeeded before failure; tests assert a failure after one successful mutation. The renderer never blindly retries that batch: later flushes reject as poisoned, and documented recovery requires `resetTree` plus a fresh renderer or a replacement process. This is safer than silently continuing a divergent tree, though inferior to transactional publication for our needs. [H2] [H4] [H24] [H41]
3. **A pragmatic authoring bridge.** TSX goes through the Solid universal compiler, not the React automatic factory; `h()` is a compiler-free path for small probes. The reactivity canary executes before spawning, so SSR resolution failure need not leak a process. Utility classes compile into existing style/state mutations, with explicit-over-class precedence and diagnostics for unknown utilities. That is a useful pattern for adding sugar without redesigning the wire. [H15] [H18] [H25] [H26] [H42]
4. **Domain-sized native work.** Markdown arrives as a source string rather than per-block mutation traffic. Rust parses/highlights it, caches by exact source, and resolves fences by language **and content**. Tests cover distinct same-language fences so cached syntax runs do not bleed between documents. A native whole-document component can be a deeper abstraction than exporting every parser node to JS. This does not prove the synchronous build/highlight path is cheap for large documents. [H12] [H27]
5. **Useful correctness evidence and lifecycle observability.** Source includes real-window apply/error tests, deterministic GPUI render/input/overlay tests, and lifecycle benchmarks that count retained nodes and side maps after unique-ID teardown. The lifecycle harness asserts state-map emptiness rather than relying only on RSS. [H4] [H28] [H38] [H39]
6. **Performance reporting with named exclusions—and a boundary-label caveat.** The 2026-08-28 report separates Solid creation, protocol, stdio, TestApp draw, and lifecycle, and records a rejected compact JSON experiment. But the source shows `benchmark:consumer` and `benchmark:stdio` use transport-only mode: `run_stdio` decodes and acknowledges counts **without any retained tree apply**. Their included-boundary labels say “decode/apply/ack,” which overstates those numbers. These are signal/serialization/pipe/decode/ack latencies, not native UI-update or input-to-present measurements. The compact candidate also includes JS row conversion before stringify and intentionally lighter candidate validation; its result cannot rank all compact or binary encoders. A separate window-mode perf test asserts a 10 ms p95 **retained-tree build** budget, skips with missing helper/`SOLID_GPUI_SKIP_GUI_TESTS`, and does not measure complete draw/presentation. Their separation of real paint from headless work is useful; preserve it and tighten the labels. [H8] [H43] [H44] [H45] [H46] [H53]
7. **There is actual hosted GUI evidence.** At pinned HEAD the GitHub API reports successful macOS, Windows, and Linux/Xvfb GUI jobs, plus Rust and Node smoke jobs. This is stronger than “builds on three OSes.” The workflow still sets `continue-on-error` on GUI evidence, and success of those smokes is not signing, clean-machine delivery, all-widget correctness, or physical IME acceptance. [H7] [M6]

### What is weaker, wrong, or easy to overstate

- **Batch atomicity is not implemented.** Acknowledgement correlation does not undo mutations. Window-mode applies directly to `view.tree`, stopping at the first error, and the client poisons itself afterward. Our atomic candidate/journal design prevents publication of this partial host tree. Our fatal runtime policy still means rollback is a correctness boundary, not automatic recovery from arbitrary protocol rejection. [H2] [H4] [O3] [O15]
- **The text input is not a complete native editor.** It stores UTF-16 caret/anchor/marked state, but `bounds_for_range` and `character_index_for_point` return `None`; rendering draws text plus an end-of-value caret, not a selection/caret at the actual edit position. Textarea height uses logical line count rather than wrapped geometry. `set_input_value` unconditionally moves caret to the end and clears anchor/composition. Even an ordinary controlled echo can therefore lose transient editing state. Comments referring to native “undo” are not proof of a history implementation: the inspected state, InputHandler, and key/render paths contain no undo/redo history or actions. [H6] [H20] [H21]
- **Utility docs/types exceed the mapper in concrete places.** `font-*` compiles to numeric `fontWeight`, and the StyleKey union admits that property, but the generic Rust `apply_style` match has no `fontWeight` arm. Styled **text runs** do apply a run weight; that is a different path. Similarly, bracket `%` produces a percent string, while generic width/height use `parse_px`, which only strips `px` and parses a number. Do not transplant the claim that every recognized utility is natively applied. These are static producer→consumer mismatches, not executed reproductions. [H29] [H30] [H31] [H47] [H48] [H49]
- **Virtualized paint is not virtualized ownership.** The List retains all children, clones the child-ID baseline, and compares prefix/suffix to splice measured state. Rendering only visible rows is valuable, but it does not bound initial Solid owners, wire creation, native nodes, or per-frame list-child reconciliation to the viewport. Our core VirtualList already solves a larger portion of that problem. [H9] [H22] [O9] [O27] [O44]
- **Events lack the revision identity we rely on.** JS fetches the current handler by ID/name when an event arrives. An event emitted before a callback change is not associated with the callback generation that produced its native frame. This is a source-visible hazard for in-flight events; it was not independently reproduced here. [H18] [O5] [O24]
- **IO and command scheduling have tradeoffs.** Native event emission writes/flushes stdout synchronously, and the stdin thread waits for each job's reply. Awaiting a dialog in that ordered loop also queues subsequent mutation batches behind the unanswered dialog. Native event loop responsiveness is a separate question from progress of JS updates through this command channel. No frame/line admission limit or drain-aware JS write policy is visible in the inspected reader/client paths. [H35] [H3] [H32] [H50]
- **The supported consumer/compiler story is less integrated than ours.** The TSX preload is a repository script; package exports cover core/JSX runtimes rather than a compiler integration. Bun HMR example globals and `update` reuse a live window but dispose/remount its state; this is not automatic per-component state preservation. Node ≥20 is supported by code and a CI smoke, but default export conditions are still an explicit user responsibility. [H13] [H15] [H18] [H23]
- **Shipping is not proven by the release workflow alone.** Platform npm packages are assembled/published before JS packages and there is artifact-backed Node smoke. Nevertheless, public metadata shows no GitHub Releases, public registry lookup cannot establish restricted package access, and packaging docs explicitly leave signing/clean-machine and some platform application packaging work open. The README's platform status and ROADMAP's recorded GUI successes are not fully synchronized. [H10] [H33] [H34] [H51] [H52] [M3] [M9] [M10] [M6]

### Ideas worth adopting from this project

- Adopt the **closed, diagnosed utility subset** idea only after every token is traced through our validated Style producer to the native consumer. Prefer a separate authoring helper/compiler, explicit merge precedence, no CSS cascade promises, and no silent unknown keys. Do not copy the mapper gaps above. [H26] [O33]
- Use **domain-sized native components** for expensive or synchronous-native domains when the consumer needs them; our NativeView contract is the correct home. Markdown/text parsing placement should be measured against our existing JS/build-time approach, not assumed faster because it is Rust. [H12] [O28]
- Preserve their **measurement decomposition and negative results**, but correct the transport-only benchmark labels and compare equivalent validation/work. Add operation-count/correctness signals and teardown state counts; our existing soak tests already cover side-map and undo bounds, so extend rather than duplicate them. [H8] [H28] [H43] [H45] [H46] [O34]
- Keep **version-paired binary packages and package-before-client ordering** in any proposed prebuilt-host distribution. Borrow the contract, not the private npm visibility or global sequence-number ranges. [H3] [H33]

## 4. `lxsmnsyc/solid-gpui`

### What it does well

1. **A cohesive minimal renderer.** Universal callbacks update a local node graph and queue wire ops; stable native IDs are preserved during reordering. Scalar no-op text/property updates are suppressed. Tests compile real TSX against a fake host and verify signals, listener dispatch, keyed reorders without recreation, and unreachable-subtree drops. This is meaningful renderer evidence even though it is not native-window evidence. [L1] [L19]
2. **Compiler as a reusable module, not a Vite-only implementation detail.** `compileOptions`, synchronous `compile`, and `compileAsync` target universal output. The Vite plugin consumes the same function and configures browser/client resolution. The package exports `/compiler` separately. This is a concrete portability advantage over our currently internal transform, independent of whether we should adopt their older compiler version. [L2] [L8] [L17] [O13]
3. **Expressive, normalized styles.** Authors get px/rem/percent/auto lengths, X/Y shorthands, independent overflow axes, explicit flex basis, aspect ratio, text run details, and native hover/active/group styles. The wire length tagged union keeps units explicit, and Rust maps normalized data onto StyleRefinement. Parsed style state is retained on node updates instead of reparsed from JSON each repaint. “One-to-one” still has limits: absolute-only contexts convert percent/auto to zero, and unknown enum values are dropped. [L3] [L5] [L12] [L20] [L31] [L32]
4. **Recorded canvas is a sensible cross-process seam.** A `draw` callback runs in a Solid effect and depends on resize feedback; it records quads, paths and text. Rust replays element-local commands when painting. There is no JS callback during native paint and no pretend GPU readback. It explicitly documents quadratic curves, fixed-step stroke flattening, lack of state stack/readback, and their costs. This is a useful pattern for JS-authored diagrams or custom graphics. [L1] [L21] [L22]
5. **Native input is more than an IME string bridge.** Input owns shaped wrapped lines, selection, marked text, clipboard actions, grapheme boundaries, point-to-character mapping and candidate bounds. Equal-value controlled echoes return before clearing selection/composition. It separates editable state from the window root via a retained input Entity. These are concrete mechanisms; full IME correctness has not been established by our read-only study. [L13] [L14]
6. **Logical lists and committed row windows are distinguished.** UniformList filters its measurement-row callback out of visible-range reporting. Variable-height List requests chunks, inserts estimated-height placeholders, invalidates newly committed window rows, and supports bottom alignment/tail following. It documents the one-frame catch-up cost instead of claiming synchronous cross-process row production. [L15] [L23]
7. **A low-ceremony default app and binary packaging design.** `render(() => <App />, windowOptions)` gives a quick entry point. The release workflow builds macOS ARM/x64 and Linux x64/ARM hosts, stages platform npm packages, and publishes them before a client with exact optionalDependency pins. macOS enables runtime shaders so building need not require full Xcode. Those are implemented setup/package mechanisms, not confirmed available published binaries. [L4] [L8] [L9] [L24]

### What is weaker, wrong, or unverified

- **Mutation validation is permissive, not transactional.** `Tree::apply` writes directly to live state, replacement IDs overwrite prior nodes, and missing nodes generally cause no-ops. Parsing is handwritten over positional JSON, with defaulted props/anchors/arguments and string coercion. No surface/epoch/revision schema or rollback journal exists in this inspected path. The small implementation is easier to read, but our native contract boundary cannot safely be simplified this far. [L11] [L12] [O3] [O18]
- **Transport pressure and lifecycle are less bounded.** The native reader uses `async_channel::unbounded`; JS writes ignore `Writable.write`'s pressure result, and line accumulation has no shown size budget. Native `emit` holds a stdout mutex and flushes synchronously. `Session` rejects pending calls on a `closed` message, but its transport exit callback only clears the connection/notifies close listeners; an abrupt exit without `closed` does not perform that pending-call rejection. These are concrete code-path differences, not measured freezes or memory growth. [L6] [L7] [L25] [L28] [O4]
- **Event identity is only current node/handler identity.** Listener changes intentionally avoid wire work when presence remains true. Incoming events call the current handler map, and each event independently flushes Solid and the operation batch. There is no event generation/revision protection. Our per-chunk semantic event batch and bounded callback generations should not be replaced with this convenience. [L1] [L6] [O5] [O23]
- **Native input and JS input callbacks have a concrete shape mismatch.** Rust emits `{ value, selectionStart, selectionEnd }`; `Session.#receive` unwraps every object containing `value` to `detail.value`. The exported InputEvent and Showcase callbacks expect an object (`event.value`). Thus the code path hands those callbacks a string instead of the declared event object. Fake tests cover bare-value unwrapping, not this native InputEvent payload. This is a static cross-boundary finding, not an executed regression. [L6] [L13] [L29] [L30] [L19]
- **One session/window is a structural limitation.** Renderer callbacks close over a singleton `session`, `Session.start` refuses another running session, and the host holds one root/window. Exporting a `Session` class alone does not give another renderer its own session. Our transport-sharing multi-surface abstraction is a genuine capability difference. [L4] [L6] [L7] [O22]
- **List mutation information is weaker than ours.** Variable-list count growth uses `insertedAt` and shrink resets; the committed window drives remeasurement. Equal-count data replacement/reorder is not described by a native data-revision/replacement-span protocol. Our revision-chained edit contract is worth keeping. This statement is scoped to the inspected list property/update paths, not a claim that Solid keyed reordering never works. [L12] [L15] [O9]
- **Input richness is not complete input qualification.** No undo/redo actions/history appear in the complete input state/action file. `set_value` clamps byte offsets to length but does not visibly normalize them to new UTF-8 boundaries; a changed value containing multibyte characters deserves a regression check before copying this policy. Marked-range replacement/selection arithmetic likewise deserves native IME tests. These are source-review concerns, not a reproduced panic or failed composition. [L13] [L14]
- **No general native service/component registration seam was found.** There is a transport abstraction for tests/tooling and a fixed list of window/dialog/shell calls; adding application Rust services or native components requires changing/extending host source in the inspected design. Our NativeModule/NativeView seam offers more than a string-named built-in command dispatcher. [L16] [O16] [O28]
- **Vite integration is not persistent native HMR.** The plugin has config/transform hooks, not a host/module-runner reload supervisor. Counter setup explicitly builds and starts Node. No HMR lifecycle/state handoff implementation was found in the bounded source/config search. [L17] [L18] [O30] [O31]
- **Verification is concentrated on the fake boundary.** Renderer, styles, lengths/colors, animation normalization, and event names have JS tests; Rust has scrollbar geometry unit tests. In the inspected checkout/search there is no native input/TestApp application integration suite or performance harness/report comparable to ours or `heyhuynhgiabuu`'s. GitHub's Actions API returned no runs, and release workflow existence/public npm optionalDependency declarations do not prove that a consumer can download a matching host today. [L19] [L26] [L24] [M4] [M8] [M7]

### Ideas worth adopting from this project

- **Export our actual transform as a compiler integration seam.** Keep the canonical official compiler pin, TypeScript erasure/source-map composition, and our stable Solid runtime. Do not adopt their Solid 2 built-in import list blindly; our transform deliberately disables compiler auto-import defaults so the runtime ABI remains ours. [L2] [O13]
- **Normalize authoring ergonomics to the strict native contract.** Consider X/Y spacing and canonical color parsing first; explicit unit unions/flex basis/overflow axes require deliberate schema/native support rather than accepting strings and hoping. Keep our rejection of unknown fields. [L3] [L5] [O33]
- **Consider a recorded-paint component through NativeView.** Required acceptance: bounded command/vertex/text counts, finite coordinates, explicit size feedback, transform/clip/theme/DPI behavior, no JS callbacks on GPUI foreground, and precise “not browser Canvas” limits. The inspected alternative has no command-count admission budget in its paint loop, so the pattern needs our resource policy. [L21] [L22] [O28]
- **Give users a simple default-host path.** Maintain the advanced Rust-owned host/custom-service path, but do not require every counter/prototype to learn all of it. A pinned prebuilt host is an alternative authoring/installation path, not a reason to put closures or mutable JS values on native threads. [L4] [L8] [O6]

## 5. What we do well, and what we got wrong

### Preserve these local advantages

**Transactional state is real code on both sides.** JS finalizes private props, materializes sibling order, publishes a complete Snapshot/Patch only after transport admission, and rolls back host graph state on failure. Rust validates the revision header, holds an undo journal through semantic/provider validation, then commits the revision. Tests assert rejected operations leave the tree unchanged and that batch text ancestors derive once. This does not roll back arbitrary application signal writes, filesystem IO, or asynchronous native side effects. [O2] [O3] [O21] [O35]

**Native input owns the transient state it must answer synchronously.** We have native history (bounded to 100 snapshots), composition, selection direction, caret scrolling, geometry, grapheme-aware motion/deletion, and an edit-sequence-controlled synchronization guard. Our code already avoids the “every JS echo is a new edit” mistake. `lxsmnsyc` validates the native-input direction and offers a nicely isolated Entity model, but it is not a more qualified replacement for our full input path. [O8] [O25] [O26] [O45] [L13]

**Events and runtime boundaries are designed for asynchronous reality.** We route shared transport events per surface, reject wrong epochs/future revisions/nonmonotonic sequences, retain current/previous callback generations, and batch each delivered surface event list inside one host transaction. The Rust foreground pump explicitly yields after bounded message/byte work. QuickJS owns every JS value on one worker and exchanges bounded bytes; this is not simply an embedded engine invoked on the UI thread. [O4] [O5] [O15] [O17] [O23] [O24] [O46]

**Native extensibility is a deep module boundary.** NativeModule distinguishes short foreground UI operations from worker execution and cancellation policy; NativeView declares mount/update/unmount and typed properties/events/children. Generated SDK/catalog identity prevents an apparently available but unrenderable extension from being published. A fixed built-in helper is simpler to package, but it does not replace this application-owned capability. [O16] [O28] [O29]

**Development tooling and meaningful verification are already substantial.** A staged candidate avoids publishing failed setup/render output, explicit captureState avoids retaining owners across generations, and Rust edits replace the host/bindings together. The profiler guide separates CPU draw, native presentation, physical latency, and HUD effects. Website packaging builds/extracts/checksums platform artifacts and explicitly preserves interactive qualification/signing limits. Neither alternative's smaller codebase is evidence that these requirements were unnecessary. [O30] [O31] [O36] [O37]

### Correct these weaknesses without regressing those advantages

1. **We exposed too much integration cost at the entry point.** Getting started requires an exact SDK source checkout, Cargo patches and root-owned optimization profiles even though npm supplies the JS SDK. `doctor` diagnoses this burden but cannot remove it. The smaller alternatives show that a no-custom-Rust default host can be a separately versioned binary dependency. Recommendation: one standalone scaffold, exact JS/native/protocol pairing, and a prebuilt default host option; keep custom native module builds explicit. This is an adoption proposal, not an existing supported feature. [O6] [O7] [L8] [H33]
2. **Our style authoring API is narrower and more cumbersome in specific useful dimensions.** Core styles use separate pixel/percent width fields, physical side fields, a small named font-weight union, and six/eight-digit hex colors; core View/Pressable do not expose `hoverStyle`, `activeStyle`, `groupHoverStyle`, `flexBasis`, or per-axis overflow. The complete exported core Style/View/Pressable sources and bounded SDK search establish that narrow claim; generated native controls can have their own state styles, and this is not “our project has no hover, graphics, or responsive UI.” Learn the normalization pattern without claiming browser CSS. [O33] [O38] [O47] [L3] [L5]
3. **We have no first-class public JS recorded-canvas API in the inspected SDK, despite broad native widgets.** Core host kinds and maintained generated component SDK searches found no Canvas/draw-list contract. Native modules, charts/plots, images, and Rust GPUI custom painting remain available. The opportunity is an author-friendly arbitrary recorded-drawing seam, not a replacement renderer. Only adopt if it solves a concrete diagram/visualization consumer need. [O18] [O28] [L21] [L22]
4. **Fine-grained Solid updates are not automatic native render-region isolation.** Our root still constructs elements from the retained root after accepted changes, like the alternatives. Do not present byte-sized patches as proof of tiny CPU draw cost. A future render-region/cache change must model inherited style, geometry, native-only input, lists, animations and async notifications, and must preserve actual content. The existing measurement guide already says this; product claims should remain equally precise. [O32] [O36] [O39]
5. **Vendored upstream patches buy behavior but create a consumer and maintenance burden.** We patch GPUI for text/cache geometry, list height estimates, bounds invalidation, animation clock, and managed images. `lxsmnsyc` uses an explicit unmodified upstream `rev`; `heyhuynhgiabuu` uses a lockfile-pinned stock checkout. Their upstream simplicity is good, but their limitations do not prove our patches were wrong. Recommendation: keep a small patch inventory, upstream the generally applicable fixes, and reduce external workspace-root patch requirements where feasible. Also reconcile provenance notes: our vendor PATCHES.md opening still says 0.3.5 while the actual manifest is 0.3.7. This note does not edit it. [O7] [O10] [O40] [L9] [H16] [H17]
6. **Do not replace evidence with feature breadth.** Our published npm SDK and desktop packaging checks are stronger delivery evidence than declarations in the alternatives, but extracted-artifact checks still do not establish physical input, every Linux compositor, signing/notarization, or all Embedded Bun targets. Our website/release notes explicitly call out those limits. The correct response is a concise capability/qualification matrix, not “we support more platforms, therefore faster/more stable.” [O6] [O37] [O41] [M5]

## 6. Adoption priorities and explicit non-adoptions

| Priority | Adopt or investigate | Reason and acceptance boundary |
| --- | --- | --- |
| High | Standalone scaffold + optional pinned prebuilt default host | Reduces simple-consumer setup; verify from a new application directory with installed packages, actual native launch/input/teardown, no repository/dev-host fallback. Preserve exact protocol/native catalog identity. [O6] [L8] [H10] |
| High | Public compiler-only API using our canonical transform | Lets Bun plugins/other bundlers/tools use real universal compilation and source maps without implementing Vite lifecycle. Preserve one runtime instance and stable-runtime ABI. [L2] [O13] [O14] |
| Medium | Optional style sugar compiled to current strict Style | X/Y spacing and parsed color sugar are low-risk; classes/length units require a capability matrix and producer→native checks. Unsupported input must be diagnosed, not silently ignored. [L3] [L5] [H26] [H30] [O33] |
| Medium, consumer-led | Bounded recorded-paint NativeView | A useful extension for JS-authored graphics; replay retained records, not callbacks. Qualification includes counts, finite coordinates, resize/theme/DPI, clipping, ownership and teardown. [L21] [L22] [O28] |
| Medium | A short cross-boundary benchmark/capability report | Retain named workloads, producer mutations, native work, presentation boundaries, raw artifacts, and negative results. Extend existing local profiling/soak evidence rather than inventing a new renderer-only score. [H8] [H28] [O34] [O36] |
| Ongoing | Native domain components and upstream patch reduction | Application/native modules already support the abstraction; placement and caching are justified by synchronous requirements or measurements, not language prestige. [H12] [O16] [O28] [O40] |

**Do not adopt:** partial apply plus poison as a replacement for our atomic commit; unchecked current-handler dispatch; unbounded NDJSON/channel/write queues; retain-all ownership for core VirtualList; singleton session as our multi-window foundation; open-ended unsupported style keys; global disjoint request-number ranges; an automatic React JSX factory; or a Solid 2 runtime migration merely because both projects use prereleases. None solves a demonstrated deficiency in our current design. [H1] [H2] [H3] [H9] [L6] [L7] [L11] [L12] [O2] [O4] [O5] [O9] [O13]

Adoption above means adapting the ideas to our contracts, not silently copying source. If code is reused, retain the actual per-file license and attribution: `heyhuynhgiabuu` is Apache-2.0 with attributed MIT-derived content, while `lxsmnsyc` is MIT. [H11] [H12] [L8]

## 7. Verification limitations and search bounds

- **Executed:** safe Git clone/identity/status commands; read-only GitHub API and public npm metadata requests; file reads and bounded content searches. **Not executed:** any repository dependency installation/build/test/profile/application/release scripts. Existing CI outcomes and reported measurements are attributed to their owners, not to this session.
- The note's reference definitions and line ranges were checked locally against each cited pinned Git blob, including confirming that inspected working-copy files match those blobs. A whitespace check of this new note produced no whitespace diagnostics. These are document/source-integrity checks, not capability tests or proof that all web links are reachable forever.
- Remote source search covered both complete `packages`/`crates` trees plus examples, scripts, documentation, and workflows. Extension/service searches used terms including `native_module`, registration, `extension`, custom properties and concrete dispatcher/API matches. HMR searches covered `hot`, `reload`, `epoch`, `revision` and example launch scripts. Performance/tests searches covered benchmark files, test files, Rust test attributes, and workflow gates. Negative findings above are scoped to these **pinned checkouts**, not all branches, private consumer code, or future releases.
- Local source was read starting from `CONTEXT.md`, then concrete renderer HostTree/router/listener/props files, Rust tree/input/virtual-list/runtime/native/host seams, Vite transform/application reload, package manifests, dependency patches, and packaging/performance guides. This is not a complete review of every native widget or platform backend.
- No common production workload was run. Different GPUI revisions, Solid/runtime versions, native trees, debug/release builds, cache states, viewport geometry, and measurement endpoints make reported timings incomparable. “Binary”, “fewer processes”, “Rust parser”, “Solid 2”, and “GPU renderer” are **not speed results**.
- GitHub metadata/job status and npm registry observations are time-bound first-party records, not immutable source blobs. Code/document citations below are pinned `blob/<commit>/...#L...` links. A green job does not prove skipped tests ran; `heyhuynhgiabuu`'s GUI jobs were separately checked, and native/headless/human acceptance remains distinct.
- Static mismatches and input concerns are labeled accordingly. No alternative defect was reproduced by executing an application. Private npm packages may legitimately return public 404. Absence of GitHub Releases does not rule out npm/private distribution.
- Documentation/website ownership indexes (`docs/README.md`, `examples/website/README.md`) were inspected. **No maintained guides, API generators, website content, navigation, or project code were changed**: this task creates an investigation record only and changes no shipped behavior/API. Build/test documentation and website checks were intentionally not run under the read-only/no-build scope.

## Sources

### Immutable sources: `heyhuynhgiabuu` at `05f84e9f17cd8360b9af92f615a884527fa3a986`

[H1]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/packages/solid/src/renderer.ts#L201-L274
[H2]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/main.rs#L797-L949
[H3]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/packages/client/src/client.ts#L78-L283
[H4]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/tests/stdio_window.rs#L38-L101
[H5]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/host.rs#L1572-L1755
[H6]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/host.rs#L1307-L1445
[H7]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/.github/workflows/ci.yml#L97-L172
[H8]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/docs/performance.md#L1-L94
[H9]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/host.rs#L3039-L3204
[H10]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/docs/packaging.md#L3-L61
[H11]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/Cargo.toml#L1-L13
[H12]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/markdown/mod.rs#L1-L110
[H13]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/packages/solid/package.json#L1-L33
[H14]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/package.json#L12-L59
[H15]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/scripts/solid-jsx-preload.ts#L1-L27
[H16]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/Cargo.lock#L2232-L2235
[H17]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/Cargo.toml#L9-L57
[H18]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/packages/solid/src/render.ts#L27-L174
[H19]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/packages/solid/src/jsx.ts#L1-L100
[H20]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/host.rs#L824-L838
[H21]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/host.rs#L2918-L3036
[H22]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/host.rs#L521-L578
[H23]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/examples/counter.ts#L72-L104
[H24]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/packages/solid/src/renderer.ts#L902-L930
[H25]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/packages/solid/src/h.ts#L1-L69
[H26]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/packages/solid/src/utilities.ts#L15-L24
[H27]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/markdown/mod.rs#L112-L167
[H28]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/headless_lifecycle_benchmark.rs#L359-L417
[H29]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/packages/solid/src/utilities.ts#L147-L157
[H30]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/host.rs#L3240-L3437
[H31]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/packages/solid/src/utilities.ts#L226-L233
[H32]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/main.rs#L638-L679
[H33]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/.github/workflows/release.yml#L1-L141
[H34]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/ROADMAP.md#L35-L94
[H35]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/main.rs#L240-L336
[H36]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/protocol/tests/round_trip.rs#L44-L71
[H37]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/packages/protocol/src/batch.test.ts#L70-L85
[H38]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/host.rs#L4667-L4696
[H39]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/host.rs#L5170-L5304
[H40]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/packages/protocol/src/batch.ts#L24-L48
[H41]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/docs/packaging.md#L88-L117
[H42]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/packages/solid/src/renderer.ts#L277-L319
[H43]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/scripts/benchmark-consumer.ts#L58-L126
[H44]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/scripts/benchmark-stdio.ts#L139-L190
[H45]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/main.rs#L144-L225
[H46]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/scripts/benchmark-protocol.ts#L118-L186
[H47]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/packages/protocol/src/style.ts#L9-L78
[H48]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/host.rs#L1780-L1821
[H49]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/docs/tailwind-subset.md#L3-L67
[H50]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/crates/helper/src/host.rs#L352-L359
[H51]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/docs/packaging.md#L139-L166
[H52]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/README.md#L11-L27
[H53]: https://github.com/heyhuynhgiabuu/solid-gpui/blob/05f84e9f17cd8360b9af92f615a884527fa3a986/packages/solid/src/perf.test.ts#L1-L96

### Immutable sources: `lxsmnsyc` at `196aa6edc779cb39f37a3ade4517ed197ad58813`

[L1]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/packages/solid-gpui/src/renderer.ts#L55-L247
[L2]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/packages/solid-gpui/src/compiler.ts#L11-L66
[L3]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/packages/solid-gpui/src/style.ts#L108-L219
[L4]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/packages/solid-gpui/src/index.ts#L75-L163
[L5]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/packages/solid-gpui/src/length.ts#L1-L47
[L6]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/packages/solid-gpui/src/session.ts#L24-L204
[L7]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/crates/solid-gpui-host/src/main.rs#L250-L380
[L8]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/packages/solid-gpui/package.json#L1-L74
[L9]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/crates/solid-gpui-host/Cargo.toml#L1-L47
[L10]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/pnpm-lock.yaml#L45-L71
[L11]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/crates/solid-gpui-host/src/protocol.rs#L77-L188
[L12]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/crates/solid-gpui-host/src/tree.rs#L163-L534
[L13]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/crates/solid-gpui-host/src/input.rs#L18-L172
[L14]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/crates/solid-gpui-host/src/input.rs#L482-L615
[L15]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/crates/solid-gpui-host/src/render.rs#L703-L937
[L16]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/crates/solid-gpui-host/src/commands.rs#L31-L150
[L17]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/packages/solid-gpui/src/vite.ts#L11-L66
[L18]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/examples/counter/package.json#L1-L21
[L19]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/packages/solid-gpui/test/renderer.test.tsx#L86-L202
[L20]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/crates/solid-gpui-host/src/style.rs#L17-L59
[L21]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/packages/solid-gpui/src/canvas.ts#L1-L217
[L22]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/crates/solid-gpui-host/src/canvas.rs#L16-L144
[L23]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/README.md#L135-L150
[L24]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/.github/workflows/release.yml#L1-L124
[L25]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/packages/solid-gpui/src/transport.ts#L99-L175
[L26]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/crates/solid-gpui-host/src/scrollbar.rs#L369-L411
[L27]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/crates/solid-gpui-host/src/main.rs#L85-L203
[L28]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/crates/solid-gpui-host/src/main.rs#L37-L57
[L29]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/packages/solid-gpui/src/events.ts#L93-L99
[L30]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/examples/showcase/src/app.tsx#L120-L151
[L31]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/crates/solid-gpui-host/src/render.rs#L128-L204
[L32]: https://github.com/lxsmnsyc/solid-gpui/blob/196aa6edc779cb39f37a3ade4517ed197ad58813/crates/solid-gpui-host/src/style.rs#L393-L523

### Immutable local sources: `Cyenoch` at `e9bc4389c5394e5de1fecb6758d2d06e48877fdb`

[O1]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/CONTEXT.md#L7-L93
[O2]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/renderer/host-tree.ts#L108-L225
[O3]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/tree.rs#L260-L399
[O4]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/host/commit_pump.rs#L13-L165
[O5]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/renderer/nodes.ts#L45-L181
[O6]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/docs/getting-started.md#L14-L59
[O7]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/Cargo.toml#L15-L72
[O8]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer/input.rs#L46-L110
[O9]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer/virtual_lists.rs#L1-L188
[O10]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/vendor/gpui/Cargo.toml#L12-L35
[O11]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/package.json#L1-L104
[O12]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/package.json#L29-L41
[O13]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui-vite/src/transform.ts#L1-L33
[O14]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui-vite/src/index.ts#L97-L179
[O15]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/transport.rs#L59-L148
[O16]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/native/mod.rs#L1-L53
[O17]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/runtime/quickjs.rs#L1-L175
[O18]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/docs/protocol.md#L1-L151
[O19]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/protocol/guard.rs#L1-L79
[O20]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/renderer/host-config.ts#L78-L217
[O21]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer.rs#L557-L704
[O22]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/surface-host.ts#L70-L168
[O23]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/renderer/root-container.ts#L843-L879
[O24]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/tests/renderer.test.ts#L1065-L1171
[O25]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer/input.rs#L654-L672
[O26]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer/input.rs#L1733-L1899
[O27]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/docs/scroll-performance.md#L158-L210
[O28]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/native/component.rs#L203-L280
[O29]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/host/mod.rs#L64-L141
[O30]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/application.ts#L14-L159
[O31]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/docs/hot-reload.md#L139-L215
[O32]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer.rs#L1268-L1293
[O33]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/style.ts#L61-L205
[O34]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/tests/surface_soak.rs#L161-L219
[O35]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/tree/transaction_tests.rs#L96-L174
[O36]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/docs/performance-analysis.md#L9-L48
[O37]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/.github/workflows/website-packages.yml#L20-L125
[O38]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/renderer/types.ts#L247-L315
[O39]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/docs/native-composition.md#L34-L48
[O40]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/vendor/gpui/PATCHES.md#L1-L39
[O41]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/docs/distribution.md#L1-L40
[O42]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/Cargo.toml#L34-L52
[O43]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/host/mod.rs#L322-L363
[O44]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/index.ts#L167-L269
[O45]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer/input.rs#L470-L486
[O46]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/renderer/surface-router.ts#L75-L226
[O47]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/style.ts#L465-L477

### Time-bound first-party metadata, observed 2026-10-01

[M1]: https://api.github.com/repos/heyhuynhgiabuu/solid-gpui
[M2]: https://api.github.com/repos/lxsmnsyc/solid-gpui
[M3]: https://api.github.com/repos/heyhuynhgiabuu/solid-gpui/releases?per_page=5
[M4]: https://api.github.com/repos/lxsmnsyc/solid-gpui/releases?per_page=5
[M5]: https://registry.npmjs.org/@solid-gpui%2Fcore
[M6]: https://github.com/heyhuynhgiabuu/solid-gpui/actions/runs/34584041961
[M7]: https://api.github.com/repos/lxsmnsyc/solid-gpui/actions/runs?per_page=5
[M8]: https://registry.npmjs.org/solid-gpui
[M9]: https://registry.npmjs.org/@solid-gpui%2Fsolid
[M10]: https://registry.npmjs.org/@solid-gpui%2Fclient
