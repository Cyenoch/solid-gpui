# Native application migration

The migration API targets Rust `solid-gpui`, `@solid-gpui/core`, and
`@solid-gpui/router` **0.3.0**, from the same source revision. These changes use
protocol v5 with an updated schema digest; deploy the Rust host and JavaScript
packages together. This is source delivery, not a crates.io/npm publication.
Use the existing `mountApplication`, Vite/Bun reload, typed native modules, and
`@solid-gpui/router` APIs with an application-owned native host. The migration
example combines those APIs with window profiles, application theme overrides,
edge styles, and embedded icon resources.

The website exposes this guide at `/docs/reference/native-migration` (under the
site's hash route in the browser). Run the shared desktop website with
`bun run website:native`; run the dedicated migration fixture with the commands below.
Native window chrome and process-backed services require the desktop host. The
browser website documents these APIs without simulating operating-system windows.

## Application-owned runtime and window profile

`run_application_with_profile(profile, Arc<dyn RuntimeAdapter>)` does not parse
CLI arguments or create a runtime. It uses the same host runner as `run`:
commit admission, native events, protocol handlers, surface ownership, overlays,
close handling, and final runtime shutdown remain framework-owned.

```rust
use solid_gpui::{gpui::*, components::host::ComponentHost};

let profile = ComponentHost::new(vec![
    solid_gpui::components::native_module(),
    application_module,
])
.with_window_options(|_, cx| {
    let mut options = gpui_component::TitleBar::window_options();
    options.window_bounds = Some(WindowBounds::Windowed(Bounds::centered(
        None, size(px(1100.), px(720.)), cx,
    )));
    options.window_min_size = Some(size(px(960.), px(640.)));
    options.titlebar.as_mut().unwrap().traffic_light_position =
        Some(point(px(16.), px(17.)));
    options
})
.with_performance_monitor(false);
solid_gpui::run_application_with_profile(profile, runtime);
```

The window options callback runs for each opened surface. Use the supplied
options when retaining renderer-requested bounds. Explicit open-surface title,
kind, resizability, and minimum size are applied after the callback. Custom
profiles can implement `HostProfile::window_options` directly.

`TitleBar::window_options()` supplies transparent macOS titlebar options and
`app_owns_titlebar_drag: true`. Its traffic-light position is the top-left of the
close button; 17 px centers a 14 px button in 48 px. The host does not add another
Solid titlebar. Render exactly one:

```tsx
<TitleBar
  style={{
    height: 48,
    padding: 0,
    paddingLeft: 88,
    paddingRight: 16,
    backgroundColor: "#131217",
    borderWidth: 0,
    borderBottomWidth: 1,
    borderColor: "#2C2B33",
  }}
>
  <View style={{ flexDirection: "row", flexGrow: 1, alignItems: "center" }}>
    <Text>Application</Text>
    <Input placeholder="Search" style={{ width: 160 }} />
  </View>
</TitleBar>
```

Style refinements replace the TitleBar's native defaults, including its left
padding. There is no additional fullscreen padding. The native TitleBar owns
blank-area dragging and calls macOS `titlebar_double_click`, respecting the
system preference. Controls claiming mouse-down do not initiate titlebar drag
or double-click zoom. Core `Pressable` also claims this native default action.

The performance monitor defaults to **off in every build**. The website opts in
with `with_performance_monitor(true)`; environment variables do not override
application policy.

## Application theme

The generated native client exposes:

```ts
const native = createClient(root); // @solid-gpui/core/components
await native.setTheme("dark");
await native.setApplicationTheme({
  fontFamily: ".SystemUIFont",
  fontSize: 14,
  lineHeight: 20,
  radius: 6,
  radiusLg: 10,
  colors: {
    background: "#131217",
    sidebar: "#0F0E12",
    foreground: "#ECEAF1",
    input: "#1B1A20",
    popover: "#222127",
    border: "#2C2B33",
    mutedForeground: "#A09DA9",
    primary: "#D4688C",
    primaryHover: "#E07B9E",
    primaryForeground: "#241219",
    buttonPrimary: "#D4688C",
    buttonPrimaryHover: "#E07B9E",
    buttonPrimaryForeground: "#241219",
    danger: "#C4574E",
    ring: "#D4688C",
  },
});
```

`ApplicationThemeColors` exposes the linked component library's complete solid
color token set, including component-specific hover/active/selected states,
input/focus colors, menu/popover/dialog chrome, overlays, and disabled/muted
colors. Generic `primary` and component-specific `buttonPrimary` are distinct
tokens; set both when the application uses the same color. The example includes
one shared palette for native tokens and Solid styles.

Calls replace the previous application overrides, using the selected light/dark
base for omitted values. Overrides survive subsequent mode and system appearance
changes. Invalid colors, unknown fields, invalid font names, and invalid lengths
are rejected before mutation. Both native component and Base theme snapshots are
updated, as are TextView defaults. Rich text resolves inherited native typography
when laid out instead of storing default-black text runs in its content cache.

`components` configures shared native metrics for `button`, `input`, `select`,
`tag`, `menu`, and `dialog`. Each accepts `height`, `fontSize`, `lineHeight`,
`paddingX`, `paddingY`, and `radius`, in logical pixels. Metrics apply before
per-instance styles. For example:

```ts
await native.setApplicationTheme({
  colors: { foreground: "#ECEAF1" },
  inputBackground: "#1B1A20",
  components: {
    button: { height: 32, fontSize: 14, lineHeight: 20, paddingX: 12 },
    input: { height: 32, fontSize: 14, lineHeight: 20 },
    select: { height: 32, fontSize: 14, lineHeight: 20 },
    menu: { height: 30, fontSize: 14 },
  },
});
```

`inputBackground` supplies an exact fill, bypassing the native dark-mode mixing
rule. Omitted component metrics retain the native `ControlSize` recipes.

## Layout and paint

The following fields are validated in TypeScript and Rust, encoded by the
canonical Bebop schema, and applied by the native renderer:

| Capability              | API                                                                                                |
| ----------------------- | -------------------------------------------------------------------------------------------------- |
| Padding edges           | `paddingTop`, `paddingRight`, `paddingBottom`, `paddingLeft`                                       |
| Border edges            | `borderTopWidth`/`borderTopColor`, and corresponding right/bottom/left fields                      |
| Corner radii            | `borderTopLeftRadius`, `borderTopRightRadius`, `borderBottomRightRadius`, `borderBottomLeftRadius` |
| Wrapping                | `flexWrap: "nowrap" \| "wrap" \| "wrap-reverse"`                                                   |
| Proportional dimensions | `widthPercent`, `heightPercent` (50 means 50%)                                                     |
| Background gradient     | `linearGradient: { angle, stops: [{ color, position }, { color, position }] }`                     |

Edge and corner values override shorthands, including explicit zero. Percent and
pixel values for the same dimension are mutually exclusive. Percent dimensions
are relative to the containing layout block. Existing `flexGrow`, `flexShrink`,
`minWidth`, and `maxWidth` remain useful for proportional layout.

Gradients use degrees clockwise from up; 180 paints top-to-bottom. Colors accept
`#RRGGBB` or `#RRGGBBAA`. Stops are strictly increasing positions in [0, 1]. The
linked GPUI gradient primitive supports exactly two stops; arbitrary multi-stop
gradients are not included. Per-edge colors use four bounded native paths around
the original layout box, including corner arcs; they do not introduce flex items.

```tsx
<View style={{ position: "relative", height: 230, overflow: "hidden" }}>
  <Image source="assets/cover.png" objectFit="cover" style={{ widthPercent: 100, heightPercent: 100 }} />
  <View
    style={{
      position: "absolute",
      left: 0,
      right: 0,
      top: 0,
      bottom: 0,
      justifyContent: "flex-end",
      padding: 20,
      linearGradient: {
        angle: 180,
        stops: [
          { color: "#13121700", position: 0 },
          { color: "#131217FF", position: 1 },
        ],
      },
    }}
  >
    <Text style={{ color: "#FFFFFF" }}>Instance title</Text>
  </View>
</View>
```

For segmented controls, set the left button's right radii and the right button's
left radii to zero. For a two-column summary at arbitrary wide sizes, make each
row a wrapping flex row containing two `width: 0, flexGrow: 1, minWidth: 300`
items with `gap: 16`. This keeps two columns and wraps below the two-item minimum.
The runnable fixture demonstrates these patterns and single-edge separators.

## Offline application icons

Register a bounded catalog before starting the runtime **and before exporting
native bindings**:

```rust
use solid_gpui::icons::{register_icons, IconResource, IconColorMode};
register_icons(&[
    IconResource {
        name: "app:brand",
        svg: include_bytes!("../assets/brand.svg"),
        color_mode: IconColorMode::Original,
    },
    IconResource {
        name: "lucide:application-needed-icon",
        svg: include_bytes!("../assets/application-needed-icon.svg"),
        color_mode: IconColorMode::Monochrome,
    },
])?;
```

Use a real vendored Lucide SVG and its real name in an application. There is no
runtime Iconify request. Missing embedded files fail compilation; duplicate or
reserved names, malformed XML, unsupported SVG elements/attributes, external
resources, and oversized catalogs fail registration. The catalog is immutable,
limited to 256 entries, 64 KiB per SVG, 1024 elements per SVG, and bounded viewBox
and intrinsic dimensions. Built-in icons keep their existing allowlist.

`ComponentHost::native_bindings()` appends `applicationIcons` from the actual
registered catalog. Import that generated tuple and pass its names to `<Icon>`.
Registration and export use the same executable in development and production.
`registerIconNames` is also public for custom generators; the Rust host remains
an independent validator of names and resources.

## Grid and migration styles

`gridColumns` and `gridRows` define equal native tracks; `gridColumnSpan` and
`gridRowSpan` set item spans. Each accepts an integer from 1 through 64. A grid
container cannot also set `flexDirection`. Grid fields can be combined with
per-edge padding, borders, corner radii, and gradients. The canonical schema
assigns distinct field numbers to all of them; regenerate both language bindings
and golden vectors when the schema changes.

## Runnable fixture and verification

`examples/native-migration` is a runnable API fixture. Its pages,
layout, titlebar composition, router, and interactions are SolidJS. Rust provides
the native host and a process-lifetime service counter.

From the repository root:

```sh
bun install --frozen-lockfile
bun run build
bun run --cwd examples/native-migration dev
```

Production uses the same host and native contract:

```sh
bun run --cwd examples/native-migration build
bun run --cwd examples/native-migration start
```

The native binary launches Bun. Run it with the example directory as its working
directory. Distribute `dist/main.js` and `assets/cover.png` with the host; SVG icons
are embedded in the executable. Vite's native plugin generates `src/native.ts`
from this exact host. No runtime assets require network access.

Save a TSX change, introduce and repair a syntax error, then introduce and repair
an error directly inside `setup`. The previous generation remains mounted during
candidate failure. The root's surface and native window stay alive; click Start
game before and after reload to observe the same Rust service counter.
Component-local signals and native input state remount unless explicitly captured.
Router route state is captured by this fixture. Asynchronous router/page-loading
failures and `onMount` failures remain outside the synchronous rollback boundary.
QuickJS HMR is not part of this example.

### Integration verification (2026-09-08)

The merged workspace passes 220 Rust library tests, four cross-language protocol
tests, 37 migration/renderer tests, workspace TypeScript checks, and all five
website tests. The migration fixture's host and generated bindings compile.
The WASM release host and website production bundle build successfully with
`wasm-bindgen 0.2.121`. Website checks cover component examples, documentation
highlighting, and retained navigation. The native window interactions recorded
below were not manually repeated during this integration.

### Verification record (2026-09-07)

Environment: macOS 26.6.2 on Apple Silicon, Bun 1.4.2, Rust 1.98.1.

- `bun run check`: passed, including generated protocol/native contract checks,
  workspace builds, TypeScript checks, and workspace Clippy with QuickJS enabled.
- Core/router/HMR tests: 72 passed. Rust library tests with `gpui-component`:
  205 passed. Bidirectional protocol golden checks: passed.
- The example's TypeScript check and Vite production build passed.
- Native UI smoke tests used the example host in a temporary macOS app bundle.
  Initial content was 1100 × 720; resizing below the minimum stopped at 960 × 640.
  Both sizes retained the two-column summary and bottom action border.
- A single 48 px titlebar was visible, with traffic lights at the configured left
  inset and vertical center. Header input typing, input double-click, navigation,
  and native button clicks worked without zooming the window. Blank-area
  double-click zoomed and restored through the macOS window behavior. Fullscreen
  and restoration retained the titlebar/content layout. A blank-area drag was
  exercised; absolute screen displacement was not measured by the UI capture.
- The native input, button, tag, text, background, custom SVG, cover image,
  gradient, segment seam, and edge borders were visually inspected. No FPS
  overlay appeared in the debug application.
- A TSX title edit painted without input to wake the window. A syntax error and
  a synchronous render exception retained the last successful UI; restoring the
  source recovered in the same host process (PID 31772 during the final run).
  The Rust service counter advanced from 1 before HMR to 2 after recovery.
- The production JavaScript bundle ran in the same native host without Vite,
  rendered the local image and embedded icon, and invoked the Rust service.
  Closing both development and production windows exited the host successfully
  and shut down their Bun children.

This is a framework API smoke test, not an RMCL pixel-diff acceptance test.
RMCL was not modified. Select, dialog, menu, and TextView theme integration is
covered by the shared native theme implementation and automated checks, but
their entire state matrix was not visually exercised in this fixture. The
linked GPUI background primitive supports exactly two gradient stops; additional
stops are rejected explicitly. Linux/Windows window behavior and QuickJS HMR
were not manually verified in this delivery.
