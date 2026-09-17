# 分发原生应用

运行时策略将内嵌 Bun 用于依赖 Bun 服务的生产应用，QuickJS 用于主要能力由 Rust 实现的应用，外部 Bun 用于快速开发。本指南第一部分记录 QuickJS website 打包，这是仓库目前唯一交付并验证的流水线。第二部分记录实验性的内嵌 Bun 静态打包流程，它把 Bun/JSC 运行时与应用模块图链接进单个可执行文件；目前只有有限的运行时验证，不构成发布支持。输入与当前验证门槛见[内嵌 Bun 静态应用](#内嵌-bun-静态应用)，预期职责见[运行时策略](runtime-strategy.zh-CN.md)。

website 是参考应用包：Solid UI 编译为一个 ESM 模块，嵌入 Rust 可执行文件并由 QuickJS 执行，生产包不带内联 source map。用户无需 Bun、Node、仓库 checkout 或旁置 JavaScript bundle。Rust 拥有原生服务和渲染，同一界面组合也可在开发时通过 Bun 执行。

## 交付的是什么

三类产物有三个归属，只有最后一类才是交付物：

- **bundle** —— `bun --bun vite build` 的产物：供宿主执行的单个 JavaScript 入口模块，不含原生代码，也不是安装包。
- **原生可执行文件** —— Cargo 按目标与 profile 构建的 GPUI 宿主。`bun run generate`（`solid-gpui prepare`）与 Vite 构建都会为构建机生成一个；交叉目标可执行文件由同一 manifest 显式指定 `target` 得到。`.solid-gpui/artifacts.json` 记录可执行文件与 bundle 路径，`solid-gpui preview` 在构建机上把两者一起运行。
- **可分发包** —— 可执行文件加 bundle、资源、元数据、许可与签名，由你自己的打包脚本组装。本指南中没有任何构建命令会产出它。

平台能力同样如此区分：本仓库在下方记录的目标上验证 QuickJS website 包，而内嵌 Bun 静态打包器仍是实验性的，不声明任何受支持或已验证的目标。你自己应用在本地构建成功，只能说明你的应用，不代表其他平台已通过验证。

## 构建环境

安装 `.bun-version` 固定的 Bun 和 `rust-toolchain.toml` 指定的 rustup 工具链。从仓库根目录构建，打包任务在构建前按提交的工作区锁安装依赖。在准备发布的操作系统与架构上构建。

- **macOS：** 安装 Xcode 并选择开发者目录，工具链需提供 macOS SDK、C/C++ 编译器及 Metal 工具。release 前验证 `xcrun --sdk macosx --find clang`、`xcrun --sdk macosx --find metal`、`xcrun --sdk macosx --find metallib`。打包还使用系统 ditto、otool、plutil、codesign。
- **Windows：** 使用 MSVC Rust 工具链，安装含 Windows SDK 的 Visual Studio C++ Build Tools。release 使用 fxc.exe 编译 HLSL，GPUI 从 PATH 或 SDK 查找；自定义 SDK 布局可设置绝对 `GPUI_FXC_PATH`。打包使用 PowerShell Compress-Archive 和 Expand-Archive。
  Debug 构建把 HLSL 源码及其 include 一起嵌入可执行文件，运行时使用 Windows 系统组件 `d3dcompiler_47.dll` 从内存编译。不需要构建机器的源码目录，也不提取着色器文件；修改 HLSL 后需要重新构建可执行文件。
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

Website Packages 工作流在 macOS ARM64、Linux x86-64、Windows x86-64 构建并上传已验证候选包，发布验证时手动触发；普通 PR 执行开发和跨平台宿主检查。触发条件、缓存与其他候选工作流见[持续集成](ci.zh-CN.md)。产物是未签名候选，不是公开发布。检查不证明 Windows/Linux 的显示、GPU、无障碍、输入法、菜单或通知正确性，发布前需真实桌面验证。

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

将 `INSTALL_PREFIX` 设为预期的系统安装前缀，再从解压目录执行。以下命令拒绝未设置或为空的前缀：

```sh
sudo install -Dm755 usr/bin/solid-gpui-website "${INSTALL_PREFIX:?}/bin/solid-gpui-website"
sudo install -Dm644 usr/share/applications/io.github.cyenoch.solid-gpui-website.desktop \
  "${INSTALL_PREFIX:?}/share/applications/io.github.cyenoch.solid-gpui-website.desktop"
sudo mkdir -p "${INSTALL_PREFIX:?}/share/doc/solid-gpui-website"
sudo cp usr/share/doc/solid-gpui-website/* "${INSTALL_PREFIX:?}/share/doc/solid-gpui-website/"
```

desktop entry 按[Desktop Entry 标准](https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html)通过桌面会话 PATH 解析可执行文件名。卸载时删除这些应用专用路径。发行版管理安装可将 usr/ 树作为 .deb/RPM 载荷，并声明基线依赖。AppImage 和 Flatpak 是独立目标，需要 runtime 与 portal 验证，仓库不将未经测试的包装视为支持。

## Windows

使用 MSVC target、Visual Studio C++ Build Tools 和 Windows SDK。便携程序使用 Windows GUI 子系统，启动不弹控制台，需要可用原生图形栈及所选工具链运行时 DLL。托管 runner 已有开发依赖，仍需在干净 Windows 上验证。用 dumpbin /DEPENDENTS 检查导入，依赖时部署适当 [Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/choosing-a-deployment-method?view=msvc-170)，不能假定复制 exe 就具备全部运行依赖。

公开分发前使用发布者证书和时间戳服务通过 SignTool 签署最终 exe，再生成归档校验和。证书和时间戳选项见 [SignTool 参考](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool)。MSI/MSIX 增加安装、卸载、身份和更新策略，应在便携程序验证后按应用选择。仓库提供便携包和验证任务，不提供未经验证的安装器或自动更新器。

## 内嵌 Bun 静态应用

**实验性。** 目标机器无需旁置 JavaScript、仓库 checkout 或另行安装 Bun。目前 Windows ARM64 release 覆盖最多；签名、更广泛的安装环境策略、桌面与实体设备覆盖，以及所有其他 release 目标仍未完成，仓库 CI 尚未构建或验证静态应用包。仅构建成功不等于可发布。目前没有任何目标被声明为受支持或已验证；下方[平台状态表](#平台状态与当前证据)是唯一的验证证据，不支持的 triple 会提前失败并列出支持矩阵。

同一驱动有三条入口：应用使用公开 CLI 或库，本仓库使用脚本。

| 入口 | 用途 |
| --- | --- |
| `solid-gpui embedded package [flags]` | 随 `@solid-gpui/vite` 发布的公开 CLI。 |
| `@solid-gpui/vite/embedded` 的 `packageEmbeddedApplication({ sdkRoot, ... })` | 供构建脚本使用的库 API，返回打包报告。 |
| `bun packages/solid-gpui-vite/src/embedded/command.ts [flags]`（npm 脚本 `embedded:package`） | 本仓库 checkout 内的入口，参数相同并额外接受 `--sdk-root`。 |

消费方需显式传入 `sdkRoot`：即拥有固定 Bun/Rust 后端的 SDK checkout。这是受支持的接缝——消费方不导入仓库私有文件，也不复制该驱动。库参数为
`{ sdkRoot, entry, output, bun, target?, profile?, application?, assets?, workers?, baseExecutable?, cacheDir?, sourceCheckout?, ninja?, macosSdk?, deploymentTarget?, winsysroot?, prepareOnly? }`，
其中 `application` 是应用自有的 Cargo 输入：`{ manifest, package, features?, main? }`。
`manifest` 是应用清单（包或工作区根），`package` 是其包名，`features` 追加该包的 feature，`main` 是被 `include!` 进生成 bin crate 的 Rust 入口。

驱动在缓存目录准备带补丁的固定版本 Bun checkout，用 Bun 自己的构建脚本配合预编译 WebKit 构建原生图，用固定版本 Bun 序列化器序列化已构建的 Vite 入口，再把应用自身的 Rust crate 图与原生图一起编译。可执行文件写入 `--output` 并返回报告：输出路径及其 SHA-256、模块图 SHA-256、Rust triple 与图目标、profile、提取出的原生清单（若有）、类型化的 `entry` 身份（`role: "application"`、来源、身份）以及每个 `workers` 身份。生成的 Rust 暴露 `BUN_EMBEDDED_ENTRY` 与 `BUN_EMBEDDED_WORKERS`。打印结果中这些身份以 `Entry:` 与 `Worker:` 行给出虚拟图键。应用可用 `@solid-gpui/core/embedded` 的 `completeEmbedded(code)` 报告自身退出码（以 `supportsEmbeddedCompletion()` 探测支持），宿主经 `EmbeddedBunAdapter::result()` 读取，即使 VM 退出状态只有一个字节也能保留完整 `u32`。

### 静态打包输入

在当前宿主上原生构建，序列化器需由固定版本构建：

```sh
solid-gpui embedded package \
  --entry "$PWD/dist/app/index.js" \
  --bun "$PINNED_BUN" \
  --output "$PWD/dist/app/solid-gpui-embedded-app"
```

在 SDK checkout 内构建时，本仓库的 `bun packages/solid-gpui-vite/src/embedded/command.ts`（即 `bun run embedded:package`，会自动加上 `--sdk-root`）接受相同参数。在 macOS 上交叉构建 Windows x64 可执行文件时，额外给出目标、同版本的目标平台 Bun 与 sysroot：

```sh
solid-gpui embedded package \
  --entry "$PWD/dist/app/index.js" \
  --bun "$PINNED_BUN" \
  --base-executable "$PINNED_BUN_WINDOWS_X64" \
  --target x86_64-pc-windows-msvc \
  --winsysroot "$WINDOWS_SYSROOT" \
  --output "$PWD/dist/app/solid-gpui-embedded-app.exe"
```

将 `WINDOWS_SYSROOT` 设为已准备好的 Windows SDK/CRT 目录。

| 参数 | 含义 |
| --- | --- |
| `--entry`（必需） | Vite 生产构建产出的单一 ESM 应用入口的绝对路径。序列化器以该文件名命名模块图入口。 |
| `--bun`（必需） | 由 `crates/solid-gpui-bun-sys/bun-build.json` 固定版本构建的 Bun 可执行文件绝对路径。驱动读取 `bun --revision`，拒绝任何其他 commit，且无法关闭该检查：序列化载荷本身不带格式版本，仓库也不会替你构建、下载或安装该序列化器。 |
| `--output`（必需） | 输出可执行文件路径。父目录会自动创建，并打印结果的 SHA-256。 |
| `--target <triple>` | 目标 Rust triple，默认取固定 `nightly-2026-07-20` rustup 工具链报告的宿主 triple。支持 `aarch64-`/`x86_64-apple-darwin`、`x86_64-`/`aarch64-pc-windows-msvc` 以及 `x86_64-`/`aarch64-unknown-linux-gnu`/`-musl`，其他 triple 直接拒绝。 |
| `--profile debug\|release` | 默认 `release`。`debug` 使用 Bun 的 `debug-no-asan` 原生配置和 Cargo 的 `dev` 配置。 |
| `--source <dir>`、`--cache <dir>` | 复用已有的固定版本 checkout 而不重新克隆固定仓库，并指定准备好的 checkout 存放位置（默认 `target/bun-static`）。目录名由固定版本、内嵌补丁与覆盖源码派生，pin 变化必然重建。 |
| `--manifest <file>`、`--package <name>`、`--main <file>`、`--feature <name>` | 应用自有的 Cargo 输入：要构建的清单（包或工作区根）、其包名、被 `include!` 进生成 bin crate 的 Rust 入口（`--manifest` 必须与 `--main` 同时给出；单独给出 `--main` 时替换默认入口），以及可重复的 `--feature` 追加到该包。不传时驱动构建默认宿主入口。 |
| `--assets <file>`、`--workers <entry>` | 可重复。`--assets` 转为 `--asset` 参数，`--workers` 转为序列化器额外入口。运行时加载的每个资源和 Worker 都要在此声明；`new Worker` 或动态计算的 import 不会被自动发现。 |
| `--base-executable <file>` | 同固定版本的目标平台 Bun。当 `--target` 与本机平台架构不同时必需：否则序列化器需要下载 pin 未覆盖的 base 可执行文件，此时它会直接停止。 |
| `--macos-sdk <dir>`、`--deployment-target <version>`、`--winsysroot <dir>` | 以 `--macos-sdk=`、`--osx-deployment-target=`、`--winsysroot=` 转发给 Bun 构建脚本。 |
| `--ninja <file>` | 当 ninja 不在 `PATH` 时指定。 |
| `--prepare-only` | 只配置并构建原生图，打印 `embed-native.json` 路径，不构建应用。这是 Linux 目标目前唯一可用的模式；三个必需参数仍须提供，尽管该模式不会用到它们。 |
| `--help` | 打印参数列表。 |

### 静态打包前置条件

安装 `.bun-version` 固定的 Bun 并用它运行驱动：驱动通过当前解释器重新执行 Bun 自己的 `scripts/build.ts`。`crates/solid-gpui-bun-sys/bun-build.json` 固定 Bun revision、rustup 工具链 `nightly-2026-07-20` 与 ninja 1.13.0。该工具链、对应 `--target` 标准库，以及 `PATH` 中或用 `--ninja` 指定的 ninja 都必须已安装；原生图还会用 cargo 构建 vendored Rust 依赖。

先用 `bun install --frozen-lockfile` 安装仓库依赖，再在编译器可工作的宿主上用 Vite 编译 JSX/TSX 输入；打包器消费已编译的 JavaScript。

原生部分需要 C/C++ 工具链：LLVM 21.1.x（固定构建接受 `>=21.1.0 <21.2.0` 并自行解析）、cmake 3.24 或更新版本、`git`、用于 LUT 代码生成的 Perl，以及 Bun 构建脚本解析到的平台 SDK（macOS 上即 `brew install llvm@21` 与 Xcode）。准备好的 checkout 会自行安装其 JavaScript 依赖，因此首次运行需要网络。构建产物与准备好的 checkout 会缓存，首次构建远慢于后续，但缓存不会跳过版本与补丁校验。

- **macOS：** 通过 `xcrun` 解析 SDK，可用 `--macos-sdk` 覆盖。低于其最低部署目标（`13.0`）的 SDK 会被拒绝，`--deployment-target` 可记录你选择的部署目标。非 darwin 构建宿主还需要显式提供 macOS SDK；该跨宿主路径尚未验证。已观察到的宿主组合（2026-09-17，本机）：Bun 构建系统优先使用 Homebrew 的 `/opt/homebrew/opt/llvm@21/bin/clang++` 而非 `PATH`，而原版 LLVM 21 会拒绝 macOS 27 SDK 在 `os/trace_base.h` 中使用的 `stack_protector_ignore` 属性，报 `-Werror,-Wunknown-attributes`（`src/jsc/bindings/c-bindings.cpp` 中 59 处错误），因此本机无法用该 SDK 构建原生内嵌库。无需改代码的覆盖方式：打包器使用既有的 `--macos-sdk /Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk --deployment-target 26.5`；对于自行配置 Bun 的路径（例如 `cargo build/test --features embedded-bun`，它会运行 `crates/solid-gpui-bun-sys/build.rs`），使用 `SOLID_GPUI_BUN_MACOS_SDK` 与 `SOLID_GPUI_BUN_DEPLOYMENT_TARGET` 环境变量。ninja 1.13.0 必须在 `PATH` 中（本仓库为 `target/bun-tools/bin/ninja`）。请将其视为构建宿主的工具链要求——固定 LLVM 加匹配的 SDK——以及上述显式开关；它不对操作系统支持作任何声明。
- **Windows：** 可用 MSVC 工具链原生构建，也可在 macOS 上交叉编译：需要 LLVM 的 `clang-cl` 与 `lld-link`，以及 xwin 风格的 MSVC CRT/STL 与 Windows SDK splat，用 `--winsysroot` 或 `WINDOWS_SYSROOT` 指定。交叉构建的 ARM64 debug 还需要 SDK 的 ARM64 debug CRT（`libcmtd.lib`、`libcpmtd.lib`、`libvcruntimed.lib`），部分 xwin splat 会漏掉该载荷：应在 SDK 许可下获取并保留原始库，不能用 release CRT 替代。Bun 构建脚本在其错误信息中给出固定版本的 xwin 命令（SDK `10.0.26100`、CRT `14.44.17.14`）。原生构建先初始化真正的 Visual Studio 开发环境（`VsDevCmd.bat`；ARM64 使用 `-arch=arm64 -host_arch=arm64`），由它提供真实的 `VSINSTALLDIR`、SDK 工具、`INCLUDE` 与 `LIB`；Git for Windows 可能已在 `Git/usr/bin` 提供 Perl，把所需工具加入本次构建进程的 `PATH`，不要修改全局设置或 PowerShell 执行策略。
- **Linux：** 仅实现 `--prepare-only` 准备方式，含义见[平台状态](#平台状态与当前证据)。

以下四项构建宿主细节决定产物能否运行：

- **运行时与 profile。** 原生部分针对静态 MSVC 运行时编译（debug `/MTd`、release `/MT`），Rust 部分追加 `-Ctarget-feature=+crt-static`，因此应用必须用 `--target` 构建，才能让 build script 与 proc macro 继续使用动态 CRT。
- **ABI，而不只是架构。** 必须匹配固定预构建 WebKit 的 C++ ABI、架构与 CRT 模式。已验证的 Windows SDK/CRT splat 使用 MSVC `14.44.17.14`，原生 Windows 构建也可通过 `--winsysroot` 选择它。MSVC `14.51.36231` 头文件配合 LLVM 21.1.8 会产生不兼容的 ARM64 `std::partial_ordering` 返回约定，在 release 链接成功后触发 JSC 时钟比较断言，因此安装了更新的 Visual Studio 不等于 ABI 兼容。应一起使用匹配的原始头文件和库，不要修补 JSC 布局或关闭断言。
- **符号链接。** 解压源码归档需要构建宿主具备创建符号链接的能力，例如 zstd 的测试链接。应显式准备该能力；开启开发者模式是操作者控制的安全设置变更，不是打包器的职责。不要忽略解压失败或伪造缓存完成标记。这是构建宿主要求，绝不是应用运行要求。
- **真实着色器编译器。** Windows release 构建用 Windows SDK 中可执行的 `fxc.exe`（自定义位置用 `GPUI_FXC_PATH`）编译 HLSL。字节码必须为 DXBC（`vs_4_1`/`ps_4_1`），不能用 DXIL 替代；仅含头文件和库的 sysroot 不是着色器编译器，也不提供编译器转发或预编译着色器交换协议，不存在回退到 debug 着色器的路径，缺少编译器即构建失败。

始终传入 `--webkit=prebuilt`，源构建 WebKit 在配置阶段即被拒绝：打包器复用为固定 Bun 版本发布的预编译 WebKit/JavaScriptCore 归档（按操作系统、架构与 libc ABI 区分），既不从源码重建 JSC，也不替换为其他引擎构建。原生 manifest 记录其来源模式，消费者可据此拒绝源 WebKit 产物。打包器同样拒绝链接单独编译的 Bun Rust staticlib 的 manifest：只有一张 Cargo 图、一份 `std`，并继承 Bun 分配器作为唯一全局分配器。生成的根 crate 不得声明自己的全局分配器，否则构建会因分配器冲突失败。

### 驱动的执行步骤

1. 解析并校验目标 triple，不在支持表内即拒绝；Linux 未传 `--prepare-only` 时在此停止：ELF 应用模块图传输尚未实现。
2. 在缓存中准备固定版本 Bun checkout，应用 `crates/solid-gpui-bun-sys/bun_embed.patch`，复制内嵌覆盖源码，并校验 checkout 仍处于固定版本。
3. 以 `--mode=embed-native` 配置 Bun 原生图并用 ninja 构建：解析 codegen、vendored 依赖与预编译 WebKit 归档，编译全部 C/C++ 目标文件，但不链接 Bun 可执行文件或 Rust staticlib。
4. 校验 `embed-native.json`：schema 与目标匹配、Bun revision 等于 pin、WebKit 模式为 `prebuilt`、Cargo profile 与 `--profile` 一致、所有 object/archive 路径为绝对路径，且不包含 `bun_rust` staticlib。
5. 序列化应用。固定版本序列化器对入口及 `--workers` 入口执行 `bun build`，使用 `--compile`、目标的 Bun target 名、`--conditions=browser`，`--outfile` 指向私有临时文件，按需加入 `--asset` 与 `--compile-executable-path`；它在私有临时目录中运行，不会采用你项目里的 `bunfig.toml`、`tsconfig.json` 或 `package.json`。提取出模块图 section 后即丢弃该中间可执行文件。
6. 使用前校验模块图：每个文件键必须位于目标的虚拟根前缀之下（为其他平台序列化的图会被拒绝）、键唯一、入口键必须等于打包入口标识，且载荷必须能放入容器的 32 位 section 大小。PE machine 或 Mach-O CPU type 也必须与请求的架构匹配，因为仅靠共享的虚拟根前缀不能区分 x64 与 ARM64。
7. 生成应用 crate 与重放 manifest 中 object、archive 及链接策略的 build script，生成一次锁文件，再用 `--locked --target <triple>` 配合 manifest 的 profile、Rust flags、Cargo 参数与环境编译。写入 `--output` 前读取最终可执行文件、校验架构，并将提取的模块图 SHA-256 与序列化载荷比较；输出的是这些已验证字节，而非未经检查的链接结果，并保留链接产物的权限，保证新发布的 macOS 可执行文件仍可运行。

打包器让 `solid-gpui-bun-sys` 直接使用准备好的 manifest，该 build script 因此消费其中的 object、archive、链接策略与编译设置，而不再自行编译和链接内嵌库。不经过该驱动的直接 `embedded-bun` 库构建仍只支持 macOS，其他目标会被拒绝。

模块图字节成为最终镜像的一个 section：macOS 为 16 KiB 对齐的 `__BUN,__bun`，Windows 为 `.bun`；它经 dead stripping 保留，且生成在应用 crate 自己的目标文件中，避免归档提取时被丢弃。运行时直接从映射的镜像读取载荷，运行期不落盘、不重写。C ABI 为六个函数（`bun_embedded_create`、`bun_embedded_run`、`bun_embedded_result`、`bun_embedded_wake`、`bun_embedded_terminate`、`bun_embedded_destroy`）：打包入口带标签且与磁盘路径区分，`start_packaged(entry)` 不会回退到文件系统，无法启动的打包会话以负状态码 fail closed。其中 `bun_embedded_result` 是新增且叠加的，只读取应用声明的完成结果（`{ present, code }`），不改变运行状态。

### 默认应用入口

不传 `--main` 时，生成的 crate 会 `include!` `crates/solid-gpui-bun-sys/static-application.rs`：它定义启动打包模块图的进程入口，保留 crate 的固定标识（`solid-gpui-embedded-app 0.1.0`，即 `--version` 打印的内容），并提供一个验证开关：

- `--check-bundle` 在同一进程内启动两次会话，要求每次会话在 15 秒内交付初始 Snapshot，然后关闭运行时；任一步未完成即失败。
- 其他参数组合视为用法错误；无参数时正常启动应用。

`--check-bundle` 只是启动冒烟检查，不是验收：它不发送输入，因此不覆盖任何交互，也不证明应用用到的每个资源、Worker、动态 import 或服务都可达。Windows 目标以 Windows GUI 子系统编译，进程不会创建控制台窗口。

传入 `--main <file>` 可提供自定义 `main`：该文件在图 include 之后被 `include!` 进生成的 `src/main.rs`，因此可直接调用内嵌运行时，并加入自己的版本处理、参数与诊断；自定义入口需要自行承担默认入口的全部职责。

### 平台状态与当前证据

| 目标 | 当前状态 |
| --- | --- |
| macOS ARM64 debug | 已端到端构建并运行：无显示的两个会话探针完成计数输入并干净退出，另有真实 GPUI 窗口接受交互。它不是自包含的，限制见下文。 |
| Windows x64 debug | 在 macOS 上用固定工具链交叉链接，单独复制到未安装 Bun 与 Node 的 Windows 11 ARM64 虚拟机运行；两次同进程会话复现了相同的输入与退出结果。 |
| macOS ARM64 release | 未构建、未验证。 |
| Windows x64 release | 未构建、未验证。 |
| Windows ARM64 debug | 经完整打包器交叉链接，在 Windows 11 ARM64 虚拟机中原生通过两个模块图、输入和重启会话。尚无 GUI 或实体设备验收。 |
| Windows ARM64 release | 使用匹配的 MSVC 14.44 头文件/库、固定 release JSC 与真实 SDK DXBC 生成，在 Windows 上原生构建。完整模块图探针在独立标准用户下、以及从只读安装目录通过。另一次用户控制的 GUI 运行确认了预期标记、计数交互、缩放与正常关闭。更广泛的桌面、实体设备、依赖闭包与签名要求仍未完成。 |
| Linux | 仅 `--prepare-only` 原生准备。 |

该探针是真实的 Vite 编译 Solid 界面：Solid JSX、动态 `import()`、显式嵌入的 `node:worker_threads` Worker、通过 `Bun.embeddedFiles` 读取的资源、三次计数输入与干净退出。主 VM 与 Worker 都拒绝原生提取，默认 `fork()`/`cluster.fork()` 拒绝把宿主当解释器而外部系统命令仍可执行；固定版本序列化器会把 Vite 的动态 chunk 合并进入口，因此这不是最终模块图含独立动态模块记录的证据。这些是针对指定 fixture 的检查，不等于完整文件系统追踪或应用依赖闭包。

标准用户检查使用不同于构建者的本地账户，仅属于内置 `Users` 组，令牌为中等完整性，且不含 Administrators SID（包括 deny-only）。可写安装的正向对照允许新建文件和非截断写打开 EXE；在只读安装中，这两项操作在运行前后均因权限拒绝而失败，安装路径含 Unicode 与空格。两次运行均使用空 `PATH` 与私有 `TEMP`/`TMP`，通过两个会话、退出码为零、无临时文件残留且 EXE 哈希不变。这些无界面检查不构成标准用户或只读安装限制下的 GUI 验收。

不要把该表理解为更大范围的支持，尤其注意：

- **macOS debug 可执行文件不是自包含的。** 它继承工具链提供的非系统 UBSan 运行时，无法复制到缺少该运行时的机器，也不构成依赖闭包证据。把 debug 二进制复制到干净机器不是受支持的分发方式。
- **Windows 结果仍是有限 fixture 证据，现已包含 ARM64 release。** 其他机器上的 GUI 正确性、实体设备覆盖、完整系统依赖闭包、签名与公开支持仍未验证，因此公开 Windows 支持保持实验性。release 着色器生成跟随目标平台和目标 debug-assertion 配置，而不是 build script 的宿主或 profile；Windows ARM64 release 构建生成了整套 DXBC 着色器并通过 SDK 反汇编器检查，这证明从源码生成着色器可行，不等于 GPU 或完整 release 验收。
- **图形栈与驱动仍由目标环境提供。** 该可执行文件与任何原生 GPUI 应用一样需要可用的图形栈、字体与 GPU 驱动，并导入正常操作系统库，例如 Windows 上的 `icuuc.dll`；Windows debug 构建还会导入 `d3dcompiler_47.dll`。不含 Bun 与 VC 运行时 DLL 不等于目标环境什么都不需要。

### 运行时与打包限制

- **不提取捆绑原生库。** 模块图校验会拒绝 JSC 字节码与 module-info（运行时需就地改写字节码）、N-API addon，以及任何 `.node`、`.dll`、`.so`、`.dylib` 条目，因为它们只能靠运行时写盘来服务。需要运行时提取的原生依赖无法打包，应改为静态集成。仅检查扩展名无法识别改名后的原生资源。内嵌构建还会禁用主 VM 与 Worker 共用的原生提取函数，并拒绝 FFI 模块图路径；这不是外部文件系统或 FFI 访问沙盒。
- **没有通用 fork/cluster JavaScript 解释器。** 内嵌 `fork()` 会拒绝默认解释器，以及与 `process.execPath` 完全相等的显式 `execPath`；`cluster.fork()` 复用此限制。宿主不是 Bun CLI，普通外部进程启动不变。路径别名和一般同 EXE 启动并未被全面拦截，显式指定外部解释器会成为额外的应用依赖。镜像内并发应使用声明过的 Worker 入口，并相对于 `import.meta.dirname` 而不是进程工作目录解析路径。
- **资源闭包由应用负责。** 通过 `--workers` 与 `--assets` 声明应用 Worker 和资源。运行时打开的任意目标文件系统路径不会自动嵌入；应把这些应用资源加入模块图，或明确准备所需外部文件。
- **打包器只产出裸可执行文件。** 它不附带应用 bundle、图标、元数据、entitlement、许可说明或归档，因此生成 macOS `.app`、归档或校验和文件仍属于你的发布步骤；macOS 参考上文 bundle、签名与公证流程，Windows 参考上文 SignTool 流程。
- **本流程不改变 website 包。** website 继续使用上文 QuickJS 流水线，本流程不会替换或扩展该包、其归档布局或验证方式。

### Windows 验收门槛

以下门槛彼此独立：候选编译成功或静态检查通过，不代表其余项目通过。每次结果均保留 EXE 哈希、Windows build、架构、账户证据、命令、输出和退出码。打包流程不会创建账户、修改 ACL、放宽 PowerShell 执行策略或安装证书信任。

Windows ARM64 release 候选已通过原生架构、release 构建与基本 GUI、独立标准用户无界面、只读安装无界面检查。这些结果不代表更广泛的 GUI/设备覆盖、依赖闭包或签名通过；范围见[平台状态](#平台状态与当前证据)。

| 门槛 | 实施方式 | 所需证据 |
| --- | --- | --- |
| 独立标准用户 | 使用操作者预先准备、不同于构建/原测试账户的本地非管理员账户。 | SID 不同、账户组归属已核查、令牌未提升，且实际应用探针通过。UAC 过滤后的管理员不算标准用户。 |
| 只读 EXE 目录 | 将已校验候选放到预先准备的只读/可执行位置，包含 Unicode 与空格路径；原地启动，工作文件、日志和 TEMP 位于其他可写目录。 | 目录中新建文件与对 EXE 的非截断写打开均因权限拒绝失败；启动、输入、关闭通过，EXE 哈希不变。 |
| 依赖闭包 | 先执行下述静态门禁，再在每个声明支持的 OS/架构上检查应用场景的实际模块加载及文件访问。 | 普通/延迟导入经过审核；运行时、图形、字体、配置、资源分别有证据。采样不等于完整跟踪。 |
| Release | 用 `--profile release`、固定 release JSC 产物和可执行 Windows SDK DXBC 编译器构建。 | release EXE 本身通过应用探针、导入审计及用户控制的 GUI 场景，不能借用 debug 证据。 |
| 签名 | 最终链接后，显式选择发布者证书及 RFC 3161 服务，验证后再发布。 | SignTool 签名和验证都返回零、签署者匹配、时间戳存在，校验和对应签名后的字节。 |
| 原生 ARM64 | 同固定版本的 Bun 基座和应用都构建为 `aarch64-pc-windows-msvc`，不复用 x64 基座；x64 与 ARM64 的基座、输出和证据必须分开保存。 | PE machine 为 ARM64，探针在 ARM64 Windows 通过，`IsWow64Process2` 返回 process machine `0`、native machine `0xAA64`。 |

账户检查在测试账户自身执行，不能通过提升的辅助进程执行：

```powershell
whoami /user /fo csv /nh
whoami /groups /fo csv /nh
Get-LocalGroupMember -SID 'S-1-5-32-544' | Select-Object Name, SID
```

将用户 SID 与单独记录的构建账户 SID 比较，同时核对令牌组（包括 deny-only Administrators）和账户数据库。`.NET WindowsIdentity.Groups` 在受限制令牌下可能省略管理员 SID，不能单独作为证据。High/System 完整性、服务账户、未解析的嵌套组归属或无法读取的组信息均不通过；域账户还需要目录侧成员证据，不能仅靠本地组列表下结论。

只读检查使用 `File.Open`：对 EXE 目录里的唯一探针名指定 `CreateNew`/`Write`，对 EXE 本身指定 `Open`/`Write`。两者都不会截断已有文件，只有 access denied 才是预期失败；共享冲突、路径不存在等错误不能代替。若成功，关闭句柄并仅删除本次创建的唯一探针，判定该位置未通过。文件只读属性或不可写 CWD 均不等于只读安装目录。

以 `--check-graph` 运行使用 `fixtures/embedded-static-check.rs` 构建的候选：清空子进程 `PATH`、使用私有可写 `TEMP`/`TMP`、清除 Bun/Node 覆盖变量，并发排空 stdout/stderr。外层期限为 60 秒，超时或验收框架异常时只终止自己创建的子进程。必须同时看到两条会话输出、两条 `PASS:` 和退出码零，运行后重新核对哈希。即使设置 `CreateNoWindow`，原生断言也可能弹窗；GUI 验收仍由用户单独控制。默认应用入口仅有覆盖更窄的 `--check-bundle`。

宿主侧静态门禁复用 LLVM，而不是新造 PE 依赖解析器：

```sh
bun scripts/check-embedded-dependencies.ts --exe target/application.exe --arch arm64 --readobj "$LLVM_READOBJ"
```

将 `LLVM_READOBJ` 设为选定的工具路径，或省略 `--readobj`，使用 `PATH` 中的 `llvm-readobj`。它输出 JSON，将普通与延迟导入同已审核系统导入集合比较：退出 `0` 只表示此有限契约通过，`1` 表示架构或导入违规，`2` 表示输入/工具输出不可用。新增导入必须审核；不允许 Bun/JSC DLL 或动态 CRT 导入。报告绑定 SHA-256，检查期间镜像变化会被拒绝，但它不解析 API set、不递归 OS DLL、不认证实际解析到的模块、不跟踪 `LoadLibrary`，也不证明图形、字体或资源闭包；仅凭 DLL 名不能确定应用最低 Windows 版本。完整场景证据需另行授权 Windows 镜像/文件跟踪，覆盖启动、Worker、动态导入、图形、字体、输入、关闭，并一并审查可选 `.env`/`bunfig.toml` 读取：本流程没有改变固定序列化器的运行时自动加载默认值。不要为了制造绿色结果添加自制加载器、ETW 服务或 API-set 解析器。

### 签名顺序

先嵌入模块图、编译并链接，再对最终产物签名，最后计算发布的校验和。签名后再做任何修改，或签名后重写镜像、追加载荷，都会使证据失效。macOS 上交付物通常是包含该可执行文件的已签名并公证 `.app`：应先组装，再签署最终 bundle，而不只是其中的二进制。Windows 上先用 SignTool 与时间戳服务签署最终 `.exe`、验证签名，然后才发布校验和，具体见上文 Windows 章节。签名凭证不得进入包与构建缓存。

内嵌 Bun 候选的发布者签名仍暂缓，因此在拿到真实证书、时间戳并成功验证之前，签名门槛保持开放；提供证书从不代表授权静默接受硬件或提供商提示。

## 应用于其他应用

将 transport 选择留在运行入口：website main.native.tsx 选择 StdioTransport，quickjs.tsx 选择 EmbeddedTransport，mountWebsite 接收 transport factory 并拥有共享界面生命周期。先生成原生绑定再打包；普通原生开发构建不能依赖需要这些绑定才能生成的 bundle。

distribution Cargo feature 只启用独立 website binary 和 QuickJS。build script 要求绝对 SOLID_GPUI_WEBSITE_BUNDLE 路径，检查非空 UTF-8 源码并跟踪环境与文件变化。binary 通过 include_bytes! 嵌入复制的模块，启动 QuickJsAdapter::from_source 并传给 solid_gpui::run_application，共用正常宿主生命周期和退出行为，不解释开发宿主 CLI 参数，不添加临时文件启动器或第二应用进程。

依赖 Bun 服务的应用，在其当前限制可接受时使用[内嵌 Bun 静态打包流程](#内嵌-bun-静态应用)，或显式保留 Bun runtime 与宿主一起打包。host-release 任务生成 macOS 进程宿主归档，不会将其变成自包含应用。直接构建 `embedded-bun` Cargo feature（不经静态打包器）仍只支持 macOS 并拒绝其他目标，因此 Windows 及其他架构要靠静态打包器，而它仍是实验性的。打包时不要静默把依赖 Bun 服务的应用切到 QuickJS。
