# 入门

从零开始的外部应用只需一条顺序：安装、prepare、类型检查、开发、测试、构建，以及预览生产 bundle。进阶集成各自成篇，并在每一步中链接：

- [Vite 集成](vite.zh-CN.md) — 插件完整选项面、`host: false` 与自定义 transport。
- [Rust 集成](rust-bridge.zh-CN.md) — Native Module、桌面宿主与窗口配置。
- [选择运行时](runtimes.zh-CN.md) — 外部 Bun、内嵌 Bun 与 QuickJS。
- [开发流程](hot-reload.zh-CN.md) — watch 行为与重载状态契约。
- [桌面分发](distribution.zh-CN.md) — bundle、可执行文件、签名与目标平台支持。

## 前置条件

- **Bun 1.4.2 或更高版本。** Vite 与 `solid-gpui` 工具都在 Bun 下运行；用 `bun --bun vite` 启动 Vite。
- **仅在应用构建 Rust 宿主时需要 Rust 工具链。** 用 rustup 安装 `rust-toolchain.toml` 指定的工具链，并安装平台原生构建依赖。参见[构建环境](distribution.zh-CN.md#构建环境)。
- **消费方工作区根需包含 SDK 的 Cargo 要求。** 你的宿主 crate 通过 path 或 git 依赖同一份固定 checkout 中的 `solid-gpui` crate（目前未发布到 registry）。Cargo 会忽略依赖内声明的 profile 设置，因此消费方工作区根必须带上本仓库 `Cargo.toml` 中的 `[patch.crates-io]` gpui-pre 条目与 debug profile（QuickJS 宿主还需 `rquickjs-sys`）。`solid-gpui doctor` 会推导当前生效的依赖、profile 与 patch 要求并报告不一致。

## 1. 安装

SDK 尚未发布到公共 registry：`@solid-gpui/core` 与 `@solid-gpui/vite` 目前无法从 npm 解析，`solid-gpui` Rust crate 也从 checkout 消费。请用同一份固定的 SDK checkout 打包出配套 tarball 后安装，不要混用不同批次的版本。

在 SDK checkout 中：

```sh
bun install --frozen-lockfile
bun run task sdk-pack ../sdk-tarballs
```

它会写出 `solid-gpui-core.tgz`、`solid-gpui-vite.tgz`、`solid-gpui-router.tgz` 与 `solid-gpui-shiki.tgz`，且不编译原生宿主。

在你的应用中：

```sh
bun init
bun add ../sdk-tarballs/solid-gpui-core.tgz solid-js
bun add -d ../sdk-tarballs/solid-gpui-vite.tgz vite
```

`package.json` 保持 `"type": "module"`。`@solid-gpui/core` 是渲染器与组件包；`@solid-gpui/vite` 提供 Vite 插件、`solid-gpui` CLI 与测试入口。需要时从同一批次加入 `@solid-gpui/router` 与 `@solid-gpui/shiki`；拉取新的 SDK 修订后重新运行 `sdk-pack` 并重装。

将来这些包发布到 registry 后可按名称安装，其余步骤不变；目前并非如此。直接调试 SDK 源码是另一个显式选项，见[源码消费](vite.zh-CN.md#从源码消费包)。

## 2. 配置 Vite 与 TypeScript

`vite.config.ts` 指定应用入口及其原生宿主：

```ts
import { defineConfig } from "vite";
import { solidGpui } from "@solid-gpui/vite";

export default defineConfig({
  plugins: [
    solidGpui({
      entry: "src/app.tsx",
      runtime: "bun",
      native: { manifestPath: "native/Cargo.toml", bin: "my-app" },
    }),
  ],
});
```

`entry` 是调用 `mountApplication` 的模块。`native` 负责构建并 watch 该 Cargo 宿主，并导出其绑定。自建的既有可执行文件使用 `host: { command }`；由自己的 Vite 环境与 ModuleRunner 掌管的进程使用 `host: false`。两者见 [Vite 集成](vite.zh-CN.md)。

`runtime` 显式选择执行模型，且从不自动转换：为 `bun` 构建的 bundle 保留 Bun 与 Node 导入，而 `runtime: "quickjs"` 构建单个自包含 ES 模块并拒绝这些导入。请在构建时选定运行时，并用匹配的宿主运行产物。

`tsconfig.json` 保持精简，其余映射由生成文件提供：

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

`.solid-gpui/tsconfig.json` 由 `prepare` 生成，把 `#native` 和 `@solid-gpui/core/components` 映射到插件实际选中的绑定文件，因此不必手工维护 `paths`；它还携带解析模式、`jsx`，以及 QuickJS runtime 下的无 DOM `lib`/`types` 组合，路径为绝对路径，消费方的 `baseUrl` 无法破坏它。你自己的文件只需加入严格性选项与 `include`。把 `.solid-gpui/` 加入 `.gitignore`。

TypeScript 需要 `jsx: "preserve"` 与 `jsxImportSource: "@solid-gpui/core"`，后者提供宿主元素类型。编译器固定为 `@solidjs/compiler` 2.0.0-rc.6，应用运行时仍为 Solid 1.9.15。

## 3. Prepare 原生绑定

先添加脚本，再在安装后运行生成器（包安装过程绝不编译原生宿主，因此脚本不叫 `prepare`）：

```json
{
  "scripts": {
    "generate": "solid-gpui prepare",
    "check:generated": "solid-gpui prepare --check",
    "doctor": "solid-gpui doctor",
    "typecheck": "tsc --noEmit",
    "dev": "bun --bun vite",
    "test": "solid-gpui test",
    "build": "bun --bun vite build",
    "preview": "solid-gpui preview"
  }
}
```

```sh
bun run generate
```

`prepare` 读取你的 `vite.config.ts`，按需构建所配置的 Cargo 宿主，执行其 `--export-native` 入口，并写入：

| 产物                         | 内容                                                                        |
| ---------------------------- | --------------------------------------------------------------------------- |
| `.solid-gpui/tsconfig.json`  | 针对所选绑定的 TypeScript 映射生成文件，请勿手改。                          |
| `.generated/native.ts`       | 宿主导出的组件目录、命令与类型（即 `native.output`）。                      |
| `.solid-gpui/artifacts.json` | 解析后的产物记录：`root`、`runtime`、`entry`、`outDir`、bindings、tsconfig、host，以及构建后实际产出的 bundle 路径。 |

它不运行应用，也不产出生产 bundle。

- `bun run check:generated`（`solid-gpui prepare --check`）不写入任何文件；绑定或生成的 tsconfig 过期或缺失时以非零状态退出。它仍会按需解析并构建宿主，以便与实际目录比对，也不会重写 `.solid-gpui/artifacts.json`；CI 用它作为新鲜度门槛，而不是依赖开发/构建命令的副作用。
- `bunx solid-gpui prepare --json` 输出解析后的产物记录而不是叙述性文本。
- `bun run doctor` 报告环境与依赖问题，包括 Cargo profile/patch 不一致以及不支持的 runtime/target 组合。

当默认值不合适时，`prepare`、`prepare --check`、`doctor`、`preview`、`test` 都接受 `--root`、`--config`、`--mode`；`preview` 还会透传 `--` 之后的宿主参数。

## 4. 类型检查

```sh
bunx tsc --noEmit
```

## 5. 开发

```sh
bun --bun vite
```

Vite 编译 JSX/TSX，并把模块提供给宿主掌管的 Bun 子进程；原生窗口由宿主管理。Rust 源码、Cargo manifest 与配置、声明的 build-script 输入、声明的原生资源，以及 `native.watch` 指定的路径，会在每次突发变更后合并为一次宿主重建与会话替换；应用模块仍走普通 Vite HMR。`.solid-gpui/`、`target/`、构建输出目录和 `node_modules/` 永不触发重建。状态保留、失败恢复以及 Rust 与 JavaScript 的分工见[开发会话管理](hot-reload.zh-CN.md#开发会话管理)。

## 6. 测试

```sh
bunx solid-gpui test                     # 全量
bunx solid-gpui test src/app.test.tsx    # 单个文件
```

该命令委托给 `@solid-gpui/vite/test` 的 `runTests({ root?, configFile?, args? })`，它加载你开发所用的同一份 `vite.config.ts`，因此测试看到的是真实的 JSX 转换、alias、`?inline` 资源与原生绑定契约，而不是手搭的替身。运行器自行加入 `browser` condition 并去重 `solid-js`，因此无需传 `--conditions=browser`，测试与打包应用共享同一个响应式图。参数处理沿用 Bun 语义：过滤与标志原样转给 `bun test`，普通单文件调用同样可用。测试 API 从 `bun:test` 导入。

## 7. 构建 bundle

```sh
bun --bun vite build
```

Vite 向 `build.outDir`（默认 `dist/`）写入单个生产入口模块：Bun 模式打包 JavaScript 依赖并把 Bun/Node 内建保留为运行时导入；QuickJS 模式写出单个自包含 ES 模块，并拒绝 Bun/Node 导入、遗留动态导入与外部资源。构建过程同样会 prepare 所配置的原生宿主（通常是增量 Cargo 重建），因此 `build` 与 `generate` 都会让宿主保持最新。bundle 路径记录在 `.solid-gpui/artifacts.json` 中，构建会用实际产出的文件更新它，无需自行推导。

这一步产出 **bundle**（配置了 `native` 时也产出 **原生可执行文件**）。它不打包、不签名、也不产出 **可分发包**。区别与目标平台支持见[桌面分发](distribution.zh-CN.md)。

## 8. 预览生产 bundle

```sh
bun run preview          # solid-gpui preview
```

`preview` 用已构建的宿主运行 `.solid-gpui/artifacts.json` 记录的 bundle：不重建、不启动 watcher，并以宿主退出码结束。宿主参数放在 `--` 之后。记录或 bundle 缺失时，它会直接给出应执行的命令（例如 `vite build`，或 `solid-gpui prepare && vite build`）；当记录中的 runtime、bundle、宿主包、二进制、profile 或 target 与你指定的配置不一致时它会拒绝运行（对构建产物所用的配置传 `--config`）。编程调用使用 `@solid-gpui/vite/project` 的 `previewApplication({ root?, configFile?, args?, env? })`；只需要路径的脚本通过 `@solid-gpui/vite/artifacts` 的 `prepareProject`/`readNativeArtifacts` 读取同一条记录，而不要硬编码 `target/<profile>/<bin>` 或 bundle 路径。由 Rust 选择 transport 的应用（例如用 `--production` 选择 `EmbeddedTransport` 而不是 `StdioTransport`）把该标志透传进来。

## 创建组件

Solid GPUI 的 runtime 模块提供 Solid 客户端响应式原语，以及通用渲染器的 `createComponent` 函数。

```ts
import { Pressable, Text, View } from "@solid-gpui/core";
import { createComponent, createSignal } from "@solid-gpui/core/runtime";

export function Counter() {
  const [count, setCount] = createSignal(0);
  return createComponent(View, {
    style: { padding: 24, gap: 12 },
    get children() {
      return [
        createComponent(Text, { children: () => `Count: ${count()}` }),
        createComponent(Pressable, {
          accessibilityRole: "button",
          accessibilityLabel: "Increment",
          onPress: () => setCount((value) => value + 1),
          children: createComponent(Text, { children: "Increment" }),
        }),
      ];
    },
  });
}
```

getter 和函数形式的子内容会建立响应式读取。当 `count` 改变时，Solid 渲染 effect 更新原始文本 Host Node，根节点发送一个增量 Patch。

同一个组件用 JSX 编写时无需额外配置：

```tsx
import { Pressable, Text, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";

export function Counter() {
  const [count, setCount] = createSignal(0);

  return (
    <View style={{ padding: 24, gap: 12 }}>
      <Text>Count: {count()}</Text>
      <Pressable accessibilityRole="button" onPress={() => setCount((value) => value + 1)}>
        <Text>Increment</Text>
      </Pressable>
    </View>
  );
}
```

`For`、`Index`、`Show`、`Switch`、`Match` 请从 `@solid-gpui/core/runtime` 导入，而不是 `solid-js`：runtime 入口提供原生元素类型，并且是生命周期、owner、context、数组与 store 辅助函数（`onMount`、`onCleanup`、`getOwner`、`runWithOwner`、`createContext`、`useContext`、`splitProps`、`createStore` 等）的统一客户端入口。参见[原生控制流](native-composition.zh-CN.md#solid-异步控制流)。

## 挂载根节点

配置的入口模块负责挂载应用并拥有其 transport：

```tsx
import { mountApplication } from "@solid-gpui/core";
import { StdioTransport } from "@solid-gpui/core/stdio";
import { Counter } from "./counter";

mountApplication({
  transport: () => new StdioTransport(),
  setup: () => ({ render: () => <Counter /> }),
});
```

transport 工厂只创建一次应用自有连接，并在最终销毁时关闭它。进程（Bun）运行时使用 `StdioTransport`，内嵌 Bun 与 QuickJS 使用 `@solid-gpui/core/embedded` 的 `EmbeddedTransport`；自定义 transport 实现同一接口。参见[选择运行时](runtimes.zh-CN.md)与 [Vite 集成](vite.zh-CN.md)。

不使用 `mountApplication` 时，自行创建根节点并向 `root.render` 传入创建函数，使组件在根节点的 Solid owner 下创建，并将后续响应式工作绑定到正确的原生 surface：

```ts
import { createRoot } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";
import { StdioTransport } from "@solid-gpui/core/stdio";
import { Counter } from "./counter";

const root = createRoot(new StdioTransport(), { surfaceId: 1, epoch: 1 });
root.render(() => createComponent(Counter, {}));
```

## 无显示环境测试

```ts
import { MemoryTransport, View, createRoot } from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";

const transport = new MemoryTransport();
const root = createRoot(transport);
root.render(() => createComponent(View, {}));
console.log(transport.submitted.length); // Snapshot frame
root.unmount();
```

`MemoryTransport` 用于验证带帧渲染输出和根命令契约，可放在 `solid-gpui test` 文件或普通 `bun test` 文件中。原生布局、绘制、对话框和平台窗口行为需要有显示环境的宿主验证。

## 其他消费方式

- **改用源码而非 `dist`：** 加入 `solidGpuiSource()` 插件，并以 `--conditions=solid-gpui-source` 运行 Bun（`bunfig.toml` 无法设置自定义 condition）。普通用法仍基于 `dist`。参见[源码消费](vite.zh-CN.md#从源码消费包)。
- **由 Rust 掌管 Vite：** `solid_gpui::runtime::vite::Vite` 从 Rust 进程启动同一份配置。参见 [Rust 主导的 Vite 开发](vite.zh-CN.md#rust-主导的-vite-开发)。
- **单一可执行文件：** 内嵌 Bun 打包仍是实验性的。参见[运行时策略](runtime-strategy.zh-CN.md)与[内嵌 Bun 静态应用](distribution.zh-CN.md#内嵌-bun-静态应用)。

仓库贡献者运行共享网站即可：`bun run website:native`（原生）、`bun run website:native:dev`（原生热重载）、`bun run website`（浏览器）。见[网站 README](../examples/website/README.md)。
