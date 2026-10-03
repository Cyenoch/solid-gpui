# 签名应用更新

签名更新是应用显式注册的原生服务。Rust 启用 `signed-updater` feature，
通过 `updater::native_module(Some(service))` 注册 `Arc<SignedUpdater>`。
默认宿主和 Gallery 不配置更新。Solid 管理用户同意和重启提示；Rust 管理
信任、下载、验证、解包、安装和回滚。密钥、路径和归档字节不跨 JavaScript 边界。

## 原生信任配置

`UpdaterConfig` 由应用的 Rust 代码设置，JavaScript 请求不能修改它：
`app_id`、`channel`、`feed_url`、32 字节 Ed25519 `public_key`、递增的
`current_sequence`、已有的绝对规范 `.app` `install_path`、位于
`Contents/MacOS` 下的相对 `executable`。拒绝路径中的符号链接。
`max_archive_bytes` 上限 256 MiB，`max_unpacked_bytes` 上限 1 GiB，
`max_entries` 上限 100,000，`timeout` 必须大于零且不超过五分钟。
生产保持 `allow_loopback_http: false`；临时测试可显式允许字面回环 IP 的 HTTP。

使用宿主现有的 `Arc<dyn gpui::http_client::HttpClient>` 或 `ReqwestClient`。
只允许 HTTPS，无凭证、片段或重定向，归档 URL 必须与信任 feed 同源。
原生执行器在有界阻塞线程池运行，检查取消并在任务真正结束前保留执行许可。
HTTP 首部和响应体停滞也可取消。每个实例只允许一个更新操作，OS 建议锁阻止
另一个更新器实例同时拥有该安装目录。
最后一个更新器所有者销毁时会在关闭文件句柄前显式解锁，因此子进程保留的共享
文件描述符不会延迟重新打开更新服务。
状态查询等待服务操作互斥锁，只检查完整事务状态，不在安装持锁期间读取中间的
暂存文件。安装、回滚、确认及返回状态共用此锁。

## 生成的命令

实际宿主生成 `#native` 类型。命令沿用原生字节限制、Surface 生命周期和
`NativeCallOptions.signal` 取消行为。

| 方法                       | 行为                                                               |
| -------------------------- | ------------------------------------------------------------------ |
| `updateStatus()`           | 启用状态、平台、待重启和回滚可用性                                 |
| `checkUpdate()`            | 验证 feed，返回 token、版本、序号和归档大小；旧版本返回 null       |
| `installUpdate({ token })` | 使用最近检查的 offer，验证下载并暂存、原子替换，保留旧包           |
| `rollbackUpdate()`         | 原子交换回旧包，再清理拒绝的候选包                                 |
| `confirmUpdate()`          | 新进程完成应用健康检查后确认；其原生 release sequence 必须匹配签名 |

`native_module(None)` 导出相同的真实合同，报告禁用并拒绝安装。桌面示例采用
此配置，并展示查询服务状态。token 不授予任意 URL 或路径访问权。
重新检查替换 offer；下一次更新前必须确认或回滚待确认的安装。

## Feed 与打包接口

feed 是严格 JSON `{ "payload": "小写十六进制", "signature": "小写十六进制" }`。
payload 是使用 Ed25519 签名的**原始 UTF-8 JSON 字节**，签名为 64 字节。
`ed25519-dalek::verify_strict` 在解析 payload 前执行，拒绝弱密钥和未知字段。
feed 上限 32 KiB，解码 payload 上限 12 KiB。私钥只属于发布工具，不放入应用。

payload 必须包含以下字段，具体示例和准确英文格式见
[权威英文指南](signed-updates.md#feed-and-packaging-format)：

| 字段                            | 值                                                   |
| ------------------------------- | ---------------------------------------------------- |
| `format`                        | `solid-gpui-update-v1`                               |
| `appId`, `channel`, `platform`  | 与原生配置完全匹配；例如 `macos-aarch64`             |
| `sequence`, `version`           | 递增整数与展示版本；不高于当前序号的更新不提供 offer |
| `archiveFormat`                 | `app-tar-v1`                                         |
| `url`, `archiveBytes`, `sha256` | 同源 URL、准确字节数、64 字符小写 SHA-256            |
| `bundle`, `executable`          | 精确 `.app` 根目录名和相对可执行文件路径             |

整个归档的签名哈希和大小必须匹配，暂存前再次验证 manifest 签名。
签名覆盖应用、平台、渠道、序号、URL、格式、大小和哈希；版本标签不参与排序。

`app-tar-v1` 是**未压缩 USTAR**，只包含一个准确 `.app` 根目录，且仅包含
普通文件和目录。拒绝符号/硬链接、特殊文件、稀疏条目、PAX/GNU 扩展首部、
绝对路径、反斜线、冒号、穿越、重复路径及超过 32 层的路径。不会恢复所有者、
xattrs 或特权权限位；普通文件为 0644 或 0755。配置的可执行文件必须存在并可执行。
此格式要求不含 framework 符号链接的自包含应用包。

通用打包器必须先完成 bundle、代码签名、公证策略、提取产物验证和 release
identity，再生成额外的更新 tar 与签名 manifest。普通 ZIP 和校验和 JSON
不是更新 feed。`@solid-gpui/vite/package` 的 `packageSignedUpdate()` 生成额外 USTAR，并验证应用拥有的 Ed25519 signer；新格式需要新合同，没有解包回退。

## 安装、恢复与平台限制

实际安装后端支持 macOS。在同一文件系统的私有相邻目录暂存、同步数据并原子
写入恢复记录，使用 `renameatx_np(RENAME_SWAP)` 一次交换。安装路径不会消失。
不支持该操作的文件系统明确报错。交换后的错误/取消立即尝试原子回滚；失败时
保留事务并报告错误。不会运行安装 shell 脚本。

`.Bundle.app.solid-update` 保存锁、签名记录和旧包，记录明确的事务阶段：

| 阶段 | 重新启动后的行为 |
| --- | --- |
| `staged` | 旧包仍安装时清理候选；新包已安装时保留旧包供确认或回滚。 |
| `restoring` | 新包仍安装时完成记录的回滚；旧包已恢复时同步目录，再清理拒绝的候选。 |
| `cleaning` | 验证记录的保留安装，继续删除候选；允许候选已经不存在。 |

清理先原子写入并同步 `cleaning` 阶段，保留安装身份和签名发布记录。
候选递归删除及目录同步完成前不会删除此记录；随后才删除记录并再次同步目录。
最后删除前失败会保留恢复元数据。记录删除后同步失败时，重启可见无记录，或
`cleaning` 记录与已不存在的候选，两者均对应相同的保留安装。
部分递归删除只会在候选目录身份匹配时继续。身份不符时停止并要求人工检查。
更新器不会使用 `current_exe` 自动选择安装目录。

**不会自动退出、启动进程或重新启动应用**。应用负责保存文档、协调其他实例、
提示用户退出并重开。`pendingRestart` 描述磁盘安装状态。新进程进行应用健康检查
后调用 `confirmUpdate`；旧进程无法确认更高序号的版本。
回滚也会在执行交换的进程中报告 `pendingRestart`：磁盘文件变化不会替换正在运行的代码。

Windows/Linux 构造更新器时返回不支持，可导出禁用合同；WASM 不提供模块。
macOS 临时 fixture 检查不代表其他 OS、公证/Gatekeeper、断电持久性或外部重启
资格。信任边界要求签名代码/密钥、原生配置和安装父目录属于可信应用；不提供
提权，也不防御有权并发改写这些目录的其他进程。测试只使用临时目录、本地 HTTP
和 fixture 密钥，不修改当前应用、凭证或发布产物。
