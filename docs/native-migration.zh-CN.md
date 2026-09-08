# 原生应用迁移

本指南适用于来自同一源码版本的 Rust `solid-gpui`、`@solid-gpui/core` 和 `@solid-gpui/router` **0.3.0**。协议仍为 v5，但 schema 摘要已更新，Rust 宿主与 JavaScript 包必须一起部署。本次交付采用源码形式，并非 crates.io/npm 发布。

应用可以继续使用 `mountApplication`、Vite/Bun 热重载、类型化原生模块及路由 API，并拥有自己的原生宿主。迁移示例展示窗口配置、应用主题、边缘样式和内嵌图标。

网站路由为 `/docs/reference/native-migration`，浏览器使用 hash 路由。运行 `bun run website:native` 启动共享桌面网站。原生窗口装饰和进程服务需要桌面宿主，浏览器文档不会模拟操作系统窗口。

示例使用 `@solid-gpui/vite`：`bun run dev` 由 Vite 启动原生宿主，
`bun run dev:rust` 则由 Rust 通过 `runtime::vite::Vite` 启动开发。两者使用
同一份 Vite 配置和 native bindings，参见 [Vite 集成](vite.md)。

## 应用拥有的运行时与窗口配置

`run_application_with_profile(profile, Arc<dyn RuntimeAdapter>)` 不解析命令行，也不创建运行时。它复用 `run` 的宿主运行器：提交准入、原生事件、协议处理、Surface 所有权、覆盖层、窗口关闭和运行时最终关闭仍由框架管理。

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

窗口选项回调会在每个 Surface 打开时执行。需要保留渲染器请求的窗口边界时，应使用传入的选项。显式指定的标题、窗口类型、可调整大小和最小尺寸在回调之后应用。自定义配置可直接实现 `HostProfile::window_options`。

`TitleBar::window_options()` 提供透明 macOS 标题栏以及 `app_owns_titlebar_drag: true`。红绿灯位置指关闭按钮左上角：17 px 可将 14 px 按钮居中于 48 px 标题栏。宿主不会额外添加 Solid 标题栏，应用只应渲染一个：

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

实例样式覆盖 TitleBar 原生默认值，包括左内边距。全屏不会增加额外内边距。原生 TitleBar 处理空白区域拖拽与 macOS `titlebar_double_click`，遵循系统偏好。接管鼠标按下事件的控件不会触发标题栏拖拽或双击缩放；核心 `Pressable` 也会接管此原生默认动作。

性能监视器在所有构建中默认关闭。网站通过 `with_performance_monitor(true)` 显式开启，环境变量不会覆盖应用策略。

## 应用主题

生成的原生客户端提供以下接口：

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

`ApplicationThemeColors` 覆盖所链接组件库的完整纯色 token，包括悬停、激活、选中、输入、焦点、菜单、弹层、对话框、覆盖层、禁用和弱化颜色。通用 `primary` 与按钮专用 `buttonPrimary` 是不同 token；应用使用同一颜色时应同时设置。示例在原生 token 与 Solid 样式之间共享调色板。

每次调用替换先前的应用覆盖值，未指定项使用当前浅色或深色基础主题。覆盖值在模式或系统外观变化后保留。无效颜色、未知字段、无效字体名称或长度在变更前被拒绝。原生组件、Base 主题快照及 TextView 默认值一起更新。富文本在布局时解析继承的原生排版，不在内容缓存中保存默认黑色文本片段。

`components` 配置 `button`、`input`、`select`、`tag`、`menu` 和 `dialog` 的共享原生尺寸。每项接受逻辑像素单位的 `height`、`fontSize`、`lineHeight`、`paddingX`、`paddingY` 和 `radius`，在实例样式之前应用。

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

`inputBackground` 提供精确填充色，跳过原生深色模式混色规则。未指定的尺寸保留原生 `ControlSize` 配置。

## 布局与绘制

以下字段由 TypeScript 与 Rust 验证，通过规范 Bebop schema 编码，并由原生渲染器应用：

| 能力 | API |
| --- | --- |
| 各边内边距 | `paddingTop`、`paddingRight`、`paddingBottom`、`paddingLeft` |
| 各边边框 | `borderTopWidth` / `borderTopColor`，以及右、下、左对应字段 |
| 独立圆角 | `borderTopLeftRadius`、`borderTopRightRadius`、`borderBottomRightRadius`、`borderBottomLeftRadius` |
| 换行 | `flexWrap: "nowrap" \| "wrap" \| "wrap-reverse"` |
| 比例尺寸 | `widthPercent`、`heightPercent`，50 表示 50% |
| 背景渐变 | `linearGradient: { angle, stops: [{ color, position }, { color, position }] }` |

各边与圆角值覆盖简写，显式零值也生效。同一维度的百分比与像素值互斥；百分比相对包含布局块计算。`flexGrow`、`flexShrink`、`minWidth` 和 `maxWidth` 仍用于比例布局。

渐变角度从向上方向顺时针计算，180 表示自上而下。颜色接受 `#RRGGBB` 或 `#RRGGBBAA`，色标位置在 [0, 1] 内严格递增。链接的 GPUI 原语只支持两个色标，更多色标会被明确拒绝。各边颜色通过原始布局盒周围的四条有界原生路径绘制，包含圆角，不增加 flex 子项。

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

分段按钮可将左按钮右侧圆角和右按钮左侧圆角设为零。宽屏双列摘要可采用允许换行的 flex 行，包含两个 `width: 0, flexGrow: 1, minWidth: 300` 子项及 `gap: 16`；低于两项最小宽度时自动换行。可运行示例同时展示这些模式和单边分隔线。

## 离线应用图标

在启动运行时及导出原生绑定之前注册有界图标目录：

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

使用实际随应用保存的 Lucide SVG 及其名称，运行时不请求 Iconify。缺失内嵌文件会导致编译失败；重复或保留名称、格式错误的 XML、不支持的 SVG 元素或属性、外部资源及超大目录会导致注册失败。目录不可变，最多 256 项，每项 SVG 最大 64 KiB、1024 个元素，viewBox 与固有尺寸也有边界。内置图标保持现有允许列表。

`ComponentHost::native_bindings()` 从真实注册目录添加 `applicationIcons`。导入生成的元组，将其中的名称传给 `<Icon>`。开发与生产使用同一可执行文件注册和导出。自定义生成器也可使用公开的 `registerIconNames`；Rust 宿主仍独立验证名称和资源。

## 网格与迁移样式

`gridColumns` 和 `gridRows` 定义等宽原生轨道，`gridColumnSpan` 与 `gridRowSpan` 设置跨越数量。每项接受 1–64 的整数。网格容器不能同时设置 `flexDirection`。网格字段可与独立内边距、边框、圆角和渐变组合。规范 schema 为每项分配独立字段编号；修改 schema 后须重新生成双语言绑定和 golden 向量。

## 可运行示例与验证

`examples/native-migration` 是可运行 API 示例。页面、布局、标题栏组合、路由与交互使用 SolidJS；Rust 提供原生宿主和进程生命周期服务计数器。

在仓库根目录运行：

```sh
bun install --frozen-lockfile
bun run build
bun run --cwd examples/native-migration dev
```

生产构建使用相同宿主与原生契约：

```sh
bun run --cwd examples/native-migration build
bun run --cwd examples/native-migration start
```

原生二进制启动 Bun，工作目录应为示例目录。分发时将 `dist/main.js` 和 `assets/cover.png` 与宿主一起提供，SVG 图标已内嵌于可执行文件。Vite 原生插件从该宿主生成 `src/native.ts`，运行时资源不依赖网络。

保存 TSX 修改，依次引入并修复语法错误与 `setup` 内同步错误。候选版本失败时，上一代页面继续挂载，根 Surface 和原生窗口保留。重载前后点击 Start game，可观察同一 Rust 服务计数器继续递增。组件局部 signal 与原生输入状态会重新挂载，除非显式捕获；本示例捕获路由状态。异步路由、页面加载及 `onMount` 错误不属于同步回滚边界。此示例不包含 QuickJS HMR。

### 集成验证记录（2026-09-08）

英文源文档记录：合并工作区通过 220 项 Rust 库测试、4 项跨语言协议测试、37 项迁移及渲染器测试、工作区 TypeScript 检查和 5 项网站测试。迁移宿主及生成绑定可编译；WASM release 宿主与网站生产包使用 `wasm-bindgen 0.2.121` 构建成功。网站检查覆盖组件示例、文档高亮和导航保留。本次集成没有重新手动执行下列原生窗口交互。

### 原生验证记录（2026-09-07）

环境为 Apple Silicon、macOS 26.6.2、Bun 1.4.2、Rust 1.98.1。英文源文档记录了以下结果：

- `bun run check`、协议及原生契约生成检查、工作区构建、TypeScript 和启用 QuickJS 的 Clippy 通过。
- 核心、路由与 HMR 测试 72 项通过，启用 `gpui-component` 的 Rust 库测试 205 项通过，双向协议 golden 检查通过。
- 示例类型检查与 Vite 生产构建通过。
- 临时 macOS app bundle 的初始内容尺寸为 1100 × 720，最小尺寸为 960 × 640，两个尺寸都保留双列摘要及底部操作边框。
- 48 px 标题栏只显示一次，红绿灯位置正确；标题栏输入、双击输入、导航及原生按钮不触发窗口缩放。空白区双击缩放与恢复、全屏与恢复工作正常。执行了空白区域拖拽，但没有测量屏幕绝对位移。
- 检查了输入、按钮、标签、文字、背景、自定义 SVG、封面图、渐变、分段接缝及边缘边框；debug 应用没有 FPS 覆盖层。
- TSX 标题更新无需输入唤醒窗口；语法错误及同步渲染异常保留上一版 UI，修复后在同一宿主进程恢复。Rust 计数器从 1 增至 2。
- 生产 JavaScript 包不依赖 Vite，在同一原生宿主运行，显示本地图像与内嵌图标并调用 Rust 服务。关闭开发和生产窗口后，宿主退出并关闭 Bun 子进程。

这些检查覆盖框架 API，应用仍需执行自身的视觉与交互验收。Select、Dialog、Menu 和 TextView 主题由共享原生实现及自动检查覆盖，但没有对全部状态矩阵进行视觉验证。Linux/Windows 窗口行为与 QuickJS HMR 未在该交付中手动验证。
