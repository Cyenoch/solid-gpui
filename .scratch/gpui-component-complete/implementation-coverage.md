# gpui-component implementation coverage

Historical baseline: upstream `928c3eb776a3d733d9b771f7dea27a6a79242ced`, with the September 15 `501c73923280859a5de2b16fe64d4aac960bb040` additions (Empty) and removals (tiles canvas). The current pin is `0e63ea799766c486022a0cecfda6e48c5183a2d7`, which also adds InputGroup and Questionnaire and expands input, text and motion. The tables below remain a historical implementation mapping, not a current catalog count or manual hardware acceptance claim.

At the September 15 baseline, the host generated **159 JSX components/descriptors and 10 native functions (8 computations, 2 appearance operations)** with mappings for **144 ordinary upstream rendering interfaces**. Composite native machinery maps to its public owner rather than draw-nothing JSX aliases. For current support, the generated [SDK catalog](../../packages/solid-gpui/src/components.ts), [adapter source](../../crates/solid-gpui/src/components/mod.rs), [usage guide](../../docs/gpui-components.md) and [vendor changes](../../vendor/gpui-kit/SOLID-GPUI.md) are authoritative.

## Rendering interfaces: 144

| Pinned public interface | JS / host entry | Implementation boundary |
| --- | --- | --- |
| stateless / `alert::Alert` | `Alert` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `avatar::Avatar` | `Avatar` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `avatar::AvatarGroup` | `AvatarGroup` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `badge::Badge` | `Badge` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `button::Button` | `Button` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `button::ButtonGroup` | `ButtonGroup` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `button::DropdownButton` | `DropdownButton` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `button::Toggle` | `Toggle` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `button::ToggleGroup` | `ToggleGroup` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `checkbox::Checkbox` | `Checkbox` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `clipboard::Clipboard` | `Clipboard` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `root reexport::Icon` | `Icon` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `kbd::Kbd` | `Kbd` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `label::Label` | `Label` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `link::Link` | `Link` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `pagination::Pagination` | `Pagination` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `progress::Progress` | `Progress` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `progress::ProgressCircle` | `ProgressCircle` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `radio::Radio` | `Radio` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `radio::RadioGroup` | `RadioGroup` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `rating::Rating` | `Rating` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `separator::Separator` | `Separator` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `shimmer::ShimmerText` | `ShimmerText` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `skeleton::Skeleton` | `Skeleton` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `spinner::Spinner` | `Spinner` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `switch::Switch` | `Switch` | Generated JSX entry; native type/state is used by the adapter. |
| stateless / `tag::Tag` | `Tag` | Generated JSX entry; native type/state is used by the adapter. |
| input / `input::Input` | `Input` | Generated JSX entry; native type/state is used by the adapter. |
| input / `input::Textarea` | `Textarea` | Generated JSX entry; native type/state is used by the adapter. |
| input / `input::Editor` | `Editor` | Generated JSX entry; native type/state is used by the adapter. |
| input / `input::NumberInput` | `NumberInput` | Generated JSX entry; native type/state is used by the adapter. |
| input / `input::OtpInput` | `OtpInput` | Generated JSX entry; native type/state is used by the adapter. |
| input / `color_picker::ColorPicker` | `ColorPicker` | Generated JSX entry; native type/state is used by the adapter. |
| input / `slider::Slider` | `Slider` | Generated JSX entry; native type/state is used by the adapter. |
| input / `calendar::Calendar` | `Calendar` | Generated JSX entry; native type/state is used by the adapter. |
| input / `date_picker::DatePicker` | `DatePicker` | Generated JSX entry; native type/state is used by the adapter. |
| input / `select::Select` | `Select` | Generated JSX entry; native type/state is used by the adapter. |
| input / `combobox::Combobox` | `Combobox` | Generated JSX entry; native type/state is used by the adapter. |
| input / `select / combobox reexport::Caret` | `Caret` | Generated JSX entry; native type/state is used by the adapter. |
| stateful data / `list::List` | `List` | Generated JSX entry; native type/state is used by the adapter. |
| stateful data / `list::ListItem` | `ListItem` | Generated JSX entry; native type/state is used by the adapter. |
| stateful data / `list::ListSeparatorItem` | `ListSeparatorItem` | Generated JSX entry; native type/state is used by the adapter. |
| stateful data / `searchable_list / select alias::SearchableListItemElement` | `SearchableListItemElement` | Generated JSX entry; native type/state is used by the adapter. |
| stateful data / `table::DataTable` | `DataTable` | Generated JSX entry; native type/state is used by the adapter. |
| stateful data / `tree::Tree` | `Tree` | Generated JSX entry; native type/state is used by the adapter. |
| stateful data / `root reexport::VirtualList` | `VirtualList` | Generated JSX entry; native type/state is used by the adapter. |
| stateful data / `message_scroller::MessageScroller` | `MessageScroller` | Generated JSX entry; native type/state is used by the adapter. |
| stateful data / `command::Command` | `Command` | Generated JSX entry; native type/state is used by the adapter. |
| stateful data / `text::TextView` | `TextView` | Generated JSX entry; native type/state is used by the adapter. |
| stateful data / `text::Text` | `Text` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `accordion::Accordion` | `Accordion` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `accordion::AccordionItem` | `AccordionItem` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `breadcrumb::Breadcrumb` | `Breadcrumb` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `breadcrumb::BreadcrumbItem` | `BreadcrumbItem` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `collapsible::Collapsible` | `Collapsible` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `description_list::DescriptionList` | `DescriptionList` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `description_list::DescriptionText` | `DescriptionText` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `form::Form` | `Form` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `form::Field` | `Field` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `form::FieldBuilder` | `Field` | Named label/description/error/control content slots on the real native Field. |
| composition / `group_box::GroupBox` | `GroupBox` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `attachment::Attachment` | `Attachment` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `attachment::AttachmentMedia` | `AttachmentMedia` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `attachment::AttachmentContent` | `AttachmentContent` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `attachment::AttachmentTitle` | `AttachmentTitle` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `attachment::AttachmentDescription` | `AttachmentDescription` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `attachment::AttachmentActions` | `AttachmentActions` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `attachment::AttachmentGroup` | `AttachmentGroup` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `bubble::Bubble` | `Bubble` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `bubble::BubbleContent` | `BubbleContent` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `bubble::BubbleGroup` | `BubbleGroup` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `bubble::BubbleReactions` | `BubbleReactions` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `marker::Marker` | `Marker` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `marker::MarkerIcon` | `MarkerIcon` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `marker::MarkerContent` | `MarkerContent` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `empty::Empty` | `Empty` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `empty::EmptyHeader` | `EmptyHeader` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `empty::EmptyMedia` | `EmptyMedia` | Generated JSX entry; native type/state is used by the adapter; `variant` selects framed icon or unframed media. |
| composition / `empty::EmptyTitle` | `EmptyTitle` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `empty::EmptyDescription` | `EmptyDescription` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `empty::EmptyContent` | `EmptyContent` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `message::Message` | `Message` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `message::MessageGroup` | `MessageGroup` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `message::MessageAvatar` | `MessageAvatar` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `message::MessageHeader` | `MessageHeader` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `message::MessageContent` | `MessageContent` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `message::MessageFooter` | `MessageFooter` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `sidebar::Sidebar` | `Sidebar` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `sidebar::SidebarToggleButton` | `SidebarToggleButton` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `sidebar::SidebarHeader` | `SidebarHeader` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `sidebar::SidebarFooter` | `SidebarFooter` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `setting::Settings` | `Settings` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `status_bar::StatusBar` | `StatusBar` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `stepper::Stepper` | `Stepper` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `stepper::StepperItem` | `StepperItem` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `tab::Tab` | `Tab` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `tab::TabBar` | `TabBar` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `table::Table` | `Table` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `table::TableHeader` | `TableHeader` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `table::TableBody` | `TableBody` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `table::TableFooter` | `TableFooter` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `table::TableRow` | `TableRow` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `table::TableHead` | `TableHead` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `table::TableCell` | `TableCell` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `table::TableCaption` | `TableCaption` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `root reexport::TitleBar` | `TitleBar` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `root reexport::WindowBorder` | `WindowBorder` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `root reexport::Root` | `ComponentHost` | Installed once per native window; dialog, sheet, notification and selection commands use this owner. |
| composition / `scroll::Scrollable` | `Scrollable` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `scroll::Scrollbar` | `Scrollable, List, DataTable, VirtualList, MessageScroller` | Native scrollbar bound to the control's retained scroll handle. |
| composition / `scroll::ScrollableMask` | `Scrollable` | Native edge mask attached to the retained scroll container. |
| composition / `root / resizable reexports::ResizablePanelGroup` | `ResizablePanelGroup` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `root / resizable reexports::ResizablePanel` | `ResizablePanel` | Generated JSX entry; native type/state is used by the adapter. |
| composition / `root::FocusTrapElement return type::FocusTrapContainer` | `FocusTrap` | Concrete native focus-trap container. |
| overlays / `dialog::Dialog` | `Dialog` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `dialog::AlertDialog` | `AlertDialog` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `dialog::DialogContent` | `DialogContent` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `dialog::DialogDescription` | `DialogDescription` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `dialog::DialogFooter` | `DialogFooter` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `dialog::DialogClose` | `DialogClose` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `dialog::DialogAction` | `DialogAction` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `dialog::DialogHeader` | `DialogHeader` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `dialog::DialogTitle` | `DialogTitle` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `sheet::Sheet` | `Sheet` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `popover::Popover` | `Popover` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `hover_card::HoverCard` | `HoverCard` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `tooltip::Tooltip` | `Tooltip` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `menu::PopupMenu` | `PopupMenu` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `menu::ContextMenu` | `ContextMenu` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `menu::AppMenuBar` | `AppMenuBar` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `notification::Notification` | `Notification` | Generated JSX entry; native type/state is used by the adapter. |
| overlays / `notification::NotificationList` | `Notification / ComponentHost` | Native Root notification stack owns placement, timers and dismissal. |
| overlays / `menu::DropdownMenu return type::DropdownMenuPopover` | `DropdownMenu` | Real native popover factory behind DropdownMenu. |
| dock / `dock reexport::DockArea` | `DockArea` | Generated JSX entry; native type/state is used by the adapter. |
| charts / `chart::AreaChart` | `AreaChart` | Generated JSX entry; native type/state is used by the adapter. |
| charts / `chart::LineChart` | `LineChart` | Generated JSX entry; native type/state is used by the adapter. |
| charts / `chart::BarChart` | `BarChart` | Generated JSX entry; native type/state is used by the adapter. |
| charts / `chart::CandlestickChart` | `CandlestickChart` | Generated JSX entry; native type/state is used by the adapter. |
| charts / `chart::PieChart` | `PieChart` | Generated JSX entry; native type/state is used by the adapter. |
| charts / `chart::RadarChart` | `RadarChart` | Generated JSX entry; native type/state is used by the adapter. |
| charts / `chart::SankeyChart` | `SankeyChart` | Generated JSX entry; native type/state is used by the adapter. |
| charts / `plot::tooltip::CrossLine` | `PlotCrossLine` | Real native plot tooltip cross-line helper. |
| charts / `plot::tooltip::Dot` | `PlotDot` | Real native plot tooltip dot helper. |
| charts / `plot::tooltip::Tooltip` | `PlotTooltip` | Real native plot tooltip, distinct from ordinary Tooltip. |

## Composition descriptors: 18

| Upstream descriptor | Implemented JS representation |
| --- | --- |
| DescriptionItem | Typed DescriptionItem child of DescriptionList; span checked against parent columns before publication |
| CommandItem, CommandGroup, CommandEntry | Command sections and typed CommandChoice entries; stable keys, native selection/query/confirmation |
| SidebarGroup, SidebarMenu, SidebarMenuItem | Typed JSX children consumed by native Sidebar composition |
| SettingPage, SettingGroup, SettingItem, SettingField | Typed JSX hierarchy; stable page/group keys and native field render boundary |
| PopupMenuItem | MenuSpec/MenuItem tagged data; element entries reference committed child slots |
| Column, ColumnGroup | DataTable columns and groupHeaders; native sorting, resize, move, selection and custom slots |
| TreeItem | Keyed TreeNode data; native expansion, lazy load and visible row rendering |
| DateRangePreset | DatePicker presets with CivilDate and DateValue |
| RadarLabel, SankeyLabel | Typed labels and custom child slot indices; native label layout |

## Plot and auxiliary native surfaces

| Upstream API | Implemented representation |
| --- | --- |
| PlotAxis / AxisText, Grid, PlotLabel / Text, Line, Area, Bar, RadialLine, Arc / ArcData | Plot primitive tagged union; compiled to real native plots at commit |
| ScaleLinear / Point / Band / Ordinal | Native scaleLinear, scalePoint, scaleBand, scaleOrdinal commands |
| Pie / Stack / Sankey | Native pieArcs, arcCentroid, stackSeries, sankeyLayout commands; typed results |
| NativeMenu | NativeMenu JSX trigger and show command; real OS popup and scoped native action routes |
| TabGroup | DockArea layout tab nodes; native drag, focus, resize, menu and undo behavior. The upstream tiles canvas was removed at the current pin, so no tile layout, state or command remains. |
| Panel traits / registry / DockAreaState | DockPane configuration and typed DockSnapshot, with stable native pane entities |
| Scroll/resize/focus entities | Retained per committed control; accessed through typed control ref commands |
| DivInspector | Conditional native host inspector and native keyboard shortcut |
| Theme mode | getTheme/setTheme foreground commands; system appearance observer; native Component/Base synchronization |
| IconName / Size / theme and style enums | Typed icon, control-size and style data; native runtime owns theme and assets |

## Verification ledger

- Native upstream libraries (2026-09-15, after the `501c7392` / `gpui-pre 0.3.5` sync, run in the vendored workspace): **1478 tests passed, 0 failed** across `gpui-base`, `gpui-component`, `gpui-kit` and `gpui-fps`, including close/move split proportions, settings keys, pagination page-jump, native plot and menu behavior. The earlier 773 + 423 counts describe the previous pin.
- Root testing (2026-09-15): `cargo test --workspace` is green, including **200 passing `solid-gpui` unit tests** (1 ignored) with invalid publication, native child descriptors, stateful controls, owned overlays, geometry, Dock snapshots and menu action lifetime. The website suite (`bun run --cwd examples/website test`) passes 6 tests, including the check that every exported component has type-checking examples against the generated SDK — the new `Empty` parts included. A real-window smoke run of the native website host rendered the showcase page with its stateful controls under `gpui-pre 0.3.5`; visual acceptance of the individual catalog previews remains with the user.
- The website (`examples/website`) drives the generated SDK directly: `/components/$` renders every catalog entry with its live preview, and `/showcase/workspace|account|collections` compose stateful controls. Web and native smoke runs cover routing and rendering; per-component visual acceptance stays with the user.
- [Numeric review findings](prop-validation-review.md): bounded Rating/PageButtons/grid columns/spans/pixels implemented before publication; DescriptionItem checks parent column span. Pagination ellipsis now uses a native jump input with constant opening work, including `u32::MAX` page counts.
- Deterministic GPUI rendering, protocol tests, TypeScript route tests and native GPU interactions are separate evidence. No presentation timing claim follows from library tests.
