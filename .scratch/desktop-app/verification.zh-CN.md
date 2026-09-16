# 桌面应用历史验证记录

以下记录产生于示例从 `native-migration` 更名为 `desktop-app` 之前。
名称和测量结果描述对应日期的运行，不代表当前验收，也不是迁移流程。
可复用的 API 指南保留在 `docs/`；当前[桌面应用示例](../../examples/desktop-app/README.zh-CN.md)
的运行方法以其 README 为准。

## 集成验证记录（2026-09-16）

- macOS：254 项 Rust 库测试通过（1 项忽略），86 项核心包测试、12 项 Vite/导出/构建/API 测试、4 项跨语言协议测试和 8 项网站测试通过。包类型检查、迁移示例生产构建、网站 WASM/TypeScript/生产构建通过。消费项目保持 `verbatimModuleSyntax`，直接引用当前 SDK 源码通过 TypeScript 检查，未修改该项目。
- Windows：在 Windows 11 ARM64 Parallels VM 中运行 Zig 交叉构建的 x86-64 GNU debug 宿主，以及便携版 ARM64 Bun 1.4.2。QuickJS 网站与 Bun 迁移宿主都以 1 MiB 初始栈链接；正常入口报告 16 MiB 应用线程栈并成功渲染。同一网站可执行文件仅设置 `SOLID_GPUI_APP_STACK_BYTES=1048576` 后，报告原生栈溢出并以 `0xC000041D` 退出；页面内容和动画策略保持不变。
- Windows 实际像素与交互确认中文可读、原生输入、复选框、普通动画按钮和 Rust 命令往返可用。迁移 Select 在加载目录、打开菜单、确认当前行时均不触发变更；选择不同项只触发一次。卸载目录后保留受控 key 与计数。滚轮可到达第 14 行及结束标记，页头/页脚保持固定；macOS 的有界布局也可滚至末行。
- 强制终止 Windows 迁移宿主后，Bun 子进程于 0.68 秒内退出，无 renderer 孤儿；macOS 强杀冒烟验证也未留下 renderer。独立 stdio 回归覆盖 detached 服务保留与自定义流所有权。
- Windows 字体探针成功解析并栅格化 `.SystemUIFont`，采样字形与该 zh-CN 来宾中的 Microsoft YaHei UI 一致。Windows 核心测试记录为 196 项通过、7 项失败、1 项忽略：5 项依赖 POSIX `sh`/`sleep`，2 项需要尚未部署的 Windows JSX 工具链。另两项绝对路径 fixture 已修正并单独通过。

这不是 MSVC/原生 ARM64 宿主认证，也不代表测得了每个消费应用所需的栈。Linux 和完整 Windows 发布矩阵未验证。最初“仅设置 `hotKey` 就使新应用空白”的报告未独立定位；已证明的修复针对受管理状态交接的重复捕获，以及后开 Surface 的首 Snapshot 准备，并通过回归和受管理重载/回滚检查。

## 集成验证记录（2026-09-08）

英文源文档记录：合并工作区通过 220 项 Rust 库测试、4 项跨语言协议测试、37 项迁移及渲染器测试、工作区 TypeScript 检查和 5 项网站测试。迁移宿主及生成绑定可编译；WASM release 宿主与网站生产包使用 `wasm-bindgen 0.2.121` 构建成功。网站检查覆盖组件示例、文档高亮和导航保留。本次集成没有重新手动执行下列原生窗口交互。

## 原生验证记录（2026-09-07）

环境为 Apple Silicon、macOS 26.6.2、Bun 1.4.2、Rust 1.98.1。英文源文档记录了以下结果：

- `bun run check`、协议及原生契约生成检查、工作区构建、TypeScript 和启用 QuickJS 的 Clippy 通过。
- 核心、路由与 HMR 测试 72 项通过，启用 `gpui-component` 的 Rust 库测试 205 项通过，双向协议 golden 检查通过。
- 示例类型检查与 Vite 生产构建通过。
- 临时 macOS app bundle 的初始内容尺寸为 1100 × 720，最小尺寸为 960 × 640，两个尺寸都保留双列摘要及底部操作边框。
- 48 px 标题栏只显示一次，红绿灯位置正确；标题栏输入、双击输入、导航及原生按钮不触发窗口缩放。空白区双击缩放与恢复、全屏与恢复工作正常。执行了空白区域拖拽，但没有测量屏幕绝对位移。
- 检查了输入、按钮、标签、文字、背景、自定义 SVG、封面图、渐变、分段接缝及边缘边框；debug 应用没有 FPS 覆盖层。
- TSX 标题更新无需输入唤醒窗口；语法错误及同步渲染异常保留上一版 UI，修复后在同一宿主进程恢复。Rust 计数器从 1 增至 2。
- 生产 JavaScript 包不依赖 Vite，在同一原生宿主运行，显示本地图像与内嵌图标并调用 Rust 服务。关闭开发和生产窗口后，宿主退出并关闭 Bun 子进程。

这些检查覆盖框架 API，应用仍需执行自身的视觉与交互验收。Select、Dialog、Menu 和 TextView 主题由共享原生实现及自动检查覆盖，但没有对全部状态矩阵进行视觉验证。Linux/Windows 窗口行为与 QuickJS HMR 未在该交付中手动验证。
