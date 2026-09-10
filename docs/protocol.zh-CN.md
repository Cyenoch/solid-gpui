# Solid GPUI 协议 v5

本文描述 TypeScript 渲染器与 Rust/GPUI 宿主之间已实现的线协议。v5 是两端同步切换到 Bebop 的全新契约。权威 schema 位于 [protocol.bop](../packages/solid-gpui/src/protocol/protocol.bop)，已校验生成绑定分别位于 TypeScript 和 Rust 协议模块。schema-lock.json 将版本 5 固定到 SHA-256 `e0fcd0e7b6c78ce18dce79c5d5d54be7e6fe27a0c717d4b88132ca13f0c2e3b3`，漂移会使生成检查失败。普通包和 Rust 构建消费已提交文件，不调用 bebopc。重新生成与检查：

```sh
bun run task protocol-codegen
bun run task protocol-codegen-check
```

公开协议接口是 `packages/solid-gpui/src/protocol/types.ts` 和 `crates/solid-gpui/src/protocol.rs` 中的语义 DTO，生成的 Bebop 类型只留在编解码适配器内。领域校验手写，生成解码只负责结构解析。

Rust 在协议边界使用类型化语义消息：Command 包含 CommandMeta 和 CommandOperation；Event 包含 EventMeta 和 EventPayload，event_kind() 标识载荷家族。可选样式使用封闭枚举，语义缺失用省略/None 表示，不用零值兼容哨兵。

## 1. 帧与封套

传输帧结构：

```text
[u32 little-endian payload length][Bebop Envelope bytes]
```

四字节前缀只计算 Bebop 载荷，最大 `16 * 1024 * 1024` 字节。Rust read_frame/write_frame 保留精确读取、截断、大小边界和 flush 行为，TypeScript FrameDecoder 保留分片及合并输入处理。

根 Bebop 记录：

```text
Envelope {
  protocolVersion: u32 = 5,
  body: Body,
}
```

Body 稳定标签为 1=Snapshot、2=Event、3=Patch、4=Command。解码要求 protocolVersion 存在且为 5、一个已知 body union、无尾随字节，不尝试旧版解码。不提供版本协商、双解码器或宽松回退，两端必须共用 v5 契约。

Bebop 消息是带长度的记录，字段 ID 单调递增并以 0 结束；数组带有界 u32 数量，字符串和字节数组带有界 u32 字节长度。生成读取器执行前，Rust guard.rs 和 TypeScript bebop-guard.ts 的 schema 派生防护检查：

- 帧、消息、字符串、字节数组、重复项、总项数和递归预算。
- bool 严格为 0/1，枚举在声明范围内，UTF-8 错误为致命错误。
- 无重复、乱序或未知字段 ID。
- 仅已知 union discriminator，终止符、已消费长度精确且无尾随字节。
- 在重复项、字符串和字节解码分配内存之前执行各字段资源预算。

Rust guard 执行已提交的 generated/schema_guard.rs 静态计划，不在首次解码解析 JSON 元数据。结构解码后，语义校验器检查节点所有权、surface/epoch/revision、资源、样式、命令/事件所有权和发布约束。畸形 Snapshot/Patch 不发布到保留树。

Extension 的完整身份为 `(providerId, catalogDigest, entryId, entryVersion)`，宿主必须解析到适配器，并在发布前验证属性与子内容。未知身份按事务拒绝，因此默认 NoExtensions 拒绝全部 Extension，而不发布无法渲染的节点。

## 2. Snapshot 与 Patch

Snapshot 包含 surfaceId、epoch、baseRevision、revision 和完整 nodes；Patch 使用相同 revision 头及有序 operations。首个 Snapshot 的 baseRevision 为零，后续 Patch 匹配当前 revision，且 revision > baseRevision。

Node 携带 id、parentId、index、kind、全部 42 个 Style 槽、可选 RawText text、listenerId、可选带标签 HostProperties、AccessibilityProperties、focusable、selectable、tooltip 和 acceptsPointerMove。NodeKind 为 1=View、2=Text、3=Pressable、4=RawText、5=TextInput、6=VirtualList、7=Image、8=Extension、9=Icon。

HostProperties 包含：

- TextInput：值、占位文本、多行、禁用、受控状态、编辑序号、选区、marked range、最大长度和反向选区。
- VirtualList：项数、可见范围、估计尺寸和 overscan。
- Image：来源、object-fit 代码和备用来源。
- Drag：拖动类型、导出文件、接受 drag-over/drop 标记。
- Extension：16 字节 providerId、32 字节 catalogDigest、非零 entryId/entryVersion、有序唯一非零字段和事件 ID。字段支持 bool、i32、u32、有限 f32、有界文本和字节。字段和事件 ID 最多各 256 个；文本及字节的单值与合计均限制 1 MiB。

无障碍属性含 role、label、description、disabled、checked、selected、value、expanded、heading level。Style 含尺寸、flex/对齐、间距、颜色、透明度、transition、边框、字体、定位、光标、文本对齐、最多两个 BoxShadow 和字体族。数值布局在线上用 f32，ID、枚举码、mask 和数量用 u32，语义校验约束有限/非负范围及所有权。

可选枚举样式仅以 None/省略表示缺失。flexDirection 代码为 1..=4，textAlign 为 1..=3；Some(0)/wire 0 无效，不代表未设置。

Patch 操作为 1=Create、2=Update、3=Move、4=Delete。PatchUpdate 精确保留 mask 存在性：

|  位 | 字段         |
| --: | ------------ |
|   1 | style        |
|   2 | text         |
|   4 | listener     |
|   8 | 宿主属性     |
|  16 | 无障碍       |
|  32 | focusable    |
|  64 | selectable   |
| 128 | tooltip      |
| 256 | 指针移动能力 |

style 位设置时必须携带 style（设置）或空 clearStyle 标记（清除），未设置时两者均省略（不变）。focusable/selectable 同样在更新位未设置时省略，设置时必须存在，包括显式 false。位未设置却出现字段会被拒绝，省略字段转为语义 false 占位。带 mask 的可选字段区分省略清除与不变，显式 false/零保留。可选尾部仍是语义缺失，不是兼容路径。Create 携带完整 Node，Move 重新挂接/排序，Delete 移除子树，树校验和原子回滚保持权威。

## 3. 命令与命令值

Command 携带 surfaceId、epoch、afterRevision、requestId、nodeId、数值 kind 和类型化 CommandPayload。仅 surface、epoch、afterRevision 匹配当前树时准入。窗口入口在消费下一消息前执行命令，后续 Patch 不会让已准入命令过期。异步服务可以稍后完成并返回自身请求结果。布局查询和焦点遍历使用当前已绘制帧，树提交不等于新帧已绘制。

完整语义命令集合：

| 代码 | 类型                 | 载荷 / 所有权                                                  |
| ---: | -------------------- | -------------------------------------------------------------- |
|    1 | Focus                | null； 可聚焦节点                                              |
|    2 | Blur                 | null； 可聚焦节点                                              |
|    3 | SetSelection         | u32 start/end; TextInput                                       |
|    4 | ScrollToIndex        | u32 index/alignment; VirtualList                               |
|    5 | ScrollToEnd          | null； VirtualList                                             |
|    6 | SetTitle             | 文本； 根节点                                                  |
|    7 | ResizeWindow         | u32 width/height; 根节点                                       |
|    8 | ZoomWindow           | null； 根节点                                                  |
|    9 | ToggleFullscreen     | null； 根节点                                                  |
|   10 | OpenUrl              | 文本； 根节点                                                  |
|   11 | FocusNext            | null； 根节点                                                  |
|   12 | FocusPrev            | null； 根节点                                                  |
|   13 | GetWindowSize        | null； 根节点                                                  |
|   14 | GetFocus             | null； 可聚焦节点                                              |
|   15 | ClipboardWrite       | 文本； 根节点                                                  |
|   16 | ClipboardRead        | null； 根节点                                                  |
|   17 | OpenSurface          | title, u32 width/height, optional creation options; 根节点     |
|   18 | FileDialogOpen       | title, directories, multiple; 根节点                           |
|   19 | FileDialogSave       | default name 文本； 根节点                                     |
|   20 | ShowNotification     | title, body, optional action list; 根节点                      |
|   21 | SetMenus             | 完整菜单树; 根节点                                             |
|   22 | SetKeybindings       | 完整绑定列表; 根节点                                           |
|   23 | SetClosePolicy       | `allow` or `require-confirmation`; 根节点                      |
|   24 | ResolveCloseRequest  | request ID and allow bool; 根节点                              |
|   25 | ReadTextFile         | absolute path 文本； 根节点                                    |
|   26 | WriteTextFile        | 绝对路径与内容; 根节点                                         |
|   27 | ClipboardWriteImage  | format code and bounded bytes; 根节点                          |
|   28 | ClipboardReadImage   | null； 根节点                                                  |
|   29 | LoadFont             | absolute path 文本； 根节点                                    |
|   30 | MinimizeWindow       | null； 根节点                                                  |
|   31 | GetWindowBounds      | null； 根节点                                                  |
|   32 | GetWindowState       | null； 根节点                                                  |
|   33 | ActivateWindow       | null； 根节点                                                  |
|   34 | GetScrollOffset      | null； VirtualList                                             |
|   35 | ScrollToOffset       | finite non-negative f32; VirtualList                           |
|   36 | InvokeNative         | catalog identity, function ID, bounded args; root or component |
|   37 | CancelNative         | original invocation request ID; root                           |
|   38 | ConfigureApplication | lifecycle policy and acknowledged sequence; application scope  |
|   39 | OpenPopup            | anchor node, content size, placement, gap; owner root          |
|   40 | ClosePopup           | original OpenPopup request ID; owner root                      |

生成载荷 union 使用 u32 pair、f32、文本、字符串 pair、OpenSurface、FileDialogOpen、通知、菜单、快捷键、剪贴板图片和关闭确认等类型化记录。解码验证载荷变体与命令类型匹配，以及 root/node 所有权。

`OpenPopup` 要求已挂载锚点（node ID 至少为 2）、每轴 1–16384 的内容尺寸、0–11 的 placement 和 0–1024 的有限 gap。成功后返回新子 Surface ID。`ClosePopup` 在所属 Surface 与 epoch 内引用原始打开请求，因此子 ID 尚未返回时也能取消；重复取消幂等。参见[系统弹层](system-popover.md)。

命令结果以 Event 返回 request ID、command code、node ID、success、可选错误和 CommandValue。值标签为 1=number、2=pair、3=boolean、4=text、5=paths、6=file text、7=image、8=bounds、9=window state、10=scroll offset。文件、图片、剪贴板、路径、菜单、通知和快捷键限制在发布到 JS 前检查。

已退役 Surface 的合法在途提交不再发布，也不会终止应用。迟到的初始 Snapshot 会收到匹配的 SurfaceClosed，让客户端释放子根。此规则不接受从未分配的 ID，也不恢复已退役 ID。

## 4. 事件

每个 Event 携带 surface、epoch、revision、sequence、node、listener、数值 eventType 和可选类型化 EventPayload：

| 代码 | 类型                 | 载荷                                                |
| ---: | -------------------- | --------------------------------------------------- |
|    1 | Press                | null                                                |
|    2 | Change               | TextInput 数据                                      |
|    3 | Selection            | TextInput 数据                                      |
|    4 | Focus                | null or TextInput 数据                              |
|    5 | Blur                 | null or TextInput 数据                              |
|    6 | CommandResult        | 命令结果                                            |
|    7 | VisibleRange         | u32 start/end                                       |
|    8 | AnimationComplete    | u32 generation                                      |
|    9 | Key                  | key, modifiers, action down/repeat/up               |
|   10 | Pointer              | pointer down/up or pointer-move payload             |
|   11 | Hover                | null                                                |
|   12 | Scroll               | delta kind, f32 deltas/position, modifiers          |
|   13 | Submit               | text                                                |
|   14 | WindowResize         | f32 width/height/scale                              |
|   15 | WindowActivation     | active bool                                         |
|   16 | SurfaceClosed        | null； node/listener `0/0`                          |
|   17 | Action               | action 文本； 根节点/listener `1/0`                 |
|   18 | WindowAppearance     | light/dark; 根节点/listener `1/0`                   |
|   19 | Layout               | f32 x/y/width/height                                |
|   20 | Drag                 | drag-over, drag-drop, or external paths             |
|   21 | NotificationResponse | tag and optional action ID; 根节点/listener `1/0`   |
|   22 | PointerDownOutside   | f32 x/y                                             |
|   23 | CloseRequested       | request ID; 根节点/listener `1/0`                   |
|   24 | Extension            | event ID and sorted Extension fields; node/listener |

Focus/Blur 有意保留双形式：View/Pressable 焦点观察者省略载荷，TextInput 则携带编辑数据。Pointer down/up/move 是 Event 10 下的类型化变体；drag-over/drop/外部文件 drop 是 Event 20 下的变体。修饰键列表由唯一 cmd、ctrl、alt、shift、function 成员组成。坐标在契约要求处有限且非负，布局和滚动偏移保留 f32 语义。

宿主维持事件序列顺序，渲染器按 surface 检查 surface/epoch/revision。Listener ID 是 revision 拥有的回调标识，当前和上一代只为合法在途事件及脱离焦点保留必要时长，旧 revision 不会调用新回调。Event 24 先按 revision 解析 listener 代次，再要求已附着 Extension 节点匹配 node ID，event ID 在订阅列表中。仅订阅回调收到 `{eventId, fields, target}`，未知或未订阅事件忽略。SurfaceClosed 在宿主移除前发出。根 action、appearance、notification、close-request 保持原 node/listener 所有权。宿主文本选择和编辑状态不虚构额外 wire event。

## 5. 运行时所有权边界

Rust NativeStateRegistry 拥有 surface、窗口、焦点、输入和保留状态。CommitPump 是有界交接点，接收 runtime 提交载荷并在 registry 前台 owner 应用，只验证和发布完整 Snapshot/Patch 状态。

原生 Patch 使用被修改节点和子列表的事务日志，不再克隆整个节点存储。结构校验与 Extension 合约校验位于同一回滚范围，全部成功后才发布 revision 并更新原生状态。失败时恢复结构、派生文本与旧 revision。内部变更集合包含修改/删除的身份、原始与最终祖先，以及类型化子节点的归属依赖；校验、事件路由、原生实例更新和缓存失效共同消费该集合。结构变更可以访问被移动的兄弟节点和依赖子树，局部属性更新不复制无关节点。首次 Snapshot 仍完整验证树，v5 线协议不变。

TypeScript SurfaceRouter 是唯一帧解码和事件路由器，将输入 chunk 分组成各 surface 有序语义事件批次。HostTree 拥有私有 NodeGraph、事务日志、脏属性定稿和 Snapshot/Patch 生产；CommandClient 拥有请求 ID、待处理结果与终止拒绝。HostKind 事实模块拥有允许属性、子规则、投射类别和运行值能力。这些模块不向 Solid 组件暴露传输内部或生成协议记录。

## 6. 一致性与切换

`bun run task protocol-golden-check` 重新生成代表性的 v5 Snapshot、Patch、全部 40 种 Command、全部 Event 载荷（含双形式 focus/blur）、畸形案例及帧边界，提交 fixture 漂移则失败。TypeScript 生成 ts_to_rust.hex，Rust 独立构造同类语义数据生成 rust_to_ts.hex。Rust 解码并重编码每个 TS 行，TS 结构/语义解码每个 Rust 行并验证规范字节，两端执行 invalid.hex 和 frames.hex 期望。语义错误包含有界 body、operation、node、field 路径，预解码 guard 保持通用有界结构错误。

可复用带帧 Event writer 为每个 Rust 输出 worker 持有一个有界缓冲区，原地序列化，直接写四字节长度与载荷，flush 流块，避免每事件 payload Vec 加帧复制。TS 适配器使用可复用 BebopView 写入，只复制调用方拥有的返回传输帧。生成 runtime 不暴露给渲染器调用方。
