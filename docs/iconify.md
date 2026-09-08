# Iconify

Use `Icon` from `@solid-gpui/core` to display embedded Iconify icons in Solid GPUI.
The Rust host renders the SVG assets through `gpui-iconify`; no browser icon
component, icon font, or runtime Iconify API request is needed. The standard host
ships a bounded Lucide catalog, available offline on desktop and Web.

## Basic usage

```tsx
import { Icon, Text, View } from "@solid-gpui/core";

export function DownloadLabel() {
  return (
    <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
      <Icon name="lucide:download" size={20} color="#2563EB" />
      <Text>Download</Text>
    </View>
  );
}
```

Names use Iconify's `collection:icon` format. Only built-in or registered
application names are accepted; a name from the full Iconify collection is not
automatically available. No additional JavaScript icon package is required.

## Available names

Import `ICON_NAMES` to enumerate the built-in catalog and `IconName` to type an
application's icon choices. Common names include `lucide:search`,
`lucide:settings`, `lucide:check`, `lucide:moon`, and `lucide:sun`.

```tsx
import { For } from "solid-js";
import { ICON_NAMES, Icon, View } from "@solid-gpui/core";

export function IconGallery() {
  return (
    <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 12 }}>
      <For each={ICON_NAMES}>{(name) => <Icon name={name} size={24} accessibilityLabel={name} />}</For>
    </View>
  );
}
```

The SDK validates names before sending them to the host, and Rust independently
validates its embedded catalog. Unknown names are errors; they do not trigger a
download. Keep SDK bindings and the host build synchronized.

## Size, color, and layout

| Prop                 | Usage                                                                                         |
| -------------------- | --------------------------------------------------------------------------------------------- |
| `name`               | Required `IconName`: a built-in name or a typed application icon.                             |
| `size`               | Logical pixel size; defaults to `16`. Must be positive, finite, and representable as float32. |
| `color`              | Tint for monochrome icons, for example `"#2563EB"`. Takes precedence over `style.color`.      |
| `style`              | Styles the wrapper, including spacing and layout. Use `size` to size the icon itself.         |
| `accessibilityLabel` | An accessible name when the icon conveys meaning.                                             |
| `onLayout`           | Receives layout measurements through the standard host callback.                              |

Monochrome icons use `color`, then `style.color`, then the inherited text color.
Palette and mixed-color built-in icons preserve their original colors and ignore
tint. `Icon` has no children or `onPress` prop; place it inside a `Pressable` for
an action and give that control a meaningful accessible label.

## Reactive icons

Read signals directly in JSX so Solid updates the existing host node's name and
color when state changes. GPUI owns the painting.

```tsx
import { createSignal } from "solid-js";
import { Icon, Pressable, type IconName } from "@solid-gpui/core";

export function PlaybackButton() {
  const [playing, setPlaying] = createSignal(false);
  const name = (): IconName => (playing() ? "lucide:pause" : "lucide:play");
  return (
    <Pressable accessibilityLabel={playing() ? "Pause" : "Play"} onPress={() => setPlaying((value) => !value)}>
      <Icon name={name()} size={24} color={playing() ? "#2563EB" : "#64748B"} />
    </Pressable>
  );
}
```

## Add application icons

For an icon outside the built-in catalog, vendor its SVG and register it in your
Rust application before starting the runtime and before exporting bindings:

```rust
use solid_gpui::icons::{register_icons, IconColorMode, IconResource};

register_icons(&[
    IconResource {
        name: "app:brand",
        svg: include_bytes!("../assets/brand.svg"),
        color_mode: IconColorMode::Original,
    },
])?;
```

Choose `Original` to preserve a logo's colors, or `Monochrome` to allow tinting.
`ComponentHost::native_bindings()` exports the registered names as the typed
`applicationIcons` tuple. Import it from your generated bindings module (the
example below uses `./native`) and pass an entry to `Icon`:

```tsx
import { Icon } from "@solid-gpui/core";
import { applicationIcons } from "./native";

const [brand] = applicationIcons;
export function BrandIcon() {
  return <Icon name={brand} size={32} accessibilityLabel="Application logo" />;
}
```

The tuple is sorted by icon name. Regenerate bindings after changing the
catalog. `registerIconNames` is available for custom binding generators, but
registering JavaScript names alone does not embed SVG assets in the host.

See [Offline application icons](native-migration.md#offline-application-icons)
for catalog limits and SVG validation, and
[Rust integration](rust-bridge.md) for generated bindings. Custom icons require
registration in the host you ship, including a custom Web host when targeting
the browser.
