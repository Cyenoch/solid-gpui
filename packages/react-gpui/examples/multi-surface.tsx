import { useEffect, useMemo, useState, type ReactNode } from "react";
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
  readonly themeStore: AppearanceStore;
  readonly root?: Root;
  readonly content?: ReactNode;
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
    root: {
      flexDirection: "column",
      flexGrow: 1,
      gap: 12,
      padding: 20,
      overflow: "scroll",
      backgroundColor: theme.canvas,
    },
    title: { fontSize: 20, lineHeight: 26, fontWeight: "bold", color: theme.text },
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
    control: { padding: 6, borderRadius: 6, backgroundColor: theme.accentSoft, cursor: "pointer" },
    controlLabel: { fontSize: 12, lineHeight: 16, color: theme.accentText },
    button: {
      alignItems: "center",
      justifyContent: "center",
      padding: 10,
      borderRadius: 8,
      backgroundColor: theme.accent,
      cursor: "pointer",
    },
    buttonDisabled: { backgroundColor: theme.disabled, cursor: "not-allowed" },
    buttonLabel: { fontSize: 13, lineHeight: 18, fontWeight: "semibold", color: theme.onAccent },
    secondaryButton: { padding: 8, borderRadius: 7, backgroundColor: theme.accentSoft, cursor: "pointer" },
    secondaryLabel: { fontSize: 12, lineHeight: 16, color: theme.accentText },
    dangerButton: { padding: 8, borderRadius: 7, backgroundColor: theme.dangerSoft, cursor: "pointer" },
    dangerLabel: { fontSize: 12, lineHeight: 16, color: theme.danger },
    status: { fontSize: 12, lineHeight: 18, color: theme.text },
    caption: { fontSize: 11, lineHeight: 16, color: theme.textSubtle },
    confirmation: {
      position: "overlay",
      left: 16,
      top: 16,
      flexDirection: "column",
      gap: 8,
      padding: 12,
      borderWidth: 1,
      borderRadius: 8,
      borderColor: theme.borderInput,
      backgroundColor: theme.surface,
    },
    confirmationActions: { flexDirection: "row", gap: 8 },
  });
}

interface WindowControlsProps {
  readonly root: Root;
  readonly styles: WindowControlStyles;
  readonly surfaceName: string;
}

function WindowControls({ root, styles, surfaceName }: WindowControlsProps) {
  const [bounds, setBounds] = useState<{ x: number; y: number; width: number; height: number } | null>(null);
  const [state, setState] = useState<{ fullscreen: boolean; maximized: boolean } | null>(null);
  const [status, setStatus] = useState("Reading window state…");

  const refresh = () => {
    void Promise.all([root.getWindowBounds(), root.getWindowState()])
      .then(([nextBounds, nextState]) => {
        setBounds(nextBounds);
        setState(nextState);
        setStatus("Window state refreshed");
      })
      .catch((error: unknown) => setStatus(`Window state failed: ${String(error)}`));
  };

  useEffect(() => {
    refresh();
  }, []);

  const runCommand = (label: string, command: () => Promise<void>) => {
    void command()
      .then(() => setStatus(`${label} requested`))
      .catch((error: unknown) => setStatus(`${label} failed: ${String(error)}`));
  };

  return (
    <View style={styles.controls} accessibilityRole="generic" accessibilityLabel={`${surfaceName} window controls`}>
      <Text style={styles.controlsTitle}>Window controls</Text>
      <View style={styles.controlsRow}>
        <Pressable
          style={styles.control}
          focusable
          accessibilityRole="button"
          accessibilityLabel={`Minimize ${surfaceName} window`}
          onPress={() => runCommand("Minimize", () => root.minimizeWindow())}
        >
          <Text style={styles.controlLabel}>Minimize</Text>
        </Pressable>
        <Pressable
          style={styles.control}
          focusable
          accessibilityRole="button"
          accessibilityLabel={`Activate ${surfaceName} window`}
          onPress={() => runCommand("Activate", () => root.activateWindow())}
        >
          <Text style={styles.controlLabel}>Activate</Text>
        </Pressable>
        <Pressable
          style={styles.control}
          focusable
          accessibilityRole="button"
          accessibilityLabel={`Refresh ${surfaceName} window state`}
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

function SurfacePanel({ name, sizeStore, appearanceStore, themeStore, root, content }: SurfacePanelProps) {
  const { width, height, scaleFactor } = useWindowSize(sizeStore);
  const systemAppearance = useAppearance(appearanceStore);
  const themeAppearance = useAppearance(themeStore);
  const theme = useTheme(themeAppearance);
  const styles = useMemo(() => createStyles(theme), [theme]);

  return (
    <View style={{ ...styles.root, width, height }} accessibilityRole="generic" accessibilityLabel={`${name} surface`}>
      <Text style={styles.title}>{name}</Text>
      <Text style={styles.detail}>
        {width}×{height} logical pixels at {scaleFactor}x
      </Text>
      <Text style={styles.detail}>System appearance: {systemAppearance}</Text>
      <Text style={styles.detail}>Shared theme preference: {themeAppearance}</Text>
      {root ? <WindowControls root={root} styles={styles} surfaceName={name} /> : null}
      {content}
      <Text style={styles.helper}>
        Each root owns an explicit size and system-appearance bridge; the application-owned theme store is shared by
        both roots so a settings change repaints both surfaces.
      </Text>
    </View>
  );
}

interface MainSurfaceProps {
  readonly appearanceStore: AppearanceStore;
  readonly themeStore: AppearanceStore;
  readonly sizeStore: WindowSizeStore;
  readonly onOpenSettings: () => Promise<void>;
}

function MainSurface({ appearanceStore, themeStore, sizeStore, onOpenSettings }: MainSurfaceProps) {
  const theme = useTheme(useAppearance(themeStore));
  const styles = useMemo(() => createStyles(theme), [theme]);
  const [status, setStatus] = useState("Settings surface is not open");

  const openSettings = () => {
    setStatus("Opening settings surface…");
    void onOpenSettings()
      .then(() => setStatus("Settings surface is ready"))
      .catch((error: unknown) => setStatus(`Settings surface failed: ${String(error)}`));
  };

  return (
    <SurfacePanel
      name="Main surface"
      sizeStore={sizeStore}
      appearanceStore={appearanceStore}
      themeStore={themeStore}
      content={
        <View style={styles.controls}>
          <Text style={styles.controlsTitle}>Settings surface</Text>
          <Pressable
            style={styles.button}
            focusable
            accessibilityRole="button"
            accessibilityLabel="Open settings"
            onPress={openSettings}
          >
            <Text style={styles.buttonLabel}>Open settings</Text>
          </Pressable>
          <Text style={styles.status}>{status}</Text>
        </View>
      }
    />
  );
}

interface SettingsSurfaceProps {
  readonly appearanceStore: AppearanceStore;
  readonly themeStore: AppearanceStore;
  readonly sizeStore: WindowSizeStore;
  readonly root: Root;
  readonly mainRoot: Root;
}

let settingsCloseRequest: ((requestId: number) => void) | undefined;

function SettingsSurface({ appearanceStore, themeStore, sizeStore, root, mainRoot }: SettingsSurfaceProps) {
  const themePreference = useAppearance(themeStore);
  const theme = useTheme(themePreference);
  const styles = useMemo(() => createStyles(theme), [theme]);
  const [savedTheme, setSavedTheme] = useState(themePreference);
  const [pendingCloseRequestId, setPendingCloseRequestId] = useState<number | null>(null);
  const [status, setStatus] = useState("No unsaved changes");
  const dirty = themePreference !== savedTheme;

  useEffect(() => {
    settingsCloseRequest = (requestId) => {
      if (!dirty) {
        void root.resolveCloseRequest(requestId, true);
        return;
      }
      setStatus("Unsaved theme change. Save or discard before closing.");
      setPendingCloseRequestId(requestId);
    };
    return () => {
      settingsCloseRequest = undefined;
    };
  }, [dirty, root]);

  const toggleTheme = () => {
    const next = themePreference === "dark" ? "light" : "dark";
    themeStore.set(next);
    setStatus(`Shared theme changed to ${next}; save before closing.`);
  };

  const saveTheme = () => {
    setSavedTheme(themePreference);
    setStatus(`Saved shared theme: ${themePreference}`);
  };

  const focusMain = () => {
    void mainRoot
      .activateWindow()
      .then(() => mainRoot.focusNext())
      .then(() => setStatus("Main surface activated and focus advanced"))
      .catch((error: unknown) => setStatus(`Could not focus main surface: ${String(error)}`));
  };

  const resolveClose = (decision: "save" | "discard" | "keep"): void => {
    const requestId = pendingCloseRequestId;
    if (requestId === null) return;
    setPendingCloseRequestId(null);
    if (decision === "save") {
      setSavedTheme(themePreference);
      setStatus("Saved shared theme and closing…");
    } else if (decision === "discard") {
      themeStore.set(savedTheme);
      setStatus("Discarded shared theme and closing…");
    } else {
      setStatus("Close canceled; keep editing.");
    }
    void root.resolveCloseRequest(requestId, decision !== "keep").catch((error: unknown) => {
      setStatus(`Close decision failed: ${String(error)}`);
    });
  };

  return (
    <SurfacePanel
      name="Settings surface"
      sizeStore={sizeStore}
      appearanceStore={appearanceStore}
      themeStore={themeStore}
      root={root}
      content={
        <>
          <View style={styles.controls}>
            <Text style={styles.controlsTitle}>Shared theme preference</Text>
            <Text style={styles.detail}>Current preference: {themePreference}</Text>
            <Pressable
              style={styles.button}
              focusable
              accessibilityRole="button"
              accessibilityLabel="Toggle shared theme"
              onPress={toggleTheme}
            >
              <Text style={styles.buttonLabel}>Use {themePreference === "dark" ? "light" : "dark"} theme</Text>
            </Pressable>
            <Pressable
              disabled={!dirty}
              style={{ ...styles.secondaryButton, ...(!dirty ? styles.buttonDisabled : {}) }}
              focusable
              accessibilityRole="button"
              accessibilityLabel="Save shared theme"
              accessibilityDisabled={!dirty}
              onPress={saveTheme}
            >
              <Text style={styles.secondaryLabel}>Save theme</Text>
            </Pressable>
            <Text style={styles.status}>
              {dirty ? "Unsaved changes" : "Saved"} · {status}
            </Text>
          </View>
          <View style={styles.controls}>
            <Text style={styles.controlsTitle}>Cross-surface focus</Text>
            <Pressable
              style={styles.secondaryButton}
              focusable
              accessibilityRole="button"
              accessibilityLabel="Focus main surface"
              onPress={focusMain}
            >
              <Text style={styles.secondaryLabel}>Focus main</Text>
            </Pressable>
            <Text style={styles.caption}>
              Activates the main native surface, then asks that root to advance its focus target with focusNext().
            </Text>
          </View>
          <Text style={styles.caption}>
            Close confirmation is asynchronous. While this surface is dirty, use its native close control and resolve
            the request with Save and close, Discard and close, or Keep editing.
          </Text>
          <Text style={styles.caption}>
            Activation and exact window placement are display-backed boundaries; headless tests cover routing and
            command identity rather than native pixels.
          </Text>
          {pendingCloseRequestId !== null ? (
            <View style={styles.confirmation} onPointerDownOutside={() => undefined}>
              <Text style={styles.controlsTitle}>Unsaved shared theme</Text>
              <Text style={styles.detail}>Save or discard this window's change before closing.</Text>
              <View style={styles.confirmationActions}>
                <Pressable focusable style={styles.secondaryButton} onPress={() => resolveClose("keep")}>
                  <Text style={styles.secondaryLabel}>Keep editing</Text>
                </Pressable>
                <Pressable focusable style={styles.secondaryButton} onPress={() => resolveClose("save")}>
                  <Text style={styles.secondaryLabel}>Save and close</Text>
                </Pressable>
                <Pressable focusable style={styles.dangerButton} onPress={() => resolveClose("discard")}>
                  <Text style={styles.dangerLabel}>Discard and close</Text>
                </Pressable>
              </View>
            </View>
          ) : null}
        </>
      }
    />
  );
}

const host = createSurfaceHost(new StdioTransport(), {
  onTransportTermination: createProcessTerminationHandler(),
});
const sharedThemeStore = createAppearanceStore();
const mainAppearanceStore = createAppearanceStore();
const mainSizeStore = createWindowSizeStore({ width: 800, height: 600 });
let mainRoot: Root;
let settingsRoot: Root | undefined;
let settingsOpening: Promise<void> | undefined;

const openSettings = (): Promise<void> => {
  if (settingsRoot !== undefined) return settingsRoot.activateWindow();
  if (settingsOpening !== undefined) return settingsOpening;

  const opening = mainRoot
    .openSurface({
      title: "Settings",
      width: 480,
      height: 360,
      kind: "floating",
      resizable: true,
      minSize: [360, 260],
    })
    .then((surfaceId) => {
      const settingsSizeStore = createWindowSizeStore({ width: 480, height: 360 });
      const settingsAppearanceStore = createAppearanceStore();
      let childRoot: Root;
      childRoot = host.createRoot({
        surfaceId,
        onClose: () => {
          settingsCloseRequest = undefined;
          if (settingsRoot === childRoot) settingsRoot = undefined;
        },
        onCloseRequested: (requestId) => {
          if (settingsCloseRequest === undefined) {
            void childRoot.resolveCloseRequest(requestId, true);
          } else {
            settingsCloseRequest(requestId);
          }
        },
        onWindowResize: (width, height, scaleFactor) => settingsSizeStore.set(width, height, scaleFactor),
        onAppearance: (appearance) => settingsAppearanceStore.set(appearance),
      });
      settingsRoot = childRoot;
      void childRoot.setClosePolicy("require-confirmation");
      childRoot.render(
        <SettingsSurface
          appearanceStore={settingsAppearanceStore}
          themeStore={sharedThemeStore}
          sizeStore={settingsSizeStore}
          root={childRoot}
          mainRoot={mainRoot}
        />,
      );
    })
    .catch((error: unknown) => {
      console.error("Unable to open settings surface", error);
      throw error;
    })
    .finally(() => {
      settingsOpening = undefined;
    });
  settingsOpening = opening;
  return opening;
};

mainRoot = host.createRoot({
  surfaceId: 1,
  onWindowResize: (width, height, scaleFactor) => mainSizeStore.set(width, height, scaleFactor),
  onAppearance: (appearance) => mainAppearanceStore.set(appearance),
});
mainRoot.render(
  <MainSurface
    appearanceStore={mainAppearanceStore}
    themeStore={sharedThemeStore}
    sizeStore={mainSizeStore}
    onOpenSettings={openSettings}
  />,
);
