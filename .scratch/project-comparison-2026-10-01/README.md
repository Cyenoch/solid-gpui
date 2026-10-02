# Solid/GPUI project comparison

Assessment date: **2026-10-01**. Local baseline: `Cyenoch/solid-gpui` at `e9bc4389c5394e5de1fecb6758d2d06e48877fdb`.

[Chinese edition](README.zh-CN.md).

This report answers four questions: what we do well, what the alternatives do well, what we got wrong, and what to adopt. It compares implementation boundaries rather than repository popularity or README feature counts. Source observations do not establish relative speed, memory use, binary size, or platform qualification.

### Reading the evidence

- **Implemented:** found in the pinned source path; not necessarily executed during this research.
- **Tested here:** a listed check was run against this local baseline.
- **Published:** a first-party registry or release endpoint exposes an artifact; its presence does not qualify destination-machine behavior.
- **Not found in inspected scope:** a bounded source search did not locate a capability. This is narrower than claiming the ecosystem cannot provide it.
- **Recommendation:** our synthesis, not a capability already supplied by this project or the alternative.

### Compared revisions and project relationships

| Project | Pinned HEAD | Main shape | Most useful lesson |
| --- | --- | --- | --- |
| `heyhuynhgiabuu/solid-gpui` | `05f84e9f17cd8360b9af92f615a884527fa3a986` | Solid 2 RC renderer → JSON lines → Rust helper | Lifecycle/GUI evidence and domain-sized native components |
| `remorses/gpuix` | `4ecca30f68057b4d9830d32675ba4ed999eeeaaa` | React or its own Solid 1 adapter → N-API batched JSON → GPUI fork | Prebuilt first-app path, GPU/live automation, native styles and reclamation |
| `jhomra21/gpuix-solid` | `ad384ff3755761269902dcd7db26ef6163fa722c` | Separate Solid 1/2 adapters → exact GPUIX native 0.9.0 | Clean-consumer and exact-source application acceptance |
| `lxsmnsyc/solid-gpui` | `196aa6edc779cb39f37a3ade4517ed197ad58813` | Solid 2 RC renderer → JSON lines → Rust host | Public compiler, normalized styles, recorded canvas |
| Our `Cyenoch/solid-gpui` | `e9bc4389c5394e5de1fecb6758d2d06e48877fdb` | Stable Solid 1.9 → framed Bebop → runtime-independent Rust host | Transactional/native ownership foundations to retain |

These are not four independent native engines. `jhomra21/gpuix-solid` depends on GPUIX; current GPUIX also contains its own Solid adapter with host ideas derived from that project. The separate adapter's default native dependency is 0.9.0, not current upstream 0.10.0; its experimental Canvas/video lane applies native patches to another pinned source revision. [Dependency/provenance evidence](gpuix-alternatives.md#2-relationship-what-actually-depends-on-what), [other pinned identities](solid-alternatives.md#1-pinned-identities-versions-and-observation-dates).

## Executive judgment

Our strongest differentiator is not merely “Solid renders through GPUI.” It is the ownership and correctness model around that renderer: transactional publication, bounded cross-runtime traffic, native-owned editing, exact contracts, retained native components, and development recovery. Those are worth preserving.

The main improvement opportunity is to turn those foundations into a smaller, easier default product. External setup exposes SDK internals; primitive native invalidation is coarser than Solid's reactive updates; exact build-source identity and interface identity are conflated; release/capability prose has drifted. None of these findings justifies replacing the byte boundary with direct per-property FFI without measurements.

In module-design terms, reduce the **Interface** a first application must learn without removing the **Implementation** that earns its keep. The runtime seam already has real adapters; it is not purely hypothetical abstraction. The opportunity is greater **Depth** at the application/tooling interface: hide SDK vendoring and artifact alignment inside a supported scaffold or default host instead of distributing that knowledge across callers. Code size alone is not a useful measure of depth.

## 1. What we do well

### A real Solid renderer, with coherent commit boundaries

Our universal renderer updates host nodes through Solid's mutation operations. `HostTree` finalizes properties after the transaction, emits a Snapshot once and Patches subsequently, and rolls back failed publication. Rust validates the candidate before revision/state publication. This protects coherent multi-property and structural updates instead of relying on a frame not happening between native setter calls. [Implementation and source links](local-baseline.md#implemented-strengths).

The rollback scope is the private/published host tree and native publication state. It is not a database transaction for arbitrary Solid signal writes, application I/O, `onMount` effects, or already-running native services.

### Correct ownership of native interaction state

Native text, UTF-16 selection, IME composition, caret scrolling, undo, focus, and list geometry do not belong in an asynchronously mirrored JS model. We retain those in Rust, synchronize controlled state using edit acknowledgements, and expose semantic events. NativeView gives custom Rust components an explicit lifetime and prop-update contract. This is the right direction for a desktop framework, including when services remain in Bun. [Input and NativeView evidence](local-baseline.md#implemented-strengths).

### A boundary that can evolve without exposing GPUI or JS handles

External Bun and embedded engines use the same immutable-byte interface. Schema generation, strict structural guards, semantic validation, and exact Extension identities address mismatches and malformed traffic. Queue bounds and foreground yield budgets address sustained producers. These are architectural properties, not proof of sandboxing or a latency target. [Protocol and scheduling evidence](local-baseline.md#implemented-strengths).

The outer protocol is binary Bebop, but Native Module props/arguments/results include strict JSON DTOs carried in its byte fields. It would be inaccurate to call the complete system zero-copy or free of JSON serialization. [Native Module decision](../../docs/adr/0016-rust-owned-native-modules.md).

### Engineering beyond a counter demo

The current project includes generated Rust services/components, virtualized ownership, routing, themes, a shared native/WASM gallery, application-config-aware tests, reload recovery, and a verified QuickJS gallery packaging pipeline. More importantly, it has meaningful failure/lifecycle tests and explicitly separates source claims, headless correctness, native behavior, and display performance. [Local evidence](local-baseline.md), [native composition](../../docs/native-composition.md), [delivery boundaries](../../docs/distribution.md).

This does not mean every exposed feature or platform is production-qualified. Component presence, a successful build, and a passing state test answer different questions.

The two independent process-host alternatives also enter their retained root to construct ordinary elements, so they do not demonstrate that our primitive render-region limitation is solved simply by choosing a smaller JSON host. Conversely, `lxsmnsyc`'s independent input Entity is a useful example of state ownership locality. [Comparative source table](solid-alternatives.md#2-architecture-comparison-at-the-actual-seams).

## 2. What the alternatives do well

### `heyhuynhgiabuu/solid-gpui` — evidence and domain-sized native functionality

Pinned HEAD: `05f84e9f17cd8360b9af92f615a884527fa3a986`.

- The small renderer/client/helper decomposition is inspectable. Apply replies correlate requests, report how many operations succeeded, and poison the client after partial failure rather than silently resuming a divergent stream.
- There are lifecycle state-count checks, cross-language fixtures, real-window tests, and actual successful hosted macOS/Windows/Linux GUI jobs at the inspected HEAD. This is better evidence than a workflow file alone. GUI jobs are not a mandatory failure gate, and they are not physical IME or clean-machine release acceptance.
- Markdown is a domain-sized native component: Rust owns parsing/highlighting and content-specific caches rather than exposing every parsing node to JS.
- Utility classes normalize to host style mutations with precedence and unknown-token diagnostics. That pattern is useful; the mapper has concrete font-weight and percent-length gaps that should not be copied.

**Learn:** lifecycle ownership counts, explicit partial-failure diagnostics, closed style sugar, and native domain components where justified. **Do not replace our atomic publication with their partial apply/poison model.** Native selection/candidate geometry is incomplete, controlled input pushes reset transient state, and list painting is virtualized while ownership still retains all children. [Detailed assessment and pinned sources](solid-alternatives.md#3-heyhuynhgiabuusolid-gpui).

### `lxsmnsyc/solid-gpui` — authoring ergonomics and a reusable compiler

Pinned HEAD: `196aa6edc779cb39f37a3ade4517ed197ad58813`.

- `/compiler` exports synchronous/asynchronous compilation independently of Vite; the Vite adapter uses that same transform. We have the canonical transform, but do not currently expose it through a public compiler subpath.
- Style normalization supports X/Y spacing, explicit length units, per-axis overflow, flex basis, aspect ratio, and state styles. Parsed native style is retained rather than reparsed on every paint.
- Recorded canvas runs the JS `draw` callback reactively, sends drawing records, and replays them in Rust. It does not run JS callbacks during native paint. This is a sensible extension pattern for diagrams.
- Input is a separate retained native Entity with shaped selection/hit testing/IME geometry, and equal-value controlled echoes do not clear editing state.
- The minimal `render` entry point and version-paired platform-binary package mechanism show how a default application can avoid custom Rust setup. The inspected public metadata does **not** prove those platform packages are available today.

**Learn:** compiler-only access to our own canonical transform, strict style normalization, and a bounded recorded-paint NativeView if a consumer needs it. **Do not adopt** singleton session ownership, permissive live-tree edits, current-handler routing, or unbounded transport queues. There is also a static InputEvent shape mismatch: native emits an object, Session unwraps `value` to a string, while declarations/examples expect `event.value`. [Detailed assessment and pinned sources](solid-alternatives.md#4-lxsmnsycsolid-gpui).

### `remorses/gpuix` — delivery and native verification deserve attention

Pinned HEAD: `4ecca30f68057b4d9830d32675ba4ed999eeeaaa`.

- Prebuilt native npm targets and a no-custom-Rust first-app path exist. The observed public native/React/Solid packages are 0.10.0. Its scaffold chooses moving `main` and `latest`; adopt the low ceremony but use our exact version-paired template/artifacts instead.
- Public JavaScript testing reaches native GPU-backed rendering, painted bounds, screenshots, and simulated interaction. Live automation complements that test renderer. Linux lacks its test-renderer lane, so this is not uniform cross-platform GPU test coverage.
- Native styles use shared `Arc` values, collision-confirmed interning, no-op pointer/equality checks, and reclamation of unreferenced styles. This is a worthwhile local memory/journal experiment, not proof it improves our already more compact binary style representation.
- Hover/active/focus-visible refinements stay native; ordinary interaction appearance does not require a JS signal/bridge update at every transition.
- It implements renderer-wide cross-element text selection/search, raw live-image updates, and a minisign-verified updater. We have per-node selection, optional TextView/Input capabilities, and substantial image ownership already; these are specific additional product capabilities, not a total absence on our side.

**The architecture tradeoff matters:** UI mutation transport is batched JSON through in-process N-API, not stdio IPC and not `bun:ffi`. On macOS, JS `tick()` pumps AppKit; long synchronous JS work can delay that pump. Other desktop platforms run GPUI on a native thread with tree locks and an unbounded command channel. It parses styles before tree mutation, but retained-tree operations remain permissive and are not our full revision/ownership/rollback contract. Ordinary native subtrees still reconstruct from the root. [Detailed implementation evidence](gpuix-alternatives.md#3-remorsesgpuix-implemented-behavior-and-tradeoffs).

**Learn:** first-app delivery, native acceptance interfaces, native state styles, reclaimable sharing, and precise renderer-wide selection. **Do not infer that N-API is faster or transplant the macOS JS pump into our native-owned host.** Also, its public `canvas` type is not backed by a default Canvas native factory at this HEAD.

### `jhomra21/gpuix-solid` — realistic application and package acceptance

Pinned HEAD: `ad384ff3755761269902dcd7db26ef6163fa722c`.

- It keeps Solid 1 and Solid 2 packages explicit, with actual runtime probes. Its source-pinned application ports verify source blobs and mappings rather than calling a loosely similar screenshot a port.
- Exact tarball smoke checks and a starter installed outside the workspace address the real consumer seam. Native interaction tests cover input, focus, selection, identity/reorders, drops, and styling; missing addon test support causes skips, so a defined lane is not proof every job exercised it.
- Strict utility/manifest style lowering and semantic-tag/SVG lowering improve productivity without requiring CSS in the native renderer.
- The patched Canvas/video lane contains real drawing/frame ownership code and acceptance fixtures. It is an experimental source-edge capability, not a stock published native 0.9/0.10 feature. Its IOSurface pointer-byte path is valid only inside the same address space; it must not cross our process/thread protocol as a pretend portable handle.

**Learn:** one exact-source multi-pane/large-history reference app, explicit native substitutions, clean-consumer installation, and update → paint → hit-test → callback acceptance. **Avoid:** its broad fake-DOM compatibility surface, no-op approximations, legacy mutation fallback, and extra JSON rewrite passes. Kobalte source examples working under its aliases do not prove arbitrary DOM libraries work natively. [Detailed evidence](gpuix-alternatives.md#4-jhomra21gpuix-solid-implemented-behavior-and-tradeoffs).

## 3. What we got wrong, or should reconsider

| Finding | Classification | Why it matters | Recommended response |
| --- | --- | --- | --- |
| TestHost defaults input selection to UTF-8 byte length instead of UTF-16 offsets | Confirmed and locally reproduced test-helper defect | Unicode input tests generate incorrect selection/controlled-ack state | Use the native selection unit; one CJK + astral-character test should assert offsets and acknowledgement |
| Host release and candidate checks still assert `protocol=v5`, while the current host emits v6 | Confirmed source-level verification defect; rehearsal not run here | A correctly rebuilt current host cannot pass those stale exact-string gates | Derive protocol/release expectations from canonical metadata and rerun the affected rehearsal |
| README says Pages is awaiting first deployment, but the public site is live | Confirmed documentation error | Users cannot reliably infer current capability from entry-point documentation | Keep capability status in one authoritative source and remove stale summaries |
| Ordinary applications must align npm and source revisions and reproduce workspace-root Cargo patches/profiles | Confirmed onboarding burden; a product-design weakness | The SDK's own vendoring/build requirements become application setup work | Ship a standalone scaffold; provide a prebuilt default-host lane when no custom native module is needed |
| Accepted patches notify one SolidRoot, whose ordinary primitive children are recursively constructed | Confirmed render-granularity limitation, not a benchmark result | A tiny JS patch can still incur broad native element-construction/layout work | Design native render regions with explicit dependency ownership, then compare end-to-end performance |
| Catalog digest hashes full included implementation source as well as interface description | Confirmed coupling; intentional strict lock with debatable responsibility | Source-only and line-ending changes invalidate bindings even when the public contract is unchanged | Separate semantic contract identity from exact SDK/build identity; preserve explicit checks for both |
| Compiler transform exists but is not publicly exported independently; common style shorthand/state authoring is narrow | Concrete authoring/tooling gaps, not runtime correctness bugs | Consumers either use Vite lifecycle or reproduce transform integration; ordinary layout recipes are verbose | Expose the canonical compiler and normalize a diagnosed style subset to existing contracts |
| Public TestHost intentionally stops at semantic wire events, not native paint/hit testing | Acceptance-interface gap; existing Rust/native tests are not absent | JS authors cannot use the public helper to prove native geometry, interaction targeting, and rendered outcomes | Add an opt-in native test/automation interface over the production path, with honest platform limits |
| Multiple native runtime modes and experimental WASM/embedded packaging coexist with narrower qualification | Strategic prioritization risk | Maintenance breadth can grow faster than first-app and delivery maturity | Name one simplest default; keep experimental modes clearly tiered and qualify actual distributables |

Source details, scope, and counterarguments are in [the local baseline](local-baseline.md#concrete-weaknesses-and-boundaries).

An additional documentation ambiguity is the package README's unqualified “macOS-only” embedded-Bun summary. Direct library builds do remain macOS-only; experimental Windows static packaging is a different path. Clarify that scope rather than turning packaging evidence into a claim of supported Windows delivery.

### The most important technical distinction

```text
Solid reactive scope != encoded Patch scope != native render/layout/paint scope
```

We have already improved local commit application. That does not remove native redraw costs. A historical cached-pane experiment produced excellent timings while displaying a stale counter; it was correctly rejected. Proper native regions must invalidate for commit changes **and** native-only editing, scrolling, async content, animation, inherited text style, geometry, clipping, and scale. This is a design problem, not a request to add arbitrary caches. [Recorded experiment](../native-incremental-commit/report.md#render-region-decision), [current cache contract](../../docs/performance-analysis.md#native-update-and-cache-boundaries).

### What this assessment does not call a mistake

- A binary protocol is not inherently worse than N-API/FFI; it trades encoding/queue costs for a stable engine/thread/process seam and transactional publication.
- The present source does not establish that IPC is the dominant bottleneck. Attribute queueing, decode, commit application, element construction, layout, text shaping, and presentation before replacing the runtime adapter.
- Keeping IME/edit history in Rust is not unnecessary duplication of Solid state.
- QuickJS not providing Node/Bun services is a deliberate Rust-led application boundary.
- Strict rejection of an unknown native contract is useful; the concern is which identities are being conflated, not that checking should be removed.
- A smaller alternative is not automatically faster, and a larger component catalog is not automatically more usable.

## 4. What to adopt, in order

Adopt conveniences on top of our existing ownership and contract model, not a second renderer or compatibility layer. The alternative-specific lessons above and independent local priorities lead to this sequence:

### Acceptance scenarios, not more feature checklists

Keep a few end-to-end scenarios that can disprove the proposed improvement:

| Proposed improvement | Key acceptance scenario | Evidence that would reject it |
| --- | --- | --- |
| Standalone first-app path | Published packages in a directory outside the SDK workspace; signal update, Unicode editing, Rust service, production preview, and an extracted package without SDK checkout | Success depends on workspace aliases, undeclared patches, development runtime, or repository-relative assets |
| Native render regions | Change a counter beside a large static pane; then edit text, resize, switch theme, load async content, remove/remount the region, and press a retained button after unrelated commits | Stale displayed content, wrong callback generation, missed inherited-style/layout changes, or timing gains from omitted work |
| Separate identities | Comment-only implementation edit, LF/CRLF checkout comparison, true DTO/slot/event change, and a behavioral-contract semantic-version change | A real incompatible contract is accepted, or an unrelated source-format change is misdiagnosed as a public-contract difference |
| Qualified app delivery | Launch the extracted application outside the source tree under the documented runtime and target requirements | Package checks pass only because the build machine's Bun, fonts, assets, paths, or development libraries are available |

These are recommendations for future work, not additional tests added in this task. Existing relevant regressions should be extended or replaced, rather than accumulating redundant smoke tests.

### Sequence

Fix Unicode TestHost offsets, the stale v5 release-rehearsal assertions, public-status drift, and the standalone starter first. They address actual helper/gate defects and improve adoption without changing rendering semantics. Treat native regions as the highest-value technical design investigation, not a quick cache patch. Identity separation is a deliberate contract/tooling refactor and should be planned independently; styling conveniences and bridge micro-optimizations should follow evidence from real applications rather than expand the default interface preemptively.

| Priority | Scope | Main source of the lesson |
| --- | --- | --- |
| **P0** | Fix Unicode TestHost offsets and stale v5 release checks; clean public status notes | Local reproduced/static defects |
| **P0** | Exact-version standalone starter and optional prebuilt stock host | GPUIX delivery; all three smaller app surfaces |
| **P1** | Opt-in native acceptance interface plus one exact-source realistic reference app | GPUIX/GPUix Solid testing and port provenance |
| **P1** | Profile native input/root work; design explicit native render regions | Local evidence, independent input owners in alternatives |
| **P1** | Public canonical compiler interface | lxsmnsyc compiler module |
| **P1, measured experiment** | Native style sharing/reclamation; separate contract/build responsibilities | GPUIX interner; local source-hash coupling |
| **P2** | Strict style conveniences, renderer-wide selection, consumer-led recorded paint/media | GPUIX state styles; lxsmnsyc canvas; GPUix Solid edge work |
| **P2, after packaging** | Application-owned signed updater | GPUIX updater implementation, requiring a separate safety review |

1. **First-app experience:** a clean external project that installs the published SDK, opens a window, updates a signal, edits Unicode text, and builds/previews through one documented path. Extend existing packed-consumer tests; they already exercise installed JS packages, but their small Rust exporter does not link GPUI.
2. **Native render regions:** design one stable region ownership seam rather than adding public performance hints everywhere. Verify a changed counter actually renders while an unchanged sibling region avoids redundant construction. Cover input, scrolling, resize, theme, inherited styles, async assets, and removal before accepting timings. Run the same workload through each relevant runtime with the same viewport, scale, features, and monitor policy; report commit cost, CPU draw, and input-to-presentation separately. Include both a large static sibling pane and a layout-coupled counterexample where wider text must relayout its neighbor. Optimize without hiding required work.
3. **Identity separation:** keep an exact release/build lock while making catalog identity describe the canonical exported contract and explicit semantic versions. Source-only edits and true contract changes should have intentionally different consequences; no legacy fallback is needed. This improves responsibility and diagnostics, but does not automatically permit mixed releases or eliminate every build/binding regeneration requirement.
4. **A narrow qualified delivery path:** prioritize packaging an ordinary external application, not just the repository gallery. Start with a Rust-led QuickJS template because that is the repository's existing delivered gallery model, while continuing to test its UI in the real QuickJS VM. This is not a silent conversion of a Bun application: apps that need Bun/Node services must keep that runtime and its explicit experimental embedded-delivery status. Retain runtime choice, but avoid presenting experimental combinations as equivalent defaults.
5. **Documentation status cleanup:** fix the confirmed drift as an explicit follow-up and link capability notes to their authoritative matrix.
6. **Compiler integration:** export a small compiler-only interface using our existing universal compiler, TypeScript erasure, runtime target, and composed source maps. Vite and other consumers must use the same implementation; do not migrate the application to Solid 2 prereleases just to match an alternative.
7. **Strict authoring sugar:** start with `paddingX/Y`, `marginX/Y`, and explicit state-style helpers where the native contract supports them. Define merge precedence and diagnose unsupported tokens. Length-unit unions, flex basis, and per-axis overflow need deliberate schema/native support rather than strings that the mapper ignores.
8. **Consumer-led recorded paint:** if a diagram/custom-graphics application needs it, add a NativeView that retains bounded draw records. Validate command/vertex/text budgets and finite coordinates; specify resize, DPI, clipping, transform, and theme behavior. No JS callbacks or Solid owners cross to the GPUI foreground; native paint replays retained records.
9. **Reduce upstream maintenance pressure:** keep an accurate inventory of vendored patches and upstream generally useful fixes where feasible. The alternatives' stock upstream revisions simplify their build graph, but their input/list/cache gaps do not prove our patches were unnecessary. The current vendor note still names `0.3.5` while the manifest/lockfile resolve `0.3.7`; correct provenance when doing this follow-up. [Patch inventory](../../vendor/gpui/PATCHES.md), [alternative comparison](solid-alternatives.md#5-what-we-do-well-and-what-we-got-wrong).
10. **Native acceptance automation:** stable locators and actual painted text/bounds, native click/type/drag/wheel, screenshots, owned lifetimes, and a clear unsupported-platform result. Keep the automation transport separate and opt-in; piped stdin must not automatically enable a powerful command server. This supplements TestHost and native Rust tests, not replaces them.
11. **Real application evidence:** one exact-source multi-pane app and one editable timeline/large-history workload, with named interactions and native substitutions. Verify scroll displacement, resizing, selection/copy, cleanup, and follow-up clicks—not just screenshots or importability. Keep public npm, source-edge, test-renderer, and physical-input results distinct.
12. **Style interning experiment:** measure our current Style/StoredNode sizes and journal cloning, mount, no-op updates, drag-style churn, and root collapse. Only retain immutable sharing if it reduces actual cost without leaked interned entries. Do not introduce a second style-reference wire format speculatively.
13. **Future product capabilities:** renderer-wide selection/search, live media, and signed updates are useful for some applications. Build them on declared host-owned resources and native modules; portable resource identities need ownership/import rules, and updater verification/install/rollback/relaunch requires its own review. They are not prerequisites for a correct Solid renderer.

## Evidence and verification

### Reuse and licensing

GitHub's first-party license endpoint identifies heyhuynhgiabuu/solid-gpui and remorses/gpuix as **Apache-2.0**, and jhomra21/gpuix-solid and lxsmnsyc/solid-gpui as **MIT**. Learning an architectural idea is different from copying implementation code. If code is imported later, inspect the pinned files and notices, retain the applicable upstream terms, and update our third-party inventory; do not relabel imported Apache code as solely our project's MIT code. No external code was copied by this research. [heyhuynhgiabuu license metadata](https://api.github.com/repos/heyhuynhgiabuu/solid-gpui/license), [GPUiX license metadata](https://api.github.com/repos/remorses/gpuix/license), [gpuix-solid license metadata](https://api.github.com/repos/jhomra21/gpuix-solid/license), [lxsmnsyc license metadata](https://api.github.com/repos/lxsmnsyc/solid-gpui/license).

Supporting notes:

- [Local baseline and immutable source links](local-baseline.md).
- [heyhuynhgiabuu and lxsmnsyc implementations](solid-alternatives.md).
- [remorses GPUIX and jhomra21's Solid adapter](gpuix-alternatives.md).

Checks performed during the research:

- Local renderer/control-flow/pressure tests: **47 passed, 0 failed**.
- Existing public TestHost suite: **7 passed, 0 failed**; these assertions do not catch the reproduced default Unicode selection defect.
- Website headless/content tests: **7 passed, 0 failed**.
- The first-party npm registry reports `@solid-gpui/core@0.5.2`; the live Pages landing page was visually inspected.
- GitHub reports a published `v0.5.2` with macOS ARM64, Windows x86-64, and Linux x86-64 Gallery archives/checksums. They were not downloaded or launched during this research.
- Static protocol/release-script consistency check: **failed as expected**, identifying five stale v5 assertions in three scripts; the actual release build/rehearsal was not run.
- A local TestHost controlled-input probe reproduced default offsets: `新值` → **6** instead of **2**; `🙂` → **4** instead of **2**. Both were echoed in an acknowledged input Patch. The initial two-case probe assumed surface 1 could be reused; after the first case it correctly hit the project's retired-ID constraint. The corrected probe used distinct explicit surface IDs and completed both cases. No implementation or regression test was changed.
- Report/source integrity check passed: **345 pinned citation ranges**, **219 unique Git blobs**, and **31 relative research/documentation links**, plus title/newline/whitespace checks. This verifies references, not the runtime truth of every source claim.

No remote-project installs or builds, native-window comparison, physical-input measurements, binary-size comparison, or cross-project benchmark was run. Source audits identify mechanisms and gaps; they do not establish a performance ranking.

Only investigation notes are added. Related guide/index/website ownership was inspected, and the website checks above passed. No API, behavior, generated contract, catalog, or website navigation changed; public documentation synchronization is therefore unnecessary for this research artifact. Identified stale public notes remain reported follow-ups, not silently repaired implementation work.
