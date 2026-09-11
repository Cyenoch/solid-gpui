# 使用 captureState 保留界面状态

原生 UI 热更新会创建新的 Solid owner 和组件实例。通过 `mountApplication` 的
`captureState` 显式捕获应用数据，再在新一代的 `setup(previous)` 中恢复。
只增加捕获回调而不读取 `previous`，保存源码后信号仍会回到初始值。

## 完整的 QuickJS 示例

Vite 配置使用 `solidGpui({ entry: "src/main.tsx", runtime: "quickjs", native })`，
入口采用 `EmbeddedTransport`：

```tsx
import { mountApplication, Pressable, Text, TextInput, View } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";
import { createSignal } from "@solid-gpui/core/runtime";

type ReloadState = {
  page: "downloads" | "settings";
  query: string;
  directoryDraft: string;
};

mountApplication<ReloadState>({
  transport: () => new EmbeddedTransport(),
  setup(previous) {
    const [page, setPage] = createSignal<ReloadState["page"]>(previous?.page ?? "downloads");
    const [query, setQuery] = createSignal(previous?.query ?? "");
    const [directoryDraft, setDirectoryDraft] = createSignal(previous?.directoryDraft ?? "");
    return {
      captureState: () => ({ page: page(), query: query(), directoryDraft: directoryDraft() }),
      render: () => (
        <View style={{ padding: 24, gap: 12 }}>
          <Text>Current page: {page()}</Text>
          <Pressable onPress={() => setPage(page() === "downloads" ? "settings" : "downloads")}>
            <Text>Switch page</Text>
          </Pressable>
          <TextInput value={query()} onChangeText={setQuery} accessibilityLabel="Search" />
          <TextInput value={directoryDraft()} onChangeText={setDirectoryDraft} accessibilityLabel="Folder draft" />
        </View>
      ),
    };
  },
});
```

切换页面并输入两个字段，再修改 JSX 并保存。新一代会先恢复捕获的数据，再渲染。
`captureState` 与 `render` 一起从 `setup` 返回；在回调内部读取信号，才能每次
捕获最新值。

外部 Bun HMR 使用 `StdioTransport`，并在 `mountApplication` 配置
`hotKey: import.meta.hot ? import.meta.url : undefined`。QuickJS 的 generation
标识由宿主提供，不需要 `hotKey`。运行时配置见 [Vite 集成](vite.zh-CN.md)。

## 显式捕获少量数据

| 适合捕获 | 应重新创建或读取 |
| --- | --- |
| 当前页面、搜索、筛选、排序和选中 ID | Solid owner、accessor、effect 和定时器 |
| 未保存表单及校验结果的展示选择 | Native client、组件 ref、窗口和 transport |
| 草稿所基于的保存 revision | Rust 中的实时引擎状态和下载进度 |
| 手动重试需要沿用的请求 ID | Promise 和尚未完成的回调 |

将重要表单状态放在 `setup` 所拥有的模型里，再传给组件。组件内部信号不会自动被
捕获。每个模型可以接受自己的旧数据并提供 `captureState`，应用再组合成一个快照。
数组和 record 返回副本，例如 `selected: [...selected()]`、`draft: { ...draft() }`。

QuickJS 只接受无环 JSON：普通对象、稠密数组、字符串、布尔值、除负零外的有限数字和 null。
缺省属性应省略或使用 null；嵌套 undefined、函数、getter、Map/Set、symbol 和
原生句柄都会被拒绝。上限为 1 MiB、100,000 个值和 64 层。不要用 JSON
stringify/parse 往返来静默丢弃不支持的值。详见[捕获契约](hot-reload.zh-CN.md#捕获状态)。

## 重连服务时保留草稿

在 `setup` 同步地根据 `previous` 创建信号；待候选激活后，在 `onMount` 连接原生
客户端并读取最新后端数据。旧订阅和定时器通过 `onCleanup` 清理。

如果加载函数总是执行 `draft = saved.values`，恢复的表单会在请求返回后再次丢失。
刷新目录时保留脏草稿和原 revision；保存值已改变时显式保留冲突，用户放弃草稿后
再接受最新值。恢复界面数据不应自动提交表单或重放原生命令。

对于不能安全跨越重载的写操作，可以明确拒绝捕获：

```ts
captureState() {
  if (saving()) throw new Error("Wait for the save to finish, then save source again to reload.");
  return { draft: { ...draft() }, revision: revision() };
}
```

QuickJS 捕获失败会保留当前 generation 的交互能力。操作完成后再次保存源码即可
重试。不要恢复 busy 标志或 Promise，假装旧 VM 的操作仍绑定在新 VM 上。

## 各类重启的保留范围

| 事件 | 行为 |
| --- | --- |
| TS/TSX 成功热更新 | 新 UI generation 收到捕获数据，Rust 宿主仍运行。 |
| 捕获或候选准备失败 | 恢复上一代 QuickJS UI。 |
| Rust 重建、原生契约改变或宿主崩溃 | 新建宿主会话，内存检查点丢失。 |
| 完整应用重启 | 只恢复应用明确持久化的数据。 |

原生焦点、选区、滚动缓存及组件实例会重新挂载，除非应用自行恢复受支持的状态。
`captureState` 不替代持久化，也不会让异步原生副作用变得可回滚。

## 验证真实重载

修改草稿、切换导航并选中条目，再保存 TSX。等原生数据加载完成后确认值仍保留，
并继续操作它们。还应验证读取失败和保存期间触发重载。QuickJS 除了 `applied`
还要检查捕获错误；成功激活不等于后续加载没有覆盖草稿。见[重载验证](hot-reload.zh-CN.md#验证应用重载)。
