import { recipes } from "./shared";

export const chartsRecipes = [
  ...recipes("LineChart", [
    {
      id: "styled",
      title: "Styled series",
      description: "Emphasize individual values while keeping axes and grid lines readable.",
      jsx: '<N.LineChart data={[{ label: "Mon", value: 24 }, { label: "Tue", value: 52 }, { label: "Wed", value: 37 }, { label: "Thu", value: 68 }, { label: "Fri", value: 46 }]} dot stroke="#10b981" style={{ height: 240 }} />',
      setup: "",
    },
    {
      id: "minimal",
      title: "Minimal chart",
      description: "Remove supporting grid lines when the overall trend is the focus.",
      jsx: '<N.LineChart data={[{ label: "Mon", value: 24 }, { label: "Tue", value: 52 }, { label: "Wed", value: 37 }, { label: "Thu", value: 68 }, { label: "Fri", value: 46 }]} grid={false} xAxis={false} style={{ height: 180 }} />',
      setup: "",
    },
  ]),
  ...recipes("BarChart", [
    {
      id: "styled",
      title: "Styled series",
      description: "Emphasize individual values while keeping axes and grid lines readable.",
      jsx: '<N.BarChart data={[{ label: "Mon", value: 24 }, { label: "Tue", value: 52 }, { label: "Wed", value: 37 }, { label: "Thu", value: 68 }, { label: "Fri", value: 46 }]} labels gradient={{ from: "#3b82f6", to: "#10b981" }} style={{ height: 240 }} />',
      setup: "",
    },
    {
      id: "minimal",
      title: "Minimal chart",
      description: "Remove supporting grid lines when the overall trend is the focus.",
      jsx: '<N.BarChart data={[{ label: "Mon", value: 24 }, { label: "Tue", value: 52 }, { label: "Wed", value: 37 }, { label: "Thu", value: 68 }, { label: "Fri", value: 46 }]} grid={false} labelAxis={false} valueAxis={false} style={{ height: 180 }} />',
      setup: "",
    },
  ]),
  ...recipes("AreaChart", [
    {
      id: "comparison",
      title: "Series comparison",
      description: "Compare two sources using consistent colors across categories.",
      jsx: '<N.AreaChart data={[{ label: "Mon", values: [24, 18] }, { label: "Tue", values: [52, 32] }, { label: "Wed", values: [37, 42] }, { label: "Thu", values: [68, 54] }]} series={[{ name: "Desktop", stroke: "#3b82f6" }, { name: "Mobile", stroke: "#10b981" }]} interactive style={{ height: 260 }} />',
      setup: "",
    },
    {
      id: "minimal",
      title: "Minimal chart",
      description: "Remove supporting grid lines when the overall trend is the focus.",
      jsx: '<N.AreaChart data={[{ label: "Mon", values: [24, 18] }, { label: "Tue", values: [52, 32] }, { label: "Wed", values: [37, 42] }, { label: "Thu", values: [68, 54] }]} series={[{ name: "Desktop", stroke: "#3b82f6" }, { name: "Mobile", stroke: "#10b981" }]} grid={false} xAxis={false} style={{ height: 220 }} />',
      setup: "",
    },
  ]),
  ...recipes("RadarChart", [
    {
      id: "comparison",
      title: "Series comparison",
      description: "Compare two sources using consistent colors across categories.",
      jsx: '<N.RadarChart data={[{ label: "Mon", values: [24, 18] }, { label: "Tue", values: [52, 32] }, { label: "Wed", values: [37, 42] }, { label: "Thu", values: [68, 54] }]} series={[{ name: "Desktop", stroke: "#3b82f6" }, { name: "Mobile", stroke: "#10b981" }]} dot maxValue={100} outerRadius={80} style={{ height: 260 }} />',
      setup: "",
    },
    {
      id: "minimal",
      title: "Minimal chart",
      description: "Remove supporting grid lines when the overall trend is the focus.",
      jsx: '<N.RadarChart data={[{ label: "Mon", values: [24, 18] }, { label: "Tue", values: [52, 32] }, { label: "Wed", values: [37, 42] }, { label: "Thu", values: [68, 54] }]} series={[{ name: "Desktop", stroke: "#3b82f6" }, { name: "Mobile", stroke: "#10b981" }]} grid={false} outerRadius={70} style={{ height: 220 }} />',
      setup: "",
    },
  ]),
  ...recipes("CandlestickChart", [
    {
      id: "bodies",
      title: "Candle proportions",
      description: "Adjust the body width to separate neighboring trading periods.",
      jsx: '<N.CandlestickChart data={[{ label: "Mon", open: 24, high: 36, low: 18, close: 32 }, { label: "Tue", open: 32, high: 40, low: 26, close: 28 }, { label: "Wed", open: 28, high: 42, low: 24, close: 38 }]} bodyWidthRatio={0.45} style={{ height: 220 }} />',
      setup: "",
    },
    {
      id: "minimal",
      title: "Minimal chart",
      description: "Remove supporting grid lines when the overall trend is the focus.",
      jsx: '<N.CandlestickChart data={[{ label: "Mon", open: 24, high: 36, low: 18, close: 32 }, { label: "Tue", open: 32, high: 40, low: 26, close: 28 }, { label: "Wed", open: 28, high: 42, low: 24, close: 38 }]} grid={false} xAxis={false} style={{ height: 180 }} />',
      setup: "",
    },
  ]),
  ...recipes("PieChart", [
    {
      id: "pie",
      title: "Pie distribution",
      description: "Show each category as a share of the whole.",
      jsx: '<N.PieChart data={[{ value: 52, label: "Desktop", color: "#3b82f6" }, { value: 32, label: "Mobile", color: "#10b981" }, { value: 16, label: "Tablet", color: "#f59e0b" }]} innerRadius={0} outerRadius={80} labels style={{ height: 220 }} />',
      setup: "",
    },
    {
      id: "donut",
      title: "Donut chart",
      description: "Use an inner radius and spacing to distinguish the segments.",
      jsx: '<N.PieChart data={[{ value: 52, label: "Desktop", color: "#3b82f6" }, { value: 32, label: "Mobile", color: "#10b981" }, { value: 16, label: "Tablet", color: "#f59e0b" }]} innerRadius={50} outerRadius={80} padAngle={0.04} style={{ height: 220 }} />',
      setup: "",
    },
  ]),
  ...recipes("SankeyChart", [
    {
      id: "flows",
      title: "Branching flows",
      description: "Split one source into several destinations and show their relative volumes.",
      jsx: '<N.SankeyChart nodes={[{ label: "Visits", color: "#3b82f6" }, { label: "Trial", color: "#10b981" }, { label: "Other", color: "#a1a1aa" }]} links={[{ source: 0, target: 1, value: 60 }, { source: 0, target: 2, value: 40 }]} valueLabels nodeWidth={14} style={{ height: 240 }} />',
      setup: "",
    },
    {
      id: "appearance",
      title: "Flow appearance",
      description: "Tune node spacing and link opacity for a quieter diagram.",
      jsx: '<N.SankeyChart nodes={[{ label: "Visits", color: "#3b82f6" }, { label: "Trial", color: "#10b981" }, { label: "Other", color: "#a1a1aa" }]} links={[{ source: 0, target: 1, value: 60 }, { source: 0, target: 2, value: 40 }]} nodePadding={24} nodeCornerRadius={4} linkOpacity={0.25} style={{ height: 240 }} />',
      setup: "",
    },
  ]),
  ...recipes("Plot", [
    {
      id: "series",
      title: "Custom line series",
      description: "Layer several typed line primitives in the same plotting area.",
      jsx: '<N.Plot primitives={[{ kind: "line", points: [{ x: 20, y: 130 }, { x: 100, y: 50 }, { x: 200, y: 90 }], stroke: { kind: "solid", color: "#3b82f6" }, strokeWidth: 3 }, { kind: "line", points: [{ x: 20, y: 150 }, { x: 100, y: 100 }, { x: 200, y: 40 }], stroke: { kind: "solid", color: "#10b981" }, strokeWidth: 2 }]} style={{ height: 180 }} />',
      setup: "",
    },
    {
      id: "stroke",
      title: "Stroke weight",
      description: "Use stroke width to distinguish a highlighted series.",
      jsx: '<N.Plot primitives={[{ kind: "line", points: [{ x: 20, y: 120 }, { x: 100, y: 60 }, { x: 200, y: 40 }], stroke: { kind: "solid", color: "#10b981" }, strokeWidth: 5 }]} style={{ height: 160 }} />',
      setup: "",
    },
  ]),
  ...recipes("PlotDot", [
    {
      id: "size-4",
      title: "Small plot marker",
      description: "Control the size and color of a point without changing its coordinates.",
      jsx: '<N.PlotDot point={{ x: 100, y: 60 }} size={4} fill="#10b981" stroke="#fafafa" style={{ height: 140 }} />',
      setup: "",
    },
    {
      id: "size-10",
      title: "Highlighted plot marker",
      description: "Control the size and color of a point without changing its coordinates.",
      jsx: '<N.PlotDot point={{ x: 100, y: 60 }} size={10} fill="#10b981" stroke="#fafafa" style={{ height: 140 }} />',
      setup: "",
    },
  ]),
  ...recipes("PlotCrossLine", [
    {
      id: "vertical",
      title: "Vertical crosshair",
      description: "Limit the crosshair to the axis relevant to the comparison.",
      jsx: '<N.PlotCrossLine point={{ x: 100, y: 60 }} direction="vertical" style={{ height: 140 }} />',
      setup: "",
    },
    {
      id: "horizontal",
      title: "Horizontal crosshair",
      description: "Limit the crosshair to the axis relevant to the comparison.",
      jsx: '<N.PlotCrossLine point={{ x: 100, y: 60 }} direction="horizontal" style={{ height: 140 }} />',
      setup: "",
    },
  ]),
  ...recipes("PlotTooltip", [
    {
      id: "series",
      title: "Multi-series tooltip",
      description: "Group values from several series at the same position.",
      jsx: '<View style={{ height: 180 }}><N.PlotTooltip cursor={{ x: 40, y: 40 }} within={{ width: 280, height: 180 }} title="Monday" rows={[{ color: "#3b82f6", label: "Desktop", value: "52" }, { color: "#10b981", label: "Mobile", value: "32" }]} style={{ height: 180 }} /></View>',
      setup: "",
    },
    {
      id: "plain",
      title: "Plain tooltip",
      description: "Show a compact value label without a surrounding panel.",
      jsx: '<View style={{ height: 180 }}><N.PlotTooltip cursor={{ x: 40, y: 40 }} within={{ width: 280, height: 140 }} appearance={false} rows={[{ color: "#10b981", label: "Visits", value: "128" }]} style={{ height: 140 }} /></View>',
      setup: "",
    },
  ]),
];
