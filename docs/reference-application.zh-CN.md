# 参考应用

Reference Studio 是网站 Showcase 与桌面示例共用的原生应用工作负载。
Gallery Desktop 路由为 `/showcase/reference-studio` 和
`/showcase/reference-studio-history`；桌面示例的 Studio 链接打开
`/studio/timeline`，另一个视图为 `/studio/history`。运行界面和复制的源码
使用同一组 `ReferenceStudio.tsx`、`reference/state.ts` 与 `reference/model.ts`。

## 来源与适配

实现是原创代码，参考了固定提交中的 GPUIX timeline、infinite-chat 和
jhomra21/gpuix-solid timeline 示例。没有复制上游源码、资源或数据。
`src/showcase/reference/provenance.json` 记录提交、Git blob、SHA-256、源码映射
与明确的适配。固定来源证明所检查文件的身份，不代表与上游应用完全一致。

```sh
bun examples/website/scripts/check-reference-provenance.ts
bun --conditions=browser test examples/website/tests
```

离线校验可依次传入 GPUIX 和 gpuix-solid 仓库路径，读取固定提交的 Git 对象。
HTTP 校验使用不可变提交 URL、有限超时，并拒绝任何哈希不匹配。

指针裁剪、吸附、框选、播放和 JS 滚动偏移适配为标题/起点/时长受控输入、
原生拖放、时间窗口和缩放按钮，以及 GPUI 拥有的垂直滚动。轨道标题与片段
共用一个虚拟行，避免独立滚动流。历史使用确定性的内存评审数据和不同段落
数量，不宣称具备上游网络分页 MDX 聊天能力。`preview` 插槽可组合生成的
NativeView 绘制和媒体组件，资源由该组件负责取消与释放。

## 交互与所有权

初始数据有 240 个轨道、480 个片段和 10,000 条评审记录。原生 VirtualList
将 Solid 行所有者限制在视口与 overscan 2 范围内，首帧请求 8 个轨道与
5 条历史记录。数据存储和查询与数据量成正比，渲染和行所有者内存与可见
范围成正比。查询按受控文本记忆化；没有逐帧 JS 回调或全局订阅。

将片段拖到另一个轨道会保留时间与 ID。拖动轨道标题到另一个轨道会将其
插入目标之前。检查器可编辑 Unicode 标题和有限数值时间，位置限制在
120 秒项目内，时长至少一秒。发布评审插入唯一记录、清空查询并请求原生
列表滚到首行。文本段落可使用平台快捷键复制，也可用 Copy review 通过
根节点剪贴板命令复制完整评审。

Timeline 和 History 仅改变保留窗格的比例。窄窗口将非活动窗格宽度设置为
零而保持挂载；Inspector 打开保留的详情窗格。恢复宽窗口时，输入模型、
滚动句柄和选择仍然存在。离开应用后释放行所有者与句柄引用。

## 验收

语义夹具覆盖受控编辑、行所有者上限、末行查询、视图/尺寸变化后的节点
身份、发布/复制和卸载清理，不能证明原生几何、滚动位移、拖放命中、
选择绘制、系统剪贴板或物理显示。

`tests/reference-studio.native.ts` 提供可执行原生交互工作负载。驱动必须封装
显式启用的原生验收 API，返回实际绘制边界、节点身份和原生状态，并明确
拒绝不支持的必需能力。构建后串行运行，最长五分钟，不能用语义可见范围
事件替代原生滚动。相邻清单记录定位器与不变量。

验收包括非空首帧、实际滚轮位移和行上限、可到达末行、Unicode 保存、
拖放后继续点击、路由和宽/窄/宽调整后的身份与滚动保留、跨段落选择复制，
以及窗口关闭后的清理。平台支持时保存原生窗口截图；不支持的截图或输入
需明确记录。原生测试渲染与操作系统物理输入属于不同证据类别，不据此
宣称显示 FPS 或延迟。
