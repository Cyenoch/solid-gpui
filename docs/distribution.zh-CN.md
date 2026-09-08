# 分发原生应用

运行时策略将内嵌 Bun 用于依赖 Bun 服务的生产应用，QuickJS 用于主要能力由 Rust 实现的应用，外部 Bun 用于快速开发。本指南当前实现 QuickJS website 打包，不提供等价的内嵌 Bun 发布流水线。差距与定位见[运行时策略](runtime-strategy.md)。

website 是参考应用包：Solid UI 编译为一个 ESM 模块，嵌入 Rust 可执行文件并由 QuickJS 执行，生产包不带内联 source map。用户无需 Bun、Node、仓库 checkout 或旁置 JavaScript bundle。Rust 拥有原生服务和渲染，同一界面组合也可在开发时通过 Bun 执行。

## 构建环境

安装 `.bun-version` 固定的 Bun 和 `rust-toolchain.toml` 指定的 rustup 工具链。从仓库根目录构建，打包任务在构建前按提交的工作区锁安装依赖。在准备发布的操作系统与架构上构建。

- **macOS：** 安装 Xcode 并选择开发者目录，工具链需提供 macOS SDK、C/C++ 编译器及 Metal 工具。release 前验证 `xcrun --sdk macosx --find clang`、`xcrun --sdk macosx --find metal`、`xcrun --sdk macosx --find metallib`。打包还使用系统 ditto、otool、plutil、codesign。
- **Windows：** 使用 MSVC Rust 工具链，安装含 Windows SDK 的 Visual Studio C++ Build Tools。release 使用 fxc.exe 编译 HLSL，GPUI 从 PATH 或 SDK 查找；自定义 SDK 布局可设置绝对 `GPUI_FXC_PATH`。打包使用 PowerShell Compress-Archive 和 Expand-Archive。
- **Linux：** 候选流水线使用 Ubuntu 24.04。安装 C/C++ 工具链及以下包，其他发行版安装对应开发包；打包还需 tar 与 ldd。

```sh
sudo apt-get update
sudo apt-get install --no-install-recommends \
  build-essential libfontconfig1-dev libfreetype6-dev libxcb1-dev \
  libxkbcommon-dev libxkbcommon-x11-dev pkg-config
```

原生窗口需要具有可用图形栈的桌面会话。内嵌 bundle 无显示检查不能验证图形栈。

## 构建与验证

网站应用包统一使用[项目确定的图标](../assets/branding/README.md)。macOS 包含 `solid-gpui.icns` 并通过 `CFBundleIconFile` 声明；Windows 通过 Windows SDK 资源编译器将 ICO 嵌入可执行文件；Linux 归档包含 hicolor PNG 图标及对应的 desktop-entry `Icon` 字段。共享导航的 PNG 嵌入 JavaScript 包，脱离仓库后也可显示。仅在原图变化时重新生成品牌资源，正常打包直接使用已提交文件。

在每个目标操作系统执行：

```sh
bun run task website-package
```

命令构建工作区包和原生绑定，针对当前 Rust host triple 生成 release 可执行文件，将归档和 SHA-256 写入 `dist/website/`。可选参数改变输出位置：`bun run task website-package ./dist/candidate`。随后解压到临时目录，验证每个文件，在 checkout 外、PATH 不含 Bun 的环境运行版本与内嵌 bundle 检查。

bundle 检查在真实 QuickJS VM 计算交付的 JavaScript，在统一 15 秒截止内消费有序 Snapshot/Patch，验证树和每个原生组件标识，要求 SDK 与 website Native Module 均有实际内容，然后关闭 runtime。只有路由 loading Snapshot 无法通过。

| 构建宿主     | 产物                                 | 解压后启动                                  |
| ------------ | ------------------------------------ | ------------------------------------------- |
| macOS        | 包含 Solid GPUI.app 的 .zip          | 打开 .app，可移到 /Applications             |
| Linux        | 包含 usr/bin 与 usr/share 的 .tar.gz | 在解压目录执行 ./usr/bin/solid-gpui-website |
| Windows MSVC | 包含 solid-gpui-website.exe 的 .zip  | 打开可执行文件                              |

归档名包含版本和准确 Rust target，例如 `solid-gpui-website-0.2.0-aarch64-apple-darwin.zip`，并携带许可说明、内部 SHA256SUMS 和本指南。Unix 归档以 NATIVE-DEPENDENCIES.txt 记录观测到的原生依赖。校验和检测损坏，发布者身份认证需要签名。

THIRD-PARTY-NOTICES.md 是依赖清单，不是完整第三方许可文本集合。包中包含该清单及项目 MIT LICENSE；公开分发前仍需汇总并验证完整再分发说明。

Website Packages 工作流在 macOS ARM64、Linux x86-64、Windows x86-64 构建并上传已验证候选包，支持手动触发，也在打包输入变化的 PR 中运行。产物是未签名候选，不是公开发布。检查不证明 Windows/Linux 的显示、GPU、无障碍、输入法、菜单或通知正确性，发布前需真实桌面验证。

## macOS

.app 包含可执行文件、应用标识、英文元数据、版本和许可资源。构建默认 MACOSX_DEPLOYMENT_TARGET=13.0，并在 Info.plist 记录相同值。这是编译目标，不代表最旧版本已经测试；构建时应显式设置经过测试的最低版本。一次原生构建只生成一个架构，Intel 发布需 Intel 构建及独立冒烟测试。

打包拒绝非系统动态库，并添加用于本地评估的 ad hoc 签名。公开分发需 Developer ID 签名与公证。解压候选后签署最终 .app，以 `xcrun notarytool submit --keychain-profile <profile> --wait` 提交 ZIP，接受后用 `xcrun stapler staple` 附加票据，再重新创建 ZIP。验证签名与 Gatekeeper，并重新生成全部校验和，因为签名和 stapling 改变内容。凭证放在 Keychain 或发布 secrets。遵循 Apple [公证流程](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution)和 [bundle 元数据参考](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html)。

公证前替换候选的 ad hoc 签名：

```sh
codesign --force --options runtime --timestamp \
  --sign "Developer ID Application: YOUR NAME (TEAMID)" "Solid GPUI.app"
codesign --verify --strict --verbose=2 "Solid GPUI.app"
ditto -c -k --keepParent "Solid GPUI.app" Website-notarization.zip
```

QuickJS 是解释器，此包不嵌入 Bun JSC/JIT。图标、文档关联、entitlement、沙盒、更新与安装器品牌由实际交付应用负责，并需独立验证。

## Linux

便携归档依赖目标系统库与 GPU 驱动，不是通用静态二进制。在支持的最旧发行版基线上构建，检查 NATIVE-DEPENDENCIES.txt。候选使用 Ubuntu 24.04。使用目标字体、图形栈、桌面 portal 和缩放，分别测试 Wayland 与 X11。

从解压目录安装到 /usr/local：

```sh
sudo install -Dm755 usr/bin/solid-gpui-website /usr/local/bin/solid-gpui-website
sudo install -Dm644 usr/share/applications/io.github.cyenoch.solid-gpui-website.desktop \
  /usr/local/share/applications/io.github.cyenoch.solid-gpui-website.desktop
sudo mkdir -p /usr/local/share/doc/solid-gpui-website
sudo cp usr/share/doc/solid-gpui-website/* /usr/local/share/doc/solid-gpui-website/
```

desktop entry 按[Desktop Entry 标准](https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html)通过桌面会话 PATH 解析可执行文件名。卸载时删除这些应用专用路径。发行版管理安装可将 usr/ 树作为 .deb/RPM 载荷，并声明基线依赖。AppImage 和 Flatpak 是独立目标，需要 runtime 与 portal 验证，仓库不将未经测试的包装视为支持。

## Windows

使用 MSVC target、Visual Studio C++ Build Tools 和 Windows SDK。便携程序使用 Windows GUI 子系统，启动不弹控制台，需要可用原生图形栈及所选工具链运行时 DLL。托管 runner 已有开发依赖，仍需在干净 Windows 上验证。用 dumpbin /DEPENDENTS 检查导入，依赖时部署适当 [Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/choosing-a-deployment-method?view=msvc-170)，不能假定复制 exe 就具备全部运行依赖。

公开分发前使用发布者证书和时间戳服务通过 SignTool 签署最终 exe，再生成归档校验和。证书和时间戳选项见 [SignTool 参考](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool)。MSI/MSIX 增加安装、卸载、身份和更新策略，应在便携程序验证后按应用选择。仓库提供便携包和验证任务，不提供未经验证的安装器或自动更新器。

## 应用于其他应用

将 transport 选择留在运行入口：website main.native.tsx 选择 StdioTransport，quickjs.tsx 选择 EmbeddedTransport，mountWebsite 接收 transport factory 并拥有共享界面生命周期。先生成原生绑定再打包；普通原生开发构建不能依赖需要这些绑定才能生成的 bundle。

distribution Cargo feature 只启用独立 website binary 和 QuickJS。build script 要求绝对 SOLID_GPUI_WEBSITE_BUNDLE 路径，检查非空 UTF-8 源码并跟踪环境与文件变化。binary 通过 include_bytes! 嵌入复制的模块，启动 QuickJsAdapter::from_source 并传给 solid_gpui::run_application，共用正常宿主生命周期和退出行为，不解释开发宿主 CLI 参数，不添加临时文件启动器或第二应用进程。

依赖 Bun 服务的应用应显式保留 Bun runtime，与宿主一起打包。现有 host-release 任务生成 macOS 进程宿主归档，不会将其变成自包含应用。内嵌 Bun 仍是独立 macOS 构建路径，打包时不要静默把依赖 Bun 服务的应用切到 QuickJS。
