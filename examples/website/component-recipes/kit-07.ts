import type { ComponentVariant } from "../component-variants.ts";

export const kit07Recipes: ComponentVariant[] = [
  {
    component: "Select",
    id: "Select--dismiss",
    title: "Menu dismissal",
    description: "Observe native dismissal, including confirmation, separately from value changes.",
    source: `import * as N from "@solid-gpui/core/components";
import { View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
export default function Example() {
  const [status, setStatus] = createSignal("Open the menu, then select or dismiss.");
  return <View style={{ gap: 12 }}><N.Select items={[{ key: "numbers", items: [{ key: "one", label: "One" }, { key: "two", label: "Two" }] }]} onDismiss={() => setStatus("Menu dismissed")} /><N.Label text={status()} /></View>;
}`,
  },
  {
    component: "DockArea",
    id: "DockArea--close-buttons",
    title: "Tab close controls",
    description: "Opt into native close buttons for closable tabs; removing a tab preserves neighboring panes.",
    source: `import * as N from "@solid-gpui/core/components";
export default function Example() {
  return <N.DockArea closeButtonVisible panelStyle="tabBar" panes={[{ name: "editor", title: "Editor", contentSlot: 0 }, { name: "preview", title: "Preview", contentSlot: 1 }]} initialLayout={{ center: { kind: "tabs", panes: ["editor", "preview"] } }} style={{ height: 260 }}><N.Label text="Editor content" /><N.Label text="Preview content" /></N.DockArea>;
}`,
  },
  {
    component: "BarChart",
    id: "BarChart--reserved-bands",
    title: "Reserved bands",
    description: "Keep bar widths stable while data loads and show a minimum-length stub for zero values.",
    source: `import { BarChart } from "@solid-gpui/core/components";
export default function Example() {
  return <BarChart data={[{ label: "Mon", value: 12 }, { label: "Tue", value: 0 }, { label: "Wed", value: 24 }]} bandCount={5} bandTickCount={5} paddingInner={0.25} paddingOuter={0.15} maxBandWidth={48} minLength={2} valueAxis gridDashed={false} style={{ height: 240 }} />;
}`,
  },
  {
    component: "DatePicker",
    id: "DatePicker--time",
    title: "Date and time",
    description: "Edit a single date and its time in one native popup; time values have no timezone.",
    source: `import * as N from "@solid-gpui/core/components";
import { View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
export default function Example() {
  const [value, setValue] = createSignal<N.DateValue>({ kind: "single", date: "2026-09-28" });
  const [time, setTime] = createSignal("09:30:00");
  return <View style={{ gap: 12 }}><N.DatePicker value={value()} time={time()} timePrecision="minute" hourCycle="h12" format="%Y/%m/%d %I:%M %p" onChange={event => { setValue(event.value); if (event.time) setTime(event.time); }} /><N.Label text={time()} /></View>;
}`,
  },
  {
    component: "LineChart",
    id: "LineChart--pinned-axis",
    title: "Pinned axis and reference line",
    description: "Keep an intraday scale stable as points arrive, with labeled ticks and a previous-close reference.",
    source: `import { LineChart } from "@solid-gpui/core/components";
export default function Example() {
  return <LineChart data={[{ label: "09:30", value: 101 }, { label: "10:00", value: 103 }, { label: "10:30", value: 102 }]} yDomain={[98, 106]} pointCount={6} yAxis yTickCount={5} xTickCount={6} gridColumns={6} gridDashed referenceLines={[100]} style={{ height: 240 }} />;
}`,
  },
  {
    component: "AreaChart",
    id: "AreaChart--value-axis",
    title: "Value axis",
    description: "Label the native value axis and keep a fixed range across series updates.",
    source: `import { AreaChart } from "@solid-gpui/core/components";
export default function Example() {
  return <AreaChart data={[{ label: "Mon", values: [20, 30] }, { label: "Tue", values: [35, 25] }, { label: "Wed", values: [25, 40] }]} series={[{ name: "Desktop" }, { name: "Web" }]} yDomain={[0, 50]} yAxis referenceLines={[30]} gridDashed={false} style={{ height: 240 }} />;
}`,
  },
  {
    component: "Attachment",
    id: "Attachment--retry",
    title: "Retry and remove",
    description: "Native attachment controls expose failed-upload retry, removal and bounded progress.",
    source: `import * as N from "@solid-gpui/core/components";
import { View } from "@solid-gpui/core";
import { createSignal, Show } from "@solid-gpui/core/runtime";
export default function Example() {
  const [visible, setVisible] = createSignal(true);
  const [status, setStatus] = createSignal<N.AttachmentStatus>("failed");
  return <View style={{ gap: 12 }}><Show when={visible()} fallback={<N.Label text="Attachment removed" />}><N.Attachment status={status()} progress={status() === "uploading" ? 45 : 0} onRetry={() => setStatus("uploading")} onRemove={() => setVisible(false)} slots={{ content: <N.AttachmentContent slots={{ title: <N.AttachmentTitle text="Report.pdf" />, description: <N.AttachmentDescription text={status()} /> }} /> }} /></Show></View>;
}`,
  },
  {
    component: "Popover",
    id: "Popover--arrow",
    title: "Anchored arrow",
    description: "Keep the native arrow aligned to its trigger while offset and collision handling position the popup.",
    source: `import * as N from "@solid-gpui/core/components";
export default function Example() {
  return <N.Popover arrow offset={8} slots={{ trigger: <N.Button label="Details" /> }}><N.Label text="The arrow follows the trigger." /></N.Popover>;
}`,
  },
  {
    component: "TextView",
    id: "TextView--range-highlights",
    title: "Rendered-text highlights",
    description: "Search rendered text, then highlight and reveal a UTF-8 range using its snapshot revision.",
    source: `import * as N from "@solid-gpui/core/components";
import { View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
export default function Example() {
  let text: N.TextViewRef | undefined;
  const [status, setStatus] = createSignal("Highlight the word native.");
  const highlight = async () => {
    if (!text) return;
    try {
      const snapshot = await text.getRenderedText();
      const index = snapshot.text.indexOf("native");
      if (index < 0) { setStatus("No match yet; try after parsing completes."); return; }
      const startByte = new TextEncoder().encode(snapshot.text.slice(0, index)).length;
      const range = { startByte, endByte: startByte + 6 };
      await text.setRangeHighlights({ revision: snapshot.revision, highlights: [{ range, background: "#eab30866" }] });
      await text.revealRange({ revision: snapshot.revision, range });
      setStatus("Highlighted one match");
    } catch (error) { setStatus(String(error)); }
  };
  return <View style={{ gap: 12 }}><N.TextView ref={value => text = value} text="Search **native** rendered text, without counting Markdown markers." selectable scrollable style={{ height: 100 }} /><N.Button label="Highlight match" onPress={() => void highlight()} /><N.Label text={status()} /></View>;
}`,
  },
  {
    component: "SettingGroup",
    id: "SettingGroup--footer",
    title: "Group footer",
    description: "Place help outside the group surface and override one group's appearance.",
    source: `import * as N from "@solid-gpui/core/components";
export default function Example() {
  return <N.Settings><N.SettingPage name="general" title="General"><N.SettingGroup name="profile" title="Profile" variant="outline" slots={{ footer: <N.Label text="Changes apply to this device." /> }}><N.SettingItem title="Display name"><N.SettingField><N.Input defaultValue="Alex" /></N.SettingField></N.SettingItem></N.SettingGroup></N.SettingPage></N.Settings>;
}`,
  },
  {
    component: "Marker",
    id: "Marker--alignment",
    title: "Centered marker",
    description: "Center a transcript marker without changing its content or status semantics.",
    source: `import * as N from "@solid-gpui/core/components";
export default function Example() {
  return <N.Marker alignment="center" variant="separator" slots={{ content: <N.MarkerContent text="Today" /> }} />;
}`,
  },
  {
    component: "Textarea",
    id: "Textarea--sizes",
    title: "Control sizes",
    description: "Use the shared native control size scale for multiline editing.",
    source: `import { Textarea } from "@solid-gpui/core/components";
import { View } from "@solid-gpui/core";
export default function Example() {
  return <View style={{ gap: 12 }}><Textarea size="small" rows={2} placeholder="Compact notes" /><Textarea size="large" rows={2} placeholder="Comfortable notes" /></View>;
}`,
  },
];
