/**
 * The navigation groups of the Components catalog, mirroring the native family
 * table in `docs/gpui-components.md`.
 *
 * Group order is the sidebar order and the previous/next order of the component
 * pages. Inside a group, pages appear alphabetically, so a group lists its members
 * alphabetically as well. Every catalog page names exactly one group; the catalog
 * refuses to build for a page it cannot place or for a name without a page.
 *
 * Compound parts stay with their owning page, so this lists owners only:
 * `component-families.ts` decides which names share a page.
 */

export const componentGroups = [
  {
    label: "Basic controls",
    members: [
      "Alert",
      "Avatar",
      "Badge",
      "BaseButton",
      "BaseCheckbox",
      "BaseSwitch",
      "BaseToggle",
      "Button",
      "ButtonGroup",
      "Checkbox",
      "Clipboard",
      "Icon",
      "Kbd",
      "Label",
      "Link",
      "Pagination",
      "Progress",
      "ProgressCircle",
      "Radio",
      "Rating",
      "Separator",
      "ShimmerText",
      "Skeleton",
      "Spinner",
      "Switch",
      "Tag",
      "Toggle",
      "ToggleGroup",
    ],
  },
  {
    label: "Editing and choices",
    members: [
      "Calendar",
      "Caret",
      "ColorPicker",
      "Combobox",
      "DatePicker",
      "Editor",
      "Input",
      "NumberInput",
      "OtpInput",
      "Select",
      "Slider",
      "Textarea",
    ],
  },
  {
    label: "Data and scrolling",
    members: [
      "Command",
      "DataTable",
      "FocusTrap",
      "List",
      "ScrollShadow",
      "Scrollable",
      "Table",
      "Text",
      "TextView",
      "Tree",
      "VirtualList",
    ],
  },
  {
    label: "Composition",
    members: [
      "Accordion",
      "Breadcrumb",
      "Carousel",
      "Collapsible",
      "DescriptionList",
      "Empty",
      "Form",
      "GroupBox",
      "ResizablePanelGroup",
      "Stepper",
      "TabBar",
    ],
  },
  {
    label: "Messages and attachments",
    members: ["Attachment", "Bubble", "Marker", "Message"],
  },
  {
    label: "Navigation and settings",
    members: ["Settings", "Sidebar", "StatusBar", "TitleBar", "WindowBorder"],
  },
  {
    label: "Overlays",
    members: [
      "AlertDialog",
      "AppMenuBar",
      "ContextMenu",
      "Dialog",
      "DropdownButton",
      "DropdownMenu",
      "HoverCard",
      "NativeMenu",
      "Notification",
      "Popover",
      "PopupMenu",
      "Sheet",
      "Tooltip",
    ],
  },
  {
    label: "Docking",
    members: ["DockArea"],
  },
  {
    label: "Charts",
    members: ["AreaChart", "BarChart", "CandlestickChart", "LineChart", "PieChart", "RadarChart", "SankeyChart"],
  },
  {
    label: "Low-level drawing",
    members: ["Plot"],
  },
  {
    label: "Motion and presence",
    members: ["Motion", "NativePresence"],
  },
];
