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
    solid_gpui::run(app::native_module);
}
```

`#[command] async fn` 生成 Promise 客户端，整个函数在宿主共享 Tokio runtime 上执行，包括定时器和 I/O driver，可直接使用 Tokio 网络、定时器和生态库。同步命令运行在 Tokio blocking pool。普通 GPUI 工作应放在前台生命周期或 NativeView 实例方法中。

runtime 为宿主模块集合延迟创建，导出 TypeScript 不启动线程。每个 Surface 最多 32 个在途请求，模块集合最多 128 个，超量立即失败。Surface 关闭、epoch 替换和 Root 释放取消托管异步调用。Tokio panic 转为请求错误。已开始的同步阻塞函数无法强制中断，结束前持续占用容量。长期任务必须协作取消，脱离父调用的子任务不会自动取消。

## 桌面宿主配置

`solid_gpui::run_application(module_factory, runtime)` 使用框架默认 profile 运行应用拥有的 runtime，宿主负责关闭并在应用退出时 join 该 runtime。`solid_gpui::run_application_with_profile(profile_factory, runtime)` 运行应用拥有的 profile 和 runtime，不解析宿主命令行，也不创建 runtime。两者都复用 `run` 的宿主运行器：提交准入、原生事件、协议处理、Surface 所有权、覆盖层、窗口关闭和运行时最终关闭仍由框架管理。

宿主入口负责平台启动。传入在应用线程内构造 profile 的工厂，因为 profile 可以包含非 `Send` 的 GPUI 状态；工厂及捕获值须满足 `Send + 'static`。应用无需另建线程，也不要为运行器添加平台启动包装。

Windows 调用线程栈不足时，运行器为应用线程预留 16 MiB，而不是依赖可执行文件通常只有 1 MiB 的初始线程栈；原生布局和绘制会沿元素树递归。其他平台原地执行，macOS 的 AppKit 仍在真正主线程运行。可通过 `SOLID_GPUI_APP_STACK_BYTES` 覆盖 Windows 栈预留做测量，`SOLID_GPUI_LOG=info` 会报告实际应用线程栈大小；无效预算会使启动失败。预留不代表每个任意深度的应用都能容纳。消费工作区应保持[开发优化配置](hot-reload.zh-CN.md#应用构建配置)一致，测量方法见 [Windows 宿主栈溢出](troubleshooting.zh-CN.md#windows-宿主栈溢出)。

```rust
use solid_gpui::{gpui::*, components::host::ComponentHost};

let profile = || ComponentHost::new(vec![
    solid_gpui::components::native_module(),
    app::native_module(),
])
.with_window_options(|_, cx| {
    let mut options = gpui_component::TitleBar::window_options();
    options.window_bounds = Some(WindowBounds::Windowed(Bounds::centered(
        None, size(px(1100.), px(720.)), cx,
    )));
    options.window_min_size = Some(size(px(960.), px(640.)));
    options.titlebar.as_mut().unwrap().traffic_light_position =
        Some(point(px(16.), px(17.)));
    options
})
.with_performance_monitor(false);
solid_gpui::run_application_with_profile(profile, runtime);
```

性能监视器在所有构建中**默认关闭**。应用通过 `with_performance_monitor(true)` 显式开启，环境变量不会覆盖该策略。

`ComponentHost::with_initialize(|cx| { ... })` 在组件主题初始化之后、首个窗口创建之前运行，用于配置首帧原生状态，无需重新实现并转发整个 `HostProfile` trait。例如产品选择 reduced motion 时，可在此调用 `solid_gpui::motion::set(MotionMode::Reduced, cx)`。关闭动画是应用策略，不是栈溢出修复。

`useNative().setApplicationTheme` 在运行时设置应用的颜色、排版和控件尺寸，参见[应用主题覆盖](gpui-components.zh-CN.md#应用主题覆盖)。图标在运行时启动前注册，参见[添加应用图标](iconify.zh-CN.md#添加应用图标)。可运行宿主、窗口配置与标题栏组合见[桌面应用示例](../examples/desktop-app/README.zh-CN.md)。

### 窗口选项与标题栏

`ComponentHost::with_window_options` 在渲染器打开 Surface 的覆盖项之前配置原生窗口默认值；自定义 profile 可直接实现 `HostProfile::window_options`。回调会在每个 Surface 打开时执行，并接收渲染器请求的选项，只想修改部分字段时应保留其余选项。显式指定的标题、窗口类型、可调整大小和最小尺寸在回调之后应用。

`gpui_component::TitleBar::window_options()` 提供透明 macOS 标题栏以及 `app_owns_titlebar_drag: true`。红绿灯位置指关闭按钮左上角：17 px 可将 14 px 按钮居中于 48 px 标题栏。宿主不会额外添加 Solid 标题栏，应用只应渲染一个：

```tsx
<TitleBar
  style={{
    height: 48,
    padding: 0,
    paddingLeft: 88,
    paddingRight: 16,
    backgroundColor: "#131217",
    borderWidth: 0,
    borderBottomWidth: 1,
    borderColor: "#2C2B33",
  }}
>
  <View style={{ flexDirection: "row", flexGrow: 1, alignItems: "center" }}>
    <Text>Application</Text>
    <Input placeholder="Search" style={{ width: 160 }} />
  </View>
</TitleBar>
```

样式设置会覆盖 TitleBar 原生默认值，包括左内边距。全屏不会增加额外内边距。原生 TitleBar 处理空白区域拖拽与 macOS `titlebar_double_click`，遵循系统偏好。接管鼠标按下事件的控件不会触发标题栏拖拽或双击缩放；核心 `Pressable` 也会接管此原生默认动作。自定义标题栏的应用需要在 `solid-gpui` 之外同时依赖固定版本的 `gpui-component`。

## 请求取消与截止时间

生成客户端方法的第二个参数是 `NativeCallOptions`：

```tsx
const controller = new AbortController();
const result = native.greet(
  { name: "Ada" },
  {
    signal: controller.signal,
    timeoutMs: 5_000,
  },
);
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

原生绑定导出会拒绝 DTO 类型中的 TypeScript `any` 和 `bigint`，包括嵌套对象、数组、联合类型、泛型参数及模板字面量的类型插值。文档注释（例如 “The pending request, if any.”）、名为 `any` 或 `bigint` 的属性，以及字符串或模板字面量的文本不会触发此校验，也不会从生成绑定中移除。不受支持的值类型应改用有界 Rust 值或显式字符串。

`on_press: Event<()>` 生成 `onPress?: () => void`，`on_change: Event<MyChange>` 生成类型化回调。`event.emit(value)` 发送 Native Event，没有订阅者时跳过编码。只有 `event.is_subscribed()` 时才安装高频 GPUI handler。事件不提供同步 JavaScript 返回值。

组件默认允许 JS 子内容，用 `cx.children()` 放入原生布局。不消费子内容的组件声明 `children = false`，让宿主在发布树前拒绝无效嵌套。交互后代需要从 `cx.id()` 派生不同 GPUI ID，不要跨实例使用全局固定 ID。

## 有状态组件与实例方法

为 `impl NativeView for Editor` 标注 `#[component]`，定义 Props 和 Event，实现 mount、update 及 GPUI Render。首次提交节点时宿主创建 Entity，更新期间保留；移除、类型或 epoch 替换释放 Entity 及其订阅和 Task。需要前台 Window/Context 的实例方法用 `ViewCommand::new` 声明，生成类型化 ref；异步应用操作使用模块级 `#[command]`。

`NativeView::Event` 可用带标签枚举在一个类型化事件流中表达多种变化。复杂编辑器、树表和虚拟列表的 delegate 与瞬态状态由 Rust 拥有，不必将每个内部状态变化映射为 JS 属性。大数据集和高频输入使用批量 DTO 或 Rust 模型，避免逐项往返。

`NativeView::controlled()` 通过 `ControlledBinding::value_props` 声明可替代的受控数据属性列表，
这些属性共享一个编辑序号事件和确认属性。只要任一属性存在，生成绑定就保持内部订阅，
即使应用没有回调。Input 使用 `["value", "content"]`，让原子 token 快照与普通文本遵循
相同确认路径。该元数据由宿主生成；修改后重建所有宿主目录，不要手工编辑生成的 TypeScript。

原生滚动视图可通过 `NativeView::scroll_viewport()` 返回其拥有的 `native::ScrollViewport` 克隆（基于 `ScrollHandle` 或 `ListState`）。宿主在原生提交协调后发布此能力；`NativeChildren::content().scroll_viewport()` 只解析一个直接子项，不绘制行，也不在命令执行期间重新借用根实体。装饰器持有 `ScrollViewport::decorate()` 返回的租约；`is_decorated()` 为真时，视图省略自己的滚动条。内容、方向变化或装饰器卸载时释放租约。句柄只留在 Rust，不进入生成的 DTO。

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

原生组件 props 和命令 DTO 中，值为 `undefined` 的可选对象字段会递归省略。例如 `{ items: [{ key: "mode", label: "Mode", description: undefined }] }` 编码时不包含 `description`，不需要应用自行清洗。数组中的 `undefined` 和稀疏项仍被拒绝，不会转换为 `null`。这不改变 QuickJS 独立的[捕获状态契约](capture-state.zh-CN.md)。

每个 DTO 限制 1 MiB，必须是普通 JSON 数据。循环、非有限数、安全整数范围外的整数、BigInt、类实例、未知字段和无效枚举均被拒绝。JS 回调、GPUI Entity 和线程对象不跨运行时边界。领域约束应写在 Rust DTO 的 Deserialize 实现中，例如内置 Percentage 的 0–100 验证。

运行 `bun run task native-codegen` 生成核心组件和 website 绑定，运行 `bun run task native-codegen-check` 检查漂移。提交生成文件以支持编辑器与验证，不要手工修改。Vite `native` 选项调用相同宿主导出器并提供 `#native` 别名。Rust 变化需要重建并重启宿主。

可运行示例 `examples/website/native/src/lib.rs` 声明 BuildBadge 和 analyze_workspace，website TSX 使用生成组件与 Promise 客户端。设计依据见 [ADR-0016](adr/0016-rust-owned-native-modules.md)。

`@solid-gpui/vite` 从 Cargo 的实际可执行文件导出 bindings。由 Rust 启动 Vite 时使用当前宿主；直接 Bun JS 也能导入生成文件，无需打包器，Native Contract 不变。配置与生命周期见 [Vite 集成](vite.zh-CN.md)。
