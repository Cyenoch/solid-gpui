# gpui-performance

可复用的 GPUI 原生性能组件，无 SolidJS 或 gpui-component 依赖。
半透明浮层显示实时 FPS 和最近15秒曲线，可放左上或右上角。

## 接入

与 APP 使用同一个 GPUI 版本。本 workspace 通过路径依赖使用：

```toml
gpui-performance = { path = "../gpui-performance" }
```

在窗口初始化时创建一次，在窗口大小的相对定位容器中挂载实体：

```rust,ignore
use gpui_performance::{MonitorCorner, PerformanceMonitor};

// cx: &mut App
let monitor = cx.new(|cx| {
    PerformanceMonitor::new(MonitorCorner::TopRight, cx)
});
// 把 Entity 存在你的窗口 view 中；Render 中组合：
div().relative().size_full()
    .child(content.clone())
    .child(monitor.clone())
```

`MonitorCorner::TopLeft` 改为左上角。组件不创建采样 Task；
不要在每次 Render 中重新创建实体。无点击监听，不改变内容布局。
右侧预留56px以让出常见窗口工具按钮。

solid-gpui 的 gpui-component host 默认挂载该组件。
使用 `SOLID_GPUI_PERF_MONITOR=0` 启动该 host 可关闭监视器做性能对照。
该环境变量由 host 解释，通用组件本身不读取环境变量。

## 实时指标口径

- 被动随原生绘制采样；活跃时每累计至少250ms更新读数和曲线，不主动请求重绘。
- FPS = 连续绘制间隔数 / 实际经过时间，是 draw-to-draw cadence，不是 CPU draw 时长倒数。
- 两次绘制间隔达到500ms时开启新活跃段；空闲不写低值、不连接跨段曲线，画面保留最近读数。
  这是按需绘制的活动判定阈值，不能区分500ms以上的主线程阻塞和真正空闲；严重卡顿需结合
  外部调用栈、原生 draw duration 与输入延迟。250ms慢帧仍计入活跃FPS，不过滤低FPS样本。
- 曲线保留最近15秒的活跃样本，以真实时间定位，纵轴从120 FPS开始、按60递增扩展。
- 这是窗口 **draw cadence**，不是GPU完成或显示器scanout。浮层初次挂载/恢复活动尚未
  积累足够样本时显示 `FPS —`；静止画面的旧读数不是实时刷新率承诺。
- 将实体直接挂到每次绘制都会经过的窗口overlay，不放进可跳过render的缓存子树。

[性能分析指南](../../docs/performance-analysis.md) 说明如何控制监视器的观察开销。
