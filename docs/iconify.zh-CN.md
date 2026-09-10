# Iconify 图标

使用 `@solid-gpui/core` 的 `Icon` 在 Solid GPUI 中显示内嵌的 Iconify 图标。
Rust 宿主通过 `gpui-iconify` 渲染 SVG 资源，无需浏览器图标组件、图标字体或运行时
Iconify API 请求。标准宿主内置有限的 Lucide 图标目录，可在桌面和 Web 离线使用。

## 基本用法

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

名称采用 Iconify 的 `collection:icon` 格式。只能使用内置或已注册的应用图标；
Iconify 完整图标库中的名称不会自动可用。无需安装额外的 JavaScript 图标包。

## 可用名称

GPUI Kit 的 `gpui-kit-assets` 是独立的内部控件资源包，通过 `icons/` 提供箭头、勾选、关闭等默认图形。
宿主只嵌入 `default-icons.txt` 中的资源，Web 也一样，不使用资源 CDN。
升级 Kit 不会向 `ICON_NAMES` 或应用图标目录添加名称。

应用图标归 `gpui-iconify` 和 `solid_gpui::icons`。内置 SVG 使用 `iconify/` 缓存键，
注册的应用 SVG 使用 `application-icons/`。`ApplicationIconAssets` 只处理这两个命名空间，不解析 Kit 路径。
应用通过 `collection:name` 使用图标，不直接使用内部缓存键。

生成组件的图标 slot 同样使用应用目录：

```tsx
import { Button, Icon } from "@solid-gpui/core/components";
<Button label="Save" icon="lucide:check" />;
<Icon source="lucide:check" />;
```

slot 接受已注册名称或显式 `{ svg: "…" }`，一次校验后保留原生图标，以单色字形着色。
品牌原色或调色板图标应放在普通子内容中，使用核心 `Icon`。
应用图标属性拒绝 Kit 内部的 `icons/*.svg` 路径，两个目录之间不做回退。

导入 `ICON_NAMES` 枚举内置目录，使用 `IconName` 为应用的图标选择添加类型。
常用名称包括 `lucide:search`、`lucide:settings`、`lucide:check`、
`lucide:moon` 和 `lucide:sun`。

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

SDK 在发送给宿主前验证名称，Rust 也会独立验证内嵌目录。
未知名称会报错，不会触发下载。请保持 SDK 绑定与宿主构建同步。

## 尺寸、颜色与布局

| 属性                 | 用法                                                             |
| -------------------- | ---------------------------------------------------------------- |
| `name`               | 必填的 `IconName`：内置名称或带类型的应用图标。                  |
| `size`               | 逻辑像素尺寸，默认为 `16`。必须为正数、有限且可用 float32 表示。 |
| `color`              | 单色图标的着色，例如 `"#2563EB"`，优先于 `style.color`。         |
| `style`              | 设置外层容器的间距和布局等样式。图标本身的大小由 `size` 控制。   |
| `accessibilityLabel` | 图标承载含义时使用的无障碍名称。                                 |
| `onLayout`           | 通过标准宿主回调接收布局测量结果。                               |

单色图标依次使用 `color`、`style.color` 和继承的文字颜色。
内置的多色与混合色图标保留原色，忽略着色。
`Icon` 不接受子元素或 `onPress`；需要点击操作时，将其放入 `Pressable`，
并为该控件设置有意义的无障碍名称。

## 响应式图标

在 JSX 中直接读取信号，Solid 会在状态变化时更新现有宿主节点的名称和颜色。
绘制由 GPUI 负责。

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

## 添加应用图标

需要内置目录之外的图标时，将 SVG 纳入应用源码，并在启动运行时和导出绑定前
在 Rust 应用中注册：

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

选择 `Original` 保留标志的原色，或使用 `Monochrome` 允许着色。
`ComponentHost::native_bindings()` 将注册名称导出为带类型的 `applicationIcons` 元组。
从生成的绑定模块导入它（下面的示例使用 `./native`），将条目传给 `Icon`：

```tsx
import { Icon } from "@solid-gpui/core";
import { applicationIcons } from "./native";

const [brand] = applicationIcons;
export function BrandIcon() {
  return <Icon name={brand} size={32} accessibilityLabel="Application logo" />;
}
```

元组按图标名称排序。目录变化后需要重新生成绑定。
`registerIconNames` 可供自定义绑定生成器使用，但仅注册 JavaScript 名称不会在宿主中内嵌 SVG。

目录限制与 SVG 验证规则见[原生应用迁移](native-migration.md#offline-application-icons)，
生成绑定的用法见 [Rust 集成](rust-bridge.md)。自定义图标需要在交付的宿主中注册；
面向浏览器时，也需要在自定义 Web 宿主中注册。
