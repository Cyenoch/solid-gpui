import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Pressable,
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
  type Root,
  type Style,
  type WindowSizeStore,
} from "../src/index";
import { useTheme, type Theme } from "./theme";

interface SurfacePanelProps {
  readonly name: string;
  readonly sizeStore: WindowSizeStore;
  readonly appearanceStore: AppearanceStore;
  readonly root?: Root;
}

interface WindowControlStyles {
  readonly controls: Style;
  readonly controlsTitle: Style;
  readonly controlsRow: Style;
  readonly control: Style;
  readonly controlLabel: Style;
  readonly detail: Style;
  readonly helper: Style;
}

function createStyles(theme: Theme) {
  return StyleSheet.create({
    root: { flexDirection: "column", gap: 12, padding: 20, backgroundColor: theme.canvas },
    title: { fontSize: 18, lineHeight: 24, fontWeight: "bold", color: theme.text },
    detail: { fontSize: 13, lineHeight: 18, color: theme.textMuted },
    helper: { fontSize: 12, lineHeight: 18, color: theme.textMuted },
    controls: {
      flexDirection: "column",
      gap: 8,
      padding: 12,
      borderWidth: 1,
      borderRadius: 8,
      borderColor: theme.border,
      backgroundColor: theme.surface,
    },
    controlsTitle: { fontSize: 14, lineHeight: 18, fontWeight: "semibold", color: theme.text },
    controlsRow: { flexDirection: "row", gap: 12 },
    control: { padding: 6, borderRadius: 6, backgroundColor: theme.accentSoft },
    controlLabel: { fontSize: 12, lineHeight: 16, color: theme.accentText },
  });
}

interface WindowControlsProps {
  readonly root: Root;
  readonly styles: WindowControlStyles;
}

function WindowControls({ root, styles }: WindowControlsProps) {
  const [bounds, setBounds] = useState<{ x: number; y: number; width: number; height: number } | null>(null);
  const [state, setState] = useState<{ fullscreen: boolean; maximized: boolean } | null>(null);
  const [status, setStatus] = useState("Reading window state…");

  const refresh = useCallback(() => {
    void Promise.all([root.getWindowBounds(), root.getWindowState()])
      .then(([nextBounds, nextState]) => {
        setBounds(nextBounds);
        setState(nextState);
        setStatus("Window state refreshed");
      })
      .catch((error: unknown) => setStatus(`Window state failed: ${String(error)}`));
  }, [root]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const runCommand = (label: string, command: () => Promise<void>) => {
    void command()
      .then(() => setStatus(`${label} requested`))
      .catch((error: unknown) => setStatus(`${label} failed: ${String(error)}`));
  };

  return (
    <View style={styles.controls} accessibilityRole="generic" accessibilityLabel="Window controls">
      <Text style={styles.controlsTitle}>Window controls</Text>
      <View style={styles.controlsRow}>
        <Pressable
          style={styles.control}
          focusable
          accessibilityRole="button"
          accessibilityLabel="Minimize inspector window"
          onPress={() => runCommand("Minimize", () => root.minimizeWindow())}
        >
          <Text style={styles.controlLabel}>Minimize</Text>
        </Pressable>
        <Pressable
          style={styles.control}
          focusable
          accessibilityRole="button"
          accessibilityLabel="Activate inspector window"
          onPress={() => runCommand("Activate", () => root.activateWindow())}
        >
          <Text style={styles.controlLabel}>Activate</Text>
        </Pressable>
        <Pressable
          style={styles.control}
          focusable
          accessibilityRole="button"
          accessibilityLabel="Refresh inspector window state"
          onPress={refresh}
        >
          <Text style={styles.controlLabel}>Refresh</Text>
        </Pressable>
      </View>
      <Text style={styles.detail}>
        Bounds: {bounds ? `${bounds.x}, ${bounds.y} · ${bounds.width}×${bounds.height}` : "not loaded"}
      </Text>
      <Text style={styles.detail}>
        State: {state ? `fullscreen=${state.fullscreen}, maximized=${state.maximized}` : "not loaded"}
      </Text>
      <Text style={styles.helper}>{status}</Text>
    </View>
  );
}

function SurfacePanel({ name, sizeStore, appearanceStore, root }: SurfacePanelProps) {
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
      {root ? <WindowControls root={root} styles={styles} /> : null}
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
        root={inspectorRoot}
      />,
    );
  })
  .catch((error: unknown) => {
    console.error("Unable to open inspector surface", error);
  });
