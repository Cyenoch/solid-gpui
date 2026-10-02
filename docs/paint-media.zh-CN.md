# 保留绘制与实时帧

`RecordedPaint` 与 `LiveFrame` 是从 `@solid-gpui/core/components` 导出的生成式
NativeView 组件。组件 host 与网站 WASM host 通过
`solid_gpui::components::native_module()` 注册；自定义 host 必须注册同一模块并
生成匹配绑定。它们沿用现有 Extension 与原生命令协议。

## 保留绘制

Solid 响应式构造 `PaintRecording`，原生代码在树发布前验证完整记录，在记录变化时
准备路径几何，并在 GPUI paint 中重放。原生 paint 不调用 JavaScript。网站
**RecordedPaint** 示例可移动工作流节点、缩放图表并显示原生视口反馈。

坐标采用逻辑绘制单位，必须有限且绝对值不超过 16,384；记录宽高及视口高度范围为
1..16,384。组件宽度填充父容器，`viewportHeight` 默认 240。外层 `View` 负责
边框、背景、输入与宽度约束。`onViewport` 仅在实际逻辑宽高或设备 scale 变化时
报告布局信息，不是 paint 回调。

默认 `fit: "contain"` 居中并缩小记录以适应视口，不放大；`fit: "none"` 居中并保持
1:1 逻辑大小。`transform` 提供 0.01..16 的正统一缩放与 `translateX`/`translateY`。
缩放影响几何、线宽及文本，平移单位乘以适应缩放。旋转、倾斜及非统一文本变换不受
支持。GPUI 仅在构建 scene 时应用设备缩放一次，无需 JS 重绘。

绘制始终受视口和祖先裁剪约束。可选 `clip` 是绘制坐标中的矩形，也随变换缩放。
`foreground`、`background`、`accent`、`border` 在每次原生重放时解析当前主题；
`solid` 使用经过验证的 `#RRGGBB` 或 `#RRGGBBAA`。

路径为直线折线，描边由独立平端线段组成；闭合、严格凸多边形支持填充，顺逆时针
均可。凹多边形、自交填充不被接受；契约未提供 Bézier 曲线。文本使用原生单行
shaping，继承原生字体族，origin 为左上角；fontSize 范围 1..256，显式 transform
scale × fontSize 也必须不超过 256。不提供换行、控制字符、浏览器文本测量或编辑。

每个记录最多 1,024 个命令、合计 4,096 个路径点、32,768 个生成三角形顶点、每条
文本 2,048 UTF-8 字节及合计 16,384 文本字节。现有原生 JSON/commit 字节预算也
适用。路径准入与准备复杂度随输入点线性增长，重放受命令、顶点与文本预算约束。
几何仅在记录变化时重建；视口、裁剪、DPI、字体与主题在原生重放中读取，避免旧
scene 缓存。卸载释放记录。

## CPU 帧所有权

`LiveFrame` 的挂载原生组件实例拥有资源。生成的 ref 是带 surface、epoch、node、
module 与 catalog 身份的能力，不暴露全局图像 ID 或原生地址。新挂载/epoch 创建
新所有者；关闭、移除或替换组件释放可见帧与暂存帧，并在最后一个 scene 所有者释放
时清理 GPUI 图像 atlas。

```tsx
import { LiveFrame, type LiveFrameRef } from "@solid-gpui/core/components";
let frame: LiveFrameRef | undefined;
<LiveFrame ref={(value) => (frame = value)} viewportHeight={180} fit="contain" />;
await frame?.replaceFrame({ sequence: 1, width: 2, height: 1, rgba: [255, 0, 0, 255, 0, 255, 0, 255] });
await frame?.clear();
```

像素为紧密排列、由上至下、直 alpha 的 sRGB RGBA8，字节数严格等于宽 × 高 × 4。
原生适配器在呈现时转为 GPUI BGRA。生成式 JSON 接受 `number[]`，原生逐字节验证
u8，不编码图像或 data URL。`replaceFrame` 最多 240 KiB，即使每字节十进制为三位
也能满足 1 MiB 原生 JSON 上限。

较大帧通过 `beginFrame({sequence,width,height})` 预留暂存帧，然后顺序等待
`writeFrameChunk({sequence,offset,rgba})`。每块 1..240 KiB，offset 必须连续，
总量不能超过预留。完整上传后调用 `presentFrame(sequence)` 原子替换可见帧。
暂存不完整时不绘制；重复 begin 要求先取消。无效请求保留当前状态。

`cancelFrame()` 只释放暂存。`clear()` 释放可见与暂存像素但保留最后 sequence，
拒绝延迟旧帧。`dispose()` 永久关闭当前所有者，可重复调用；再次上传被拒绝，必须
重挂载。`getState()` 返回 disposed、最后 sequence、当前尺寸与保留/预留 CPU 字节。
成功 `replaceFrame` 会取消旧暂存。

单边尺寸 1..4,096，每帧最多 16 MiB。每组件仅一个可见帧、一个暂存帧，整个 host
共 64 MiB 预算，包括暂存预留以及等待 scene 释放的像素所有者。预算不足拒绝获取，
不会删除可见内容。替换期间可能同时占用新旧帧。GPUI 在 update 结束时释放实体及
atlas；clear/dispose 不保证 GPU 驱动立即回收内存。

原生不排队帧。生产者必须等待确认并限制自身待处理任务。网站 **LiveFrame** 示例
始终仅一个上传，提供暂停、清除、错误显示，并在卸载时清理定时器。`fit` 提供
contain、cover、fill，受视口与祖先裁剪约束。CPU 像素颜色不随主题变化。

begin 成功即保留 sequence，即使之后取消也不能重用。新的 beginFrame/replaceFrame
必须大于当前所有者所有曾准入的 sequence；FrameState.sequence 表示最后呈现帧。
这可避免旧上传块进入重启上传，clear 也保留此高水位。

## 平台与限制

CPU 资源与 GPUI 图像路径由 macOS、Linux、Windows、WASM 共用。本地原生像素
验收在 macOS；其他平台仍需各自的窗口/像素验收，网站示例使用同一生成契约。
上传与呈现有固定预算，但包含复制与 RGBA 转换，不声明高分辨率视频解码或零复制。
应用可输入自己解码的帧。

不提供 IOSurface、共享 GPU 句柄、摄像头、编解码器、音频或硬件解码导入。
后续原生加速需要明确平台契约、所有权验证及验收；指针字节不是可移植媒体句柄。

原生改动后运行 `bun scripts/native-codegen.ts` 与
`bun scripts/native-codegen.ts --check`，网站检查为
`bun --conditions=browser test examples/website/tests`。聚焦验收与限制记录在
[ticket 07 delivery](../.scratch/comparison-adoption/delivery-07.md)。
