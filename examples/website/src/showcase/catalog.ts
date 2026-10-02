import workspaceSource from "./Workspace.tsx?raw";
import accountSource from "./Account.tsx?raw";
import collectionsSource from "./Collections.tsx?raw";
import studioSource from "./ReferenceStudio.tsx?raw";
import studioState from "./reference/state.ts?raw";
import studioModel from "./reference/model.ts?raw";
const referenceSource = `// ReferenceStudio.tsx\n${studioSource}\n// reference/state.ts\n${studioState}\n// reference/model.ts\n${studioModel}`;
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
  {
    id: "reference-studio",
    title: "Reference Studio",
    description: "Edit a 240-track cut alongside ten thousand review notes.",
    source: referenceSource,
    components: ["Input", "Button"],
    note: "Drag clips between tracks, reorder track headers, edit titles and timing, and post Unicode review notes. Native scrolling renders a bounded row window. The inspector copies review text; selectable paragraphs support native selection. See the Reference application guide for source provenance and acceptance requirements.",
  },
  {
    id: "reference-studio-history",
    title: "Studio History",
    description: "Keep the same editing panes while browsing a large review history.",
    source: referenceSource,
    components: ["Input", "Button"],
    note: "Switch between Timeline and History without replacing the studio shell, inputs, or lists. Narrow windows retain the inactive pane at zero width; Inspector toggles the detail pane. Search after scrolling to the final review, then clear the search and post a note.",
  },
] as const;
