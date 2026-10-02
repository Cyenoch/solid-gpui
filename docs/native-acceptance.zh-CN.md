# 原生验收测试

每次验收绘制都会执行 GPUI 排队的下一帧回调。窗口 resize 反馈来自原生 bounds
变化后的生产 viewport observer，不会根据提交的树合成。
Surface 和节点命令会等待当前同步 Solid batch 完成，并在确定 `afterRevision`
前提交该 batch 的 host 变更。
GPU action 还会在确认排队的 AppKit 窗口操作之前驱动 macOS 主 run loop。

保留原生基础区域重放同一绘制场景代次的观察结果。缓存 miss 记录新的文字和裁剪
几何，hit 恢复已经绘制的事实，不把当前 store 文字作为绘制证据。移除区域后下一帧
移除观察数据。因此无关提交后定位器和几何仍可用，内容检查仍能发现区域更新停滞。

`@solid-gpui/core/testing` 的 `NativeAcceptance` 让 Bun 测试拥有一个显式启动的
原生 host。应用仍使用普通 `createRoot`、Snapshot/Patch 解码器、生产
`NativeStateRegistry`、`SolidRoot`、GPUI 布局/绘制和原生输入处理器。
Solid 拥有应用状态和回调代次；GPUI 拥有编辑、命中测试、选区、滚动和绘制。

`TestHost` 仍是基于 `MemoryTransport` 的语义协议辅助工具，不提供原生几何。
断言依赖布局、绘制或原生输入路由时，使用原生验收接口。

## 构建与启动

从匹配的原生源码构建可选执行文件：

```sh
CARGO_BUILD_JOBS=2 cargo build -p solid-gpui --features native-acceptance --bin solid-gpui-acceptance
```

此 feature 包含 GPUI 测试支持。普通生产 host 不暴露自动化服务；验收执行文件
必须显式接收 `--native-acceptance deterministic` 或 `--native-acceptance gpu`。
JS 控制器自动添加参数，重定向 stdin 不会自动开启功能。请求通过独立的、长度
前缀字节通道传递，内部提交/事件保留规范 renderer framing。没有 TCP 监听器、
原生指针或 JS paint 回调。

```ts
import { createRoot, Pressable, Text } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { NativeAcceptance } from "@solid-gpui/core/testing";

const host = await NativeAcceptance.launch({
  command: ["./target/debug/solid-gpui-acceptance"],
  mode: "deterministic",
});
const root = createRoot(host.transport, { surfaceId: 1 });
try {
  root.render(() => {
    const [count, setCount] = createSignal(0);
    return Pressable({
      accessibilityLabel: "Increment",
      style: { width: 140, height: 40 },
      onPress: () => setCount(count() + 1),
      get children() {
        return Text({ children: String(count()) });
      },
    });
  });
  await host.click(await host.locate({ label: "Increment" }));
  const text = await host.locate({ text: "1" });
  if (text.bounds.width <= 0) throw new Error("Updated text did not paint");
} finally {
  root.unmount();
  await host.close();
}
```

每个原生进程先打开 Surface 1，创建 root 时显式指定 ID。TSX 测试通过公开
`solid-gpui test` runner 运行；本仓库源码测试使用 Bun 的 `browser` 和
`solid-gpui-source` conditions。不要与其他 checkout 共享可写的 package dist。

发出应用 root 命令前需提交首屏；异步 router 应先完成首个路由加载，再 flush。
等待中的路由尚未建立原生树，提前 root 命令会明确拒绝。命令回复（包括拒绝）
保留请求的 Surface 与 epoch，旧请求不会结算新 epoch 的同编号请求。
`resize(width, height, surfaceId = 1)` 发送原生尺寸事件与生产 bounds 反馈：
deterministic 使用 GPUI 平台 resize 事件，GPU 调整真实窗口。

## 公开接口

| 接口                                                           | 行为                                                                                                                                                |
| -------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `NativeAcceptance.launch({ command, mode, cwd?, timeoutMs? })` | 拥有 Bun 子进程，显式选 mode 与 v2 验收契约；启动、请求和清理默认超时 30 秒；启动失败会 reject。                                                                   |
| `transport`                                                    | 传给 `createRoot(host.transport, { surfaceId: 1 })`；使用真实原生 admission/event 路径，其他 Surface 使用普通 host 命令。                           |
| `capabilities`                                                 | 验收版本、mode、平台、截图和时钟支持。                                                                                                              |
| `flush()`                                                      | 提交、绘制、向 Solid 分发事件并绘制受控确认；32 个反馈轮次仍不稳定则报错。                                                                          |
| `snapshot(surfaceId = 1)`                                      | 返回隔离的已绘制节点、Surface/epoch/revision；节点有稳定 ID、父 ID、数字 native kind、listener ID、label、可观察原生文本和裁剪后的逻辑像素 bounds。 |
| `locate({ id?, label?, text? }, surfaceId = 1)`                | 精确匹配且必须只有一个已绘制节点；label 来自已有 `accessibilityLabel`。                                                                             |
| `click(target)`                                                | 在可见中心移动、左键按下/抬起；GPUI 真正执行命中测试，覆盖层、disabled 和传播规则仍有效。                                                           |
| `type(text, surfaceId = 1)`                                    | 向原生焦点发送 Unicode 按键；先点击编辑器，每次最多 512 个 Unicode 标量值。                                                                         |
| `key(keystroke, surfaceId = 1)`                                | 一个 GPUI keystroke，例如 `secondary-a`、`backspace`、`enter`。                                                                                     |
| `drag(target, { x, y }, { from? }?)`                           | 从中心或 bounds 内显式 `from` 左键按下，8 次原生移动后抬起；坐标为该 Surface 的逻辑像素。                                                           |
| `wheel(target, { x, y })`                                      | 中心位置的原生像素滚动量，负 `y` 使内容向上移动。                                                                                                   |
| `advanceClock(milliseconds)`                                   | 推进拥有的 GPUI 测试时钟并绘制，每次最多 60 秒；不控制 Bun 定时器和真实网络/OS 服务。                                                               |
| `screenshot(surfaceId = 1)`                                    | 返回原生场景的 `{ width, height, png: Uint8Array }`，尺寸为物理像素；需要支持的 GPU mode。                                                          |
| `clipboardText()`                                              | 原生 copy 后读取拥有的 GPUI 测试 clipboard，返回文本或 `null`；两种模式都与用户桌面 clipboard 隔离。                                                |
| `close()`                                                      | 幂等清理：绘制空帧释放输入/帧引用，移除所有窗口和 root，关闭通道并等待退出；返回 `{ surfaces: 0, windows: 0, popups: 0 }`，非零退出报错。           |

target 绑定生成它的会话以及 Surface/epoch/revision/listener。每次应用提交后
重新 locate；过期 revision、替换 epoch、移除或未绘制节点会报错，不会改为调用
较新的 handler。节点挂载期间 ID 稳定。bounds 是原生内容 mask 裁剪的布局框，
不是字形轮廓或遮挡图；即使被覆盖，输入仍通过 GPUI 命中测试。

文本来自已绘制 Text/RawText 和核心 TextInput 当前显示值；空输入显示 placeholder。
Extension 内部 widget 树保持不透明：用外层 label/ID 定位，断言公开回调/状态或截图。
嵌套富文本在父 Text 中组装，不一定有独立绘制框。这是绘制时观察，不是 OCR
或完整无障碍树导出。

核心 TextInput 的 `input` 包含 `value`、UTF-16 `selectionStart`/`selectionEnd`、
`reversed`、`editSeq`、`focused` 和可空 `markedStart`/`markedEnd`。
`scrollOffset` 是原生 VirtualList 的正偏移，其他元素为 `null`；普通 overflow
通过绘制位置变化验证。`selectedText` 返回已有逐节点 selectable Text 选区。
全 renderer selection/search 仍由消费应用的原生模块公开 contract 所有；用
原生 drag/key 驱动，通过模块公开结果或 `clipboardText()` 检查，不替代应用回调。

队列最多 256 个提交/4 MiB 提交字节，4096 个事件/4 MiB framed event 字节，
每个 JSON packet 最多 16 MiB。截图最多 400 万物理像素、2 MiB PNG；超限报错。
大更新之间调用 flush。JS 串行化动作、处理反馈；Rust 在拥有的原生前台线程绘制。
观察工作为 O(已绘制 Host Node)，仅在可选 Cargo feature 下编译。

## 自定义生产 host

自定义执行文件可传入与应用相同的 `HostProfile`，包含生成的 NativeModule 和 catalog：

```rust
fn main() {
    if let Err(error) = solid_gpui::host::acceptance::run(my_application_profile()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
```

启用 `solid-gpui/native-acceptance`。复用生产初始化、registry、window root、
命令 admission 和原生 handler。stock 执行文件使用 `NoExtensions`，会拒绝应用
extension contract。GPU 窗口通过原生 WindowOptions 放到屏幕外。

## 平台和证据

| 模式            | 绘制与输入                                                                         | 截图                                                    | 验证状态                                                       |
| --------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------- | -------------------------------------------------------------- |
| `deterministic` | GPUI TestAppContext 平台，生产原生布局、scene 和命中分发；文本/OS 服务使用测试平台 | 显式不支持                                              | macOS 自动化原生正确性已执行；其他桌面平台需各自构建/运行。    |
| macOS `gpu`     | VisualTestAppContext 原生文本/渲染和生产 tree/input，屏幕外 Metal 窗口             | 从原生 scene 读取 Metal texture 的 PNG；需桌面/GPU 会话 | macOS 已执行变化像素截图；无需屏幕录制权限，也不截取可见桌面。 |
| 其他平台 `gpu`  | 显式启动错误：链接的 GPUI visual context 仅支持 macOS                              | 不支持                                                  | 不从其他 GPUI fork 推断 Windows/Linux 支持。                   |
| 物理前台输入    | 普通应用窗口，由人操作鼠标、触控板和 IME                                           | 独立平台工具                                            | 手动验证；验收不会注入 OS 硬件事件。                           |

原生验收不证明物理显示延迟、FPS、触控板、IME composition 或 compositor blur。
分别报告确定性 native 测试和 Metal 截图；GPUI 测试 HTTP/OS 服务也不证明生产
网络/服务集成。

`root.unmount()` 释放 JS owner/event subscription；仍必须在 finally 中
`await host.close()` 清理原生窗口、实体和进程。在运行中的会话，通过条件内容提交
测试删除；同一 Surface 重挂载使用较新 epoch。因缺少 binary/GPU opt-in 而 skip
不构成支持证据。

```sh
SOLID_GPUI_ACCEPTANCE_BINARY="$PWD/target/debug/solid-gpui-acceptance" \
bun --conditions=browser --conditions=solid-gpui-source test packages/solid-gpui/tests/native-acceptance.test.ts

SOLID_GPUI_ACCEPTANCE_BINARY="$PWD/target/debug/solid-gpui-acceptance" \
SOLID_GPUI_ACCEPTANCE_GPU=1 \
bun --conditions=browser --conditions=solid-gpui-source test packages/solid-gpui/tests/native-acceptance.test.ts
```

GPU lane 在有桌面会话的 macOS 执行。物理输入和性能验证参阅
[性能分析](performance-analysis.zh-CN.md) 和实际消费应用的验收 workload。
