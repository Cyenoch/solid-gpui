import { recipes } from "./shared";

export const dataRecipes = [
  ...recipes("Table TableHeader TableHead TableBody TableRow TableCell TableFooter TableCaption", [
    {
      id: "invoices",
      title: "Aligned amounts and totals",
      description: "Right-align numeric values and summarize the collection in a footer.",
      jsx: '<N.Table  accessibilityLabel="Invoices"><N.TableCaption><N.Text value="Recent invoices" /></N.TableCaption><N.TableHeader><N.TableRow><N.TableHead><N.Text value="Invoice" /></N.TableHead><N.TableHead align="right"><N.Text value="Amount" /></N.TableHead></N.TableRow></N.TableHeader><N.TableBody><N.TableRow><N.TableCell><N.Text value="INV-001" /></N.TableCell><N.TableCell align="right"><N.Text value="$250" /></N.TableCell></N.TableRow><N.TableRow><N.TableCell><N.Text value="INV-002" /></N.TableCell><N.TableCell align="right"><N.Text value="$150" /></N.TableCell></N.TableRow><N.TableRow><N.TableCell><N.Text value="INV-003" /></N.TableCell><N.TableCell align="right"><N.Text value="$350" /></N.TableCell></N.TableRow></N.TableBody><N.TableFooter><N.TableRow><N.TableCell><N.Text value="Total" /></N.TableCell><N.TableCell align="right"><N.Text value="$750" /></N.TableCell></N.TableRow></N.TableFooter></N.Table>',
      setup: "",
    },
    {
      id: "compact",
      title: "Compact table",
      description: "Reduce row density for a small inspector or dashboard.",
      jsx: '<N.Table size="small" accessibilityLabel="Invoices"><N.TableCaption><N.Text value="Recent invoices" /></N.TableCaption><N.TableHeader><N.TableRow><N.TableHead><N.Text value="Invoice" /></N.TableHead><N.TableHead align="right"><N.Text value="Amount" /></N.TableHead></N.TableRow></N.TableHeader><N.TableBody><N.TableRow><N.TableCell><N.Text value="INV-001" /></N.TableCell><N.TableCell align="right"><N.Text value="$250" /></N.TableCell></N.TableRow><N.TableRow><N.TableCell><N.Text value="INV-002" /></N.TableCell><N.TableCell align="right"><N.Text value="$150" /></N.TableCell></N.TableRow><N.TableRow><N.TableCell><N.Text value="INV-003" /></N.TableCell><N.TableCell align="right"><N.Text value="$350" /></N.TableCell></N.TableRow></N.TableBody><N.TableFooter><N.TableRow><N.TableCell><N.Text value="Total" /></N.TableCell><N.TableCell align="right"><N.Text value="$750" /></N.TableCell></N.TableRow></N.TableFooter></N.Table>',
      setup: "",
    },
  ]),
  ...recipes("DataTable", [
    {
      id: "striped",
      title: "Striped data table",
      description: "Distinguish adjacent rows and allow sorting by project name.",
      jsx: '<N.DataTable columns={[{ key: "name", label: "Project", sortable: true, width: 140 }, { key: "status", label: "Status", width: 100 }]} rows={[{ key: "1", cells: { name: "Website", status: "Active" } }, { key: "2", cells: { name: "Mobile app", status: "Review" } }, { key: "3", cells: { name: "Design system", status: "Published" } }]} stripe bordered rowSelectable style={{ height: 240 }} />',
      setup: "",
    },
    {
      id: "empty",
      title: "Empty collection",
      description: "Explain an empty result inside the same table structure.",
      jsx: '<N.DataTable columns={[{ key: "name", label: "Project", sortable: true, width: 140 }, { key: "status", label: "Status", width: 100 }]} rows={[]} slots={{ empty: <N.Label text="No projects match your search." /> }} style={{ height: 200 }} />',
      setup: "",
    },
    {
      id: "loading",
      title: "Loading rows",
      description: "Reserve the table area while the collection is being fetched.",
      jsx: '<N.DataTable columns={[{ key: "name", label: "Project", sortable: true, width: 140 }, { key: "status", label: "Status", width: 100 }]} loading style={{ height: 200 }} />',
      setup: "",
    },
  ]),
  ...recipes("ListItem ListSeparatorItem SearchableListItemElement", [
    {
      id: "selection",
      title: "Selectable rows",
      description: "Mark the active row and keep unavailable rows visible.",
      jsx: '<View style={{ gap: 8 }}><N.ListItem selected={selected() === "Inbox"} onPress={() => setSelected("Inbox")} slots={{ suffix: <N.Label text="3" /> }}><N.Label text="Inbox" /></N.ListItem><N.ListSeparatorItem /><N.ListItem selected={selected() === "Archive"} onPress={() => setSelected("Archive")}><N.SearchableListItemElement selected={selected() === "Archive"}><N.Label text="Archive" /></N.SearchableListItemElement></N.ListItem><N.ListItem disabled><N.Label text="Team inbox" /></N.ListItem></View>',
      setup: 'const [selected, setSelected] = createSignal("Inbox");',
    },
    {
      id: "compact",
      title: "Compact action list",
      description: "Group a short set of actions with a separator and status suffix.",
      jsx: '<View style={{ gap: 4 }}><N.ListItem slots={{ suffix: <N.Kbd stroke="secondary-n" /> }}><N.Label text="New document" /></N.ListItem><N.ListSeparatorItem /><N.ListItem><N.SearchableListItemElement checked><N.Label text="Show archived" /></N.SearchableListItemElement></N.ListItem></View>',
      setup: "",
    },
  ]),
  ...recipes("List", [
    {
      id: "search",
      title: "Searchable collection",
      description: "Search grouped results without changing the surrounding layout.",
      jsx: '<N.List searchable filterable sections={[{ key: "projects", header: "Projects", items: [{ key: "web", label: "Website" }, { key: "mobile", label: "Mobile app" }, { key: "design", label: "Design system" }] }]} searchPlaceholder="Find a project" style={{ height: 240 }} />',
      setup: "",
    },
    {
      id: "empty",
      title: "Empty collection",
      description: "Explain an empty result inside the same collection structure.",
      jsx: '<N.List sections={[]} slots={{ empty: <N.Label text="No projects found." /> }} style={{ height: 180 }} />',
      setup: "",
    },
  ]),
  ...recipes("VirtualList", [
    {
      id: "vertical",
      title: "Large vertical list",
      description: "Keep a long collection in a bounded viewport with consistent row heights.",
      jsx: '<N.VirtualList itemSize={36} style={{ height: 220 }}>{Array.from({ length: 500 }, (_, i) => <View style={{ height: 36, justifyContent: "center" }}><N.Label text={`Project ${String(i + 1).padStart(3, "0")}`} /></View>)}</N.VirtualList>',
      setup: "",
    },
    {
      id: "horizontal",
      title: "Horizontal collection",
      description: "Browse equally sized cards in a horizontal viewport.",
      jsx: '<N.VirtualList orientation="horizontal" itemSize={160} style={{ height: 100 }}>{Array.from({ length: 30 }, (_, i) => <View style={{ width: 160, height: 90, padding: 12 }}><N.Label text={`Collection ${i + 1}`} /></View>)}</N.VirtualList>',
      setup: "",
    },
  ]),
  ...recipes("Tree", [
    {
      id: "nested",
      title: "Nested folders",
      description: "Expand a file hierarchy and select individual documents.",
      jsx: '<N.Tree nodes={[{ key: "src", label: "src", expanded: true, children: [{ key: "components", label: "components", expanded: true, children: [{ key: "button", label: "Button.tsx" }] }, { key: "app", label: "App.tsx" }] }, { key: "package", label: "package.json" }]} style={{ height: 220 }} />',
      setup: "",
    },
    {
      id: "dense",
      title: "Dense hierarchy",
      description: "Use smaller row spacing for a compact file navigator.",
      jsx: '<N.Tree rowHeight={28} indent={12} nodes={[{ key: "docs", label: "Documentation", expanded: true, children: [{ key: "intro", label: "Introduction.md" }, { key: "install", label: "Installation.md" }, { key: "api", label: "API.md" }] }]} style={{ height: 180 }} />',
      setup: "",
    },
  ]),
  ...recipes("Command", [
    {
      id: "groups",
      title: "Grouped commands",
      description: "Organize searchable actions by their purpose and show keyboard shortcuts.",
      jsx: '<N.Command bordered groups={[{ key: "file", label: "File", items: [{ key: "new", label: "New document", shortcut: "secondary-n" }, { key: "open", label: "Open document", shortcut: "secondary-o" }] }, { key: "settings", label: "Settings", items: [{ key: "profile", label: "Profile" }, { key: "billing", label: "Billing", disabled: true }] }]} placeholder="Search commands…" maxHeight={220} onConfirm={setCommand} /><N.Label text={command() ? `Selected: ${command()}` : "Choose a command"} />',
      setup: 'const [command, setCommand] = createSignal("");',
    },
    {
      id: "empty",
      title: "No matching commands",
      description: "Give search results a useful empty state.",
      jsx: '<N.Command groups={[]} bordered slots={{ empty: <N.Label text="No commands found. Try another search." /> }} maxHeight={180} />',
      setup: "",
    },
  ]),
  ...recipes(
    "Attachment AttachmentGroup AttachmentActions AttachmentContent AttachmentDescription AttachmentMedia AttachmentTitle",
    [
      {
        id: "uploading",
        title: "Attachment uploading",
        description: "Keep the file identity visible while communicating its transfer state.",
        jsx: '<N.AttachmentGroup><N.Attachment status="uploading" slots={{ media: <N.AttachmentMedia />, content: <N.AttachmentContent><N.AttachmentTitle text="Design brief.pdf" status="uploading" /><N.AttachmentDescription text="Uploading 2.4 MB…" status="uploading" /></N.AttachmentContent>, actions: <N.AttachmentActions><N.Button label="Cancel" /></N.AttachmentActions> }} /></N.AttachmentGroup>',
        setup: "",
      },
      {
        id: "failed",
        title: "Attachment failed",
        description: "Keep the file identity visible while communicating its transfer state.",
        jsx: '<N.AttachmentGroup><N.Attachment status="failed" slots={{ media: <N.AttachmentMedia />, content: <N.AttachmentContent><N.AttachmentTitle text="Design brief.pdf" status="failed" /><N.AttachmentDescription text="Upload failed. Please try again." status="failed" /></N.AttachmentContent>, actions: <N.AttachmentActions><N.Button label="Retry" /></N.AttachmentActions> }} /></N.AttachmentGroup>',
        setup: "",
      },
    ],
  ),
  ...recipes("Bubble BubbleContent BubbleGroup BubbleReactions", [
    {
      id: "outline",
      title: "Outline conversation",
      description: "Separate incoming and outgoing messages using alignment and appearance.",
      jsx: '<N.BubbleGroup style={{ gap: 36, marginBottom: 28 }}><N.Bubble alignment="start" variant="outline" slots={{ reactions: <N.Button label="Like" variant="ghost" size="small" /> }}><N.Text value="Is the new design ready?" /></N.Bubble><N.Bubble alignment="end" variant="filled"><N.Text value="Yes, ready for review." /></N.Bubble></N.BubbleGroup>',
      setup: "",
    },
    {
      id: "secondary",
      title: "Secondary conversation",
      description: "Separate incoming and outgoing messages using alignment and appearance.",
      jsx: '<N.BubbleGroup style={{ gap: 36, marginBottom: 28 }}><N.Bubble alignment="start" variant="secondary" slots={{ reactions: <N.Button label="Like" variant="ghost" size="small" /> }}><N.Text value="Is the new design ready?" /></N.Bubble><N.Bubble alignment="end" variant="filled"><N.Text value="Yes, ready for review." /></N.Bubble></N.BubbleGroup>',
      setup: "",
    },
    {
      id: "tinted",
      title: "Tinted conversation",
      description: "Separate incoming and outgoing messages using alignment and appearance.",
      jsx: '<N.BubbleGroup style={{ gap: 36, marginBottom: 28 }}><N.Bubble alignment="start" variant="tinted" slots={{ reactions: <N.Button label="Like" variant="ghost" size="small" /> }}><N.Text value="Is the new design ready?" /></N.Bubble><N.Bubble alignment="end" variant="filled"><N.Text value="Yes, ready for review." /></N.Bubble></N.BubbleGroup>',
      setup: "",
    },
  ]),
  ...recipes("Message MessageAvatar MessageContent MessageFooter MessageGroup MessageHeader MessageScroller", [
    {
      id: "start",
      title: "Incoming messages",
      description: "Compose the author, timestamp, content, and delivery status as one message.",
      jsx: '<N.MessageScroller style={{ height: 220 }}><N.MessageGroup><N.Message alignment="start" slots={{ avatar: <N.Avatar name="Alex Chen" />, header: <N.Label text="Alex Chen" secondary="09:41" />, footer: <N.Label text="Delivered" /> }}><N.Text value="The updated prototype is ready for review." /></N.Message></N.MessageGroup></N.MessageScroller>',
      setup: "",
    },
    {
      id: "end",
      title: "Outgoing messages",
      description: "Compose the author, timestamp, content, and delivery status as one message.",
      jsx: '<N.MessageScroller style={{ height: 220 }}><N.MessageGroup><N.Message alignment="end" slots={{ avatar: <N.Avatar name="Alex Chen" />, header: <N.Label text="Alex Chen" secondary="09:41" />, footer: <N.Label text="Delivered" /> }}><N.Text value="The updated prototype is ready for review." /></N.Message></N.MessageGroup></N.MessageScroller>',
      setup: "",
    },
  ]),
  ...recipes("Marker MarkerContent MarkerIcon", [
    {
      id: "separator",
      title: "Separator marker",
      description: "Mark a meaningful event between items in an activity stream.",
      jsx: '<N.Marker variant="separator"><N.MarkerIcon><N.Icon path="icons/check.svg" /></N.MarkerIcon><N.MarkerContent text="All changes saved" /></N.Marker>',
      setup: "",
    },
    {
      id: "border",
      title: "Border marker",
      description: "Mark a meaningful event between items in an activity stream.",
      jsx: '<N.Marker variant="border"><N.MarkerIcon><N.Icon path="icons/check.svg" /></N.MarkerIcon><N.MarkerContent text="All changes saved" /></N.Marker>',
      setup: "",
    },
  ]),
  ...recipes("Settings SettingPage SettingGroup SettingItem SettingField SettingCustomItem", [
    {
      id: "outline",
      title: "Outline preferences",
      description: "Edit a preference, then reset it to its original value.",
      jsx: '<N.Settings groupVariant="outline" sidebarWidth={160} style={{ height: 300 }}><N.SettingPage name="general" title="General" resettable><N.SettingGroup name="privacy" title="Privacy" description="Choose what your team can see."><N.SettingItem title="Activity" description="Share your recent activity." orientation="vertical"><N.SettingField dirty={!enabled()} onReset={() => setEnabled(true)}><N.Switch checked={enabled()} onChange={setEnabled} /></N.SettingField></N.SettingItem><N.SettingCustomItem><N.Label text="Changes apply immediately." /></N.SettingCustomItem></N.SettingGroup></N.SettingPage></N.Settings>',
      setup: "const [enabled, setEnabled] = createSignal(true);",
    },
    {
      id: "fill",
      title: "Fill preferences",
      description: "Edit a preference, then reset it to its original value.",
      jsx: '<N.Settings groupVariant="fill" sidebarWidth={160} style={{ height: 300 }}><N.SettingPage name="general" title="General" resettable><N.SettingGroup name="privacy" title="Privacy" description="Choose what your team can see."><N.SettingItem title="Activity" description="Share your recent activity." orientation="vertical"><N.SettingField dirty={!enabled()} onReset={() => setEnabled(true)}><N.Switch checked={enabled()} onChange={setEnabled} /></N.SettingField></N.SettingItem><N.SettingCustomItem><N.Label text="Changes apply immediately." /></N.SettingCustomItem></N.SettingGroup></N.SettingPage></N.Settings>',
      setup: "const [enabled, setEnabled] = createSignal(true);",
    },
  ]),
  ...recipes("DockArea", [
    {
      id: "tabs",
      title: "Tabbed workspace",
      description: "Compose document and preview panes with a shared docking layout.",
      jsx: '<N.DockArea locked={false} panes={[{ name: "notes", title: "Notes", contentSlot: 0 }, { name: "preview", title: "Preview", contentSlot: 1 }]} initialLayout={{ center: { kind: "tabs", panes: ["notes", "preview"], activeIndex: 0 } }} style={{ height: 240 }}><N.Textarea placeholder="Write a note…" rows={3} /><N.Label text="Your document preview appears here." /></N.DockArea>',
      setup: "",
    },
    {
      id: "locked",
      title: "Locked workspace",
      description: "Compose document and preview panes with a shared docking layout.",
      jsx: '<N.DockArea locked={true} panes={[{ name: "notes", title: "Notes", contentSlot: 0 }, { name: "preview", title: "Preview", contentSlot: 1 }]} initialLayout={{ center: { kind: "tabs", panes: ["notes", "preview"], activeIndex: 1 } }} style={{ height: 240 }}><N.Textarea placeholder="Write a note…" rows={3} /><N.Label text="Your document preview appears here." /></N.DockArea>',
      setup: "",
    },
  ]),
];
