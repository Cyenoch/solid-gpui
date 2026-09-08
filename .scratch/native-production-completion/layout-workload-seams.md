# Native layout, internationalization, and workload seams

Date: 2026-09-07. Read-only source investigation against the current working tree and linked gpui-pre 0.3.3. No new desktop measurements or implementation changes. Windows/Linux real-desktop acceptance remains explicitly deferred by the user.

## Implementable now

### 1. Expose the existing native reduced-motion control

The renderer already respects `cx.reduce_motion()` when retargeting and sampling animations ([animation.rs](../../crates/solid-gpui/src/renderer/animation.rs:139), [retargeting](../../crates/solid-gpui/src/renderer/animation.rs:330)). Native component motion also checks the flag ([motion.rs](../../vendor/gpui-component/crates/base/src/motion.rs:286), [shimmer](../../vendor/gpui-component/crates/component/src/shimmer.rs:211)). This is a missing application control/preference seam, not missing animation support.

The linked [App::reduce_motion / set_reduce_motion](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/app.rs:1059) are public. The setter updates the application-wide bool and refreshes windows only when it changes. The initial value is false ([App initialization](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/app.rs:864)). Inspected GPUI platform sources and host do not automatically load or observe the OS reduce-motion preference.

**Minimal coherent change:** add generated foreground native `getReducedMotion()` and `setReducedMotion(bool)` commands beside the existing [theme commands](../../crates/solid-gpui/src/components/theme.rs:51); use the existing App value rather than a duplicate store. Expose a user setting in the Gallery. Document that the choice is application-wide and explicit. Applications can persist it through their chosen service. Do not call false “system” or claim automatic OS following.

**Acceptance:** set true while a transition is active → final value and one completion, with no continuing animation frame loop; native spinner/shimmer respects the same flag; changing it refreshes all windows. One focused integration scenario is sufficient. True OS-follow mode would separately need actual OS query/notification code (macOS accessibility display preference, Windows animation setting, Linux desktop-specific setting), then target-platform acceptance; no such shared GPUI API was found.

### 2. Correct grapheme navigation/deletion in primitive TextInput

[previous_utf16_boundary / next_utf16_boundary](../../crates/solid-gpui/src/renderer/input.rs:1421) iterate Rust `chars()`: they preserve scalar/UTF-16 boundaries but split a visible grapheme such as `e` + combining accent, an emoji with skin tone, or a ZWJ family. These helpers feed [left/right selection movement](../../crates/solid-gpui/src/renderer/input.rs:1501) and [deletion](../../crates/solid-gpui/src/renderer/input.rs:1558). Existing tests mainly establish astral/surrogate safety ([tests](../../crates/solid-gpui/src/renderer/input.rs:1930)), which does not establish grapheme behavior. This is a concrete algorithmic gap, not an RTL assumption.

**Minimal change:** use the existing `unicode-segmentation` dependency ([Cargo.toml](../../crates/solid-gpui/Cargo.toml:19)) to compute extended grapheme boundaries while preserving UTF-16 offsets at the external selection/IME API. Keep OS-supplied marked ranges and `maxLength` explicitly in their existing units; do not rewrite the public selection contract.

**Acceptance:** one combined fixture containing combining accents, ZWJ emoji and non-Latin text verifies arrow/Shift-arrow/backspace/delete move/remove one user-perceived character, while existing IME/undo/UTF-16 fixtures pass. Selection commands that explicitly point inside a grapheme need a documented normalization rule distinct from keyboard movement.

**Generated editor is a separate seam:** gpui-component [previous_boundary / next_boundary](../../vendor/gpui-component/crates/base/src/input/base/state.rs:2341) clip byte offsets to character boundaries, and [left/right](../../vendor/gpui-component/crates/base/src/input/base/movement.rs:168) use them. Primitive fixes do not fix generated Input/Editor automatically. If applying the same grapheme contract there, preserve rope performance, folded-region skipping and cursor affinity; add one corresponding native-editor regression rather than flattening the entire rope on every keypress.

### 3. Add a bounded real native grid surface if layout breadth is in this completion slice

Grid is genuinely available in the linked GPUI, even though the public SDK [Style](../../packages/solid-gpui/src/style.ts:51) currently exposes flex only. GPUI provides [Styled::grid](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/styled.rs:52), [grid_cols/grid_rows](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/styled.rs:752), and [column/row spans](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/styled.rs:836). [Style grid fields](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/style.rs:306) lower directly into [Taffy grid tracks and placement](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/taffy.rs:506).

**Minimal coherent contract:** a closed display choice plus bounded positive integer `gridColumns`, `gridRows`, `gridColumnSpan`, and `gridRowSpan`, generated through the canonical schema and applied to these actual builders. Choose explicitly whether to expose only equal fractional tracks initially; GPUI's simple `grid_cols(n)` means equal fractions with zero minimum. Reject contradictory/invalid inputs before publishing the tree. Do not parse CSS track strings or silently approximate grid using wrapped flex. This is real implementation work across TS validation/schema/Rust paint, but no dependency upgrade is necessary.

**Acceptance:** one native layout scenario with a spanning item verifies the intended positions/sizes before and after a narrow/wide resize; one invalid zero/oversized count verifies atomic rejection. Equal-track grid is an honest bounded API; arbitrary CSS Grid parity is unnecessary.

## Full bidirectional editing is not a one-property patch

The linked macOS platform uses CoreText shaping and preserves glyph positions and source byte indexes ([CoreText run extraction](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-macos-0.3.3/src/text_system.rs:575)). It even resets its UTF16-to-UTF8 converter when glyph source indexes decrease. That is evidence that RTL glyph order can exist; it is not evidence of correct caret/selection behavior.

Concrete limitations in the current shared geometry path:

- [LineLayout::x_for_index](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/text_system/line_layout.rs:105) returns the first glyph with `glyph.index >= index`, assuming useful logical monotonicity through the glyph order. `closest_index_for_x` and line-end behavior also assume particular visual endpoints ([hit testing](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/text_system/line_layout.rs:75)).
- [Primitive selection painting](../../crates/solid-gpui/src/renderer/input.rs:351) emits one single-line rectangle only when `x(end) > x(start)`. Mixed bidi selection can be several disjoint visual segments, and RTL endpoints can reverse.
- [Primitive left/right](../../crates/solid-gpui/src/renderer/input.rs:1501) follows logical offsets. Native generated [Input movement](../../vendor/gpui-component/crates/base/src/input/base/movement.rs:168) does too. Full visual movement needs embedding levels and caret affinity, not reversal of an array.
- The linked [ShapedRun/ShapedGlyph](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/text_system/line_layout.rs:33) carry font/glyph position/index but no explicit bidi embedding level or complete caret-stop map. Exposing a `direction: rtl` field cannot create that missing editing geometry.

**Do now:** keep native Unicode text and real font fallback; add a mixed Arabic/Hebrew + Latin/numeric sample to qualification; fix grapheme movement separately. Document layout mirroring via existing `row-reverse` as an application composition choice, with physical left/right properties; do not describe it as Unicode bidi support. `textAlign: right` is alignment only.

**What a full fix requires:** a native text-layout contract supplying visual caret stops (with affinity), logical-range-to-visual-segments, and point-to-logical-index for each shaped/wrapped line. Then update primitive and generated editor consumers together. A direct CoreText offset/hit-test implementation can provide a macOS-specific route, but it does not solve the shared Windows/Linux contract. Sorting glyphs or wrapping selection endpoints with min/max is insufficient for mixed-direction runs. This scope must remain explicit; it cannot honestly be declared complete through documentation or a trivial style property.

Locale/date/number formatting similarly remains application-owned Rust/Bun functionality unless a typed native formatter is deliberately added. QuickJS's bounded platform bootstrap is not an advertised full `Intl` runtime. Do not emulate broad Intl APIs speculatively.

## Representative workload evidence: extend existing scenarios

The repo already has meaningful isolated evidence:

| Existing evidence | What it establishes | What it does not establish |
| --- | --- | --- |
| [VirtualList empty/filter tests](../../packages/solid-gpui/tests/renderer.test.ts:460) and [filter after scrolling](../../packages/solid-gpui/tests/renderer.test.ts:547) | Range validity through data changes | Combined native scrolling, native identity, and resize behavior |
| [Nested native scrolling](../../crates/solid-gpui/src/renderer/scroll_tests.rs:640) and [list edge propagation](../../crates/solid-gpui/src/renderer/scroll_tests.rs:709) | Content displacement and scroll ownership | Production GPU frame cadence |
| [Native resize feedback loop](../../crates/solid-gpui/src/renderer/resize_tests.rs:249) | Retained gallery-shaped layout/counter evidence | Cross-platform desktop qualification |
| [Input performance measurements](../input-perf/measurement.md) | Whole-line shaping attribution for 32 edits and bounded host calls | Mixed bidi correctness, real presentation latency |
| [Protocol/tree budget](../../crates/solid-gpui/tests/perf_budget.rs:163) | CPU smoke budgets | A large-document interactive UI budget |

**Minimal next key scenarios:**

1. One keyed data workload: mount a large dataset, scroll into its middle, update/filter/reorder/append data, resize, then inspect mounted range, key identity, selection and reachable content. Assert visible work bounds and the intended scroll-anchor policy, not elapsed milliseconds in ordinary CI. Reuse actual native delegates and renderer fixtures rather than testing only an in-memory array.
2. One editing workload: multiline mixed Unicode content, selection, marked text, parent prop update, async result, resize and undo. Assert no stale controlled acknowledgement overwrites a newer edit and no invalid boundary appears. Reuse host-owned editing tests; add only a missing combined interaction or a reproducible regression.
3. For macOS performance qualification, run existing measurement tools serially with build work stopped and a fixed window/scale/monitor policy; verify content displacement first. Keep CPU timings separate from actual frame cadence and input-to-present latency under [the performance guide](../../docs/performance-analysis.md). Windows/Linux actual desktop evidence is deferred, not replaced with headless figures.

## Completion recommendation

Ship explicit reduced-motion control, grapheme-safe primitive editing (and generated editing if included in advertised contract), and bounded native grid if required by the requested layout completion. Add the two high-value combined workloads. Record full bidi visual editing and OS-follow reduced-motion as concrete remaining implementation scopes if not built; neither can be closed by reclassifying them as desktop-only acceptance.
