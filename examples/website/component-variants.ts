import { kitRecipes } from "./component-recipes/gpui-kit";
import { controlsRecipes } from "./component-recipes/controls";
import { presentationRecipes } from "./component-recipes/presentation";
import { compositionRecipes } from "./component-recipes/composition";
import { dataRecipes } from "./component-recipes/data";
import { chartsRecipes } from "./component-recipes/charts";
import { overlaysRecipes } from "./component-recipes/overlays";
/** Additional examples share the documentation and executable preview pipeline. */
export type ComponentVariant = { component: string; id: string; title: string; description: string; source: string };
const example = (id: string, title: string, description: string, jsx: string, setup = ""): ComponentVariant => ({
  component: "Button",
  id: `Button--${id}`,
  title,
  description,
  source: `import * as N from "@solid-gpui/core/components";
import { View${jsx.includes("<Icon") ? ", Icon" : ""} } from "@solid-gpui/core";
${setup ? `import { createSignal${setup.includes("onCleanup") ? ", onCleanup" : ""} } from "@solid-gpui/core/runtime";` : ""}

export default function Example() {
  ${setup}
  return <View style={{ gap: 16, alignItems: "center" }}>${jsx}</View>;
}`,
});
export const componentVariants: ComponentVariant[] = [
  ...kitRecipes,
  {
    component: "ScrollShadow",
    id: "ScrollShadow--vertical",
    title: "Vertical",
    description: "A bounded vertical viewport with wider fades and room for its scrollbar.",
    source: `import * as N from "@solid-gpui/core/components";
import { View } from "@solid-gpui/core";
export default function Example() {
  return <N.ScrollShadow axis="vertical" color="#0a0a0a" fadeSize={40} style={{ height: 180 }}><View style={{ paddingRight: 14 }}>{Array.from({ length: 12 }, (_, i) => <View style={{ padding: 12 }}><N.Label text={"Project " + (i + 1)} /></View>)}</View></N.ScrollShadow>;
}`,
  },
  {
    component: "ScrollShadow",
    id: "ScrollShadow--horizontal",
    title: "Horizontal",
    description: "A content-sized horizontal viewport with a visible scrollbar and dynamic edge fades.",
    source: `import * as N from "@solid-gpui/core/components";
import { View } from "@solid-gpui/core";
export default function Example() {
  return <N.ScrollShadow axis="horizontal" color="#0a0a0a" style={{ width: 280 }}><View style={{ width: 720, flexDirection: "row", paddingBottom: 14 }}>{Array.from({ length: 6 }, (_, i) => <View style={{ width: 120, flexShrink: 0, padding: 12 }}><N.Label text={\`Column \${i + 1}\`} /></View>)}</View></N.ScrollShadow>;
}`,
  },
  {
    component: "ScrollShadow",
    id: "ScrollShadow--virtual-list",
    title: "Virtualized list",
    description:
      "Compose a bounded ScrollShadow directly with a core VirtualList without eager row creation or a duplicate scrollbar.",
    source: `import { View, VirtualList } from "@solid-gpui/core";
import { ScrollShadow, Label } from "@solid-gpui/core/components";
const data = Array.from({ length: 10000 }, (_, index) => ({ id: index, label: \`Project \${index + 1}\` }));
export default function Example() {
  return <ScrollShadow axis="vertical" style={{ height: 240 }}><VirtualList data={data} itemKey={(item) => item.id} estimatedItemSize={32} style={{ widthPercent: 100, heightPercent: 100 }} renderItem={(item) => <View style={{ height: 32, justifyContent: "center", paddingLeft: 12, paddingRight: 14 }}><Label text={item.label} /></View>} /></ScrollShadow>;
}`,
  },
  example(
    "primary",
    "Primary",
    "Give the main action a clear visual priority.",
    '<N.Button label="Continue" variant="primary" />',
  ),
  example(
    "outline",
    "Outline",
    "Use a bordered button for an alternative action.",
    '<N.Button label="View details" outline />',
  ),
  example(
    "secondary",
    "Secondary",
    "A quieter filled button for supporting actions.",
    '<N.Button label="Save draft" variant="secondary" />',
  ),
  example(
    "ghost",
    "Ghost",
    "Keep toolbars and repeated actions visually light.",
    '<N.Button label="More options" variant="ghost" dropdownCaret />',
  ),
  example(
    "danger",
    "Destructive",
    "Make destructive actions easy to recognize.",
    '<N.Button label="Delete project" variant="danger" />',
  ),
  example(
    "link",
    "Link",
    "Use link styling for a low-emphasis action.",
    '<N.Button label="Learn more" variant="link" />',
  ),
  example(
    "text",
    "Text button",
    "A plain text action without a filled background.",
    '<N.Button label="Skip for now" variant="text" />',
  ),
  example(
    "colors",
    "Semantic colors",
    "Use status colors when an action has a specific meaning.",
    '<N.Button label="Information" variant="info" /><View style={{ flexDirection: "row", gap: 12 }}><N.Button label="Approve" variant="success" /><N.Button label="Review" variant="warning" /></View>',
  ),
  example(
    "sizes",
    "Sizes",
    "Choose from four sizes to match the density of your interface.",
    '<View style={{ flexDirection: "row", gap: 12, alignItems: "center" }}><N.Button label="Extra small" size="xsmall" /><N.Button label="Small" size="small" /></View><View style={{ flexDirection: "row", gap: 12, alignItems: "center" }}><N.Button label="Medium" size="medium" /><N.Button label="Large" size="large" /></View>',
  ),
  example(
    "icons",
    "Icon buttons",
    "Give icon-only actions an accessible label and a tooltip.",
    '<View style={{ flexDirection: "row", gap: 12 }}><N.Button accessibilityLabel="Add item" tooltip="Add item" outline><Icon name="lucide:plus" size={16} color="#fafafa" /></N.Button><N.Button accessibilityLabel="Open settings" tooltip="Open settings" variant="ghost"><Icon name="lucide:settings" size={16} color="#fafafa" /></N.Button></View>',
  ),
  example(
    "with-icon",
    "With icon",
    "Pair a familiar icon with a short action label.",
    '<N.Button><View style={{ flexDirection: "row", gap: 8, alignItems: "center" }}><Icon name="lucide:plus" size={16} color="#fafafa" /><N.Label text="New project" /></View></N.Button><N.Button><View style={{ flexDirection: "row", gap: 8, alignItems: "center" }}><N.Label text="Continue" /><Icon name="lucide:chevron-right" size={16} color="#fafafa" /></View></N.Button>',
  ),
  example(
    "rounded",
    "Rounded corners",
    "Adjust the corner radius to suit the surrounding interface.",
    '<View style={{ flexDirection: "row", gap: 12 }}><N.Button label="Square" rounded="none" /><N.Button label="Small" rounded="small" /></View><View style={{ flexDirection: "row", gap: 12 }}><N.Button label="Medium" rounded="medium" /><N.Button label="Large" rounded="large" /></View>',
  ),
  example(
    "loading",
    "Loading",
    "Show progress while an action is running and prevent repeated submissions.",
    '<N.Button label={pending() ? "Saving…" : saved() ? "Saved" : "Save changes"} variant="primary" loading={pending()} disabled={pending()} onPress={() => { setPending(true); setSaved(false); timer = setTimeout(() => { setPending(false); setSaved(true); }, 1200); }} />',
    "const [pending, setPending] = createSignal(false);\n  const [saved, setSaved] = createSignal(false);\n  let timer: ReturnType<typeof setTimeout> | undefined;\n  onCleanup(() => clearTimeout(timer));",
  ),
  example(
    "disabled",
    "Disabled",
    "Keep unavailable actions visible while preventing interaction.",
    '<View style={{ flexDirection: "row", gap: 12 }}><N.Button label="Continue" variant="primary" disabled /><N.Button label="Save draft" outline disabled /></View>',
  ),
  example(
    "selected",
    "Selected state",
    "Use a toggle for an action that stays enabled until pressed again.",
    '<N.Button label={pinned() ? "Pinned" : "Pin item"} toggled={pinned()} onPress={() => setPinned(!pinned())} />',
    "const [pinned, setPinned] = createSignal(false);",
  ),
  example(
    "group",
    "Button group",
    "Keep related choices together in a compact group.",
    '<N.ButtonGroup><N.Button label="Day" selected={period() === "Day"} onPress={() => setPeriod("Day")} /><N.Button label="Week" selected={period() === "Week"} onPress={() => setPeriod("Week")} /><N.Button label="Month" selected={period() === "Month"} onPress={() => setPeriod("Month")} /></N.ButtonGroup>',
    'const [period, setPeriod] = createSignal("Week");',
  ),
  ...controlsRecipes,
  ...presentationRecipes,
  ...compositionRecipes,
  ...dataRecipes,
  ...chartsRecipes,
  ...overlaysRecipes,
];
