import { createRoot, Pressable, Text, View } from "../../packages/solid-gpui/src/index";
import { NativeAcceptance } from "../../packages/solid-gpui/src/testing";
const binary = "/private/var/folders/sl/42r5kc756mj96rlxkglrpr900000gn/T/opencode/solid-gpui-adoption-target/debug/solid-gpui-acceptance";
const host = await NativeAcceptance.launch({ command: [binary], mode: "gpu" });
const root = createRoot(host.transport, { surfaceId: 1 });
let presses = 0;
try {
  root.render(() => View({
    style: { backgroundColor: "#101827", paddingX: 24, paddingY: 16, gap: 12, color: "#ffffff" },
    children: [
      Text({ children: "Native style contract", style: { fontSize: 24 } }),
      Pressable({
        accessibilityLabel: "Save changes", focusable: true, onPress: () => presses++,
        style: {
          width: { unit: "rem", value: 14 }, paddingX: 16, paddingY: 8, borderWidth: 2,
          borderColor: "#00000000", backgroundColor: "#26344a",
          hover: { backgroundColor: "#344866" }, active: { backgroundColor: "#182235" },
          focusVisible: { borderColor: "#82b4ff" },
        },
        children: Text({ children: "Save changes" }),
      }),
      View({ style: { width: 280, height: 120, overflowX: "hidden", overflowY: "scroll" },
        children: View({ style: { width: { unit: "percent", value: 100 }, height: 320, backgroundColor: "#25344a" },
          children: Text({ children: "Scroll this vertical pane" }),
        }),
      }),
    ],
  }));
  await host.key("tab");
  await host.flush();
  const target = await host.locate({ label: "Save changes" });
  if (target.bounds.width <= 0 || target.bounds.height <= 0) throw new Error("Button is not painted");
  const screenshot = await host.screenshot();
  await Bun.write(new URL("style-native.png", import.meta.url), screenshot.png);
  await host.click(target);
  if (presses !== 1) throw new Error(`Native hit test failed: ${presses}`);
  console.log(JSON.stringify({ screenshot: [screenshot.width, screenshot.height], button: target.bounds, presses }));
} finally {
  root.unmount();
  console.log(await host.close());
}
