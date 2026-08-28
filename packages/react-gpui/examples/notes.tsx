import { useEffect, useMemo, useState } from "react";
import {
  Pressable,
  StdioTransport,
  Text,
  TextInput,
  View,
  StyleSheet,
  createAppearanceStore,
  createProcessTerminationHandler,
  createRoot,
  useAppearance,
  type AppearanceStore,
  type Root,
} from "../src/index";
import { useTheme, type Theme } from "./theme";

function createStyles(theme: Theme) {
  return StyleSheet.create({
    root: { flexDirection: "column", flexGrow: 1, gap: 8, padding: 16, backgroundColor: theme.canvas },
    input: {
      flexGrow: 1,
      padding: 8,
      borderWidth: 1,
      borderColor: theme.borderInput,
      backgroundColor: theme.input,
      color: theme.text,
    },
    label: { color: theme.text },
    action: { color: theme.accentText, textDecoration: "underline", cursor: "pointer" },
    confirmation: {
      position: "overlay",
      left: 16,
      top: 16,
      flexDirection: "column",
      gap: 8,
      padding: 12,
      borderWidth: 1,
      borderColor: theme.borderInput,
      backgroundColor: theme.surface,
    },
  });
}

let notesCloseRequest: ((requestId: number) => void) | undefined;

function Notes({ appearanceStore }: { readonly appearanceStore: AppearanceStore }) {
  const theme = useTheme(useAppearance(appearanceStore));
  const styles = useMemo(() => createStyles(theme), [theme]);
  const [path, setPath] = useState<string | null>(null);
  const [content, setContent] = useState("");
  const [saved, setSaved] = useState("");
  const [status, setStatus] = useState("Choose a note to begin");
  const [pendingCloseRequestId, setPendingCloseRequestId] = useState<number | null>(null);
  const dirty = content !== saved;

  useEffect(() => {
    notesCloseRequest = (requestId) => {
      if (!dirty) {
        void root.resolveCloseRequest(requestId, true);
        return;
      }
      setStatus("Unsaved changes. Save or discard before closing.");
      setPendingCloseRequestId(requestId);
    };
    return () => {
      notesCloseRequest = undefined;
    };
  }, [dirty]);

  const open = async (): Promise<void> => {
    const paths = await root.pickFiles({ title: "Open note" });
    if (paths === null) return;
    const selected = paths[0];
    if (selected === undefined) return;
    try {
      const text = await root.readTextFile(selected);
      setPath(selected);
      setContent(text);
      setSaved(text);
      setStatus(`Opened ${selected}`);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  };
  const save = async (): Promise<boolean> => {
    const selected = path ?? (await root.pickSavePath({ defaultName: "notes.txt" }));
    if (selected === null || selected === undefined) return false;
    try {
      const bytes = await root.writeTextFile(selected, content);
      setPath(selected);
      setSaved(content);
      setStatus(`Saved ${bytes} bytes`);
      return true;
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
      return false;
    }
  };
  const saveAndClose = async (): Promise<void> => {
    const requestId = pendingCloseRequestId;
    if (requestId === null || !(await save())) return;
    setPendingCloseRequestId(null);
    void root.resolveCloseRequest(requestId, true);
  };
  return (
    <View style={styles.root} accessibilityRole="generic" accessibilityLabel="Notes editor">
      <Text style={styles.label}>
        {path ?? "Untitled note"} {dirty ? "*" : ""}
      </Text>
      <TextInput
        style={styles.input}
        value={content}
        onChangeText={setContent}
        multiline
        accessibilityLabel="Note contents"
        accessibilityDescription="Edit a text note"
      />
      <View style={{ flexDirection: "row", gap: 12 }}>
        <Pressable onPress={() => void open()}>
          <Text style={styles.action}>Open</Text>
        </Pressable>
        <Pressable onPress={() => void save()}>
          <Text style={styles.action}>Save</Text>
        </Pressable>
      </View>
      <Text style={styles.label}>{status}</Text>
      {pendingCloseRequestId !== null ? (
        <View style={styles.confirmation} onPointerDownOutside={() => undefined}>
          <Text style={styles.label}>This note has unsaved changes.</Text>
          <View style={{ flexDirection: "row", gap: 12 }}>
            <Pressable onPress={() => void saveAndClose()}>
              <Text style={styles.action}>Save</Text>
            </Pressable>
            <Pressable
              onPress={() => {
                const requestId = pendingCloseRequestId;
                setPendingCloseRequestId(null);
                void root.resolveCloseRequest(requestId, true);
              }}
            >
              <Text style={styles.action}>Save</Text>
            </Pressable>
            <Pressable
              onPress={() => {
                const requestId = pendingCloseRequestId;
                setPendingCloseRequestId(null);
                void root.resolveCloseRequest(requestId, true);
              }}
            >
              <Text style={styles.action}>Discard</Text>
            </Pressable>
          </View>
        </View>
      ) : null}
    </View>
  );
}
const appearanceStore = createAppearanceStore();
let root: Root;
root = createRoot(new StdioTransport(), {
  surfaceId: 1,
  epoch: 1,
  onAppearance: (appearance) => appearanceStore.set(appearance),
  onCloseRequested: (requestId) => {
    if (notesCloseRequest === undefined) {
      void root.resolveCloseRequest(requestId, true);
    } else {
      notesCloseRequest(requestId);
    }
  },
  onTransportTermination: createProcessTerminationHandler(),
});
void root.setClosePolicy("require-confirmation");
root.render(<Notes appearanceStore={appearanceStore} />);
