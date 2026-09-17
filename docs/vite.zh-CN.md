# Vite 集成

Solid GPUI 支持两种编写方式：直接写 JavaScript 并运行，或者写 JSX/TSX，
交给 Vite 编译。Bun 和 QuickJS 是执行 runtime；Vite 是唯一支持的应用打包器。
不提供 TSX 直接运行命令、内置打包器或 Bun compile 路径。

`@solid-gpui/vite` 是工具包：提供本 Vite 插件、`solid-gpui` CLI（`prepare`、
`preview`、`doctor`、`test`）、`@solid-gpui/vite/test` 运行器，以及
`@solid-gpui/vite/artifacts`、`@solid-gpui/vite/project` 辅助模块。
完整流程见[入门](getting-started.zh-CN.md)；本篇讲选项面与进阶路径。

过程中会产出三类不同的东西，只有第三类是交付物：

| 产物               | 产出方式                                    | 内容                                                                                                                                        |
| ------------------ | ------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| **绑定**           | `solid-gpui prepare`（`bun run generate`）  | 宿主导出的组件目录、命令与类型，以及生成的 TypeScript 工程。                                                                                |
| **bundle**         | `bun --bun vite build`                      | 供宿主执行的单个 JavaScript 入口模块。构建同时会 prepare 所配置的原生宿主（增量 Cargo 构建），但不会把原生代码写进 bundle，也不产出安装包。 |
| **原生可执行文件** | Cargo（由插件 `native` 或你自己的构建触发） | 渲染 bundle 的 GPUI 宿主。                                                                                                                  |
| **可分发包**       | 你自己的打包脚本                            | 可执行文件加 bundle、资源、许可与签名。见[分发](distribution.zh-CN.md)。                                                                    |

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

## 工程初始化

安装 `@solid-gpui/core` 和 `solid-js`，将 `@solid-gpui/vite` 与 Vite 8 加入开发依赖。这些包尚未发布到公共 registry：请用同一份固定的 SDK checkout 执行 `bun run task sdk-pack <dir>`，并安装同一批次的 `solid-gpui-core.tgz` 与 `solid-gpui-vite.tgz`（见[入门](getting-started.zh-CN.md#1-安装)）。只有在包正式发布后才按名称安装；目前按名称安装什么都解析不到。原生开发工具运行在 Bun 1.4.2+
下。在 `package.json` 中设置 `"type": "module"`，创建 `vite.config.ts`：

```ts
import { defineConfig } from "vite";
import { solidGpui } from "@solid-gpui/vite";

export default defineConfig({
  plugins: [solidGpui({ entry: "src/app.tsx", runtime: "bun" })],
});
```

入口显式调用 `mountApplication`；外部 Bun 使用 `StdioTransport`。
`runtime` 必须显式指定，且从不推断或转换：`"bun"` 把 Bun/Node 导入保留为运行时
导入，`"quickjs"` 生成单个自包含 ES 模块并拒绝这些导入。

`solidGpui()` 同时设置 `resolve.dedupe: ["solid-js"]`，因此 SDK 与应用共享同一个 Solid 实例；
不要再加入第二份 `solid-js`，也不要手写 dedupe 条目。`solidGpuiSource()` 既可从
`@solid-gpui/vite/source` 导入，也从包根导出。

脚本只需添加一次，安装依赖后运行生成器。不要把脚本命名为 `prepare`：Bun 会在
安装期间运行根包的 `prepare`，而安装过程绝不能编译原生宿主。

```json
{
  "scripts": {
    "generate": "solid-gpui prepare",
    "check:generated": "solid-gpui prepare --check",
    "doctor": "solid-gpui doctor",
    "dev": "bun --bun vite",
    "build": "bun --bun vite build",
    "preview": "solid-gpui preview",
    "test": "solid-gpui test"
  }
}
```

`prepare` 写入三个生成文件：`.solid-gpui/tsconfig.json`（把 `#native` 和
`@solid-gpui/core/components` 映射到插件实际选中的绑定）、绑定文件
（`native.output`，默认 `.generated/native.ts`）和 `.solid-gpui/artifacts.json`
（解析后的产物记录）。请把 `.solid-gpui/` 排除在版本控制之外；绑定文件可以提交，
此时用 `solid-gpui prepare --check` 作为 CI 新鲜度门槛。`prepare --check`
不写入任何内容（它仍会解析并构建宿主以便与实际目录比对），并在任一文件过期时失败；
`prepare --json` 输出产物记录。工程命令接受 `--root` 和 `--config`。
`prepare`、`preview` 支持 `--mode`（默认 `production`）；`test --mode`
默认使用 Vite 的 `development`。`doctor` 不支持选择 Vite mode。
`preview`、`test` 中 `--` 之后的参数属于宿主或 Bun，不再由本 CLI 解析。

你自己的 `tsconfig.json` 只需继承生成工程：

```json
{
  "extends": "./.solid-gpui/tsconfig.json",
  "compilerOptions": {
    "strict": true,
    "jsx": "preserve",
    "jsxImportSource": "@solid-gpui/core"
  },
  "include": ["src", "vite.config.ts"]
}
```

不要手工维护 SDK 的 `paths`、`baseUrl` 或 `customConditions`：生成工程与 Vite
别名来自同一次解析。使用 `import.meta.hot` 时加入 Vite client 类型。

`solid-gpui doctor` 报告环境与依赖问题：当前生效的 Cargo 依赖、profile、工作区根
必须携带的 `[patch.crates-io]` 条目，以及不支持的 runtime/target 组合。Cargo 会忽略
依赖内声明的 profile，因此消费方工作区根必须自行声明；见[应用构建配置](hot-reload.zh-CN.md#应用构建配置)
与[故障排查](troubleshooting.zh-CN.md)。

## 用 Vite 编译 JSX 和 TSX

```sh
bun --bun vite
bun --bun vite build
bun run preview          # 用构建好的宿主运行构建好的 bundle
```

`vite` 默认启动 `solid-gpui-host`。已有自定义宿主可设置
`host: { command: "./target/debug/my-host", args: [], output: ".generated/native.ts" }`。
Vite 在加载组件导入前以 `--export-native` 运行该命令，将 `#native` 和
`@solid-gpui/core/components` 指向导出的文件。`command` 以 Vite root
作为工作目录启动，因此相对可执行文件路径按该 root 解析。`output` 默认是相对
Vite root 的 `.generated/native.ts`。即使没有自定义 native 模块，内置控件也必须使用实际
宿主的精确目录。宿主必须实现 `--export-native`，并使用 Solid GPUI 宿主入口或
Rust `Vite` helper 接收受管理的 renderer。不会隐式下载或编译宿主。

`host.args` 放在 `--export-native` 之前，因此解释型 exporter 可配置为
`{ command: "bun", args: ["host.ts"] }`。Rust 宿主导出不接受额外启动参数。
未显式配置的默认宿主使用由标准 `solid-gpui-host` 生成的 SDK 目录，两端必须保持
相同版本。`host: false` 不选择原生契约，解析由调用方负责。

Vite 管理配置、别名、虚拟模块、转换、监听和构建。Bun 开发时，ModuleRunner
位于宿主管理的 Bun 子进程中；独立且经过验证的 loopback 通道传递模块与 HMR，
原生协议继续使用 stdio。该子进程的 console 输出转到 stderr。插件自动为入口
加入 HMR acceptance；显式状态保存见[热重载](hot-reload.zh-CN.md)。

标准 console 日志（包括 `console.dir` 和 `console.table`）输出到 stderr。原始 stdin/stdout API（包括 `Bun.stdin`、`Bun.stdout` 和 `console.write`）由原生协议使用；其他 I/O 使用 stderr 或应用自己的文件与 socket。

生产构建默认打包 JS 依赖，保留 Bun/Node 内置导入供 Bun 执行。
需要随应用分发的外部包可通过 Vite `ssr.external` 指定。
配置了 `native` 时构建也会 prepare 宿主，因此 `generate` 会重建的内容在这里同样会
编译；未配置 `native` 时不构建任何宿主。构建不会安装 runtime、打包原生动态库或签名应用；
这些工作属于[分发](distribution.zh-CN.md)。

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
多个候选会报错。随后运行该文件的 exporter，原子写入 bindings，将 `#native`
和 `@solid-gpui/core/components` 都指向该文件，再加载应用；Motion 也使用同一
宿主的组件目录。内容未变化时不重写文件。

### native 构建选项

| 选项             | 默认值                 | 含义                                                         |
| ---------------- | ---------------------- | ------------------------------------------------------------ |
| `manifestPath`   | —                      | 宿主的 Cargo manifest，必填。                                |
| `package`、`bin` | —                      | Cargo 选择器；manifest 中存在多个二进制时两者都需要。        |
| `features`       | —                      | Cargo feature，例如 `["quickjs"]`。                          |
| `output`         | `.generated/native.ts` | 绑定输出位置，相对 Vite root。                               |
| `profile`        | `"dev"`                | Cargo profile；接受 `"debug"` 作为别名。                     |
| `target`         | —                      | 交叉构建的 Cargo `--target` triple。                         |
| `locked`         | `true`                 | 传递 `--locked`。仅在确实需要首次解析时设为 `false`。        |
| `check`          | `false`                | 校验模式：绑定过期时报错而不写入。                           |
| `watch`          | —                      | 额外触发原生重建的文件或目录。                               |
| `exporter`       | —                      | 当 `target` 无法在本机执行时，用于导出绑定的宿主可执行文件。 |

`prepare` 与开发都需要本机可运行的宿主。当 `target` 指向与构建机不同的平台或架构时，
两者都会拒绝执行并提示：为 prepare/开发宿主去掉 `target`，或传入 `exporter`；
`--export-native` 绝不会在异构 target 上运行。交叉构建产物由 Cargo 在打包时生成，
其路径来自产物记录。

用 `native` 或 `host` 选择宿主；两者都从实际可执行文件导出 bindings，
`native` 还负责构建和监听 Cargo 项目，不要求声明应用自己的 native 模块。
只注册内置组件的应用宿主同样可以使用 `native`。首次 Rust 构建不能依赖尚待
生成 bindings 的 JS bundle。开发时，Rust 源码、Cargo
manifest、lockfile 和工作区 Cargo 配置变化会自动重建宿主与 bindings，也覆盖
本地路径依赖；`native.watch` 可补充 Cargo 无法推断的文件或目录。Vite 先停止旧
runtime，再发布 bindings 并启动新宿主。编译或首次
加载失败会继续监听；修复并保存即可重试。关闭原生窗口后也会保留监听，Ctrl+C
结束开发。监听范围、宿主替换的状态丢失和生命周期归属见[开发会话管理](hot-reload.zh-CN.md#开发会话管理)。

Native 调用不依赖打包器。直接 Bun JS 应用可以导入宿主导出的 `native.ts`，
因为 Bun 自身支持加载 TypeScript；无需 Vite 或 `#native` 别名。
两种编写方式使用相同的 `createClient(root)`、`useNative()`、身份、命令分发、
取消和校验契约。见 [Rust bridge](rust-bridge.zh-CN.md)。

## 产物与工程解析

`.solid-gpui/artifacts.json` 是权威记录：`root`、`runtime`、`entry`、`outDir`、
`bindings`、`tsconfig`、存在时的 `bundle`、选中的 `host`（`command`、`args`、`output`），以及
`native` 构建设置（`manifestPath`、`package`、`bin`、`features`、`profile`、
`profileDirectory`、`target`、`locked`、`targetDirectory`、`executable`）。
`outDir` 是配置的 Vite 构建输出目录（绝对路径），也是本次构建产物归属的稳定标识；`bundle` 必定位于其中。
`profileDirectory` 把 `dev` 映射为 `debug`、`release` 映射为 `release`，其他 profile 保留自身名称；
`target` 是实际生效的 Cargo target，包含隐式的 `CARGO_BUILD_TARGET` 或 `.cargo/config.toml`
中的 `build.target`。`prepare` 写入的 `bundle` 只是依据 Vite 配置做出的预判（字符串形式的
`entryFileNames` 模式会保留占位符），生产构建会把它替换为 Vite 实际产出的入口，该入口可能位于子目录
（例如使用 output 函数或 `assets/[name].js` 模式）；因此打包脚本或 `preview` 需要真实路径时应在
`vite build` 之后读取该记录，绝不要从配置推导。读取要求 `root`、`runtime`、`entry`、`outDir`、
`tsconfig` 存在；由其他版本写入或损坏的记录会给出明确提示，要求重新运行 `solid-gpui prepare`。
请读取它，而不是自行拼出 `target/<profile>/<bin>` 或 bundle 路径：

```ts
import { readNativeArtifacts, prepareProject } from "@solid-gpui/vite/artifacts";

const record = await readNativeArtifacts(process.cwd()); // 或 (await prepareProject()).artifacts
const executable = record?.native?.executable ?? record?.host?.command;
```

`prepareProject` 解析工程并以 `artifacts` 返回同一记录；`cargoProfileDirectory(profile)`
把 Cargo profile 名映射到输出目录（`dev` → `debug`、`release` → `release`，
其余为 profile 名本身）。`recordedHostExecutable(record)` 返回 Cargo 实际构建出的宿主，
是启动或打包你所构建 profile 的权威路径；`expectedExecutablePath(record, profile?)` 只是
推测另一个 profile 的可执行文件位置，交付前请先验证或先构建该 profile。`@solid-gpui/vite/project` 的
`previewApplication({ root?, configFile?, mode?, args?, env? })` 用已构建的宿主运行已构建的
bundle 且不重建，`solid-gpui preview` 命令与 `preview` 脚本正是它的封装。
若配置按 mode 选择入口或 native Cargo profile，构建与预览必须使用同一个 mode，
例如先执行 `vite build --mode release`，再执行 `solid-gpui preview --mode release`。
预览先按该 mode 解析配置，再检查产物记录，不会静默替换 profile。

## 测试

`solid-gpui test` 委托给 `@solid-gpui/vite/test`：

```sh
bun run test                       # 全量
bun run test src/app.test.tsx      # 单个文件
```

```ts
import { runTests } from "@solid-gpui/vite/test";

const exitCode = await runTests({ root: process.cwd(), configFile: "vite.config.ts", args: ["src/app.test.tsx"] });
```

`runTests({ root?, configFile?, mode?, args? })` 返回进程退出码，并使用应用自己的
`vite.config.ts`：真实的 JSX 转换、alias、`?inline` 资源、去重后的 `solid-js`
以及原生绑定契约。`args` 原样转发给 `bun test`，因此文件过滤与所有 `bun:test` 标志都可用。
运行器会自行加入 `browser` condition，测试无需传 `--conditions=browser`。整个测试套件运行在
同一个 Bun 进程和同一个 Solid 客户端 runtime 中，而打包后的应用加载的正是这个 runtime，
因此测试与应用共享同一个响应式图。Bun 测试运行器保持其语义，
普通单文件测试与 `bun:test` API 照常可用。测试无需导入
`dist` 私有模块、复制编译器或自建 Vite 流水线。

测试在 Bun 中运行，保留真实 `process.env`、对它的修改以及子进程继承的环境，
即使应用目标为 QuickJS 也如此。此覆盖仅限测试，不会把 Node/Bun 能力或环境变量
开放给生产 QuickJS bundle。`mode` 选择应用的 Vite 配置、alias、define 及
`.env.<mode>` 加载，不切换测试 runtime，也不额外设置 `NODE_ENV`。
省略时保留 Vite 的 `development` 默认值。可使用 `solid-gpui test --mode staging`
或 `runTests({ mode: "staging" })`。

### 无需导入私有协议即可检查渲染结果

`@solid-gpui/core/testing` 的 `TestHost` 检查 `MemoryTransport`，把 Snapshot、Patch
帧重放为按原生顺序排列、与后续修改隔离的树视图：

```tsx
import { expect, test } from "bun:test";
import { createRoot, Pressable, Text } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { TestHost } from "@solid-gpui/core/testing";

test("press updates the committed text", () => {
  const host = new TestHost();
  const root = createRoot(host.transport, { surfaceId: 1 });
  try {
    root.render(() => {
      const [count, setCount] = createSignal(0);
      return (
        <Pressable onPress={() => setCount(count() + 1)}>
          <Text>{count()}</Text>
        </Pressable>
      );
    });
    const button = host.surface(1)!.nodes.find((node) => node.kind === "Pressable")!;
    host.dispatch(button, { type: "press" });
    expect(host.surface(1)!.nodes.some((node) => node.text === "1")).toBe(true);
  } finally {
    root.unmount();
  }
});
```

| 接口                                           | 行为                                                                                                                                                                                                             |
| ---------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `new TestHost(transport?)`                     | 使用传入的 `MemoryTransport`（包括已有帧），或新建一个。                                                                                                                                                         |
| `surface(id)`                                  | 返回最新提交的 Surface；首个 Snapshot 之前返回 `undefined`。`nodes` 按前序排列，包含合成根节点；节点提供 kind、父节点/有序子节点 ID、文本、输入值、placeholder、无障碍标签和 tooltip。旧视图不会随后续提交变化。 |
| `commits`                                      | 按提交顺序返回 Snapshot/Patch 的类型、Surface、epoch 和 revision；wire tag 与更新掩码保持私有。                                                                                                                  |
| `dispatch(node, event)`                        | 发送 `press`、`focus`、`blur`、`input`（`text` 及可选的 UTF-8 字节选区偏移），或 `native`（`eventId`、JSON `value`）。保留所捕获节点的 revision/epoch，可验证过期事件处理。                                      |
| `nativeProps(node)`                            | 解码 `createNativeComponent` 节点的 JSON DTO props。                                                                                                                                                             |
| `nativeCalls`                                  | 查看模块函数与组件方法请求：Surface、epoch、节点/请求 ID、模块身份、函数 ID 和原始 `args` 字节。生成的 DTO 调用用 `@solid-gpui/core/native` 的公开 `decodeJson` 解码。                                           |
| `reply(call, bytes)` / `reject(call, message)` | 经真实事件路径完成对应请求；JSON DTO 回复使用 `encodeJson(value)`。允许乱序回复，但不能重复回复或交给另一 TestHost 回复。                                                                                        |

读取只消费已提交的帧。应用有调度任务时，先等待任务再检查新提交；事件分发之外的
同步 signal 修改通常需要 `await Promise.resolve()`。Native client 也会等当前
Solid batch 完成后才提交请求。该辅助接口管理注入事件的序号；不要混用手工编码的
事件或清空 `transport.submitted`。每个测试使用独立 host，并在清理时卸载 root。
它不计算原生样式/布局、不绘制像素、不执行 Rust handler，也不模拟平台服务；
这些行为仍须用真实宿主验证。

## 从源码消费包

安装后的包默认解析到构建好的 `dist`，普通用法无需额外配置。要调试或迭代 SDK 内部实现时，需显式选择：

```ts
import { defineConfig } from "vite";
import { solidGpui } from "@solid-gpui/vite";
import { solidGpuiSource } from "@solid-gpui/vite/source";

export default defineConfig({
  plugins: [solidGpuiSource(), solidGpui({ entry: "src/app.tsx" })],
});
```

```sh
bun --conditions=solid-gpui-source vite      # 以源码解析运行、测试或构建
```

Bun 不接受 `bunfig.toml` 中的自定义 condition（已在 1.4.2 验证），因此请在命令行传入
`--conditions=solid-gpui-source`，例如 `bun --conditions=solid-gpui-source test`。
TypeScript 侧二选一：`"extends": "./.solid-gpui/tsconfig.json"`（生成工程已包含源码
`paths`），或 `"customConditions": ["solid-gpui-source"]` 配合
`"moduleResolution": "bundler"` 或 `"nodenext"`。

`@solid-gpui/vite/source` 导出 `SOURCE_CONDITION`、`SDK_PACKAGES`、
`solidGpuiSource(options?)`（插件）、`sdkSource(options?)` 与 `sdkPackage(name, root?)`。
`sdkSource({ root?, packages?, exclude? })` 返回解析结果：`aliases`（按最长优先，
可直接用于 `resolve.alias`）、`paths`（specifier 到绝对源码文件，即 `prepare` 写入
`.solid-gpui/tsconfig.json` 的同一张表）、`dedupe` 与 `names`，供完全不支持 condition
的工具使用。`@solid-gpui/core/components` 有意不在源码映射中，因为插件会把它重定向到
所选宿主的生成绑定；没有原生宿主的纯 Web 应用传 `exclude: []`。

请保持源码模式为显式选项：打包消费方与源码消费方必须共享同一个响应式图，普通用法仍
基于 `dist`。

## Rust 主导的 Vite 开发

```rust
use solid_gpui::runtime::vite::Vite;

let runtime = Vite::new("path/to/frontend")
    .config_file("vite.config.ts")
    .spawn()?;
solid_gpui::run_application(app::native_module, runtime);
```

省略 `config_file` 使用 Vite 默认配置发现。前端目录必须安装 npm 依赖。
需要设置环境变量时，`command()` 返回普通 `std::process::Command`。
helper 设置 Bun 的 client condition 和协议日志规则，并从当前原生可执行文件
导出 bindings，避免递归构建。启动 runtime 前必须处理 `--export-native`。

直接由 Rust 启动时，该 Rust 进程仍需由外部重建和重启。要自动重建原生代码并在
失败后继续监听，请通过配置了 `native` 的 Vite 启动应用。

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
bun run preview          # 或：<host> --runtime quickjs <bundle>
```

Vite 生成一个自包含 ESM，初始化明确的 QuickJS UI 平台，拒绝 Bun/Node 导入、
残留动态导入和外部资源文件。使用内联资源或宿主管理的文件。QuickJS 不提供
Bun API、Node 服务、DOM 或网络 `fetch`；这些能力应由 native 模块提供。运行时选择是
构建期决定：不会把 Bun bundle 转成 QuickJS bundle，也不会用另一个运行时顶替。

无 DOM 的 QuickJS 应用直接使用随包发布的 ambient 类型，无需自己手写声明：

```json
{
  "compilerOptions": {
    "lib": ["ES2024"],
    "types": ["@solid-gpui/core/quickjs"]
  }
}
```

该入口只声明引擎真正提供的能力——定时器（`setTimeout`、`setInterval`、`clearTimeout`、
`clearInterval`、`queueMicrotask`）、`performance.now()`、`console`、仅支持 UTF-8 的
`TextEncoder`/`TextDecoder`、`self`、`URL`/`URLSearchParams`/`DOMException`/`Event`/
`EventTarget`/`AbortController`/`AbortSignal`/`Headers`/`Response`（原生路由使用的无 body
重定向响应）、`import.meta.url`，以及内联资源用的 `declare module "*?inline"`（默认导出
`string`）。它有意不声明 `fetch`/`Request`、文件系统或 socket API、`requestAnimationFrame`、
`import.meta.hot` 以及任何 Node/Bun API，因此不可用的能力会在编译期报错而不是运行时抛错。
仍可用 `types: []` 配合手写 `.d.ts`，但会失去该校验。生成的 `.solid-gpui/tsconfig.json`
已包含该入口。

开发使用 Vite build watcher，保留配置中的插件和别名，再把成功产物交给 Rust
generation supervisor。替换 VM 时保留 Rust 服务和原生窗口；构建失败保留当前
应用。QuickJS 内部没有 ModuleRunner，也没有第二个打包器。
遵守工作区的解释器优化配置和[状态契约](hot-reload.zh-CN.md#捕获状态)。

组件宿主可以在 SolidRoot 外包装 provider 根节点；QuickJS 重载使用已保存的 renderer entity 验证快照，不要求窗口最外层就是 SolidRoot。

实验性浏览器宿主使用 `solidGpui({ target: "web" })`。它应用同一套 universal
JSX 转换，保留 Vite 的 HTML、client 环境和浏览器构建配置。见 [Web 宿主](web.zh-CN.md)。
