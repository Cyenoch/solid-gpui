# Rust 组件与 JavaScript 调用

只需 Rust 依赖 `solid-gpui` 和 npm 依赖 `@solid-gpui/core`。Rust 声明定义契约，宿主导出组件、事件、实例 ref 和 Promise 客户端。无需手写字段 ID、JSON 编解码、TypeScript 接口，也不需要独立 schema 或代码生成包。

## 最小应用

应用 `Cargo.toml`：

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
        if name.trim().is_empty() {
            return Err("Name is required".into());
        }
        Ok(format!("Hello, {name}!"))
    }
}

fn main() {
    solid_gpui::run(app::native_module());
}
```

`#[command] async fn` 生成 Promise 客户端，整个函数在宿主共享 Tokio runtime 上执行，包括定时器和 I/O driver，可直接使用 Tokio 网络、定时器和生态库。同步命令运行在 Tokio blocking pool。普通 GPUI 工作应放在前台生命周期或 NativeView 实例方法中。

runtime 为宿主模块集合延迟创建，导出 TypeScript 不启动线程。每个 Surface 最多 32 个在途请求，模块集合最多 128 个，超量立即失败。Surface 关闭、epoch 替换和 Root 释放取消托管异步调用。Tokio panic 转为请求错误。已开始的同步阻塞函数无法强制中断，结束前持续占用容量。长期任务必须协作取消，脱离父调用的子任务不会自动取消。

## 请求取消与截止时间

生成客户端方法的第二个参数是 `NativeCallOptions`：

```tsx
const controller = new AbortController();
const result = native.greet({ name: "Ada" }, {
  signal: controller.signal,
  timeoutMs: 5_000,
});
controller.abort();
await result;
```

取消以 signal reason 拒绝，超时以 `TimeoutError` 拒绝。已取消 signal 不发送请求。deadline 是 0–2,147,483,647 毫秒的整数。没有请求 DTO 的调用将首参设为 `undefined`。成功和取消都会释放 JS listener 与 timer。取消只针对原始 Surface 和 epoch 内的一个请求，迟到结果被丢弃。

可选 Rust `NativeCallContext` 参数由命令宏注入，不进入生成的请求 DTO：

```rust
#[command]
fn scan(paths: Vec<String>, context: solid_gpui::native::NativeCallContext)
    -> Result<u32, String>
{
    let mut completed = 0;
    for path in paths {
        context.check_cancelled()?;
        std::fs::metadata(path).map_err(|error| error.to_string())?;
        completed += 1;
    }
    Ok(completed)
}
```

阻塞工作应在有界工作单元之间检查取消；实际返回前继续占用准入槽。`context.cancelled().await` 可协调应用拥有的子任务，普通异步命令 future 也由执行器 abort。取消不会撤销已完成副作用。直接注册的 `CommandDefinition` 接收 `(request, context)`。

宿主在启动窗口或 runtime 前处理 `--export-native`：

```sh
cargo run --manifest-path native/Cargo.toml -- --export-native > src/native.ts
```

在 Solid 组件初始化时捕获当前 Surface 的客户端：

```tsx
import { Button } from "@solid-gpui/core/components";
import { useNative } from "./native";

function Page() {
  const native = useNative();
  return (
    <Button
      label="Greet"
      onPress={async () => {
        console.error(await native.greet({ name: "Ada" }));
      }}
    />
  );
}
```

组件外使用 `createClient(root)`。不要在异步回调中调用 `useNative()`，它必须在组件初始化时捕获 Solid owner。诊断写 stderr，stdout 承载协议帧。

## 自定义 Rust 组件

在同一 `#[native_module]` 中定义函数：

```rust
#[component(children = false)]
fn badge(
    label: String,
    #[prop(default)] highlighted: bool,
    cx: &mut solid_gpui::native::ElementContext,
) -> impl solid_gpui::gpui::IntoElement {
    use solid_gpui::gpui::{div, rgb, InteractiveElement, ParentElement, Styled};
    div().id(cx.id())
        .text_color(if highlighted { rgb(0x60a5fa) } else { rgb(0xffffff) })
        .child(label)
}
```

生成组件支持 `<Badge label="Ready" highlighted />`。Rust 参数转为 camelCase 属性。`#[prop(default)]` 使用 Rust `Default`，`#[prop(default = expression)]` 指定默认表达式。自定义 DTO 使用 `#[native_type]`，支持嵌套结构体、Vec、Option、对称 serde rename/tag/content 及枚举。独立 TypeScript 覆盖、flatten、跳过字段和不对称输入输出序列化会被拒绝。

`on_press: Event<()>` 生成 `onPress?: () => void`，`on_change: Event<MyChange>` 生成类型化回调。`event.emit(value)` 发送 Native Event，没有订阅者时跳过编码。只有 `event.is_subscribed()` 时才安装高频 GPUI handler。事件不提供同步 JavaScript 返回值。

组件默认允许 JS 子内容，用 `cx.children()` 放入原生布局。不消费子内容的组件声明 `children = false`，让宿主在发布树前拒绝无效嵌套。交互后代需要从 `cx.id()` 派生不同 GPUI ID，不要跨实例使用全局固定 ID。

## 有状态组件与实例方法

为 `impl NativeView for Editor` 标注 `#[component]`，定义 Props 和 Event，实现 mount、update 及 GPUI Render。首次提交节点时宿主创建 Entity，更新期间保留；移除、类型或 epoch 替换释放 Entity 及其订阅和 Task。需要前台 Window/Context 的实例方法用 `ViewCommand::new` 声明，生成类型化 ref；异步应用操作使用模块级 `#[command]`。

`NativeView::Event` 可用带标签枚举在一个类型化事件流中表达多种变化。复杂编辑器、树表和虚拟列表的 delegate 与瞬态状态由 Rust 拥有，不必将每个内部状态变化映射为 JS 属性。大数据集和高频输入使用批量 DTO 或 Rust 模型，避免逐项往返。

从主包子路径导入内置组件：

```tsx
import { Button, Input, type InputRef } from "@solid-gpui/core/components";
import { createSignal } from "@solid-gpui/core/runtime";

function Editor() {
  const [value, setValue] = createSignal("");
  let editor: InputRef | undefined;
  return (
    <>
      <Input
        value={value()}
        onChange={(event) => setValue(event.value)}
        ref={(ref) => {
          editor = ref;
        }}
      />
      <Button label="Focus" onPress={() => editor?.focus()} />
    </>
  );
}
```

`Input` 使用真实 gpui-component InputState。返回相同值会保留选区、滚动和撤销历史。受控 `onChange` 应同步更新 `value`，再进行异步验证。若异步结果决定是否替换文本，使用 `defaultValue` 将所有权留在 Rust，并显式调用 `ref.replaceValue(text)`。该操作遵循上游替换语义，结束输入法组合并更新撤销历史。`defaultValue` 只在挂载时应用。

方法调用等待当前 Solid 事务生成完整 Patch。卸载将 ref 设为 `undefined` 并拒绝待处理调用。跨 Surface、过期 epoch、契约 digest 不匹配和无效目标都返回显式错误。

## 数据限制与生成

每个 DTO 限制 1 MiB，必须是普通 JSON 数据。循环、非有限数、安全整数范围外的整数、BigInt、类实例、未知字段和无效枚举均被拒绝。JS 回调、GPUI Entity 和线程对象不跨运行时边界。领域约束应写在 Rust DTO 的 Deserialize 实现中，例如内置 Percentage 的 0–100 验证。

运行 `bun run task native-codegen` 生成核心组件和 website 绑定，运行 `bun run task native-codegen-check` 检查漂移。提交生成文件以支持编辑器与验证，不要手工修改。Vite `native` 选项调用相同宿主导出器并提供 `#native` 别名。Rust 变化需要重建并重启宿主。

可运行示例 `examples/website/native/src/lib.rs` 声明 BuildBadge 和 analyze_workspace，website TSX 使用生成组件与 Promise 客户端。设计依据见 [ADR-0016](adr/0016-rust-owned-native-modules.md)。
