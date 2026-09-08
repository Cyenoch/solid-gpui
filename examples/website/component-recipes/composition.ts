import { recipes } from "./shared";

export const compositionRecipes = [
  ...recipes("Accordion AccordionItem", [
    {
      id: "multiple",
      title: "Multiple sections",
      description: "Keep more than one answer open while comparing details.",
      jsx: '<N.Accordion multiple bordered onChange={setOpen}><N.AccordionItem open={open().includes(0)} slots={{ title: <N.Label text="Can I invite my team?" /> }}><N.Label text="Invite teammates from workspace settings." /></N.AccordionItem><N.AccordionItem open={open().includes(1)} slots={{ title: <N.Label text="Can I export my data?" /> }}><N.Label text="Export a copy at any time." /></N.AccordionItem></N.Accordion>',
      setup: "const [open, setOpen] = createSignal([0, 1]);",
    },
    {
      id: "disabled",
      title: "Unavailable section",
      description: "Disable a section while leaving other answers accessible.",
      jsx: '<N.Accordion><N.AccordionItem open={open()} onChange={setOpen} slots={{ title: <N.Label text="Personal workspace" /> }}><N.Label text="Manage your own projects." /></N.AccordionItem><N.AccordionItem disabled slots={{ title: <N.Label text="Enterprise policies" /> }}><N.Label text="Contact your administrator." /></N.AccordionItem></N.Accordion>',
      setup: "const [open, setOpen] = createSignal(false);",
    },
  ]),
  ...recipes("Collapsible", [
    {
      id: "controlled",
      title: "Expandable details",
      description: "Let a visible trigger reveal and hide supporting content.",
      jsx: '<N.Collapsible open={open()} slots={{ trigger: <N.Button label={open() ? "Hide files" : "Show files"} onPress={() => setOpen(!open())} /> }}><N.Label text="App.tsx" /><N.Label text="package.json" /></N.Collapsible>',
      setup: "const [open, setOpen] = createSignal(false);",
    },
    {
      id: "summary",
      title: "Optional form details",
      description: "Keep advanced preferences out of the main form until requested.",
      jsx: '<N.Input placeholder="Project name" /><N.Collapsible open={open()} slots={{ trigger: <N.Button label="Advanced options" variant="ghost" onPress={() => setOpen(!open())} /> }}><N.Switch label="Include archived items" /><N.Switch label="Show activity timestamps" /></N.Collapsible>',
      setup: "const [open, setOpen] = createSignal(false);",
    },
  ]),
  ...recipes("Breadcrumb BreadcrumbItem", [
    {
      id: "navigation",
      title: "Interactive path",
      description: "Make ancestor locations actionable and identify the current destination.",
      jsx: '<N.Breadcrumb><N.BreadcrumbItem label="Home" onPress={() => setLocation("Home")} /><N.BreadcrumbItem label="Projects" onPress={() => setLocation("Projects")} /><N.BreadcrumbItem label="Website" disabled /></N.Breadcrumb><N.Label text={`Selected: ${location()}`} />',
      setup: 'const [location, setLocation] = createSignal("Website");',
    },
    {
      id: "short",
      title: "Short hierarchy",
      description: "Use a compact path for shallow settings navigation.",
      jsx: '<N.Breadcrumb><N.BreadcrumbItem label="Settings" /><N.BreadcrumbItem label="Profile" disabled /></N.Breadcrumb>',
      setup: "",
    },
  ]),
  ...recipes("Tab TabBar", [
    {
      id: "pill",
      title: "Pill tabs",
      description: "Choose a tab treatment that fits the surrounding navigation.",
      jsx: '<N.TabBar variant="pill" selectedIndex={selected()} onChange={setSelected}><N.Tab label="Overview" /><N.Tab label="Activity" /><N.Tab label="Settings" disabled /></N.TabBar><N.Label text={selected() === 0 ? "Your project at a glance." : "Recent changes from your team."} />',
      setup: "const [selected, setSelected] = createSignal(0);",
    },
    {
      id: "outline",
      title: "Outline tabs",
      description: "Choose a tab treatment that fits the surrounding navigation.",
      jsx: '<N.TabBar variant="outline" selectedIndex={selected()} onChange={setSelected}><N.Tab label="Overview" /><N.Tab label="Activity" /><N.Tab label="Settings" disabled /></N.TabBar><N.Label text={selected() === 0 ? "Your project at a glance." : "Recent changes from your team."} />',
      setup: "const [selected, setSelected] = createSignal(0);",
    },
    {
      id: "segmented",
      title: "Segmented tabs",
      description: "Choose a tab treatment that fits the surrounding navigation.",
      jsx: '<N.TabBar variant="segmented" selectedIndex={selected()} onChange={setSelected}><N.Tab label="Overview" /><N.Tab label="Activity" /><N.Tab label="Settings" disabled /></N.TabBar><N.Label text={selected() === 0 ? "Your project at a glance." : "Recent changes from your team."} />',
      setup: "const [selected, setSelected] = createSignal(0);",
    },
    {
      id: "underline",
      title: "Underline tabs",
      description: "Choose a tab treatment that fits the surrounding navigation.",
      jsx: '<N.TabBar variant="underline" selectedIndex={selected()} onChange={setSelected}><N.Tab label="Overview" /><N.Tab label="Activity" /><N.Tab label="Settings" disabled /></N.TabBar><N.Label text={selected() === 0 ? "Your project at a glance." : "Recent changes from your team."} />',
      setup: "const [selected, setSelected] = createSignal(0);",
    },
  ]),
  ...recipes("Stepper StepperItem", [
    {
      id: "vertical",
      title: "Vertical steps",
      description: "Present a longer sequence in a narrow layout.",
      jsx: '<N.Stepper orientation="vertical" selectedIndex={step()} onChange={setStep}><N.StepperItem><N.Label text="Create account" /></N.StepperItem><N.StepperItem><N.Label text="Choose a plan" /></N.StepperItem><N.StepperItem><N.Label text="Finish setup" /></N.StepperItem></N.Stepper>',
      setup: "const [step, setStep] = createSignal(0);",
    },
    {
      id: "progress",
      title: "Step progression",
      description: "Advance through the sequence with a clear next action.",
      jsx: '<N.Stepper selectedIndex={step()} size="small"><N.StepperItem><N.Label text="Account" /></N.StepperItem><N.StepperItem><N.Label text="Review" /></N.StepperItem><N.StepperItem><N.Label text="Done" /></N.StepperItem></N.Stepper><N.Button label={step() === 2 ? "Start again" : "Next step"} onPress={() => setStep((step() + 1) % 3)} />',
      setup: "const [step, setStep] = createSignal(0);",
    },
  ]),
  ...recipes("Pagination", [
    {
      id: "compact",
      title: "Compact pagination",
      description: "Navigate a large result set with minimal horizontal space.",
      jsx: "<N.Pagination compact currentPage={page()} totalPages={24} onChange={setPage} /><N.Label text={`Page ${page()} of 24`} />",
      setup: "const [page, setPage] = createSignal(1);",
    },
    {
      id: "limited",
      title: "Limited page window",
      description: "Keep the current page and its neighbors visible.",
      jsx: '<N.Pagination currentPage={page()} totalPages={12} visiblePages={5} size="small" onChange={setPage} /><N.Pagination currentPage={1} totalPages={3} compact disabled />',
      setup: "const [page, setPage] = createSignal(6);",
    },
  ]),
  ...recipes("Form Field", [
    {
      id: "help",
      title: "Field descriptions",
      description: "Explain a required field close to where it is edited.",
      jsx: '<N.Form orientation="vertical"><N.Field required slots={{ label: <N.Label text="Workspace name" />, description: <N.Label text="Visible to everyone on your team." /> }}><N.Input placeholder="Acme design" /></N.Field><N.Field slots={{ label: <N.Label text="Description" /> }}><N.Textarea rows={2} placeholder="What is this workspace for?" /></N.Field></N.Form>',
      setup: "",
    },
    {
      id: "conditional",
      title: "Conditional field",
      description: "Reveal a follow-up field only when it is relevant.",
      jsx: '<N.Form><N.Field slots={{ label: <N.Label text="Notifications" /> }}><N.Switch label="Send email updates" checked={enabled()} onChange={setEnabled} /></N.Field><N.Field visible={enabled()} slots={{ label: <N.Label text="Email" /> }}><N.Input placeholder="you@example.com" /></N.Field></N.Form>',
      setup: "const [enabled, setEnabled] = createSignal(false);",
    },
  ]),
  ...recipes("GroupBox", [
    {
      id: "fill",
      title: "Fill group",
      description: "Give related preferences a shared title and visual boundary.",
      jsx: '<N.GroupBox variant="fill" slots={{ title: <N.Label text="Notifications" /> }}><N.Switch label="Email updates" /><N.Switch label="Weekly summary" checked /></N.GroupBox>',
      setup: "",
    },
    {
      id: "outline",
      title: "Outline group",
      description: "Give related preferences a shared title and visual boundary.",
      jsx: '<N.GroupBox variant="outline" slots={{ title: <N.Label text="Notifications" /> }}><N.Switch label="Email updates" /><N.Switch label="Weekly summary" checked /></N.GroupBox>',
      setup: "",
    },
  ]),
  ...recipes("DescriptionList DescriptionItem DescriptionText", [
    {
      id: "vertical",
      title: "Stacked details",
      description: "Place labels above values when the available width is limited.",
      jsx: '<N.DescriptionList orientation="vertical" columns={1} bordered><N.DescriptionItem slots={{ label: <N.Label text="Project" /> }}><N.DescriptionText><N.Label text="Website redesign" /></N.DescriptionText></N.DescriptionItem><N.DescriptionItem slots={{ label: <N.Label text="Owner" /> }}><N.DescriptionText><N.Label text="Alex Chen" /></N.DescriptionText></N.DescriptionItem></N.DescriptionList>',
      setup: "",
    },
    {
      id: "compact",
      title: "Compact details",
      description: "Use a compact key-value summary inside a card or inspector.",
      jsx: '<N.DescriptionList size="small" labelWidth={88}><N.DescriptionItem slots={{ label: <N.Label text="Status" /> }}><N.DescriptionText><N.Label text="Published" /></N.DescriptionText></N.DescriptionItem><N.DescriptionItem slots={{ label: <N.Label text="Updated" /> }}><N.DescriptionText><N.Label text="Today" /></N.DescriptionText></N.DescriptionItem></N.DescriptionList>',
      setup: "",
    },
  ]),
  ...recipes("ResizablePanel ResizablePanelGroup", [
    {
      id: "horizontal",
      title: "Horizontal panes",
      description: "Drag the divider to allocate space between adjacent panes.",
      jsx: '<N.ResizablePanelGroup orientation="horizontal" style={{ height: 220, borderWidth: 1, borderColor: "#292929", borderRadius: 8 }}><N.ResizablePanel size={120}><View style={{ flexGrow: 1, padding: 16, backgroundColor: "#161616" }}><N.Label text="Files" /></View></N.ResizablePanel><N.ResizablePanel><View style={{ flexGrow: 1, padding: 16 }}><N.Text value="Document preview" /></View></N.ResizablePanel></N.ResizablePanelGroup>',
      setup: "",
    },
    {
      id: "vertical",
      title: "Vertical panes",
      description: "Drag the divider to allocate space between adjacent panes.",
      jsx: '<N.ResizablePanelGroup orientation="vertical" style={{ height: 220, borderWidth: 1, borderColor: "#292929", borderRadius: 8 }}><N.ResizablePanel size={120}><View style={{ flexGrow: 1, padding: 16, backgroundColor: "#161616" }}><N.Label text="Files" /></View></N.ResizablePanel><N.ResizablePanel><View style={{ flexGrow: 1, padding: 16 }}><N.Text value="Document preview" /></View></N.ResizablePanel></N.ResizablePanelGroup>',
      setup: "",
    },
  ]),
  ...recipes("Sidebar SidebarHeader SidebarFooter SidebarGroup SidebarMenu SidebarMenuItem SidebarToggleButton", [
    {
      id: "active",
      title: "Active destination",
      description: "Keep one destination active and preserve the surrounding sidebar.",
      jsx: '<N.Sidebar style={{ height: 280 }} collapsed={collapsed()} slots={{ header: <N.SidebarHeader><N.Label text={collapsed() ? "" : "Workspace"} /><N.SidebarToggleButton collapsed={collapsed()} onPress={() => setCollapsed(!collapsed())} /></N.SidebarHeader>, footer: <N.SidebarFooter><N.Label text={collapsed() ? "" : "Personal account"} /></N.SidebarFooter> }}><N.SidebarGroup label="Projects"><N.SidebarMenu><N.SidebarMenuItem label="Overview" icon="icons/building-2.svg" active={selected() === "Overview"} onPress={() => setSelected("Overview")} /><N.SidebarMenuItem label="Activity" icon="icons/bell.svg" active={selected() === "Activity"} onPress={() => setSelected("Activity")} /></N.SidebarMenu></N.SidebarGroup></N.Sidebar>',
      setup:
        'const [collapsed, setCollapsed] = createSignal(false); const [selected, setSelected] = createSignal("Overview");',
    },
    {
      id: "collapse",
      title: "Collapsible navigation",
      description: "Toggle the sidebar while retaining its selected destination.",
      jsx: '<N.Sidebar style={{ height: 280 }} collapsed={collapsed()} slots={{ header: <N.SidebarHeader><N.Label text={collapsed() ? "" : "Workspace"} /><N.SidebarToggleButton collapsed={collapsed()} onPress={() => setCollapsed(!collapsed())} /></N.SidebarHeader>, footer: <N.SidebarFooter><N.Label text={collapsed() ? "" : "Personal account"} /></N.SidebarFooter> }}><N.SidebarGroup label="Projects"><N.SidebarMenu><N.SidebarMenuItem label="Overview" icon="icons/building-2.svg" active={selected() === "Overview"} onPress={() => setSelected("Overview")} /><N.SidebarMenuItem label="Activity" icon="icons/bell.svg" active={selected() === "Activity"} onPress={() => setSelected("Activity")} /></N.SidebarMenu></N.SidebarGroup></N.Sidebar>',
      setup:
        'const [collapsed, setCollapsed] = createSignal(true); const [selected, setSelected] = createSignal("Activity");',
    },
  ]),
  ...recipes("Scrollable", [
    {
      id: "vertical",
      title: "Scrollable content",
      description: "Constrain a long list to a fixed-height region.",
      jsx: '<N.Scrollable axis="vertical" containScroll style={{ height: 180 }}>{Array.from({ length: 20 }, (_, i) =><N.Label text={`Project ${i + 1}`} />)}</N.Scrollable>',
      setup: "",
    },
    {
      id: "horizontal",
      title: "Horizontal scrolling",
      description: "Keep wide content inside its own horizontal scroll area.",
      jsx: '<N.Scrollable axis="horizontal" style={{ height: 80 }}><View style={{ width: 720, flexDirection: "row", gap: 24 }}><N.Label text="Overview" /><N.Label text="Recent activity" /><N.Label text="Team members" /><N.Label text="Project settings" /><N.Label text="Archived projects" /></View></N.Scrollable>',
      setup: "",
    },
  ]),
  ...recipes("FocusTrap", [
    {
      id: "form",
      title: "Focused form",
      description: "Use Tab and Shift+Tab to move between fields in this region.",
      jsx: '<N.FocusTrap autoFocus={false}><N.Input placeholder="First name" /><N.Input placeholder="Last name" /><N.Button label="Save profile" /></N.FocusTrap>',
      setup: "",
    },
    {
      id: "search",
      title: "Focused search",
      description: "Keep search and its action together in one keyboard region.",
      jsx: '<N.FocusTrap autoFocus={false}><N.Input placeholder="Search this collection" /><N.Button label="Search" /></N.FocusTrap>',
      setup: "",
    },
  ]),
  ...recipes("StatusBar", [
    {
      id: "window-0",
      title: "Document window",
      description: "Compose window-level controls in the desktop host.",
      jsx: '<N.StatusBar slots={{ left: <N.Label text="All changes saved" />, right: <N.Label text="UTF-8" /> }} />',
      setup: "",
    },
    {
      id: "window-1",
      title: "Workspace window",
      description: "Compose window-level controls in the desktop host.",
      jsx: '<N.StatusBar slots={{ left: <N.Label text="3 selected" />, right: <N.Label text="120 items" /> }} />',
      setup: "",
    },
  ]),
  ...recipes("TitleBar", [
    {
      id: "window-0",
      title: "Document window",
      description: "Compose window-level controls in the desktop host.",
      jsx: '<N.WindowBorder><N.TitleBar><N.Label text="Project notes" /></N.TitleBar><N.Textarea rows={3} placeholder="Start writing…" /><N.StatusBar slots={{ left: <N.Label text="All changes saved" /> }} /></N.WindowBorder>',
      setup: "",
    },
    {
      id: "window-1",
      title: "Workspace window",
      description: "Compose window-level controls in the desktop host.",
      jsx: '<N.WindowBorder shadowSize={8}><N.TitleBar><N.Label text="Project overview" /></N.TitleBar><N.Label text="Your workspace is ready." /><N.StatusBar slots={{ left: <N.Label text="Online" /> }} /></N.WindowBorder>',
      setup: "",
    },
  ]),
  ...recipes("WindowBorder", [
    {
      id: "window-0",
      title: "Document window",
      description: "Compose window-level controls in the desktop host.",
      jsx: '<N.WindowBorder><N.TitleBar><N.Label text="Project notes" /></N.TitleBar><N.Textarea rows={3} placeholder="Start writing…" /><N.StatusBar slots={{ left: <N.Label text="All changes saved" /> }} /></N.WindowBorder>',
      setup: "",
    },
    {
      id: "window-1",
      title: "Workspace window",
      description: "Compose window-level controls in the desktop host.",
      jsx: '<N.WindowBorder shadowSize={8}><N.TitleBar><N.Label text="Project overview" /></N.TitleBar><N.Label text="Your workspace is ready." /><N.StatusBar slots={{ left: <N.Label text="Online" /> }} /></N.WindowBorder>',
      setup: "",
    },
  ]),
];
