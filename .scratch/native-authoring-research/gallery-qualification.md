# Embedded Gallery 原生验收

2026-09-06，macOS，当前 dev 构建，`gpui-component` + `embedded-bun`。用本次 Vite production bundle 和本次重建的 Bun 动态库运行，临时 .app 仅用于原生自动化识别；不是发布包。

- Host SHA-256：`94a16cd217ad97d6233d021f0e05132631ab2bbf8286bd468945d7de7c4ef582`
- Gallery bundle SHA-256：`4171c4f759fb6886d04a7b937d1b2f3656e606e6c2d1745efb957ce8f20eb54c`
- Bun library SHA-256：`9e9831857148c6b013ef46a0534ba00ed49653dfbbd7b3e4cd1666d0dae303c0`
- 启动入口：`examples/gallery-vite/dist/main.js`，`--runtime embedded`，实际 PID 84149。
- 初始逻辑窗口 800×633；执行原生 zoom 后继续操作。截图为 CUA 返回的 1304×768 图像，不用截图像素反推逻辑窗口或设备缩放。

实际操作：

1. 原生 Build workspace 按钮使进度从 0 到 20，Rust BuildBadge 显示 `Rust component · 1 builds`。
2. Analyze in Rust 返回 `Untitled workspace`、`untitled-workspace` 和 20% 进度。
3. 将原生输入设为空后再次调用，显示 Rust 的 `Workspace name must not be blank` 错误。
4. 输入带空格的 `  Rust   Studio  ` 后再次调用，Rust 返回规范化名称 `Rust Studio` 和 slug `rust-studio`；错误已消失。
5. 点击原生关窗按钮；后台清理后日志记录 `runtime terminated status=Shutdown`，应用进程退出，启动器返回 0。

[成功截图](gallery-embedded.jpg)、[启动与正常退出日志](gallery-embedded.log)、[协议记录](gallery-embedded.tap)。这是交互、业务往返和生命周期证据，不是滚动性能或显示帧率验收。

验收中还定位了宏把注入的 Clippy 属性计入契约摘要的问题：改为先捕获开发者声明，再添加编译辅助属性。经 canonical native-codegen 重建，SDK 与 Gallery 实际链接 Host 的组件摘要一致。
