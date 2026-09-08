# 快捷键与原生菜单

通过 `root.setKeybindings(...)` 注册应用命令，在根节点的 `onAction` 中按名称处理。GPUI 先解析修饰键、键盘布局和多段快捷键，再将未消费的原生按键事件交给焦点控件。控件局部交互使用 `onKeyDown`。

```ts
const root = createRoot(transport, {
  onAction(action) {
    if (action === "Open") openDocument();
  },
});

await root.setKeybindings([
  { keystrokes: "secondary-o", actionName: "Open" },
  { keystrokes: "secondary-k secondary-p", actionName: "Command Palette" },
]);
```

示例假定应用已经提供 transport 和命令处理函数。与其他 surface 命令一样，在根节点渲染后注册快捷键。

| GPUI 语法 | 含义 |
| --- | --- |
| `secondary-o` | macOS 上为 Command+O；Windows 和 Linux 上为 Control+O |
| `ctrl-o` | 所有平台均为 Control+O |
| `cmd-o`、`super-o`、`win-o` | 平台修饰键；在 Windows 或 Linux 上不代表 Control |
| `secondary-shift-p` | 主快捷键修饰键加 Shift+P |
| `secondary-k secondary-p` | 连续两次按键，以空格分隔 |

键名不区分大小写。大写单字母会添加 Shift，因此建议使用小写字母并显式写出修饰键。语法和布局匹配来自已链接的 `gpui-pre 0.3.3` 的 `platform/keystroke.rs` 与 `keymap/binding.rs`。

每次调用替换该 surface 的完整绑定集合，`[]` 清空绑定。解析失败时保留原集合。宿主保存编译后的绑定，只安装当前活动窗口的集合，并保留宿主配置的基础绑定。不同窗口可以将同一快捷键映射到不同命令。没有控件焦点或 provider 覆盖层获得焦点时，快捷键仍可用；原生控件保留自身动作分派行为。关闭 surface 或接受新的协议 epoch 会移除旧绑定，应在新 epoch 的应用初始化中重新注册。

## 应用菜单栏

`root.setMenus(...)` 替换应用共享菜单定义。在 macOS 上由 GPUI 提供系统菜单栏；gpui-component 宿主同时将定义发布给 `<AppMenuBar />`，可放在 Windows 或 Linux 的窗口装饰区。

```ts
await root.setMenus([
  {
    title: "File",
    items: [
      { type: "action", name: "Open" },
      { type: "separator" },
      { type: "action", name: "Save", disabled: true },
    ],
  },
]);
```

动作的 `name` 同时作为显示文字和 `onAction` 值。菜单动作发往活动 surface，因此使用共享菜单的窗口都应处理对应命令。共享定义由应用中的单一 owner 管理；`setMenus` 不会创建独立窗口菜单。根据应用状态更新 `checked` 和 `disabled`。需要键盘快捷方式时，注册同名快捷键动作。禁用菜单项不会移除独立注册的快捷键；命令不可用时也要更新绑定集合。

## 上下文菜单

生成的 `<NativeMenu />` 支持右键、左键按下及手动触发。`show({ x, y })` 使用相对原生窗口的逻辑坐标。macOS 和 Windows 显示系统弹出菜单，Linux 使用限制在窗口内的 gpui-component 覆盖层。

菜单项 ID 必须在整个子菜单树中唯一。宿主最多允许 1,024 项和 16 层子菜单。选择回调要求组件仍已挂载且可用，菜单项及祖先均启用，而且属性代次与创建弹窗时一致。更新属性会撤销已打开菜单的动作快照；重新启用或复用 ID 不会恢复该快照。卸载组件会撤销路由。这些检查同样约束延迟到达的系统选择回调。

## 验证

宿主命令往返测试覆盖原生快捷键、平台主修饰键、原子替换失败、多窗口同键不同命令、焦点文本输入、清空绑定和 epoch 退役。原生菜单测试覆盖选择、禁用祖先、过期快照及卸载。确定性测试不能替代各发布平台上系统菜单栏、键盘布局、输入法和弹窗位置的实机检查。
