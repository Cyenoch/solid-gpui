# 桌面应用示例

一个完整的 Solid GPUI 桌面应用：SolidJS 页面与布局、拥有窗口的 Rust 宿主，以及 Vite 打包器。它可以作为普通桌面应用的起点。[文档](../../docs/README.md)分别说明各项 API，本示例展示如何把它们组合成一个应用。

[English](README.md)

## 概述

- 一个原生窗口承载路由器驱动的 SolidJS 树：标题栏外壳加上 Home 与 Settings 两个路由。
- `desktop-app-host`（Rust）配置原生窗口与标题栏选项，在窗口打开前注册内嵌品牌图标，并暴露带 `serviceCount` 命令的 `desktop` 原生模块。
- `@solid-gpui/vite` 构建该宿主并把 bindings 导出到 `src/native.ts`，TypeScript 由此获得组件目录和 `serviceCount` 的签名。
- 渲染不依赖网络：封面图在运行时从磁盘读取，品牌图标编译进可执行文件。

## 运行示例

使用仓库固定版本的 Bun 和 Rust 工具链。各平台构建前提见[快速开始](../../docs/getting-started.zh-CN.md)。

共享 SDK 需要在仓库根目录安装并构建一次——`@solid-gpui/vite` 解析到构建产物 `dist/`，workspace 包也按名称链接：

```sh
bun install --frozen-lockfile
bun run build                 # workspace 包与二进制
bun run --cwd examples/desktop-app dev
```

`dev` 会启动 Vite、构建宿主、重新生成 `src/native.ts` 并打开窗口；保存 TSX 文件会在原地重载页面。

在本目录中也可以使用同一组命令：

```sh
bun run dev        # 由 Vite 构建并启动宿主，再把应用提供给宿主
bun run dev:rust   # 宿主优先：由宿主自己启动 Vite
bun run build      # 生成 dist/main.js
bun run start      # 用 --production 让宿主运行 dist/main.js
```

`dev` 与 `dev:rust` 从两端进入同一套开发环境：要么 Vite 启动宿主，要么宿主启动 Vite。生产模式下宿主在本目录运行，`bun dist/main.js` 因此能解析到 `assets/cover.png`。

## 示例展示的内容

### 首页布局与绘制

首屏让封面图用 `widthPercent` 与 `heightPercent` 填满固定高度的首屏区块，用 `linearGradient` 色标让渐变层淡入页面背景，并用带 `minWidth` 与 `flexGrow` 的卡片自动换行。单个按钮调用 `serviceCount` 并渲染返回值，因此 Rust 往返在界面上可见。见[布局与绘制](../../docs/native-composition.zh-CN.md#布局与绘制)。

### 有边界的设置页

Settings 路由由固定表头、可滚动的 14 行表单和固定表尾组成。滚动视口在受外壳约束的列布局中使用 `height: 0`、`flexGrow: 1` 和 `minHeight: 0`，因此只有中间区域滚动，最后一行始终可达。该页同时展示 Solid 的 `For`/`Show`，以及选项可加载与卸载、而应用持有的值保持不变的受控 `Select`。见[有边界的页面滚动](../../docs/scroll-performance.zh-CN.md#有边界的页面滚动)。

### 应用图标

`native/src/main.rs` 把 `assets/brand.svg` 编译进二进制并注册为 `desktop:brand`；生成的 `applicationIcons` 导出让标题栏的 `<Icon>` 能使用该名称。图标在窗口打开前注册，运行时不需要任何文件。见[添加应用图标](../../docs/iconify.zh-CN.md#添加应用图标)。

## 文件职责

| 路径                            | 职责                                                                                 |
| ------------------------------- | ------------------------------------------------------------------------------------ |
| `src/main.tsx`                  | 应用入口：主题、路由器、标题栏外壳、Home 与 Settings 路由、`mountApplication` 配置。 |
| `src/native.ts`                 | 由构建后的宿主生成。不要手改，应重新生成。                                           |
| `native/src/main.rs`            | Rust 宿主：窗口配置与标题栏、内嵌图标、`desktop` 原生模块、开发/生产运行时选择。     |
| `native/Cargo.toml`             | `desktop-app-host` crate 清单，workspace 成员。                                      |
| `vite.config.ts`                | Vite 根目录、`solidGpui` 插件选项，以及指向 SDK 源码的绝对别名。                     |
| `assets/cover.png`              | 运行时从磁盘读取的封面图，必须随应用一起分发。                                       |
| `assets/brand.svg`              | 编译进可执行文件的品牌图标。                                                         |
| `tsconfig.json`、`package.json` | 类型检查与上面的脚本。                                                               |

Vite 每次准备会话都会重新生成 `src/native.ts`，也可以单独生成：

```sh
cargo run --manifest-path native/Cargo.toml -- --export-native > src/native.ts
```

（`--export-native` 直接打印 bindings；插件还会额外格式化。）

## 打包

分发 `desktop-app-host`、`dist/main.js` 和 `assets/cover.png`，保留相对路径，并以本目录作为宿主工作目录。此示例启动外部 Bun 进程，因此 `PATH` 中必须能找到 `bun`。品牌图标已内嵌，两种图像都不需要网络请求。原生依赖、应用包和签名见[桌面分发](../../docs/distribution.zh-CN.md)。

## 延伸阅读

- [桌面宿主配置](../../docs/rust-bridge.zh-CN.md#桌面宿主配置)与[窗口选项与标题栏](../../docs/rust-bridge.zh-CN.md#窗口选项与标题栏)，对应 `native/src/main.rs` 中的宿主与窗口设置。
- [应用主题覆盖](../../docs/gpui-components.zh-CN.md#应用主题覆盖)，说明挂载后如何应用 `theme`。
- [原生网格](../../docs/native-composition.zh-CN.md#原生网格)，介绍网格布局字段。
- [Rust 组件与 JavaScript 调用](../../docs/rust-bridge.zh-CN.md)，介绍原生模块、命令与 DTO。
