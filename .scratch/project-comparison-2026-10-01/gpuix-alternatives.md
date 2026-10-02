# GPUIX alternatives: source comparison with Solid GPUI

Research date: **2026-10-01**. This is a read-only implementation assessment, not a framework speed ranking. The only project file created by this investigation is this note.

## Executive conclusion

**Keep Solid GPUI's native ownership and byte-only runtime boundary. Adopt the alternatives' delivery and verification discipline before considering a transport rewrite.**

- **Our strongest implemented advantages:** bounded runtime/foreground handoff, transactional tree and extension validation, revision-aware callbacks, explicit controlled-input acknowledgements, generated application-owned Rust contracts, and automatic JavaScript row-owner virtualization. These are real source properties, not evidence that we render faster.
- **GPUIX's strongest advantages:** prebuilt native npm binaries, a small first-app path, GPU-backed testing exposed to JavaScript, live native automation, retained style interning with reclamation, native hover/active refinements, and a concrete signed-update implementation.
- **The separate `gpuix-solid` project's strongest advantages:** disciplined Solid 1/2 boundaries, source-pinned application ports, exact-package/clean-consumer checks, interactive native acceptance, ergonomic style compilation, and experimental Canvas/video work that tests actual rendering instead of merely adding JSX declarations.
- **Our clearest mistakes/weaknesses:** a hard external-consumer native setup; a public testing boundary that stops before native layout/hit testing; primitive-root invalidation that is broader than fine-grained Solid updates suggest; and a concrete UTF-8/UTF-16 mismatch in `TestHost`'s default input selection offsets.
- **Do not copy blindly:** the alternatives' macOS JavaScript-driven AppKit pump, global DOM-shaped compatibility, legacy mutation fallbacks, or pointer bytes masquerading as a portable resource handle. None is required to get their useful ergonomics.

The evidence for these conclusions is developed below. “Implemented” means traced in the pinned source; it does **not** mean run or benchmarked during this investigation.

## 1. Versions, provenance, and availability

| Project | Inspected revision | Default branch / archived | HEAD commit time, UTC | GitHub `pushed_at`, UTC | Manifest baseline |
| --- | --- | --- | --- | --- | --- |
| `remorses/gpuix` | `4ecca30f68057b4d9830d32675ba4ed999eeeaaa` | `main` / `false` | 2026-09-27 19:54:30 | 2026-09-27 19:55:31 | `@gpuix/native`, `@gpuix/react`, `@gpuix/solid` 0.10.0 |
| `jhomra21/gpuix-solid` | `ad384ff3755761269902dcd7db26ef6163fa722c` | `main` / `false` | 2026-09-23 14:47:37 | 2026-10-01 02:11:09 | Solid 2 package 0.2.1-beta.0; Solid 1 package 0.1.0-beta.0; both pin native 0.9.0 |
| Local `Cyenoch/solid-gpui` | `e9bc4389c5394e5de1fecb6758d2d06e48877fdb` | local checkout | 2026-09-29 10:23:21 | not queried | SDK/Rust workspace 0.5.2; Solid runtime 1.9.15; compiler 2.0.0-rc.9 |

GitHub metadata was read from the first-party [GPUIX API](https://api.github.com/repos/remorses/gpuix) and [GPUix Solid API](https://api.github.com/repos/jhomra21/gpuix-solid), with default-branch commits independently checked through `/commits/main`. Both report `fork: false`; that metadata does not settle source derivation. `pushed_at` concerns the repository, not necessarily its default branch: GPUix Solid's October push did not change the inspected September `main` HEAD. The local tree was clean when first checked; other comparison notes appeared while this subtask ran and were left untouched.

Package and framework pins come from source: [GPUIX native manifest][G-native-pkg], [GPUIX React manifest][G-react-pkg], [GPUIX Solid manifest][G-solid-pkg], [GPUix Solid 2 manifest][S-pkg], [GPUix Solid 1 manifest][S1-pkg], [local Cargo requirements][L-cargo], and [local framework pins][L-js-pins]. GPUIX uses the `remorses/zed` fork's `gpuix` branch as a submodule, with gitlink `ea042f2f045157ada1d0ad71009520931dcace83`; its Cargo file uses that checkout's GPUI, not a crates.io GPUI version. Local Cargo resolves `gpui-pre = 0.3.7` with local vendor patches. These are different native dependency baselines. [Submodule configuration][G-submodule] [GPUIX Cargo][G-cargo] [Local Cargo][L-cargo]

First-party public npm registry observations on the research date:

- `@gpuix/native`, `@gpuix/react`, and `@gpuix/solid`: `latest = 0.10.0`, published September 23. The published adapters depend on native `^0.10.0`, despite upstream's advice to add exact direct dependencies.
- `gpuix-solid`: `latest = 0.2.0` (September 17); `beta = 0.2.1-beta.0` (September 18). Stable 0.2.0 pins native `0.9.0` and universal `2.0.0-rc.8`. The inspected repository contains later changes; **HEAD is not the published beta tarball**.
- `@jhomra21/gpuix-solid1`: the unauthenticated public registry request returned **HTTP 404**. Its source/package and repository tests exist, but its README's public-install command was not confirmed as usable from the public registry. This does not establish whether private or previously removed versions exist.
- `@solid-gpui/core`: `latest = 0.5.2`, published September 29.

Registry sources: [native](https://registry.npmjs.org/@gpuix%2Fnative), [React](https://registry.npmjs.org/@gpuix%2Freact), [upstream Solid](https://registry.npmjs.org/@gpuix%2Fsolid), [separate Solid](https://registry.npmjs.org/gpuix-solid), [Solid 1 lookup](https://registry.npmjs.org/@jhomra21%2Fgpuix-solid1), [our core](https://registry.npmjs.org/@solid-gpui%2Fcore). Dist-tags and metadata are mutable observations; implementation citations below are immutable commit links.

Sources were cloned without installing dependencies or initializing submodules, into unique approved temporary directories:

```text
/private/var/folders/sl/42r5kc756mj96rlxkglrpr900000gn/T/opencode/gpuix-source.nw08py
/private/var/folders/sl/42r5kc756mj96rlxkglrpr900000gn/T/opencode/gpuix-solid-source.zNhQ74
/private/var/folders/sl/42r5kc756mj96rlxkglrpr900000gn/T/opencode/gpuix-0.9-source.gN9hc0
```

The third clone pins the native 0.9.0 release tag, commit `7ac9880abd8e91e5bf0e4feb0fa850729cf95a68`, so GPUix Solid's native baseline was checked rather than assuming current upstream behavior applies to it.

## 2. Relationship: what actually depends on what

```text
remorses/gpuix
  @gpuix/react 0.10.0 ─────┐
  @gpuix/solid 0.10.0 ─────┴─> @gpuix/native 0.10.0 ─> remorses/zed GPUI fork

jhomra21/gpuix-solid
  gpuix-solid (Solid 2) ───────────┐
  @jhomra21/gpuix-solid1 (Solid 1) ┴─> exact @gpuix/native 0.9.0
  source-edge lane ─> upstream 410fb56f… + three committed native patches
  MediaBunny integration ─> separate Objective-C++ N-API decoder addon

our solid-gpui
  Solid universal renderer ─> framed byte contract ─> our Rust host
  runtime adapters: external Bun / worker-owned QuickJS / experimental embedded Bun
```

Concrete evidence:

1. GPUix Solid constructs `GpuixRenderer` directly from `@gpuix/native`; both manifests pin `0.9.0`. It is a framework adapter over GPUIX's renderer, not an independent Rust GPUI implementation. [Runtime import and construction][S-runtime] [Solid 2 dependencies][S-pkg] [Solid 1 dependencies][S1-pkg]
2. Current upstream GPUIX has **its own** Solid 1 adapter. Its `host.ts` explicitly says the host-tree shape is based on `jhomra21/gpuix-solid`, stripped to GPUIX's native contract. It imports shared host/event machinery from `@gpuix/native/host`, not the separate `gpuix-solid` package. The relationship includes upstream adoption of adapter ideas, not just one-way use of GPUIX. [Derived host and imports][G-solid-host] [Upstream Solid dependency range][G-solid-pkg]
3. GPUix Solid's source-edge pin is `410fb56f2e599ef49b1dabfc43872b6ff8047916`, with Canvas, video-frame, and IOSurface patches. Its script applies these patches to the upstream checkout before building. “Consumes upstream rather than carrying a Rust fork” is accurate for its **published default dependency**, but incomplete for its **patched development lane**. [Pin and patch list][S-edge] [Patch application][S-edge-script]
4. MediaBunny adds `node-addon-api`, native `.mm` source, and a platform-specific decoder binding. It is not merely a Solid wrapper and is not part of the stock native 0.9.0 contract. [Package contents/dependencies][S-media-pkg] [Addon loading][S-media-runtime]

## 3. `remorses/gpuix`: implemented behavior and tradeoffs

### Compiler, renderer, and reactivity

React uses a mutation-mode `react-reconciler` concurrent root. Host objects created during speculative render stay in JavaScript; `materialize()` emits native creation only during placement, avoiding abandoned-render native allocations. `resetAfterCommit()` flushes the commit as one native call. Updating a host always resends its complete style; custom props are diffed, and callback replacement changes the JavaScript event map without enabling an already-enabled native listener again. This is real incremental **mutation delivery**, not a claim of per-node retained GPUI scenes. [Materialization and flush][G-react-host] [Update path][G-react-update]

Upstream's Solid adapter uses `solid-js/universal`, with host mutation callbacks rather than a virtual DOM. Its Bun plugin applies `babel-preset-solid` with `generate: "universal"` and redirects Solid's server runtime/store files to live client implementations. It is Solid 1 (`>=1.9 <2`), distinct from the other repository's Solid 2 package. Native mutations flush at mount, explicit synchronous operations, event completion, or a scheduled microtask. [Compiler plugin][G-solid-compiler] [Solid root][G-solid-root] [Package peer][G-solid-pkg]

### Bridge and threading: N-API, not renderer IPC

`createMutationQueue()` collects tuples and calls `applyBatch(JSON.stringify(queue))`. Rust receives one `String`, locks its retained tree once, parses/applies the batch, drops the lock, then invalidates the window. Event payloads return through a nonblocking N-API `ThreadsafeFunction`. The UI bridge is **in-process**; SSE/stdin/stdout in this project belongs to **automation**, not to the JS-to-native UI mutation transport. [Mutation queue][G-queue] [Native bridge][G-bridge] [Automation protocol][G-automation-protocol]

- **macOS:** `MacPlatform::new_embedded()` and `run_embedded()` live on the calling thread. JavaScript must keep calling `tick()`, which pumps AppKit. The shared JS loop uses self-scheduled `setTimeout`, default 8 ms, subtracting the previous tick's duration. A long synchronous JavaScript task delays the next pump; same-process N-API does not remove this coupling. The 8 ms timer is not evidence of 125 displayed FPS. [macOS initialization][G-macos] [Tick][G-tick] [Frame loop][G-loop]
- **Windows/Linux/FreeBSD:** GPUI's blocking event loop runs on a spawned `gpuix-ui` Rust thread. Commands use an explicitly unbounded `mpsc` channel; some queries synchronously wait on a response channel. `tick()` only observes whether that UI thread is still running. Rendering and mutation application share `Arc<Mutex<RetainedTree>>`. They avoid pipe framing/copying but introduce lock contention and do not make all native calls asynchronous. [Thread creation/commands][G-threaded] [Synchronous query examples][G-queries]
- **0.9 baseline:** the separate Solid project uses the same broad N-API/tree-lock/nonblocking-event architecture and macOS pump split; checked in the release source. [0.9 bridge][G09-bridge] [0.9 batch/tick][G09-tick]

Do not confuse this with `bun:ffi`, a browser web view, or an IPC renderer. It is an addon in the Node/Bun process, plus a separate native UI thread on some operating systems.

### Atomic parsing is useful, but not our tree-validation contract

The batch parser reads tuples into typed `BatchOp` values and borrows raw style JSON, avoiding an intermediate `Vec<serde_json::Value>` for the whole batch. It rejects unknown operations and invalid numeric IDs. It resolves all styles before touching elements; malformed JSON/styles therefore leave the element tree unchanged. This optimization and atomicity already exist in native 0.9.0. [Parser and style resolution][G-parser] [Current apply loop][G-apply] [0.9 apply loop][G09-apply]

The guarantee is narrower than ours. Mutation methods can silently skip missing nodes, replace duplicate IDs, or fall back to appending when an `insertBefore` anchor is absent; they do not enforce a validated reachable acyclic tree or a surface/epoch/base-revision chain. The `append_child` implementation removes the child from its old parent before attempting the new parent, without candidate-tree validation. That is demonstrated permissiveness, **not an assertion that normal adapter output is broken**. [Retained mutations][G-tree-mutations]

### Retention and frame costs

Native tree nodes have stable numeric identities. **Styles are `Option<Arc<StyleDesc>>`, hash-consed from payload bytes with collision confirmation.** Equal style pointers bypass wide equality checks; semantic equality handles differently ordered JSON. The style table reclaims entries with no element references and sweeps on growth or tree collapse. This is more convincing than “deduplicate styles” alone because it includes lifetime/reclamation behavior. The byte-size and 10,000-turn numbers in comments are author observations, not measurements reproduced here. [Style representation][G-tree-style] [Interning/reclamation][G-style-table] [No-op style updates][G-tree-mutations]

Every `GpuixView::render()` still locks the retained tree, synchronizes focus state, prunes resources, builds the root's ephemeral GPUI elements, and copies scroll/list handles into thread-local maps. Ordinary host subtrees recurse; custom adapters retain their own state but `render()` is called again when reached. A small mutation queue does **not** establish a small native render region. [Frame preparation and build][G-render] [Custom adapter lifecycle][G-custom-trait]

Virtual lists genuinely defer offscreen subtree construction to `gpui::list` row requests. However, `build_virtual_list()` collects all retained child IDs/revisions and inspects focusable rows each root render. Keeping every React/Solid row mounted still pays retained-tree/framework memory and some linear list bookkeeping. `itemCount`/`windowStart` supports a bounded mounted window; the infinite-chat fixture also has a bounded page cache. Deferred row construction re-locks the tree and checks that the child still matches the requested index. [List construction][G-list] [Deferred row safety][G-list-child] [Bounded application page cache][G-infinite]

### Events and resource cleanup

The shared JavaScript renderer state owns one active root per renderer, monotonically allocates element IDs, rotates window event IDs on attach, and rejects old window event generations. Element dispatch looks up the **current** `(elementId, eventType)` callback. The native payload has no generic surface epoch/revision/event sequence. Thus same-element callback replacement does not provide our older-event/older-callback-generation contract; it is simpler and avoids callback-only native enablement traffic. [Renderer ownership/dispatch][G-events] [Payload fields][G-payload]

Native destruction recursively removes nodes and reports all IDs so JS removes event handlers. On a subsequent render the custom registry calls `destroy()` for disappeared IDs, removes focus subscriptions and handles, and removes scroll/list state. Live images explicitly drop their GPU image on removal. The GPU test renderer additionally clears entities and paints an empty frame before dropping the App, because input handlers can retain entities in the previous rendered frame. This is meaningful cleanup engineering; “Rust drops it” alone would not capture those dependencies. [Tree removal][G-tree-mutations] [Frame pruning][G-render] [Custom registry cleanup][G-registry] [Test teardown][G-test]

Asynchronous key callbacks cannot retroactively stop GPUI's native key dispatch. A per-keystroke ID lets JS propagate `preventDefault`/`stopPropagation` among its own emitted element/window events and implement JS-owned Tab/Escape defaults. This is an explicitly acknowledged semantic boundary. [Native key dispatch explanation][G-key-events] [JS key defaults][G-events]

### Text input, IME, focus, and selection

The editor is a retained `Entity<TextEditorState>` owning content, UTF-8 internal selection, UTF-16 platform conversion, marked text, scroll offsets, caret blink task, drag state, and bounded undo. Input and textarea implement GPUI `EntityInputHandler`, including marked-text replacement and IME candidate geometry. `destroy()` releases the retained entity. This is an implemented native editor, not “input” represented only as a string prop. [Editor mount/state][G-input-state] [IME implementation][G-input-ime]

Controlled echoes use a queue of up to 32 previously emitted text values: matching a pending value acknowledges it without replacing the native content/caret. A nonmatching external value replaces text, moves selection to the end, clears marked text/scroll/undo. Our explicit edit-sequence acknowledgement distinguishes stale echoes by generation rather than just matching strings and separately preserves active composition. Neither implementation was physically IME-tested here. [Echo synchronization][G-input-echo] [Our acknowledgement/composition][L-input-controlled]

GPUIX implements **window-wide cross-element text selection**, including retained selected spans when virtualization removes the anchor, and a root-scoped text-search cache keyed separately by text/search revision and matcher identity. Selection state is per renderer, not process-global. Those are useful capabilities beyond one selectable paragraph. [Cross-element model][G-selection] [Search cache][G-search]

### Styling and extension/service surface

Styles map to GPUI flex/grid, dimensions, scrolling, text, gradients, shadows, and native hover/active/focus-visible refinements. Native hover/active does not require a Solid signal write and a bridge style update for every interaction transition. CSS-shaped authoring is still a subset: strings are often permissively interpreted or ignored, not full browser CSS. [Native pseudo styles/grid][G-styles] [Focus-visible implementation][G-focus-style] [Declared style fields][G-host-style]

Custom rendering is organized around `CustomElementFactory`/`CustomElement` and a retained registry with changed-prop synchronization. At this HEAD the registry is initialized with input/textarea, anchored, image/SVG, code, diff, and markdown. **The public JS type includes `canvas`, but the default native registry has no Canvas factory; an unknown custom type warns and renders `Empty`.** A JSX/type declaration is not implementation evidence. [Type includes Canvas][G-host-element-types] [Defaults/unknown type][G-registry]

Scoped search of native exports, `lib.rs`, registry construction, and framework public APIs found no application-facing registration route comparable to our generated `NativeModule` contract. The Rust module containing custom element traits is private; GPUIX's supplied native service methods are a fixed addon API. Adding a Rust adapter to the renderer is different from exporting arbitrary typed application Rust commands/components. Native 0.10 **does** implement minimize, zoom, fullscreen, and path prompts; do not repeat GPUix Solid's older 0.9 limitations as current upstream gaps. [Module visibility][G-lib] [Window/service methods][G-window]

### HMR, packaging, testing, and performance evidence

- `render()` stores the native renderer/frame loop across hot re-evaluation and remounts the application on the existing window. The React reconciler also registers with the DevTools hook so React Refresh can drive mounted roots in the browser development path. These are not the same guarantee as preserving all component state on every desktop hot reload. Rust addon changes rebuild/restart the process; the addon cannot be unloaded safely. [React lifecycle][G-react-runtime] [Refresh hook][G-refresh] [Solid lifecycle][G-solid-runtime] [Rust watcher][G-dev]
- The CLI downloads `main.zip`, extracts only the example app, resolves npm React `latest`, and writes a caret dependency. It is an implemented no-Rust first-app workflow, **but the generated template/native pairing is not immutable or exact-versioned**. Adopt its simplicity, not its moving-target selection. [CLI][G-cli]
- Prebuilt native targets are macOS ARM64, Linux x86-64 GNU, and Windows x86-64 MSVC. CI builds all three and packages a Bun-compiled chat executable. The Hermes path bundles CJS with native external and stubs automation/safe-MDX; README binary-size figures are not a like-for-like benchmark. [Targets/scripts][G-native-pkg] [Build/compile CI][G-ci] [Standalone compile][G-compile] [Hermes build][G-hermes]
- There is an actual updater: N-API worker tasks check/download/install, minisign verification precedes installation, and relaunch is explicitly not automatic. That is product-delivery functionality, not a rendering optimization. We did not audit installation safety or exercise updates. [Signature verification][G-update-verify] [Worker API][G-update-api]
- `TestGpuixRenderer` uses native Metal/DirectX `VisualTestAppContext`, offscreen windows, the same view/build/style/event code, screenshots, painted bounds, and input simulation. Its callback queues to a Vec rather than production's N-API callback, so live tests remain valuable. Linux's test renderer is explicitly unavailable; production Linux works. [Test implementation][G-test] [Platform gating][G-lib]
- Chat/timeline benchmarks are real workload harnesses, not just counter examples. Chat times native wheel draw rather than a later idle flush, and comments explain why timing budgets cannot prove cache reuse. The Rust serialization benchmark uses the production apply function and a counting allocator. **Thresholds and source comments are not current measured results.** No numbers from these harnesses establish physical display latency or superiority over our different GPUI/runtime baseline. [Chat harness][G-chat-perf] [Allocator/serialization harness][G-serde-bench]

## 4. `jhomra21/gpuix-solid`: implemented behavior and tradeoffs

### Solid versions and compilation are explicit

Solid 2 compiles through `@solidjs/universal 2.0.0-rc.8`; Solid 1 uses its separate package/runtime. The Vite helper sets universal output, selects live browser runtime conditions, bundles the Solid/universal/framework runtime, and leaves the native addon external. A startup probe actually checks that an effect reruns. Our startup guard already detects Solid's server build and duplicate runtime identity structurally; a runtime guard is **not** an adoption gap for us. [Solid 2 universal host][S-universal] [Vite configuration][S-vite] [Behavioral guard][S-client] [Our guard][L-runtime-guard]

Framework-neutral host files are duplicated across the two packages, with a check requiring byte equality for seven mirrored host files. That reduces drift, but is a maintenance technique rather than a single compiled shared host module. [Mirror check][S-host-parity]

### Mutation delivery has more compatibility work than the README diagram implies

`MutationDriver` queues native operations, schedules a microtask, and flushes through `applyBatch`. Root event dispatch flushes Solid computations and then native mutations; mount, queries, `flushSync`, remount, and unmount also have explicit flush boundaries. Failure keeps the queue; an automatic flush logs an error rather than throwing into the originating signal write. There is still a legacy direct-method fallback. [Driver and failure behavior][S-mutations] [Root flush/dispatch][S-root]

The production `adaptBatchRenderer()` additionally parses the JSON batch, expands pointer-capture listener requirements, and stringifies it again before calling the native addon. So the full path is **JSON stringify → JSON parse/rewriting → JSON stringify → N-API Rust parse**, not merely one cheap call. It has useful semantics, but its serialization and allocation cost must be measured on real workloads. [Batch bridge][S-batch]

Host children use arrays with `indexOf`/`splice`. Activation-listener synchronization can walk descendants and repeatedly search ancestors. Root event liveness checking traverses the host tree, with further bounds-based tree walks for pointer/drag retargeting; bounds queries synchronously cross the native API. These are source-visible work costs, not a demonstrated latency problem. Our sibling links, deferred order materialization, and incremental patch planner avoid some analogous repeated sibling work. [Host insertion/removal][S-nodes-order] [Activation traversal][S-mutations] [Root liveness/retargeting][S-root] [Our host links][L-host-config] [Our settled-work contract][L-work-bounds]

### Ownership, cleanup, and hot remount

Native input/IME comes from GPUIX 0.9, not a new Solid-side editor. Stable host IDs and events are adopted into the native tree; removal marks subtrees dead and destroys native elements. Remount disposes the Solid owner, destroys the old host root, clears event/pointer/drag state, and rotates window keyboard/selection event IDs. Selection primitives retain the native subscription under a Solid owner and release it in `onCleanup`. Window-size/inset primitives clean up their polling timers. [Host removal][S-nodes-order] [Root cleanup][S-root-cleanup] [Selection ownership][S-selection] [Window polling/cleanup][S-window-size]

At HEAD, `runtime.ts` **does implement** same-window hot remount under `bun --hot`, guarded by a global render slot and generation-safe handles; its tests reject a stale handle unmounting a replacement. The README/stable starter still describes rebuild-and-restart development and the starter's script actually does that. Keep these claims separate: source capability exists; the published starter does not expose a qualified equivalent hot-development command. [Hot remount implementation][S-runtime] [Lifecycle regression][S-runtime-test] [Starter scripts][S-starter] [README boundary][S-readme-dev]

Element callbacks are the current JavaScript handlers, with root/native-ID liveness checks and special remount window IDs. As in GPUIX, that is not general revision-stamped callback retention. Root runtime errors render a recoverable native error overlay; process listeners and frame-loop timers are uninstalled/stopped by reset. [Runtime cleanup/recovery][S-runtime] [Event implementation][S-events]

### Source compatibility and styling are real, but deliberately incomplete

The Solid host accepts semantic HTML-like tags, serializes inline SVG markup into a native source/data image, normalizes style shorthands/units, and resolves classes from a generated style manifest or a strict utility subset. Missing manifest entries/unsupported utility tokens throw. These conveniences need not be browser CSS at runtime. [Semantic/SVG/style lowering][S-universal] [Manifest/utility resolution][S-native-style] [Hyperscript][S-h]

The Solid 1 Kobalte example compiles Kobalte's published source through the universal renderer and aliases `solid-js/web` to its compatibility layer. Its adapters provide DOM-shaped focus, bounds, selectors, pointer capture, portals, observers, and global objects. This enables concrete reused application source; it does **not** prove arbitrary browser libraries work. [Kobalte dependency][S-kobalte-pkg] [Kobalte compiler aliases][S-kobalte-config] [DOM environment][S1-dom] [Web compatibility][S1-web]

Some DOM-looking capabilities are explicit approximations: host `select()` calls `focus()`, `scrollWidth`/`scrollHeight` return client dimensions, and `classList.add/remove` are no-ops in the shown host class. `preventDefault` flags operate within the JS compatibility event chain, not retroactive cancellation of completed native IME/edit dispatch. Avoid importing that whole facade into our native-first contract. [Host approximations][S-nodes-methods] [JS event facade][S-events]

### Services, Canvas, video, and resource lifetime

Desktop helpers are **JavaScript child-process integrations** for some OS services: AppleScript on macOS, PowerShell/WinForms on Windows, `zenity` on Linux, and shell launch tools. They are not app-defined GPUI extension contracts. On the native 0.9 baseline, `appWindow` exposes title/activate and explicitly declares minimize/zoom/fullscreen unavailable; current upstream 0.10 has those methods, but this adapter does not thereby expose/qualify them automatically. [Desktop implementation][S-desktop] [Current upstream methods][G-window]

The edge Canvas implementation is substantive: a versioned draw-list recorder and patched native adapter paint paths/text through GPUI, with bounds and pointer-continuity tests. It is **experimental and patched**, not stock GPUIX native 0.9/0.10 functionality. The JS recorder throws on unsupported operations such as partial clears; the native patch logs and clears invalid draw lists, and clones its retained draw list for render/prepaint closures. Adopting the capability does not require adopting those permissive native-validation or copying choices. The screenshot test checks file existence/size, not pixel-for-pixel visual correctness. [Recorder][S-canvas-recorder] [Native adapter][S-canvas-patch] [Native test][S-canvas-test]

Video-frame patches keep BGRA payloads outside JSON custom props, replace retained native frames, clear on removal, and add a macOS CoreVideo surface path. The native parity test proves replacement/clear produce different screenshot bytes; it is not a decode-throughput or zero-copy benchmark. [Binary frame patch][S-video-patch] [Surface cleanup patch][S-surface-patch] [Presentation regression][S-video-test]

The MediaBunny addon wraps `CVPixelBuffer` in N-API objects with explicit `close()` and destructor release. Its `iosurfaceHandle` **copies the bytes of an `IOSurfaceRef` pointer**; the GPUIX patch reconstructs that same-process pointer and passes it to CoreVideo. That is a same-address-space native lifetime contract, **not a serializable process-independent handle**. We must not put it into our byte-only cross-process protocol. Prefer a host-owned resource ID plus validated native import, or a separately designed shared-resource transport if needed. [Frame ownership and pointer export][S-media-frame] [Pointer import][S-video-pointer]

Also, not every media path is zero-copy: the CanvasSink bridge transforms a sample, copies RGBA into a `Uint8Array`, uploads it to `@napi-rs/canvas`, and closes original/transformed samples. A claimed zero-copy presentation path must name its exact path and platform, not characterize all Canvas/media operations. [CanvasSink copy/cleanup][S-media-canvas]

### Testing and performance evidence

There are host/logic tests and real native tests for controlled input, textarea submission, focus, file drops, node identity/reorder, selection, styling, drag, and media. The test adapter drains native events through the Solid root, so some tests exercise more than retained JSON. Native tests are skipped when the installed addon lacks test support. [Native input tests][S-input-test] [Retained identity test][S-retained-test] [Selection test][S-selection-test] [Event draining][S-test-events]

Source-port provenance has executable checks, not just README claims. The example checker verifies vendored snapshots against Git blob hashes and separately locks the source-to-port mappings for Mail, timeline, and other GPUIX fixtures. Kobalte's checker independently hashes the copied documentation examples; Mail differential testing pins both the React checkout and the Mail source blob. A snapshot hash proves reference identity, not behavioral parity of a translated application; the interactive acceptance lane supplies the latter evidence. None of these scripts was run here. [Snapshot and mapping checks][S-source-check] [Kobalte source check][S-kobalte-source] [Mail reference identity][S-mail-reference]

CI defines Ubuntu/macOS/Windows verification, exact Solid 2/Solid 1 tarball smoke, macOS live-native checks, and a patched upstream-edge lane. The public starter is copied outside repository workspaces and installed from public npm. This is stronger first-app verification than an in-workspace example alone; reading workflows is not evidence that every current job passed. [CI definition][S-ci] [Exact package smoke][S-smoke] [Public starter check][S-public-starter]

Release documentation identifies an exact post-beta candidate and explicitly separates live native stdio automation from physical foreground mouse/trackpad qualification. Its documented selection-forwarding bug is a useful lesson: a native feature and test-renderer method can exist while the production adapter fails to forward the subscription. These are author-reported historical results, not rerun by this investigation. [Qualification and physical-input limit][S-release-record]

Benchmark scripts record date, commit, CPU/OS/memory/Bun/Node and emit chat/timeline/serialization distributions. The chat benchmark times a path through its test wrapper, including wrapper flush/event work; upstream's path times its own renderer dispatch. Even matching datasets do not make those timer boundaries identical. Reference budgets are explicitly not Solid performance claims. [Report metadata][S-perf-report] [Chat measurement boundaries][S-chat-perf]

## 5. Comparison with our current source

Our [CONTEXT.md][L-context] promises Solid composition/ownership, GPUI native rendering/input, and only immutable bounded bytes across runtime boundaries. Relevant source supports those invariants; this comparison is not based on the domain document alone.

| Axis | Our implemented behavior | Concrete contrast / judgment |
| --- | --- | --- |
| Compiler/reactivity | Universal transform targets our runtime; TS erasure/source maps are separate. Stable Solid 1.9 runtime is not inferred from the compiler's 2.0 version. Client/duplicate-runtime guard is present. [Compiler][L-compiler] [Guard][L-runtime-guard] | All have real framework host renderers. GPUix Solid 2 is an RC-specific runtime; upstream GPUIX Solid is Solid 1. Do not collapse them into one “Solid binding.” |
| Runtime topology | External Bun is a child using framed stdio; QuickJS has a dedicated worker; experimental embedded Bun owns a process-scoped VM thread. [Process][L-process] [QuickJS][L-quickjs] [Embedded Bun][L-embedded] | Native rendering is independent of the JS main-thread AppKit pump. Process isolation and a byte seam have copies/queue/decode cost; N-API has lock/pump/ABI constraints. Neither implies lower latency without measurement. |
| Admission/validation | JS HostTree journals mutations, finalizes coherent props, publishes only admitted commits, and rolls back failed publication. Rust journals and validates changed-tree/extension dependencies. [JS transaction][L-host-tree] [Rust transaction][L-tree-transaction] [Native commit][L-commit] | Stronger semantics than parse-all-then-apply permissive retained mutations. Preserve them when optimizing. |
| Scheduling/pressure | CommitPump queue capacity 32, foreground yields after 16 messages or 1 MiB per turn; SurfaceRouter and event writer bound count/bytes. [Pump][L-pump] [Router][L-router] [Writer][L-writer] | Stronger explicit pressure/foreground work policy than the inspected N-API UI command channel. Not a guarantee of fixed memory for all active application state. |
| Event identity | Surface/epoch/revision/sequence checks plus bounded current/previous callback generations. [Acceptance][L-event-admission] [Listener registry][L-listeners] | Stronger same-node callback history; theirs favor simple current-handler dispatch and remount IDs. Callback safety is worth preserving. |
| Native state/resource ownership | Side maps prune from the deleted change set; epoch reset clears native state/routes/calls. NativeView has explicit mount/update/unmount. [Cleanup][L-cleanup] [NativeView][L-native-view] | We do not need a cleanup rewrite just because GPUIX has one. Learn from its empty-frame teardown test and explicit GPU image release. |
| Input | Core state retains selection, marked text, edit sequence, history and scrolling; controlled application waits for acknowledgement and no active composition. [State][L-input-state] [Synchronization][L-input-controlled] [Platform input][L-input-ime] | An advantage over string-matching echo suppression. Core input is owned by SolidRoot; GPUIX owns an editor Entity. Native control isolation deserves profiling, not a declaration that one editor is faster. |
| Native render cost | Patches reconcile affected state, but successful commits notify SolidRoot and ordinary primitives recursively rebuild GPUI elements. [Notify][L-notify] [Render][L-render] [Primitive recursion][L-recursion] | Both primitive tree renderers have broad root construction. Fine-grained Solid and incremental wire output are not automatic incremental layout/scene regions. |
| Retained style cost | StoredNode has inline `Option<Style>`; native Style already uses f32/color u32/closed enums. Journal captures clone StoredNode. [Storage][L-tree-storage] [Style][L-native-style] [Journal][L-tree-transaction] | GPUIX's Arc/interner still offers a plausible memory/journal benefit. Do not transfer its larger String/f64 StyleDesc byte numbers to our compact representation. |
| Virtualization | Public VirtualList retains only the current row-owner window; native list fills missing rows with estimates and emits viewport range. Data snapshot/diff remains O(data length). [JS owners][L-list-js] [Native list][L-list-rust] [Bounds][L-work-bounds] | Strong default owner/mount bound. GPUIX can do mounted-window lists too; its ordinary retained-child list is not automatically framework-memory bounded. |
| Extensions/services | Exact provider/catalog/entry/version identity, adapter validation before publication, host-generated component/client types, worker/foreground commands, bounded executor/cancellation. [Extension contract][L-extensions] [Generator][L-native-module] [Commands][L-native-commands] [Execution][L-native-executor] | Much better fit for app-owned Rust state/services than a fixed addon plus JS OS shell integrations. This is a differentiator, not accidental overhead to remove. |
| Styling/productivity | Explicit native Style and Row/Column helpers; onHoverChange exists, but core Style does not contain nested hover/active, class manifests, or CSS unit strings. [Style contract][L-style] [Layout][L-layout] [Host props][L-host-props] | Their declarative native pseudo styles and class/shorthand lowering reduce app boilerplate. Optional native controls already have their own styling, so this is specifically a core authoring gap. |
| Selection/search | Core selectable-text dragging is tied to one node; selectable nested styled Text is explicitly rejected. Optional TextView provides selection/highlight methods, and Input has search APIs. [Core selection][L-selection] [Validation][L-tree-storage] [TextView][L-text-view] | GPUIX's renderer-wide cross-element selection/search is a real alternative capability. Do not falsely say we lack all text selection/search. |
| Images/media | Our core Image is source/sourceSet/fallback driven, with target-size buckets, source/decode budgets, shared variants, tasks and explicit GPU release. No core Canvas/video-frame/raw-pixel image update API was found in scoped public host/protocol/native source searches. [Image props][L-host-props] [Image provider][L-image] [Ownership][L-image-resource] | GPUIX 0.10 has raw live-image setters; GPUix Solid's patched media lane goes further. We already have substantial image-resource engineering; media needs a distinct declared resource contract, not data URLs each frame. |
| Testability | Public TestHost inspects real wire output and delivers semantic events, explicitly not layout/paint/platform emulation. Rust/native correctness tests and a production profiler also exist. [Public TestHost][L-testing] [Measurement][L-perf] | Their public JS GPU/live-native locators cover the missing seam between framework updates and native hit-tested interaction. Extend ours; do not replace protocol tests with screenshots. |
| Delivery/HMR | Managed Bun HMR, captureState/epochs, staged QuickJS candidate validation, native rebuild/session replacement. Distribution qualifies the QuickJS website; embedded Bun static packaging remains experimental. [HMR state][L-app-reload] [QuickJS staging][L-reload] [Supervisor][L-dev-session] [Distribution][L-distribution] | Our lifecycle contract is richer. Their prebuilt default host/first-app path is much easier. Packaging breadth and qualification need to catch up with our runtime ambitions. |

### What we specifically got wrong or under-prioritized

1. **The external-consumer happy path exposes too much SDK implementation.** A native consumer needs our matching source checkout, unpublished Rust crate, workspace-root Cargo patches/debug profiles, bindings generation, and aligned npm packages. `doctor` diagnoses this but does not remove it. Our packed-consumer smoke already exists; its Rust host only prints a test catalog and does not link GPUI/open a window. The mistake is treating that as enough first-app coverage, not “we have no consumer tests.” [Consumer prerequisites][L-get-started] [Packed host stub][L-pack-smoke]
2. **Small Solid updates still invalidate the primitive native root.** This is a real architectural limitation, not a proven regression against GPUIX, which also broadly rebuilds ordinary hosts. Our own rejected cached-pane experiment demonstrates why adding `.cached()` indiscriminately can freeze visible content. Use native owners/regions only with explicit invalidation from commits, native edits, scrolling, animation, async resources, geometry and inherited styles. [Root/recursion][L-render] [Rejected-cache lesson][L-cache-limits]
3. **Our public test helper emits the wrong default selection units for Unicode.** `TestHost.dispatch({type:"input"})` sets the default selection to `utf8ByteLength(event.text)`. Native input selections are UTF-16 offsets, forwarded as-is to JS. For `新值`, the helper emits 6 instead of 2; for `🙂`, 4 instead of 2. Existing Unicode TestHost coverage checks value/text, not selection. This is a code-backed defect in the test helper, not a claimed native editor failure. Only arithmetic was independently checked here; no production bug reproduction or code fix was performed. [Helper][L-testing] [Native range contract][L-input-ime] [Native event emission][L-input-event] [Event forwarding][L-input-dispatch] [Existing Unicode test][L-testing-unicode]
4. **Authoring/acceptance work has lagged behind protocol/native capability work.** Their native hover/active declarations, class lowering, source-pinned Mail/timeline/Kobalte fixtures, and public native automation exercise things a JSON tree cannot prove. We have strong transactional invariants and a substantial GPUI Kit catalog, so the response is targeted ergonomic and acceptance layers, not a wholesale DOM compatibility runtime. [Native controls][L-controls] [Native pseudo styles][G-styles] [Source provenance][S-source-check] [Kobalte source check][S-kobalte-source] [Source compatibility][S-kobalte-config] [Automation exports][S-automation]

## 6. Prioritized lessons to adopt

| Priority | Recommendation | Acceptance / constraints |
| --- | --- | --- |
| **P0 — small, definite defect** | Correct TestHost's default input selection units and state them explicitly in the public testing contract. | One key test uses both CJK and an astral character and asserts callback selection plus controlled acknowledgement, not just the text value. Research only: no fix in this task. |
| **P0 — first-app delivery** | Publish one standalone scaffold with exact npm/native identity and a prebuilt default host for applications using stock capabilities; keep custom Rust builds explicit. | Copy outside this workspace; install, typecheck, start a real window, edit a signal, type into input, close, and launch a packaged result on each qualified platform. Version the template with its matching native artifact; do not download moving `main` plus unrelated `latest`. |
| **P1 — native acceptance API** | Add an opt-in public JS automation/native-test layer: stable locators, actual painted bounds/text, native click/typing/drag/wheel, screenshots, and an owned clock where supported. | Drive the same production event/commit path. Test update → paint → hit-test → callback → paint. Include follow-up action after selection/drag, stale listener/root teardown, and an unsupported-platform result. Do not auto-enable a powerful command server merely because stdin is piped. |
| **P1 — application-level proof** | Port one exact-source multi-pane application and one editable timeline/large-history workload with explicit framework/native substitutions. | Record source SHA/blob hashes, meaningful interactions, resize/scroll displacement, selection/copy, IME/manual acceptance, and native-resource cleanup. Capability docs should name published, source-edge, and unqualified lanes. Avoid broad snapshot counts without interaction invariants. |
| **P1 — memory/cost experiment** | Benchmark native style interning and immutable sharing using GPUIX's collision-safe, reclaimable design. | Measure our actual `Style`/StoredNode, journal allocation churn, mount, style updates, drag-produced style churn, and root collapse. Keep binary wire DTOs canonical; do not create a second style-reference wire format until evidence warrants it. |
| **P1 — native invalidation** | Profile core input/root construction and isolate hot native state into retained GPUI owners/regions where measurements justify it. | Count region/root renders and verify changed pixels/content. Include native-only input, scrolling, animation, image load, inherited style and repeated resize. GPUIX's separate editor Entity is an ownership example, not proof that its whole tree is incremental. |
| **P2 — lightweight authoring** | Add declarative native hover/active/focus refinements and a strict optional shorthand/utility compiler over our real Style contract. | Lower once to validated native semantics. Unsupported classes/units fail clearly. Preserve Row/Column and existing native controls. Do not introduce global fake DOM APIs just for convenience. |
| **P2 — media resource seam** | Add a declared host-owned live-image/Canvas/media module when an actual app needs it. | Separate paint commands, encoded media, decoded frame ownership and GPU resources. Use bounded binary data or opaque validated resource identities; define close/replacement/cancellation/epoch behavior. Never transport an in-process pointer as if byte encoding made it portable. |
| **P2 — product distribution** | Reuse the signed-updater integration pattern as an application-owned native module after generic packaging is qualified. | Own signed feed identity, bounded downloads, signature validation, installation rollback and explicit relaunch policy. Updater correctness/security is a separate review from UI rendering. |

**Do not adopt:** unconditional fallback to older mutation APIs; broad silent CSS approximation; current-only callbacks for our revisioned transport; unbounded UI command queues; same-process IOSurface pointer encoding across processes; or published performance claims based on different machines, JS engines, GPUI forks, test-support builds, and timer boundaries.

## 7. Verification limits and scoped negative findings

- Remote repository scripts, install hooks, binaries, builds, tests, application windows, and benchmarks were **not executed**. Submodules and transitive dependencies were not initialized. Native GPU/platform/physical input behavior is therefore traced source, not locally qualified behavior.
- Git clones, git metadata, file reads, scoped searches, GitHub API GETs, and npm metadata GETs were performed. A confined JavaScript arithmetic check confirmed UTF-16/UTF-8 lengths for the TestHost finding; it did not import either repository.
- Every immutable reference definition was checked against the pinned Git objects for commit/file existence and valid line ranges; no missing, duplicate, or unused reference definitions remained. This is citation integrity verification, not execution of the cited implementations.
- CI files prove defined gates; historical release docs record authors' qualification. Neither proves the inspected HEAD passed those gates. Public npm metadata proves observed availability/tags, not package runtime behavior or binary reproducibility.
- Negative claims are scoped: application Rust registration was searched in GPUIX's native exports/registry/framework entrypoints and GPUix Solid's public source; local Canvas/video/live pixel updates were searched in public host types, protocol fields, renderer dispatch, and native/components source; renderer-wide selection was contrasted with core selection plus optional TextView/Input APIs. These are not universal claims about what downstream apps could add.
- Public JSX types were cross-checked against real native dispatch: GPUIX's typed `canvas` is the notable mismatch. GPUix Solid's patched Canvas/video capabilities are not assumed to exist in the public native 0.9 package.
- Nothing here establishes “N-API is faster,” “Solid beats React,” end-to-end input latency, real display FPS, GPU memory totals, arbitrary Kobalte/browser compatibility, zero-copy media throughout, or general cross-platform product readiness.
- Documentation/website scope was inspected through `docs/README.md`, `examples/website/README.md`, and the website's documentation loader. This note changes no API, runtime, guide, catalog, or navigation. The website imports top-level `docs/*.md`, not `.scratch/` notes, so no public documentation/API regeneration or website synchronization is required. No build/website checks were run for this research-only note. Recommendations remain proposals, not capability claims.

## Immutable source references

[G-native-pkg]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/package.json#L1-L90
[G-react-pkg]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/react/package.json#L81-L103
[G-solid-pkg]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/solid/package.json#L86-L107
[G-submodule]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/.gitmodules#L1-L4
[G-cargo]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/Cargo.toml#L22-L105
[G-solid-host]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/solid/src/host.ts#L1-L22
[G-react-host]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/react/src/reconciler/host-config.ts#L180-L288
[G-react-update]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/react/src/reconciler/host-config.ts#L350-L379
[G-solid-compiler]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/solid/src/bun-plugin.ts#L1-L64
[G-solid-root]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/solid/src/root.ts#L41-L168
[G-queue]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/js/mutations.ts#L11-L72
[G-bridge]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/renderer.rs#L898-L935
[G-automation-protocol]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/js/automation/protocol.ts#L1-L12
[G-macos]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/renderer.rs#L1121-L1226
[G-tick]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/renderer.rs#L1364-L1453
[G-loop]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/js/runtime.ts#L49-L84
[G-threaded]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/renderer.rs#L1260-L1360
[G-queries]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/renderer.rs#L995-L1062
[G09-bridge]: https://github.com/remorses/gpuix/blob/7ac9880abd8e91e5bf0e4feb0fa850729cf95a68/packages/native/src/renderer.rs#L819-L856
[G09-tick]: https://github.com/remorses/gpuix/blob/7ac9880abd8e91e5bf0e4feb0fa850729cf95a68/packages/native/src/renderer.rs#L1228-L1285
[G-parser]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/renderer.rs#L6153-L6387
[G-apply]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/renderer.rs#L6389-L6469
[G09-apply]: https://github.com/remorses/gpuix/blob/7ac9880abd8e91e5bf0e4feb0fa850729cf95a68/packages/native/src/renderer.rs#L5744-L5807
[G-tree-style]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/retained_tree.rs#L15-L79
[G-style-table]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/retained_tree.rs#L81-L181
[G-tree-mutations]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/retained_tree.rs#L208-L371
[G-render]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/renderer.rs#L4808-L5055
[G-custom-trait]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/custom_elements/mod.rs#L199-L258
[G-list]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/renderer.rs#L5058-L5199
[G-list-child]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/renderer.rs#L3857-L3952
[G-infinite]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/examples/infinite-chat.tsx#L1-L45
[G-events]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/js/renderer-state.ts#L135-L305
[G-payload]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/element_tree.rs#L22-L131
[G-registry]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/custom_elements/mod.rs#L269-L450
[G-test]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/test_renderer.rs#L1-L275
[G-key-events]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/renderer.rs#L3533-L3618
[G-input-state]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/custom_elements/input.rs#L310-L555
[G-input-ime]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/custom_elements/input.rs#L1490-L1629
[G-input-echo]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/custom_elements/input.rs#L678-L751
[G-selection]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/text/selection.rs#L1-L176
[G-search]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/renderer.rs#L3620-L3694
[G-styles]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/renderer.rs#L5684-L5721
[G-focus-style]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/renderer.rs#L4105-L4132
[G-host-style]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/js/host.ts#L119-L229
[G-host-element-types]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/js/host.ts#L231-L244
[G-lib]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/lib.rs#L14-L90
[G-window]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/renderer.rs#L1629-L1777
[G-react-runtime]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/react/src/reconciler/renderer.ts#L293-L447
[G-refresh]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/react/src/reconciler/reconciler.ts#L25-L50
[G-solid-runtime]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/solid/src/renderer.ts#L34-L103
[G-dev]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/scripts/dev.ts#L1-L18
[G-cli]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/cli/src/cli.ts#L15-L121
[G-ci]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/.github/workflows/ci.yml#L21-L172
[G-compile]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/examples/compile-chat.ts#L119-L162
[G-hermes]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/hermes/build.ts#L14-L61
[G-update-verify]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/updater.rs#L1117-L1134
[G-update-api]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/src/updater.rs#L1280-L1345
[G-chat-perf]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/examples/chat.perf.test.tsx#L117-L221
[G-serde-bench]: https://github.com/remorses/gpuix/blob/4ecca30f68057b4d9830d32675ba4ed999eeeaaa/packages/native/examples/bench_serde.rs#L1-L110

[S-pkg]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/package.json#L1-L67
[S1-pkg]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid1/package.json#L78-L96
[S-runtime]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/runtime.ts#L165-L335
[S-edge]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/.gpuix/edge.json#L1-L10
[S-edge-script]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/scripts/gpuix-edge.mjs#L76-L99
[S-media-pkg]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/mediabunny/package.json#L15-L42
[S-media-runtime]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/mediabunny/src/videotoolbox-mediabunny.ts#L75-L113
[S-universal]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/host/universal.ts#L113-L437
[S-vite]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/vite.ts#L1-L49
[S-client]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/client-runtime.ts#L1-L48
[S-host-parity]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid1/scripts/check-host-parity.ts#L16-L26
[S-mutations]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/host/mutations.ts#L100-L358
[S-root]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/root.ts#L53-L463
[S-batch]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/batch-renderer-adapter.ts#L50-L173
[S-nodes-order]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/host/nodes.ts#L647-L700
[S-root-cleanup]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/root.ts#L466-L484
[S-selection]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/primitives/create-text-selection.ts#L1-L28
[S-window-size]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/hooks/use-window-size.ts#L14-L77
[S-runtime-test]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/test/runtime.test.ts#L156-L196
[S-starter]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/templates/solid2-vite-bun/package.json#L1-L22
[S-readme-dev]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/README.md#L147-L157
[S-events]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/host/events.ts#L137-L289
[S-native-style]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/native-style.ts#L39-L87
[S-h]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/h.ts#L33-L118
[S-kobalte-pkg]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/examples/solid1-kobalte/package.json#L5-L28
[S-kobalte-config]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/examples/solid1-kobalte/vite.config.ts#L7-L33
[S1-dom]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid1/src/dom-environment.ts#L179-L229
[S1-web]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid1/src/web.ts#L49-L193
[S-nodes-methods]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/host/nodes.ts#L92-L307
[S-desktop]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/desktop.ts#L46-L388
[S-canvas-recorder]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/host/canvas.ts#L97-L380
[S-canvas-patch]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/patches/gpuix/canvas-v1.patch#L16-L188
[S-canvas-test]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/test/native-canvas-parity.test.ts#L22-L102
[S-video-patch]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/patches/gpuix/video-frame-v1.patch#L1-L132
[S-surface-patch]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/patches/gpuix/video-frame-iosurface-v1.patch#L12-L101
[S-video-test]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/test/native-video-frame-parity.test.ts#L27-L72
[S-media-frame]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/mediabunny/native/videotoolbox/frame.mm#L48-L166
[S-video-pointer]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/patches/gpuix/video-frame-v1.patch#L142-L194
[S-media-canvas]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/mediabunny/src/gpuix-canvas-sink.ts#L51-L101
[S-input-test]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/test/native-event-input-parity.test.ts#L33-L253
[S-retained-test]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/test/native-retained-tree-parity.test.ts#L198-L219
[S-selection-test]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/test/native-selection-layout-parity.test.ts#L30-L134
[S-test-events]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/testing.ts#L152-L173
[S-source-check]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/examples/counter/scripts/check-upstream-gpuix-source.mjs#L10-L138
[S-kobalte-source]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/examples/solid1-kobalte/scripts/check-upstream-source-parity.mjs#L6-L37
[S-mail-reference]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/scripts/mail-parity/reference.mjs#L6-L87
[S-ci]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/.github/workflows/ci.yml#L16-L149
[S-smoke]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/scripts/smoke-package.mjs#L34-L140
[S-public-starter]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/.github/workflows/public-starter.yml#L18-L61
[S-release-record]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/docs/release-candidate.md#L5-L40
[S-perf-report]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/scripts/perf-report.mjs#L25-L43
[S-chat-perf]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/examples/counter/src/benchmarks/chat.tsx#L59-L171
[S-automation]: https://github.com/jhomra21/gpuix-solid/blob/ad384ff3755761269902dcd7db26ef6163fa722c/packages/solid/src/automation/index.ts#L1-L43

[L-context]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/CONTEXT.md#L1-L93
[L-cargo]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/Cargo.toml#L15-L72
[L-js-pins]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/package.json#L29-L41
[L-compiler]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui-vite/src/transform.ts#L5-L32
[L-runtime-guard]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/runtime-guard.ts#L1-L44
[L-host-config]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/renderer/host-config.ts#L78-L204
[L-host-tree]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/renderer/host-tree.ts#L108-L225
[L-tree-transaction]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/tree/transaction.rs#L34-L164
[L-commit]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer.rs#L557-L704
[L-process]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/transport.rs#L537-L578
[L-quickjs]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/runtime/quickjs.rs#L1-L174
[L-embedded]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/runtime/embedded.rs#L1-L104
[L-pump]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/host/commit_pump.rs#L13-L117
[L-router]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/renderer/surface-router.ts#L75-L174
[L-writer]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/transport.rs#L147-L319
[L-listeners]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/renderer/nodes.ts#L45-L176
[L-event-admission]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/renderer/root-container.ts#L864-L879
[L-cleanup]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer.rs#L848-L927
[L-native-view]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/native/component.rs#L234-L270
[L-input-state]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer/input.rs#L46-L81
[L-input-controlled]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer/input.rs#L577-L672
[L-input-ime]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer/input.rs#L1733-L1812
[L-input-event]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer/input.rs#L942-L960
[L-input-dispatch]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/renderer/dispatch.ts#L236-L285
[L-notify]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer.rs#L728-L738
[L-render]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer.rs#L1268-L1293
[L-recursion]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer/paint/mod.rs#L285-L347
[L-tree-storage]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/tree.rs#L83-L141
[L-native-style]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/protocol.rs#L809-L879
[L-list-js]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/index.ts#L167-L260
[L-list-rust]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer/paint/virtual_list.rs#L16-L120
[L-extensions]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer/extensions.rs#L114-L258
[L-native-module]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/native/module.rs#L228-L333
[L-native-commands]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/native/module.rs#L24-L79
[L-native-executor]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/native/executor.rs#L17-L138
[L-style]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/style.ts#L61-L131
[L-layout]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/layout.ts#L6-L88
[L-host-props]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/renderer/types.ts#L247-L315
[L-selection]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer/input.rs#L1212-L1270
[L-text-view]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/components.ts#L5025-L5055
[L-image]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer/paint/image.rs#L22-L134
[L-image-resource]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/renderer/paint/image_resource.rs#L192-L258
[L-testing]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/testing.ts#L30-L164
[L-testing-unicode]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/tests/testing.test.ts#L195-L236
[L-perf]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/docs/performance-analysis.md#L9-L48
[L-app-reload]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui/src/application.ts#L142-L176
[L-reload]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/runtime/reload.rs#L170-L248
[L-dev-session]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/packages/solid-gpui-vite/src/dev-session.ts#L27-L172
[L-distribution]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/docs/distribution.md#L1-L40
[L-get-started]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/docs/getting-started.md#L14-L59
[L-pack-smoke]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/scripts/package-pack-smoke.ts#L415-L456
[L-cache-limits]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/docs/performance-analysis.md#L304-L321
[L-work-bounds]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/docs/performance-analysis.md#L325-L349
[L-controls]: https://github.com/Cyenoch/solid-gpui/blob/e9bc4389c5394e5de1fecb6758d2d06e48877fdb/crates/solid-gpui/src/components/mod.rs#L144-L246
