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

## 内嵌 Bun 构建失败

内嵌 Bun/JSC 适配器只支持 macOS。进程模式使用 `bun run website:native`。启用 `embedded-bun` 的宿主接受显式应用入口并启动内嵌路径。失败时检查能否获取固定版本 Bun 源码，以及生成的原生构建图是否与仓库补丁匹配。

## QuickJS 无法解析服务或传输

QuickJS 宿主使用 `@solid-gpui/core/embedded` 的 `EmbeddedTransport`。`StdioTransport` 属于 Bun 进程或内嵌 stdio 环境，需要 `process.stdin` 和 `process.stdout`。

使用 `solid-gpui-build --runtime quickjs <entry.tsx> <output.js>` 将依赖打成一个 ESM 模块。Node/Bun 导入会被拒绝，环境不提供 `process`、`Bun`、文件系统或 `fetch` 等网络 API。将服务移到 Rust Native Module，调用生成客户端。打包器的 browser target 只选择可移植依赖，不会为 QuickJS 创建浏览器环境。

## 渲染器失败后进程宿主退出

检查 stderr 中的渲染错误及宿主崩溃报告路径。协议解码或验证失败是致命错误，继续执行会丢失 revision 一致性。应用渲染错误会抛给应用，Solid GPUI 不会自行生成备用界面。

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

通过 Vite 插件或 `solid-gpui-build` 使用共享 Solid/Oxc 通用转换。普通 React 风格 JSX 转换不能生成该渲染器的响应式宿主操作。

## 滚动没有边界或很慢

沿路由包装层检查整条 flex 父链，再检查填满窗口的工作区是否从不必要的内容派生 flex basis 开始。真实 Gallery 回归、校准后的 CPU 预算和原生验收要求见[原生滚动性能](scroll-performance.md)。

## 升级依赖后工具失效

TypeScript 7 通过 `typescript/unstable/async` 暴露编译器服务，根 `typescript` 不再提供 `createProgram`。在 Bun 中使用异步 API；同步客户端私有 Node 管道句柄不可用。等待 `api.close()`，让快照释放完成后再关闭连接。升级后重新生成 API fixture，运行语义重导出测试和 `package-typecheck`。

保持官方 Solid 编译器固定版本：候选版本与 Solid 1 运行时独立，共享转换禁用 Solid 2 内置自动导入。升级必须保留 Bun preload 和 Vite 两条路径的响应式更新、owner 清理、导入副作用和 source map。

## 关联原生命令错误

从 `@solid-gpui/core` 或 `/native` 入口导入 `NativeCommandError`。原生拒绝及结果格式错误的 `error.identity` 包含 Surface、epoch、请求、目标节点、命令类型及适用时的原生函数 ID。将这些字段与错误消息一起记录，关联协议 trace。标识不保留参数字节或返回载荷。应用错误字符串可能包含应用数据，应谨慎选择内容。信号取消保留原始 abort reason；传输关闭仍使用独立的 `TransportTerminatedError` 类型。
