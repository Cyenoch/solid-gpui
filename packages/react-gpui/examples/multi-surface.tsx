import { useMemo } from "react";
import {
  StdioTransport,
  StyleSheet,
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
import { useTheme, type Theme } from "./theme";

interface SurfacePanelProps {
  readonly name: string;
  readonly sizeStore: WindowSizeStore;
  readonly appearanceStore: AppearanceStore;
}

function createStyles(theme: Theme) {
  return StyleSheet.create({
    root: { flexDirection: "column", gap: 12, padding: 20, backgroundColor: theme.canvas },
    title: { fontSize: 18, lineHeight: 24, fontWeight: "bold", color: theme.text },
    detail: { fontSize: 13, lineHeight: 18, color: theme.textMuted },
    helper: { fontSize: 12, lineHeight: 18, color: theme.textMuted },
  });
}

function SurfacePanel({ name, sizeStore, appearanceStore }: SurfacePanelProps) {
  const { width, height, scaleFactor } = useWindowSize(sizeStore);
  const appearance = useAppearance(appearanceStore);
  const theme = useTheme(appearance);
  const styles = useMemo(() => createStyles(theme), [theme]);

  return (
    <View style={{ ...styles.root, width, height }} accessibilityRole="generic" accessibilityLabel={`${name} surface`}>
      <Text style={styles.title}>{name}</Text>
      <Text style={styles.detail}>
        {width}×{height} logical pixels at {scaleFactor}x
      </Text>
      <Text style={styles.detail}>System appearance: {appearance}</Text>
      <Text style={styles.helper}>
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
