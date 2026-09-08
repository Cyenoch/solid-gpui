# 系统弹层

`SystemPopover` 将内容挂载到有明确父窗口的原生窗口，因此可以超出父窗口边界。
从 `@solid-gpui/core` 导入。`@solid-gpui/core/components` 中的 `Popover`
仍在当前窗口内呈现，也适用于 Web。

## 受控表单

```tsx
import { Pressable, SystemPopover, Text, TextInput, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";

export function EditName() {
  const [open, setOpen] = createSignal(false);
  const [name, setName] = createSignal("Ada");
  const [error, setError] = createSignal("");
  return (
    <View style={{ gap: 12 }}>
      <Text>Saved name: {name()}</Text>
      <SystemPopover
        open={open()}
        onOpenChange={setOpen}
        width={340}
        height={220}
        placement="bottom-start"
        gap={8}
        accessibilityLabel="Edit name"
        onError={(error) => setError(String(error))}
        slots={{ trigger: <Text>Edit name</Text> }}
        content={() => (
          <View
            accessibilityRole="dialog"
            accessibilityLabel="Edit name"
            style={{ padding: 20, gap: 12, widthPercent: 100, heightPercent: 100 }}
          >
            <TextInput accessibilityLabel="Name" value={name()} onChangeText={setName} />
            <Pressable focusable onPress={() => setOpen(false)}>
              <Text>Save {name()}</Text>
            </Pressable>
          </View>
        )}
      />
      <Text>{error()}</Text>
    </View>
  );
}
```

触发插槽只放视觉内容。组件提供可聚焦的 Pressable、按钮角色、展开状态及切换动作，
不要在里面再嵌套独立的交互按钮。纯图标触发器需要 `accessibilityLabel`。

`content` 是内容工厂：在工厂内创建目标 Surface 的元素，不要返回父树中提前创建的元素。
Solid context 和 signal 通过既有 owner 共享。需要跨关闭保留的状态（如 `name`）放在
工厂外；内容内部状态和资源随弹层关闭释放。
卸载触发器会释放其弹层；若重新挂载时受控 `open` 仍为 true，则创建新的子 Surface。
创建或渲染子根不会改变父组件所属的 Host Tree，父树中的条件渲染和重新挂载仍由父 Surface 承载。

## 属性

| 属性                    | 约定                                                                                    |
| ----------------------- | --------------------------------------------------------------------------------------- |
| `open` / `onOpenChange` | 必填受控状态。触发器切换状态；原生关闭或创建失败请求 `false`。                          |
| `width`, `height`       | 必填逻辑内容尺寸，正整数，单轴不超过 16384。可用屏幕区域可能约束实际尺寸。              |
| `placement`             | 默认 `bottom-start`。支持四个方向及各自的 `-start` / `-end`；对齐按物理左右、上下定义。 |
| `gap`                   | 逻辑间距，范围 0–1024，默认 8。                                                         |
| `slots.trigger`         | 父 Surface 中的触发器视觉内容。                                                         |
| `content`               | 返回弹层 Solid 内容的工厂。                                                             |
| `accessibilityLabel`    | 触发器的无障碍名称；内容对话框另行命名。                                                |
| `onError`               | 处理原生创建、定位准入错误。未提供时，错误进入捕获的 Solid 错误 owner。                 |

尺寸、方向和间距在打开时读取，更改后于下次打开生效。触发器布局、滚动和父窗口边界
变化会驱动原生重定位，无需 JavaScript 几何订阅。完全裁剪或移除触发器会关闭弹层。
关闭后不保留几何观察器或动画定时器。宿主最多允许 32 个存活弹层。

## 平台与多显示器

| 宿主                     | 实现及边界                                                                                                                                                               |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| macOS                    | 可获取键盘焦点的无边框 NSPanel，附属于实际父窗口。AppKit 按逻辑点转换视图到屏幕坐标，由锚点选择屏幕及可见区域。它不是 AppKit NSPopover，也不是 SwiftUI 容器。            |
| Windows                  | 有 owner 的 `WS_POPUP` 和 `WS_EX_TOOLWINDOW`。父窗口客户区原点、工作区、父窗口 DPI 和目标 DPI 统一转换为物理屏幕坐标，不使用全局置顶或禁用父窗口。                       |
| Linux X11                | override-redirect 临时窗口，RandR 显示器区域与 WM 提供的工作区相交。使用后端的桌面统一缩放，不承诺独立的每屏缩放；需要有效的 RandR 显示器。                              |
| Linux Wayland            | `xdg_popup`、reactive positioner 和 reposition，需要 xdg-shell 版本 3。最终位置和缩放由 compositor 决定。无显式 grab 弹层的键盘焦点取决于 compositor，需在目标桌面验证。 |
| Web / 没有原生窗口的宿主 | 创建明确报错，不自动替换为窗口内弹层。浏览器界面使用普通 `Popover`。                                                                                                     |

绝对坐标后端按触发器选择屏幕，不假定主屏或父窗口中心。屏幕工作区约束可翻转、平移
和缩小过大的内容。保留请求尺寸，移到更大工作区时可恢复。负屏幕坐标和混合缩放都由
平台后端处理；不要在 JS 中叠加 `screenX` / `screenY` 或全局缩放系数。

此 API 创建没有显式菜单 grab 的表单弹层。Wayland 的 grab 要求有效的原生输入激活，
异步 JS 请求不能提供该保证，因此不会猜测或复用陈旧 serial。Windows 和 Linux 的键盘、
IME、无障碍以及点击外部应用关闭行为，发布前仍须在真实会话中验收。

## 关闭与生命周期

Escape 在子元素处理按键后关闭最内层弹层。在父窗口内点击子弹层之外会关闭子弹层。
激活离开弹层家族后，已激活的弹层会关闭；激活嵌套弹层则保留祖先。
关闭判断使用系统当前焦点窗口，即使 GPUI 激活通知尚未更新，也不会在焦点切换时误关父弹层。
显式关闭且弹层持有激活时恢复父窗口焦点；转去其他窗口引起的关闭不请求焦点恢复。

父窗口关闭或 epoch 替换时关闭后代。每个弹层获得新的 Surface ID 以及独立事件、修订序列。
取消按原始打开请求定位，即使 ID 尚未返回也可以关闭或卸载，不遗留窗口。
已退役 Surface 的合法迟到提交会被丢弃；迟到的首个 Snapshot 会收到关闭事件，让刚挂载的子根释放资源。
从未分配的 Surface ID 仍会被拒绝，不能用于创建或恢复窗口。
QuickJS 新代校验保留持久窗口，弹层随旧代释放，待新代激活后按受控状态重建。
保留父 context，拒绝已退役 Surface 的回调。

## 原生验收 fixture

```sh
bun run task build
SOLID_GPUI_FIXTURE=fixtures/system-popover.tsx SOLID_GPUI_FIXTURE_OUTPUT=/tmp/system-popover.js bun --bun vite build --config fixtures/vite.config.ts
cargo run -p solid-gpui --features quickjs --bin solid-gpui-host -- --runtime quickjs /tmp/system-popover.js
```

自动原生生命周期 fixture `fixtures/system-popover-lifecycle.tsx` 连续三轮打开嵌套弹层、卸载并重新挂载触发器，
每轮检查主窗口的原生命令仍可响应，成功后自动退出。运行时保持测试应用在前台：

```sh
SOLID_GPUI_FIXTURE=fixtures/system-popover-lifecycle.tsx SOLID_GPUI_FIXTURE_OUTPUT=/tmp/system-popover-lifecycle.js bun --bun vite build --config fixtures/vite.config.ts
cargo run -p solid-gpui --features quickjs --bin solid-gpui-host -- --runtime quickjs /tmp/system-popover-lifecycle.js
```

macOS 首次定位回归会遍历当前连接的显示器：

```sh
cargo run -p solid-gpui --example popup_geometry
```

检查四边打开、父窗口跨屏移动、滚动和移除触发器、反复开关、嵌套 Escape、点击外部应用、
Tab / Shift-Tab、中文输入组合、剪贴板和撤销以及无障碍遍历。使用用户实际的显示器排列、
缩放和桌面会话。确定性测试与跨平台编译不能代替原生输入和 compositor 验收。
证据及待验收项见[实施记录](../.scratch/native-presentation/implementation.md)。

SwiftUI / AppKit 视图嵌入是独立阶段，见[原生组合](native-composition.md)。
