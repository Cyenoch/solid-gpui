# Solid/GPUI 项目对比调研

调研日期：**2026-10-01**。我们的基线：`Cyenoch/solid-gpui`，提交 `e9bc4389c5394e5de1fecb6758d2d06e48877fdb`。

本文件是 [英文报告](README.md) 的中文版本。判断基于实际源码与明确的验证范围，不按 star、README 功能数量或项目名称排名。

| 项目 | 固定提交 | 实际定位 | 最值得学 |
| --- | --- | --- | --- |
| heyhuynhgiabuu/solid-gpui | `05f84e9` | Solid 2 RC + 独立 Rust helper | 生命周期／GUI 验证、原生领域组件 |
| remorses/gpuix | `4ecca30` | N-API native 引擎，React 与自己的 Solid 1 adapter | 预构建分发、native automation、样式共享 |
| jhomra21/gpuix-solid | `ad384ff` | 基于 GPUIX 0.9 的 Solid 1/2 adapter | 独立消费检查、真实应用交互验收 |
| lxsmnsyc/solid-gpui | `196aa6e` | Solid 2 RC + 独立 Rust host | 编译器、样式归一化、recorded canvas |

**注意：这不是四个独立原生引擎。** jhomra21 使用 GPUIX native；当前 GPUIX 本身也有 Solid adapter，而且注明采用了 jhomra21 的部分 host 设计。jhomra21 的默认 native 仍为 0.9，而不是上游当前 0.10；它的实验性 Canvas/video 另需固定源码与 native patches。

## 总结

**我们的优势在底层正确性、原生状态所有权和完整应用能力；最值得补的不是更多功能，而是更简单的默认使用路径，以及更细的原生渲染边界。**

不能因为其他项目用 FFI/N-API，就认定我们的二进制协议做错了；也不能因为我们有增量 Patch，就认定原生更新已经足够细粒度。两种误判都应避免。

## 一、我们做得好的地方

### 1. 没有牺牲 Solid 的响应式语义

我们使用 universal renderer，节点变更由 Solid 驱动；初次发布 Snapshot，后续发布 Patch。`HostTree` 在事务结束时统一计算属性和结构变更，失败时回滚。Rust 侧也在验证成功后才发布新 revision。

这不只是“能把 JSX 画出来”，还处理了多字段一致性、移动/删除、失败提交和回调版本归属。应保留这套设计，而不是退回每个 setter 单独暴露一个原生中间状态。

回滚指 host graph／原生发布状态，不会把任意 signal 写入、文件 I/O 或已经执行的 native service 副作用一起撤销。

### 2. 把输入法、选区、撤销和滚动放在正确的一侧

Rust 保留原生文本、UTF-16 选区、IME marked text、编辑序号、光标跟随滚动和撤销历史；JS 的 controlled value 用编辑确认序号同步，不直接覆盖正在发生的输入法组合。

这是桌面 UI 的关键，不该为了让桥接显得“简单”而把瞬时编辑状态搬到 JS，制造来回传输和状态竞争。原生自定义组件的 `NativeView` 同样有明确的 mount/update 生命周期。

### 3. JS/Rust 边界是一套真实契约

同一份 schema 生成两侧绑定；结构检查、语义验证、Extension 身份、surface/epoch/revision、背压和终止规则都明确。外部 Bun、嵌入式 Bun、QuickJS 不需要各自暴露一套 GPUI 对象接口。

这增加了实现成本，也换来了可测试、可替换的运行时边界。**它不是应用沙箱，也不自动证明低延迟，但不是无意义的复杂度。**

### 4. 我们在做框架，而不只是演示

生成的 Rust 服务/组件、虚拟列表、路由、主题、开发失败恢复、真实 QuickJS 执行、应用配置驱动的测试以及 Gallery 分发流程，都已经超出计数器样例。

更重要的是，我们区分“状态测试通过”“原生窗口表现正确”和“真实呈现性能”。这是应该继续保持的工程优势，不意味着所有平台和组件都已完成生产验收。

上述结论的源码证据见 [本地基线](local-baseline.md#implemented-strengths)。

## 二、他们做得好的地方

### heyhuynhgiabuu/solid-gpui：验证意识和原生领域组件

固定版本：`05f84e9`。

- 有真实窗口测试、生命周期状态数量检查，以及该版本实际通过的 macOS/Windows/Linux GUI CI。不是只有 README 写“跨平台”。但 GUI jobs 不是强制失败门禁，不能当作完整平台验收。
- mutation 部分失败时会返回已成功数量，并把 client 标记为 poisoned，避免继续往已经分歧的树上写。这比静默失败好，但不如我们的原子发布。
- Markdown 以完整文档为原生组件输入，Rust 负责解析、高亮和内容缓存，不把每个语法节点都拆成跨语言操作。
- utility classes 有归一化、优先级和未知 token 诊断，值得学；它的 fontWeight、百分比尺寸存在 producer/mapper 不匹配，不应照搬。

**适合学验证手法和组件抽象，不适合换掉我们的协议与输入模型。** 它的输入选区／IME 几何不完整，controlled push 会重置编辑状态；列表是绘制虚拟化，但仍保留全部子节点所有权。[详细证据](solid-alternatives.md#3-heyhuynhgiabuusolid-gpui)。

### lxsmnsyc/solid-gpui：作者体验最值得参考

固定版本：`196aa6e`。

- 独立 `/compiler`，不必使用 Vite 才能获得同一份 JSX 编译能力。我们的 transform 已有实现，但没有对应公开入口。
- `paddingX/Y`、单位归一化、overflowX/Y、flexBasis、aspectRatio、hover/active/group 样式，比我们核心 style 更易表达常见界面。
- recorded canvas：JS 响应式地记录绘图命令，Rust paint 时重放。这个模型适合流程图、图表、编辑器，不需要在 GPUI 绘制线程调用 JS。
- native input 是独立 retained Entity，有选区绘制、点击定位和 IME 几何；相同 controlled value 不重置编辑状态。
- 简单 `render()` 入口和精确版本配对的平台 binary package 设计，降低了“必须写 Rust 才能试用”的门槛。不过公开 registry/Release 元数据未证明这些二进制今天可下载。

**学它的易用性，不学它的宽松数据一致性。** singleton session、无界队列、当前 handler 路由和直接改 live tree 都不适合替代我们的基础；另发现 native InputEvent object 被解包成 string、但示例期待 `event.value` 的静态契约错配。[详细证据](solid-alternatives.md#4-lxsmnsycsolid-gpui)。

### remorses/gpuix：分发和原生测试最值得学

固定版本：`4ecca30`。

- 预构建 native npm 包和不写 Rust 的 first-app 路径真实存在；公开 native／React／Solid 包为 0.10.0。它的 scaffold 会下载移动的 main 并选 latest，我们应学简单，不学不确定版本配对。
- JavaScript 可驱动真正的 native GPU test renderer，读取 painted bounds、截图、模拟输入；还有 live automation。Linux 没有同等 test-renderer 支持，不能概括成所有平台统一覆盖。
- style 用 `Arc` 共享、哈希去重并确认碰撞、无变化检查、无人引用后的回收。它不只“缓存一下”，连生命周期都设计了；我们值得测量是否能减少 StoredNode／journal 开销。
- hover／active／focus-visible 外观直接由 native style refinement 处理，不需要每次交互都写 JS signal 再传一次样式。
- renderer-wide 跨元素文本选区／搜索、live image 更新和签名 updater 都是具体能力。我们已有单节点 selection、TextView／Input 能力和完整 Image 所有权，不是“完全没有”。

**不能把 N-API 当成无成本捷径。** 它仍传 batched JSON；macOS 的 AppKit 由 JS `tick()` 驱动，长同步 JS 会延迟 pump。其他平台有 native 线程、tree lock 和无界 command queue。普通 native 子树仍从 root 构造，未证明比我们更快。

它解析样式后再改树，有一定原子性，但没有我们的完整 revision／结构验证／回滚模型。另外 public type 虽列 `canvas`，默认 native registry 没有 Canvas factory。[详细证据](gpuix-alternatives.md#3-remorsesgpuix-implemented-behavior-and-tradeoffs)。

### jhomra21/gpuix-solid：真实消费与真实应用验收

固定版本：`ad384ff`。

- Solid 1/2 分包明确，测试真实 client reactivity，不把不同 runtime 混叫一个 binding。
- exact tarball smoke、移出 workspace 后安装 starter、固定源码 blob 的 Mail／timeline／Kobalte port，关注的是真实使用者而不是仓库内部能运行。
- native tests 覆盖输入、focus、selection、节点身份／reorder、drop 和 style；缺少 addon test support 时会 skip，不能把“有测试文件”当成当前 CI 全通过。
- utility／manifest style 编译、语义标签和 SVG lowering 提高作者效率。
- Canvas/video edge 有实际绘制与资源代码，但不是 public native 0.9/0.10 默认能力。IOSurface pointer bytes 只是同进程指针契约，不能因为编码成 bytes 就拿来跨进程传输。

**学它的 port 溯源、独立安装和 update → paint → hit-test → callback 验收，不学整套 fake DOM。** 里面有 `select()` 变 focus、classList no-op 等近似，还存在旧 mutation fallback 和额外 JSON rewrite；Kobalte 样例可用不等于任意浏览器组件可用。[详细证据](gpuix-alternatives.md#4-jhomra21gpuix-solid-implemented-behavior-and-tradeoffs)。

## 三、我们有什么做错了

这里区分已经证实的错误、设计弱点和仍需测量的取舍。

### 1. 已复现错误：TestHost 的 Unicode 默认选区单位错误

`TestHost.dispatch({ type: "input" })` 用 UTF-8 byte length 作为默认光标位置，但协议／native input 使用 UTF-16。

本地 controlled-input probe 已复现：`新值` 返回 selection 6，正确为 2；`🙂` 返回 4，正确为 2。错误值还进入了 ack Patch。**这是 public testing helper 的缺陷，不是证明真实原生编辑器错误。** 应增加一个同时断言 CJK／astral 字符选区和 acknowledgement 的关键测试。

### 2. 明确错误：协议升级后，release rehearsal 仍检查 v5

当前 Rust `PROTOCOL_VERSION` 是 6，host 的启动信息和 `--version` 都从这个常量生成；但 `host-release.sh`、process/embedded candidate smoke 仍精确断言 `protocol=v5`。

因此，正确构建的当前 v6 host 也过不了这些旧断言。这个结论由源码直接证实，但本次没有执行完整 release rehearsal。它不意味着另一条 QuickJS Gallery 发布流程或已发布压缩包损坏。

应从规范化协议／发布元数据读取预期值，而不是在多个脚本手写版本号，然后重新跑受影响的交付验收。

### 3. 明确错误：入口文档与现状不一致

- 根 README 仍说 GitHub Pages 等待首次部署，但调研时公共网站已经可以正常渲染首页。

我们要求文档同步，但这次检查说明执行并不彻底。另外 core 包 README 的 macOS-only 表述需要限定范围：直接嵌入式 library build 确实仍限 macOS，实验性 static packager 则已有 Windows 证据。这里应消除歧义，不能把它误报为“Windows 已受生产支持”或“macOS-only 完全错误”。

### 4. 产品设计弱点：让使用者承担 SDK 内部构建约束

用户安装 npm 包以后，仍要找到匹配版本的 Rust 源码，并把 workspace-root patches/profiles 带入自己的应用。`doctor` 能诊断，不能消除这份负担。

我们已有独立 packed-consumer 测试，不能说完全没测外部用户；但其中的小 Rust exporter 只打印测试绑定，不链接 GPUI、不打开窗口。仓库内的 desktop example 也依赖我们自身 workspace。**SDK 可以被安装，不等于普通用户能轻松构建并交付第一个应用。**

### 5. 技术设计短板：JS 细粒度，没有完整传递到原生渲染边界

```text
Solid 的响应式范围 ≠ Patch 范围 ≠ 原生构造/布局/绘制范围
```

Rust 的局部状态 reconciliation 已经消费 Patch change set，但提交之后仍通知 `SolidRoot`；普通 View 子树由 root 递归构造 GPUI 元素。Extension 和 VirtualList 有各自路径，不能把它概括成“所有东西都全量重建”。

准确的问题是：**一个很小的 JS 更新仍可能引发较宽的原生元素构造和布局工作。** 这不是跨项目性能排名，也不是新发现的崩溃 bug。

我们过去试过 cached pane，计时明显变好，但计数器显示停在旧值，已经正确否决。因此需要设计真正的 region owner 和失效规则，不能只加 `.cached()`。

### 6. 值得重构：契约身份与实现源码身份混在一起

catalog digest 不只包含公开 props/events/commands，还包含被 `include_str!` 引入的实现源码。仅注释或换行变化也可能改变 digest，导致绑定不匹配；CI 因此专门处理 Windows CRLF。

严格锁定实现版本有价值，问题不是“验证太严格”，而是一个“契约摘要”同时承担“精确构建身份”的职责。

建议拆成两种明确身份：规范化公开契约与显式语义版本；精确 SDK/构建身份。两者都可以继续严格验证，不需要兼容旧版本，也不能悄悄接受错配。

### 7. 战略风险：支持面比验证面长得更快

三个原生运行时加实验性 WASM、嵌入式 Bun 打包和自维护依赖，会形成持续的维护矩阵。实际经过仓库分发验证的是 QuickJS Gallery；嵌入式 Bun 的分发指南仍明确不宣称任何 target 已受支持或验证。

多运行时不必删除，但应该先给普通用户一个最短、已验收的默认路径，其他路线保持清楚的实验等级。不是先添加更多选择，再让用户自行识别哪些真正成熟。

### 8. 作者体验不足：已有能力没有形成小而独立的接口

我们的 canonical JSX transform 不公开，style 的常见简写和核心节点状态样式也偏窄。这里不是“缺少 hover／图形／响应式”，而是使用者表达常见 UI 需要更繁琐的写法。可以学习 lxsmnsyc 的模块拆分和归一化，不必照搬它的 Solid 2 runtime 或宽松解析。

### 9. public testing 缺一段原生验收接口

TestHost 能验证真实协议和 semantic event，但明确不做 layout／paint／hit testing；我们也有 Rust/native tests，所以不是“没有原生测试”。差距在于 JS 应用作者缺少公开的 painted bounds／native interaction 验收入口。GPUIX 这条路值得补。

证据与反面论据见 [本地设计边界](local-baseline.md#concrete-weaknesses-and-boundaries)。

## 四、可以学到什么，怎么应用

### 初步优先级

| 优先级 | 建议 |
| --- | --- |
| **P0** | TestHost Unicode offset、旧 v5 release 断言、公开状态漂移 |
| **P0** | exact-version standalone starter；stock 功能可选预构建 host |
| **P1** | opt-in native acceptance 接口，加一个固定源码真实多 pane／长历史应用 |
| **P1** | 原生 render regions／input owners 的测量与设计；独立 compiler 导出 |
| **P1，先实验** | native style sharing/reclamation；contract/build 身份分离 |
| **P2** | strict style sugar、跨元素 selection、按需 paint/media；打包成熟后考虑 updater |

先修复两个明确缺陷和公开状态漂移，不应等新功能完成后再处理。

1. **第一个应用体验：** 提供独立 scaffold，锁定 npm/native revision，正确生成 workspace 约束；不需要自定义 Rust 的应用考虑使用预构建 default host。
2. **原生渲染区域：** 不改变 Solid 作者体验，建立稳定 region owner，验证局部更新是否真能减少不相关子树构造。必须同时覆盖输入、滚动、主题、尺寸、继承样式、异步资源和卸载。
3. **身份职责分离：** 区分契约、语义版本和精确构建锁，不引入宽松兼容，不以“类型一样”代替行为契约。
4. **普通应用分发：** 优先闭环独立应用的 build/preview/package，而不只让仓库 Gallery 打包成功。可以先扩展已有 Rust-led + QuickJS 交付模型，但必须在真实 QuickJS VM 验证；依赖 Bun/Node 服务的应用不能自动转换引擎，应保留自己的运行时与实验性交付说明。
5. **能力状态统一：** 修正文档漂移，入口只链接权威能力矩阵，不维护多份会过期的平台断言。
6. **编译器独立导出：** 暴露我们已有的 canonical transform，让 Vite、Bun plugin 或其他工具复用同一份 universal 编译、TypeScript 擦除和 source map。
7. **严格的 style sugar：** 从 X/Y spacing、明确的状态样式开始，定义合并优先级；不支持的 utility 必须报错，不制造浏览器 CSS 兼容承诺。
8. **按需 recorded paint：** 有真实流程图／自定义绘图需求再做，通过 NativeView 保留有界命令，验证数量、坐标、DPI、主题、裁剪和卸载，不把 JS callback 搬到 paint。
9. **降低上游维护负担：** 保留必要补丁，但维护准确清单、优先上游化通用修复。我们的 `vendor/gpui/PATCHES.md` 仍写来源 0.3.5，而实际 manifest/lockfile 是 0.3.7。其他项目使用 stock GPUI 更简单，不代表我们为输入、列表和缓存补的行为都不必要。
10. **native acceptance：** 通过 opt-in locators、painted text/bounds、native click/type/drag/wheel 和 screenshot 补齐应用验收；不能因为 stdin 被 pipe 就自动打开强力命令服务器。
11. **应用级证明：** 固定源码与 port 映射，验收真实滚动位移、resize、copy／selection、清理和后续点击。不要用一批相似截图代替行为一致性。
12. **style sharing 实验：** 测量我们实际 compact Style、journal clone、重复更新和 drag churn，再决定是否引入 reclaimable interning；暂不新增 wire style-reference 协议。

region 性能验收要包含一个大静态 sibling pane，也要包含“文本变宽导致相邻内容必须重新布局”的反例；分别记录提交成本、CPU draw 和 input-to-present。不能通过跳过必要更新来换漂亮的数字。

落地时只保留关键验收场景：独立目录安装后的真实窗口；region 更新后的实际可见内容和回调归属；只改注释与真正改契约的身份差异；移出仓库后仍可启动的分发包。应扩展已有测试，不累积一批只能证明模块可 import 的重复 smoke tests。

## 验证与限制

学习设计不等于直接复制代码：heyhuynhgiabuu/solid-gpui 和 remorses/gpuix 的根许可证为 Apache-2.0；另外两个为 MIT。未来若引入源码，需要按固定版本检查文件与 NOTICE、保留上游条款并更新第三方清单，不能统一改标成我们自己的 MIT。本次没有复制外部实现。

- 本地 renderer / control-flow / pressure 相关测试：**47 通过，0 失败**。
- 已有 TestHost suite：**7 通过，0 失败**，但现有断言没有覆盖这次复现的 Unicode 默认选区缺陷。
- 网站 headless / 内容测试：**7 通过，0 失败**。
- 第一方 npm registry 当前返回 `@solid-gpui/core@0.5.2`；公共网站首页已实际截图检查。
- GitHub 已发布 `v0.5.2`，包含 macOS ARM64、Windows x86-64、Linux x86-64 Gallery 压缩包和校验文件；本次没有下载或启动这些产物，发布存在不等于物理桌面验收完成。
- 协议／release 脚本静态一致性检查：**失败，定位到三个脚本中的五处旧 v5 断言**；未执行完整 release 构建或验收。
- 本地 TestHost probe 已复现两个 Unicode 默认 offset 错误；初次 probe 第二例误用可复用的隐式 surface 1，随后改用不同显式 surface ID 完成两例。没有修改实现或新增 regression test。
- 引用／文档完整性检查通过：345 个固定版本源码行范围、219 个 Git blobs、31 个相对链接，以及标题／换行／空白检查；不代替执行所有实现。
- 未安装或运行其他项目依赖，未做四个项目的原生窗口、真实输入、内存、包体积或性能对跑；不能据此断言谁更快。
- 仅新增 `.scratch/` 调研记录，没有修改实现、API 或网站内容。已经检查文档与网站的内容归属；研究记录不进入网站目录，无需生成 API 或同步导航。发现的过期公开说明记录为后续事项。
