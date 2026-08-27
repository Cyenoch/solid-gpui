import { useMemo, useState } from "react";
import {
  Pressable,
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

function createStyles(theme: Theme) {
  return StyleSheet.create({
    root: { flexDirection: "column", flexGrow: 1, gap: 12, padding: 20, backgroundColor: theme.canvas },
    form: {
      flexDirection: "column",
      gap: 8,
      padding: 16,
      borderWidth: 1,
      borderRadius: 10,
      borderColor: theme.border,
      backgroundColor: theme.surface,
    },
    title: { fontSize: 18, lineHeight: 24, fontWeight: "bold", color: theme.text },
    helper: { fontSize: 12, lineHeight: 18, color: theme.textMuted },
    field: {
      padding: 12,
      borderWidth: 1,
      borderRadius: 8,
      borderColor: theme.borderInput,
      backgroundColor: theme.surface,
    },
    fieldFocused: { borderColor: theme.focusRing, backgroundColor: theme.accentSoft },
    fieldLabel: { fontSize: 13, lineHeight: 18, fontWeight: "medium", color: theme.text },
    submit: {
      alignItems: "center",
      justifyContent: "center",
      padding: 10,
      borderRadius: 8,
      backgroundColor: theme.accent,
    },
    submitFocused: { backgroundColor: theme.accentHover },
    submitLabel: { fontSize: 13, lineHeight: 18, fontWeight: "semibold", color: theme.onAccent },
    status: { fontSize: 12, lineHeight: 18, color: theme.textMuted },
  });
}

type FocusTarget = "name" | "email" | "submit";

interface FocusFlowProps {
  readonly onFocusNext: () => void;
  readonly onFocusPrevious: () => void;
  readonly appearanceStore: AppearanceStore;
}

function FocusFlow({ onFocusNext, onFocusPrevious, appearanceStore }: FocusFlowProps) {
  const [focused, setFocused] = useState<FocusTarget | null>(null);
  const [submitted, setSubmitted] = useState(false);
  const theme = useTheme(useAppearance(appearanceStore));
  const styles = useMemo(() => createStyles(theme), [theme]);

  const clearFocus = (target: FocusTarget) => {
    setFocused((current) => (current === target ? null : current));
  };
  const handleKeyDown = ({ key, action, modifiers }: { key: string; action: string; modifiers: string[] }) => {
    if (action !== "down" || key !== "tab") return;
    if (modifiers.includes("shift")) onFocusPrevious();
    else onFocusNext();
  };

  return (
    <View style={styles.root} accessibilityRole="generic" accessibilityLabel="Focusable form flow">
      <Text style={styles.title}>Focusable form flow</Text>
      <Text style={styles.helper}>
        Use Tab to move forward, Shift-Tab to move backward, and watch the focused control change style.
      </Text>
      <View style={styles.form}>
        <Pressable
          focusable
          style={{ ...styles.field, ...(focused === "name" ? styles.fieldFocused : {}) }}
          accessibilityRole="button"
          accessibilityLabel="Name field"
          onFocus={() => setFocused("name")}
          onBlur={() => clearFocus("name")}
          onKeyDown={handleKeyDown}
          onPress={() => setFocused("name")}
        >
          <Text style={styles.fieldLabel}>Name</Text>
        </Pressable>
        <Pressable
          focusable
          style={{ ...styles.field, ...(focused === "email" ? styles.fieldFocused : {}) }}
          accessibilityRole="button"
          accessibilityLabel="Email field"
          onFocus={() => setFocused("email")}
          onBlur={() => clearFocus("email")}
          onKeyDown={handleKeyDown}
          onPress={() => setFocused("email")}
        >
          <Text style={styles.fieldLabel}>Email</Text>
        </Pressable>
        <Pressable
          focusable
          style={{ ...styles.submit, ...(focused === "submit" ? styles.submitFocused : {}) }}
          accessibilityRole="button"
          accessibilityLabel="Submit form"
          onFocus={() => setFocused("submit")}
          onBlur={() => clearFocus("submit")}
          onKeyDown={handleKeyDown}
          onPress={() => setSubmitted(true)}
        >
          <Text style={styles.submitLabel}>Submit</Text>
        </Pressable>
      </View>
      <Text style={styles.status}>
        Focused: {focused ?? "none"}; submitted: {submitted ? "yes" : "no"}
      </Text>
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
root.render(
  <FocusFlow
    appearanceStore={appearanceStore}
    onFocusNext={() => {
      void root.focusNext();
    }}
    onFocusPrevious={() => {
      void root.focusPrev();
    }}
  />,
);
