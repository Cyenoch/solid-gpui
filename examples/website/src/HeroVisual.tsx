import { View, type LayoutFrame, type SolidChild } from "@solid-gpui/core";
import { onCleanup } from "@solid-gpui/core/runtime";
import { Button } from "./ui";

/** A native foreground over a separate, non-interactive GPU artwork layer. */
export function HeroVisual(props: { width: number; height: number; children: SolidChild }) {
  let frame: LayoutFrame | undefined;
  let background: HTMLDivElement | undefined;
  function remove() {
    background?.remove();
    background = undefined;
  }
  function synchronize() {
    if (!frame || document.hidden || frame.y + frame.height <= 65 || frame.y >= innerHeight) {
      remove();
      return;
    }
    if (!background) {
      background = document.createElement("div");
      background.className = "hero-background";
      const iframe = document.createElement("iframe");
      iframe.src = "https://vgpu.sh/preview/optimized-black-hole";
      iframe.title = "Optimized Black Hole by vgpu";
      iframe.allow = "autoplay";
      iframe.tabIndex = -1;
      iframe.setAttribute("aria-hidden", "true");
      background.append(iframe);
      document.body.append(background);
    }
    Object.assign(background.style, {
      left: `${frame.x}px`,
      top: `${frame.y}px`,
      width: `${frame.width}px`,
      height: `${frame.height}px`,
      clipPath: `inset(${Math.max(0, 65 - frame.y)}px 0 ${Math.max(0, frame.y + frame.height - innerHeight)}px 0)`,
    });
  }
  document.addEventListener("visibilitychange", synchronize);
  onCleanup(() => {
    remove();
    document.removeEventListener("visibilitychange", synchronize);
  });
  return (
    <View
      onLayout={(value) => {
        frame = value;
        synchronize();
      }}
      style={{
        width: props.width,
        minHeight: props.height,
        alignSelf: "center",
        flexShrink: 0,
        alignItems: "center",
        justifyContent: "center",
        padding: props.width < 600 ? 24 : 60,
        gap: 20,
      }}
    >
      {props.children}
      <View
        style={{ flexDirection: props.width < 400 ? "column" : "row", alignItems: "center", gap: 8, marginTop: 12 }}
      >
        <Button
          ghost
          icon="lucide:external-link"
          onPress={() => window.open("https://vgpu.sh/examples/optimized-black-hole", "_blank", "noopener")}
        >
          Artwork by vgpu
        </Button>
      </View>
    </View>
  );
}
