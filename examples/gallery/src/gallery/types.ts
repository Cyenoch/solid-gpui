import type { IconName, SolidElement } from "@solid-gpui/core";

export type SolidChild =
  | SolidElement
  | (() => SolidChild)
  | string
  | number
  | boolean
  | null
  | undefined
  | readonly SolidChild[]
  | SolidChild[];

export type PageId =
  | "overview"
  | "view"
  | "text"
  | "pressable"
  | "text-input"
  | "image"
  | "virtual-list"
  | "flexbox"
  | "styling"
  | "events"
  | "drag-drop"
  | "transitions"
  | "cursors"
  | "native-platform"
  | "native-controls"
  | "native-overlays"
  | "native-settings"
  | "native-dock"
  | "native-charts"
  | "todo-app"
  | "form-builder"
  | "benchmark"
  | "palette";

export type CategoryId =
  | "getting-started"
  | "components"
  | "layout-styling"
  | "interaction"
  | "motion"
  | "native"
  | "mini-apps";

export interface CategoryInfo {
  readonly id: CategoryId;
  readonly title: string;
  readonly icon: IconName;
  readonly description: string;
}

export interface PageInfo {
  readonly id: PageId;
  readonly title: string;
  readonly category: CategoryId;
  readonly description: string;
  readonly badge?: string;
  readonly icon: IconName;
  readonly keywords: readonly string[];
}

export const CATEGORIES: readonly CategoryInfo[] = [
  {
    id: "getting-started",
    title: "Getting Started",
    icon: "lucide:star",
    description: "Introduction and core architectural concepts",
  },
  {
    id: "components",
    title: "Core Components",
    icon: "lucide:puzzle",
    description: "Built-in native host components",
  },
  {
    id: "layout-styling",
    title: "Layout & Styling",
    icon: "lucide:palette",
    description: "Flexbox, box model, borders, and shadows",
  },
  {
    id: "interaction",
    title: "Interactions & Events",
    icon: "lucide:zap",
    description: "Pointer, keyboard, focus, and drag-and-drop",
  },
  {
    id: "motion",
    title: "Motion & Effects",
    icon: "lucide:play",
    description: "Transitions, animations, and native cursors",
  },
  {
    id: "native",
    title: "Native Platform",
    icon: "lucide:monitor",
    description: "Window APIs, clipboard, notifications, and menus",
  },
  {
    id: "mini-apps",
    title: "Mini Applications",
    icon: "lucide:rocket",
    description: "Interactive examples",
  },
];

export const PAGES: readonly PageInfo[] = [
  {
    id: "overview",
    title: "Overview & Features",
    category: "getting-started",
    description: "Solid GPUI architecture and capabilities",
    icon: "lucide:book-open",
    badge: "Core",
    keywords: ["solid", "gpui", "architecture", "overview", "intro", "welcome"],
  },
  {
    id: "view",
    title: "View",
    category: "components",
    description: "The fundamental container element for layout, styling, and event handling",
    icon: "lucide:box",
    badge: "Host",
    keywords: ["view", "container", "box", "div", "layout", "host"],
  },
  {
    id: "text",
    title: "Text",
    category: "components",
    description: "Typography component with font styling, selectable text, and line clamping",
    icon: "lucide:type",
    badge: "Host",
    keywords: ["text", "typography", "font", "bold", "italic", "selectable", "clamp"],
  },
  {
    id: "pressable",
    title: "Pressable",
    category: "components",
    description: "Interactive click/press wrapper with hover, press states, and accessibility",
    icon: "lucide:mouse-pointer-click",
    badge: "Host",
    keywords: ["pressable", "button", "click", "press", "hover", "touch", "action"],
  },
  {
    id: "text-input",
    title: "TextInput",
    category: "components",
    description: "Single and multi-line editable text input with selection and event hooks",
    icon: "lucide:text-cursor-input",
    badge: "Host",
    keywords: ["textinput", "input", "textarea", "forms", "edit", "typing"],
  },
  {
    id: "image",
    title: "Image",
    category: "components",
    description: "Native image rendering with objectFit scaling, fallback, and layout rules",
    icon: "lucide:image",
    badge: "Host",
    keywords: ["image", "picture", "photo", "objectfit", "contain", "cover"],
  },
  {
    id: "virtual-list",
    title: "VirtualList",
    category: "components",
    description: "Virtualized list rendering tens of thousands of items with low memory footprint",
    icon: "lucide:list",
    badge: "Perf",
    keywords: ["virtuallist", "virtualization", "scroll", "large", "list", "infinite"],
  },
  {
    id: "flexbox",
    title: "Flexbox Layout",
    category: "layout-styling",
    description: "Interactive playground for direction, justification, alignment, and gap",
    icon: "lucide:layout",
    keywords: ["flexbox", "flex", "layout", "justify", "align", "gap", "row", "column"],
  },
  {
    id: "styling",
    title: "Styles & Shadows",
    category: "layout-styling",
    description: "Borders, border radius, multi-layer shadows, background colors, and opacity",
    icon: "lucide:layers",
    keywords: ["styles", "shadows", "borders", "radius", "elevation", "colors", "boxshadow"],
  },
  {
    id: "events",
    title: "Event System",
    category: "interaction",
    description: "PointerDown, PointerUp, PointerMove, PointerDownOutside, Focus, and Key events",
    icon: "lucide:mouse-pointer-click",
    keywords: ["events", "pointer", "mouse", "keyboard", "focus", "blur", "hover"],
  },
  {
    id: "drag-drop",
    title: "Drag & Drop",
    category: "interaction",
    description: "Item dragging, custom drop zones, drag types, and external OS file drops",
    icon: "lucide:move",
    badge: "New",
    keywords: ["drag", "drop", "draggable", "files", "payload", "transfer"],
  },
  {
    id: "transitions",
    title: "Transitions & Motion",
    category: "motion",
    description: "CSS-like transitions for opacity, backgroundColor, width, and height",
    icon: "lucide:refresh-cw",
    keywords: ["transition", "animation", "motion", "easing", "duration", "fade"],
  },
  {
    id: "cursors",
    title: "Native Cursors",
    category: "motion",
    description: "Desktop cursor styles supported by GPUI",
    icon: "lucide:mouse-pointer-click",
    keywords: ["cursor", "pointer", "grab", "resize", "crosshair", "mouse"],
  },
  {
    id: "native-platform",
    title: "Native Platform APIs",
    category: "native",
    description: "Window sizing, clipboard read/write, file pickers, notifications, and menus",
    icon: "lucide:monitor",
    badge: "System",
    keywords: ["native", "platform", "window", "clipboard", "dialog", "notification", "menu"],
  },
  {
    id: "native-controls",
    title: "Native Controls",
    category: "native",
    description: "Inputs, pickers and data views",
    icon: "lucide:puzzle",
    keywords: ["gpui-component", "native", "controls"],
  },
  {
    id: "native-overlays",
    title: "Menus & Overlays",
    category: "native",
    description: "Native menus, dialogs, sheets and notifications",
    icon: "lucide:puzzle",
    keywords: ["gpui-component", "native", "overlays"],
  },
  {
    id: "native-settings",
    title: "Settings",
    category: "native",
    description: "Searchable native application preferences",
    icon: "lucide:puzzle",
    keywords: ["gpui-component", "native", "settings"],
  },
  {
    id: "native-dock",
    title: "Dock & Tiles",
    category: "native",
    description: "Tabs, draggable panes and layout persistence",
    icon: "lucide:puzzle",
    keywords: ["gpui-component", "native", "dock"],
  },
  {
    id: "native-charts",
    title: "Charts & Plot",
    category: "native",
    description: "Seven native charts and plot geometry",
    icon: "lucide:puzzle",
    keywords: ["gpui-component", "native", "charts"],
  },
  {
    id: "todo-app",
    title: "Todo Task Manager",
    category: "mini-apps",
    description: "Task manager with reactive state and filtering",
    icon: "lucide:check-square",
    badge: "Demo",
    keywords: ["todo", "tasks", "app", "crud", "store", "reactive"],
  },
  {
    id: "form-builder",
    title: "Form & Validation",
    category: "mini-apps",
    description: "Form with multi-field input, validation, and submission",
    icon: "lucide:clipboard-list",
    badge: "Demo",
    keywords: ["form", "validation", "submit", "fields", "errors", "inputs"],
  },
  {
    id: "benchmark",
    title: "Reactive Benchmark",
    category: "mini-apps",
    description: "Stress test signal updates, batching, and render throughput",
    icon: "lucide:gauge",
    badge: "Benchmark",
    keywords: ["benchmark", "stress", "performance", "fps", "signals", "speed"],
  },
  {
    id: "palette",
    title: "Color Palette Studio",
    category: "mini-apps",
    description: "Design-system token explorer with live theme preview",
    icon: "lucide:palette",
    badge: "Studio",
    keywords: ["palette", "colors", "tokens", "design", "theme", "generator"],
  },
];

export function pagePath(id: PageId): string {
  return id === "overview" ? "/" : `/${id}`;
}

export function pageFromPathname(pathname: string): PageInfo | undefined {
  if (pathname === "/" || pathname === "") return PAGES.find((page) => page.id === "overview");
  const id = pathname.startsWith("/") ? pathname.slice(1) : pathname;
  return PAGES.find((page) => page.id === id);
}

export interface NavTab {
  readonly label: string;
  readonly pageId: PageId;
  readonly category: CategoryId;
  readonly icon: IconName;
}

export const NAV_TABS: readonly NavTab[] = [
  { label: "Overview", pageId: "overview", category: "getting-started", icon: "lucide:book-open" },
  { label: "Components", pageId: "view", category: "components", icon: "lucide:puzzle" },
  { label: "Layout", pageId: "flexbox", category: "layout-styling", icon: "lucide:layout" },
  { label: "Events", pageId: "events", category: "interaction", icon: "lucide:mouse-pointer-click" },
  { label: "Motion", pageId: "transitions", category: "motion", icon: "lucide:refresh-cw" },
  { label: "Native", pageId: "native-platform", category: "native", icon: "lucide:monitor" },
  { label: "Demos", pageId: "todo-app", category: "mini-apps", icon: "lucide:rocket" },
];
