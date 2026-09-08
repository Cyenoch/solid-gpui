# Bidirectional editing: implementation plan and executed failure

Date: 2026-09-07. Research only. Production files and dependencies were not modified. Two temporary Rust probes compiled against existing GPUI/unicode-bidi artifacts and failed as expected; no new Cargo build or desktop session was started.

## Decision

A project-local **consumer geometry module is viable**, but a complete, font-correct implementation cannot generally derive visual caret stops from the current `ShapedGlyph` records alone. The smallest correct complete change needs the native shaper to retain information it currently discards. Avoid pretending a sorted glyph-origin algorithm provides full bidirectional editing.

There are two implementation routes:

1. **Recommended:** extend the linked GPUI text-layout seam with explicit visual caret/cluster geometry, then consume it from a project-local shared module used by both primitive and generated editors. This requires a reviewed GPUI patch/release or maintained dependency patch, not speculative dependency upgrading. It preserves one authoritative shaping/painting result.
2. **No GPUI dependency patch:** own a new native text-layout/paint provider for editable text, with platform-specific shaping and geometry. A project-local module can do this, but must own both drawing and hit testing; shaping the text a second time just for geometry is not reliable with fallback fonts, ligatures, styles, tabs or wrapping. This is a larger change, not a small adapter over existing `ShapedLine`.

A geometry-only patch can correctly handle a deliberately restricted class (non-ligating, monotonic-advance glyph clusters with known cluster widths), and the tests below are useful there. It must not be advertised as complete Arabic/Hebrew editing.

## Why the current records are insufficient

[ShapedRun/ShapedGlyph](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/text_system/line_layout.rs:33) retain font ID, glyph ID, one origin, source byte index and emoji flag. They do not retain advances, complete source cluster extents, embedding level, primary/secondary caret positions or intra-ligature caret positions. An Arabic ligature can cover multiple graphemes; a combining mark can share a source cluster and have a nonzero drawing offset. Neither the next glyph origin nor isolated-character advance gives a generally correct selection boundary.

`unicode-bidi 0.3.18` is already locked transitively ([lockfile](../../Cargo.lock:6888)). Its [visual_runs API](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unicode-bidi-0.3.18/src/lib.rs:669) supplies UAX #9 paragraph/line reordering levels and run ranges. Its documentation explicitly leaves combining-character handling and mirroring to the rendering engine. It does not supply font-dependent caret geometry. Reordering logical text into a new string and shaping it is wrong: the platform shaper may reorder it again, and source/IME offsets no longer address the original string.

### Native shapers have the information before projection

| Platform | Existing evidence | Correct seam |
| --- | --- | --- |
| macOS CoreText | [run extraction](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-macos-0.3.3/src/text_system.rs:575) has CTLine/CTRun, glyph positions and string indexes, then projects into GPUI records. The UTF converter explicitly restarts when indexes decrease. | Retain/query CTLine while producing geometry. `CTLineGetOffsetForStringIndex` supplies a secondary-offset output; the installed [core-text convenience method](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/core-text-21.0.0/src/line.rs:88) passes null and therefore discards it. The exact [FFI signature](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/core-text-21.0.0/src/line.rs:121) is available to implement a targeted binding. Preserve cluster/caret affinity rather than inferring RTL endpoints. |
| Windows DirectWrite | [DrawGlyphRun](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-windows-0.3.3/src/direct_write.rs:1493) ignores callback baseline origins. [projection loop](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-windows-0.3.3/src/direct_write.rs:1565) has clusterMap, advances and offsets but writes positions using monotonically accumulated context.width. | Retain the IDWriteTextLayout used by layout_line, and query [HitTestPoint/HitTestTextPosition/HitTestTextRange](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/windows-0.62.2/src/Windows/Win32/Graphics/DirectWrite/mod.rs:10753). Fix visual run painting positions from baseline origin/direction at the same seam. A geometry fix alone cannot guarantee matching Windows pixels. The `bidiLevel: 0` constants elsewhere are one-glyph rasterization, not sufficient evidence by themselves about paragraph shaping. |
| Linux Cosmic | [projection](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-wgpu-0.3.3/src/cosmic_text_system.rs:728) keeps only `glyph.x/y/start`; the Cosmic layout is still available before projection. | Retain cluster end, advance width and bidi level/cursor information from that exact resolved layout and use its hit-test/cursor semantics. Do not infer that API's exact behavior from CoreText; inspect/pin the installed Cosmic API during implementation. |

## Smallest coherent shared data contract

Introduce a native-only visual text geometry module (e.g. a small `gpui-text-geometry` crate, used by `solid-gpui` and vendored `gpui-base`, avoiding a dependency cycle). It must not add glyph geometry to the JS protocol.

```rust
struct CaretStop {
    byte_offset: usize,       // original logical UTF-8 text
    affinity: CaretAffinity,  // leading/trailing at bidi and wrap boundaries
    x: Pixels,
}
struct VisualCluster {
    logical_range: Range<usize>,
    advance_bounds: Range<Pixels>,
    level: u8,
    caret_stops: Vec<CaretStop>,
}
struct VisualLine {
    logical_range: Range<usize>,
    clusters: Vec<VisualCluster>, // actual visual order
    carets: Vec<CaretStop>,       // visual navigation order, preserves coincident distinct stops
    baseline_y: Pixels,
    height: Pixels,
}
```

Exact representation can use flat arrays/ranges rather than per-cluster Vec allocation. Store resolved native positions, not separately reshaped estimates. A native backend can supply selection segments directly; the shared module should preserve them rather than reconstructing an incompatible policy.

**Algorithms:**

- Resolve bidi at paragraph scope using original text/base direction. Break logical text into actual lines, then apply line-level UAX #9 rules (including trailing whitespace levels) to each line. Do not slice a visually reordered full paragraph into rows.
- A logical boundary may have two valid x coordinates and a soft-wrap boundary may occur on two rows. Preserve affinity as native editing state. UTF-16 offsets remain the external IME contract; convert at the boundary, not throughout geometry.
- Pointer hit testing chooses the nearest supported visual caret and its affinity on the target row. Left/right moves through visual stops, while delete/backspace removes logical grapheme ranges under the selected platform editing policy. Selection collapse chooses the appropriate visual edge rather than `range.start`/`range.end` by number.
- To highlight a logical range, intersect it with each visual cluster/run, use the native cluster's caret positions for partial intersections, then merge **only visually adjacent selected spans on the same row**. Preserve gaps occupied by unselected content. Min/max of the two logical endpoints is wrong.
- Up/down keeps the preferred visual x; Home/End has an explicit visual-line versus document policy. IME candidate bounds use the active caret affinity; an API requiring one rectangle may use the active segment/caret rather than inventing a cross-row union.
- Cache paragraph bidi analysis by content revision and base direction; cache visual layout by text/style/font-fallback/wrap width/scale and resolved line breaks. Generated editor should compute geometry only for already materialized visible lines; no full-rope flatten per mouse move.

## Consumer file targets

| Target | Required change |
| --- | --- |
| [primitive layout](../../crates/solid-gpui/src/renderer/input.rs:80) | Replace x_for_index/closest_index and one-rectangle selection logic with authoritative visual geometry. Store caret affinity beside selection head; preserve controlled value, undo, marked text and UTF-16 commands. |
| [primitive paint](../../crates/solid-gpui/src/renderer/paint/text_input.rs) | Paint caret/marked range/selection from the same geometry used by hit testing, and scroll to the active visual caret. |
| [primitive selection movement](../../crates/solid-gpui/src/renderer/input.rs:1501) | Split visual cursor movement from logical grapheme deletion, retaining existing key aliases and shift anchor rules. |
| [generated editor LineLayout](../../vendor/gpui-component/crates/base/src/input/editor/display_map/text_wrapper.rs:519) | Store shared geometry next to each shaped visual line. Replace position_for_index/closest_index/selection-background calculations; current line-end affinity only represents wrap ambiguity and must be extended for bidi affinity. |
| [generated editor movement](../../vendor/gpui-component/crates/base/src/input/base/movement.rs:168) | Visual left/right/up/down, selection collapse and focus-scroll policy. |
| [generated editor paint](../../vendor/gpui-component/crates/base/src/input/base/element.rs:661) | Replace endpoint-based selection rectangles, including search/marked text highlights. |
| [generated editor IME](../../vendor/gpui-component/crates/base/src/input/base/state.rs:2984) | Route bounds_for_range and character_index_for_point through shared visual geometry; preserve fold/display and masked-text index maps. |

These are two distinct editors; fixing one does not complete the other. The generated editor already stores logical rows, visible line byte offsets, wrapping indentation and cursor state in [LastLayout](../../vendor/gpui-component/crates/base/src/input/base/layout.rs:16); retain that ownership rather than adding a second editor state model.

## Executed failing probes

**Actual library execution:** compiled with `rustc` against the existing `target/debug/deps/libgpui-341b4a7cbdf5bbf1.rlib`, then executed. This exercises linked GPUI methods on explicit synthetic shaped glyph fixtures; it is **not** an actual font-shaping or desktop screenshot test.

1. Hebrew `אב`, visual glyphs `index=2,x=0` then `index=0,x=10`, total width 20. Actual `LineLayout::x_for_index(0)` returns `0px`; leading logical start must be `20px`. Process exit 101.
2. Mixed text `abc אבג 123 العربية xyz`, logical wrap before Arabic, UAX #9 visual runs from actual `unicode-bidi 0.3.18`, explicit 10px character-cell fixture. Both visual lines queried through actual GPUI `x_for_index`. **11 mismatches**, including second row byte 0: actual `0px`, expected leading-edge `70px`. Process exit 101. The synthetic Arabic cells deliberately exclude real joining/ligature shaping, so this proves a geometry-index defect and does not prove the proposed native shaping fix.

Reproducible second probe (the source used in `/tmp/solid-gpui-bidi-mixed.rs`):

```rust
use gpui::{LineLayout, ShapedRun, ShapedGlyph, FontId, GlyphId, point, px};
use unicode_bidi::BidiInfo;
fn main() {
    let text = "abc אבג 123 العربية xyz";
    let bidi = BidiInfo::new(text, None);
    let para = &bidi.paragraphs[0];
    let split = text.find("العربية").unwrap();
    let mut failures = 0;
    for (row, logical) in [0..split, split..text.len()].into_iter().enumerate() {
        let (levels, runs) = bidi.visual_runs(para, logical.clone());
        let mut glyphs = Vec::new();
        let mut expected = Vec::new();
        let mut x = 0.;
        for run in runs {
            let rtl = levels[run.start].is_rtl();
            let mut chars: Vec<_> = text[run.clone()].char_indices().map(|(i,_)|run.start+i).collect();
            if rtl { chars.reverse(); }
            for index in chars {
                glyphs.push(ShapedGlyph { id: GlyphId(1), position: point(px(x),px(0.)), index:index-logical.start, is_emoji:false });
                expected.push((index-logical.start, if rtl { x+10. } else { x }));
                x += 10.;
            }
        }
        let line = LineLayout { width:px(x), len:logical.end-logical.start, runs:vec![ShapedRun {font_id:FontId(0),glyphs}], ..Default::default() };
        for (index,x) in expected {
            if line.x_for_index(index) != px(x) { failures+=1; println!("row {row}, byte {index}: actual {:?}, expected {:?}",line.x_for_index(index),px(x)); }
        }
    }
    assert_eq!(failures,0,"mixed/wrapped visual caret leading-edge mismatch");
}

```

## Key regression design for the implementation

Use one deterministic geometry regression and one real native-font integration regression, not a test per code path.

- Deterministic: keep the mixed logical text above; test both a full line and the stated logical wrap. Assert visual left/right caret sequence with affinity, x→caret→x roundtrips, and a logical range whose selected cells form two separated visual spans. Include two font runs across one RTL span to reject the assumption that font-run boundaries are bidi boundaries. Existing scalar/ZWJ tests remain separate invariants.
- Native macOS: shape `abc אבג 123 العربية xyz` plus an Arabic lam-alef ligature and combining marks with the actual chosen font/fallback; obtain the oracle from the **same retained CTLine**, including secondary caret offsets. Place the native input in a narrow window so wrapping occurs, select across RTL/LTR/numeric boundaries, hit-test each chosen caret and request IME bounds. Assert painted selection segments exclude the unselected visual gap. Compare positions within a small pixel tolerance, not hardcoded font widths. Run for both primitive TextInput and generated Input/Editor through the shared geometry seam.
- Do not record the native mixed/wrapped test as executed yet: only the two synthetic-layout failures above were run. Windows/Linux actual desktop acceptance remains deferred, but platform projection implementations must exist before claiming cross-platform bidi support.

## Practical scope verdict

A local geometry module avoids spreading bidi logic across consumers. It cannot recover information discarded by the GPUI shaper. For a full correct implementation, choose a targeted authoritative shaping extension or an owned editable-text renderer; do not silently substitute equal glyph spacing, prefix reshaping, reversed strings, or min/max endpoint rectangles. Those would satisfy easy fixtures while failing the Arabic/ligature/wrapped cases the user explicitly needs.

## Exact dependency patch scope and cache-safe ownership

This is implementable through deliberate dependency refactoring. Patch the current pinned sources rather than hunting for an unverified release:

| Crate | Files to patch | Responsibility |
| --- | --- | --- |
| `gpui-pre 0.3.3` | `src/text_system/line_layout.rs`, `src/text_system.rs`, `src/platform.rs`, and text paint/wrap consumers | Own immutable visual geometry DTOs, query API, shaping request/base direction, cache key changes, and wrapped layout semantics. Update NoopTextSystem/test fixtures for the new required contract. |
| `gpui-pre-macos 0.3.3` | `src/text_system.rs` | Extract owned geometry from the same CTLine that produces glyphs before its lifetime ends; expose secondary offsets; provide actual native line wrapping/geometry as needed. |
| `gpui-pre-windows 0.3.3` | `src/direct_write.rs` | Retain local IDWriteTextLayout during extraction, query hit tests, retain cluster ranges/levels, and use actual baseline/RTL run direction for glyph positions. |
| `gpui-pre-wgpu 0.3.3` | `src/cosmic_text_system.rs` | Preserve Cosmic cluster geometry/levels before flattening into GPUI records. This supplies **both Linux and Web/WASM**. |
| existing vendored `gpui-base` | input `text_wrapper.rs`, `layout.rs`, `element.rs`, `movement.rs`, `state.rs` listed above | Consume shared geometry throughout generated Input/Editor, including folds/masking/IME. |
| project `solid-gpui` | primitive input/paint files above | Consume same contract in primitive TextInput/rich selectable text. |

`gpui-pre-linux` forwards to `gpui_wgpu::CosmicTextSystem`; it should need only rebuild/constructor adjustments. `gpui-pre-web/src/platform.rs:161` constructs `CosmicTextSystem::new_without_system_fonts("IBM Plex Sans")`, so Web is not a browser canvas/DOM text-metrics implementation. No separate Web text algorithm is necessary. Rebuild `gpui-pre-platform` for dependency identity consistency, but it need not own geometry logic. Use Cargo patches for these exact package names with the same versions, retain upstream license/provenance, and avoid two gpui-pre copies in the graph. If all types reside directly in patched GPUI, a new helper crate is optional and unnecessary for sharing with gpui-base.

**Proposed public query signatures (new API, not existing API):**

```rust
pub struct TextCaret { pub byte_offset: usize, pub affinity: CaretAffinity }
impl LineLayout {
    pub fn caret_position(&self, caret: TextCaret) -> Option<Point<Pixels>>;
    pub fn closest_caret(&self, point: Point<Pixels>) -> TextCaret;
    pub fn adjacent_caret(&self, caret: TextCaret, direction: VisualDirection) -> TextCaret;
    pub fn selection_segments(&self, range: Range<usize>) -> impl Iterator<Item = Bounds<Pixels>>;
}
```

Store `Box<[CaretStop]>`, `Box<[ClusterGeometry]>`, and row/run index ranges directly in `LineLayout` (or one owned immutable `Arc<VisualGeometry>`). No CTLine pointers, COM objects, platform closure trait objects, borrowed text, or unsafe Send/Sync implementations belong in the cache. Native handles remain temporary within backend extraction. GPUI's current `FrameCache` stores `Arc<LineLayout>` behind Mutex/RwLock (`line_layout.rs:454`), so owned pure data fits both multithreaded native builds and WASM. Existing glyph/geometry cache invalidation remains text/font/run/size based, with explicit new base-direction and line-break parameters included where applicable.

**Wrapping requires an actual contract change**, not just new LineLayout fields. Current `PlatformTextSystem::layout_line(&self, text: &str, font_size: Pixels, runs: &[FontRun]) -> LineLayout` (`platform.rs:1103`) has neither base direction nor logical wrap-line context. Current `WrappedLineLayout` clips a single pre-shaped line using glyph wrap boundaries. Add a paragraph/wrapped-layout operation taking original paragraph text, font runs, optional wrap width and base direction and returning owned visual rows. Preserve paragraph bidi context across soft wraps. Route editor and primitive wrapping through that operation; do not ask `unicode-bidi` to repair pixels after the current glyph slicing. The plain-line API can be implemented as the same operation with no wrap, not retained as a legacy alternate geometry path.

**Native signatures:** CoreText's existing C API is `CTLineGetOffsetForStringIndex(CTLineRef, CFIndex, *mut CGFloat) -> CGFloat`, where the pointer receives the secondary offset; `CTLineGetStringIndexForPosition(CTLineRef, CGPoint) -> CFIndex` supplies hit indexing. The convenience Rust wrapper currently discards the secondary pointer, so add a small direct binding or patch that wrapper's narrow API. No full CoreText crate fork is inherently required. DirectWrite exposes `IDWriteTextLayout::HitTestPoint(pointx, pointy, *mut BOOL, *mut BOOL, *mut DWRITE_HIT_TEST_METRICS)`, `HitTestTextPosition(textposition: u32, istrailinghit: bool, *mut f32, *mut f32, *mut DWRITE_HIT_TEST_METRICS)`, and `HitTestTextRange(textposition: u32, textlength: u32, originx: f32, originy: f32, Option<&mut [DWRITE_HIT_TEST_METRICS]>, *mut u32)`; use the documented two-pass sizing pattern for range metrics, then convert UTF-16 positions to original UTF-8 offsets once.

**Cosmic 0.19.0 is available and relevant:** its `LayoutGlyph` (`src/layout.rs:16`) carries `start`, `end`, `x`, `y`, `w`, and `level`, exactly the fields GPUI currently drops. `LayoutRun::cursor_position(&Cursor) -> Option<f32>` (`src/buffer.rs:120`) handles RTL leading/trailing edges; `cursor_glyph` carries cursor affinity. The backend deliberately divides a multi-grapheme cluster's width for internal cursors. Reusing a documented backend caret policy is different from inventing equal-width glyph reconstruction after information loss. However, `LayoutRun::highlight` (`buffer.rs:65`) must be checked against RTL partial-cluster cases: its current loop walks logical graphemes while increasing x. Do not blindly treat it as an oracle for all partial Arabic cluster selection. Extract authoritative cluster widths/levels, define a consistent within-cluster native policy, and test partial selection explicitly. This may require a narrow Cosmic fix in addition to the four GPUI crates above; decide after the concrete regression, not by assumption.

## Exact probe commands and artifact lifetime

The probes were deliberately temporary and are outside the repository. Source paths: `/tmp/solid-gpui-bidi-probe.rs` and `/tmp/solid-gpui-bidi-mixed.rs`; executables omit `.rs`. The complete mixed source is preserved above. Commands executed from repo root:

```sh
rustc --edition=2024 /tmp/solid-gpui-bidi-probe.rs -L dependency=target/debug/deps --extern gpui=target/debug/deps/libgpui-341b4a7cbdf5bbf1.rlib -o /tmp/solid-gpui-bidi-probe
/tmp/solid-gpui-bidi-probe
rustc --edition=2024 /tmp/solid-gpui-bidi-mixed.rs -L dependency=target/debug/deps --extern gpui=target/debug/deps/libgpui-341b4a7cbdf5bbf1.rlib --extern unicode_bidi=target/debug/deps/libunicode_bidi-9b7a0283409def88.rlib -o /tmp/solid-gpui-bidi-mixed
/tmp/solid-gpui-bidi-mixed
```

Both compiles exited 0; both probes exited 101 on their assertions. Artifact hashes are local build identities and must be rediscovered after a rebuild. Mixed result: 11 leading-edge mismatches across the two rows. No glyph metrics, native wrapping, ligature shaping or desktop presentation claims are derived from the synthetic fixture.

## CoreText refinement: enumerate actual logical edges (executed)

**Use `CTLineEnumerateCaretOffsets` as the primary macOS extraction API.** It is more direct than probing CTLineGetOffsetForStringIndex at every guessed boundary: CoreText supplies every logical caret edge in left-to-right visual order, including distinct coincident edges. Installed objc2-core-text 0.3.2 exposes:

```rust
pub unsafe fn CTLine::enumerate_caret_offsets(
    &self,
    block: &block2::DynBlock<dyn Fn(c_double, CFIndex, bool, NonNull<bool>)>,
);
```

The parameters are `(offset, char_index, leading_edge, stop)`. Offset is line-relative; leading_edge refers to **logical** order. These are explicit [binding/API documentation](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-core-text-0.3.2/src/generated/CTLine.rs:600) guarantees. It requires the block2 feature. Preserve the callback sequence instead of sorting/deduplicating by x.

### Actual macOS probe and its result

Executed `swift /tmp/solid-gpui-coretext-carets.swift` on the available macOS system; exit 0. Unlike the preceding synthetic GPUI probes, this uses **real CoreText font shaping and typesetter soft wrapping** (Arial 16, width 130) without opening a desktop window. The source is preserved below. It emitted two rows for `abc אבג 123 العربية xyz لا é 😀`:

- Row 1: CTLine string range `(location: 0, length: 20)`.
- Row 2: range `(location: 20, length: 12)`; caret callback character indexes remain **global into the original attributed string**, starting at 20, not rebased to zero.
- RTL edges on row 1 arrive in visual order with descending logical indexes. Arabic character 18 emitted trailing at 30.24 then leading at 36.24.
- Lam-alef on row 2 emitted index 25 trailing at 28.45, index 25 leading at 32.80, index 24 trailing at 32.80, index 24 leading at 37.15. CoreText supplied its internal caret split; the project need not invent equal division.
- Combining `é`: leading character index 27 at 41.59, trailing index 28 at 50.49. Emoji: leading index 30 at 54.94, trailing index 31 at 75.94. These verify that trailing index is **not already an insertion offset**: map leading to `char_index`, trailing to `char_index + 1` in UTF-16 units, yielding grapheme boundaries 27/29 and 30/32.

This probe establishes the extraction approach on the tested native font/text. It does not establish every font's ligature policy, frontend IME behavior, or patched GPUI rendering, which remain integration acceptance.

### Extraction procedure

1. Build one UTF-16-boundary → original UTF-8-byte map for the full paragraph. Include the terminal boundary and mark interior surrogate-unit positions invalid. Also record original extended-grapheme boundaries. Never use a forward-only cursor conversion across visually ordered edges.
2. For each enumerated edge, validate index/range, compute UTF-16 insertion offset using leading versus trailing, then map to UTF-8. Preserve an edge record with original character index, logical edge kind, visual-row identity and x. Filter keyboard cursor stops to valid grapheme boundaries **after** mapping. Preserve native metadata needed for IME/index APIs; do not let keyboard filtering rewrite the original text.
3. Distinguish bidi affinity from wrap affinity: one byte offset can appear at several x/row positions. Retain the identity of its edge and row. Do not merge coincident edges unless their logical offset and affinity are equivalent under an explicitly chosen navigation policy.
4. Build each grapheme's selected visual interval from its actual leading/trailing logical-edge records. For a multi-code-unit grapheme the first leading and last trailing character indexes can differ (as observed above). For a logical selection, gather the intersected grapheme/cluster intervals, then merge only touching selected intervals on the same row. A native range may be discontiguous; preserve the gaps. Partial-range IME operations must use the retained native character-edge geometry rather than fabricated glyph widths.
5. CTLineGetOffsetForStringIndex with secondary offset and CTLineGetStringIndexForPosition remain useful **oracles** for integration tests and cases needing native point-index semantics. They are not needed as a second approximate production layout beside enumerated edges.

### Typesetter wrapping with original paragraph context

The installed APIs are:

```rust
CTTypesetter::with_attributed_string(&CFAttributedString) -> CFRetained<CTTypesetter>;
CTTypesetter::suggest_line_break(start_index: CFIndex, width: c_double) -> CFIndex;
CTTypesetter::line(string_range: CFRange) -> CFRetained<CTLine>;
CTTypesetter::suggest_line_break_with_offset(start_index, width, offset) -> CFIndex;
CTTypesetter::line_with_offset(string_range, offset) -> CFRetained<CTLine>;
```

All are unsafe typed methods; [construction](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-core-text-0.3.2/src/generated/CTTypesetter.rs:95), [line construction](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-core-text-0.3.2/src/generated/CTTypesetter.rs:150), [contextual break](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-core-text-0.3.2/src/generated/CTTypesetter.rs:209). `suggest_line_break` returns a **count from start_index**, not an absolute end. Create the next CTLine using `CFRange(start, count)` and advance by count. Critically, a `line` range length of **zero means through the remainder of the string**; never pass a zero break result hoping to create an empty row.

Construct one attributed paragraph/typesetter, with the same font/fallback/features used for final glyph extraction. Ask it for all soft-wrapped line ranges without recreating an attributed substring for each row. This preserves paragraph-level context and CoreText's contextual break/line behavior. Explicit base direction belongs in the paragraph style ([BaseWritingDirection](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-core-text-0.3.2/src/generated/CTParagraphStyle.rs:390)); retain natural direction by default. Do not set `kCTTypesetterOptionDisableBidiProcessing`. Explicit hard breaks can delimit separate paragraphs; retain each paragraph's global byte/UTF-16 base when assembling the document.

For nonpositive available width or a nonprogressing break, apply a declared layout policy (e.g. place one entire grapheme overflow row) or return a bounded layout error; do not infinite-loop. For an actual empty paragraph, create the explicit empty-row DTO instead of invoking zero-length CTLine range on nonempty text. Continuation indent should be a layout request parameter; use consistent offset variants or shifted origins, with the same effective width for shaping/painting/geometry.

For each actual CTLine extract its **own** owned glyph runs using CTLine's run array, font identity, [CTRun positions and advances](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-core-text-0.3.2/src/generated/CTRun.rs:233), [source indexes/ranges](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-core-text-0.3.2/src/generated/CTRun.rs:348) and [run status](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-core-text-0.3.2/src/generated/CTRun.rs:154). Keep glyph positions row-relative as returned, glyph source indexes paragraph-global, and the row logical range explicit. Do not subtract an old full-paragraph glyph origin to simulate rewrapping. After glyph/edge extraction all CT objects can drop; cached rows contain only owned Rust arrays.

The existing backend uses the older core-text crate. Either migrate the touched native extraction code coherently to installed objc2-core-text 0.3.2, or add a narrowly documented C-ABI bridge against the existing retained CF types. Do not transmute Rust wrapper layouts between the two Core Foundation generations. The typed methods above describe real installed APIs; a dependency declaration/feature change is still required before use.

### Smallest safe WrappedLineLayout refactor

Replace `unwrapped_layout + wrap_boundaries` as the authoritative representation with owned visual rows:

```rust
struct VisualRowLayout {
    logical_utf8_range: Range<usize>,
    layout: LineLayout, // row glyphs + native caret edges; indexes stay paragraph-global
    origin_x: Pixels,
}
struct WrappedLineLayout {
    rows: Box<[VisualRowLayout]>,
    paragraph_byte_len: usize,
    wrap_width: Option<Pixels>,
}
```

Names/field placement can follow GPUI conventions; the invariant is that every painted row is the same native-shaped row used for interaction. Current [WrappedLineLayout](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/text_system/line_layout.rs:275) and [cache miss construction](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/text_system/line_layout.rs:602) assume one layout followed by glyph slicing. Replace that cache miss path with the backend paragraph/wrapped operation. Cache keys must include base direction, wrap width, continuation indent and line limit if the cached geometry is truncated. `width`, `size`, ascent/descent and row lookup should derive from actual rows. A no-wrap paragraph simply has one row; do not retain the obsolete alternate path for edited text.

[WrappedLine::paint](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/text_system/line.rs:284) should loop actual rows and call the shared single-row glyph painter with row origin/width; `paint_background` does likewise. Remove wrap-boundary switching from the edited-text paint path. The current [glyph/decorations loop](/Users/jgbingzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-pre-0.3.3/src/text_system/line.rs:390) also advances decoration ranges in logical order while visiting visual glyphs. Resolve a glyph's style from its original logical source range (binary lookup or pre-indexed owned visual paint spans), and create underline/background/strike spans from selected visual intervals. Preserve current font-run splits introduced at decoration changes by TextSystem, so ligatures are not unexpectedly restyled midway.

The primitive editor should consume these rows directly. Generated gpui-base `text_wrapper.rs` currently pre-splits into `ShapedLine`s; refactor it to request a full logical paragraph's native wrapped result, then use its row ranges for display-map/indent/fold calculations. Otherwise the new core representation would still receive paragraph context too late. This is the necessary migration, not a compatibility wrapper.

Tests must cover actual wrapped row glyph order, caret sequence, selection gaps, IME bounds and colored/underlined text across bidi boundaries. Retain a LTR/empty-line/indent case to detect a regression from deleting the old slicer. The previous synthetic test still proves the old geometry failure; the new native test can compare patched geometry against enumeration of the same CTTypesetter-created CTLines.

### Executed CoreText probe source

Command: `swift /tmp/solid-gpui-coretext-carets.swift` (exit 0). Source:

```swift
import Foundation
import CoreText
let text = "abc אבג 123 العربية xyz لا e\u{301} 😀"
let attrs = [kCTFontAttributeName as NSAttributedString.Key: CTFontCreateWithName("Arial" as CFString, 16, nil)]
let attributed = NSAttributedString(string: text, attributes: attrs)
let typesetter = CTTypesetterCreateWithAttributedString(attributed)
var start = 0
while start < attributed.length {
    let count = CTTypesetterSuggestLineBreak(typesetter, start, 130)
    if count <= 0 { fatalError("no progress") }
    let line = CTTypesetterCreateLine(typesetter, CFRange(location:start,length:count))
    print("LINE",start,count,CTLineGetStringRange(line))
    CTLineEnumerateCaretOffsets(line) { offset,index,leading,stop in
        print(String(format:"%.2f",offset),index,leading ? "leading" : "trailing")
    }
    start += count
}

```
