# Rust 组件与 JavaScript 调用

使用一个 Rust 依赖 `solid-gpui` 和一个 npm 依赖 `@solid-gpui/core`。Rust 是声明源，宿主自动导出组件、事件、实例 ref 和 Promise 客户端。无需手写字段 ID、JSON 编解码、TypeScript 接口或单独建立 schema/codegen 包。

## 应用最小入口

应用的 `Cargo.toml`：

```toml
[dependencies]
solid-gpui = { path = "path/to/solid-gpui/crates/solid-gpui", features = ["gpui-component"] }
```

`src/main.rs`：

```rust
use solid_gpui::native_module;

#[native_module(name = "my-app")]
mod app {
    #[command]
    fn greet(name: String) -> Result<String, String> {
        if name.trim().is_empty() { return Err("Name is required".into()); }
        Ok(format!("Hello, {name}!"))
    }
}

fn main() { solid_gpui::run(app::native_module()); }
```

`#[command] async fn` 同样生成 Promise，完整函数在宿主共享的 Tokio 运行时执行，包含 timer 和 I/O driver；可以直接使用 Tokio 网络、定时器及其生态库。同步命令使用 Tokio blocking pool，普通 GPUI 工作留在 NativeView 的前台生命周期/实例方法里。

运行时按宿主模块集合懒创建；导出 TypeScript 不启动线程。每个 Surface 最多 32 个在途请求，整个模块集合最多 128 个，超限立即返回错误。Surface 关闭、epoch 替换或 Root 释放会取消受管异步调用；Tokio panic 转成请求错误。已经开始的同步 blocking 函数不能强行中断，完成前仍占用容量；需要取消的长任务应自行协作，不要把子任务 detach 后期待父调用取消它们。

宿主在启动窗口和运行时之前处理 `--export-native`：

```sh
cargo run --manifest-path native/Cargo.toml -- --export-native > src/native.ts
```

Solid 组件内直接捕获当前 Surface 的客户端：

```tsx
import { useNative } from "./native";

function Page() {
  const native = useNative();
  return <Button label="Greet" onPress={async () => {
    console.log(await native.greet({ name: "Ada" }));
  }} />;
}
```

外部调用可用 `createClient(root)`；不要在异步回调内重新调用 `useNative()`，因为 Solid owner 必须在组件初始化时捕获。

## Rust 自定义组件

在同一个 `#[native_module]` 中定义函数：

```rust
#[component(children = false)]
fn badge(
    label: String,
    #[prop(default)] highlighted: bool,
    cx: &mut solid_gpui::native::ElementContext,
) -> impl solid_gpui::gpui::IntoElement {
    use solid_gpui::gpui::{div, rgb, ParentElement, Styled, InteractiveElement};
    div().id(cx.id())
        .text_color(if highlighted { rgb(0x60a5fa) } else { rgb(0xffffff) })
        .child(label)
}
```

生成后直接 `<Badge label="Ready" highlighted />`。Rust 参数转换为 camelCase 属性；`#[prop(default)]` 使用 Rust Default，`#[prop(default = expression)]` 提供显式默认值。自定义 DTO 使用 `#[native_type]`，支持嵌套 struct、Vec、Option、对称 serde rename/tag/content 和枚举；拒绝独立 TS 覆盖、flatten、跳过字段或不同的输入/输出序列化规则。

事件参数 `on_press: Event<()>` 生成 `onPress?: () => void`，`on_change: Event<MyChange>` 生成类型化回调。`event.emit(value)` 发送语义事件；无订阅时不编码。高频 GPUI handler 应仅在 `event.is_subscribed()` 时安装。事件不提供同步 JS 返回值。

默认允许 JS children，使用 `cx.children()` 将其放到原生布局；不消费 children 的组件声明 `children = false`，宿主会在发布树之前拒绝错误嵌套。多个可交互子节点必须有不同 GPUI ID，可从 `cx.id()` 派生；不要依赖全局固定 ID。

## 有状态组件与实例方法

给 `impl NativeView for Editor` 标记 `#[component]`。实现 `Props`、`Event`、`mount`、`update`，并实现 GPUI `Render`。宿主在节点首次提交时创建 Entity，后续更新保留它；删除、类型替换或 epoch 变化释放它及其拥有的订阅和 Task。通过 `ViewCommand::new` 声明需要前台 Window/Context 的实例方法，生成 typed ref。业务异步命令使用模块级 `#[command]`。

`NativeView::Event` 可以是带标签的事件枚举，一个类型化事件流可表达多种状态变化。Rust 自己拥有复杂编辑器、树表、虚拟列表的 delegate 和瞬态数据，不要求为每个内部状态同步一个 JS 属性。大数据与高频输入应使用批量 DTO 或 Rust 内部模型，避免逐项往返。

内置组件从主包子路径导入：

```tsx
import { Button, Input, type InputRef } from "@solid-gpui/core/components";
import { createSignal } from "@solid-gpui/core/runtime";

const [value, setValue] = createSignal("");
let editor: InputRef | undefined;

<Input value={value()} onChange={(event) => setValue(event.value)}
       ref={(ref) => { editor = ref; }} />;
<Button label="Focus" onPress={() => editor?.focus()} />;
```

`Input` 使用真实 gpui-component InputState；同值回传保留选区、滚动和撤销历史。受控 `onChange` 应同步更新 `value`，异步校验在更新后执行。需要由异步结果决定是否替换时，使用 `defaultValue` 让输入由 Rust 持有，再显式 `ref.replaceValue(text)`；该操作按上游替换语义结束组合输入并更新撤销记录。`defaultValue` 只在挂载时生效。

方法调用自动等待当前 Solid 事务产生完整 Patch。实例卸载时 ref 收到 undefined，未完成的调用 reject。跨 Surface、过期 epoch、契约摘要不匹配或目标失效都产生明确错误。

## 数据限制与构建

每个 DTO 最多 1 MiB；只传普通 JSON 数据，拒绝循环引用、非有限数、超出安全范围的整数、BigInt、类实例、未知字段和非法枚举。JS 回调、GPUI Entity、线程对象不跨越运行时。业务范围约束写在 Rust DTO 的 Deserialize 中，如内置 Percentage 的 0–100 校验。

仓库执行 `bun run task native-codegen` 生成 core 组件和 Gallery 本地绑定，`bun run task native-codegen-check` 检测漂移。生成文件应提交到版本库供编辑器和检查使用，不手改。Vite 的 native 选项调用同一个实际宿主导出器，提供 `#native` 别名；Rust 改动需要重编译并重启宿主。

可运行范例：`examples/gallery/native/src/lib.rs` 同时声明 `BuildBadge` 和 `analyze_workspace`；Gallery TSX 使用生成组件和 Promise 客户端。设计依据见 [ADR-0016](adr/0016-rust-owned-native-modules.md)。
