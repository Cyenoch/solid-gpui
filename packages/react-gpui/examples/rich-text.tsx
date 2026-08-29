import { useEffect, useMemo, useState } from "react";
import {
  StdioTransport,
  StyleSheet,
  Text,
  View,
  createAppearanceStore,
  createProcessTerminationHandler,
  createRoot,
  useAppearance,
  type AppearanceStore,
} from "../src/index";
import { useTheme, type Theme } from "./theme";

const FONT_PATH = new URL("./assets/tuffy.ttf", import.meta.url).pathname;

type LinkName = "guide" | "source";

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
    intro: { fontSize: 13, lineHeight: 19, color: theme.textMuted },
    card: {
      flexDirection: "column",
      gap: 8,
      padding: 14,
      borderWidth: 1,
      borderRadius: 10,
      borderColor: theme.border,
      backgroundColor: theme.surface,
    },
    cardTitle: { fontSize: 15, lineHeight: 20, fontWeight: "semibold", color: theme.text },
    cardKicker: { fontSize: 11, lineHeight: 16, color: theme.textMuted },
    paragraph: { fontSize: 17, lineHeight: 25, color: theme.text },
    boldRun: { color: theme.accentText, fontWeight: "bold" },
    italicRun: { color: theme.success, fontStyle: "italic" },
    strikeRun: { color: theme.danger, textDecoration: "lineThrough" },
    underlineRun: { color: theme.accentText, textDecoration: "underline" },
    fontRun: { color: theme.text, fontWeight: "medium" },
    linkParagraph: { fontSize: 15, lineHeight: 23, color: theme.text },
    link: { color: theme.accentText, fontWeight: "semibold", textDecoration: "underline" },
    linkFocused: { color: theme.focusRing, fontWeight: "bold" },
    status: { fontSize: 12, lineHeight: 18, color: theme.textMuted },
    log: { fontSize: 12, lineHeight: 18, color: theme.text },
    selectable: { fontSize: 15, lineHeight: 23, color: theme.text, maxWidth: 700 },
    selectableAccent: { color: theme.accentText, fontWeight: "semibold" },
    selectableMuted: { color: theme.textMuted, fontStyle: "italic" },
    caption: { fontSize: 11, lineHeight: 16, color: theme.textSubtle },
  });
}

function RichText({
  appearanceStore,
  fontReady,
}: {
  readonly appearanceStore: AppearanceStore;
  readonly fontReady: Promise<string>;
}) {
  const theme = useTheme(useAppearance(appearanceStore));
  const styles = useMemo(() => createStyles(theme), [theme]);
  const [fontFamily, setFontFamily] = useState<string>();
  const [fontStatus, setFontStatus] = useState("Loading Tuffy…");
  const [focusedLink, setFocusedLink] = useState<LinkName | null>(null);
  const [linkClicks, setLinkClicks] = useState(0);
  const [linkLog, setLinkLog] = useState<string[]>([]);

  useEffect(() => {
    let mounted = true;
    void fontReady
      .then((family) => {
        if (!mounted) return;
        setFontFamily(family);
        setFontStatus(`Loaded runtime font: ${family}`);
      })
      .catch((error: unknown) => {
        if (mounted) setFontStatus(`Runtime font unavailable: ${String(error)}`);
      });
    return () => {
      mounted = false;
    };
  }, [fontReady]);

  const activateLink = (name: LinkName) => {
    setLinkClicks((current) => current + 1);
    setLinkLog((current) => [...current, `${name} link activated`].slice(-4));
  };
  const fontRunStyle = fontFamily ? { ...styles.fontRun, fontFamily } : styles.fontRun;
  const guideLinkStyle = { ...styles.link, ...(focusedLink === "guide" ? styles.linkFocused : {}) };
  const sourceLinkStyle = { ...styles.link, ...(focusedLink === "source" ? styles.linkFocused : {}) };

  return (
    <View style={styles.root} accessibilityRole="generic" accessibilityLabel="Rich text example">
      <Text style={styles.title}>Rich text runs</Text>
      <Text style={styles.intro}>
        One screen for inline styling, keyboard-accessible links, and native selection across a flattened paragraph.
      </Text>
      <View style={styles.card}>
        <Text style={styles.cardTitle}>Mixed typography</Text>
        <Text style={styles.cardKicker}>Each nested run keeps its own color, weight, style, and decoration.</Text>
        <Text style={styles.paragraph}>
          Base text with <Text style={styles.boldRun}>bold blue</Text>,{" "}
          <Text style={styles.italicRun}>italic green</Text>, <Text style={styles.strikeRun}>struck red</Text>,{" "}
          <Text style={styles.underlineRun}>underlined blue</Text>, and{" "}
          <Text style={fontRunStyle}>a loaded runtime font</Text>.
        </Text>
        <Text style={styles.status}>{fontStatus}</Text>
        <Text style={styles.caption}>
          Boundary: nested fontSize and lineHeight are intentionally rejected; this paragraph keeps one size.
        </Text>
      </View>
      <View style={styles.card}>
        <Text style={styles.cardTitle}>Interactive link runs</Text>
        <Text style={styles.cardKicker}>
          Tab to a link, press Enter to activate it, and watch the native focus rule.
        </Text>
        <Text style={styles.linkParagraph}>
          Read the{" "}
          <Text
            style={guideLinkStyle}
            accessibilityRole="link"
            accessibilityLabel="Rich text guide"
            onPress={() => activateLink("guide")}
            onFocus={() => setFocusedLink("guide")}
            onBlur={() => setFocusedLink((current) => (current === "guide" ? null : current))}
          >
            rich text guide
          </Text>{" "}
          or inspect the{" "}
          <Text
            style={sourceLinkStyle}
            accessibilityRole="link"
            accessibilityLabel="Rich text source"
            onPress={() => activateLink("source")}
            onFocus={() => setFocusedLink("source")}
            onBlur={() => setFocusedLink((current) => (current === "source" ? null : current))}
          >
            source example
          </Text>
          .
        </Text>
        <Text style={styles.status}>
          Focused link: {focusedLink ?? "none"} · Activations: {linkClicks}
        </Text>
        <Text style={styles.log}>{linkLog.length === 0 ? "No link activations yet." : linkLog.join(" · ")}</Text>
      </View>
      <View style={styles.card}>
        <Text style={styles.cardTitle}>Selectable rich text</Text>
        <Text style={styles.cardKicker}>
          Drag across any runs, then press Cmd-C on macOS or Ctrl-C elsewhere to copy.
        </Text>
        <Text selectable style={styles.selectable}>
          Selection crosses <Text style={styles.selectableAccent}>colored runs</Text>, preserves the flattened
          paragraph, and <Text style={styles.selectableMuted}>does not need JavaScript selection state</Text>.
        </Text>
        <Text style={styles.caption}>
          Selection and copy belong to the native host; this example intentionally has no selection callback.
        </Text>
      </View>
    </View>
  );
}

const appearanceStore = createAppearanceStore();
const root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onAppearance: (appearance) => appearanceStore.set(appearance),
  onTransportTermination: createProcessTerminationHandler(),
});
const fontReady = root.loadFont(FONT_PATH);
root.render(<RichText appearanceStore={appearanceStore} fontReady={fontReady} />);
