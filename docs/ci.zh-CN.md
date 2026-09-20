# 持续集成

GitHub Actions 分别执行开发检查、依赖审计、网站部署和发布验证。所有工作流也都支持手动运行。

## 自动检查

| 工作流                                                         | 自动触发条件                                                   | 覆盖范围                                                                                               |
| -------------------------------------------------------------- | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| [CI](../.github/workflows/ci.yml)                              | 源码、测试夹具、构建配置或 Actions 变化的 PR 与 `main` 推送    | 独立运行 macOS 原生/协议检查与 Linux SDK 包检查；保留的 `Rust and Bun checks` 汇总状态要求两者均成功。 |
| [Cross-platform host](../.github/workflows/cross-platform.yml) | 原生宿主或渲染器输入变化的 PR 与 `main` 推送                   | Linux Clippy 和库测试夹具；Windows 工作区检查与进程宿主链接。                                          |
| [Dependency audit](../.github/workflows/audit.yml)             | 依赖清单、锁文件、审计配置或许可清单输入变化；每周一 03:37 UTC | Bun/Rust 安全公告及 macOS 上的第三方许可清单校验。                                                     |
| [GitHub Pages](../.github/workflows/pages.yml)                 | 网站、文档、品牌资源、SDK、Rust 或构建输入变化                 | WASM 构建、网站类型检查和测试；仅从 `main` 部署。                                                      |
| [Embedded Bun](../.github/workflows/embedded-bun.yml)          | 内嵌运行时、宿主生命周期、内嵌夹具或工具链/依赖输入变化        | macOS 15 上检查所有 targets 的 Rust 集成 Clippy 和内嵌覆盖源码格式；不编译 Bun。                       |

仅修改网站读取的 `docs/*.md` 指南时运行网站工作流。智能体笔记和参考源码不会单独触发构建，除非也修改了列出的构建输入。
路径过滤器保存在各工作流中，YAML 锚点保持 push 与 PR 的过滤器一致。
添加分支保护的必需检查时，应考虑[路径过滤规则](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onpushpull_requestpull_request_targetpathspaths-ignore)：整个工作流被跳过时不会上报已完成的检查。

macOS 作业运行 `bun run task native-ci`：Rust 格式、协议与原生生成契约、跨语言 golden 夹具、默认特性工作区编译、严格 QuickJS Clippy 和 Rust 测试。Linux SDK 作业运行 `bun run task package-ci`：包格式、类型、测试和打包消费者冒烟检查。它为小型 native 工具夹具安装固定 Rust，但不编译 GPUI 宿主。两个作业独立启动，最终的 `Rust and Bun checks` 即使在依赖失败或跳过时也会执行，且只接受两者均成功；分支保护应继续使用这个汇总状态。

本地 `bun run ci` 仍组合两套检查，并共享记忆化的包构建。不要在同一 checkout 同时启动多个会构建包的 task 进程，它们的 `dist` 清理不相互协调。原生绑定检查归 `native-ci`，不再放在 JavaScript 包的 `package-ci` 中。

Watcher 夹具通过 `scripts/watch-fixture.ts` 发出相互独立的应用保存。Vite 内置 watcher 会合并 50ms 内的 `change` 事件，Linux inotify 交付较快，即使 HMR 已完成，下一次测试写入仍可能被合并。辅助函数将模拟保存间隔设为 100ms，不修改真实应用的 watcher 配置、不重试失败检查、不增加测试超时，也不删除断言。

macOS 的默认特性 `cargo check` 有意保留：QuickJS Clippy 启用了不同的依赖特性集合，不能验证消费者在不启用 QuickJS 时的编译契约。Linux Clippy 已用默认特性检查所有 targets，不重复执行 `cargo check` 或平台无关的格式检查。Windows 仍保留工作区检查与宿主链接。显示和 GPU 验证见[分发指南](distribution.zh-CN.md)。

Pages 精确恢复生成的 WASM 宿主缓存，未命中时执行 `build:host`，随后始终运行 `build:frontend` 和浏览器测试。SDK 包检查不依赖网站已有构建产物，也不重复这些检查。组件示例检查器按网站的 tsconfig 解析类型，因此在 Bun 隔离安装依赖后，从仓库根目录运行也能正确解析。

Linux portal 依赖显式选择 Ashpd 的 `async-io` 后端，与 GPUI 保持一致；同时启用 Ashpd 默认的 Tokio 后端会导致编译失败。宿主 HTTP 适配器和浏览器资源下载器使用官方 Reqwest，使根锁文件不再包含已停止维护的 `rustls-pemfile`。macOS CI 和 Linux 库检查也会运行 HTTP 适配器的本地服务器测试，覆盖重定向策略、流式请求体、超时和代理配置。

审计根据 Cargo 元数据、`cargo-deny` 的 JSON 许可清单和 Bun 许可清单重新生成 `THIRD-PARTY-NOTICES.md`。依赖变化后运行 `bun run task third-party-notices` 更新它。每个本地工作区包都必须声明许可证，通常使用 `license.workspace = true`。Vendored GPUI crate 还必须在 `Cargo.toml` 的 `package.metadata.solid-gpui-vendor.source` 中声明上游包 URL；仅在说明文档中记录来源不会填入生成的清单。

清单不包含生成时间戳：相同的已解析依赖和发布元数据必须在提交前后生成完全相同的清单。提交日期或无关任务定义的变化不应使清单失效。重新生成后运行 `bun run audit`；包测试本身不会校验许可清单。

## 发布验证

发布构建与归档按需执行：

- [Website Packages](../.github/workflows/website-packages.yml) 在 macOS ARM64、Linux x86-64 和 Windows x86-64 上构建并验证原生归档。
- [Host Release Candidate](../.github/workflows/host-release-candidate.yml) 先执行开发检查和审计，再构建并冒烟测试解压后的进程宿主。
- [Release Prep](../.github/workflows/release-prep.yml) 同步候选版本、执行检查和审计，再通过一次 `sdk-pack` 构建并上传四个包。带版本号的 artifact 内含 `solid-gpui-{core,vite,router,shiki}.tgz`。
- 手动运行 Embedded Bun 并启用 `candidate` 输入时，先完成轻量检查，再由独立的 macOS 26 作业编译 Bun、运行真实 VM 生命周期测试并验证内嵌发布宿主。

这些工作流上传候选产物，不公开发布。已经压缩的归档上传时不再重复压缩。

`bun run ci` 是开发检查入口。审计与发布验证使用独立命令，日常开发无需构建未使用的发布归档：

```sh
bun run ci
bun run audit
bun run task host-candidate-smoke
bun run task embedded-check
bun run task embedded-test # 编译 Bun；需要 macOS 26 SDK 和 LLVM 21.1。
bun run task website-package
```

`embedded-check` 仅为 Clippy 设置 `SOLID_GPUI_BUN_CHECK_ONLY=1`。sys crate 此时提供真实的 Rust FFI 声明，但不构建或链接 Bun，也不提供替代符号或 VM mock。这可以检查宿主侧类型和 lint（包括测试 targets），但不能验证与 Bun 的 ABI 兼容性、补丁应用、原生链接或 VM 行为。覆盖源码的格式检查会解析 Rust 语法，但不会针对 Bun 检查类型。运行时保证需通过 `embedded-test` 和手动候选工作流验证；构建可执行文件时不要设置该变量。

## 缓存与验证

共享的 [Rust 设置 action](../.github/actions/setup-rust/action.yml) 先选择固定工具链，再恢复 [Rust 依赖缓存](https://github.com/Swatinem/rust-cache)。缓存键包含 runner 镜像、架构、构建用途、已安装编译器、Cargo 配置和依赖清单/锁文件。Pages 在 WASM 成品缓存未命中时，先安装固定 Web nightly，再计算 Cargo 缓存键。Cargo 依赖缓存不为每次源码提交创建新条目。

保存前清理本地工作区/vendor 构建产物和增量状态，CI 也禁用 Cargo 增量编译。PR 只恢复缓存；只有推送或手动运行中成功的作业才能保存。精确命中的缓存不可修改，失败或仅检查的构建不应占据完整编译/测试作业的缓存。因此原生 CI 与 Embedded Bun 即使同为 macOS 15，也使用独立命名空间。审计只缓存 registry，候选构建与开发/WASM 缓存相互隔离。

Pages 将 `build:host` 的全部产物一起缓存：`examples/website/src/wasm` 和生成的 `packages/solid-gpui/src/components.ts`，不使用回退恢复键。只有 Rust 源码、嵌入资源、Cargo 配置/锁文件/清单、编译器和 bindgen 选择、原生导出器输入与构建工作流精确匹配时才跳过宿主编译；普通网站 TypeScript 和 Markdown 修改不使其失效。命中时跳过原生系统包及 Rust/bindgen 安装，但绝不跳过前端构建、类型检查或测试。仅在这些检查成功后保存，PR 不写入。新增 Rust 构建输入时同步维护 `pages.yml` 的键输入，参见 [Web 部署](web.zh-CN.md#github-pages)。

轻量内嵌作业不构建 SDK 包、不生成原生绑定、不安装 LLVM，也不下载原生构建图。其独立 Cargo 缓存只承担 `embedded-bun` Clippy 图，不与原生 CI 的测试/链接图竞争。

手动内嵌候选作业通过 `target` 目录外的 `SOLID_GPUI_BUN_CACHE`，在测试和 release profile 之间共享原生构建图。其独立缓存要求工具链和内嵌源码准确匹配，不会被通用 Cargo 缓存清理删除。

候选作业在恢复构建缓存前安装 Homebrew `llvm@21`，并将其可执行文件加入 `PATH`。固定的 Bun 源码要求 LLVM 21.1，Xcode 自带的 Apple Clang 属于另一套工具链。原生缓存键包含 LLVM 版本和工作流文件，编译器变化会使旧构建图失效。

`cargo-deny` 和 `wasm-bindgen-cli` 通过 [install-action](https://github.com/taiki-e/install-action) 安装固定版本、经过校验和验证的预编译程序，并禁用源码安装回退。Bun 保留 setup-bun 的可执行文件缓存；不再归档包缓存，因为[所检查的 CI 运行](https://github.com/Cyenoch/solid-gpui/actions/runs/34180146937)中恢复该缓存耗时五秒，而无缓存的工作区安装耗时四秒。

不要把缓存命中等同于避免了编译。基线运行精确恢复了 420 MiB 的 macOS 缓存，但仅首个协议 example 就重新编译了 412 个 crate。除命中率外还要比较编译器日志与缓存体积。优先隔离完整的依赖图，而不是叠加编译器缓存或为每个提交缓存整个 `target`；基线测量时仓库已接近默认缓存容量上限。

源码依据、基线耗时、保留的覆盖范围及托管验证结果见[优化研究](../.scratch/ci-optimization/research.md)。

工作流变更后运行 `actionlint`、`bun test scripts/task-contract.test.ts` 和相关[网站检查](../examples/website/README.md)。在后续 GitHub 运行中对比缓存恢复/保存、构建耗时及 runner 总分钟数。本地验证不能证明托管 runner 的实际提速或跨平台发布质量。
