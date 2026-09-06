# Rust 原生组件接入：更小的接口、更完整的能力

研究日期：2026-09-05 至 2026-09-06。本文保留最初的设计探索；随后已完成实现并采纳 [ADR-0016](../../docs/adr/0016-rust-owned-native-modules.md)。当前可编译 API 见 [Rust 组件与 JavaScript 调用](../../docs/rust-bridge.md)，运行时实现与真实验收见 [最终核对](runtime-audit.md)。下文草图不作为当前 API 文档。

基线：工作区 HEAD `4698b0e0b188c34b599ad0275717d17a329c79af`，存在大量既有未提交修改；本文依据当前文件内容，而非声称所有结论属于该 commit。实际依赖为 `gpui-pre 0.3.3`、`gpui-component 0.6.0 @ 928c3eb776a3d733d9b771f7dea27a6a79242ced`、`ts-rs 12.0.1`。不以 `references/gpui-component` 的另一套 shell 实现代表当前桥接能力。

## 推荐结论

采用 **Rust 源码声明导出 + 同一 Host 导出绑定 + 原生实例生命周期**。

用户在一个 Rust 应用模块里写组件和函数；一个开发命令自动编译 Host、生成一个本地 `#native` TS 模块并运行应用。JS 使用生成的 JSX 和 typed client。框架负责属性编解码、身份、注册、事件、命令、Host 配置和生成过程。

普通组件用函数，复杂组件用 GPUI 熟悉的 `Entity<T>` / `Render`，二者共享原生组件契约与传输。gpui-component 做内建可选集成，常用控件的适配由框架写一次。

目标是一个框架 Rust 依赖、一个框架 npm 包；应用自己的 Rust crate 和本地生成文件不是新的发布包。`solid-js`、Vite 和业务库仍按需依赖。框架内部保留 Rust 必需的 proc-macro crate 和可选底层 Bun 构建 crate。

这比只合并目录或替换宏语法更有价值：用户不再参与协议装配，Input/editor/list 等组件获得真实的原生状态承载能力。没有发现能直接替换整个 Solid + GPUI 桥接且同时解决这些问题的现成库；成熟方案提供的是可借鉴或复用的部分机制。

## 1. 当前复杂度具体来自哪里

| 当前事实 | 用户负担或能力缺口 | 推荐改变 |
|---|---|---|
| `declarations.rs` 每项填写 props/adapter/decode/render/entry 等名称、数值 ID、版本、字段 ID、事件 ID | 应用作者必须理解生成器的内部符号与协议身份 | 从导出的 Rust 项生成内部符号和确定性身份 |
| schema、provider renderer、Host、TS 包分离 | 修改一个控件需要跨越多个配置和生成入口 | 同一个 app/module 声明参与编译、注册和导出 |
| 组件使用 `FieldType`，普通函数使用 serde + ts-rs | 两套类型能力、契约和生成流程；组件只支持有限标量/bytes | 统一导出模型，内部区分组件、服务函数和实例命令 |
| `native-codegen.ts` 固定 Workbench exporter 和 Gallery 输出路径 | 示例结构成为框架机制的一部分 | Host 导出实际能力，生成位置由应用配置一次 |
| `gpui-component-codegen.ts` 同时承载契约校验、TS 模板和大量通用运行时文本 | 每个生成包带通用 runtime；维护位置分散 | 通用 runtime 在 npm 主包内，生成物仅含应用类型与薄描述 |
| `ExtensionAdapter` 仅 `validate/render` | 缺少 Entity、订阅和任务的挂载、更新、销毁语义 | 框架拥有按实例隔离的原生状态 |
| render context 没有 Window/App/Context，commit 入口主要更新 root | 不能仅加一个 TS wrapper 就接入 InputState | 改造窗口 foreground 的实例应用过程 |
| provider Host 默认注册 WorkbenchApi | Gallery 业务侵入通用 Host | 业务函数回到 Gallery 应用模块 |

源码：[组件声明](../../crates/solid-gpui-gpui-component-schema/src/declarations.rs)、[适配宏](../../crates/solid-gpui-bridge-schema/src/adapters.rs)、[原生渲染](../../crates/solid-gpui-gpui-component/src/lib.rs)、[Extension runtime](../../crates/solid-gpui/src/renderer/extensions.rs)、[函数桥接](../../crates/solid-gpui-bridge/src/lib.rs)、[函数生成脚本](../../scripts/native-codegen.ts)、[组件生成脚本](../../scripts/gpui-component-codegen.ts)、[provider Host](../../crates/solid-gpui-gpui-component-host/src/lib.rs)。

已有系统的价值也要保留：Solid getter 的响应式属性、Snapshot/Patch 的原子验证、Surface/epoch 隔离、listener generation 和受限字节协议。这些是框架应当隐藏的复杂度，不是应当删除的能力。

### 一个会推翻“自动 setter 就够了”的实例

锁定的 gpui-component 中，`Input::new` 需要 `&Entity<InputState>`，`InputState::new` 需要 Window 与实体 Context。更关键的是，`set_value` 会重置 selection、LSP 状态、滚动并清空 undo。

因此，自动生成 `props.value → state.set_value` 并在每次受控回传时执行，会破坏真实输入行为。普通类型反射无法知道这是一次用户输入的确认，还是有意的程序性替换。

一手源码：[Input 构造](https://github.com/longbridge/gpui-component/blob/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/component/src/input/input.rs#L167)、[set_value](https://github.com/longbridge/gpui-component/blob/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/base/src/input/base/state.rs#L834)、[InputState 构造](https://github.com/longbridge/gpui-component/blob/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/base/src/input/base/state.rs#L4978)。这些结论已从 Cargo 实际 checkout 读取，不依赖网页摘要推断。

本仓库的 [input.rs](../../crates/solid-gpui/src/renderer/input.rs) 已有 `edit_seq/ack_edit_seq` 和 marked text 保护，应把这种受控同步语义提炼复用。

## 2. 外部方案给了哪些答案

详细一手来源和版本注意事项见 [binding-sources.md](binding-sources.md)。

| 方案 | 值得采用的机制 | 对本项目的判断 |
|---|---|---|
| napi-rs | Rust 注解推导调用封装、TS 类型与类接口 | 借鉴作者体验；Node-API/env/异步运行时不负责 GPUI UI 线程、Surface 或原子树提交 |
| Tauri + Specta/tauri-specta | 一份 Rust 注册描述生成 dispatch 与 typed client，结构化类型图 | 最接近普通命令作者体验；采用模式，不能将 Tauri runtime 当作 GPUI 组件 runtime |
| UniFFI | Rust 导出描述、对象生命周期、跨语言绑定生成 | 借鉴统一契约；其对象线程约束和主要语言后端不等于 GPUI Entity 的语义 |
| wasm-bindgen | 从 Rust 类型与注解生成 JS glue、隐藏 ABI | 适用于 Wasm；不直接承载现有原生 GPUI Host |
| React Native Fabric | native component 的 props、events、commands、持久 Host View | 借鉴能力分层与实例模型；不搬 TS-first specs、多平台胶水或 JSI 指针模型 |
| flutter_rust_bridge | Rust 对象作为 opaque 能力、自动化绑定 | 借鉴 typed handle 思路；本项目 handle 必须受 Surface/epoch 管理，不能跨进程传裸指针 |

Fabric 官方将渲染、提交、挂载区分，并保留原生独有状态；这支持“组件绑定必须包含实例管理”的判断，但不是本仓库实现或性能的证明。[Render, Commit, Mount](https://reactnative.dev/architecture/render-pipeline)

Fabric 也为具体 native view 提供生成的 commands，说明实例方法应有明确目标对象。[Native Commands](https://reactnative.dev/docs/the-new-architecture/fabric-component-native-commands)

flutter_rust_bridge 的自动 opaque 类型面向 Dart/Rust 智能指针；它启发的是受控对象代理，不意味着 GPUI 对象可以跨任意线程。[RustAutoOpaque](https://cjycode.com/flutter_rust_bridge/guides/types/arbitrary/rust-auto-opaque/overview)

## 3. 三个独立设计方向的比较

| 设计 | Interface | Depth / Locality | 取舍 |
|---|---|---|---|
| A：应用模块自动导出 | app/module 宏、组件/函数标记、`#native` | 一处 Rust 声明影响实际 Host 和 JS；无需二次登记每个函数 | 多文件/跨 crate 需要显式模块组合，不能假装任意源码自动发现 |
| B：原生实体接口优先 | typed props + mount/update + GPUI Render + commands | 对 editor/list/输入的能力最完整，状态与行为集中在组件里 | 若强迫每个 Progress 都实现生命周期，会再次繁琐 |
| C：第三方 builder 自动映射 | 一个受限的组件映射表，生成 setter/event glue | 批量接入普通库控件方便 | 方法签名不能推导 controlled、资源所有权、同步回调与线程语义 |

推荐以 A 为公开主入口，B 为其有状态实现能力；C 仅用于框架内部已明确语义的机械映射。不要先造一个覆盖 GPUI 所有 builder 的新 DSL，更不要要求用户为每个控件写 descriptor/registry/decoder 三套接口。

来源附录还讨论了 `NativeScope` 的 keyed state 写法。最终不将 render 内 `scope.entity(...)` 作为默认有状态接口：它容易隐藏创建/更新副作用与提交时机。优先采用明确 mount/update + 原生 Render；若以后加入便捷函数式状态接口，也必须落到同一个实例生命周期，而不是新增第二套状态管理。

## 4. 用户最终应该怎样写

以下全部是目标 API 草图，当前仓库不能直接编译运行这些注解或 `#native`。

### 普通组件与函数

```rust
#[solid_gpui::app(components = "gpui-component")]
mod app {
    use solid_gpui::prelude::*;

    #[component]
    pub fn greeting(
        name: String,
        on_press: Event<()>,
        cx: &mut ElementContext,
    ) -> impl IntoElement {
        gpui_component::button::Button::new(cx.id())
            .label(format!("Hello, {name}"))
            .on_click(move |_, _, _| { on_press.emit(()); })
    }

    #[command]
    pub fn greet(name: String) -> String {
        format!("Hello, {name}")
    }
}
```

```tsx
import { Greeting, useNative } from "#native";

export function Page() {
  const native = useNative();
  return (
    <Greeting
      name="Ada"
      onPress={async () => console.log(await native.greet("Ada"))}
    />
  );
}
```

约定：组件名称默认 PascalCase，属性/函数默认 camelCase；覆盖名称是例外。普通参数成为 props/请求，`Event<T>` 成为 JS callback，context 由框架注入。生成器不序列化 callback 或 context。简单值不需另写 DTO；嵌套 struct/enum 用统一 `NativeType` derive。可选、默认值、范围、只在挂载时应用等仅在确有含义时标注。

`useNative()` 在 Solid setup 捕获当前 Surface client；之后的异步回调使用这个已绑定 client，不在全局函数里猜“当前窗口”。没有 owner 的地方使用显式 root client。Surface 释放或 epoch 失效后，调用明确失败。

同一个模块宏收集自己实际包含的导出项，不靠 proc macro 全局可变状态，也不遍历全部依赖源码。跨文件模块提供生成的模块描述，应用组合一次；跨 crate 库显式安装一次模块。重复导出名在构建时失败。模块注解遵守实际 cfg/feature，未启用的组件不能进入 TS 导出。

### 有状态组件

有状态组件保留 GPUI 的原生写法，额外表达初始 props、更新和可调用方法。下面省略业务实现，展示接口职责：

```rust
#[component]
impl Editor {
    fn mount(props: &EditorProps, events: Events<EditorEvent>,
             window: &mut Window, cx: &mut Context<Self>) -> Self {
        // 创建一次输入/文档 Entity，持有 subscriptions 和 tasks
    }

    fn update(&mut self, props: &EditorProps,
              window: &mut Window, cx: &mut Context<Self>) {
        // 应用确实变化的业务属性；遵守受控编辑同步语义
    }

    #[command]
    fn focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // 在 foreground 聚焦真实输入实体
    }
}

impl Render for Editor {
    // 使用持久 Entity 正常 render；不在此处重建 state、订阅或任务
}
```

JS 获得 `<Editor ... ref={...} />` 与 `EditorRef`，`await editor.focus()`。Ref 是有生命期的远程能力，不是 GPUI 指针。复杂组件必须自己表达业务更新语义；框架生成序列化、注册、生命周期调度、事件和方法代理。

## 5. 框架内部必须做好的事情

### 同一个导出模型，三种执行目标

内部契约统一描述类型、props/defaults/constraints、events、slots、commands、身份和限制。生成同一模型的 Rust glue 与 TS facades。执行仍区分：

- 组件创建/更新/实例方法：窗口 foreground，可访问 GPUI Context。
- 普通计算函数：后台执行，不允许捕获窗口实体；JS 返回 Promise。
- async 服务：显式受支持的执行器与取消策略；`async fn` 不意味着 Tokio reactor 自动存在。需要 Tokio 的业务服务应明确配置运行时，不能给每个命令隐式造一个。

类型层优先考察 Specta 的结构化类型图复用，具体选择与版本风险见来源附录。不能把 TS 类型字符串当作可逆的 schema，也不能把 Type derive 当作运行时 decoder。无论内部选哪种库，用户只看一个 NativeType 接口。迁移必须验证 serde 的 rename、tagged enum、Option/default、输入/输出差异与实际传输一致。

继续使用当前 Bebop envelope 和 bounded bytes。标量属性可保留紧凑 ExtensionField 路径；更丰富的嵌套 DTO 必须有显式的生成 codec 与深度/大小限制，不能通过不受约束的 JSON/any 偷渡。复杂 DTO 的具体 wire 扩展在原型中确定，不因作者 API 的简化仓促替换已经测量过的整套协议。大文档/大表的原生模型留 Rust，UI 事件传语义变化；不能每次按键传整个模型。

### 身份与生成物

数字 ID 不再手工维护。基于模块命名空间和规范化导出名确定性分配紧凑 ID，完整 digest 覆盖映射、类型、默认值、约束、事件、slots 与命令。自动分配不依赖源码读取顺序或 linker 顺序。不对新旧生成物做宽松兼容；不匹配就拒绝。

现有手工 tombstones 和 entry version 的维护可撤去，保留机器校验的契约身份。若某些现有 wire 字段暂时保留，值由生成器生成，不进入作者接口。删除 wire 字段需要显式更新协议与 ADR；不能仅删除 JS 字段宣称完成。opaque 自定义校验器的语义无法自动反射，必须作为显式契约策略声明；任意 Rust 函数体 hash 不是稳定的类型契约。

### 原生实例与原子提交

Host 用 `(surface, epoch, node, incarnation, component contract)` 标识实例，持有 Entity、订阅、任务和 typed props。

1. 对候选 Commit Batch 完成全部 topology、contract、props、slot 校验，生成 typed updates；不修改活实体。
2. 在窗口 foreground 完成发布和有效更新。只为新增实例 mount，仅对变化应用 update；节点移动和兄弟变化不 remount。
3. 按 GPUI Render 读取 typed props；避免每帧重新进行 wire 解码和注册表构造。
4. 节点移除、组件类型变化、epoch 替换、窗口关闭：先撤销 events/refs，再按 RAII 释放实例、订阅和任务。

当前 Host 的 commit 入口还需要调整为能取得 Window 的 foreground 更新过程；单独扩充 render context 不足以完成这项工作。

mount/update 约定为验证后的不可失败 UI 转换；外部 I/O 的失败成为组件自身状态或 typed error。框架不承诺回滚任意 Rust panic、文件写入或业务副作用。若未来引入 fallible mount，必须增加候选实例 staging 和清理设计，而不是偷偷破坏整批验证不变量。

### 事件与 Ref 的撤销

当前 `ExtensionEventSink` 固定 node/listener，但发出时读取共享 Cell 的当前 epoch/revision。它不能原样长期保存在 retained subscription 中：旧实例回调可能被贴上新 epoch。

新的 emitter 必须捕获不可变 surface/epoch/incarnation，并可撤销；每次有效提交更新活实例 listener binding。事件发生时冻结 revision/listener generation，入队后不改标。旧事件按已有 current/previous 规则处理；卸载之后发生的回调不能冒充新实例。提交过程中的事件应按提交后的明确次序释放。

实例 commands 携带相同生命期身份；未挂载、已卸载、换 epoch、跨 surface 的调用明确 reject。命令不能越过尚未提交的创建/props 更新：默认等待对应 commit revision 生效，再在 foreground 顺序执行。后台结果返回时再次验证目标仍存活。

### 输入、children 与列表

- 输入：同值回显不重置 selection/IME/undo；序列与确认避免旧 JS 值覆盖新编辑；程序性 reset 是显式命令。
- slots：保留 Host Node 身份和 Solid Owner Tree；每次 render 构造 fresh AnyElement，不保存上帧 element 或 JS closure。子内容变化能正确使 retained 父失效。
- 初期普通 children 可以复用已有有序树；具名 slots 是真实的 schema/runtime 工作，不能用 `Vec<AnyElement>` 冒充完成。
- 列表：原生保留 ListState，JS 行 owner 按已提交可见范围创建；GPUI 的同步 render-item 闭包不能阻塞等待跨进程 JS。
- 高频事件只在消费者订阅时安装；unsubscribe、listener 替换与 Patch 必须覆盖，不是初次 Snapshot 通过即可。

## 6. 构建流程如何真正变简单

```text
一个 dev/build 命令
  → 编译包含 app/module 契约的 Host
  → Host --export-native（尚未初始化 GPUI/Bun/业务）
  → 原子写入 .generated/native.ts + 类型/manifest
  → #native 映射到上述文件，进行 JSX transform/typecheck/bundle
  → 同一 Host 运行 JS
```

用户不写单独 exporter binary，也不手动先跑三个 codegen 命令。TS 编辑器读取真实生成文件；不依赖仅 Vite 能理解、tsc 看不到的动态模块。生成器不变时不重写输出，避免无效 reload。

明确不采用：proc macro 直接写 TS 文件；app 自己的 build.rs 里递归 cargo run 自己；根据 d.ts 反向推导 Rust decoder；启动后才发现 JS 与 Host 契约不一致。

Rust proc macro 必须来自独立 proc-macro crate，操作的是 token stream，并不是能查询所有依赖类型和方法的完整类型反射服务。[Rust Reference](https://doc.rust-lang.org/reference/procedural-macros.html)

build.rs 在 package 编译前执行，因此“引用本 crate 编译后的登记表来生成本 crate”不是可用的构建顺序。[Cargo Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)

此流程的代价要说清：首次导出需要编译完整 Host，比今天轻量 schema crate 更重；增量编译和绑定缓存改善后续成本，但不能未经测量承诺更快。纯前端开发可使用与 Host 配套的预生成契约产物，CI 仍校验实际 Host。缓存键含实际 target/feature/编译输入与生成器版本。

交叉编译目标 Host 未必能在构建机执行。默认使用各平台原生 CI 产出绑定/manifest；需要 cross build 的发行流程必须提供 target runner 或可验证的目标契约产物，不把 macOS 导出无条件用于 Windows cfg。这个限制是单 Host 导出的真实成本。

生产 bundle 可作为应用资源；必须内嵌二进制时做显式第二阶段打包。第一阶段 Host 不能依赖尚未由自身绑定生成的 JS bundle，否则循环。第二阶段再次比对契约。

TSX-only HMR 沿用现有新 epoch 机制。Rust 修改重编译、重新导出并重启 Host；失败时报告错误，不能把旧 Host 和新绑定混合。digest 不变也不表示 Rust 实现能热替换。

## 7. 包怎么减少

| 当前 crate/package | 推荐落点 |
|---|---|
| `solid-gpui` | 唯一公开框架 Rust crate，内部 modules 分工 |
| `solid-gpui-host` | `solid-gpui::host` 与标准启动入口 |
| `solid-gpui-bridge` | `solid-gpui::native` |
| `solid-gpui-bridge-schema` | 契约运行时进 native；语法处理进内部 macros |
| `solid-gpui-gpui-component-schema` | 删除独立 schema crate，由实际组件导出元数据 |
| `solid-gpui-gpui-component` | 主 crate 的可选 feature/module |
| `solid-gpui-gpui-component-host` | 删除独立 Host，配置启用集成 Root/overlays/init |
| `solid-gpui-workbench-api` | Gallery 自身 Rust 模块 |
| `solid-gpui-bun` | adapter 移入主 crate；必要底层 FFI/build 独立为内部 bun-sys |
| 新的 `solid-gpui-macros` | 独立 proc-macro crate，由主 crate re-export |
| npm core、gpui-component、vite | 一个框架 npm 包；runtime、components、vite/dev 以 subpath 暴露 |
| `#native` | 应用本地生成文件，无 package.json、无需发布 |

当前 `solid-gpui-bun → solid-gpui`，直接让主 crate 依赖原 bun crate 会形成循环。必须把低层 FFI 与 RuntimeAdapter 实现拆开后倒置依赖，不是加一个 re-export 就结束。

不启用 gpui-component 的应用不编译该可选依赖。启用时集成模块必须安装真正的 Root、overlay、主题和 keybindings；多个需要不同 window root 的集成不能假装自动可组合，应有明确唯一 root policy。框架内部配置这一处，普通用户不用实现 HostProfile。

npm 的 `/vite` 等构建入口不能被 runtime 入口静态引入；Vite 作为构建期可选 peer 管理。多窗口主题/provider scope 仍由实际 Host 支持，不从 npm 导出范围猜测能力。

`gpui-iconify`、`gpui-performance` 有独立 GPUI 用途，本轮不为追求数字把它们吞并。减少的是应用接入要管理的框架包，而非把整个 workspace 堆进一个文件。

## 8. 应该删除哪些代码，保留哪些代码

自动化并撤去作者负担：手写数字身份、decode/render/adapter 的命名串、重复类型定义、注册 match、独立 schema 包、单独 exporter、生成 TS 的手工入口、通用运行时模板副本。

保留或集中一次：实际 GPUI 渲染、第三方事件语义转换、原生资源所有权、受控输入策略、范围/业务约束、Host root 集成。对于已经实现的第三方控件，用户不再重复这些映射。

不推荐直接生成第三方库全部 public 方法：一个 builder 的 `.child`、同步 render callback、输入状态 setter、异步工作、窗口命令有完全不同的执行语义。可以自动化已标明含义的简单字段，不能通过“全自动”把必要规则隐藏成运行时错误。

## 9. 最小且能否定方案的验证

研究不等于新框架已经成立。落地时先做以下纵向样例，不先生成全库控件：

1. **Progress + Button**：只新增组件源文件/声明，无额外 schema/TS/registry 手改；signal 更新、listener 替换与移除都经过 Snapshot/Patch。
2. **真实 gpui-component Input**：一次 mount；同值回显、连续输入延迟确认、节点移动后 selection/IME/undo 保留；显式 reset 符合定义。真实窗口确认输入和 focus。
3. **实例销毁竞争**：删除、HMR、重建同 node ID、延迟事件/后台完成/ref command；旧实例不能触发新回调，Promise 确定完成或拒绝，任务/订阅释放。
4. **原子验证**：同批正常节点 + 非法 props/slot；无部分 publish、mount 或旧状态修改。
5. **原生列表/子内容**：可见范围、过滤/重排/resize、slot 更新；创建数量受可见范围约束，不同步回调 JS，不重建整个列表。
6. **干净生成与发行**：没有生成文件时一个命令成功；同一实际 Host 的类型与运行能力一致；Rust cfg/feature、JS HMR、process/embedded、生产二阶段打包都有关键验证。

记录作者需修改的独立位置、直接依赖数、干净/增量构建时间、每次 props 更新的 decode/mount 次数，以及输入到呈现路径。只有实际相同场景测量后才能声称性能改善。不要拿宏展开快、codec 单测快代替原生体验。

当前仅运行 `bun run task gpui-component-codegen-check`，退出码 0；它证明旧生成物一致，不证明新提案可运行。没有为研究修改产品代码或增加无意义测试。

## 10. 迁移顺序与既有决策

先以应用模块和实际 Host 导出替换两套作者流程，迁移现有四个控件与 Workbench；随后用 Input 打通原生实例生命周期，修正 emitter/ref 撤销；再验证列表/slots 和复杂 DTO；最后删除旧 schema/host/bridge 包与兼容入口，完成 npm 合并。迁移分步是实现顺序，不是长期保留两套用户接口。

不等待全库控件接入才判断设计。Input 的受控语义、foreground 实例提交、干净构建和实例撤销是决定方案是否成立的先行条件。

- 与 ADR-0001 的原子 Commit Batch、ADR-0012 的 Host-owned input 一致。
- 与 ADR-0015 的 Rust 修改需 restart、新 epoch 清理一致；不承诺原生 Entity HMR 保留。
- ADR-0014 的 Bebop 与严格验证保留；若去掉 entry version、改变 Extension 值形状、增加 slots/instance command，需明确更新 canonical schema、goldens 和该决策，不能隐式绕开。
- 本文是建议，不修改这些 accepted ADR；实现并通过上述验收后再记录最终决定。

关键源码与已有文档：[CONTEXT](../../CONTEXT.md)、[Rust bridge](../../docs/rust-bridge.md)、[ADR-0001](../../docs/adr/0001-batch-renderer-commits-into-gpui.md)、[ADR-0012](../../docs/adr/0012-host-owned-input-models.md)、[ADR-0014](../../docs/adr/0014-bebop-v5-generated-wire-protocol.md)、[ADR-0015](../../docs/adr/0015-vite-bun-native-hot-reload.md)。
