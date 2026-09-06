# Rust-owned native modules

组件与业务命令统一由应用实际链接的 Rust 模块声明；同一个 host 通过 `--export-native` 导出 TypeScript。`solid-gpui` 提供运行时、宿主、注解和可选的 gpui-component 集成，`@solid-gpui/core` 提供运行时、生成组件及 Vite 工具。应用只保留自己的 Rust crate，不再为 schema、导出程序、适配器和 JS 门面分别建包。

选择保留 Bebop 的有界消息传输，在 Extension/InvokeNative 的 bytes 字段里传递严格 JSON DTO；serde 与 ts-rs 负责类型推导。这样支持嵌套结构、枚举、默认属性和异步业务函数，并且 process 与 embedded 使用相同接口。没有选择直接暴露 JSC/GPUI 对象，因为线程、生命周期与 process 模式无法共享该模型；没有引入独立 IDL，因为它会再次分裂 Rust 实现和声明。

无状态组件以函数返回 GPUI Element；有状态组件实现 NativeView，由宿主持有 Entity、Subscription 和 Task。属性更新发生在提交时，绘制不重复解析 DTO。原生输入保留编辑状态，同值回传不调用上游 set_value；受控值同步回传，异步业务处理可与之分离。

模块标识来自命名空间，契约摘要来自导出的类型、组件/事件/方法元数据及宏可见声明。它检测注册与生成代码的契约差异，不是整个可执行文件的内容摘要。Rust 实现变化需要重新构建宿主。实例方法在完整 Solid 事务提交后调用，以节点身份、epoch 和 revision 校验；卸载撤销事件路由并拒绝未完成的 JS 实例调用。

业务 async 命令由模块集合共同拥有的 Tokio 运行时执行，同步命令进入 blocking pool；GPUI executor 不负责替代 Tokio 的 timer/I/O driver。请求以有界容量和可取消任务连接到 Surface 生命周期，结果先返回 GPUI，再作为消息进入 JS 事件循环。运行时析构启动后台关闭，避免在 UI 线程等待业务线程。已运行 blocking 函数与业务自行 detach 的子任务不具备自动强制取消保证。
