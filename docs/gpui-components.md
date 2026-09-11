# GPUI Kit from Solid

The SDK exposes generated native components, descriptors, and commands from
`@solid-gpui/core/components`. The implementation comes from [GPUI Kit](https://github.com/longbridge/gpui-kit)
at `05433bd8e9e75af2f3aa508141b78ba21bfa6261` (0.6.1 plus subsequent changes).
Local native state and lifecycle seams are recorded in
[`vendor/gpui-kit/SOLID-GPUI.md`](../vendor/gpui-kit/SOLID-GPUI.md).

Solid owns application data, routing, and child composition. Native entities own
focus, editing, scrolling, menus, docking, animation, and in-flight native work.
Native callbacks enqueue events; they do not synchronously execute Solid JS.

| Kit layer | How Solid GPUI uses it |
| --- | --- |
| `gpui-component` | Styled native controls, editor, Carousel, text, charts, and owned window overlays. The crate name stays `gpui-component`. |
| `gpui-base` | Native interaction/state, unstyled Base controls, transitions, springs, keyframes, stagger, and presence. |
| `gpui-kit` | Facade and headless interaction helpers in the isolated native integration test package. |
| `gpui-fps` | Explicitly enabled, per-window performance HUD; see [metric definitions](performance-analysis.md). |
| `gpui-kit-assets` | Only the default icons used internally by Kit controls. Application icons belong to Iconify. |

Runtime dependencies live in `vendor/gpui-kit`; `references/gpui-kit` is the
matching pinned upstream checkout. Bun and QuickJS continue to run Solid
applications. Shell is not linked; it has no role in Solid application state or routing.

```tsx
import { createSignal } from "solid-js";
import { Button, Input } from "@solid-gpui/core/components";

const [name, setName] = createSignal("");
<Input value={name()} onChange={(change) => setName(change.value)} />;
<Button label="Clear" onPress={() => setName("")} />;
```

The generated file is the API reference: `packages/solid-gpui/src/components.ts`. Do not edit it. `bun run task native-codegen` generates SDK and website host bindings from their actual Rust hosts; `bun run task native-codegen-check` verifies them. Custom native components, props, events and commands use the same generator.

With Vite's `native` option, `@solid-gpui/core/components` and Motion resolve their
component contracts from the configured host's generated bindings. Rust changes
automatically rebuild that host and replace the development session; see
[managed development sessions](hot-reload.md#managed-development-sessions).

## Coverage

### Carousel

`Carousel` retains one native viewport and selection state. Compose keyed
`CarouselItem` children; uncontrolled selection follows the same item when Solid
reorders it. Set `selectedIndex` to control selection, and acknowledge
`onChange({ index, editSeq })` with `ackEditSeq` when applying external values.
Use `ref.select(index)`, `next()`, `previous()`, or `getSelectedIndex()` for
imperative navigation. Optional `previous` and `next` slots replace the controls.
`orientation="vertical"` requires a positive `viewportHeight`; provide a bounded
width for a horizontal carousel. At most 1024 items are accepted.

```tsx
import { Carousel, CarouselItem, Label } from "@solid-gpui/core/components";
<Carousel viewportHeight={160} pagination looping>
  <CarouselItem accessibilityLabel="Overview"><Label text="Overview" /></CarouselItem>
  <CarouselItem accessibilityLabel="Details"><Label text="Details" /></CarouselItem>
</Carousel>;
```

### Editor selections and language rules

`Editor` enables native `autoClose` and `smartIndent` by default. Its retained
state supports multiple cursors, directed selections, native insertion, and undo.
`Input`, `NumberInput`, `Textarea`, and `Editor` expose `getSelections()` and
`setSelections({ ranges, editSeq })`. Each range has UTF-8 `anchorByte` and
`headByte` offsets; the first range is active. Multiple ranges require Editor.
Offsets inside a UTF-8 sequence or CRLF pair, stale edit sequences, and changes
during IME composition are rejected before any selection changes. The setter
accepts 1–1024 ranges and merges overlaps using the native editing rules.

Use `useNative().configureEditorLanguage()` to install native bracket pairs,
auto-closing pairs with `notIn` contexts, and indentation patterns. Configuration
is application-wide per language; validate/install it before opening the editor.
Patterns are Rust regular expressions, bounded to 4096 bytes and a 1 MiB compiled
program. Changing the language rules does not add a syntax grammar.

### Markdown metadata and icon sources

`TextView format="markdown" frontmatter` enables top-level YAML metadata rendering.
Simple supported scalars render as a description list; compound or unsupported
YAML uses the native code-block renderer. Frontmatter is opt-in and requires
Markdown. It is a display extension, not a general-purpose YAML parser.

Component icon slots accept a registered Iconify name or `{ svg: "<svg …>…</svg>" }` through
`ComponentIcon`. The standalone generated `Icon` uses `source`, for example
`<Icon source="lucide:check" />`. SVG is validated and retained natively, with
a 64 KiB source limit. This also works in Button, menu, sidebar, settings, tree,
command, table, and dock icon slots. The core `Icon name="…"` API and its offline
Iconify catalog remain available; see [Iconify](iconify.md).

### Native motion and custom controls

`Motion` takes a target `{ x, y, opacity }` and an `animation` discriminated by
`type`: `transition`, `spring`, or `keyframes`. Omitted target fields mean zero
offset and full opacity. Sampling and repaint requests stay in GPUI; completion
emits `onComplete({ playbackId })`. A new `playbackId` restarts keyframes.
Transitions animate changes from the current target; springs preserve velocity.
Keyframes accept 2–128 stops, direction and repeat count (`null` repeats).
Transition/keyframe `stagger` uses `{ index, count, intervalMs, origin }` with
`origin` equal to `first`, `last`, or `center`. Duration and absolute delay are
bounded to 60 seconds. Native motion respects the host's reduced-motion setting.

Use the Solid `Presence` helper to retain the child's owner through its exit:

```tsx
import { Presence } from "@solid-gpui/core/motion";
import { Label } from "@solid-gpui/core/components";
<Presence show={open()} durationMs={180} reveal>
  {() => <Label text="Details" />}
</Presence>;
```

Reversing an exit preserves the existing child. Stale completion events cannot
dispose a newer child; final exit and parent cleanup dispose its resources.
`NativePresence` is the generated low-level view for applications managing their
own child lifetime. Application routing continues to belong to `@solid-gpui/router`.

`BaseButton`, `BaseCheckbox`, `BaseSwitch`, and `BaseToggle` provide native keyboard,
pointer, focus, and accessibility behavior with application-defined children and
style. Supply `accessibilityLabel`, controlled state and callbacks; BaseCheckbox
supports `unchecked`, `checked`, and `indeterminate`.

| Native family            | JS entry points                                                                                                                                                                                                                              |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Basic controls           | Alert, Avatar/AvatarGroup, Badge, Button/ButtonGroup, Toggle/ToggleGroup, Checkbox, Clipboard, Icon, Kbd, Label, Link, Pagination, Progress/ProgressCircle, Radio/RadioGroup, Rating, Separator, ShimmerText, Skeleton, Spinner, Switch, Tag |
| Editing and choices      | Input, Textarea, Editor, NumberInput, OtpInput, ColorPicker, Slider, Calendar, DatePicker, Select, Combobox, Caret                                                                                                                           |
| Data and scrolling       | List/ListItem/ListSeparatorItem, SearchableListItemElement, DataTable, Tree, VirtualList, MessageScroller, Command, TextView/Text, Scrollable, ScrollShadow, FocusTrap                                                                       |
| Composition              | Accordion/AccordionItem, Breadcrumb/BreadcrumbItem, Collapsible, DescriptionList/DescriptionItem/DescriptionText, Form/Field, GroupBox, ResizablePanelGroup/ResizablePanel, Stepper/StepperItem, Tab/TabBar                                  |
| Messages and attachments | All Attachment, Bubble, Marker and Message elements in the generated catalog                                                                                                                                                                 |
| Navigation and settings  | Sidebar/Header/Footer/ToggleButton/Group/Menu/MenuItem, Settings/SettingPage/SettingGroup/SettingItem/SettingField/SettingCustomItem, StatusBar, TitleBar, WindowBorder                                                                      |
| Overlays                 | Dialog/AlertDialog and DialogContent/Description/Footer/Close/Action/Header/Title, Sheet, Popover, HoverCard, Tooltip, PopupMenu, ContextMenu, DropdownMenu, DropdownButton, AppMenuBar, NativeMenu, Notification                            |
| Docking                  | DockArea; its layout descriptors create actual native TabGroup and TilesState containers                                                                                                                                                     |
| Charts                   | LineChart, AreaChart, BarChart, CandlestickChart, PieChart, RadarChart, SankeyChart                                                                                                                                                          |
| Low-level drawing        | Plot with axis/grid/labels/line/area/bar/radialLine/arc primitives; PlotTooltip, PlotCrossLine, PlotDot                                                                                                                                      |
| Appearance               | useNative().getTheme/setTheme, setApplicationTheme, getMotionPreference/setMotionPreference; application theme tokens and motion preferences                                                                                                 |
| Computation              | useNative().scaleLinear/scalePoint/scaleBand/scaleOrdinal, pieArcs, arcCentroid, stackSeries, sankeyLayout                                                                                                                                   |

Some upstream types are parts of another control, not independent screen elements:

- The host installs native **Root**, its **NotificationList**, text-selection layer, modal layers and theme once per window. Dialogs, sheets and notifications use that Root.
- **Scrollbar** and **ScrollableMask** are integrated by Scrollable and the list/table/message-scroller APIs, with the corresponding native scroll handle. **FocusTrapContainer** is the native implementation behind FocusTrap; **DropdownMenuPopover** is behind DropdownMenu.

Use **ScrollShadow** for a scrollable region with dynamic edge fades. By default it owns the scroll viewport, scrollbar, and native handle. A single
**direct child** may instead be a core `VirtualList` from `@solid-gpui/core` or a
generated native `VirtualList` from `@solid-gpui/core/components`, provided its
orientation matches `ScrollShadow`'s axis. In that composition the list keeps
ownership of its native scrolling and `ScrollShadow` borrows that viewport for
fades, `scrollbarVisibility`, `scrollTo`, `getScrollPosition`, and `onScroll`;
there is no outer scroll area or duplicate scrollbar. Bound the parent viewport
(including height for horizontal virtual lists), and let the direct child fill
it with `style={{ widthPercent: 100, heightPercent: 100 }}`.

This delegation is deliberately narrow: only exactly one matching-axis direct
child is recognized. Wrappers, multiple children, nested lists, and mismatched
orientations are not auto-discovered and retain standalone `ScrollShadow`
behavior. Core `VirtualList` data/renderItem virtualizes Solid owners and host
nodes; generated native `VirtualList` children virtualize only native row
rendering and still create every supplied Solid child. Do not expand a large
dataset into JSX children; use the core data/renderItem form for that workload.

Set `axis="horizontal"` for left/right edges or `axis="vertical"` (the default)
for top/bottom edges. A standalone vertical region needs a bounded height;
standalone horizontal regions can size their height from content. No nested
`Scrollable` is needed.

`color` matches the surrounding surface (default: theme background), and
`fadeSize` sets the maximum fade extent in pixels (default: 24; 0 disables fades).
Each fade shrinks and becomes transparent over the final `fadeSize` pixels toward
its boundary. At the start the leading edge is clear; at the end the trailing
edge is clear; content that fits has neither fades nor a scrollbar. Geometry is
read during native paint, so wheel input, dragging, `scrollTo`, and resize update
the effect without JavaScript scroll subscriptions. Overlays do not intercept
input and paint below the scrollbar. `onScroll` and `getScrollPosition` are also
available.

`scrollbarVisibility` defaults to `"always"` for discoverability and also supports
`"hover"` and `"scrolling"`. Reserve content padding along the scrollbar edge when
it would otherwise cover content. Markdown and component API tables on the website
use the same horizontal `ScrollShadow` on Web and desktop.

The complete pinned upstream requirement inventory is `.scratch/gpui-component-complete/upstream-inventory.md`. It includes constructor descriptors and internal/conditional types separately from the 138 ordinary public rendering interfaces.

## Popover presentation scope

`Popover` renders inside its current GPUI window and cannot extend beyond that
window's boundary. It supports native GPUI focus and dismissal behavior, but does
not create an AppKit `NSPopover` or a separate system popup window.

For content beyond the owner window, import `SystemPopover` from
`@solid-gpui/core`. It mounts a separate owned Surface and shares Solid context
through a content factory. See [System popovers](system-popover.md) for its API,
platform and multi-display behavior, and native acceptance limits.

## Children and native state

Native compound parents enforce their child type before publishing a commit. For example, AvatarGroup consumes Avatar, ButtonGroup consumes Button, ResizablePanelGroup consumes ResizablePanel, and Settings consumes SettingPage → SettingGroup → SettingItem → SettingField. A wrong child or invalid prop rejects the candidate commit and preserves the previous tree.

Named JSX slots use a separate `slots` object, for example `<Popover slots={{ trigger: <Button label="Open" /> }}>...</Popover>`. A scalar `title` prop remains distinct from `slots.title`. These slots are separate committed subtrees. They retain their host boundary when a native parent lays them out. Delegate-backed controls use stable data keys and explicit slot indices where their native API needs arbitrary content. Keep data keys stable through sorting, filtering and loading.

`ContextMenu` also places its trigger in `slots.trigger`; default children are custom menu content. The host loads both the component library's `icons/` assets and the SDK's Iconify assets. A native child without an explicit wrapper style keeps its parent's layout constraints; its identity and event scope add no layout box.

Controlled editing events include an edit sequence. The generated binding handles acknowledgements; application code supplies the value and responds to the semantic event. Focus, selection, composition and undo are not reset merely because a parent rerenders. Explicit replacement commands deliberately replace editor contents.

Lists, tables and trees render visible ranges. Search and load events describe the current native request; completion commands reject superseded requests. Large datasets should remain data, rather than a full row tree rebuilt for every mouse move.

## Appearance

`await useNative().setTheme("dark")` changes the application-wide native Component theme and its Base projection; `"light"` and `"system"` are also supported. `getTheme()` reports the selected mode and resolved dark flag. The host follows OS appearance by default. Keep your Solid style tokens synchronized with the same choice, as the Gallery does. Themes are native App globals, so this setting applies to all windows.

`setApplicationTheme` replaces application overrides for colors, typography,
radii, input backgrounds, and component metrics. Overrides remain applied when
`setTheme` changes the base appearance. For typed examples, titlebar setup, and
local icon registration, see [native application migration](native-migration.md).
`getMotionPreference` and `setMotionPreference` expose the `system`, `full`, and
`reduced` motion modes; see [native composition](native-composition.md).

These short UI operations use `CommandDefinition::foreground`; commands that perform computation or I/O continue through the Tokio executor. Foreground commands are typed and validated, require a mounted native window and must not block it.

## Settings

```tsx
import { Input, SettingField, SettingGroup, SettingItem, SettingPage, Settings } from "@solid-gpui/core/components";

<Settings>
  <SettingPage name="general" title="General">
    <SettingGroup name="identity" title="Identity">
      <SettingItem title="Display name" keywords={["profile"]}>
        <SettingField dirty={name() !== ""} onReset={() => setName("")}>
          <Input value={name()} onChange={(change) => setName(change.value)} />
        </SettingField>
      </SettingItem>
    </SettingGroup>
  </SettingPage>
</Settings>;
```

Page/group names identify native selection and state. Changing their order or filtering the search results does not transfer an item's editing state to another item. Settings exposes selection/search events and `select`, `setQuery`, `getState` commands.

SettingField uses the supplied native control. Pass `disabled` to that control when interaction should be disabled; a custom field's appearance cannot disable arbitrary descendants. Window-local ephemeral state can expire when an entire native subtree stops rendering; retained Input/Editor entities belong to committed host nodes and follow their own lifecycle.

## Docking and persistence

```tsx
import { DockArea, Input, Label, type DockAreaRef } from "@solid-gpui/core/components";

let dock: DockAreaRef | undefined;
<DockArea
  ref={(value) => (dock = value)}
  panes={[
    { name: "editor", title: "Editor", contentSlot: 0 },
    { name: "preview", title: "Preview", contentSlot: 1 },
  ]}
  initialLayout={{ center: { kind: "tabs", panes: ["editor", "preview"] } }}
>
  <Input value={name()} onChange={(change) => setName(change.value)} />
  <Label text={name()} />
</DockArea>;
```

Pane names identify retained native Panel entities; slot indices select children of DockArea. `titleSlot`, `titleSuffixSlot`, toolbar buttons, native menus and JSON `data` are supported. Reordering the pane configuration or updating content/chrome retains its entity and current layout. Closing a pane removes it from the layout; it may stay configured so `addPane` can reopen it. Removing it from `panes` releases its configured native entity. All references in `initialLayout` must name configured panes; update that declaration too when removing a configured pane.

`initialLayout` is applied at mount. Native user interaction subsequently owns geometry. Use `replaceLayout` for an explicit replacement, or `dump`/`load` for a typed snapshot that includes version, pane data, split sizes, active tabs, dock state and tile bounds/stacking order. Loading requires all referenced panes to be configured in the current owner; unknown panes fail before changing the layout.

Ref commands also support add/remove/move, tab selection, zoom, side-dock toggle/resize/collapsibility, tile geometry, bring-to-front, undo and redo. Moves address a pane name or a region, never a persistent native NodeId. A pane moved between containers does not emit Removed. Layout change events carry a revision; obtain a snapshot when needed, rather than serializing the whole layout at every drag step.

## Menus and overlays

PopupMenu, ContextMenu and DropdownMenu support keyed menu trees and custom child items. Open menu updates retain focus and reconcile the selected item by key. AppMenuBar reflects the existing application menu API, which remains the application-wide owner.

NativeMenu uses the operating system's popup implementation (the upstream drawn implementation on platforms without an OS backend). Its `contextMenu`, `press` and `manual` triggers and `show({x,y})` ref command use window coordinates. OS menus display the snapshot captured at opening; later prop changes apply to the next opening. Selection actions are scoped to the mounted view and revoked on unmount.

Dialogs, sheets and notifications use native owner/session tokens. Resolving an old dialog request cannot close a new dialog, and unloading an old owner cannot dismiss another owner's current overlay. Native overlay controls preserve their parent action/focus behavior. Notifications update by their current key/session and use the native stack and timers.

## Plot data and work limits

Charts compile data on prop commits and perform native hit testing/painting. Line/Area/Bar/Radar expose their native interactive tooltip behavior. The pinned Candlestick/Pie/Sankey APIs have no interactive tooltip setter. Radar labels can use committed child slots; bar fills support native gradients.

Plot coordinates are logical pixels. Radial and arc angles use radians. Native line/area/radial shapes omit null points using the upstream connection semantics; null is never converted to zero. Stack explicitly follows the native algorithm's zero value for missing entries. Pie omits zero/null slices and reports their original input indices. Sankey layout values may be scaled (for example square root); use the original link values for business totals.

The bridge rejects non-finite values, invalid ranges, invalid OHLC, cyclic/missing-node Sankey links, wrong series widths and unbounded work. Current limits include 16,384 plot items, 32 chart/stack series, 256 Plot primitives, 512 Sankey nodes/4,096 links/32 iterations, and 128 dock panes/256 containers/16 layout levels. Ordinal batched lookup also bounds the native linear-search work. Pagination's omitted-page picker uses constant work even for very large totals.

## Verification

Run `bun run ci` for the repository's generated contracts, TypeScript checks/tests and Rust checks. Audits and release builds run separately; see [continuous integration](ci.md). Native integration tests live beside the adapters and test state identity, stale requests, composition boundaries, menu routing, docking persistence and numeric/work guards. Real Gallery interaction and presentation checks are separate from deterministic TestAppContext rendering. See `docs/performance-analysis.md` for native measurement requirements.
