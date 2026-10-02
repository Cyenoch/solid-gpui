# 原生文本选择与搜索

同一 Surface 中独立的核心 `<Text selectable>` 段落共享原生选区。GPUI 管理
anchor/head、富文本排版几何、拖动、键盘移动、复制与搜索高亮；Solid 管理组合
及可选事件回调。TextInput、Input、Editor、TextView 保留各自的原生编辑行为。

跨段落拖动后使用 macOS 的 Cmd+C 或其他平台的 Ctrl+C。复制在段落之间插入换行，
同一段落内的带样式 Text 连续拼接。Shift 点击扩展选区；方向键按字素移动，
Shift+方向键扩展；Home/End 在段落内移动；Cmd/Ctrl+A 选择已提交的可选文本；
Escape 清除。双击选择单词，三击选择段落。单段落内容更新仍会钳制原有选区。

## 生成的 Surface 服务

默认 host 和 NativeModules 包含 `solid-gpui-text` catalog。使用 host 的 prepare
重新生成绑定，从 `#native` 或 stock component host 的 `@solid-gpui/core/components`
导入 useNative。命令只操作发起调用的 Surface，包括弹出 Surface。自定义
ExtensionRegistry 需注册 `solid_gpui::native::text::native_module()`。

服务与观察者声明行为版本 `1.0.0`。生成 props 与 renderer 命令包含所选 host 的
严格构建信封。Rust 测试使用 `encode_native_request(module.build_digest(), &dto)`；
缺失或过期信封在读取或修改文档状态前拒绝。

```tsx
const native = useNative();
const result = await native.searchText({ query: "searchable" });
if (result.matches.length) await native.selectTextSearchMatch({
  textRevision: result.textRevision,
  searchRevision: result.searchRevision,
  matchIndex: 0,
});
```

命令包括 getTextSelection、setTextSelection、clearTextSelection、copyTextSelection、
searchText、getTextSearch 和 selectTextSearchMatch。位置为 `{ nodeId, offset }`，
offset 使用 UTF-16 单位并必须位于字素边界。设置选区需提供 getTextSelection 返回的
textRevision。结果包含 anchor、head、有序 spans、复制文本、selectionRevision 与
detached。无效或过期请求原子拒绝。nodeId 是稳定 Host Node 身份，不能使用列表索引
或无障碍标签替代；鼠标验收应使用原生 locator。

TextSelectionObserver 的 onSelectionChange 只发送 textRevision、selectionRevision、
searchRevision、selectedParagraphs、detached 元数据。卸载按生成事件的正常生命周期
结束订阅。应用按需读取选中文本，无需逐次鼠标移动镜像原生选区。

## 搜索与生命周期

搜索为区分大小写、Unicode 精确的字面量匹配，只覆盖已提交的核心可选 Text。
查询内换行可以跨段落匹配；空查询清除高亮。结果包含 matches、textRevision、
searchRevision、activeMatch、truncated。选择结果需同时提供两个 revision，原生端
设置选区、聚焦首段落，并在 VirtualList 中显示目标行。搜索与选区共用排版几何。
样式及 resize 不使文本结果失效；内容、顺序、可选状态、虚拟数据与物化窗口变更
会使缓存失效。收到 revision 事件后重新查询。未物化的 JavaScript 行不参与搜索。

选区最多保留 4,096 段落和 256 KiB 文本。查询最多 4,096 字节，搜索文档最多
256 KiB/4,096 段落；结果达到 1,024 项或 4,096 spans 后设置 truncated。超限请求
明确拒绝。绘制读取每个节点的缓存高亮，拖动命中扫描当前已绘制并考虑裁剪的段落。

重排保留稳定 anchor/head 并重新解析文档区间。跨段落选区内任一文本修改或删除
都会清除选区，单段落则钳制到新字素边界。VirtualList 行离开已提交窗口时，仅在
列表和 dataRevision 未变且行仍有效时保留有界复制字节，detached 为 true。
扩展拖动需有已选择的挂载行重叠，否则保留最后选区。替换/过滤数据或删除列表
清除保留字节。Surface epoch 替换与销毁释放选区、搜索、几何和观察者。
VirtualList 边缘拖动以有界原生步长滚动，每次活动拖动只安排一个后续回调；释放鼠标
或 epoch 销毁会停止。普通滚动容器保留滚轮/触控板滚动行为。

## 验收范围

TestAppContext 覆盖真实排版、跨段原生鼠标拖动、富文本 CJK/emoji/组合字符复制、
键盘扩展、生成命令、revision、重排、虚拟淘汰及清理。这些测试不等于物理操作系统
输入或显示呈现验收。每个平台仍需前台拖动/复制、拖动时滚动、宽窄 resize、后续
点击与卸载验收。浏览器剪贴板服从 Web host 权限限制；语义 TestHost 无原生几何。
