# Web 宿主与 GitHub Pages

网站使用 Solid GPUI 组件与通用渲染器。Rust GPUI 将首页、文档和示例绘制到浏览器 canvas，不会把组件树转换为 DOM 元素。

## 构建与运行

在仓库根目录执行：

```sh
bun install --frozen-lockfile
rustup toolchain install nightly-2026-07-28 --profile minimal --target wasm32-unknown-unknown
cargo +nightly-2026-07-28 install wasm-bindgen-cli --version 0.2.121 --locked
bun run website
```

打开 `http://127.0.0.1:5173/solid-gpui/`。生成的 WASM 绑定位于 `examples/website/src/wasm/`，部署产物位于 `examples/website/dist/`，两者均不提交。浏览器的 `dev` 和 `build` 命令会先执行 `scripts/build-web-host.sh`，从同一份 Rust 源码生成 SDK 绑定并重建 WASM。Rust 或协议变化后重启开发命令；TypeScript/TSX 变化由 Vite 处理。直接调用 Vite 会跳过同步，可能将新组件绑定与旧宿主目录混用。

`build` 由两步组合而成，本地构建始终保持全自动：`build:host` 从当前 Rust 源码重新生成 SDK 绑定与 WASM 宿主，`build:frontend` 依次执行路由生成、类型检查和 Vite 打包。`dev` 以同样方式准备宿主后再启动 Vite。没有任何脚本会复用旧的宿主产物或下载宿主，每次本地构建都会基于工作区重新生成绑定与 WASM 模块。

默认 wasm-bindgen 版本不匹配时，用 `WASM_BINDGEN` 指定匹配的可执行文件。CLI 必须与 `Cargo.lock` 中的 `wasm-bindgen` 版本完全一致，较新的 CLI 不能替代。替换已安装版本时执行：

```sh
cargo +nightly-2026-07-28 install wasm-bindgen-cli --version 0.2.121 --locked --force
wasm-bindgen --version
bun run website:build
```

这会替换 Cargo 安装目录中的 CLI。如果其他项目需要不同版本，可使用 `--root <directory>` 安装，并在构建本项目时将 `WASM_BINDGEN` 指向 `<directory>/bin/wasm-bindgen`。

部署在域名根路径时设置 `PAGES_BASE_PATH=/`，默认前缀是 `/solid-gpui/`，所有本地资源都遵循该前缀。Hash 路由允许静态托管中的文档深链接刷新。

## 渲染与所有权

`crates/solid-gpui-web` 使用固定版本 gpui-pre 0.3.3 的单线程 Web 平台，不需要 SharedArrayBuffer 或跨源隔离响应头。上游 wasm_thread 依赖仍要求构建时使用指定 nightly。宿主嵌入获授权的 Inter、IBM Plex Sans、Lilex 和 Noto Sans SC 字体；GPUI 文本塑形无法使用浏览器系统字体。

`start()` 异步初始化图形并打开窗口；`submit()` 接收一个有大小限制、带帧的协议消息，在前台应用到现有 `SolidRoot`；`drain_events()` 在 Rust 更新回调外返回带帧原生事件。`WebTransport` 负责事件轮询与宿主释放。边界只传递字节，Solid 状态留在 JavaScript。

平台选择 WebGPU/WebGL 后端。`vendor/gpui-web` 和 `vendor/gpui-wgpu` 的本地补丁允许透明浏览器 surface，使原生 Hero 文字覆盖 GPU 画面。实际可用性取决于浏览器和 GPU，启动失败会在引导阶段报告。Web 宿主支持组件，但存在明确的平台限制：

- View、Text、Pressable、TextInput、Image、Icon 和核心 VirtualList 使用现有原生渲染器；Iconify 与组件 SVG 图标均在本地嵌入。
- 宿主注册共享 gpui-component Native Module，初始化主题、焦点以及 dialog、sheet、notification 层。`component-runtime` Cargo feature 排除桌面启动器和传输层。应用自身 Native Module 仍需显式注册。
- 系统动态效果偏好通过浏览器媒体查询订阅获取，并在释放时清理。原生与 Web 宿主共用覆盖策略。
- 网站将 `component-examples.ts` 中已允许的示例编译为真实 Solid 预览，`component-previews.ts` 定义启用范围。显示代码与执行预览共用来源。编辑器语法高亮、操作系统集成及未经验证的复杂示例不承诺 Web 预览支持。
- 阻塞原生命令会被拒绝；异步 native future 由 GPUI 轮询，并随所属请求取消。
- 桌面文件系统、进程、通知、菜单及多窗口服务不属于浏览器宿主契约。
- GPUI Web 的 canvas 无障碍和移动输入法支持受到上游平台限制。网站提供元数据，但 canvas 文档不等同于服务端渲染的可搜索 HTML。

## 网站

GPUI 应用挂载前，轻量 HTML 启动页显示 Solid GPUI 标志和不定进度加载指示器。样式表先于应用模块加载，启用“减少动态效果”时指示器保持静止。引导入口在加载应用前检查 WebAssembly 和图形 API。缺少必要 API，或 WebGPU 与 WebGL2 均初始化失败时，会显示响应式的“Browser not supported”卡片，提供更新浏览器、开启图形加速的建议，以及重试、外部 Markdown 文档和可展开的技术详情。该提示独立于 GPUI 运行时；仅缺少 WebGPU 不会阻止支持 WebGL2 的浏览器启动。其他启动错误使用独立的加载失败文案和同一无障碍提示布局；禁用 JavaScript 的浏览器会显示明确的启用提示。

网站图标、启动页和共享导航统一使用[项目确定的 Solid GPUI 图标](../assets/branding/README.md)。Vite 按 Pages 基础路径发布启动资源。导航图片以 data URL 嵌入浏览器和原生构建，切换页面无需请求图标文件。

网站提供持久保存偏好的中英文切换。首页、交互指南和参考文档按所选语言显示；英文文档与显式命名的 `.zh-CN.md` 中文副本分开维护。

代码片段通过原生 GPUI 富文本呈现语法高亮，并提供选择与复制。它是展示高亮，不是语言服务器。

Hero 将官方 [vgpu Optimized Black Hole 预览](https://vgpu.sh/preview/optimized-black-hole) 嵌入 GPUI 标题与按钮后方，渐变保持前景可读。该背景需要访问 vgpu 的网络连接及兼容 WebGPU 的浏览器。原生布局边界控制背景位置和裁切，背景不接收指针或键盘输入。切换页面、滚出视口或文档隐藏时释放 iframe。Hero 自动播放，不提供暂停控件。前景界面和交互示例仍使用 GPUI WASM，导航与动作图标来自内嵌 Iconify 资源。

## GitHub Pages

部署地址为 [cyenoch.github.io/solid-gpui](https://cyenoch.github.io/solid-gpui/)，仓库 About 的网站字段和根 README 均指向该地址。仓库发布来源已设为 **GitHub Actions**，`github-pages` 环境允许从 `main` 部署。

[Pages 工作流](../.github/workflows/pages.yml) 负责完整部署：

- 修改网站、文档、品牌资源、SDK、Rust 或构建输入后，推送到 `main` 会构建、测试并发布网站。具有这些变更的拉取请求只构建和测试；手动运行也仅从 `main` 发布。
- Ubuntu 安装导出 SDK 绑定所需的 Linux 原生库、仓库固定的 Rust 和 Bun 工具链，以及匹配的 Web nightly 和经校验和验证的预编译 wasm-bindgen CLI。`build:host` 重新生成原生绑定并构建 WASM 宿主，随后 `build:frontend` 检查网站类型并打包资源，最后运行网站测试。
- 资源前缀为 `/<repository-name>/`，本站为 `/solid-gpui/`。仅上传 `examples/website/dist`。部署任务获得 Pages 写权限和 OIDC 权限，无需个人访问令牌或后端。
- 新的拉取请求运行会取消过时检查。生产部署会等待正在执行的部署完成，最后通过 HTTP 检查确认发布页面可访问。

### Pages 缓存

该任务使用两个信任模型不同的缓存。

**生成的宿主**缓存同时保存 `build:host` 的两项产物：`examples/website/src/wasm` 和 `packages/solid-gpui/src/components.ts`。将生成的 SDK 目录与 WASM 一起恢复，即使导出器改写了 checkout 中的目录，冷、暖构建也使用相同的前端输入。其键是精确输入哈希且不配置 `restore-keys`，因此任何命中都是精确命中：还原的绑定与本次提交将构建的宿主完全一致。键覆盖工作流文件本身、Rust 环境配置动作、固定的工具链与 Bun 版本、Cargo 清单与锁文件、`.cargo/**`、全部 crate、vendored 的 GPUI Kit 与平台源码、第三方占位 crate、`scripts/build-web-host.sh`、生成 SDK 绑定的导出器，以及与该绑定配套提交的 `packages/solid-gpui/src/components.ts`。网站前端源码和 `docs/*.md` 被有意排除：它们只改变网站，不会改变宿主。每个不同输入键对应一个条目，GitHub 会回收长期未使用的条目，因此缓存只保留当前使用的输入集合，而不是每次提交各存一份；条目被回收后按冷路径重新构建即可。

命中时任务跳过 Linux 原生库、nightly 安装、Cargo 依赖缓存、wasm-bindgen CLI 和 `build:host`，但绝不跳过路由生成、类型检查、Vite 打包和网站测试，因此热运行仍会校验所有消费宿主的环节。没有对应条目的键——Rust、清单、导出器或生成契约发生变化——会以与无缓存运行相同的固定版本和命令重新构建宿主。`pages-web-host-v2` 是当前产物布局的命名空间；宿主构建新增输入时须提升该命名空间并扩展哈希闭包，确保为其他输入写入的条目绝不被复用。

**Cargo 依赖**缓存是 `.github/actions/setup-rust` 中的 `rust-cache` 条目。它按运行平台和构建用途区分键，跨源码提交复用，只由成功的非 PR 作业保存，且只在冷路径使用；其中只有中间编译产物，从不包含生成的绑定。它同样不构成跳过宿主构建的理由：只有精确输入的宿主缓存命中才会跳过构建，而该缓存只由已构建宿主、检查并打包网站、通过网站测试的非拉取请求运行写入。任何部分、失败或无关的缓存条目都不能替代该构建。

由于缓存键包含提交的 SDK 绑定，凡改变 `packages/solid-gpui/src/components.ts` 的宿主改动都需一并重新生成并提交。独立的 native CI 仍会验证提交的目录是否最新，为 Pages 恢复生成产物不会绕过这项新鲜度检查。

wasm-bindgen CLI 使用固定版本、经校验和验证的预编译程序。路径过滤、缓存和独立的手动原生打包工作流见[持续集成](ci.zh-CN.md)。

首次发布时，需要一并推送工作流、`examples/website`、`assets/branding`、Web 宿主、工作区依赖、锁文件、vendor 补丁和引用的文档。只有 YAML 文件无法完成构建。不要提交生成的 `dist`、`target` 或 `src/wasm` 目录。

工作流进入 `main` 后，可以在 **Actions → GitHub Pages → Run workflow** 手动重新部署，或执行：

```sh
gh workflow run pages.yml --repo Cyenoch/solid-gpui --ref main
```

本地构建不会发布网站。发布地址和部署结果见 GitHub Pages 工作流及其 `github-pages` 部署记录。

## 关键检查

```sh
bun --conditions=browser test examples/website/tests
bun run --cwd examples/website typecheck
bun run website:build
```

`bun run website:build` 是本地完整构建：`build:host` 重建宿主与生成的 SDK 绑定，随后 `build:frontend` 执行类型检查和打包。Pages 将这两步分开执行，并且无论 WASM 缓存状态如何，都会运行前端这一步和上述测试。

在真实浏览器中验证首页计数器、文档搜索与输入、代码选择和复制、语言持久化、Hero 背景与路由清理，以及宽窄屏布局。除 Vite 开发模式外，也要验证生产预览。
