# Keyboard shortcuts and native menus

Register application commands with `root.setKeybindings(...)` and handle their
names through the root's `onAction` option. GPUI resolves modifiers, keyboard
layouts, and multi-stroke chords before delivering unconsumed native key events
to focused controls. Use `onKeyDown` for a control's local interaction.

```ts
const root = createRoot(transport, {
  onAction(action) {
    if (action === "Open") openDocument();
  },
});

await root.setKeybindings([
  { keystrokes: "secondary-o", actionName: "Open" },
  { keystrokes: "secondary-k secondary-p", actionName: "Command Palette" },
]);
```

The example assumes the application has supplied its transport and command
handlers. Register bindings after rendering the root, as with other surface
commands.

| GPUI syntax | Meaning |
| --- | --- |
| `secondary-o` | Command+O on macOS; Control+O on Windows and Linux |
| `ctrl-o` | Control+O on every platform |
| `cmd-o`, `super-o`, `win-o` | The platform modifier; these aliases do not mean Control on Windows or Linux |
| `secondary-shift-p` | The primary shortcut modifier plus Shift+P |
| `secondary-k secondary-p` | Two consecutive strokes, separated by a space |

Key names are case insensitive. An uppercase single-letter key adds Shift, so
use lowercase letters with explicit modifiers for clarity. The syntax and layout
matching come from the linked `gpui-pre 0.3.3` implementation in
`platform/keystroke.rs` and `keymap/binding.rs`.

Each call replaces that surface's complete binding set; `[]` clears it. A failed
parse leaves the prior set installed. The host retains compiled bindings and
installs only the active native window's set alongside the host profile's baseline
bindings. Two windows can use the same chord for different commands. Shortcuts
remain available when no control has focus or a provider overlay has focus;
focused native controls retain their own action dispatch behavior. Closing a
surface or accepting a new protocol epoch removes its old bindings. Register the
new epoch's bindings during application setup.

## Application menu bar

`root.setMenus(...)` replaces the application's shared menu definition. On macOS,
GPUI supplies the system menu bar. The gpui-component host also publishes the
definition to `<AppMenuBar />`, which applications can place in their Windows or
Linux window chrome.

```ts
await root.setMenus([
  {
    title: "File",
    items: [
      { type: "action", name: "Open" },
      { type: "separator" },
      { type: "action", name: "Save", disabled: true },
    ],
  },
]);
```

The action `name` is both the displayed label and the `onAction` value. Menu
actions route to the active surface, so every window using a shared menu should
handle its commands. Keep a single application owner for the shared definition;
`setMenus` does not create independent per-window menus. Update `checked` and
`disabled` from application state. Register matching shortcut action names when
the menu command should have a keyboard equivalent. Disabling a menu entry does
not remove a separately registered shortcut: update the binding set when the
command itself becomes unavailable.

## Context menus

The generated `<NativeMenu />` component supports right-click, left-press, and
manual triggers. Its `show({ x, y })` command uses logical coordinates relative to
the native window. macOS and Windows display OS popup menus; Linux uses the
gpui-component popup overlay, which is clipped to the window.

Menu item IDs must be unique across submenus. The host permits at most 1,024
entries and 16 submenu levels. Selection requires a mounted, enabled component,
an enabled item and enabled ancestors, and the same property generation that
created the popup. Updating props revokes an already open popup's action snapshot;
re-enabling or reusing an ID does not revive that snapshot. Unmounting the
component revokes its routes. These checks cover delayed OS selection callbacks.

## Verification

The host command roundtrip test covers native chords, platform-primary modifiers,
atomic replacement failure, identical shortcuts in different windows, focused
text input, empty replacement, and epoch retirement. The native-menu component
test covers selection, disabled ancestors, stale snapshots, and unmounting.
These deterministic tests do not replace smoke tests of the actual system menu
bar, keyboard layouts/IME, and native popup positioning on each release platform.
