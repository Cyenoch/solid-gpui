# 原生异步组合与无障碍

## SwiftUI 与 AppKit 视图承载

Native Module 当前渲染 GPUI 元素和持久 GPUI 视图，没有暴露可嵌入的 AppKit 视图，也不支持与 SwiftUI 双向嵌套。多个 Surface 当前使用独立 GPUI 窗口，这不代表支持在同一个 SwiftUI/AppKit 窗口内放置多个 GPUI 视图。

[原生呈现研究](../.scratch/native-presentation/spec.md)建议先实现依附所属窗口的系统弹层，再建立明确的原生视图承载能力，并记录了输入、无障碍、生命周期和布局所需的改动。这是设计提案，尚未成为 SDK 能力。

## Solid 异步控制流

从 `@solid-gpui/core/runtime` 导入原生 `Suspense`、`ErrorBoundary` 和 `lazy`。它们使用 Solid 实现与原生子类型，resource 和 transition 共用 Solid owner 与响应式图。

```tsx
import { Text } from "@solid-gpui/core";
import { createResource, ErrorBoundary, Suspense } from "@solid-gpui/core/runtime";
import { useNative } from "./native";

function Greeting() {
  const native = useNative();
  const [greeting] = createResource(() => native.greet({ name: "Ada" }));
  return <Text>{greeting()}</Text>;
}

export function Page() {
  return (
    <ErrorBoundary fallback={(_error, reset) => <Text onPress={reset}>Retry</Text>}>
      <Suspense fallback={<Text>Loading</Text>}><Greeting /></Suspense>
    </ErrorBoundary>
  );
}
```

在边界的子组件内创建 resource，让边界拥有其错误与清理。transition 在资源等待时保留已解析内容。脱离挂载的 Suspense 内容在重新附着前仍留在 Solid 宿主图中；空的原生提交不会释放它。传输失败仍回滚失败提交。根卸载会拒绝待处理原生请求并释放 owner。

`createResource` 不自动取消被替代的 fetcher。需要停止过期工作时传入请求级 signal，并在 owner 清理时 abort。参见[请求取消](rust-bridge.md)。生产 Bun 和真实 QuickJS fixture `fixtures/quickjs-async.tsx` 通过二进制原生提交与回复验证资源加载、嵌套 Suspense、lazy、transition、错误重试及释放。

## 无障碍语义

基础原生元素支持 button、text、textbox、checkbox、heading、link、status、alert、group、list、listitem 和 dialog role。generic role 不会创建 AccessKit 节点；需要可访问容器时选择语义 role。

```tsx
<View accessibilityRole="status" accessibilityLive="polite"
      accessibilityValue={status()}>
  <Text>{status()}</Text>
</View>
```

`accessibilityLive` 接受 `off`、`polite` 或 `assertive`。非 off 的 live region 必须提供语义 role 和 `accessibilityValue`；macOS 播报使用 value，不只是 label。渲染器在同一 AccessKit 节点更新禁用与 live 状态，保留标识、动作和子节点。

`accessibilityDisabled` 只向辅助技术描述状态，不抑制应用回调。真正的交互策略使用控件 `disabled`；基础 TextInput 也将该状态投射到 AccessKit。原生编辑器语义由原生控件拥有。协议与节点测试验证状态投射；屏幕阅读器播报、焦点遍历和平台无障碍验收仍需实际辅助技术会话。

## 动态效果与文本编辑

生成的原生客户端提供 `getMotionPreference()` 和 `setMotionPreference("system" | "reduced" | "full")`，返回 `mode`、实际 `reduced` 和标为 `starting`、`available` 或 `unavailable` 的来源状态。Gallery 的 Transitions 页面展示三种模式。

原生宿主以 System 启动，在首个系统值返回前减少装饰动画。macOS NSWorkspace 通知、Windows 10 2004+ UISettings 事件、Linux 标准桌面 Settings Portal 持续更新偏好。订阅属于应用，跨零窗口和 HMR 存活，退出时释放。显式 Reduced/Full 保持效果，同时仍跟踪最新系统值。

来源不可用时选择 System 会返回显式错误并保留旧模式。跟随 System 期间来源失败，保留最后有效值并报告 `unavailable`，不虚构系统偏好。没有受支持 portal 的桌面可以选择显式模式。共同 GPUI 标记刷新所有窗口，供原生和渲染器动态效果消费。原生订阅不会安装浏览器 `matchMedia` 适配器。

基础 TextInput 的左右、Shift+方向键、退格及向前删除按扩展字素簇移动，包括组合重音与 ZWJ emoji。外部选区与 IME 契约仍为 UTF-16，显式范围不会静默扩大到字素边界。撤销与 marked text 规则不变。这不代表完整双向文字视觉光标/选区支持，也不改变独立 gpui-component 编辑器实现。

## 应用生命周期与激活

`mountApplication` 独立于窗口管理连接。`lastWindowClose` 默认 `"quit"`；`"keep-alive"` 在 `application.root` 为 `undefined` 时保留 Solid 应用 owner、transport 和捕获状态。`application.quit()` 请求原生进程退出并释放连接，`dispose()` 释放应用及连接。

```tsx
import { mountApplication, Text } from "@solid-gpui/core";
import { StdioTransport } from "@solid-gpui/core/stdio";

const application = mountApplication({
  transport: () => new StdioTransport(),
  lastWindowClose: "keep-alive",
  setup() {
    return {
      render: () => <Text>Document workspace</Text>,
      onMount(root) { /* Configure each newly opened window here. */ },
      onActivate({ reason, urls, root }) {
        // Dispatch validated application URLs to the current document model.
        console.error(`Activation: ${reason}; ${urls.length} URLs`);
      },
    };
  },
});
```

激活原因包括 `launch`、`reopen` 和 `open-urls`。宿主在启动应用前安装 GPUI 系统回调，渲染器就绪前最多缓存 32 次激活，每次最多 64 个 URL、每个 4096 字节。最后窗口关闭后，激活分配新的 Surface ID，先调用 `onMount` 再调用 `onActivate`。退役 ID 不复用。应用消息使用独立于 Surface revision 的应用 epoch 和确认序列。未确认激活在 HMR 后重放，已确认的不重放。处理函数确认同步接收，后续异步文档加载与错误报告由应用负责；处理函数失败会显式终止连接。

链接的 GPUI macOS provider 分派 reopen 和 open-URL 回调。适配器尚未实现 Windows/Linux 系统投递和跨进程单实例转发。URL/文档关联注册属于安装器或 bundle 配置。进程内激活测试不能证明桌面注册或第二实例验收。

## 应用拥有的进度服务

Gallery 的 Native & Platform 页面使用生成的 `WorkspaceScan` 原生视图。真实文件系统 worker 拥有取消 token，视图拥有观察者。改变 `requestId` 会重启路径请求，清空 `path` 会取消，卸载同时取消并移除观察者。回复携带请求 ID，旧进度不会覆盖新请求界面。

示例最多允许两个存活 worker，统计最多 100,000 项，排队最多 4096 个目录，不跟随符号链接，以 20 Hz 合并到一个最新值槽。文件系统调用离开 GPUI 前台线程。取消在文件系统操作之间协作完成；被阻塞的系统读取在真实退出前仍占用准入许可。错误显示在进度结果中。这是应用服务示例，不是框架文件系统 API。

## 原生网格

基础样式支持 `gridColumns`、`gridRows`、`gridColumnSpan` 和 `gridRowSpan`，每项取 1–64 的整数。声明轨道选择 GPUI 原生 grid，gap 与对齐不会将其变成 flex。轨道等分且最小值为零。网格轨道声明不能与 `flexDirection` 合用，但网格项可同时设置 span 和自身 flex 布局。Gallery 布局页展示响应式两列/三列网格。

生成的 Input/Textarea/Editor 导航和删除也使用扩展 Unicode 字素。Rope 分段仅读取所需块及跨块上下文，不复制整个文档。这保护逻辑编辑；完整视觉双向几何仍是独立问题。

## 图片

`Image` 接受本地路径、百分号编码的 `file:` URL、HTTP(S) 和 `data:image/...`。原生宿主在开窗前安装 HTTP 客户端，浏览器宿主使用平台 fetch。内联数据通过 GPUI 异步资源缓存解码，支持 GPUI 提供的 SVG 和动画格式。主来源与备用来源各限制为 1 MiB UTF-8 文本，仍遵守总帧预算。

`fallbackSource` 仅在主来源失败时使用，不因请求等待而显示，保留视口、object fit 和圆角。需离线工作的打包应用可用内联备用图片。相对路径基于宿主工作目录解析。

直接将 URL 传给 `source`，无需 JavaScript fetch 或临时文件：

```tsx
import { Image } from "@solid-gpui/core";

<Image
  source="https://images.unsplash.com/photo-1470770841072-f978cf4d019e?w=640"
  fallbackSource="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='320' height='180'%3E%3Crect width='320' height='180' fill='%2394a3b8'/%3E%3C/svg%3E"
  objectFit="cover"
  style={{ width: 320, height: 180, borderRadius: 12 }}
/>
```

GPUI 按来源管理异步下载、解码和缓存；更新 `source` 会选择新资源。浏览器请求需满足图片服务器的 CORS 策略。自行创建 GPUI `Application` 的 Rust 应用需通过 `with_http_client` 配置 HTTP 客户端。

原生 HTTP 连接超时 10 秒，闲置读取超时 15 秒；这些是连接与闲置上限，不是整个下载时长上限。
