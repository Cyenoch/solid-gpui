import { Pressable, Text, View } from "@solid-gpui/core";
import { LiveFrame, RecordedPaint, type LiveFrameRef, type PaintRecording } from "@solid-gpui/core/components";
import { createMemo, createSignal, onCleanup, onMount } from "@solid-gpui/core/runtime";

/** Each advance has one bounded upload; native unmount owns the final frame release. */
export function StudioPreview(props: { title: string; onFrame?: (frame: LiveFrameRef | undefined) => void }) {
  let frame: LiveFrameRef | undefined;
  let upload: AbortController | undefined;
  let sequence = 0;
  let active = true;
  const [pending, setPending] = createSignal(false);
  const [status, setStatus] = createSignal("Preview ready");
  const [retainedBytes, setRetainedBytes] = createSignal(0);
  const recording = createMemo<PaintRecording>(() => ({
    width: 192,
    height: 64,
    fit: "contain",
    commands: [
      { kind: "quad", bounds: { x: 0, y: 0, width: 192, height: 64 }, color: { kind: "background" } },
      { kind: "quad", bounds: { x: 8, y: 36, width: 176, height: 16 }, color: { kind: "accent" } },
      {
        kind: "path",
        points: [
          { x: 8, y: 54 },
          { x: 48, y: 42 },
          { x: 96, y: 51 },
          { x: 144, y: 39 },
          { x: 184, y: 49 },
        ],
        closed: false,
        stroke: { kind: "foreground" },
        strokeWidth: 2,
        fill: null,
      },
      {
        kind: "text",
        origin: { x: 8, y: 4 },
        text: Array.from(props.title)
          .slice(0, 24)
          .join("")
          .replace(/[\u0000-\u001f\u007f]/g, " "),
        fontSize: 12,
        color: { kind: "foreground" },
      },
    ],
  }));
  const advance = async () => {
    if (!active || !frame || pending()) return;
    setPending(true);
    const controller = new AbortController();
    upload = controller;
    const current = ++sequence;
    const rgba = Array.from({ length: 64 * 36 * 4 }, (_, byte) => {
      const pixel = Math.floor(byte / 4);
      const channel = byte % 4;
      if (channel === 3) return 255;
      return channel === 0 ? ((pixel % 64) * 4 + current * 31) % 256 : channel === 1 ? Math.floor(pixel / 64) * 7 : 130;
    });
    try {
      const value = await frame.replaceFrame(
        { sequence: current, width: 64, height: 36, rgba },
        { signal: controller.signal },
      );
      if (active) {
        setRetainedBytes(value.retainedBytes);
        setStatus(`Preview frame ${value.sequence}`);
      }
    } catch (error) {
      if (active) setStatus(String(error));
    } finally {
      if (upload === controller) upload = undefined;
      if (active) setPending(false);
    }
  };
  onMount(() => {
    void advance();
  });
  onCleanup(() => {
    active = false;
    upload?.abort();
    upload = undefined;
    frame = undefined;
  });
  return (
    <View accessibilityLabel="studio.preview" style={{ gap: 6, minWidth: 0 }}>
      <View accessibilityLabel="studio.preview.recording">
        <RecordedPaint recording={recording()} viewportHeight={64} />
      </View>
      <View accessibilityLabel="studio.preview.frame">
        <LiveFrame
          viewportHeight={108}
          fit="contain"
          ref={(value) => {
            frame = value;
            props.onFrame?.(value);
          }}
        />
      </View>
      <Pressable
        accessibilityLabel="studio.preview.advance"
        accessibilityRole="button"
        focusable
        disabled={pending()}
        onPress={() => {
          void advance();
        }}
        style={{ padding: 7, minHeight: 32, borderWidth: 1, borderColor: "#3C3944", borderRadius: 5 }}
      >
        <Text style={{ color: "#ECEAF1", fontSize: 12 }}>{pending() ? "Loading frame…" : "Advance preview"}</Text>
      </Pressable>
      <Text accessibilityLabel="studio.preview.status" style={{ color: "#B5B1BF", fontSize: 11 }}>
        {status()} · {retainedBytes()} bytes
      </Text>
    </View>
  );
}
