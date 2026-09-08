# gpui-component from Solid

The SDK exposes **144 generated JSX components/descriptors and 13 native functions (8 computation, 5 appearance)** from `@solid-gpui/core/components`. The linked native implementation is gpui-component 0.6.0 at `928c3eb776a3d733d9b771f7dea27a6a79242ced`, with the required declarative state seams recorded in `vendor/gpui-component/SOLID-GPUI.md`.

Importing a component selects its real GPUI implementation. Solid owns application data and child composition. Native entities own focus, text editing, scrolling, menu interaction, docking and in-flight native work. Native callbacks read committed data and enqueue events; they do not synchronously execute JS.

```tsx
import { createSignal } from "solid-js";
import { Button, Input } from "@solid-gpui/core/components";

const [name, setName] = createSignal("");
<Input value={name()} onChange={(change) => setName(change.value)} />;
<Button label="Clear" onPress={() => setName("")} />;
```

The generated file is the API reference: `packages/solid-gpui/src/components.ts`. Do not edit it. `bun run task native-codegen` generates SDK and website host bindings from their actual Rust hosts; `bun run task native-codegen-check` verifies them. Custom native components, props, events and commands use the same generator.

## Coverage

| Native family | JS entry points |
| --- | --- |
| Basic controls | Alert, Avatar/AvatarGroup, Badge, Button/ButtonGroup, Toggle/ToggleGroup, Checkbox, Clipboard, Icon, Kbd, Label, Link, Pagination, Progress/ProgressCircle, Radio/RadioGroup, Rating, Separator, ShimmerText, Skeleton, Spinner, Switch, Tag |
| Editing and choices | Input, Textarea, Editor, NumberInput, OtpInput, ColorPicker, Slider, Calendar, DatePicker, Select, Combobox, Caret |
| Data and scrolling | List/ListItem/ListSeparatorItem, SearchableListItemElement, DataTable, Tree, VirtualList, MessageScroller, Command, TextView/Text, Scrollable, FocusTrap |
| Composition | Accordion/AccordionItem, Breadcrumb/BreadcrumbItem, Collapsible, DescriptionList/DescriptionItem/DescriptionText, Form/Field, GroupBox, ResizablePanelGroup/ResizablePanel, Stepper/StepperItem, Tab/TabBar |
| Messages and attachments | All Attachment, Bubble, Marker and Message elements in the generated catalog |
| Navigation and settings | Sidebar/Header/Footer/ToggleButton/Group/Menu/MenuItem, Settings/SettingPage/SettingGroup/SettingItem/SettingField/SettingCustomItem, StatusBar, TitleBar, WindowBorder |
| Overlays | Dialog/AlertDialog and DialogContent/Description/Footer/Close/Action/Header/Title, Sheet, Popover, HoverCard, Tooltip, PopupMenu, ContextMenu, DropdownMenu, DropdownButton, AppMenuBar, NativeMenu, Notification |
| Docking | DockArea; its layout descriptors create actual native TabGroup and TilesState containers |
| Charts | LineChart, AreaChart, BarChart, CandlestickChart, PieChart, RadarChart, SankeyChart |
| Low-level drawing | Plot with axis/grid/labels/line/area/bar/radialLine/arc primitives; PlotTooltip, PlotCrossLine, PlotDot |
| Appearance | useNative().getTheme/setTheme, setApplicationTheme, getMotionPreference/setMotionPreference; application theme tokens and motion preferences |
| Computation | useNative().scaleLinear/scalePoint/scaleBand/scaleOrdinal, pieArcs, arcCentroid, stackSeries, sankeyLayout |

Some upstream types are parts of another control, not independent screen elements:

- The host installs native **Root**, its **NotificationList**, text-selection layer, modal layers and theme once per window. Dialogs, sheets and notifications use that Root.
- **Scrollbar** and **ScrollableMask** are integrated by Scrollable and the list/table/message-scroller APIs, with the corresponding native scroll handle. **FocusTrapContainer** is the native implementation behind FocusTrap; **DropdownMenuPopover** is behind DropdownMenu.
- **FieldBuilder** is represented by Field's text/child slots. Command entries/groups, table columns/groups, tree items, date presets, menu items and chart labels are typed props or child descriptors. No draw-nothing aliases stand in for them.
- Native **DivInspector** is the host's debug inspector, available through its native shortcut (`cmd-alt-i` on macOS, `ctrl-shift-i` elsewhere) when the dependency enables it. It is a conditional developer tool, not an always-present production JSX element.
- Rust traits, renderer hooks, geometry handles, transition helpers and registries remain implementation machinery. Their JS-facing operations are expressed through the controls, native commands and typed data above.

The complete pinned upstream requirement inventory is `.scratch/gpui-component-complete/upstream-inventory.md`. It includes constructor descriptors and internal/conditional types separately from the 138 ordinary public rendering interfaces.

## Popover presentation scope

`Popover` renders inside its current GPUI window and cannot extend beyond that
window's boundary. It supports native GPUI focus and dismissal behavior, but does
not create an AppKit `NSPopover` or a separate system popup window. The SDK does
not currently export `SystemPopover`. See the
[native presentation research](../.scratch/native-presentation/spec.md) for the
proposed child-Surface design and platform support gaps.

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
