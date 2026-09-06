# Rust → JS 自动绑定研究与最小调用者成本方案

调研日期：2026-09-05 至 2026-09-06。范围：官方文档、维护者源码与 Rust Reference；以下设计为建议，不代表仓库已有接口，也未做可运行原型。本文包含独立备选方案，最终取舍以 [综合报告](report.md) 为准。

## 结论

推荐借鉴成熟工具的“普通 Rust 声明 + 注解 → 一份元数据 → Rust 分发与 TS 绑定共同生成”设计，保留现有 byte-only transport、Solid owner 和 GPUI foreground 执行规则。现有工具各自提供其中一部分，没有一个直接提供 Solid JSX、GPUI 原生实体生命周期和当前传输协议三者的完整适配。这个判断是下列资料与给定架构约束的工程推论，不是上游工具的自述。

| 技术 | 一手资料确认的能力 | 可借鉴/复用 | 不能据此承诺的能力 |
| --- | --- | --- | --- |
| napi-rs | Rust struct/impl 自动导出 JS class、getter/setter、TS；async fn → Promise；线程安全 callback 桥接 | 注解体验；值对象与有身份对象分开；自动代理和错误转换 | 自动导出 gpui-component 的任意 builder；自动让 GPUI 状态跨线程；直接沿用 JS GC 作为 Surface 生命周期 |
| wasm-bindgen | Rust exports → JS class；引用参数；TS 定义；Future/Promise 互通 | 单一声明来源、生成器流水线、显式所有权设计 | 原生 GPUI render 迁移到 Wasm 后仍可原样运行；替换现有 native byte bridge |
| UniFFI | proc macro 或 UDL → Rust scaffolding 与外语代理；对象 Arc 引用；异步 FFI | 元数据 IR、值/对象区分、明确资源释放 | 官方即用的 embedded Bun 后端；任意 foreground-only GPUI 对象作为 Send+Sync interface |
| Tauri 2 + tauri-specta v2 | command、state、channel；同一 builder 注册 command/event 并生成 TS | 上下文注入、统一注册与导出、请求/事件分开 | 无 WebView 的现有 Host 直接换 tauri-specta；命令绑定等于 JSX 渲染生命周期 |
| Specta v2 | Type derive 形成类型图；自动收集依赖类型；TS exporter；序列化格式解释独立 | 作为代码生成内部类型模型的候选，减少手写 TS 类型生成 | 编译时反射第三方所有方法与语义；自动生成 GPUI adapter、children/event 语义及二进制 codec |

## 1. napi-rs：最值得借鉴的是使用方式

`#[napi] struct` 表示有身份、由 Rust 值支撑的 JS class；`#[napi(object)]` 表示复制的数据记录。构造器、公开字段、getter/setter、方法与 TypeScript 声明可生成，私有原生状态无需暴露。默认在 JS 对象回收时 drop Rust struct。这说明“很多胶水都能生成”，也说明 GC 绑定的寿命与 UI 显式卸载不是同一契约。[Class](https://napi.rs/docs/concepts/class)

async 导出需要启用相应 feature，默认使用 napi-rs 提供的 Tokio runtime，结果转 Promise；`async fn(&mut self)` 被明确要求标注 unsafe，因为 JS 同时持有对象。不能把 Promise 化理解成已经解决 UI 对象并发。[async fn](https://napi.rs/docs/concepts/async-fn)

`napi_env`、`napi_value`、`napi_ref` 不能任意跨线程访问；`ThreadsafeFunction` 把 callback 调度回 JS 环境，队列可限长，非阻塞模式会报告满队列。对本项目的推论：即使引入 N-API，foreground dispatch、事件背压、owner 释放仍需自己设计。[ThreadsafeFunction](https://napi.rs/docs/concepts/threadsafe-function)

Bun 官方支持加载 `.node`，文档声称实现约 95% Node-API。这个百分比不是当前嵌入式 Bun、目标平台和所用 N-API 子集的兼容性验收，不能直接作为改桥依据。[Bun Node-API](https://bun.sh/docs/runtime/node-api)

## 2. wasm-bindgen：自动绑定不等于任意原生反射

导出的 Rust struct 可对应 JS class，`T/&T/&mut T` 有各自映射；公开 Copy 字段生成 getter/setter，非 Copy 字段有 clone 属性或手写访问器。导出函数不能具有自己的泛型参数。这些都是被支持的显式导出契约。[Exported Rust Types](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/exported-rust-types.html)

`#[wasm_bindgen] async fn` 可输出 Promise，Rust Future 与 JS Promise 有转换工具；默认生成 TS 声明，可逐项关闭。值得借鉴的是构建时派生多个产物的工作流，而不是更换 GPUI 的原生运行目标。[Promises and Futures](https://wasm-bindgen.github.io/wasm-bindgen/reference/js-promises-and-rust-futures.html)、[TypeScript generation](https://wasm-bindgen.github.io/wasm-bindgen/reference/attributes/on-rust-exports/skip_typescript.html)

## 3. UniFFI：对象寿命与代码生成 IR 值得学，线程模型不能照搬

UniFFI 可用 Rust proc macro 或 UDL 描述导出接口，生成两侧胶水。官方完整支持的语言主要是 Kotlin、Swift、Python；其他绑定需按具体第三方实现确认。因此不应把它表述为官方开箱即用的 Bun 绑定方案。[UniFFI guide](https://mozilla.github.io/uniffi-rs/latest/)

对象通过 Arc 持有，外语对象是代理；接口要求 Send+Sync，不能通过 `&mut self` 暴露独占可变借用。这个模型适合共享业务对象，但不能因为加了 Arc/Mutex 就把只能在 GPUI foreground 操作的对象变成跨线程 UI 对象。[Interfaces, Objects and Traits](https://mozilla.github.io/uniffi-rs/latest/types/interfaces.html)

UniFFI 异步不强迫单一 Rust runtime：外语侧驱动 RustFuture 的 poll/complete/free；运行时依赖仍要适配。说明“导出 async fn”与“确定在哪个线程/执行器执行”是需要分别解决的问题。[Async overview](https://mozilla.github.io/uniffi-rs/latest/internals/async-overview.html)

## 4. Tauri 2 + Specta：与当前桥更相似的抽象方式

Tauri command 用函数注解暴露参数、返回值、错误与异步，并可注入 State、Window 等上下文；对于 GPUI 可借鉴“上下文不是 JS 参数”的做法。[Calling Rust](https://v2.tauri.app/develop/calling-rust/)

Tauri state 有应用持有的生命周期；可变共享状态通过内部可变性管理。它不能替代每个挂载原生组件的实体生命周期。[State Management](https://v2.tauri.app/develop/state-management/)

Tauri channel 明确服务有序流式数据；全局 event 不适合大量数据。这支持把命令、组件事件、持续数据流的语义分开，即使底层复用同一字节桥。[Calling the Frontend / Channels](https://v2.tauri.app/develop/calling-frontend/)

tauri-specta v2 的 Builder 用一份 commands 集合导出 TS 并产生 invoke_handler；typed events 也可注册与导出。当前维护者源码仍将 v2 标为 beta，示例精确锁定 `=2.0.0-rc.25`。`docs.rs/tauri-specta/latest` 实际指向稳定版 1.0.2，不可混用 v1 示例。引用 main 的地方只确认当前源码声明，不保证所有特性都已发布。[tauri-specta 源码文档](https://raw.githubusercontent.com/specta-rs/tauri-specta/main/src/lib.rs)、[版本对应表](https://github.com/specta-rs/tauri-specta)、[stable latest](https://docs.rs/tauri-specta/latest/tauri_specta/)

Specta 可以只登记根类型并收集依赖类型，其 TypeScript 使用指南当前给出 `specta = 2.0.0-rc.25`、`specta-serde = 0.0.12`、`specta-typescript = 0.0.12`。它是可复用的类型导出基础，不是运行时 bridge。评估时需要验证本项目的整数、Option、默认值、enum、输入/输出差异和字节数组是否与实际 wire shape 一致。[Specta TypeScript](https://docs.rs/specta-typescript/latest/specta_typescript/)

尤其不能把类型导出工具的结构类型理解为通用 Rust 反射。Rust proc macro 接收/生成 token stream；它没有从一个第三方类型名自动得到其所有 impl 方法及业务语义的通用能力。针对已知语法生成代码可行；自动理解 gpui-component 的所有 builder 和交互不成立。这是依据宏输入模型的推论。[Rust Reference: Procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html)

### 类型工具选型定论

建议把 **Specta v2 作为统一内部类型图的优先 spike 候选**，但现阶段不承诺直接替换已用的 ts-rs。此处 ts-rs 12 与独立 FieldType 的现状来自主研究的代码核对，本子研究未独立验证其完整实现。原因是本次目标包括嵌套 DTO、输入/输出 wire shape 与框架 exporter，Specta 的类型图与独立 Serde 格式解释更贴近这个需求；只更换 TS 字符串生成器并不能统一契约。当前 v2 仍是 RC，必须精确 pin 并验证依赖升级成本。[Specta TypeScript 使用方式](https://docs.rs/specta-typescript/latest/specta_typescript/)、[Serde-aware phased types](https://raw.githubusercontent.com/specta-rs/tauri-specta/main/src/lib.rs)

通过 spike 的条件：一个共享 Rust DTO 含嵌套 struct/enum、Option/default、rename、bytes 和大整数，其 JS 输入类型、Rust decode、Rust encode 与 JS 输出类型严格对应；组件 props 与命令 DTO 走同一受约束类型集合。**TypeScript 导出不等于运行时 codec 生成**：Specta 负责类型元数据；Serde 负责 Rust 序列化语义；框架仍须提供/生成匹配的 JS codec，并处理订阅句柄等不属于普通 DTO 的字段。若现有 ExtensionValue 仅支持标量列表，必须扩展或重构 wire 表达，不能靠 TS 类型声明伪装嵌套数据支持。

若 spike 无法证明一致性，就先保留 ts-rs，把 authoring 和生命周期简化独立落地；不要同时更换类型工具、字节编码与生命周期而没有可定位的验收。保留 ts-rs 是控制改动面的临时工程选择，不意味着继续长期维护互不相干的 DTO 与 FieldType 两份真相。

## 5. 独立备选设计：普通 Rust 函数就是组件导出

这是供比较的第三种设计方向，目标是调用者学习成本最低。强调 Interface 的 Depth：开发者只看到组件参数、事件和正常 GPUI 构造代码；协议和生命周期由一个深 Module 实现。以下代码是设计草图，不是已支持语法。

```rust
#[native_component]
fn Checkbox(
    #[prop(default = false)] checked: bool,
    #[prop(default = false)] disabled: bool,
    on_change: Event<bool>,
    scope: &mut NativeScope,
) -> impl IntoElement {
    gpui_component::checkbox::Checkbox::new(scope.id())
        .checked(checked)
        .disabled(disabled)
        .on_click(move |next, _, _| on_change.emit(*next))
}
```

宏从同一函数参数生成 props 类型、事件描述、分发/解码以及 TS JSX 导出。`scope` 是 Rust 注入参数；`Event<bool>` 只在 Rust 表示可发送的事件端点，JS 函数本体仍留在 Solid owner。`Event::emit` 的错误/队列策略必须由框架明确，草图没有暗示传输失败可以忽略。

```tsx
import { Checkbox } from "native:app";
<Checkbox checked={enabled()} onChange={setEnabled} />
```

`native:app` 是建议的构建插件虚拟模块名。应用开发者在应用 Rust crate 内定义函数，Host 同一份显式导出集合完成注册与 codegen；不创建应用专属 schema crate、codegen crate 或 npm 包。库作者把多个组件的 Rust adapter 放进一个集成 crate，并给其同一导出集合一次性安装入口。

### 大量第三方 builder 如何减少手写

先采用普通函数，让 Rust 编译器检查 builder 调用。这里每个属性仅声明一次 wire 类型，render 中仅剩真实 `.checked(checked)` 映射。现在 adapter 中的 decode、事件是否订阅、sink 获取、emit 封装、registry 条目、TS wrappers 等应成为生成代码或通用实现，而不是每个函数再写一次。当前 [`render_checkbox`](../../crates/solid-gpui-gpui-component/src/lib.rs) 同时混有这些责任，是自动化的实际切入点。

只有确认几十个控件重复相同映射以后，才增加可选的 builder 映射宏。例如 `checked: bool = false => .checked`、`compact: bool = false => if_true(.compact)`、`size: ControlSize => .with_size(convert_size)`；一份声明同时生成参数和 builder chain。它应是集成 crate 的内部工具，不作为自定义组件作者必学的另一套 DSL。

不能省略的人工语义包括：

- `.compact()` 仅在 true 时调用，而 `.disabled(false)` 是显式设置；Option 是不调用 setter 还是清空已有值。
- `on_click` 的原生事件如何变成 `onChange(bool)`；不是所有 GPUI 事件或 Window/Context 都可跨 wire。
- children/slot 接纳方式、variant/size 枚举转换、哪些字段为 mount-only、受控与非受控状态。
- Input/editor 创建与更新时机、IME/选区保护、订阅和任务的销毁条件。

宏不应解析第三方源码猜语义，更不应把 Rust 泛型 builder 方法全部暴露给 JS。应导出用户需要的有限组件契约。

### 有状态组件与执行规则

函数形式本身不能保存 InputState。建议 `NativeScope` 对应一个真正的挂载实例，其持久化槽位能持有 GPUI Entity、订阅、任务；用显式稳定 key 如 `scope.entity("editor", ...)` 创建一次，直到卸载。所有 Entity 创建/更新/释放都在 GPUI foreground 执行；UI Context 不进入工作线程，也不跨 await 借用。该接口需要新增，不能只加 proc macro 假装生命周期存在。

状态更新不能简单“每 render 把所有 props 写回”：输入的命令、用户编辑、受控值与 revision 必须有明确先后关系。render 调用中的槽位 key/类型应稳定；复杂原生 view 可直接在槽位中持有一个已有 Render 实体，让普通 Rust 管理内部复杂度。不要为每个组件强迫写 mount/update/unmount 六件套，也不要用调用顺序推断槽位身份。

JS owner 负责 props 响应式计算与 callback；native instance 负责原生实体与任务。卸载/热重载 epoch 使旧命令和事件失效；一个实例的更新只使对应原生 view 失效。transport 只传值、实例/事件句柄与版本标识。不要沿用 N-API 的 JS GC 释放时机，也不要阻塞等待同步 JS 回调。

### 包的建议数量与真实限制

对应用作者：已有 Rust 应用 crate + 一个 JS runtime 包即可；生成文件或虚拟模块属于应用构建产物，不是新包。对 gpui-component 集成维护者：一个集成 crate 即可承载导出、Rust 构造、初始化以及 codegen 元数据。

框架可提供一个公开 Rust facade crate，重导出 proc macro；宏的实现仍必须在单独 proc-macro crate，这是 Rust 的实际限制，不能承诺物理上全合成一个 crate。codegen 可以是 facade 的构建/工具 feature 或 bin，无需为每个 provider 新建一组 crate。[Rust Reference](https://doc.rust-lang.org/reference/procedural-macros.html)

这个方向的代价是框架要一次性完成可靠的 NativeScope、导出元数据与生成器；最小原型应覆盖一个 builder 控件和一个真正有状态的 Input，验证属性更新不重建 Entity、卸载释放资源、旧 epoch 事件失效。不能只用 Progress 证明“全部接入已简单化”。
