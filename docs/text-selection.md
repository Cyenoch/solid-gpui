# Native text selection and search

Separate core `<Text selectable>` paragraphs share one native selection per
Surface. GPUI owns directed anchor/head positions, shaped styled-text geometry,
dragging, keyboard movement, copy, and search highlights. Solid owns composition
and optional notification callbacks. TextInput, Input, Editor and TextView keep
their own native editing and selection behavior.

Drag across paragraphs, then press Cmd+C on macOS or Ctrl+C on other platforms.
Copy joins paragraphs with a newline and joins inline styled Text runs without
separators. Shift-click extends the selection; arrow keys move by grapheme,
Shift+arrows extend, Home/End move within a paragraph, Cmd/Ctrl+A selects the
committed selectable document, and Escape clears it. Double-click selects a word
and triple-click selects a paragraph. A single selectable paragraph retains its
existing copy and content-update clamping behavior.

## Generated surface service

The stock host and `NativeModules` include the `solid-gpui-text` catalog. Regenerate
bindings with your host's `prepare` command. Import its generated `useNative`
client (for example `#native`), or the stock component host's client from
`@solid-gpui/core/components`. Calls target the invoking Surface, including popup
Surfaces. Custom ExtensionRegistry implementations must register the module
returned by `solid_gpui::native::text::native_module()`.

```tsx
import { Text, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { TextSelectionObserver, Button, useNative } from "@solid-gpui/core/components";

export default function Document() {
  const native = useNative();
  const [status, setStatus] = createSignal("Drag across the paragraphs, then copy.");
  return (
    <View style={{ gap: 12 }}>
      <TextSelectionObserver onSelectionChange={event =>
        setStatus(`${event.selectedParagraphs} selected paragraphs`)} />
      <Text selectable>Hello <Text style={{ color: "#5271ff" }}>🙂 新值</Text></Text>
      <Text selectable>A second paragraph with é and searchable text.</Text>
      <Button label="Find searchable" onPress={async () => {
        const result = await native.searchText({ query: "searchable" });
        if (result.matches.length) await native.selectTextSearchMatch({
          textRevision: result.textRevision,
          searchRevision: result.searchRevision,
          matchIndex: 0,
        });
      }} />
      <Text>{status()}</Text>
    </View>
  );
}
```

The service exports `getTextSelection`, `setTextSelection`, `clearTextSelection`,
`copyTextSelection`, `searchText`, `getTextSearch`, and `selectTextSearchMatch`.
Selection positions are `{ nodeId, offset }`; offsets count UTF-16 units and must
land on grapheme boundaries. `setTextSelection` requires the `textRevision` from
`getTextSelection`. Its reply includes anchor, head, ordered spans, copied text,
`selectionRevision`, and `detached`. Invalid or stale requests reject atomically.
Node IDs are native Host Node identities, not item indices or accessibility labels.
Use native acceptance locators for mouse-driven tests.

`TextSelectionObserver` emits metadata only: `textRevision`, `selectionRevision`,
`searchRevision`, `selectedParagraphs`, and `detached`. Removing the observer ends
the subscription through the normal generated listener-generation lifecycle.
Applications should query copied bytes when needed rather than mirror native
selection on every mouse movement.

## Search and lifecycle

Search is literal, case-sensitive, Unicode-exact, and includes only committed
selectable core Text paragraphs. A newline in the query can match across
paragraphs. An empty query clears highlights. Search returns ordered matches,
`textRevision`, `searchRevision`, `activeMatch`, and `truncated`. Selecting a match
requires both revisions, paints the selection, focuses its first paragraph, and
reveals its native VirtualList row when applicable. Search highlights reuse the
same shaped geometry as selection. Normal style/resize changes reshape geometry
without invalidating text results; content, order, eligibility, virtual data or
materialization changes invalidate the search cache. Reissue the query after a
revision notification. No hidden JavaScript list rows are searched.

Selection is bounded to 4,096 paragraphs and 256 KiB of retained text. Search
accepts at most 4,096 query bytes and 256 KiB/4,096 committed paragraphs; results
stop at 1,024 matches or 4,096 spans and report `truncated`. Oversized selection or
search rejects explicitly. Search paint work uses per-node cached ranges; drag
hit testing scans the current painted selectable paragraphs, including clipping.

Reorder preserves stable anchor/head identities and resolves their new document
interval. Editing or deleting any selected paragraph clears a cross-paragraph
selection; a single paragraph clamps to the updated grapheme boundaries. A native
VirtualList eviction retains bounded selected bytes only while the list and its
dataRevision remain live and the row is outside the committed window. `detached`
then reports true. Drag extension requires an overlapping selected mounted row;
without overlap it keeps the last resolved selection.
Native VirtualList edge dragging scrolls in bounded native steps and schedules
only one continuation per active drag; mouse release or epoch teardown stops it.
Ordinary scroll containers retain their existing wheel/trackpad scrolling.
Replacing/filtering list
data or removing the list clears retained bytes. Epoch replacement and surface
destruction release selections, search, geometry, and observers. Independent
Surfaces never share these resources.

## Qualification

GPUI TestAppContext tests exercise actual shaped layout, cross-paragraph native
mouse drag, styled CJK/emoji/combining-character copy, keyboard extension, generated
command dispatch, revisions, reorder, virtual eviction and cleanup. These tests
do not establish physical OS event delivery or display presentation. Qualify
foreground drag/copy, scroll while dragging, narrow/wide resize, follow-up clicks,
and teardown on each target platform. Browser clipboard permissions still follow
the Web host's platform limits; semantic TestHost has no native geometry.
