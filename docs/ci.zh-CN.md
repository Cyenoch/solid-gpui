# 持续集成

GitHub Actions 分别执行开发检查、依赖审计、网站部署和发布验证。所有工作流也都支持手动运行。

## 自动检查

| 工作流 | 自动触发条件 | 覆盖范围 |
| --- | --- | --- |
| [CI](../.github/workflows/ci.yml) | 源码、测试夹具、构建配置或 Actions 变化的 PR 与 `main` 推送 | 一个 macOS 作业运行 `bun run ci`，检查生成契约、Rust 格式/编译/Clippy/测试、包格式/类型/测试及包安装冒烟测试。 |
| [Cross-platform host](../.github/workflows/cross-platform.yml) | 原生宿主或渲染器输入变化的 PR 与 `main` 推送 | Linux Clippy 和库测试夹具；Windows 工作区检查与进程宿主链接。 |
| [Dependency audit](../.github/workflows/audit.yml) | 依赖清单、锁文件、审计配置或许可清单输入变化；每周一 03:37 UTC | Bun/Rust 安全公告及 macOS 上的第三方许可清单校验。 |
| [GitHub Pages](../.github/workflows/pages.yml) | 网站、文档、品牌资源、SDK、Rust 或构建输入变化 | WASM 构建、网站类型检查和测试；仅从 `main` 部署。 |
| [Embedded Bun](../.github/workflows/embedded-bun.yml) | 内嵌运行时、宿主生命周期、内嵌夹具或工具链/依赖输入变化 | macOS 26 上的内嵌 VM 生命周期测试与 Clippy。 |

仅修改网站读取的 `docs/*.md` 指南时运行网站工作流。智能体笔记和参考源码不会单独触发构建，除非也修改了列出的构建输入。
路径过滤器保存在各工作流中，YAML 锚点保持 push 与 PR 的过滤器一致。
添加分支保护的必需检查时，应考虑[路径过滤规则](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onpushpull_requestpull_request_targetpathspaths-ignore)：整个工作流被跳过时不会上报已完成的检查。

macOS CI 作业先检查 Rust 格式，再让 Rust 与 Bun 检查共享包生成和 Cargo 编译。Linux Clippy 已检查所有 targets，因此不重复执行 `cargo check` 或与平台无关的格式检查。Windows 仍保留工作区检查与宿主链接，以覆盖着色器编译器和链接器。显示和 GPU 验证见[分发指南](distribution.md)。

浏览器网站的类型检查和测试由 Pages 执行，先通过 `bun run website:build` 生成 WASM 模块。SDK 包检查不依赖网站已有构建产物，也不重复这些检查。组件示例检查器按网站的 tsconfig 解析类型，因此在 Bun 隔离安装依赖后，从仓库根目录运行也能正确解析。

Linux portal 依赖显式选择 Ashpd 的 `async-io` 后端，与 GPUI 保持一致；同时启用 Ashpd 默认的 Tokio 后端会导致编译失败。宿主 HTTP 适配器和浏览器资源下载器使用官方 Reqwest，使根锁文件不再包含已停止维护的 `rustls-pemfile`。macOS CI 和 Linux 库检查也会运行 HTTP 适配器的本地服务器测试，覆盖重定向策略、流式请求体、超时和代理配置。

审计根据 Cargo 元数据、`cargo-deny` 的 JSON 许可清单和 Bun 许可清单重新生成 `THIRD-PARTY-NOTICES.md`。依赖变化后运行 `bun run task third-party-notices` 更新它。每个本地工作区包都必须声明许可证，通常使用 `license.workspace = true`。Vendored GPUI crate 还必须在 `Cargo.toml` 的 `package.metadata.solid-gpui-vendor.source` 中声明上游包 URL；仅在说明文档中记录来源不会填入生成的清单。

## 发布验证

发布构建与归档按需执行：

- [Website Packages](../.github/workflows/website-packages.yml) 在 macOS ARM64、Linux x86-64 和 Windows x86-64 上构建并验证原生归档。
- [Host Release Candidate](../.github/workflows/host-release-candidate.yml) 先执行开发检查和审计，再构建并冒烟测试解压后的进程宿主。
- [Release Prep](../.github/workflows/release-prep.yml) 同步候选版本、执行检查和审计，并上传 core、Vite、router 和 Shiki 四个 npm 包。
- 手动运行 Embedded Bun 并启用 `candidate` 输入时，还会验证内嵌发布宿主；同一作业先完成内嵌功能检查。

这些工作流上传候选产物，不公开发布。已经压缩的归档上传时不再重复压缩。

`bun run ci` 是开发检查入口。审计与发布验证使用独立命令，日常开发无需构建未使用的发布归档：

```sh
bun run ci
bun run audit
bun run task host-candidate-smoke
bun run task embedded-check
bun run task website-package
```

## 缓存与验证

共享的 [Rust 设置 action](../.github/actions/setup-rust/action.yml) 先选择固定工具链，再恢复 [Rust 依赖缓存](https://github.com/Swatinem/rust-cache)。缓存键包含 runner 镜像、架构、构建用途、已安装编译器、Cargo 配置和依赖清单/锁文件。Pages 在计算缓存键之前安装固定的 Web nightly，不再为每次提交创建缓存。

保存前清理工作区构建产物和增量状态，CI 也禁用 Cargo 增量编译。PR 只恢复 Rust 缓存；推送和手动运行可保存缓存，包括检查失败之前已经编译完成的依赖。审计只缓存 registry，候选构建与开发/WASM 缓存相互隔离。

Embedded Bun 通过 `target` 目录外的 `SOLID_GPUI_BUN_CACHE`，在测试、Clippy 和 release profile 之间共享原生构建图。其独立缓存要求工具链和内嵌源码准确匹配，不会被通用 Cargo 缓存清理删除。

内嵌作业在恢复构建缓存前安装 Homebrew `llvm@21`，并将其可执行文件加入 `PATH`。固定的 Bun 源码要求 LLVM 21.1，Xcode 自带的 Apple Clang 属于另一套工具链。原生缓存键包含 LLVM 版本和工作流文件，编译器变化会使旧构建图失效。

`cargo-deny` 和 `wasm-bindgen-cli` 通过 [install-action](https://github.com/taiki-e/install-action) 安装固定版本、经过校验和验证的预编译程序，并禁用源码安装回退。Bun 保留 setup-bun 的可执行文件缓存；不再归档包缓存，因为[所检查的 CI 运行](https://github.com/Cyenoch/solid-gpui/actions/runs/34180146937)中恢复该缓存耗时五秒，而无缓存的工作区安装耗时四秒。

工作流变更后运行 `actionlint`、`bun test scripts/task-contract.test.ts` 和相关[网站检查](../examples/website/README.md)。在后续 GitHub 运行中对比缓存恢复/保存、构建耗时及 runner 总分钟数。本地验证不能证明托管 runner 的实际提速或跨平台发布质量。
