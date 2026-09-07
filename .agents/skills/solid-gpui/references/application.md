# Application Development Conventions

## Entry points and reactivity

`@solid-gpui/core/runtime` provides this project's Solid universal runtime. Read the
package README for the current JSX transform configuration: jsxImportSource supplies
types and does not replace universal compilation. Resolve Solid's client reactive
implementation with the browser condition. Follow the existing createRoot/transport
and router combination; an existing root owns the event receive loop.

A Solid component normally initializes once. Read signals in JSX properties,
memos, or tracked effects; destructuring reactive props during initialization loses
subsequent updates. Use createMemo for expensive filtering and runtime batch for
related signal changes. Bind timers, listeners, and cleanup to the Solid owner or
Surface. Follow Solid lifecycle semantics when reasoning about effects and memos.

Native code owns windows, painting, caret/selection/IME, and transient scrolling.
Solid owns application business state. Give each state value one authority rather
than advancing it independently in JS and Rust. See root CONTEXT.md and ADR-0012.

## Page structure and scrolling

Follow the Gallery App: a stable shell contains the header, navigation, content
pane, and footer; Outlet replaces page content. Navigation pane identity and scroll
handles survive route selection. Navigation and content scroll independently.
Decide content scroll reset as a page-transition UX choice while preserving
navigation position.

Style is this project's protocol type, not the full browser CSS model. Inspect
`renderer/types.ts` and native `paint/style.rs`. In particular, `gap`, `alignItems`,
and `justifyContent` implicitly enable flex column. Ordinary vertical content flow
can use block plus margins; use flex when distributing space. Constrain scroll
viewport dimensions. Panes consuming remaining space need minWidth/minHeight and
shrink constraints so long content overflows inside them.

Use the window-size store/hook and compute breakpoints reactively. For single-line
buttons, line height, padding, and borders determine height. Let descriptions,
multiline inputs, and dynamic cards grow. Test wide → narrow → wide transitions
and verify that the final content remains reachable.

## Lists

Use `VirtualList` for large datasets. itemKey returns a stable unique string or
number identifying the same business item after filtering/reordering. Only call
renderItem for the committed range. estimatedItemSize is an estimate, not a promise
of equal row heights. Overscan reduces edge blanking at the cost of creation,
protocol, and layout work; choose it from measurements.

When the scrolling window advances by one row, overlapping items retain their
Solid owners and Host Nodes. Only entering items are created and leaving items
cleaned up. A `key` alone does not guarantee reuse: verify renderItem counts and
disposal. Reuse requires an unchanged value and absolute index; replacing a value
or changing its index must update content even with the same key. Derive ranges
from the actual painted viewport and add overscan once. Native overdraw measurement
ranges are not visible ranges; reporting from both stale wheel offsets and paint
can cause range oscillation.

The first frame needs a visible native boundary with height. Gallery's 320px
container needs flex column; public VirtualList style belongs on the boundary,
with the inner List filling it. After restoring empty data or filtering from the
end, the committed range satisfies `0 <= start <= end <= count`, and nonempty data
produces actual rows. Applications use public list APIs rather than internal
properties such as `__rangeStart`.

## Input, events, and expensive work

Use the real TextInput component and its change/selection/command APIs. Controlled
value round trips preserve native caret/IME state. A clickable Text does not provide
editable input behavior.

Register onPointerMove, drag, and scroll listeners only when consumed. Measure
frequency and work per event before putting full sorting, long-tree reconstruction,
or per-event logging in handlers. Event batches have ordering semantics; see
ADR-0010.

Move computation to the appropriate runtime's background work or an asynchronous
native service, then commit a bounded result to the UI. A generated native client
provides the call channel; inspect the registered handler to establish whether
execution leaves the UI thread. Display pending/error states and define cancellation
or stale-result handling for repeated requests.

## Themes, extensions, and generated code

Read semantic colors from the reactive theme. Check text, background, border,
placeholder, disabled, selected, and hover states together. Nested Text may not
inherit the intended control color; inspect actual painting.

Use generated components and function clients exported by the package. Extension
capabilities come from HostProfile/registry. Missing host adapters produce explicit
errors. Add wire data through protocol.bop and its generator; add native functions
through Rust definitions and native-codegen. Verify generated output alongside its
source and checker.

## FPS monitor

Native `gpui-performance::PerformanceMonitor` supplies window-level performance
information for any GPUI host; see the
[crate README](../../../../crates/gpui-performance/README.md). It is a native entity,
not a JSX component or JS timer estimate. The solid-gpui provider host integrates
it; custom hosts create one entity during initialization and put it in the window
overlay. Applications do not need a second per-frame protocol stream.

Native Extension Snapshots and Patches must share capability rules. An analysis
button's disabled update once terminated the runtime because Patch omitted the
Extension listener. Initial-mount-only or native-function-only tests miss this
path. New controls must cover property changes with listeners present and listener
replacement/removal.
