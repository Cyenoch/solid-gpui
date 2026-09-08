# Vite 集成

Solid GPUI 支持两种编写方式：直接写 JavaScript 并运行，或者写 JSX/TSX，
交给 Vite 编译。Bun 和 QuickJS 是执行 runtime；Vite 是唯一支持的应用打包器。
不提供 TSX 直接运行命令、内置打包器或 Bun compile 路径。

## 不用打包器的 JavaScript

用 `createComponent` 和响应式 getter 代替 JSX：

```js
import { mountApplication, Text } from "@solid-gpui/core";
import { createComponent, createSignal } from "@solid-gpui/core/runtime";
import { StdioTransport } from "@solid-gpui/core/stdio";

mountApplication({
  transport: () => new StdioTransport(),
  setup() {
    const [message] = createSignal("Hello from JavaScript");
    return { render: () => createComponent(Text, { children: message }) };
  },
});
```

保存为 `app.js`，安装 SDK 和 Solid，由原生宿主执行：

```sh
solid-gpui-host bun --conditions=browser app.js
```

宿主可执行文件需要事先安装或构建；不在 `PATH` 时使用明确路径。
宿主管理 Bun 子进程和协议 stdio。`browser` condition 选择 Solid 的响应式实现，
不提供 DOM，也不禁用 Bun API。Bun 可以解析 npm 导入，使用 `Bun.file`、
`bun:sqlite`、Node 内置模块、Worker 等所选版本提供的能力。

QuickJS 也可以通过 `QuickJsAdapter::start` 或 `QuickJsAdapter::from_source`
直接执行 JavaScript，但当前加载器只接受一个自包含 ES 模块，不能解析 npm 包
或外部导入。直接执行不会隐式编译或打包；依赖需要打包时使用 Vite。
QuickJS 使用 `EmbeddedTransport`，宿主需要启用 `quickjs`。

## 用 Vite 编译 JSX 和 TSX

安装 `@solid-gpui/core` 和 `solid-js`，将 `@solid-gpui/vite` 与 Vite 8 加入开发依赖。原生开发工具运行在 Bun 1.4.2+
下。在 `package.json` 中设置 `"type": "module"`，创建 `vite.config.ts`：

```ts
import { defineConfig } from "vite";
import { solidGpui } from "@solid-gpui/vite";

export default defineConfig({
  plugins: [solidGpui({ entry: "src/app.tsx", runtime: "bun" })],
});
```

入口显式调用 `mountApplication`；外部 Bun 使用 `StdioTransport`。
TypeScript 设置 `jsx: "preserve"` 和 `jsxImportSource: "@solid-gpui/core"`。
使用 `import.meta.hot` 时加入 Vite client 类型。

```sh
bun --bun vite
bun --bun vite build
solid-gpui-host bun --conditions=browser dist/app.js
```

`vite` 默认启动 `solid-gpui-host`。已有自定义宿主可设置
`host: { command: "/path/to/my-host", args: [] }`。宿主需要使用 Solid GPUI
宿主入口或 Rust `Vite` helper 接收受管理的 renderer。不会隐式下载或编译宿主。

Vite 管理配置、别名、虚拟模块、转换、监听和构建。Bun 开发时，ModuleRunner
位于宿主管理的 Bun 子进程中；独立且经过验证的 loopback 通道传递模块与 HMR，
原生协议继续使用 stdio。该子进程的 console 输出转到 stderr。插件自动为入口
加入 HMR acceptance；显式状态保存见[热重载](hot-reload.md)。

标准 console 日志（包括 `console.dir` 和 `console.table`）输出到 stderr。原始 stdin/stdout API（包括 `Bun.stdin`、`Bun.stdout` 和 `console.write`）由原生协议使用；其他 I/O 使用 stderr 或应用自己的文件与 socket。

生产构建默认打包 JS 依赖，保留 Bun/Node 内置导入供 Bun 执行。
需要随应用分发的外部包可通过 Vite `ssr.external` 指定。
构建不会安装 runtime、打包原生动态库或签名应用。

## 应用自己的 native 模块

通过 `#[native_module]` 注册 Rust 命令和组件，提供 `--export-native` 入口，
再让 Vite 使用同一个宿主：

```ts
solidGpui({
  entry: "src/app.tsx",
  runtime: "bun",
  native: {
    manifestPath: "native/Cargo.toml",
    bin: "my-app",
    output: ".generated/native.ts",
  },
});
```

路径相对于 Vite root 解析。`package` 和 `features` 是可选 Cargo 参数。
插件从 Cargo artifact 消息取得实际可执行文件，包括自定义 target 目录；
多个候选会报错。随后运行该文件的 exporter，原子写入 bindings，建立 `#native`
别名，再加载应用。内容未变化时不重写文件。TypeScript `paths` 需要指向同一文件。

用 `native` 或 `host` 选择宿主；`native` 已经负责选择和构建。
首次 Rust 构建不能依赖尚待生成 bindings 的 JS bundle。修改 Rust 契约后重启
开发；Vite 配置重启会先关闭旧环境，再准备新宿主与 bindings。

Native 调用不依赖打包器。直接 Bun JS 应用可以导入宿主导出的 `native.ts`，
因为 Bun 自身支持加载 TypeScript；无需 Vite 或 `#native` 别名。
两种编写方式使用相同的 `createClient(root)`、`useNative()`、身份、命令分发、
取消和校验契约。见 [Rust bridge](rust-bridge.md)。

## Rust 主导的 Vite 开发

```rust
use solid_gpui::runtime::vite::Vite;

let runtime = Vite::new("path/to/frontend")
    .config_file("vite.config.ts")
    .spawn()?;
solid_gpui::run_application(app::native_module(), runtime);
```

省略 `config_file` 使用 Vite 默认配置发现。前端目录必须安装 npm 依赖。
需要设置环境变量时，`command()` 返回普通 `std::process::Command`。
helper 设置 Bun 的 client condition 和协议日志规则，并从当前原生可执行文件
导出 bindings，避免递归构建。启动 runtime 前必须处理 `--export-native`。

由 Vite 启动该 Rust 应用时，同一 helper 会连接既有模块通道，不再启动第二个
Vite server。它返回 Bun `ProcessAdapter`，不是 QuickJS adapter。
直接运行生产 JS 时仍使用 `ProcessAdapter`、`QuickJsAdapter` 或嵌入式 Bun adapter。

JS 测试程序自行管理 runner 时可设置 `host: false`，再通过 Vite 的 runnable
`ssr` 环境导入模块。显式设置 Vite `dev.createEnvironment` 时也由调用方管理。

## QuickJS 开发和构建

设置 `runtime: "quickjs"`，入口使用 `EmbeddedTransport`，选择启用了 `quickjs`
feature 的宿主。使用 native 模块时，在 `native` 中设置 `features: ["quickjs"]`。

```sh
bun --bun vite
bun --bun vite build
my-host --runtime quickjs dist/app.js
```

Vite 生成一个自包含 ESM，初始化明确的 QuickJS UI 平台，拒绝 Bun/Node 导入、
残留动态导入和外部资源文件。使用内联资源或宿主管理的文件。QuickJS 不提供
Bun API、Node 服务、DOM 或网络 `fetch`；这些能力应由 native 模块提供。

开发使用 Vite build watcher，保留配置中的插件和别名，再把成功产物交给 Rust
generation supervisor。替换 VM 时保留 Rust 服务和原生窗口；构建失败保留当前
应用。QuickJS 内部没有 ModuleRunner，也没有第二个打包器。
遵守工作区的解释器优化配置和[状态契约](hot-reload.md#captured-state)。

组件宿主可以在 SolidRoot 外包装 provider 根节点；QuickJS 重载使用已保存的 renderer entity 验证快照，不要求窗口最外层就是 SolidRoot。

实验性浏览器宿主使用 `solidGpui({ target: "web" })`。它应用同一套 universal
JSX 转换，保留 Vite 的 HTML、client 环境和浏览器构建配置。见 [Web 宿主](web.md)。
