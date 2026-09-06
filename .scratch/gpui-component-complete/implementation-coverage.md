# gpui-component implementation coverage

Baseline: upstream `928c3eb776a3d733d9b771f7dea27a6a79242ced`, local workspace baseline `1bf74901244d8af95d7d547de5a0fb5ce362b2fc` plus this task's changes. The [upstream inventory](upstream-inventory.md) is the requirement snapshot; this file records the implementation mapping. Presence in this table is API/source coverage, not an assertion that every control has received manual hardware acceptance.

The running host currently generates **144 JSX components/descriptors and 10 native functions (8 computations, 2 appearance operations)**. There are no unmatched entries among the **138 ordinary upstream rendering interfaces**. Composite native machinery is mapped to its public owner instead of exported as a draw-nothing JSX alias. The generated [SDK catalog](../../packages/solid-gpui/src/components.ts), [adapter source](../../crates/solid-gpui/src/components/mod.rs), [usage guide](../../docs/gpui-components.md) and [vendor changes](../../vendor/gpui-component/SOLID-GPUI.md) are authoritative.

## Rendering interfaces: 138

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
| TabGroup / TilesState | DockArea layout tabs/tiles nodes; native drag, focus, resize, menu and undo behavior |
| Panel traits / registry / DockAreaState | DockPane configuration and typed DockSnapshot, with stable native pane entities |
| Scroll/resize/focus entities | Retained per committed control; accessed through typed control ref commands |
| DivInspector | Conditional native host inspector and native keyboard shortcut |
| Theme mode | getTheme/setTheme foreground commands; system appearance observer; native Component/Base synchronization |
| IconName / Size / theme and style enums | Typed icon, control-size and style data; native runtime owns theme and assets |

## Verification ledger

- Native upstream libraries: **773 gpui-base + 423 gpui-component tests passed**, including close/move split proportions, settings keys, pagination page-jump, native plot and menu behavior.
- Root component integration: **25 tests passed** before Gallery acceptance; an additional foreground theme contract/invalid-input test covers appearance; exercises invalid publication, actual native child descriptors, stateful controls, owned overlays, geometry, Dock snapshots and menu action lifetime.
- Five Gallery routes use the generated SDK directly: `/native-controls`, `/native-overlays`, `/native-settings`, `/native-dock`, `/native-charts`. Their route/resize tests and real-window acceptance are recorded below when complete.
- [Numeric review findings](prop-validation-review.md): bounded Rating/PageButtons/grid columns/spans/pixels implemented before publication; DescriptionItem checks parent column span. Pagination ellipsis now uses a native jump input with constant opening work, including `u32::MAX` page counts.
- Deterministic GPUI rendering, protocol tests, TypeScript route tests and native GPU interactions are separate evidence. No presentation timing claim follows from library tests.
