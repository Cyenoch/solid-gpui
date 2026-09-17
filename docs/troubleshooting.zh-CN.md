# 故障排查

## 任务 CLI 无法解析包

安装仓库锁定的工作区依赖后，在仓库根目录执行：

```sh
bun install --frozen-lockfile
bun run task --help
```

不要在 `packages/solid-gpui` 中安装；仓库只有根目录的 `bun.lock`。排查依赖前，先确认 `bun --version` 与 `.bun-version` 一致。

## 信号变化但没有发出 Patch

从 `@solid-gpui/core/runtime` 导入响应式原语，不要使用面向服务器的 Solid 入口。在传给 `root.render` 的创建函数中构建应用：

```ts
root.render(() => createComponent(App, {}));
```

进入根节点之前创建的宿主节点没有 surface owner，会被拒绝。

## 出现 `host operation is not associated with a root`

组件或宿主节点在根创建函数或保留的 Solid owner 之外创建。将创建移到 `root.render(() => ...)` 内，不要在根之间共享 Host Node。

## 原始文本校验失败

字符串和数字子内容只能直接位于 `Text` 下。将标签包在 `Text` 中，不要直接放在 `View` 或 `Pressable` 下。

## 协议版本不匹配

TypeScript 包与原生宿主必须使用相同协议版本。从同一 checkout 重建两端，不添加兼容解码。

## Surface 已关闭

关闭或卸载的 surface ID 永久退役。创建新的 surface 和根，不要换一个 generation 复用旧 ID。

## 原生契约不匹配

Extension 的 provider、catalog digest、entry ID 和版本必须与宿主完全匹配。
目录不匹配时会显示模块名、renderer 与 host 的摘要，以及宿主为该 entry 注册的
名称。不同目录的 entry ID 可能重新分配，因此显示的宿主名称不一定是 renderer
尝试使用的组件。缺少适配器则需要检查宿主是否注册了该模块、是否支持相应组件。

先根据 Vite 打印的 host 路径确认实际可执行文件，启用所需 Cargo feature，并在
该宿主注册模块。配置 `native` 后，保存修正后的 Rust 源码或 Cargo manifest 会
重建宿主及 bindings；`#native` 和 `@solid-gpui/core/components` 都使用这份输出。
显式 `host` 配置同样以 `--export-native` 导出实际目录，不会替换为 SDK 中保存的组件目录。
需要自行重建该可执行文件，再保存应用源码以重新导出 bindings 并启动会话。
`native` 是由 Cargo 管理构建的对应方案，两者都不要求应用声明自定义 native 模块。

提交被拒绝后当前会话会结束，Vite 继续等待下一次修改，不会反复重启同一个失败
程序，也不会继续应用依赖错误 revision 的 Patch。见[开发会话管理](hot-reload.zh-CN.md#开发会话管理)。

## Windows 宿主栈溢出

GPUI 布局和绘制沿原生元素树递归；Windows 可执行文件默认线程栈可能不足以容纳深层 debug 布局和动画控件。宿主入口在调用线程栈不足时预留 16 MiB 应用线程栈。带 profile 的入口接收工厂，在目标线程构造 profile；应用不需要线程包装。macOS 仍在真正主线程运行。

用 `SOLID_GPUI_LOG=info` 查看实际线程栈预留，通过 `SOLID_GPUI_APP_STACK_BYTES` 对同一页面做不同预算的有界对比；无效预算会使启动失败。显式 `/STACK` 链接选项也是宿主可以选择的预留方式。对比时保持界面内容、动画策略和 Cargo profile 一致，检查启动、切换页面和重复交互。增大预算不能修复无界递归；关闭动画或把全部 Button 换成 Pressable 只是隐藏触发条件，不是可靠的启动契约。

## Windows debug 启动时无法创建 DirectWriteTextSystem

`Error creating DirectWriteTextSystem` 搭配 `os error 3` 可能表示缺少 debug 着色器，而非字体。使用当前渲染器重新构建：它嵌入 HLSL 模块和 `alpha_correction.hlsl`，并从内存编译。切换工作目录或复制 JavaScript 不能修复旧 EXE。Release 构建需要在构建时使用 SDK 着色器编译器。

应诊断底层初始化错误；CPU 特性警告或断点退出码本身不能确定原因。

## 静态内嵌 Bun 无法加载 debug 内置模块

若 debug EXE 到构建机器的 `build/.../js` 目录查找 `node:worker_threads`，使用当前静态打包器重新构建。其补丁同时关闭磁盘热加载，并通过 `--embed-modules` 生成内嵌源码；仅关闭 `BUN_DYNAMIC_JS_LOAD_PATH` 仍会留下无效的模块范围。不要把内置 JS 复制到 EXE 旁边或关闭断言。

Windows GUI 子系统设置不能屏蔽原生断言弹窗。候选失败后先停止，检查错误或调试器堆栈，再决定是否重新启动。

## 静态内嵌 Bun 在 Windows 上丢失环境或 Worker 路径

- **`os.tmpdir()` 出现 `undefined\temp`：** 设置父进程的 `TEMP`/`TMP`，并用当前内嵌覆盖源码重新构建。VM 必须在执行应用前通过 `load_process()` 导入继承环境；关闭 `.env` 加载不能替代它，也不要写死临时目录。
- **已声明的 Worker 报 `ENOENT`：** 通过 `--workers` 声明入口，并相对 `import.meta.dirname` 定位，而非相对工作目录。使用当前补丁，向加载器传递图内规范化键，而非该键的原生分隔符写法。

## 内嵌 Bun 构建失败

Windows 使用[静态应用打包器](distribution.zh-CN.md#内嵌-bun-静态应用)；直接构建 `embedded-bun` Cargo feature 仍只支持 macOS。先检查[打包前置条件](distribution.zh-CN.md#静态打包前置条件)，并运行 `bun install --frozen-lockfile`；序列化器、带补丁的原生源码和预编译 WebKit 必须匹配固定版本与目标。

| 症状 | 处理方式 |
| --- | --- |
| Windows release 找不到 `fxc.exe` | 安装 Windows SDK 编译器或设置 `GPUI_FXC_PATH`。Release 着色器需要 DXBC，不能替换成 DXIL 或 debug 着色器。 |
| ARM64 debug 找不到 `libcmtd.lib` 或 `libcpmtd.lib` | 在 SDK splat 中加入微软匹配的 `Microsoft.VC.14.44.17.14.CRT.ARM64.Desktop.debug.base.vsix`，核对官方包校验和；不要混用 debug/release CRT 库。 |
| Windows ARM64 报 `wasi.initialize is not a function` | 固定的 Solid 编译器没有原生 ARM64 binding，其 WASM 路径需要固定版本驱动 Bun 尚未提供的 WASI API。先在编译器支持的宿主生成 Vite 输入，再打包所得 JS；纯 TypeScript 命令使用当前延迟加载 JSX 编译器的 preload。 |
| 内置模块生成报 `ENAMETOOLONG` | 用当前内嵌补丁重新构建；补丁从明确的工作目录传入相对模块路径。 |
| 源码解压报 Win32 错误 `1314` | 构建账户无法创建所需符号链接。由构建宿主管理员准备该能力后再重试；运行应用的用户不需要此能力。 |

## Windows ARM64 release 以 `0xC0000409` 退出

使用匹配的 PDB 检查堆栈；单凭退出码不能区分断言、栈故障或依赖缺失。若是 JSC 时钟比较断言，通过 `--winsysroot` 使用原始 MSVC 14.44 头文件和库，以匹配预编译 WebKit 的 ABI；MSVC 14.51 改变了相关 `std::partial_ordering` 返回约定。不要关闭断言或修改时钟行为来隐藏不匹配；其他 fail-fast 堆栈仍应依据各自证据诊断。

## QuickJS 无法解析服务或传输

QuickJS 宿主使用 `@solid-gpui/core/embedded` 的 `EmbeddedTransport`。`StdioTransport` 属于 Bun 进程或内嵌 stdio 环境，需要 `process.stdin` 和 `process.stdout`。

使用 `bun --bun vite build`（配置 `runtime: "quickjs"`） 将依赖打成一个 ESM 模块。Node/Bun 导入会被拒绝，环境不提供 `process`、`Bun`、文件系统或 `fetch` 等网络 API。将服务移到 Rust Native Module，调用生成客户端。打包器的 browser target 只选择可移植依赖，不会为 QuickJS 创建浏览器环境。

## Vite 已 ready，但 QuickJS 窗口空白

Vite 的 `ready` 和 bundle 大小只证明编译成功，不证明应用已完成渲染。只显示背景的原生窗口，可能仍停留在等待后续更新的初始加载树。

先捕获实际运行时异常，再调整布局。在 `native.windowChrome().then(value => setChrome(value)).catch(...)` 这样的启动链中，catch 不仅接收原生命令失败，也会捕获 `setChrome` 触发的同步渲染异常。笼统的“窗口配置失败”提示可能掩盖主组件创建时的 `Maximum call stack size exceeded`。在 stderr 诊断中保留异常消息和栈，避免记录可能包含应用数据的原生返回值。

若 Cargo 提示 `profile package spec ... did not match any packages`，检查消费工作区实际解析的依赖：

```sh
cargo tree --locked -i rquickjs-sys
```

当前 QuickJS feature 使用 `rquickjs-sys`。旧的 `quickjs-jit-sys` 或 `quickjs-jit-core` profile 无法优化它。删除失效条目，按[应用构建配置](hot-reload.zh-CN.md#应用构建配置)设置工作区根 profile，然后重新编译并重启原生宿主。保存 TSX 不能改变原生编译参数；增大栈限制前先执行下文的栈诊断。

对于先渲染加载树的应用，可临时启用协议 tap，区分“原生命令返回成功”和“界面得到更新”：

```sh
SOLID_GPUI_TAP=target/solid-gpui-startup.jsonl ./target/debug/my-app --runtime quickjs dist/app.js
```

替换为应用实际的可执行文件和 bundle 路径。采集时用单个宿主直接加载出现故障的开发 bundle，不经过重载 supervisor，避免不同运行时代次共用同一个 tap 输出文件。

检查 `snapshot`、带 `success: true` 的命令结果事件，以及后续 `patch` 记录。成功返回却没有 Patch 可以将故障定位到原生命令之后，但不能单独证明原因。不要把它当成所有应用的启动断言：完整界面在首个 Snapshot 中渲染时，本来就不必产生 Patch。仍需检查实际页面内容和一次交互。tap 只记录元数据；完成有界复现后关闭。

一次消费项目回归保持开发 bundle 和 2 MiB JS 栈预算不变：失效的 profile 导致空白窗口、被捕获的栈溢出和缺失的 UI Patch；优化 `rquickjs-sys` 后主界面恢复。对比时固定 bundle、状态和栈预算。

## QuickJS 在深层路由上栈溢出

先检查应用工作区的 Cargo profile。在应用**工作区根目录**的 `Cargo.toml` 中设置 `[profile.dev.package.rquickjs-sys]` 和 `opt-level = 3`，然后重新编译并重启宿主。Cargo 只读取工作区根清单中的 profile，不继承依赖仓库的配置。参见 [Cargo profile 文档](https://doc.rust-lang.org/cargo/reference/profiles.html)。

解释器优化会影响原生栈占用和执行速度。在相同的 2 MiB JS 栈预算下，未优化的 debug 构建可能在初始化深层组件树时溢出，而优化后的构建能够完成初始化。本仓库中的测试通过，并不能证明应用使用了相同的构建配置。

分别对比直接启动到目标路由、从简单路由导航过去，以及重载后恢复。导航可能复用已有父布局，启动和重载则会重新创建它。比较构建 profile 时保持 bundle、捕获状态、布局和栈预算不变。如果优化后的构建仍失败，缩小路由复现，检查组件递归创建和响应式更新，再考虑调整栈预算。

## QuickJS 拒绝捕获的状态

检查错误中的字段路径，例如 `$.state[0].session.userId`；`state[0]` 是应用 `captureState` 的返回值。返回专用 JSON 状态对象，明确使用 `null` 或省略可选字段。嵌套 `undefined`、稀疏数组、访问器和运行时对象不能跨 VM 传递。`captureState` 本身返回 `undefined` 表示没有捕获值。

捕获在创建候选之前的旧 VM 中运行。修正其运行中的状态，或修复始终返回非法数据的捕获函数后重启宿主。JSON stringify/parse 往返可能静默丢失数据；应按[捕获状态契约](hot-reload.zh-CN.md#捕获状态)显式定义交接内容。

## QuickJS 报告 `applied` 后页面仍然失败

`applied` 表示宿主已激活验证通过的候选。路由加载、由加载触发的组件初始化和异步原生副作用仍可能在随后失败。这些错误不在回滚边界内；VM 停止后，最后发布的原生树仍可能可见。

检查后续运行时诊断，并验证页面特有的内容和一次交互。loading 页面或一直存在的导航标签不能证明恢复后的页面可用。参见[应用重载验证](hot-reload.zh-CN.md#验证应用重载)。

## 渲染器失败后进程宿主退出

检查 stderr 中的渲染错误及宿主崩溃报告路径。协议解码或验证失败是致命错误，继续执行会丢失 revision 一致性。应用渲染错误会抛给应用，Solid GPUI 不会自行生成备用界面。

## 强杀宿主后 Bun 渲染器仍在运行

进程渲染器应使用默认 `StdioTransport`。读取 `process.stdin` 的连接拥有渲染器生命周期：宿主管道关闭后先通知终止监听器，再退出，即使应用定时器仍活跃。正常 EOF 以状态 0 退出，读写错误输出诊断并以状态 1 退出。不轮询父 PID，不发送进程组终止信号，也不影响应用拥有的 detached 服务。

传入自定义流的连接默认由嵌入方管理，除非显式设置 `exitOnHostClose: true`。`dispose()` 不退出进程；只有进程确实需要比宿主活得更久时，才设置 `exitOnHostClose: false`。普通宿主子渲染器不应关闭此策略。

## 文件、剪贴板或对话框命令被拒绝

命令有大小限制，在跨协议前验证参数。使用非空绝对文件路径，遵守帧和资源限制；平台不支持的图片剪贴板操作应作为显式错误处理。

## TypeScript 使用了错误的 JSX 类型

使用以下配置：

```json
{
  "compilerOptions": {
    "jsx": "preserve",
    "jsxImportSource": "@solid-gpui/core"
  }
}
```

通过 Vite 插件或 Vite 插件 使用共享 Solid/Oxc 通用转换。普通 React 风格 JSX 转换不能生成该渲染器的响应式宿主操作。

遇到 `For`/`Show`/`Index`/`Switch`/`Match` 返回值或子内容类型不兼容时，从 `@solid-gpui/core/runtime` 导入，而不是 `solid-js`。上游声明使用 DOM 元素；runtime 入口使用同一实现并提供原生类型。参见[原生控制流](native-composition.zh-CN.md#solid-异步控制流)。

## 页面空白、被裁剪或滚不到最后一行

先检查 stderr：提交被拒绝不是布局失败。原生数据必须符合生成契约，裸标签应包在 `Text` 内。

布局问题需要检查完整父链，包括路由外壳。`flexGrow` 只参与父级布局，不会给节点本身启用 flex。分配剩余**高度**的容器应显式采用列布局。在有边界的列下，用 `height: 0, flexGrow: 1, minHeight: 0` 限定滚动视口，用 `flexShrink: 0` 保留内容自然高度。核心 `overflow: "scroll"` 支持滚轮，但不会创建可见滚动条；需要滚动条时使用 `Scrollable`。参见可运行的[有边界的页面滚动示例](scroll-performance.zh-CN.md#有边界的页面滚动)。

## Iconify 名称被拒绝

宿主没有打包整个 Iconify 图标库。从 `@solid-gpui/core` 导入 `ICON_NAMES` 查看内置图标。需要其他图标时，在宿主内嵌 SVG 并使用生成的 `applicationIcons` 名称；不要把任意字符串强制转换为 `IconName`。参见[添加应用图标](iconify.zh-CN.md#添加应用图标)。

## 滚动没有边界或很慢

沿路由包装层检查整条 flex 父链，再检查填满窗口的工作区是否从不必要的内容派生 flex basis 开始。真实 Gallery 回归、校准后的 CPU 预算和原生验收要求见[原生滚动性能](scroll-performance.md)。

## 升级依赖后工具失效

TypeScript 7 通过 `typescript/unstable/async` 暴露编译器服务，根 `typescript` 不再提供 `createProgram`。在 Bun 中使用异步 API；同步客户端私有 Node 管道句柄不可用。等待 `api.close()`，让快照释放完成后再关闭连接。升级后重新生成 API fixture，运行语义重导出测试和 `package-typecheck`。

保持官方 Solid 编译器固定版本：候选版本与 Solid 1 运行时独立，共享转换禁用 Solid 2 内置自动导入。升级必须保留 Vite 开发与生产构建两条路径的响应式更新、owner 清理、导入副作用和 source map。

## 关联原生命令错误

从 `@solid-gpui/core` 或 `/native` 入口导入 `NativeCommandError`。原生拒绝及结果格式错误的 `error.identity` 包含 Surface、epoch、请求、目标节点、命令类型及适用时的原生函数 ID。将这些字段与错误消息一起记录，关联协议 trace。标识不保留参数字节或返回载荷。应用错误字符串可能包含应用数据，应谨慎选择内容。信号取消保留原始 abort reason；传输关闭仍使用独立的 `TransportTerminatedError` 类型。
