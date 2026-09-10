import { componentDescriptionsChinese } from "./component-examples.zh-CN";
type Example = { names: string[]; description: string; descriptionChinese: string; source: string };
export const componentExamples: Example[] = [];
function add(names: string, description: string, descriptionChinese: string, jsx: string, setup = "") {
  componentExamples.push({
    names: names.split(" "),
    description,
    descriptionChinese,
    source: `import * as N from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\n${setup.includes("createSignal") ? 'import { createSignal } from "@solid-gpui/core/runtime";\n' : ""}\nexport default function Example() {\n${setup ? `  ${setup}\n` : ""}  return (\n    ${jsx}\n  );\n}`,
  });
}
add(
  "Button",
  "A button for actions, with variants, loading and disabled states.",
  componentDescriptionsChinese["A button for actions, with variants, loading and disabled states."],
  '<View style={{ alignItems: "center" }}><N.Button label={saved() ? "Saved" : "Save changes"} variant="primary" onPress={() => setSaved(true)} /></View>',
  "const [saved, setSaved] = createSignal(false);",
);
add(
  "ButtonGroup",
  "Group related actions in a single control.",
  componentDescriptionsChinese["Group related actions in a single control."],
  '<N.ButtonGroup><N.Button label="Day" /><N.Button label="Week" /><N.Button label="Month" /></N.ButtonGroup>',
);
add(
  "Accordion AccordionItem",
  "Reveal related sections without leaving the page.",
  componentDescriptionsChinese["Reveal related sections without leaving the page."],
  '<N.Accordion bordered><N.AccordionItem open={open()} onChange={setOpen} slots={{ title: <N.Label text="Shipping" /> }}><N.Label text="Orders ship within two business days." /></N.AccordionItem></N.Accordion>',
  "const [open, setOpen] = createSignal(true);",
);
add(
  "Alert",
  "Display a contextual message and its severity.",
  componentDescriptionsChinese["Display a contextual message and its severity."],
  '<N.Alert title="Changes saved" message="Your preferences are up to date." variant="success" />',
);
add(
  "Avatar AvatarGroup",
  "Represent people with photos, names or initials.",
  componentDescriptionsChinese["Represent people with photos, names or initials."],
  '<N.AvatarGroup limit={3}><N.Avatar name="Alex Chen" /><N.Avatar name="Sam Rivera" /><N.Avatar name="Taylor Kim" /></N.AvatarGroup>',
);
add(
  "Badge",
  "Attach a count or status indicator to another element.",
  componentDescriptionsChinese["Attach a count or status indicator to another element."],
  '<N.Badge count={3}><N.Button label="Inbox" /></N.Badge>',
);
add(
  "Breadcrumb BreadcrumbItem",
  "Show the current location within a hierarchy.",
  componentDescriptionsChinese["Show the current location within a hierarchy."],
  '<N.Breadcrumb><N.BreadcrumbItem label="Home" /><N.BreadcrumbItem label="Projects" /><N.BreadcrumbItem label="Website" /></N.Breadcrumb>',
);
add(
  "Checkbox Switch Radio Toggle",
  "Connect a boolean control to Solid state.",
  componentDescriptionsChinese["Connect a boolean control to Solid state."],
  '<N.Switch label="Email notifications" checked={enabled()} onChange={setEnabled} />',
  "const [enabled, setEnabled] = createSignal(true);",
);
// Each boolean control has the same change callback but its own rendering.
for (const name of ["Checkbox", "Radio", "Toggle"])
  componentExamples.push({
    ...componentExamples.find((e) => e.names.includes("Switch"))!,
    names: [name],
    source: componentExamples.find((e) => e.names.includes("Switch"))!.source.replace(/N\.Switch/g, `N.${name}`),
  });
componentExamples.find((e) => e.names.includes("Switch"))!.names = ["Switch"];
add(
  "RadioGroup",
  "Choose one option from a group.",
  componentDescriptionsChinese["Choose one option from a group."],
  '<N.RadioGroup selectedIndex={selected()} onChange={setSelected}><N.Radio label="Personal" /><N.Radio label="Team" /></N.RadioGroup>',
  "const [selected, setSelected] = createSignal<number | null>(0);",
);
add(
  "ToggleGroup",
  "Group toggle buttons for related preferences.",
  componentDescriptionsChinese["Group toggle buttons for related preferences."],
  '<N.ToggleGroup segmented><N.Toggle label="Bold" /><N.Toggle label="Italic" /></N.ToggleGroup>',
);
add(
  "Clipboard",
  "Copy a value to the system clipboard.",
  componentDescriptionsChinese["Copy a value to the system clipboard."],
  '<View style={{ flexDirection: "row", alignItems: "center", gap: 12 }}><N.Text value="bun run website:native" /><N.Clipboard value="bun run website:native" /></View>',
);
add(
  "Collapsible",
  "Show or hide a section of content.",
  componentDescriptionsChinese["Show or hide a section of content."],
  '<N.Collapsible open={open()} slots={{ trigger: <N.Button label={open() ? "Hide details" : "Show details"} outline onPress={() => setOpen(!open())} /> }}><N.Label text="Additional project details" /></N.Collapsible>',
  "const [open, setOpen] = createSignal(false);",
);
add(
  "Input Textarea Editor",
  "Edit text using native selection, keyboard shortcuts and input methods.",
  componentDescriptionsChinese["Edit text using native selection, keyboard shortcuts and input methods."],
  '<N.Input value={value()} onChange={change => setValue(change.value)} placeholder="Your name" />',
  'const [value, setValue] = createSignal("");',
);
for (const name of ["Textarea", "Editor"])
  componentExamples.push({
    ...componentExamples.find((e) => e.names.includes("Input"))!,
    names: [name],
    source: componentExamples
      .find((e) => e.names.includes("Input"))!
      .source.replace(/N\.Input/g, `N.${name}`)
      .replace(
        'placeholder="Your name"',
        name === "Editor"
          ? 'placeholder="Write your document…" language="markdown" softWrap style={{ height: 180 }}'
          : 'placeholder="Write a message…" rows={3}',
      ),
  });
componentExamples.find((e) => e.names.includes("Input"))!.names = ["Input"];
add(
  "NumberInput",
  "Edit a number within a range.",
  componentDescriptionsChinese["Edit a number within a range."],
  '<N.NumberInput defaultValue="12" min={0} max={100} step={1} />',
);
add(
  "OtpInput",
  "Enter a short verification code.",
  componentDescriptionsChinese["Enter a short verification code."],
  "<N.OtpInput length={6} onComplete={code => console.log(code)} />",
);
add(
  "Slider",
  "Select a value along a continuous range.",
  componentDescriptionsChinese["Select a value along a continuous range."],
  '<N.Slider defaultValue={{ kind: "single", value: 35 }} min={0} max={100} />',
);
add(
  "Rating",
  "Display or change a rating.",
  componentDescriptionsChinese["Display or change a rating."],
  "<N.Rating value={rating()} max={5} onChange={setRating} />",
  "const [rating, setRating] = createSignal(3);",
);
add(
  "Pagination",
  "Navigate between pages of results.",
  componentDescriptionsChinese["Navigate between pages of results."],
  "<N.Pagination currentPage={page()} totalPages={12} onChange={setPage} />",
  "const [page, setPage] = createSignal(1);",
);
add(
  "Select Combobox",
  "Choose from labeled, grouped options.",
  componentDescriptionsChinese["Choose from labeled, grouped options."],
  '<N.Select items={[{ key: "languages", label: "Language", items: [{ key: "en", label: "English" }, { key: "zh", label: "Chinese" }] }]} placeholder="Select a language" />',
);
componentExamples.push({
  ...componentExamples.find((e) => e.names.includes("Select"))!,
  names: ["Combobox"],
  source: componentExamples.find((e) => e.names.includes("Select"))!.source.replace(/N\.Select/g, "N.Combobox"),
});
componentExamples.find((e) => e.names.includes("Select"))!.names = ["Select"];
add(
  "Calendar DatePicker",
  "Choose a date from a calendar.",
  componentDescriptionsChinese["Choose a date from a calendar."],
  '<N.DatePicker defaultValue={{ kind: "single", date: "2026-09-07" }} />',
);
componentExamples.push({
  ...componentExamples.find((e) => e.names.includes("DatePicker"))!,
  names: ["Calendar"],
  source: componentExamples.find((e) => e.names.includes("DatePicker"))!.source.replace(/N\.DatePicker/g, "N.Calendar"),
});
componentExamples.find((e) => e.names.includes("DatePicker"))!.names = ["DatePicker"];
add(
  "ColorPicker",
  "Choose and edit a color.",
  componentDescriptionsChinese["Choose and edit a color."],
  '<N.ColorPicker defaultValue="#3b82f6" label="Accent color" />',
);
add(
  "Progress ProgressCircle",
  "Show progress toward completion.",
  componentDescriptionsChinese["Show progress toward completion."],
  "<N.Progress value={65} />",
);
componentExamples.push({
  ...componentExamples.find((e) => e.names.includes("Progress"))!,
  names: ["ProgressCircle"],
  source: componentExamples
    .find((e) => e.names.includes("Progress"))!
    .source.replace(/N\.Progress/g, "N.ProgressCircle"),
});
componentExamples.find((e) => e.names.includes("Progress"))!.names = ["Progress"];
add(
  "Spinner",
  "Indicate that an operation is in progress.",
  componentDescriptionsChinese["Indicate that an operation is in progress."],
  "<N.Spinner />",
);
add(
  "Skeleton",
  "Reserve space while content is loading.",
  componentDescriptionsChinese["Reserve space while content is loading."],
  "<N.Skeleton style={{ width: 240, height: 100 }} />",
);
add(
  "ShimmerText",
  "Animate a text label while work is in progress.",
  componentDescriptionsChinese["Animate a text label while work is in progress."],
  '<N.ShimmerText text="Preparing your workspace…" durationMs={2200} color="#ffffff" style={{ color: "#52525b", fontSize: 22, fontWeight: "semibold" }} />',
);
add(
  "Label",
  "Display a label with optional secondary text.",
  componentDescriptionsChinese["Display a label with optional secondary text."],
  '<N.Label text="Workspace" secondary="Personal" />',
);
add(
  "Text",
  "Render a text value inside component compositions.",
  componentDescriptionsChinese["Render a text value inside component compositions."],
  '<N.Text value="Welcome back" />',
);
add(
  "TextView",
  "Display selectable rich text or Markdown.",
  componentDescriptionsChinese["Display selectable rich text or Markdown."],
  '<N.TextView text={"# Hello\\n\\nWelcome to **Solid GPUI**."} format="markdown" selectable />',
);
add(
  "Icon",
  "Display an SVG icon from the host asset collection.",
  componentDescriptionsChinese["Display an SVG icon from the host asset collection."],
  '<N.Icon source="lucide:check" />',
);
add("Caret", "Draw a caret indicator.", componentDescriptionsChinese["Draw a caret indicator."], "<N.Caret />");
add(
  "Kbd",
  "Display a keyboard shortcut.",
  componentDescriptionsChinese["Display a keyboard shortcut."],
  '<N.Kbd stroke="secondary-k" />',
);
add(
  "Link",
  "Open a destination from a text label.",
  componentDescriptionsChinese["Open a destination from a text label."],
  '<N.Link href="https://github.com/Cyenoch/solid-gpui"><N.Label text="Source code" /></N.Link>',
);
add(
  "Separator",
  "Separate adjacent content or sections.",
  componentDescriptionsChinese["Separate adjacent content or sections."],
  '<N.Separator label="Or continue with" />',
);
add(
  "Tag",
  "Present a short category or status label.",
  componentDescriptionsChinese["Present a short category or status label."],
  '<View style={{ alignItems: "center" }}><N.Tag variant="success"><N.Text value="Published" /></N.Tag></View>',
);
add(
  "Tab TabBar",
  "Navigate between related views.",
  componentDescriptionsChinese["Navigate between related views."],
  '<N.TabBar selectedIndex={selected()} onChange={setSelected}><N.Tab label="Overview" /><N.Tab label="Activity" /></N.TabBar>',
  "const [selected, setSelected] = createSignal(0);",
);
add(
  "Stepper StepperItem",
  "Show the current step in a sequence.",
  componentDescriptionsChinese["Show the current step in a sequence."],
  '<N.Stepper selectedIndex={1}><N.StepperItem><N.Label text="Account" /></N.StepperItem><N.StepperItem><N.Label text="Preferences" /></N.StepperItem></N.Stepper>',
);
add(
  "Table TableHeader TableHead TableBody TableRow TableCell TableFooter TableCaption",
  "Compose a table from headers, rows, cells and a caption.",
  componentDescriptionsChinese["Compose a table from headers, rows, cells and a caption."],
  `<N.Table>
      <N.TableCaption><N.Text value="Recent invoices" /></N.TableCaption>
      <N.TableHeader><N.TableRow><N.TableHead><N.Text value="Invoice" /></N.TableHead><N.TableHead><N.Text value="Amount" /></N.TableHead></N.TableRow></N.TableHeader>
      <N.TableBody><N.TableRow><N.TableCell><N.Text value="INV-001" /></N.TableCell><N.TableCell><N.Text value="$250" /></N.TableCell></N.TableRow></N.TableBody>
      <N.TableFooter><N.TableRow><N.TableCell colSpan={2}><N.Text value="1 invoice" /></N.TableCell></N.TableRow></N.TableFooter>
    </N.Table>`,
);
add(
  "Attachment AttachmentGroup AttachmentActions AttachmentContent AttachmentDescription AttachmentMedia AttachmentTitle",
  "Compose file attachments with media, metadata and actions.",
  componentDescriptionsChinese["Compose file attachments with media, metadata and actions."],
  '<N.AttachmentGroup><N.Attachment slots={{ media: <N.AttachmentMedia />, content: <N.AttachmentContent><N.AttachmentTitle text="Report.pdf" /><N.AttachmentDescription text="2.4 MB" /></N.AttachmentContent>, actions: <N.AttachmentActions><N.Button label="Download" /></N.AttachmentActions> }} /></N.AttachmentGroup>',
);
add(
  "Bubble BubbleContent BubbleGroup BubbleReactions",
  "Group conversation bubbles with content and reactions.",
  componentDescriptionsChinese["Group conversation bubbles with content and reactions."],
  '<N.BubbleGroup style={{ marginBottom: 28 }}><N.Bubble variant="secondary" slots={{ reactions: <N.Button label={liked() ? "Liked · 1" : "Like"} variant="ghost" size="small" onPress={() => setLiked(!liked())} /> }}><N.Text value="See you tomorrow!" /></N.Bubble></N.BubbleGroup>',
  "const [liked, setLiked] = createSignal(false);",
);
add(
  "Message MessageAvatar MessageContent MessageFooter MessageGroup MessageHeader MessageScroller",
  "Compose a conversation with avatars, headers and message content.",
  componentDescriptionsChinese["Compose a conversation with avatars, headers and message content."],
  `<N.MessageScroller style={{ height: 240 }}><N.MessageGroup><N.Message alignment="start" slots={{
      avatar: <N.Avatar name="Alex" />,
      header: <N.Label text="Alex" secondary="09:41" />,
      footer: <N.Label text="Just now" />
    }}><N.Text value="The new design is ready." /></N.Message></N.MessageGroup></N.MessageScroller>`,
);
add(
  "Marker MarkerContent MarkerIcon",
  "Add a compact status marker to a message or activity.",
  componentDescriptionsChinese["Add a compact status marker to a message or activity."],
  '<N.Marker><N.MarkerIcon><N.Spinner /></N.MarkerIcon><N.MarkerContent text="Uploading" /></N.Marker>',
);
add(
  "Form Field",
  "Lay out labeled form fields.",
  componentDescriptionsChinese["Lay out labeled form fields."],
  '<N.Form><N.Field required slots={{ label: <N.Label text="Name" /> }}><N.Input placeholder="Your name" /></N.Field></N.Form>',
);
add(
  "DescriptionList DescriptionItem DescriptionText",
  "Display labeled values in a structured layout.",
  componentDescriptionsChinese["Display labeled values in a structured layout."],
  '<N.DescriptionList bordered><N.DescriptionItem slots={{ label: <N.Label text="Status" /> }}><N.DescriptionText><N.Label text="Active" /></N.DescriptionText></N.DescriptionItem></N.DescriptionList>',
);
add(
  "GroupBox",
  "Place related controls in a labeled group.",
  componentDescriptionsChinese["Place related controls in a labeled group."],
  '<N.GroupBox slots={{ title: <N.Label text="Notifications" /> }}><N.Switch label="Email" checked /></N.GroupBox>',
);
add(
  "ResizablePanel ResizablePanelGroup",
  "Let people resize adjacent panes.",
  componentDescriptionsChinese["Let people resize adjacent panes."],
  '<N.ResizablePanelGroup style={{ height: 220, borderWidth: 1, borderColor: "#292929", borderRadius: 8 }}><N.ResizablePanel size={180}><View style={{ flexGrow: 1, padding: 20, gap: 12, backgroundColor: "#161616" }}><N.Label text="Files" /><N.Label text="App.tsx" /><N.Label text="theme.ts" /></View></N.ResizablePanel><N.ResizablePanel><View style={{ flexGrow: 1, padding: 20, gap: 12 }}><N.Text value="Document preview" /><N.Text value="Drag the divider to resize." /></View></N.ResizablePanel></N.ResizablePanelGroup>',
);
add(
  "Sidebar SidebarHeader SidebarFooter SidebarGroup SidebarMenu SidebarMenuItem SidebarToggleButton",
  "Build a navigation sidebar with grouped destinations.",
  componentDescriptionsChinese["Build a navigation sidebar with grouped destinations."],
  `<N.Sidebar style={{ height: 280 }} collapsed={collapsed()} slots={{ header: <N.SidebarHeader><N.Label text={collapsed() ? "" : "Workspace"} /><N.SidebarToggleButton collapsed={collapsed()} onPress={() => setCollapsed(!collapsed())} /></N.SidebarHeader>, footer: <N.SidebarFooter><N.Label text={collapsed() ? "" : "Personal account"} /></N.SidebarFooter> }}>
      <N.SidebarGroup label="Projects"><N.SidebarMenu><N.SidebarMenuItem label="Overview" icon="lucide:house" active={selected() === "Overview"} onPress={() => setSelected("Overview")} /><N.SidebarMenuItem label="Settings" icon="lucide:settings" active={selected() === "Settings"} onPress={() => setSelected("Settings")} /></N.SidebarMenu></N.SidebarGroup>
      </N.Sidebar>`,
  'const [collapsed, setCollapsed] = createSignal(false); const [selected, setSelected] = createSignal("Overview");',
);
add(
  "StatusBar TitleBar WindowBorder",
  "Compose the chrome of a native application window.",
  componentDescriptionsChinese["Compose the chrome of a native application window."],
  '<N.WindowBorder><N.TitleBar style={{ height: 48, padding: 0, paddingLeft: 88, paddingRight: 16, borderWidth: 0, borderBottomWidth: 1, borderBottomColor: "#2C2B33" }}><N.Label text="My application" /></N.TitleBar><N.Label text="Workspace content" /><N.StatusBar slots={{ left: <N.Label text="Ready" /> }} /></N.WindowBorder>',
);
add(
  "Scrollable",
  "Keep content within a bounded, keyboard-accessible region.",
  componentDescriptionsChinese["Keep content within a bounded, keyboard-accessible region."],
  "<N.Scrollable style={{ height: 240 }}>{Array.from({ length: 30 }, (_, i) => <View style={{ padding: 10 }}><N.Label text={`Project ${i + 1}`} /></View>)}</N.Scrollable>",
);
add(
  "ScrollShadow",
  "Reveal scrollable content with edge fades that disappear at the boundaries.",
  componentDescriptionsChinese["Reveal scrollable content with edge fades that disappear at the boundaries."],
  '<N.ScrollShadow axis="vertical" color="#0a0a0a" style={{ height: 180 }}>{Array.from({ length: 12 }, (_, i) => <View style={{ padding: 12 }}><N.Label text={`Project ${i + 1}`} /></View>)}</N.ScrollShadow>',
);
add(
  "FocusTrap",
  "Keep content within a bounded, keyboard-accessible region.",
  componentDescriptionsChinese["Keep content within a bounded, keyboard-accessible region."],
  '<N.FocusTrap autoFocus={false}><View style={{ gap: 12 }}><N.Input placeholder="First name" /><N.Input placeholder="Last name" /><N.Button label="Save profile" /></View></N.FocusTrap>',
);
add(
  "ListItem ListSeparatorItem SearchableListItemElement VirtualList",
  "Compose selectable rows in a scrolling list.",
  componentDescriptionsChinese["Compose selectable rows in a scrolling list."],
  '<View style={{ gap: 8 }}><N.ListItem><N.Label text="Inbox" /></N.ListItem><N.ListSeparatorItem /><N.ListItem><N.SearchableListItemElement><N.Label text="Archive" /></N.SearchableListItemElement></N.ListItem></View>',
);
componentExamples.push({
  names: ["VirtualList"],
  description: "Render a large collection of native child elements efficiently.",
  descriptionChinese: componentDescriptionsChinese["Render a large collection of native child elements efficiently."],
  source: `import * as N from "@solid-gpui/core/components";
export default function Example() {
  return <N.VirtualList itemSize={36} style={{ height: 320 }}>
    {Array.from({ length: 1000 }, (_, i) => <N.Label text={\`Item \${i + 1}\`} />)}
  </N.VirtualList>;
}`,
});
componentExamples.find((e) => e.names.includes("ListItem"))!.names = [
  "ListItem",
  "ListSeparatorItem",
  "SearchableListItemElement",
];
add(
  "Dialog DialogAction DialogClose DialogContent DialogDescription DialogFooter DialogHeader DialogTitle AlertDialog",
  "Present a focused dialog with a title, description and actions.",
  componentDescriptionsChinese["Present a focused dialog with a title, description and actions."],
  `<View style={{ gap: 12 }}><N.Button label="Open dialog" onPress={() => setOpen(true)} /><N.Dialog open={open()} onOpenChange={event => setOpen(event.open)} slots={{ footer: <N.DialogFooter><N.DialogClose><N.Button label="Cancel" /></N.DialogClose><N.DialogAction><N.Button label="Save" variant="primary" /></N.DialogAction></N.DialogFooter> }}>
      <N.DialogContent><N.DialogHeader><N.DialogTitle><N.Label text="Save changes?" /></N.DialogTitle><N.DialogDescription><N.Label text="Your changes will be saved to this workspace." /></N.DialogDescription></N.DialogHeader>
      </N.DialogContent>
    </N.Dialog></View>`,
  "const [open, setOpen] = createSignal(false);",
);
componentExamples.push({
  ...componentExamples.find((e) => e.names.includes("Dialog"))!,
  names: ["AlertDialog"],
  source: componentExamples
    .find((e) => e.names.includes("Dialog"))!
    .source.replace(/N\.Dialog(?=[ >])/g, "N.AlertDialog"),
});
componentExamples.find((e) => e.names.includes("Dialog"))!.names = [
  "Dialog",
  "DialogAction",
  "DialogClose",
  "DialogContent",
  "DialogDescription",
  "DialogFooter",
  "DialogHeader",
  "DialogTitle",
];
add(
  "Sheet",
  "Open supporting content in a panel at the window edge.",
  componentDescriptionsChinese["Open supporting content in a panel at the window edge."],
  '<View style={{ gap: 12 }}><N.Button label="Open preferences" onPress={() => setOpen(true)} /><N.Sheet open={open()} title="Preferences" placement="right" onOpenChange={event => setOpen(event.open)}><N.Switch label="Notifications" checked /></N.Sheet></View>',
  "const [open, setOpen] = createSignal(false);",
);
add(
  "Notification",
  "Deliver a dismissible message inside the application.",
  componentDescriptionsChinese["Deliver a dismissible message inside the application."],
  '<View><N.Button label="Show notification" onPress={() => setOpen(true)} /><N.Notification open={open()} title="Upload complete" message="Your file is ready." kind="success" delivery="inApp" onClose={() => setOpen(false)} /></View>',
  "const [open, setOpen] = createSignal(false);",
);
add(
  "Popover HoverCard Tooltip",
  "Show contextual information next to a trigger.",
  componentDescriptionsChinese["Show contextual information next to a trigger."],
  '<N.Popover slots={{ trigger: <N.Button label="More information" /> }}><N.Label text="Your workspace is private." /></N.Popover>',
);
componentExamples.push({
  ...componentExamples.find((e) => e.names.includes("Popover"))!,
  names: ["HoverCard"],
  source: componentExamples.find((e) => e.names.includes("Popover"))!.source.replace(/N\.Popover/g, "N.HoverCard"),
});
const popoverExample = componentExamples.find((e) => e.names.includes("Popover"))!;
popoverExample.names = ["Popover"];
popoverExample.description =
  "Show contextual information inside the current window. For native content beyond its boundary, see the System popovers guide.";
popoverExample.descriptionChinese = componentDescriptionsChinese[popoverExample.description];
add(
  "Tooltip",
  "Explain a control on hover or keyboard focus.",
  componentDescriptionsChinese["Explain a control on hover or keyboard focus."],
  '<N.Tooltip text="Save your changes" slots={{ trigger: <N.Button label="Save" /> }} />',
);
const menu =
  '{ items: [{ kind: "item", id: "copy", label: "Copy" }, { kind: "separator" }, { kind: "item", id: "delete", label: "Delete" }] }';
for (const name of ["DropdownMenu", "DropdownButton"])
  add(
    name,
    "Offer related actions in a dropdown menu.",
    componentDescriptionsChinese["Offer related actions in a dropdown menu."],
    `<N.${name} label="Actions" menu={${menu}} onSelect={event => console.log(event.id)} />`,
  );
for (const name of ["ContextMenu", "PopupMenu"])
  add(
    name,
    "Present a menu of contextual actions.",
    componentDescriptionsChinese["Present a menu of contextual actions."],
    name === "ContextMenu"
      ? `<N.ContextMenu menu={${menu}} slots={{ trigger: <N.Button label="Right-click here" /> }} />`
      : `<N.PopupMenu menu={${menu}} />`,
  );
add(
  "NativeMenu",
  "Open an operating-system menu from a trigger.",
  componentDescriptionsChinese["Open an operating-system menu from a trigger."],
  '<N.NativeMenu trigger="press" items={[{ kind: "item", id: "copy", label: "Copy" }]}><N.Button label="Open menu" /></N.NativeMenu>',
);
add(
  "AppMenuBar",
  "Provide an application menu bar.",
  componentDescriptionsChinese["Provide an application menu bar."],
  "<N.AppMenuBar />",
);
add(
  "Command",
  "Search and select commands from grouped results.",
  componentDescriptionsChinese["Search and select commands from grouped results."],
  '<N.Command groups={[{ key: "file", label: "File", items: [{ key: "new", label: "New document" }, { key: "open", label: "Open document" }] }]} placeholder="Search commands…" />',
);
add(
  "Tree",
  "Explore nested folders or hierarchical data.",
  componentDescriptionsChinese["Explore nested folders or hierarchical data."],
  '<N.Tree nodes={[{ key: "src", label: "src", expanded: true, children: [{ key: "app", label: "App.tsx" }] }]} style={{ height: 240 }} />',
);
add(
  "DataTable",
  "Display typed rows with column sizing, sorting and selection.",
  componentDescriptionsChinese["Display typed rows with column sizing, sorting and selection."],
  '<N.DataTable columns={[{ key: "name", label: "Name", sortable: true }, { key: "status", label: "Status" }]} rows={[{ key: "1", cells: { name: "Website", status: "Active" } }]} style={{ height: 280 }} />',
);
add(
  "Settings SettingPage SettingGroup SettingItem SettingField SettingCustomItem",
  "Build searchable application preferences with resettable fields.",
  componentDescriptionsChinese["Build searchable application preferences with resettable fields."],
  `<N.Settings sidebarWidth={160} style={{ height: 320 }}><N.SettingPage name="general" title="General"><N.SettingGroup name="notifications" title="Notifications">
      <N.SettingItem title="Email"><N.SettingField dirty={!enabled()} onReset={() => setEnabled(true)}><N.Switch checked={enabled()} onChange={setEnabled} /></N.SettingField></N.SettingItem>
      <N.SettingCustomItem><N.Label text="Changes are saved automatically." /></N.SettingCustomItem>
    </N.SettingGroup></N.SettingPage></N.Settings>`,
  "const [enabled, setEnabled] = createSignal(true);",
);
add(
  "DockArea",
  "Arrange persistent workspace panes as tabs or split panels.",
  componentDescriptionsChinese["Arrange persistent workspace panes as tabs or split panels."],
  '<N.DockArea panes={[{ name: "editor", title: "Editor", contentSlot: 0 }, { name: "preview", title: "Preview", contentSlot: 1 }]} initialLayout={{ center: { kind: "tabs", panes: ["editor", "preview"] } }} style={{ height: 360 }}><N.Input placeholder="Edit a document" /><N.Text value="Document preview" /></N.DockArea>',
);
const points = '[{ label: "Mon", value: 24 }, { label: "Tue", value: 52 }, { label: "Wed", value: 37 }]';
for (const name of ["LineChart", "BarChart"])
  add(
    name,
    "Visualize a series of labeled values.",
    componentDescriptionsChinese["Visualize a series of labeled values."],
    `<N.${name} data={${points}} style={{ height: 240 }} />`,
  );
for (const name of ["AreaChart", "RadarChart"])
  add(
    name,
    "Compare multiple data series across labeled categories.",
    componentDescriptionsChinese["Compare multiple data series across labeled categories."],
    `<N.${name} data={[{ label: "Mon", values: [24, 18] }, { label: "Tue", values: [52, 32] }, { label: "Wed", values: [37, 42] }]} series={[{ name: "Desktop", stroke: "#3b82f6" }, { name: "Mobile", stroke: "#10b981" }]} style={{ height: 280 }} />`,
  );
add(
  "CandlestickChart",
  "Compare open, high, low and close values.",
  componentDescriptionsChinese["Compare open, high, low and close values."],
  '<N.CandlestickChart data={[{ label: "Mon", open: 24, high: 36, low: 18, close: 32 }, { label: "Tue", open: 32, high: 40, low: 26, close: 28 }]} style={{ height: 240 }} />',
);
add(
  "PieChart",
  "Show how categories contribute to a total.",
  componentDescriptionsChinese["Show how categories contribute to a total."],
  '<N.PieChart data={[{ value: 60, label: "Desktop", color: "#3b82f6" }, { value: 40, label: "Mobile", color: "#10b981" }]} innerRadius={45} labels style={{ height: 240 }} />',
);
add(
  "SankeyChart",
  "Visualize flows between connected categories.",
  componentDescriptionsChinese["Visualize flows between connected categories."],
  '<N.SankeyChart nodes={[{ label: "Visits", color: "#3b82f6" }, { label: "Signups", color: "#10b981" }]} links={[{ source: 0, target: 1, value: 24 }]} style={{ height: 240 }} />',
);
add(
  "Plot",
  "Draw custom charts from typed plotting primitives.",
  componentDescriptionsChinese["Draw custom charts from typed plotting primitives."],
  '<N.Plot primitives={[{ kind: "line", points: [{ x: 20, y: 140 }, { x: 100, y: 60 }, { x: 180, y: 100 }], stroke: { kind: "solid", color: "#3b82f6" }, strokeWidth: 3 }]} style={{ height: 200 }} />',
);
add(
  "PlotDot",
  "Mark a point in plot coordinates.",
  componentDescriptionsChinese["Mark a point in plot coordinates."],
  '<N.PlotDot point={{ x: 80, y: 60 }} size={6} fill="#3b82f6" stroke="#ffffff" style={{ width: 240, height: 160 }} />',
);
add(
  "PlotCrossLine",
  "Add a crosshair at a plot position.",
  componentDescriptionsChinese["Add a crosshair at a plot position."],
  '<N.PlotCrossLine point={{ x: 80, y: 60 }} direction="both" style={{ width: 240, height: 160 }} />',
);
add(
  "PlotTooltip",
  "Display labeled values near a plot cursor.",
  componentDescriptionsChinese["Display labeled values near a plot cursor."],
  '<N.PlotTooltip cursor={{ x: 80, y: 60 }} within={{ width: 320, height: 240 }} title="Monday" rows={[{ color: "#3b82f6", label: "Visits", value: "24" }]} />',
);

// Window chrome is only valid at a surface root; the status bar can be embedded.
componentExamples.find((example) => example.names.includes("StatusBar"))!.names = ["TitleBar", "WindowBorder"];
add(
  "StatusBar",
  "Show application status and contextual information.",
  componentDescriptionsChinese["Show application status and contextual information."],
  '<N.StatusBar slots={{ left: <N.Label text="All changes saved" />, right: <N.Label text="UTF-8" /> }} />',
);

add(
  "List",
  "Search and select rows from a grouped collection.",
  componentDescriptionsChinese["Search and select rows from a grouped collection."],
  '<N.List sections={[{ key: "projects", header: "Projects", items: [{ key: "web", label: "Website" }, { key: "mobile", label: "Mobile app" }] }]} style={{ height: 220 }} />',
);

add(
  "Carousel CarouselItem",
  "A retained native carousel with keyboard navigation and pagination.",
  componentDescriptionsChinese["A retained native carousel with keyboard navigation and pagination."],
  '<N.Carousel pagination viewportHeight={140}><N.CarouselItem><N.Label text="Overview" /></N.CarouselItem><N.CarouselItem><N.Label text="Details" /></N.CarouselItem><N.CarouselItem><N.Label text="Next steps" /></N.CarouselItem></N.Carousel>',
  "",
);

add(
  "Motion",
  "Native motion samples transitions, springs and keyframes without per-frame JavaScript updates.",
  componentDescriptionsChinese[
    "Native motion samples transitions, springs and keyframes without per-frame JavaScript updates."
  ],
  '<View style={{ gap: 12 }}><N.Button label="Move" onPress={() => setMoved(!moved())} /><N.Motion target={{ x: moved() ? 100 : 0 }} animation={{ type: "spring", responseMs: 350 }}><N.Label text="Native spring" /></N.Motion></View>',
  "const [moved, setMoved] = createSignal(false);",
);

add(
  "NativePresence",
  "Retain children until their native exit animation completes.",
  componentDescriptionsChinese["Retain children until their native exit animation completes."],
  '<View style={{ gap: 12 }}><N.Button label="Toggle details" onPress={() => setShown(!shown())} /><N.NativePresence present={shown()}><N.Label text="Native presence" /></N.NativePresence></View>',
  "const [shown, setShown] = createSignal(true);",
);

add(
  "BaseButton",
  "An accessible native control with application-owned visuals.",
  componentDescriptionsChinese["An accessible native control with application-owned visuals."],
  '<N.BaseButton accessibilityLabel="Save" style={{ padding: 12, backgroundColor: "#1d4ed8", borderRadius: 8 }} onPress={() => setSaved(true)}><N.Label text={saved() ? "Saved" : "Save"} /></N.BaseButton>',
  "const [saved, setSaved] = createSignal(false);",
);

add(
  "BaseCheckbox",
  "An accessible native control with application-owned visuals.",
  componentDescriptionsChinese["An accessible native control with application-owned visuals."],
  '<N.BaseCheckbox accessibilityLabel="Accept terms" state={state()} onChange={setState} style={{ padding: 12, borderWidth: 1, borderColor: "#64748b", borderRadius: 8 }}><N.Label text={state() === "checked" ? "Accepted" : "Accept terms"} /></N.BaseCheckbox>',
  'const [state, setState] = createSignal<N.BaseCheckState>("unchecked");',
);

add(
  "BaseSwitch",
  "An accessible native control with application-owned visuals.",
  componentDescriptionsChinese["An accessible native control with application-owned visuals."],
  '<N.BaseSwitch accessibilityLabel="Notifications" checked={enabled()} onChange={setEnabled} style={{ padding: 12, backgroundColor: enabled() ? "#14532d" : "#334155", borderRadius: 8 }}><N.Label text={enabled() ? "Notifications on" : "Notifications off"} /></N.BaseSwitch>',
  "const [enabled, setEnabled] = createSignal(false);",
);

add(
  "BaseToggle",
  "An accessible native control with application-owned visuals.",
  componentDescriptionsChinese["An accessible native control with application-owned visuals."],
  '<N.BaseToggle accessibilityLabel="Pin" pressed={pinned()} onChange={setPinned} style={{ padding: 12, borderWidth: 1, borderColor: "#64748b", borderRadius: 8 }}><N.Label text={pinned() ? "Pinned" : "Pin"} /></N.BaseToggle>',
  "const [pinned, setPinned] = createSignal(false);",
);
