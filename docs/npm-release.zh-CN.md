# 发布 npm 包

应用从 npm 安装 `@solid-gpui/core`、`@solid-gpui/vite`，以及可选的
`@solid-gpui/router` 和 `@solid-gpui/shiki`。四个包与 Rust workspace 使用同一个
发布版本。各 scoped package 的 `publishConfig` 声明 public 访问和 npm registry。

[Publish npm packages workflow](../.github/workflows/npm-publish.yml) 在推送
`vMAJOR.MINOR.PATCH` tag 时运行：验证已提交版本、构建并测试包、审计 npm 依赖，
最后通过 npm Trusted Publishing（OIDC）发布已验证归档。GitHub 无需保存 npm token。
仅接受非零的稳定版本，拒绝预发布 tag。

## 一次性配置

需要拥有 `@solid-gpui` scope 发布权限的 npm 账号。如果 npm 上还没有这些包，先按
下文手动完成首次发布；包的设置页存在后才能配置 Trusted Publisher。使用经过检查的
真实版本，不发布占位包。

在 GitHub 仓库 **Settings → Environments** 创建名为 `npm` 的 environment，
在 deployment branches and tags 中允许匹配 `v*` 的 release tag。自动发布无需配置
required reviewers。

分别打开 `@solid-gpui/core`、`@solid-gpui/vite`、`@solid-gpui/router` 和
`@solid-gpui/shiki` 的 npm **Settings → Trusted publishing**，添加 GitHub Actions：

| 字段 | 值 |
| --- | --- |
| Organization or user | `Cyenoch` |
| Repository | `solid-gpui` |
| Workflow filename | `npm-publish.yml` |
| Environment name | `npm` |
| Allowed actions | 启用直接 `npm publish` |

workflow 只填文件名，不填路径。字段区分大小写。新建 publisher 默认允许 staged
publishing，必须明确启用 `npm publish`。无需 GitHub `NPM_TOKEN` 或
`NODE_AUTH_TOKEN` secret。workflow 使用 GitHub-hosted runner、Node 24 和固定的
npm 11.19.0（OIDC 要求 npm 11.5.1+、Node 22.14.0+）。

最新设置与要求见 [npm 官方文档](https://docs.npmjs.com/trusted-publishers/)。

## 准备版本

先在 `CHANGELOG.md` 中添加目标 `MAJOR.MINOR.PATCH` 章节，再从 SDK checkout
运行以下命令，将 `VERSION` 替换为目标版本：

```sh
bun install --frozen-lockfile
bun run task release-prep VERSION
bun run ci
bun run audit
```

`release-prep` 同步 Rust workspace、四个 npm 包版本及 core peer 范围，更新并验证
lockfile，重新生成依赖声明。CI 构建包并执行 packed-consumer smoke：在临时应用中
安装真实归档，验证 exports、类型、Vite 构建、原生绑定导出和公开测试入口。其中的
最小 Rust exporter 不验证原生窗口或平台渲染。

提交发布改动并合入 `main`，在通过检查的 commit 上创建 tag：

```sh
git tag vVERSION
git push origin vVERSION
```

tag 触发 npm workflow。CI 检查四个 manifest、core peer、Rust workspace 和 changelog
与 tag 是否一致，不在 CI 中改版本。package job 执行 `package-ci` 和 `bun audit`，
再使用 `npm pack` 上传四个归档。只有独立的 `publish` job 有 `id-token: write`，
它下载本次运行的归档，先发布 core 再发布集成包，使用 `latest` 并附带 provenance。
发布不执行包生命周期脚本，也不安装应用依赖。发布串行执行，新 tag 不取消正在运行的发布。

tag 同时提供匹配的 Rust 源码、vendor patches 和 workspace profiles；JavaScript
包不包含这些原生构建输入。npm gate 验证 JS 包和最小 Rust exporter fixture，不替代
打 tag 前的 native CI 与 release qualification，也不创建原生应用 release。

## 首次发布

前述检查会构建 `dist`。从同一份通过检查的 release checkout 检查 npm 包内容：

```sh
for package in solid-gpui solid-gpui-vite solid-gpui-router solid-gpui-shiki; do
  (cd "packages/$package" && npm pack --dry-run) || exit 1
done
```

包包含构建后的 JavaScript 与声明、显式源码 exports、README 和许可证，没有编译
原生宿主的安装钩子。源码 exports 用于调试 SDK 和仓库内示例，普通应用导入使用 `dist`。

推送首次 release tag，等待 workflow 的 package job 完成；包和 Trusted Publisher
尚不存在时，publish job 无法认证。下载本次运行的 `npm-release` artifact 并解压。
在安装 npm 的机器上交互登录，发布这些准确归档，先 core 再集成包。在解压目录中
将 `VERSION` 设为发布版本：

```sh
npm login
VERSION=0.4.0
for package in core vite router shiki; do
  npm publish "solid-gpui-$package-$VERSION.tgz" --access public --ignore-scripts || exit 1
done
```

npm 执行账号要求的验证，创建包设置页并发布到 `latest`；首次手动发布没有 GitHub
OIDC provenance。配置四个 Trusted Publisher 后重跑失败的 publish job，它会识别
相同归档并跳过已发布版本。之后的新版本 tag 自动通过 OIDC 发布。公布 release 前
确认四个版本：

```sh
npm view @solid-gpui/core version
npm view @solid-gpui/vite version
npm view @solid-gpui/router version
npm view @solid-gpui/shiki version
```

## 重试部分发布

四个 npm publish 不是原子操作。中途失败时，在同一次 GitHub workflow run 中选择
**Re-run failed jobs**，并确保归档仍在七天保留期内。publisher 先查询全部四个版本，
仅在 registry 的 `dist.integrity` 与归档字节一致时跳过已发布版本，再发布缺失版本。
已存在的归档不同，或 registry 返回 HTTP 404 之外的错误时，立即停止。不要移动源码
tag，也不要期望重新构建后不同的归档能替换部分已发布的版本。

手动 Release Prep workflow 仍只验证并上传候选包，不发布。本地归档检查与
packed-consumer 测试属于发布验证，而非另一套应用安装流程。没有自定义单包打包命令。

## 使用发布版本

遵循[入门指南](getting-started.zh-CN.md)。SDK 包固定为同一已发布版本，提交应用
lockfile，原生宿主使用对应源码 tag 或准确 commit。升级 SDK 时重建宿主并重新生成
绑定。npm 是包 registry，开发和运行工具仍然使用 Bun。
