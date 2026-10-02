# Ticket06 delivery: native document selection and search

Branch: `adopt/06-selection-search`. Baseline `e879b66` ancestry verified.

## Public integration interface

Core `<Text selectable>` paragraphs participate automatically, including styled
inline core Text children. Host Node ID owns identity; accessibility labels are
native acceptance locators. No JS selection mirror or new canonical wire schema.
Cmd/Ctrl+C copies a native cross-paragraph drag; Shift-click/arrows extend;
Home/End, Up/Down use native text positions; Cmd/Ctrl+A and Escape operate on the
committed selectable document. Word/paragraph multiple-click selection is native.
TextInput and native Input/Editor/TextView keep independent native control state.

`solid_gpui::native::text::native_module()` defines the generated
`solid-gpui-text` service. DefaultHostProfile and NativeModules register it;
custom registries must register it explicitly. Core component host bindings
export it through `@solid-gpui/core/components`; application hosts export it
through their generated `#native`. Regenerate all host bindings on integration.

Exact generated client methods (all also accept NativeCallOptions):

```ts
getTextSelection(): Promise<TextSelectionSnapshot>;
setTextSelection(request: TextSelectionRequest): Promise<TextSelectionSnapshot>;
clearTextSelection(): Promise<TextSelectionSnapshot>;
copyTextSelection(): Promise<TextSelectionSnapshot>;
searchText(request: { query: string }): Promise<TextSearchSnapshot>;
getTextSearch(): Promise<TextSearchSnapshot>;
selectTextSearchMatch(request: {
  textRevision: number; searchRevision: number; matchIndex: number;
}): Promise<TextSelectionSnapshot>;
```

`TextPosition = {nodeId: number, offset: number}` uses UTF-16 offsets at grapheme
boundaries. `TextSpan = {nodeId: number, start: number, end: number}`.
TextSelectionRequest contains textRevision, anchor and head.
TextSelectionSnapshot contains textRevision, selectionRevision, nullable anchor
and head, ordered spans, text, detached. TextSearchSnapshot contains textRevision,
searchRevision, query, matches (arrays of spans), nullable activeMatch, truncated.
`TextSelectionObserver` has no data props and optional onSelectionChange callback
receiving textRevision, selectionRevision, searchRevision, selectedParagraphs,
detached. Observer events contain metadata, never selected text. Generated
listener lifecycle handles teardown and revisions.

New Rust extension seam: `CommandDefinition::renderer(name, fn(I, &mut SolidRoot,
&mut Window, &mut Context<SolidRoot>) -> Result<O,String>)` and default
NativeModule::invoke_renderer dispatch. Existing foreground/worker commands keep
their execution paths. Renderer commands receive the already-owned validated
surface, preventing global lookup and root entity reborrows.

## Lifecycle and work bounds

Selection owns at most 4,096 spans and 256 KiB full paragraph snapshot bytes.
Copy joins paragraphs with newline, inline styled runs without separators.
Native geometry is per painted frame, clipped, revision checked and reshaped
after resize/style changes. Drag scans painted paragraphs and resolves a bounded
selected interval; no JS handlers run per text hit. Search flattens at most
256 KiB/4,096 committed paragraphs per new query/text revision; query <=4,096
bytes; results <=1,024 matches/4,096 spans. Grapheme positions are precomputed once
per search and match mapping uses binary search. Paint reads per-node ranges.
Text data/order/materialization changes invalidate search; style-only changes do
not. Stale search/selection requests reject atomically.

Reorder resolves the interval between stable endpoints in new document order.
Ordinary selected deletion/content change clears cross-paragraph selection;
single-paragraph edits preserve existing clamping at new grapheme boundaries.
VirtualList eviction retains bounded selected bytes when the list and dataRevision
stay live and the row is outside its materialization window. Overlapping mounted
selected rows permit continued drag; no overlap keeps the last resolved range.
Data revision/filter/list removal clears snapshots. Edge drag scrolls native
VirtualList in bounded steps, one continuation per active drag; release/epoch
changes stop it. Independent Surfaces own state; closed surfaces/epoch replacement
clear document/search/paint geometry/observer/focus state.

## Verification

Use the shared target with CARGO_BUILD_JOBS=2:

```sh
export CARGO_BUILD_JOBS=2
export CARGO_TARGET_DIR=/private/var/folders/sl/42r5kc756mj96rlxkglrpr900000gn/T/opencode/solid-gpui-adoption-target
cargo test -p solid-gpui --lib selection --no-default-features
bun scripts/native-codegen.ts
bun scripts/native-codegen.ts --check
bun --conditions=browser test examples/website/tests/components.test.ts examples/website/tests/content.test.ts examples/website/tests/markdown.test.ts
cargo +nightly-2026-07-28 build --locked -p solid-gpui-web --target wasm32-unknown-unknown --release
wasm-bindgen "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/solid_gpui_web.wasm" --out-dir examples/website/src/wasm --target web
bun run --cwd examples/website build:frontend
```

11 focused native selection/control tests passed before the final integration,
including actual shaped cross-node styled Unicode drag/copy and keyboard
extension; search geometry; stale range/search rejection and reorder; independent
surfaces; virtual eviction, retained clipboard shortcut, data reset/cleanup;
observer metadata and removal. Existing per-node selection and native TextInput
clipboard/undo behavior pass in the same filter. Website content/catalog/example
checks: 5 passed. Host-generated bindings regenerated. WASM host and frontend
production build passed (existing wasm-bindgen eval/chunk-size warnings).

These are deterministic actual GPUI layout/input/paint tests, not physical OS
event/display qualification. macOS TestAppContext with linked patched gpui-pre
0.3.7, debug test profile, windows 300–320 ×150–180 logical px; no monitor or
refresh-rate performance claim. Physical foreground macOS, Windows/Linux,
browser clipboard permission and IME qualification remain integration acceptance
limits. Ordinary scrolling containers keep wheel/trackpad behavior; automatic
edge drag applies to core VirtualList. Search sees only materialized core Text,
not hidden JS data or native editor documents.

## Documentation and acceptance integration

Synchronized authoritative text-selection.md and explicit zh-CN copy,
documentation index, native-composition and Rust bridge links/commands, package
README, website guide navigation/translation, generated API catalog, observer
navigation group and an executable styled Unicode selection/search example.
Maintained documentation describes this project's behavior only.

ReferenceStudio's `studio.text.history-{index}.{paragraph}` and
`studio.review.text.{paragraph}` core selectable nodes need no new props. Use
native locator bounds/text positions to drag between the first two paragraphs,
then Cmd/Ctrl+C, verify exact joined clipboard text, click another control, and
unmount/reopen to verify resource cleanup. Programmatic search must use the
generated client bound to that Surface. Combined native acceptance tests should
query getTextSelection/getTextSearch rather than infer selected bytes from a
screenshot. Ticket04 remains the canonical schema owner.

Before final report merge `integrate/comparison-adoption` into this branch;
do not merge this branch into integration or publish. Reconcile renderer region
paint registries carefully: cached selectable elements must publish current
geometry, and native selection/search invalidation must reach the owning region.
