/** Both previews exercise the generated contract in the browser and native hosts. */
export const paintMediaExamples = [
  {
    names: ["RecordedPaint"],
    description:
      "Move a workflow node and zoom a retained native diagram. Theme colors and clipping resolve during native replay.",
    descriptionChinese: "移动工作流节点并缩放原生保留的图表。主题颜色与裁剪在原生重放时解析。",
    source: `import * as N from "@solid-gpui/core/components";
import { View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
export default function Example() {
  const [moved, setMoved] = createSignal(false);
  const [zoom, setZoom] = createSignal(1);
  const [viewport, setViewport] = createSignal("Waiting for native layout");
  const commands = (): N.PaintCommand[] => {
    const y = moved() ? 80 : 30;
    return [
      { kind: "path", points: [{ x: 95, y: 55 }, { x: 180, y: y + 25 }], closed: false, stroke: { kind: "border" }, strokeWidth: 3, fill: null },
      { kind: "quad", bounds: { x: 15, y: 30, width: 80, height: 50 }, color: { kind: "accent" } },
      { kind: "quad", bounds: { x: 180, y, width: 100, height: 50 }, color: { kind: "accent" } },
      { kind: "text", origin: { x: 25, y: 45 }, text: "Input", fontSize: 14, color: { kind: "foreground" } },
      { kind: "text", origin: { x: 190, y: y + 15 }, text: "Transform", fontSize: 14, color: { kind: "foreground" } },
    ];
  };
  return <View style={{ gap: 12 }}>
    <View style={{ flexDirection: "row", gap: 8 }}><N.Button label="Move node" onPress={() => setMoved(!moved())} /><N.Button label="Zoom" onPress={() => setZoom(zoom() === 1 ? 1.25 : 1)} /></View>
    <N.RecordedPaint viewportHeight={180} recording={{ width: 320, height: 180, commands: commands(), transform: { scale: zoom(), translateX: 0, translateY: 0 }, clip: { x: 0, y: 0, width: 320, height: 180 } }} onViewport={value => setViewport(Math.round(value.width) + " × " + Math.round(value.height) + " · scale " + value.scaleFactor)} />
    <N.Label text={viewport()} />
  </View>;
}`,
  },
  {
    names: ["LiveFrame"],
    description:
      "Run a bounded CPU test-pattern stream. Each upload is acknowledged before the next frame; clear releases pixels and unmount disposes the owner.",
    descriptionChinese: "运行有界 CPU 测试图案流。每次上传确认后才生成下一帧；清除会释放像素，卸载会释放所有者。",
    source: `import * as N from "@solid-gpui/core/components";
import { View } from "@solid-gpui/core";
import { createSignal, onCleanup } from "@solid-gpui/core/runtime";
export default function Example() {
  let frame: N.LiveFrameRef | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let sequence = 0;
  let active = false;
  let inFlight = false;
  const [running, setRunning] = createSignal(false);
  const [status, setStatus] = createSignal("Ready");
  const tick = async () => {
    if (!active || inFlight || !frame) return;
    inFlight = true;
    const owner = frame;
    const id = ++sequence;
    const rgba = Array.from({ length: 64 * 36 * 4 }, (_, index) => {
      const pixel = Math.floor(index / 4);
      const channel = index % 4;
      if (channel === 3) return 255;
      return channel === 0 ? (pixel % 64 * 4 + id * 8) % 256 : channel === 1 ? Math.floor(pixel / 64) * 7 : 120;
    });
    try {
      const state = await owner.replaceFrame({ sequence: id, width: 64, height: 36, rgba });
      if (active) setStatus("Frame " + state.sequence + " · " + state.retainedBytes + " retained bytes");
    } catch (error) { active = false; setRunning(false); setStatus(String(error)); }
    finally { inFlight = false; }
    if (active) timer = setTimeout(() => void tick(), 100);
  };
  const stop = () => { active = false; setRunning(false); clearTimeout(timer); };
  const clear = async () => {
    stop();
    try { const state = await frame?.clear(); setStatus("Cleared · " + (state?.retainedBytes ?? 0) + " retained bytes"); }
    catch (error) { setStatus(String(error)); }
  };
  onCleanup(stop);
  return <View style={{ gap: 12 }}>
    <N.LiveFrame ref={value => frame = value} viewportHeight={180} fit="contain" />
    <View style={{ flexDirection: "row", gap: 8 }}><N.Button label={running() ? "Pause" : "Run frames"} onPress={() => { if (active) stop(); else { active = true; setRunning(true); void tick(); } }} /><N.Button label="Clear" onPress={() => void clear()} /></View>
    <N.Label text={status()} />
  </View>;
}`,
  },
];
