import { Text, View } from "@solid-gpui/core";
import * as N from "@solid-gpui/core/components";
import { createSignal } from "@solid-gpui/core/runtime";
import { Card, DenseRow, SectionHeader } from "../components/ui";
import { useGallery } from "../context";
import type { SolidChild } from "../types";

const choices: N.ChoiceGroup[] = [
  {
    key: "languages",
    label: "Language",
    items: [
      { key: "rust", label: "Rust", description: "Native application" },
      { key: "typescript", label: "TypeScript", description: "Reactive interface" },
      { key: "chinese", label: "Chinese (中文)", description: "Unicode text" },
    ],
  },
];

function EventLine(props: { value: string }): SolidChild {
  const { theme } = useGallery();
  return <Text style={{ color: theme().textSecondary, fontSize: 12, minWidth: 0 }}>{props.value}</Text>;
}

export function NativeControlsShowcase(): SolidChild {
  // Keep non-ASCII sample text to exercise native editing and font fallback.
  const [text, setText] = createSignal("Edit me · 中文");
  const [checked, setChecked] = createSignal(true);
  const [selected, setSelected] = createSignal<string | null>("rust");
  const [slider, setSlider] = createSignal<N.SliderValue>({ kind: "single", value: 35 });
  const [page, setPage] = createSignal(1);
  const [event, setEvent] = createSignal("Ready");
  return (
    <View style={{ gap: 20, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Native Controls"
        tag="gpui-component"
        description="Editable inputs, native pickers and virtualized data views."
      />
      <Card title="Inputs and choices">
        <N.Input
          value={text()}
          onChange={(e) => setText(e.value)}
          placeholder="Name"
          ariaLabel="Native name"
          style={{ width: 300 }}
        />
        <EventLine value={`Input: ${text()}`} />
        <N.Textarea
          defaultValue={"A native multiline editor.\nTry selection, undo and Chinese input."}
          rows={3}
          style={{ height: 90 }}
        />
        <DenseRow gap={12}>
          <N.NumberInput defaultValue="12" min={0} max={100} step={2} style={{ width: 140 }} />
          <N.Select
            items={choices}
            value={selected()}
            onChange={(e) => {
              setSelected(e.value ?? null);
              setEvent(`Selected: ${e.value}`);
            }}
            style={{ width: 210 }}
          />
          <N.Combobox items={choices} multiple placeholder="Multiple languages" style={{ width: 210 }} />
        </DenseRow>
        <DenseRow gap={16}>
          <N.Switch checked={checked()} onChange={setChecked} label="Enabled" />
          <N.Checkbox checked={checked()} onChange={setChecked} label="Same state" />
          <N.Rating value={3} max={5} onChange={(v) => setEvent(`Rating: ${v}`)} />
        </DenseRow>
        <N.Slider
          value={slider()}
          onChange={(e) => {
            setSlider(e.value);
            setEvent(`Slider: ${JSON.stringify(e.value)}`);
          }}
          style={{ width: 320 }}
        />
        <DenseRow gap={12}>
          <N.DatePicker
            defaultValue={{ kind: "single", date: "2026-09-06" }}
            onChange={(e) => setEvent(`Date: ${JSON.stringify(e.value)}`)}
            style={{ width: 230 }}
          />
          <N.ColorPicker defaultValue="#3b82f6" label="Accent" onChange={(e) => setEvent(`Color: ${e.value}`)} />
        </DenseRow>
        <N.OtpInput length={6} onComplete={(value) => setEvent(`OTP: ${value}`)} />
        <N.Pagination currentPage={page()} totalPages={4294967295} onChange={setPage} />
        <EventLine value={`Page: ${page()} · ${event()}`} />
      </Card>
      <Card
        title="Virtualized list and table"
        description="Filter, select rows, sort columns and resize their native headers."
      >
        <N.List
          sections={[{ key: "language", header: "Languages", items: choices[0]!.items }]}
          onSelect={(e) => setEvent(`List: ${e.value}`)}
          style={{ height: 210 }}
        />
        <N.DataTable
          columns={[
            { key: "name", label: "Project", sortable: true, width: 240 },
            { key: "status", label: "Status", width: 170 },
            { key: "progress", label: "Progress", width: 170 },
          ]}
          rows={Array.from({ length: 100 }, (_, i) => ({
            key: `project-${i}`,
            cells: {
              name: `Native project ${i + 1}`,
              status: { badge: i % 2 ? "Active" : "Ready" },
              progress: { progress: i },
            },
          }))}
          onSelectionChange={(e) => setEvent(`Table: ${JSON.stringify(e.value)}`)}
          stripe
          bordered
          style={{ height: 280 }}
        />
        <N.Tree
          nodes={[
            {
              key: "root",
              label: "Workspace",
              expanded: true,
              children: [
                { key: "src", label: "src" },
                { key: "docs", label: "docs" },
              ],
            },
          ]}
          onSelect={(e) => setEvent(`Tree: ${e.value}`)}
          style={{ height: 140 }}
        />
        <EventLine value={event()} />
      </Card>
    </View>
  );
}

export function NativeOverlaysShowcase(): SolidChild {
  const [dialog, setDialog] = createSignal(false);
  const [sheet, setSheet] = createSignal(false);
  const [notification, setNotification] = createSignal(false);
  const [event, setEvent] = createSignal("Open a menu or overlay");
  const menu: N.MenuSpec = {
    items: [
      { kind: "item", id: "copy", label: "Copy selection" },
      { kind: "item", id: "checked", label: "Show details", checked: true },
      { kind: "separator" },
      {
        kind: "submenu",
        id: "more",
        label: "More",
        menu: { items: [{ kind: "item", id: "inspect", label: "Inspect" }] },
      },
    ],
  };
  return (
    <View style={{ gap: 20, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Native Menus & Overlays"
        tag="gpui-component"
        description="Native focus, keyboard navigation, dismissal and content mounted from Solid."
      />
      <Card title="Menus">
        <DenseRow gap={12}>
          <N.DropdownMenu label="Dropdown menu" menu={menu} onSelect={(e) => setEvent(`Menu: ${e.id}`)} />
          <N.ContextMenu
            menu={menu}
            slots={{ trigger: <N.Button label="Right-click here" /> }}
            onSelect={(e) => setEvent(`Context: ${e.id}`)}
          />
          <N.NativeMenu
            trigger="press"
            items={[
              { kind: "item", id: "native-copy", label: "Native Copy" },
              { kind: "item", id: "native-check", label: "Native checked item", checked: true },
            ]}
            onSelect={(e) => setEvent(`OS menu: ${e.id}`)}
          >
            <N.Button label="OS menu" />
          </N.NativeMenu>
          <N.Popover slots={{ trigger: <N.Button label="Popover" /> }}>
            <View style={{ padding: 16, gap: 12 }}>
              <N.Label text="Native popover content" />
              <N.Input placeholder="Focus inside popover" style={{ width: 230 }} />
            </View>
          </N.Popover>
        </DenseRow>
        <EventLine value={event()} />
      </Card>
      <Card title="Dialog, sheet and notification">
        <DenseRow gap={12}>
          <N.Button label="Open dialog" variant="primary" onPress={() => setDialog(true)} />
          <N.Button label="Open sheet" onPress={() => setSheet(true)} />
          <N.Button label="Notify" onPress={() => setNotification(true)} />
        </DenseRow>
        <N.Dialog
          open={dialog()}
          title="Native dialog"
          onOpenChange={(e) => {
            setDialog(e.open);
            setEvent(`Dialog: ${e.reason}`);
          }}
          onAction={(e) => setEvent(`Dialog action: ${e.kind}`)}
        >
          <View style={{ padding: 8, gap: 12 }}>
            <N.Label text="Edit text, press Escape, or choose OK." />
            <N.Input defaultValue="Dialog input" style={{ width: 300 }} />
          </View>
        </N.Dialog>
        <N.Sheet
          open={sheet()}
          title="Native sheet"
          placement="right"
          size={{ unit: "px", value: 380 }}
          onOpenChange={(e) => {
            setSheet(e.open);
            setEvent(`Sheet: ${e.reason}`);
          }}
        >
          <View style={{ padding: 16, gap: 12 }}>
            <N.Label text="Resizable native sheet" />
            <N.Input placeholder="Sheet input" />
            <N.Button label="Close sheet" onPress={() => setSheet(false)} />
          </View>
        </N.Sheet>
        <N.Notification
          open={notification()}
          title="Native notification"
          message="Delivered inside the application window."
          kind="success"
          delivery="inApp"
          onClose={() => setNotification(false)}
        />
        <EventLine value={event()} />
      </Card>
    </View>
  );
}

export function NativeSettingsShowcase(): SolidChild {
  const [name, setName] = createSignal("Solid GPUI");
  const [enabled, setEnabled] = createSignal(true);
  const [selection, setSelection] = createSignal<N.SettingsSelection>({ page: "general" });
  const [event, setEvent] = createSignal("Search preferences using the native sidebar");
  return (
    <View style={{ gap: 20, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Native Settings"
        tag="gpui-component"
        description="Search preferences, edit your profile and reset changed settings."
      />
      <Card title="Application preferences">
        <N.Settings
          selection={selection()}
          onChange={(e) => setSelection(e.selection)}
          onSearchChange={(q) => setEvent(`Search: ${q}`)}
          style={{ height: 470 }}
        >
          <N.SettingPage name="general" title="General">
            <N.SettingGroup name="profile" title="Profile">
              <N.SettingItem
                title="Display name"
                description="The name shown in your workspace."
                keywords={["name", "profile"]}
              >
                <N.SettingField dirty={name() !== "Solid GPUI"} onReset={() => setName("Solid GPUI")}>
                  <N.Input value={name()} onChange={(e) => setName(e.value)} style={{ width: 230 }} />
                </N.SettingField>
              </N.SettingItem>
              <N.SettingItem title="Notifications" description="Allow activity notifications." keywords={["alerts"]}>
                <N.SettingField dirty={!enabled()} onReset={() => setEnabled(true)}>
                  <N.Switch checked={enabled()} onChange={setEnabled} />
                </N.SettingField>
              </N.SettingItem>
            </N.SettingGroup>
          </N.SettingPage>
          <N.SettingPage name="editor" title="Editor">
            <N.SettingGroup name="text" title="Text editing">
              <N.SettingItem title="Language" description="Choose your default language.">
                <N.SettingField>
                  <N.Select items={choices} defaultValue="rust" style={{ width: 220 }} />
                </N.SettingField>
              </N.SettingItem>
              <N.SettingItem title="Tab width">
                <N.SettingField>
                  <N.NumberInput defaultValue="4" min={1} max={16} style={{ width: 130 }} />
                </N.SettingField>
              </N.SettingItem>
            </N.SettingGroup>
          </N.SettingPage>
        </N.Settings>
        <EventLine value={`${selection().page} · Name: ${name()} · Notifications: ${enabled()} · ${event()}`} />
      </Card>
    </View>
  );
}

export function NativeDockShowcase(): SolidChild {
  let dock: N.DockAreaRef | undefined;
  const [saved, setSaved] = createSignal<N.DockSnapshot>();
  const [event, setEvent] = createSignal("Drag tabs, resize dividers or move the floating tile");
  const [text, setText] = createSignal("This input survives tab moves and layout restore.");
  const { showStatus } = useGallery();
  const run = async (work: () => Promise<unknown>) => {
    try {
      await work();
    } catch (error) {
      showStatus(String(error), "warning");
    }
  };
  return (
    <View style={{ gap: 20, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Native Dock & Tiles"
        tag="gpui-component"
        description="Rearrange tabs, resize panes and save or restore your workspace layout."
      />
      <Card title="Workspace layout">
        <DenseRow gap={10}>
          <N.Button
            label="Save layout"
            onPress={() =>
              run(async () => {
                if (dock) {
                  setSaved(await dock.dump());
                  setEvent("Layout saved");
                }
              })
            }
          />
          <N.Button
            label="Restore layout"
            disabled={!saved()}
            onPress={() =>
              run(async () => {
                const value = saved();
                if (dock && value) {
                  await dock.load(value);
                  setEvent("Layout restored");
                }
              })
            }
          />
          <N.Button
            label="Reset layout"
            onPress={() =>
              run(async () => {
                await dock?.replaceLayout(dockLayout);
              })
            }
          />
          <N.Button
            label="Zoom editor"
            onPress={() =>
              run(async () => {
                await dock?.zoomPane({ pane: "editor" });
              })
            }
          />
          <N.Button
            label="Zoom out"
            onPress={() =>
              run(async () => {
                await dock?.zoomOut();
              })
            }
          />
          <N.Button
            label="Undo tile"
            onPress={() =>
              run(async () => {
                await dock?.undoTiles({ pane: "preview" });
              })
            }
          />
        </DenseRow>
        <N.DockArea
          ref={(value) => {
            dock = value;
          }}
          panes={[
            { name: "editor", title: "Editor", contentSlot: 0 },
            { name: "notes", title: "Notes", contentSlot: 1 },
            { name: "preview", title: "Preview", contentSlot: 2 },
          ]}
          initialLayout={dockLayout}
          onPaneEvent={(e) => setEvent(`Pane: ${JSON.stringify(e)}`)}
          onLayoutChange={(e) => setEvent(`Layout revision: ${e.revision}`)}
          style={{ height: 430, minWidth: 0 }}
        >
          <View style={{ padding: 16, gap: 12 }}>
            <N.Label text="Reactive editor pane" />
            <N.Input value={text()} onChange={(e) => setText(e.value)} />
            <N.Label text={text()} />
          </View>
          <View style={{ padding: 16 }}>
            <N.TextView
              text={
                "## Native dock\n\n- Drag a tab into another group.\n- Close a tab and restore the saved layout.\n- Resize the floating preview."
              }
            />
          </View>
          <View style={{ padding: 16, gap: 10 }}>
            <N.Label text="Floating tile" />
            <N.Progress value={65} />
            <N.Label text="Move and resize this tile." />
          </View>
        </N.DockArea>
        <EventLine value={event()} />
      </Card>
    </View>
  );
}

const dockLayout: N.DockLayoutSpec = {
  center: {
    kind: "split",
    orientation: "horizontal",
    children: [
      { layout: { kind: "tabs", panes: ["editor", "notes"] }, size: 400 },
      {
        layout: { kind: "tiles", panes: [{ pane: "preview", bounds: { x: 20, y: 20, width: 240, height: 230 } }] },
        size: 320,
      },
    ],
  },
};

export function NativeChartsShowcase(): SolidChild {
  const native = N.useNative();
  const [math, setMath] = createSignal("Calculate native scales and geometry");
  const { theme } = useGallery();
  const points = [
    { label: "Mon", value: 24 },
    { label: "Tue", value: 52 },
    { label: "Wed", value: 37 },
    { label: "Thu", value: 76 },
    { label: "Fri", value: 61 },
  ];
  const series = [
    { name: "Desktop", stroke: "#3b82f6", fill: { kind: "solid" as const, color: "#3b82f633" } },
    { name: "Mobile", stroke: "#10b981", fill: { kind: "solid" as const, color: "#10b98133" } },
  ];
  return (
    <View style={{ gap: 20, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Native Charts & Plot"
        tag="gpui-component"
        description="Seven native charts, composed plot primitives, scales and geometry functions."
      />
      <Card title="Line">
        <N.LineChart data={points} stroke="#3b82f6" dot style={{ height: 230 }} />
      </Card>
      <Card title="Area">
        <N.AreaChart
          data={points.map((p) => ({ label: p.label, values: [p.value, p.value / 2] }))}
          series={series}
          style={{ height: 230 }}
        />
      </Card>
      <Card title="Bar">
        <N.BarChart data={points} labels gradient={{ from: "#3b82f6", to: "#10b981" }} style={{ height: 230 }} />
      </Card>
      <Card title="Candlestick">
        <N.CandlestickChart
          data={points.map((p) => ({
            label: p.label,
            open: p.value,
            high: p.value + 12,
            low: p.value - 10,
            close: p.value + (p.value % 2 ? -5 : 8),
          }))}
          style={{ height: 230 }}
        />
      </Card>
      <Card title="Pie">
        <N.PieChart
          data={[
            { value: 48, label: "Desktop", color: "#3b82f6" },
            { value: 32, label: "Mobile", color: "#10b981" },
            { value: 20, label: "Other", color: "#f59e0b" },
          ]}
          innerRadius={45}
          labels
          style={{ height: 260 }}
        />
      </Card>
      <Card title="Radar">
        <N.RadarChart
          data={points.map((p) => ({ label: p.label, values: [p.value, 90 - p.value] }))}
          series={series}
          maxValue={100}
          style={{ height: 300 }}
        />
      </Card>
      <Card title="Sankey">
        <N.SankeyChart
          nodes={[
            { label: "Source", color: "#3b82f6" },
            { label: "Desktop", color: "#10b981" },
            { label: "Mobile", color: "#f59e0b" },
          ]}
          links={[
            { source: 0, target: 1, value: 6 },
            { source: 0, target: 2, value: 4 },
          ]}
          style={{ height: 240 }}
        />
      </Card>
      <Card title="Plot primitives and geometry">
        <N.Plot
          primitives={[
            { kind: "grid", x: [40, 120, 200, 280], y: [40, 100, 160], stroke: theme().border, dash: [4, 4] },
            { kind: "axis", x: 40, y: 180, stroke: theme().textSecondary },
            {
              kind: "line",
              points: [
                { x: 40, y: 140 },
                { x: 120, y: 60 },
                { x: 200, y: 100 },
                { x: 280, y: 40 },
              ],
              stroke: { kind: "solid", color: "#3b82f6" },
              strokeWidth: 3,
              dots: { fill: "#3b82f6", size: 6 },
            },
          ]}
          style={{ height: 210 }}
        />
        <N.Button
          label="Calculate scale"
          onPress={async () => {
            const result = await native.scaleBand({
              domain: ["A", "B", "C"],
              range: [40, 340],
              values: ["B"],
              cursor: 180,
            });
            setMath(JSON.stringify(result));
          }}
        />
        <EventLine value={math()} />
      </Card>
    </View>
  );
}
