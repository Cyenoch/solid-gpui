import workspaceSource from "./Workspace.tsx?raw";
import accountSource from "./Account.tsx?raw";
import collectionsSource from "./Collections.tsx?raw";
export const showcases = [
  {
    id: "workspace",
    title: "Workspace",
    description: "Keep your next project moving with a focused task list.",
    source: workspaceSource,
    components: ["Input", "Button", "Checkbox", "Progress"],
    note: "Add tasks, mark them complete, and filter your list. Your changes stay in this demo session.",
  },
  {
    id: "account",
    title: "Account",
    description: "A simple settings page for the details that matter.",
    source: accountSource,
    components: ["Input", "Switch", "Button", "Alert"],
    note: "Edit your profile and notification preferences, then save to see confirmation. No account is required.",
  },
  {
    id: "collections",
    title: "Collections",
    description: "Find your way through ten thousand items.",
    source: collectionsSource,
    components: ["Input", "Button"],
    note: "Search collections and select an item. Only the visible rows are rendered as you scroll.",
  },
] as const;
