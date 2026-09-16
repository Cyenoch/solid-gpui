/**
 * When each component's documentation was created and last updated, and how the website
 * marks new work.
 *
 * Seeded from repository history: `created` is the commit that first documented the
 * component, `updated` the last commit that changed its documentation entry, in
 * `component-examples.ts`, `component-recipes/` or `component-variants.ts`. Set both when
 * you document a new component and bump `updated` when you change an existing one; the
 * catalog refuses to build for a component that has no record here.
 *
 * Documentation written before `newBadgeEpoch` is never marked new, so introducing the
 * rule does not relabel an established catalog at once.
 */

export type ComponentDocumentation = {
  /** The date the component's documentation was written. */
  created: string;
  /** The date its documentation last changed. */
  updated: string;
};

export const componentDocumentation: Record<string, ComponentDocumentation> = {
  Accordion: { created: "2026-09-08", updated: "2026-09-08" },
  AccordionItem: { created: "2026-09-08", updated: "2026-09-08" },
  Alert: { created: "2026-09-08", updated: "2026-09-08" },
  AlertDialog: { created: "2026-09-08", updated: "2026-09-08" },
  AppMenuBar: { created: "2026-09-08", updated: "2026-09-08" },
  AreaChart: { created: "2026-09-08", updated: "2026-09-08" },
  Attachment: { created: "2026-09-08", updated: "2026-09-08" },
  AttachmentActions: { created: "2026-09-08", updated: "2026-09-08" },
  AttachmentContent: { created: "2026-09-08", updated: "2026-09-08" },
  AttachmentDescription: { created: "2026-09-08", updated: "2026-09-08" },
  AttachmentGroup: { created: "2026-09-08", updated: "2026-09-08" },
  AttachmentMedia: { created: "2026-09-08", updated: "2026-09-08" },
  AttachmentTitle: { created: "2026-09-08", updated: "2026-09-08" },
  Avatar: { created: "2026-09-08", updated: "2026-09-08" },
  AvatarGroup: { created: "2026-09-08", updated: "2026-09-08" },
  Badge: { created: "2026-09-08", updated: "2026-09-08" },
  BarChart: { created: "2026-09-08", updated: "2026-09-08" },
  BaseButton: { created: "2026-09-11", updated: "2026-09-11" },
  BaseCheckbox: { created: "2026-09-11", updated: "2026-09-11" },
  BaseSwitch: { created: "2026-09-11", updated: "2026-09-11" },
  BaseToggle: { created: "2026-09-11", updated: "2026-09-11" },
  Breadcrumb: { created: "2026-09-08", updated: "2026-09-08" },
  BreadcrumbItem: { created: "2026-09-08", updated: "2026-09-08" },
  Bubble: { created: "2026-09-08", updated: "2026-09-08" },
  BubbleContent: { created: "2026-09-08", updated: "2026-09-08" },
  BubbleGroup: { created: "2026-09-08", updated: "2026-09-08" },
  BubbleReactions: { created: "2026-09-08", updated: "2026-09-08" },
  Button: { created: "2026-09-08", updated: "2026-09-08" },
  ButtonGroup: { created: "2026-09-08", updated: "2026-09-08" },
  Calendar: { created: "2026-09-08", updated: "2026-09-08" },
  CandlestickChart: { created: "2026-09-08", updated: "2026-09-08" },
  Caret: { created: "2026-09-08", updated: "2026-09-08" },
  Carousel: { created: "2026-09-11", updated: "2026-09-11" },
  CarouselItem: { created: "2026-09-11", updated: "2026-09-11" },
  Checkbox: { created: "2026-09-08", updated: "2026-09-08" },
  Clipboard: { created: "2026-09-08", updated: "2026-09-08" },
  Collapsible: { created: "2026-09-08", updated: "2026-09-08" },
  ColorPicker: { created: "2026-09-08", updated: "2026-09-08" },
  Combobox: { created: "2026-09-08", updated: "2026-09-16" },
  Command: { created: "2026-09-08", updated: "2026-09-08" },
  ContextMenu: { created: "2026-09-08", updated: "2026-09-08" },
  DataTable: { created: "2026-09-08", updated: "2026-09-08" },
  DatePicker: { created: "2026-09-08", updated: "2026-09-08" },
  DescriptionItem: { created: "2026-09-08", updated: "2026-09-08" },
  DescriptionList: { created: "2026-09-08", updated: "2026-09-08" },
  DescriptionText: { created: "2026-09-08", updated: "2026-09-08" },
  Dialog: { created: "2026-09-08", updated: "2026-09-08" },
  DialogAction: { created: "2026-09-08", updated: "2026-09-08" },
  DialogClose: { created: "2026-09-08", updated: "2026-09-08" },
  DialogContent: { created: "2026-09-08", updated: "2026-09-08" },
  DialogDescription: { created: "2026-09-08", updated: "2026-09-08" },
  DialogFooter: { created: "2026-09-08", updated: "2026-09-08" },
  DialogHeader: { created: "2026-09-08", updated: "2026-09-08" },
  DialogTitle: { created: "2026-09-08", updated: "2026-09-08" },
  DockArea: { created: "2026-09-08", updated: "2026-09-08" },
  DropdownButton: { created: "2026-09-08", updated: "2026-09-08" },
  DropdownMenu: { created: "2026-09-08", updated: "2026-09-08" },
  Editor: { created: "2026-09-08", updated: "2026-09-11" },
  Empty: { created: "2026-09-15", updated: "2026-09-15" },
  EmptyContent: { created: "2026-09-15", updated: "2026-09-15" },
  EmptyDescription: { created: "2026-09-15", updated: "2026-09-15" },
  EmptyHeader: { created: "2026-09-15", updated: "2026-09-15" },
  EmptyMedia: { created: "2026-09-15", updated: "2026-09-15" },
  EmptyTitle: { created: "2026-09-15", updated: "2026-09-15" },
  Field: { created: "2026-09-08", updated: "2026-09-08" },
  FocusTrap: { created: "2026-09-08", updated: "2026-09-08" },
  Form: { created: "2026-09-08", updated: "2026-09-08" },
  GroupBox: { created: "2026-09-08", updated: "2026-09-08" },
  HoverCard: { created: "2026-09-08", updated: "2026-09-08" },
  Icon: { created: "2026-09-08", updated: "2026-09-11" },
  Input: { created: "2026-09-08", updated: "2026-09-08" },
  Kbd: { created: "2026-09-08", updated: "2026-09-08" },
  Label: { created: "2026-09-08", updated: "2026-09-08" },
  LineChart: { created: "2026-09-08", updated: "2026-09-08" },
  Link: { created: "2026-09-08", updated: "2026-09-08" },
  List: { created: "2026-09-08", updated: "2026-09-08" },
  ListItem: { created: "2026-09-08", updated: "2026-09-08" },
  ListSeparatorItem: { created: "2026-09-08", updated: "2026-09-08" },
  Marker: { created: "2026-09-08", updated: "2026-09-11" },
  MarkerContent: { created: "2026-09-08", updated: "2026-09-11" },
  MarkerIcon: { created: "2026-09-08", updated: "2026-09-11" },
  Message: { created: "2026-09-08", updated: "2026-09-08" },
  MessageAvatar: { created: "2026-09-08", updated: "2026-09-08" },
  MessageContent: { created: "2026-09-08", updated: "2026-09-08" },
  MessageFooter: { created: "2026-09-08", updated: "2026-09-08" },
  MessageGroup: { created: "2026-09-08", updated: "2026-09-08" },
  MessageHeader: { created: "2026-09-08", updated: "2026-09-08" },
  MessageScroller: { created: "2026-09-08", updated: "2026-09-08" },
  Motion: { created: "2026-09-11", updated: "2026-09-11" },
  NativeMenu: { created: "2026-09-08", updated: "2026-09-08" },
  NativePresence: { created: "2026-09-11", updated: "2026-09-11" },
  Notification: { created: "2026-09-08", updated: "2026-09-15" },
  NumberInput: { created: "2026-09-08", updated: "2026-09-08" },
  OtpInput: { created: "2026-09-08", updated: "2026-09-08" },
  Pagination: { created: "2026-09-08", updated: "2026-09-08" },
  PieChart: { created: "2026-09-08", updated: "2026-09-08" },
  Plot: { created: "2026-09-08", updated: "2026-09-08" },
  PlotCrossLine: { created: "2026-09-08", updated: "2026-09-08" },
  PlotDot: { created: "2026-09-08", updated: "2026-09-08" },
  PlotTooltip: { created: "2026-09-08", updated: "2026-09-08" },
  Popover: { created: "2026-09-08", updated: "2026-09-08" },
  PopupMenu: { created: "2026-09-08", updated: "2026-09-08" },
  Progress: { created: "2026-09-08", updated: "2026-09-08" },
  ProgressCircle: { created: "2026-09-08", updated: "2026-09-08" },
  RadarChart: { created: "2026-09-08", updated: "2026-09-08" },
  Radio: { created: "2026-09-08", updated: "2026-09-08" },
  RadioGroup: { created: "2026-09-08", updated: "2026-09-08" },
  Rating: { created: "2026-09-08", updated: "2026-09-08" },
  ResizablePanel: { created: "2026-09-08", updated: "2026-09-08" },
  ResizablePanelGroup: { created: "2026-09-08", updated: "2026-09-08" },
  SankeyChart: { created: "2026-09-08", updated: "2026-09-08" },
  ScrollShadow: { created: "2026-09-08", updated: "2026-09-11" },
  Scrollable: { created: "2026-09-08", updated: "2026-09-16" },
  SearchableListItemElement: { created: "2026-09-08", updated: "2026-09-08" },
  Select: { created: "2026-09-08", updated: "2026-09-16" },
  Separator: { created: "2026-09-08", updated: "2026-09-08" },
  SettingCustomItem: { created: "2026-09-08", updated: "2026-09-08" },
  SettingField: { created: "2026-09-08", updated: "2026-09-08" },
  SettingGroup: { created: "2026-09-08", updated: "2026-09-08" },
  SettingItem: { created: "2026-09-08", updated: "2026-09-08" },
  SettingPage: { created: "2026-09-08", updated: "2026-09-08" },
  Settings: { created: "2026-09-08", updated: "2026-09-08" },
  Sheet: { created: "2026-09-08", updated: "2026-09-08" },
  ShimmerText: { created: "2026-09-08", updated: "2026-09-08" },
  Sidebar: { created: "2026-09-08", updated: "2026-09-11" },
  SidebarFooter: { created: "2026-09-08", updated: "2026-09-11" },
  SidebarGroup: { created: "2026-09-08", updated: "2026-09-11" },
  SidebarHeader: { created: "2026-09-08", updated: "2026-09-11" },
  SidebarMenu: { created: "2026-09-08", updated: "2026-09-11" },
  SidebarMenuItem: { created: "2026-09-08", updated: "2026-09-11" },
  SidebarToggleButton: { created: "2026-09-08", updated: "2026-09-11" },
  Skeleton: { created: "2026-09-08", updated: "2026-09-08" },
  Slider: { created: "2026-09-08", updated: "2026-09-08" },
  Spinner: { created: "2026-09-08", updated: "2026-09-08" },
  StatusBar: { created: "2026-09-08", updated: "2026-09-08" },
  Stepper: { created: "2026-09-08", updated: "2026-09-08" },
  StepperItem: { created: "2026-09-08", updated: "2026-09-08" },
  Switch: { created: "2026-09-08", updated: "2026-09-08" },
  Tab: { created: "2026-09-08", updated: "2026-09-08" },
  TabBar: { created: "2026-09-08", updated: "2026-09-08" },
  Table: { created: "2026-09-08", updated: "2026-09-08" },
  TableBody: { created: "2026-09-08", updated: "2026-09-08" },
  TableCaption: { created: "2026-09-08", updated: "2026-09-08" },
  TableCell: { created: "2026-09-08", updated: "2026-09-08" },
  TableFooter: { created: "2026-09-08", updated: "2026-09-08" },
  TableHead: { created: "2026-09-08", updated: "2026-09-08" },
  TableHeader: { created: "2026-09-08", updated: "2026-09-08" },
  TableRow: { created: "2026-09-08", updated: "2026-09-08" },
  Tag: { created: "2026-09-08", updated: "2026-09-08" },
  Text: { created: "2026-09-08", updated: "2026-09-08" },
  TextView: { created: "2026-09-08", updated: "2026-09-11" },
  Textarea: { created: "2026-09-08", updated: "2026-09-08" },
  TitleBar: { created: "2026-09-08", updated: "2026-09-08" },
  Toggle: { created: "2026-09-08", updated: "2026-09-08" },
  ToggleGroup: { created: "2026-09-08", updated: "2026-09-08" },
  Tooltip: { created: "2026-09-08", updated: "2026-09-08" },
  Tree: { created: "2026-09-08", updated: "2026-09-08" },
  VirtualList: { created: "2026-09-08", updated: "2026-09-08" },
  WindowBorder: { created: "2026-09-08", updated: "2026-09-08" },
};

/** The date the New badge started applying. Earlier documentation is never marked new. */
export const newBadgeEpoch = "2026-09-15";
/** How long a component keeps its New badge after its documentation date. */
export const newBadgeWindowDays = 30;

/** Whether a component documented on `created` is still new at `now`. */
export function isNewComponent(created: string, now: Date = new Date()): boolean {
  const written = parseDocumentationDate(created);
  const epoch = parseDocumentationDate(newBadgeEpoch);
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const elapsedDays = (today.getTime() - written.getTime()) / 86_400_000;
  return written >= epoch && elapsedDays >= 0 && elapsedDays <= newBadgeWindowDays;
}

/** Parse a `YYYY-MM-DD` documentation date, rejecting anything else. */
export function parseDocumentationDate(value: string): Date {
  const parsed = new Date(`${value}T00:00:00`);
  if (Number.isNaN(parsed.getTime())) throw new Error(`Invalid documentation date: ${value}`);
  return parsed;
}
