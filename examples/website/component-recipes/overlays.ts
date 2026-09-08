import { recipes } from "./shared";

export const overlaysRecipes = [
  ...recipes("Dialog DialogAction DialogClose DialogContent DialogDescription DialogFooter DialogHeader DialogTitle", [
    {
      id: "form",
      title: "Form dialog",
      description: "Open a focused task, then save or cancel without leaving the page.",
      jsx: '<N.Button label="Edit profile" onPress={() => setOpen(true)} /><N.Dialog open={open()} onOpenChange={event => setOpen(event.open)} width={320} slots={{ footer: <N.DialogFooter><N.DialogClose><N.Button label="Cancel" /></N.DialogClose><N.DialogAction><N.Button label="Save changes" variant="primary" onPress={() => setResult("Profile saved ")} /></N.DialogAction></N.DialogFooter> }}><N.DialogContent><N.DialogHeader><N.DialogTitle><N.Label text="Edit profile" /></N.DialogTitle><N.DialogDescription><N.Label text="Update how your name appears to your team." /></N.DialogDescription></N.DialogHeader><N.Input defaultValue="Alex Chen" ariaLabel="Display name" /></N.DialogContent></N.Dialog><N.Label text={result()} />',
      setup: 'const [open, setOpen] = createSignal(false); const [result, setResult] = createSignal("");',
    },
    {
      id: "confirm",
      title: "Destructive confirmation",
      description: "Open a focused task, then save or cancel without leaving the page.",
      jsx: '<N.Button label="Delete project" onPress={() => setOpen(true)} /><N.Dialog open={open()} onOpenChange={event => setOpen(event.open)} width={320} slots={{ footer: <N.DialogFooter><N.DialogClose><N.Button label="Cancel" /></N.DialogClose><N.DialogAction><N.Button label="Delete" variant="danger" onPress={() => setResult("Project deleted in this example ")} /></N.DialogAction></N.DialogFooter> }}><N.DialogContent><N.DialogHeader><N.DialogTitle><N.Label text="Delete this project?" /></N.DialogTitle><N.DialogDescription><N.Label text="This example only updates the preview state." /></N.DialogDescription></N.DialogHeader></N.DialogContent></N.Dialog><N.Label text={result()} />',
      setup: 'const [open, setOpen] = createSignal(false); const [result, setResult] = createSignal("");',
    },
  ]),
  ...recipes("AlertDialog", [
    {
      id: "form",
      title: "Form dialog",
      description: "Open a focused task, then save or cancel without leaving the page.",
      jsx: '<N.Button label="Edit profile" onPress={() => setOpen(true)} /><N.AlertDialog open={open()} onOpenChange={event => setOpen(event.open)} width={320} slots={{ footer: <N.DialogFooter><N.DialogClose><N.Button label="Cancel" /></N.DialogClose><N.DialogAction><N.Button label="Save changes" variant="primary" onPress={() => setResult("Profile saved ")} /></N.DialogAction></N.DialogFooter> }}><N.DialogContent><N.DialogHeader><N.DialogTitle><N.Label text="Edit profile" /></N.DialogTitle><N.DialogDescription><N.Label text="Update how your name appears to your team." /></N.DialogDescription></N.DialogHeader><N.Input defaultValue="Alex Chen" ariaLabel="Display name" /></N.DialogContent></N.AlertDialog><N.Label text={result()} />',
      setup: 'const [open, setOpen] = createSignal(false); const [result, setResult] = createSignal("");',
    },
    {
      id: "confirm",
      title: "Destructive confirmation",
      description: "Open a focused task, then save or cancel without leaving the page.",
      jsx: '<N.Button label="Delete project" onPress={() => setOpen(true)} /><N.AlertDialog open={open()} onOpenChange={event => setOpen(event.open)} width={320} slots={{ footer: <N.DialogFooter><N.DialogClose><N.Button label="Cancel" /></N.DialogClose><N.DialogAction><N.Button label="Delete" variant="danger" onPress={() => setResult("Project deleted in this example ")} /></N.DialogAction></N.DialogFooter> }}><N.DialogContent><N.DialogHeader><N.DialogTitle><N.Label text="Delete this project?" /></N.DialogTitle><N.DialogDescription><N.Label text="This example only updates the preview state." /></N.DialogDescription></N.DialogHeader></N.DialogContent></N.AlertDialog><N.Label text={result()} />',
      setup: 'const [open, setOpen] = createSignal(false); const [result, setResult] = createSignal("");',
    },
  ]),
  ...recipes("Sheet", [
    {
      id: "left",
      title: "Left sheet",
      description: "Open supporting preferences from the chosen edge of the window.",
      jsx: '<N.Button label="Open left panel" onPress={() => setOpen(true)} /><N.Sheet open={open()} title="Preferences" placement="left" size={{ unit: "px", value: 280 }} onOpenChange={event => setOpen(event.open)}><N.Switch label="Email notifications" /><N.Switch label="Weekly summary" checked /></N.Sheet>',
      setup: "const [open, setOpen] = createSignal(false);",
    },
    {
      id: "right",
      title: "Right sheet",
      description: "Open supporting preferences from the chosen edge of the window.",
      jsx: '<N.Button label="Open right panel" onPress={() => setOpen(true)} /><N.Sheet open={open()} title="Preferences" placement="right" size={{ unit: "px", value: 280 }} onOpenChange={event => setOpen(event.open)}><N.Switch label="Email notifications" /><N.Switch label="Weekly summary" checked /></N.Sheet>',
      setup: "const [open, setOpen] = createSignal(false);",
    },
    {
      id: "bottom",
      title: "Bottom sheet",
      description: "Open supporting preferences from the chosen edge of the window.",
      jsx: '<N.Button label="Open bottom panel" onPress={() => setOpen(true)} /><N.Sheet open={open()} title="Preferences" placement="bottom" size={{ unit: "px", value: 280 }} onOpenChange={event => setOpen(event.open)}><N.Switch label="Email notifications" /><N.Switch label="Weekly summary" checked /></N.Sheet>',
      setup: "const [open, setOpen] = createSignal(false);",
    },
  ]),
  ...recipes("Notification", [
    {
      id: "success",
      title: "Success notification",
      description: "Trigger a dismissible in-app message from an explicit action.",
      jsx: '<N.Button label="Show success notification" onPress={() => setOpen(true)} /><N.Notification open={open()} title="Upload complete" message="Your file is ready." kind="success" delivery="inApp" onClose={() => setOpen(false)} />',
      setup: "const [open, setOpen] = createSignal(false);",
    },
    {
      id: "error",
      title: "Error notification",
      description: "Trigger a dismissible in-app message from an explicit action.",
      jsx: '<N.Button label="Show error notification" onPress={() => setOpen(true)} /><N.Notification open={open()} title="Upload failed" message="Check your connection and try again." kind="error" delivery="inApp" onClose={() => setOpen(false)} />',
      setup: "const [open, setOpen] = createSignal(false);",
    },
    {
      id: "action",
      title: "Notification action",
      description: "Offer a relevant follow-up directly inside the notification.",
      jsx: '<N.Button label="Archive document" onPress={() => setOpen(true)} /><N.Notification open={open()} title="Document archived" delivery="inApp" actionLabel="Undo" closeOnAction onAction={() => setRestored(true)} onClose={() => setOpen(false)} /><N.Label text={restored() ? "Document restored" : ""} />',
      setup: "const [open, setOpen] = createSignal(false); const [restored, setRestored] = createSignal(false);",
    },
  ]),
  ...recipes("Popover", [
    {
      id: "profile",
      title: "Profile details",
      description: "Show profile information next to the trigger, within the current window.",
      jsx: '<N.Popover slots={{ trigger: <N.Button label="Alex Chen" variant="link" /> }}><View style={{ gap: 8, padding: 12 }}><N.Avatar name="Alex Chen" /><N.Label text="Alex Chen" secondary="Product designer" /><N.Label text="Building a better workspace." /></View></N.Popover>',
      setup: "",
    },
    {
      id: "appearance",
      title: "Contextual details",
      description: "Keep a short explanation near its action, within the current window.",
      jsx: '<N.Popover anchor="topRight" slots={{ trigger: <N.Button label="Workspace access" outline /> }}><View style={{ padding: 12, gap: 8 }}><N.Label text="Invite-only workspace" /><N.Label text="Only invited members can view these projects." /></View></N.Popover>',
      setup: "",
    },
  ]),
  ...recipes("HoverCard", [
    {
      id: "profile",
      title: "Profile details",
      description: "Show supporting profile information next to the trigger.",
      jsx: '<N.HoverCard slots={{ trigger: <N.Button label="Alex Chen" variant="link" /> }}><View style={{ gap: 8, padding: 12 }}><N.Avatar name="Alex Chen" /><N.Label text="Alex Chen" secondary="Product designer" /><N.Label text="Building a better workspace." /></View></N.HoverCard>',
      setup: "",
    },
    {
      id: "appearance",
      title: "Contextual details",
      description: "Keep a short explanation close to the action it describes.",
      jsx: '<N.HoverCard anchor="topRight" slots={{ trigger: <N.Button label="Workspace access" outline /> }}><View style={{ padding: 12, gap: 8 }}><N.Label text="Invite-only workspace" /><N.Label text="Only invited members can view these projects." /></View></N.HoverCard>',
      setup: "",
    },
  ]),
  ...recipes("Tooltip", [
    {
      id: "shortcut",
      title: "Tooltip with shortcut",
      description: "Teach the keyboard shortcut alongside the action description.",
      jsx: '<N.Tooltip text="Save changes" keyBinding="secondary-s" slots={{ trigger: <N.Button label="Save" /> }} />',
      setup: "",
    },
    {
      id: "placement",
      title: "Tooltip placement",
      description: "Place the hint where it will not obscure nearby content.",
      jsx: '<N.Tooltip text="More information" placement="bottom" slots={{ trigger: <N.Button label="Hover for details" outline /> }} />',
      setup: "",
    },
  ]),
  ...recipes("DropdownMenu", [
    {
      id: "groups",
      title: "Grouped menu actions",
      description: "Group related actions, expose their availability, and report the selected command.",
      jsx: '<N.DropdownMenu label="Document actions" menu={{ items: [{ kind: "label", label: "Document" }, { kind: "item", id: "rename", label: "Rename" }, { kind: "item", id: "duplicate", label: "Duplicate" }, { kind: "separator" }, { kind: "item", id: "delete", label: "Delete", disabled: true }] }} onSelect={event => setAction(event.id)} /><N.Label text={action() ? `Selected: ${action()}` : "No action selected"} />',
      setup: 'const [action, setAction] = createSignal("");',
    },
    {
      id: "submenu",
      title: "Checks and submenus",
      description: "Group related actions, expose their availability, and report the selected command.",
      jsx: '<N.DropdownMenu label="Document actions" menu={{ items: [{ kind: "item", id: "pinned", label: "Pinned", checked: true }, { kind: "submenu", id: "export", label: "Export", menu: { items: [{ kind: "item", id: "pdf", label: "PDF" }, { kind: "item", id: "md", label: "Markdown" }] } }] }} onSelect={event => setAction(event.id)} /><N.Label text={action() ? `Selected: ${action()}` : "No action selected"} />',
      setup: 'const [action, setAction] = createSignal("");',
    },
  ]),
  ...recipes("DropdownButton", [
    {
      id: "groups",
      title: "Grouped menu actions",
      description: "Group related actions, expose their availability, and report the selected command.",
      jsx: '<N.DropdownButton label="Document actions" menu={{ items: [{ kind: "label", label: "Document" }, { kind: "item", id: "rename", label: "Rename" }, { kind: "item", id: "duplicate", label: "Duplicate" }, { kind: "separator" }, { kind: "item", id: "delete", label: "Delete", disabled: true }] }} onSelect={event => setAction(event.id)} /><N.Label text={action() ? `Selected: ${action()}` : "No action selected"} />',
      setup: 'const [action, setAction] = createSignal("");',
    },
    {
      id: "submenu",
      title: "Checks and submenus",
      description: "Group related actions, expose their availability, and report the selected command.",
      jsx: '<N.DropdownButton label="Document actions" menu={{ items: [{ kind: "item", id: "pinned", label: "Pinned", checked: true }, { kind: "submenu", id: "export", label: "Export", menu: { items: [{ kind: "item", id: "pdf", label: "PDF" }, { kind: "item", id: "md", label: "Markdown" }] } }] }} onSelect={event => setAction(event.id)} /><N.Label text={action() ? `Selected: ${action()}` : "No action selected"} />',
      setup: 'const [action, setAction] = createSignal("");',
    },
  ]),
  ...recipes("ContextMenu", [
    {
      id: "groups",
      title: "Grouped menu actions",
      description: "Group related actions, expose their availability, and report the selected command.",
      jsx: '<N.ContextMenu menu={{ items: [{ kind: "label", label: "Document" }, { kind: "item", id: "rename", label: "Rename" }, { kind: "item", id: "duplicate", label: "Duplicate" }, { kind: "separator" }, { kind: "item", id: "delete", label: "Delete", disabled: true }] }} onSelect={event => setAction(event.id)} slots={{ trigger: <N.Button label="Right-click for actions" outline /> }} /><N.Label text={action() ? `Selected: ${action()}` : "No action selected"} />',
      setup: 'const [action, setAction] = createSignal("");',
    },
    {
      id: "submenu",
      title: "Checks and submenus",
      description: "Group related actions, expose their availability, and report the selected command.",
      jsx: '<N.ContextMenu menu={{ items: [{ kind: "item", id: "pinned", label: "Pinned", checked: true }, { kind: "submenu", id: "export", label: "Export", menu: { items: [{ kind: "item", id: "pdf", label: "PDF" }, { kind: "item", id: "md", label: "Markdown" }] } }] }} onSelect={event => setAction(event.id)} slots={{ trigger: <N.Button label="Right-click for actions" outline /> }} /><N.Label text={action() ? `Selected: ${action()}` : "No action selected"} />',
      setup: 'const [action, setAction] = createSignal("");',
    },
  ]),
  ...recipes("PopupMenu", [
    {
      id: "groups",
      title: "Grouped menu actions",
      description: "Group related actions, expose their availability, and report the selected command.",
      jsx: '<N.PopupMenu menu={{ items: [{ kind: "label", label: "Document" }, { kind: "item", id: "rename", label: "Rename" }, { kind: "item", id: "duplicate", label: "Duplicate" }, { kind: "separator" }, { kind: "item", id: "delete", label: "Delete", disabled: true }] }} onSelect={event => setAction(event.id)} /><N.Label text={action() ? `Selected: ${action()}` : "No action selected"} />',
      setup: 'const [action, setAction] = createSignal("");',
    },
    {
      id: "submenu",
      title: "Checks and submenus",
      description: "Group related actions, expose their availability, and report the selected command.",
      jsx: '<N.PopupMenu menu={{ items: [{ kind: "item", id: "pinned", label: "Pinned", checked: true }, { kind: "submenu", id: "export", label: "Export", menu: { items: [{ kind: "item", id: "pdf", label: "PDF" }, { kind: "item", id: "md", label: "Markdown" }] } }] }} onSelect={event => setAction(event.id)} /><N.Label text={action() ? `Selected: ${action()}` : "No action selected"} />',
      setup: 'const [action, setAction] = createSignal("");',
    },
  ]),
  ...recipes("AppMenuBar", [
    {
      id: "groups",
      title: "Grouped menu actions",
      description:
        "Configure application menus once through the root, then place the menu bar in desktop window chrome.",
      jsx: "<N.AppMenuBar />",
      setup: "",
      preamble:
        'export async function configureMenus(root: import("@solid-gpui/core").Root) { await root.setMenus([{ title: "File", items: [{ type: "action", name: "Open" }, { type: "separator" }, { type: "action", name: "Save", disabled: true }] }]); }',
    },
    {
      id: "checked",
      title: "Checked menu actions",
      description:
        "Configure application menus once through the root, then place the menu bar in desktop window chrome.",
      jsx: "<N.AppMenuBar />",
      setup: "",
      preamble:
        'export async function configureMenus(root: import("@solid-gpui/core").Root) { await root.setMenus([{ title: "View", items: [{ type: "action", name: "Show sidebar", checked: true }, { type: "action", name: "Show status bar", checked: false }] }]); }',
    },
  ]),
  ...recipes("NativeMenu", [
    {
      id: "groups",
      title: "Grouped menu actions",
      description: "Group related actions, expose their availability, and report the selected command.",
      jsx: '<N.NativeMenu trigger="press" items={[{ kind: "item", id: "copy", label: "Copy" }, { kind: "separator" }, { kind: "item", id: "paste", label: "Paste", disabled: true }]} onSelect={event => setAction(event.id)}><N.Button label="Open native menu" /></N.NativeMenu><N.Label text={action() ? `Selected: ${action()}` : "No action selected"} />',
      setup: 'const [action, setAction] = createSignal("");',
    },
    {
      id: "submenu",
      title: "Checks and submenus",
      description: "Group related actions, expose their availability, and report the selected command.",
      jsx: '<N.NativeMenu trigger="contextMenu" items={[{ kind: "item", id: "pinned", label: "Pinned", checked: true }, { kind: "submenu", label: "Export", items: [{ kind: "item", id: "pdf", label: "PDF" }, { kind: "item", id: "markdown", label: "Markdown" }] }]} onSelect={event => setAction(event.id)}><N.Button label="Right-click for native menu" /></N.NativeMenu><N.Label text={action() ? `Selected: ${action()}` : "No action selected"} />',
      setup: 'const [action, setAction] = createSignal("");',
    },
  ]),
];
