# GPUI Kit adoption research

Status: initial research complete. Implementation and the later scope correction are tracked in [spec.md](spec.md).
Date: 2026-09-10.

## Evidence boundary

At the start of research, the reference checkout was pinned at
`928c3eb776a3d733d9b771f7dea27a6a79242ced` (2026-09-05). A read-only source
comparison fetched upstream `main` into remote refs without changing that checkout.
The fetched tip is `05433bd8e9e75af2f3aa508141b78ba21bfa6261`
(2026-09-10T10:19:34Z), **52 commits ahead**. Links below name immutable commits;
“latest” means this captured tip, not a moving promise.
[Pinned tree](https://github.com/longbridge/gpui-kit/tree/928c3eb776a3d733d9b771f7dea27a6a79242ced),
[captured tip](https://github.com/longbridge/gpui-kit/tree/05433bd8e9e75af2f3aa508141b78ba21bfa6261).

The task renamed the reference and vendor directories to `references/gpui-kit`
and `vendor/gpui-kit`. A source URL or directory rename does not update the
component implementation. At that point, the vendored subset contained Base, Component,
Component macros, and assets; the reference includes additional crates.
[Local dependencies](../../Cargo.toml),
[vendor provenance](../../vendor/gpui-kit/SOLID-GPUI.md).

The existing SDK already documents 144 generated JSX components/descriptors and
13 native functions, including tables, editor, docking, charts, chat controls,
virtual lists, menus, and application appearance. These are existing adoption,
not features newly unlocked by the Kit name.
[Current integration guide](../../docs/gpui-components.md).

## What actually changed after the pin

| Addition or change | Upstream evidence | Value and integration boundary |
| --- | --- | --- |
| Carousel | [Addition, `382fc28`](https://github.com/longbridge/gpui-kit/commit/382fc28feb2de118c7a523c1b3a22d43aacd2f04); [API guide at captured tip](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/website/component/carousel.md) | A genuinely new control family: horizontal/vertical snapping, keyboard and gesture navigation, looping, controlled selection, pagination, and reduced motion. A Solid adapter needs one retained native state per viewport, keyed item composition, count reconciliation, and semantic change events. |
| Multiple editor cursors | [Addition, `cbdf5ba`](https://github.com/longbridge/gpui-kit/commit/cbdf5baa26a5c20ae5c1d7481bffdd1d0d2abd3d); [native state](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/base/src/input/base/state.rs) | Useful for a real code editor: multiple selections, column selection, distributed paste, and undo/redo. Local `InputSelection` currently reports one start/end byte range. Upgrading native behavior and exposing multiple selections are separate changes; preserve the existing edit acknowledgement and IME ownership contract. |
| Editor auto-close and smart indent | [Addition, `e181255`](https://github.com/longbridge/gpui-kit/commit/e181255783a4bbc4ea19ef7d1a7f9428598e7bf8); [language configuration](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/component/src/input/language_config.rs) | Native bracket completion and Enter indentation should remain native editing behavior. Expose the real native options and language configuration instead of recreating editing in JavaScript. |
| Markdown frontmatter | [Addition, `f6e796d`](https://github.com/longbridge/gpui-kit/commit/f6e796deccd94f6c7cd28d4129d37a343f4bba79); [plugin implementation](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/component/src/text/frontmatter.rs) | An optional parser extension plus a renderer for top-level YAML metadata as a description list. This is not a general YAML parser: quoted/compound values and unsupported scalar forms use the native YAML code-block rendering. Useful for document metadata, with explicit opt-in behavior. |
| SVG bytes in component icon slots; shared Lucide names and preserved default icons | [SVG addition, `4475eeb`](https://github.com/longbridge/gpui-kit/commit/4475eeb56ae78a5bf40ef23c2fb18396bdfff52b); [asset change, `fbd51d2`](https://github.com/longbridge/gpui-kit/commit/fbd51d29474ac4b6a2615357922d318a39cf17dc) | Can extend slots beyond the existing icon enum and make application asset composition safer. Keep Solid's existing offline Iconify catalog; inspect overlap and size before adding another complete icon payload. |
| Headless UI test helpers | [Addition, `b6befdf`](https://github.com/longbridge/gpui-kit/commit/b6befdfdbcc070087a7acbdbd7aa2e2c847786b9); [testing contract](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/kit/TESTING.md) | Scoped queries, immutable state snapshots, real hit-tested clicks, keyboard/scroll/drag interactions, and bounded async waits. Valuable for critical native adapter workflows. Requires `test-support` and native instrumentation; it is not packaged application automation. |
| FPS metric/lifecycle corrections | [Readout-frame exclusion, `8741431`](https://github.com/longbridge/gpui-kit/commit/87414314ef74552297dd5faf614aaaf89f085829); [display/hidden lifecycle, `01cded5`](https://github.com/longbridge/gpui-kit/commit/01cded54ddb928aa18b7740ea584ac4107806d1f) | Changes the monitor's meaning and idle behavior; see the dedicated comparison below. Import the inspected implementation rather than assuming the pinned and latest FPS APIs behave the same. |
| Shell uses QuickJS JIT | [Change, `88a1bdc`](https://github.com/longbridge/gpui-kit/commit/88a1bdc80d01f6fb24e95cd0a1419d709bcca843); [dependency manifest](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/Cargo.toml) | Shell now pins `quickjs-jit`/runtime 0.12.7 from Git and uses a compatibility facade for its standard-library dependencies. This does not accelerate Solid's existing QuickJS engine automatically and is not evidence of a measured gain for our workload. |

There are also behavior fixes worth bringing along: editor search visibility,
CRLF boundaries, column selection over short rows, changed-language highlighting,
avoiding unchanged input notifications, selection drag-autoscroll in virtualized
lists, list measurement, Select dismissal/value accessibility, tab scrolling/flex,
text descender clipping, and accessible controls. These are reasons to evaluate
an upstream refresh beyond adding named controls.
[Source comparison](https://github.com/longbridge/gpui-kit/compare/928c3eb776a3d733d9b771f7dea27a6a79242ced...05433bd8e9e75af2f3aa508141b78ba21bfa6261).

The workspace version changed from 0.6.0 to 0.6.1 at
`f9621f14afc8056a3f25df1fa66b2f118d9731f3`. Carousel and the latest FPS fixes were
committed afterwards. A dependency labelled `0.6.1` alone therefore does not prove
that it contains every change above; qualification should name the exact source
revision. The captured upstream manifest requests `gpui-pre` 0.3.1 while Solid
requests 0.3.3 and patches it locally, so the resolved dependency graph remains
part of adoption verification.
[Version commit](https://github.com/longbridge/gpui-kit/commit/f9621f14afc8056a3f25df1fa66b2f118d9731f3),
[upstream workspace](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/Cargo.toml),
[local workspace](../../Cargo.toml).

## Existing Kit capabilities outside our current integration

| Layer | Source and actual role | Recommendation for Solid GPUI |
| --- | --- | --- |
| `gpui-kit` | Already at the pin. A facade re-exporting GPUI, platform, Base, optional Component/assets, with an initialization helper. Shell is a separate dependency. [Pinned implementation](https://github.com/longbridge/gpui-kit/blob/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/kit/src/lib.rs). | Can simplify Rust consumer imports and feature management, but adding the facade alone does not expose additional JSX components. Evaluate its new test module on a compatible vendor refresh. |
| `gpui-base` | Already vendored and used directly for native editing/state, in addition to Component's dependency. Exposes unstyled controls, motion, presence, springs, keyframes, NavStack, text selection, positioning, docking, and virtualization. [Pinned exports](https://github.com/longbridge/gpui-kit/blob/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/base/src/lib.rs). | The largest remaining distinct Base opportunities are native motion and custom visual systems. Adapt these through native modules with retained state; do not send per-frame animation updates from JS. |
| `gpui-shell` | Already at the pin; a hosted JS runtime with script views, retained render snapshots, manifests, declared capabilities, host functions, plugins, watchers, and dockable script panels. [Public surface](https://github.com/longbridge/gpui-kit/blob/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/shell/src/lib.rs), [plugin lifecycle](https://github.com/longbridge/gpui-kit/blob/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/shell/src/plugin.rs). | Suitable for optional application plugins authored against Shell. It owns a different JS view/runtime contract from Solid. A useful integration is an explicit native plugin-panel host; adopting it as Solid's renderer would be a separate architecture change. |
| `gpui-component-shell` | Already at the pin. A concrete Component registry for Shell, with descriptor schemas, native materialization, state, and a Component `Root`/overlay window opener. [Pinned implementation](https://github.com/longbridge/gpui-kit/blob/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/component-shell/src/lib.rs). | Useful with Shell plugins and as a reference for missing native contracts. Its registration API is for Shell's runtime, not Solid's generated native module registry; it cannot be installed as a drop-in JSX catalog. |
| `gpui-fps` | Already at the pin, with substantive later changes. Independent of Component; collects native frame and platform resource data. [Captured source](https://github.com/longbridge/gpui-kit/tree/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/fps). | Strong candidate to replace the small local display HUD after an explicit semantic and lifecycle integration. Keep separate latency profiling. |
| `gpui-wry` | Already at the pin. Experimental Wry webview integration; documented for macOS/Windows, overlays GPUI content within its native bounds. [Pinned README](https://github.com/longbridge/gpui-kit/blob/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/webview/README.md), [handle lifecycle](https://github.com/longbridge/gpui-kit/blob/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/webview/src/lib.rs). | Optional desktop browser content/auth/document panels. Needs explicit platform scope, focus, clipping, URL/navigation policy, and destruction before the owner window. No claimed Linux/browser parity. |
| Story and Story Web | Already at the pin; native/Web component gallery applications, not new reusable runtime layers. [Pinned gallery](https://github.com/longbridge/gpui-kit/tree/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/story), [Web gallery](https://github.com/longbridge/gpui-kit/tree/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/story-web). | Reference actual composition and regression scenarios when extending Solid's own website/catalog. |

The local audit found no `NavStack`, `MotionValue`, `MotionReveal`, `Presence`,
`Spring`, or `animate_keyframes` integration in the Solid SDK/host. However,
adding a second navigation state owner would conflict with the existing application
router (`@solid-gpui/router`). Reuse Base navigation motion beneath that router or expose a bounded
native stack only where a product needs one. This is a design recommendation,
not a shipped API.
[Local routing ownership](../../docs/router.md),
[native composition guidance](../../docs/native-composition.md),
[upstream motion exports](https://github.com/longbridge/gpui-kit/blob/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/base/src/lib.rs).

Shell and Component Shell are both marked `publish = false`; the runtime's
public docs mark it experimental, and enabling Shell also enables Base's
inspector feature through Cargo feature unification. Its README still contains
early milestone descriptions that understate the real component catalog; the
public Rust registry and `PluginManager` implementation are stronger evidence of
what exists. Plugin discovery, an authorization callback before evaluation, and
host-owned cancellation on unload are implemented. This does not establish
complete plugin packaging/distribution or a finished general contribution system.
[Shell manifest](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/Cargo.toml),
[Component Shell manifest](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/component-shell/Cargo.toml),
[public lifecycle implementation](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/plugin.rs).

## FPS integration: benefit and semantic differences

The local `gpui-performance` HUD counts draw-to-draw intervals during active
bursts. It has no timer or monitor-driven redraws, samples every 250 ms, breaks
the history after a 500 ms gap, and retains 15 seconds of history. Its small
passive display is distinct from the host's separate cumulative frame and input
latency instrumentation.
[Local HUD](../../crates/gpui-performance/src/lib.rs),
[profiling guide](../../docs/performance-analysis.md).

The captured upstream FPS implementation adds per-window frame trace readings,
draw mean/P95, invalidation count, observed presentation cadence, display refresh
information, and process CPU/memory/GPU sampling. That is a useful expansion,
but its labels need precise interpretation:

- Default **MAX FPS** is derived from reciprocal mean `Window::draw` duration,
  capped by display refresh when available. It is not a measured sustained
  presentation result. Right-click switches to observed presentation FPS.
- **DROP** is the proportion of retained draw samples exceeding the selected
  frame budget. It is not a measurement of compositor/display dropped frames.
- First eight draw samples and the pre-enable backlog are discarded. This HUD
  therefore cannot validate cold-start rendering latency.
- The readout clock requests a window redraw every 500 ms. Draws attributed only
  to this clock are excluded from draw cost samples, but this still differs
  from our current zero-timer passive HUD. The source excludes own **draw**
  samples; its presentation sampler separately consumes platform present events.
- The hidden HUD stops its clock and releases its trace guard after two
  unrendered ticks, about one second. Resource readers are native-only; GPU
  values depend on available OS/driver counters.

[Metric calculations](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/fps/src/sampler.rs),
[readout and lifecycle](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/fps/src/monitor.rs),
[platform dependencies](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/fps/Cargo.toml).

The pinned FPS version still has `continuous(true)` by default; latest removes
that option. Do not import the old version assuming latest's idle behavior.
Preserve user visibility/placement controls, ensure one monitor per window,
verify trace ownership alongside local profiling, and decide whether to expose
observed cadence directly or extend the upstream API for that choice. Replace
the old HUD once adopted; retain input-to-present and invalidation-to-present
profiling because those measure different questions.
[Pinned FPS contract](https://github.com/longbridge/gpui-kit/blob/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/fps/README.md),
[captured FPS contract](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/fps/README.md).

## Recommended adoption order

1. **Qualify the upstream revision and native diagnostics.** Reconcile our
   declarative vendor seams against a pinned refresh, adopt FPS with documented
   metric semantics, and use Kit headless helpers for a small number of critical
   adapter scenarios. Keep `test-support` out of performance/release runs. The
   upstream optional image-rendering test target requires a real Metal device;
   portable headless interaction tests are separate.
2. **Deliver distinct user-visible additions.** Carousel; native editor
   multi-cursor/auto-close/indent; optional Markdown frontmatter; richer icon
   slots. Generate the SDK from native contracts and add real catalog examples.
   For editor state, explicitly redesign the single-selection seam where needed
   instead of preserving an incompatible historical wrapper.
3. **Expose Base behavior where Solid lacks it.** Native motion/presence and
   custom visual composition, with state owned by mounted native instances and
   reduced-motion behavior. Reuse the existing router for application location.
4. **Add Shell when an application needs plugins.** An optional native module
   mounts/removes Shell plugin panels and supplies declared host capabilities.
   Solid remains the application's composition/runtime owner; plugins own their
   Shell views. Host grants, unload cancellation, theme/root integration, and
   separate VM costs require explicit acceptance. WebView remains a separate
   platform-specific capability when there is a concrete product use case.

These are recommendations inferred from upstream sources and the local ownership
model, not promises that every layer can be installed without adaptation.
[Solid invariants](../../CONTEXT.md),
[Kit test boundaries](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/kit/TESTING.md).

## Verification and synchronization

Research used upstream Git history/source plus the official repository and
website, with local source comparison. It did not build upstream main, run its
test suite, execute Shell plugins, or measure FPS performance. Claims about
runtime performance and platform parity remain unqualified until integration
tests/native measurements establish them.

Inspected `docs/README.md`, `docs/gpui-components.md`,
`docs/performance-analysis.md`, `CONTEXT.md`, and `examples/website/README.md`.
This internal research note changes no shipped API, component availability, or
published website article, so it requires no generated API or website content
update. The concurrent directory rename and its public source/provenance links
are verified separately by the main task. This file was checked for whitespace
errors and broken local Markdown links; the fetched reference checkout stayed
at its original pin.
