# React GPUI 0.2.0

React GPUI 0.2.0 is a native-app bridge: a React renderer running with Bun and a Rust/GPUI host, connected by protocol v3 with 35 commands and 23 native events. React owns components, hooks, context, and JavaScript closures; the host owns the retained native tree, layout, input state, and drawing. This release turns the current end-to-end surface into a coherent consumer-facing cut without pretending it is a browser or a finished cross-platform distribution.

## Highlights

- **Rich text and interactive link runs.** A `Text` paragraph can mix raw strings with one level of styled nested `Text` runs while preserving selection and copy across the flattened paragraph. A run with `onPress` can be a clickable range, receives native focus and a tab stop, activates through unmodified Enter, and paints a high-contrast focus affordance on each wrapped line; Space remains non-activating.

- **A complete native TextInput interaction model.** Single-line inputs support click-to-place, drag selection, UTF-16-safe keyboard extension, and visible selection; double-click selects a UAX #29 word and triple-click selects the clicked logical line. Host-owned Cmd/Ctrl-C, X, V, and A, Option/Alt word navigation, bounded undo/redo, IME composition boundaries, and caret/marked-range following keep editing behavior on the native side while controlled inputs continue to receive the normal change and selection events.

- **Opt-in high-frequency pointer movement.** `View` and `Pressable` can subscribe with `onPointerMove`; coordinates are finite logical window pixels, modifiers use the ordered native names, and nodes without the handler register no native move listener. The registered 143-node measurement delivered 240 events and 240 commits at 240 Hz within its measured 4.17 ms frame interval; this is a scenario result, not a universal throughput guarantee.

- **Virtual lists with native variable-height layout.** `VirtualList` commits only the required range, measures committed rows at their natural heights, and uses `estimatedItemSize` only as an initial hint for unmeasured rows. Visible-range and end-reached behavior, empty states, index scrolling, overscan, and row eviction/remount semantics are covered; `getScrollOffset()` and `scrollToOffset()` preserve precise logical-pixel positions across refreshes and clamp restores to the new content end.

- **Multiple surfaces and asynchronous close decisions.** `createSurfaceHost` routes several native surfaces over one runtime, with creation-time normal/floating/dialog kinds, resizability, and minimum sizes. Closed or explicitly unmounted surface IDs are retired permanently, and a `require-confirmation` close policy sends one request at a time to JavaScript so an application can allow or keep the surface open without synchronous cancellation.

- **File, image, clipboard, and font resources.** Root commands now cover asynchronous file and save dialogs, bounded UTF-8 text-file reads and writes, clipboard text, and encoded PNG/JPEG/GIF/SVG clipboard images. `Image` supports host-visible paths, `fallbackSource`, and native object fitting; `Root.loadFont()` registers bundled TTF/OTF fonts asynchronously and returns the metadata family for `fontFamily` use.

- **Notifications, menus, keybindings, and accessibility.** Applications can submit platform notifications, compose static menus with disabled and checked action items, and replace a surface's validated keybinding set; menu and keybinding actions arrive through the same `onAction` route. Focusable `View` and `Pressable` nodes expose focus/blur and keyboard notifications, focus traversal uses the native tab-stop graph, and recognized accessibility roles, labels, descriptions, expanded state, and heading levels are forwarded to the AccessKit-backed tree.

- **Window controls and themed examples.** Root methods cover title, resize, size/bounds/state reads, minimize, activation, zoom, fullscreen, URL opening, and multi-surface creation. Appearance observation reports light/dark values, while shared application-owned theme tokens now drive the gallery and focused examples; the runnable set includes rich text, text input, virtual lists, focus flow, dropdowns, drag reorder, multi-surface controls, notes persistence, and runtime-font loading.

- **Consumer test facade, performance guards, and transport diagnostics.** `@react-gpui/dev` provides `renderTestApp` locators and behavior-level interactions over the real headless dispatch path, while `TestApp` can inject selection, external-file-drop, and layout events and drain emitted commands. Rich-text, TextInput, event-storm, and large-tree guards protect measured hot paths; bounded ordered transport queues, typed termination causes, malformed-frame fail-fast handling, panic crash reports, and payload-free protocol taps make failures diagnosable instead of silently stranded.

## Breaking and behavior notes

- **Protocol v3 is lockstep.** Renderer and host versions must match. Decoders now enforce the current tuple arities for TextInput, Image, Drag, WindowResize, and CommandResult; historical forms are rejected with typed diagnostics rather than silently filling fields. Upgrade the renderer and host together.

- **Submit callbacks receive authoritative text.** `onSubmitEditing` is `(value: string) => void`, and the value is the native text at the time of Enter, including a valid empty string. Submit is emitted for focused single-line inputs; multiline Enter remains text insertion.

- **Length units are deliberate.** TextInput `maxLength` remains measured in UTF-16 units, while image paths, URL/resource limits, and other wire caps are measured in UTF-8 bytes. TypeScript numeric encoders now force float32 values before they cross the protocol.

- **Surface epochs do not revive IDs.** A closed or explicitly unmounted surface cannot be recreated by changing its epoch. Register a fresh host-allocated surface ID; pending commands on a closed root reject with `SurfaceClosedError`, and same-ID recreation raises `SurfaceIdReusedError`.

- **Native notifications are not browser events.** Press, keyboard, pointer, hover, scroll, layout, and lifecycle callbacks are semantic notifications and cannot be synchronously canceled with `preventDefault()`. Pointer coordinates are supplied on down/up and opt-in move events; hover remains an edge notification.

- **The wire shape is stable where the API is unchanged.** Internal renderer, host, and protocol module splits preserve the public API and wire behavior; the responsive gallery and documentation changes do not require application migration.

## Fixed

- **Rich-text correctness:** targeted cache invalidation now rebuilds only changed paragraphs and required ancestors, and interactive cursor feedback stays scoped to the listener-bearing clickable range instead of leaking across sibling text.

- **Focus lifecycle:** native tab stops now include eligible `View`, `Pressable`, `TextInput`, and selectable `Text` nodes; disabled controls are skipped, windows remain isolated, and unmounting the focused node emits blur and restores a live focus target. Pressable focusability patches are accepted correctly.

- **Pointer and overlay interaction:** platform mouse-up values with a zero click count are normalized to the protocol minimum, pointer dispatch retains its event kind while carrying down/up actions, TextInput blur is exposed, and anchored overlay placement follows the corrected path.

- **Transitions:** retargets preserve the prior transition metadata when a style update omits it, sample the current presentation value, restart the declared delay, and emit one completion per generation.

- **Command and event decoding:** acknowledgements for surface and clipboard command kinds are no longer dropped; integer resize dimensions are disambiguated from animation payloads; malformed event payloads now report their actual decode errors directly.

- **Native layout and glyph rendering:** macOS text rendering now exercises the font-kit path instead of a false-green no-op path, and nested flex direction/gap styles reach GPUI layout so compact gallery content can scroll without horizontal overflow.

- **TextInput history:** undo and redo use the bounded host-owned input history while preserving ordinary change and selection events, including controlled-input acknowledgement behavior.

## Known boundaries

- **Per-run typography is intentionally bounded.** The pinned GPUI text API shapes one paragraph with one font size and line height. A nested Text Run cannot set its own `fontSize` or `lineHeight` (or layout/non-typography fields); those props are rejected loudly. Apply paragraph-level size and line height, and use runs for color, weight, style, decoration, or family.

- **The macOS host candidate is unsigned and not notarized.** Signing, notarization, and publisher authentication are still pending human release decisions. `SHA256SUMS` verifies archive contents and consistency; it does not authenticate the publisher.

- **Cross-platform runner evidence is not complete.** Linux Wayland/X11 and Windows have feature/build coverage, but display-backed runtime validation and release artifacts are pending. The cross-platform workflow remains runner-only until its first push/PR jobs execute; embedded Bun is currently macOS-only.

- **Quartz display behavior needs a real display.** Headless checks do not prove native file-picker interaction, clipboard/menu/window effects, glyph pixels, drag visuals, tooltip appearance, pointer placement, or the AccessKit tree. The current no-display Quartz environment therefore cannot substitute for a display-backed macOS acceptance run.

- **Other intentional omissions remain.** There is no DOM/browser compatibility layer or synchronous native cancellation. Secure/password TextInput display, JavaScript `Image.onError`, explicit RTL base direction and bidi-aware caret/hit geometry, `letterSpacing`, and `pointerEvents` are outside this protocol version; X11 and Wayland explicitly reject clipboard-image operations rather than converting them to text.

## Install and get started

Follow the [consumer getting-started guide](docs/getting-started.md) for the pinned Bun and Rust toolchains, package installation, process host, optional macOS embedded-Bun runtime, first surface, progressive recipes, platform matrix, and troubleshooting pointers. The guide starts with `bun add react @react-gpui/core` and explains how the host Runtime Adapter must own the renderer process pipe.
