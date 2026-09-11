# Preserve UI state with captureState

Native UI reloads create new Solid owners and component instances. Use
`mountApplication`'s `captureState` callback to carry explicit application data
into `setup(previous)` in the replacement generation. Without these two pieces,
signals return to their initial values when you save application source.

## A complete QuickJS example

Configure Vite with `solidGpui({ entry: "src/main.tsx", runtime: "quickjs", native })`
and use an `EmbeddedTransport` entry:

```tsx
import { mountApplication, Pressable, Text, TextInput, View } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";
import { createSignal } from "@solid-gpui/core/runtime";

type ReloadState = {
  page: "downloads" | "settings";
  query: string;
  directoryDraft: string;
};

mountApplication<ReloadState>({
  transport: () => new EmbeddedTransport(),
  setup(previous) {
    const [page, setPage] = createSignal<ReloadState["page"]>(previous?.page ?? "downloads");
    const [query, setQuery] = createSignal(previous?.query ?? "");
    const [directoryDraft, setDirectoryDraft] = createSignal(previous?.directoryDraft ?? "");
    return {
      captureState: () => ({ page: page(), query: query(), directoryDraft: directoryDraft() }),
      render: () => (
        <View style={{ padding: 24, gap: 12 }}>
          <Text>Current page: {page()}</Text>
          <Pressable onPress={() => setPage(page() === "downloads" ? "settings" : "downloads")}>
            <Text>Switch page</Text>
          </Pressable>
          <TextInput value={query()} onChangeText={setQuery} accessibilityLabel="Search" />
          <TextInput value={directoryDraft()} onChangeText={setDirectoryDraft} accessibilityLabel="Folder draft" />
        </View>
      ),
    };
  },
});
```

Switch the page and type in both inputs, then save a JSX edit. The replacement
reads the captured values before rendering. `captureState` belongs to the
definition returned by `setup`, alongside `render`. Read signals inside the
callback so it captures their current values each time it runs.

For external Bun HMR, use `StdioTransport` and set
`hotKey: import.meta.hot ? import.meta.url : undefined` on `mountApplication`.
QuickJS generation reload does not require `hotKey`; its host supplies the
generation identity. See [Vite integration](vite.md) for runtime setup.

## Keep the snapshot small and explicit

| Capture | Recreate or reload |
| --- | --- |
| Current page, search, filters, sorting and selected IDs | Solid owners, accessors, effects and timers |
| Unsaved form values and validation selection | Native clients, component refs, windows and transports |
| The revision a draft was edited against | Live engine status and task progress from Rust |
| A stable request ID needed for an explicit retry | Promises and in-flight callbacks |

Put important form state in a model owned by `setup`, then pass that model to the
form component. A component-local signal is not captured automatically. Each
model can accept its previous data and expose `captureState`; the application
combines those results into one object. Return copies of mutable arrays and
records, such as `selected: [...selected()]` and `draft: { ...draft() }`.

QuickJS requires acyclic JSON data: plain objects, dense arrays, strings,
booleans, finite numbers other than negative zero, and null. Omit absent properties or use null; nested
undefined values, functions, accessors, Maps/Sets, symbols and native handles
are rejected. The snapshot is limited to 1 MiB, 100,000 values and 64 levels.
Do not use a JSON stringify/parse round trip to hide unsupported values. See
[the complete capture contract](hot-reload.md#captured-state).

## Reconnect services without overwriting drafts

Create signals from `previous` synchronously in `setup`. Connect native clients
and fetch current backend data in `onMount`, after the candidate is activated.
Clean up old subscriptions and timers with `onCleanup`.

A load method that always assigns `draft = saved.values` will erase the restored
form as soon as its request resolves. Preserve a dirty draft and its original
revision while refreshing the catalog. If saved settings changed meanwhile,
keep the conflict visible; explicitly discard the draft to accept current values.
Restoring UI data must not automatically submit a form or replay a native command.

For a write that cannot safely span reload, reject capture explicitly:

```ts
captureState() {
  if (saving()) throw new Error("Wait for the save to finish, then save source again to reload.");
  return { draft: { ...draft() }, revision: revision() };
}
```

A QuickJS capture failure keeps the current generation interactive. Once the
operation finishes, another source edit can retry the reload. Busy flags and
pending promises cannot be restored as if the operation were still attached to
the new VM.

## What survives each kind of restart

| Event | Behavior |
| --- | --- |
| Successful TS/TSX reload | New UI generation receives the captured data; the Rust host remains alive. |
| Failed capture or candidate preparation | The previous QuickJS generation resumes. |
| Rust rebuild, native contract change, or host crash | A fresh host session starts; the in-memory checkpoint is lost. |
| Full application restart | Only data explicitly persisted by the application is restored. |

Native focus, selection, scroll caches and component instances remount unless
the application deliberately restores their supported state. `captureState`
does not replace durable storage or make asynchronous native effects reversible.

## Verify the actual reload

Edit a draft, change navigation and select an item before saving TSX. Verify the
restored values after native loading finishes, then interact with them again.
Exercise a failed load and a reload during a pending save. In QuickJS, check for
capture errors as well as the `applied` message: activation alone does not prove
that later loading preserved your draft. See [reload verification](hot-reload.md#verify-application-reload).
