import {
  StdioTransport,
  Text,
  View,
  createAppearanceStore,
  createProcessTerminationHandler,
  createSurfaceHost,
  createWindowSizeStore,
  useAppearance,
  useWindowSize,
  type AppearanceStore,
  type WindowSizeStore,
} from "../src/index";

interface SurfacePanelProps {
  readonly name: string;
  readonly sizeStore: WindowSizeStore;
  readonly appearanceStore: AppearanceStore;
}

function SurfacePanel({ name, sizeStore, appearanceStore }: SurfacePanelProps) {
  const { width, height, scaleFactor } = useWindowSize(sizeStore);
  const appearance = useAppearance(appearanceStore);
  const dark = appearance === "dark";
  const backgroundColor = dark ? "#111827" : "#f7f8fa";
  const textColor = dark ? "#f9fafb" : "#172033";
  const mutedColor = dark ? "#cbd5e1" : "#5b6b7f";

  return (
    <View
      style={{ flexDirection: "column", gap: 12, width, height, padding: 20, backgroundColor }}
      accessibilityRole="generic"
      accessibilityLabel={`${name} surface`}
    >
      <Text style={{ fontSize: 18, lineHeight: 24, fontWeight: "bold", color: textColor }}>{name}</Text>
      <Text style={{ fontSize: 13, lineHeight: 18, color: mutedColor }}>
        {width}×{height} logical pixels at {scaleFactor}x
      </Text>
      <Text style={{ fontSize: 13, lineHeight: 18, color: mutedColor }}>System appearance: {appearance}</Text>
      <Text style={{ fontSize: 12, lineHeight: 18, color: mutedColor }}>
        Each root owns an explicit size and appearance bridge; changing one window does not use a module-global
        singleton.
      </Text>
    </View>
  );
}

const host = createSurfaceHost(new StdioTransport(), {
  onTransportTermination: createProcessTerminationHandler(),
});
const mainSizeStore = createWindowSizeStore({ width: 800, height: 600 });
const mainAppearanceStore = createAppearanceStore();
const mainRoot = host.createRoot({
  surfaceId: 1,
  onWindowResize: (width, height, scaleFactor) => mainSizeStore.set(width, height, scaleFactor),
  onAppearance: (appearance) => mainAppearanceStore.set(appearance),
});
mainRoot.render(<SurfacePanel name="Main surface" sizeStore={mainSizeStore} appearanceStore={mainAppearanceStore} />);

void mainRoot
  .openSurface({
    title: "Inspector",
    width: 480,
    height: 320,
    kind: "floating",
    resizable: true,
    minSize: [320, 240],
  })
  .then((surfaceId) => {
    const inspectorSizeStore = createWindowSizeStore({ width: 480, height: 320 });
    const inspectorAppearanceStore = createAppearanceStore();
    const inspectorRoot = host.createRoot({
      surfaceId,
      onClose: () => console.log("Inspector closed"),
      onWindowResize: (width, height, scaleFactor) => inspectorSizeStore.set(width, height, scaleFactor),
      onAppearance: (appearance) => inspectorAppearanceStore.set(appearance),
    });
    inspectorRoot.render(
      <SurfacePanel
        name="Inspector surface"
        sizeStore={inspectorSizeStore}
        appearanceStore={inspectorAppearanceStore}
      />,
    );
  })
  .catch((error: unknown) => {
    console.error("Unable to open inspector surface", error);
  });
