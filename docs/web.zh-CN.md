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

打开 `http://127.0.0.1:5173/solid-gpui/`。生成的 WASM 绑定位于 `examples/website/src/wasm/`，部署产物位于 `examples/website/dist/`，两者均不提交。浏览器 `dev` 和 `build` 先执行 `build:host`，从相同 Rust 源码生成绑定并重建 WASM。Rust 或协议变化后重启开发命令；TypeScript/TSX 变化由 Vite 处理。直接调用 Vite 会绕过同步。

`build` 由两步组合而成，本地构建始终保持全自动：`build:host` 从当前 Rust 源码重新生成 SDK 绑定与 WASM 宿主，`build:frontend` 依次执行路由生成、类型检查和 Vite 打包。`dev` 以同样方式准备宿主后再启动 Vite。没有任何脚本会复用旧的宿主产物或下载宿主，每次本地构建都会基于工作区重新生成绑定与 WASM 模块。
`build:bindings` 运行原生目录导出器，`build:wasm` 运行 `scripts/build-web-wasm.sh`；Pages 可以分别执行这两个生产步骤。

默认 wasm-bindgen 版本不匹配时，用 `WASM_BINDGEN` 指定匹配的可执行文件。CLI 必须与 `Cargo.lock` 中的 `wasm-bindgen` 版本完全一致，较新的 CLI 不能替代。替换已安装版本时执行：

```sh
cargo +nightly-2026-07-28 install wasm-bindgen-cli --version 0.2.121 --locked --force
wasm-bindgen --version
bun run website:build
```

这会替换 Cargo 安装目录中的 CLI。如果其他项目需要不同版本，可使用 `--root <directory>` 安装，并在构建本项目时将 `WASM_BINDGEN` 指向 `<directory>/bin/wasm-bindgen`。

部署在域名根路径时设置 `PAGES_BASE_PATH=/`，默认前缀是 `/solid-gpui/`，所有本地资源都遵循该前缀。Hash 路由允许静态托管中的文档深链接刷新。

## 渲染与所有权

`crates/solid-gpui-web` 使用固定版本 gpui-pre 0.3.5 的单线程 Web 平台，不需要 SharedArrayBuffer 或跨源隔离响应头。上游 wasm_thread 依赖仍要求构建时使用指定 nightly。宿主嵌入获授权的 Inter、IBM Plex Sans、Maple Mono 和 Noto Sans SC 字体；GPUI 文本塑形无法使用浏览器系统字体。字体注册后，宿主通过 Kit 的 `Theme::update` 选择 Inter，在打开窗口前同步 Component 与 Base 排版。

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
- Ubuntu 按需安装原生绑定导出所需系统库、固定 Rust/Bun、Web nightly 与校验过的 wasm-bindgen。`build:bindings` 和 `build:wasm` 独立生成未命中的产物；随后始终执行 `build:frontend` 与网站测试。
- 资源前缀为 `/<repository-name>/`，本站为 `/solid-gpui/`。仅上传 `examples/website/dist`。部署任务获得 Pages 写权限和 OIDC 权限，无需个人访问令牌或后端。
- 新的拉取请求运行会取消过时检查。生产部署会等待正在执行的部署完成，最后通过 HTTP 检查确认发布页面可访问。

### Pages 缓存

两个产物缓存和一个依赖缓存各有独立职责。

**WASM 产物**缓存保存 `examples/website/src/wasm`。精确键覆盖工作流/Rust 设置 action、Cargo 清单/锁文件/配置、固定 Rust 工具链、所有 crate/vendor/第三方源码、嵌入资源、`scripts/build-web-wasm.sh` 和 wasm-bindgen 版本，不包含 Bun/package 锁文件或 TS 导出器。命名空间为 `pages-web-wasm-v1-bindgen-0.2.121`。

**SDK 绑定**缓存保存 `packages/solid-gpui/src/components.ts`。键在广泛原生输入外加入 Bun、包/格式化器/TS 配置、导出器源码与提交的目录，命名空间为 `pages-web-bindings-v1`。两个产物都不设置 `restore-keys`，各自未命中才执行对应生产步骤。TS 工具变化可以只生成绑定而不重编未变的 WASM；普通前端或 Markdown 修改均不使两个缓存失效，但始终构建并测试网站。

| WASM 缓存 | 绑定缓存 | 前端检查前执行的工作 |
| --- | --- | --- |
| 命中 | 命中 | 恢复两个产物；不设置 Rust、不编译宿主。 |
| 命中 | 未命中 | 安装原生前置依赖，仅生成绑定。 |
| 未命中 | 命中 | 安装 wasm-bindgen，仅构建 WASM。 |
| 未命中 | 未命中 | 安装两种生产步骤前置依赖，生成绑定与 WASM。 |

任一产物未命中时，在恢复 Cargo 依赖前安装同一固定 Web nightly，包括仅绑定未命中的情况。Cargo 缓存会哈希已安装工具链，不能让其集合随路径变化而分裂命名空间。所有路径均执行路由生成、类型检查、打包与网站测试；产物缓存仅在这些检查成功后保存，PR 不写入。生产步骤新增输入时扩展相应键并提升其命名空间。

**Cargo 依赖**缓存来自 `.github/actions/setup-rust`，按 runner 与用途区分，跨源码提交复用，成功的非 PR 作业才保存，任一产物未命中时使用。它只提供中间编译产物，不能作为跳过生产步骤的证据；只有该产物精确命中才可跳过，部分、失败或无关缓存都不能替代构建。

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
