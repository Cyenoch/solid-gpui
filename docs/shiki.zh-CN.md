# 使用 Shiki 进行语法高亮

`@solid-gpui/shiki` 使用 GPUI 原生文本系统渲染可选择代码。当前实时高亮后端在持久 Bun Worker 中运行 Shiki 与 Oniguruma，尚不支持 QuickJS。

## 安装与初始化

```sh
bun add @solid-gpui/shiki @solid-gpui/core solid-js
```

在挂载前，为应用或显式拥有生命周期的功能创建一个服务：

```ts
import { createBunHighlighter } from "@solid-gpui/shiki/bun";

const highlighter = await createBunHighlighter({
  languages: ["typescript", "tsx", "rust"],
  themes: ["github-dark", "github-light"],
});
```

与应用其余部分共用 Solid GPUI JSX 转换和 Bun `--conditions=browser` 设置。在应用入口处理初始化失败，在渲染组件外使用 Solid GPUI `ErrorBoundary` 展示高亮错误。

## 渲染代码

```tsx
import { CodeBlock } from "@solid-gpui/shiki";
import { onCleanup } from "@solid-gpui/core/runtime";

function App() {
  onCleanup(() => highlighter.dispose());
  return (
    <CodeBlock
      highlighter={highlighter}
      code={'const greeting = "Hello, GPUI!";\n'}
      language="typescript"
      theme="github-dark"
      style={{ fontSize: 14, lineHeight: 22 }}
    />
  );
}
```

代码、语言和主题属性均为响应式。组件取消被替代的请求，忽略迟到结果。等待结果时，使用周围文本样式显示当前源码。结果主题提供前景和背景颜色，布局与字体来自 `style`。

组件默认支持选择。所有文本片段构成一个原生段落，选择与复制可以跨 token 边界。原始 Unicode 和换行符保持不变。这是只读代码块，限制为 64 KiB UTF-8 源码及 4,096 个合并后的片段，不是虚拟化编辑器。

## 构建时高亮

已知代码片段可在 Bun 执行的 Vite 构建期间高亮，并序列化为数据：

```ts
const result = await highlighter.highlight({
  code: "const answer = 42;",
  language: "typescript",
  theme: "github-dark",
});
highlighter.dispose();
```

```tsx
import { HighlightedCode } from "@solid-gpui/shiki";

<HighlightedCode highlighted={result} />;
```

`HighlightedCode` 只渲染传入结果，不依赖 Bun，也不启动 worker。文档网站采用此路径：Vite 通过 Bun 后端生成结果，浏览器中的 GPUI WebAssembly 宿主渲染原生文本片段。分词器及 Oniguruma WASM 不进入浏览器包。

## 所有权与分发

应用负责在卸载、热重载和退出时释放服务。组件只取消自身请求，不销毁共享服务。后端同时处理一个请求，最多排队 32 个请求。硬截止时间会终止卡住的工作并拒绝服务中的请求，不会静默替换引擎或自动重启。

Bun 应用包必须同时携带已安装包的 Worker 资源及 Shiki 依赖，仅复制应用 JavaScript 文件不够。内嵌 Bun 和独立可执行文件打包仍需分别验证。

API 细节、支持样式、限制、Worker URL 覆盖、取消行为和可运行原生示例见[包 README](../packages/solid-gpui-shiki/README.md)。
