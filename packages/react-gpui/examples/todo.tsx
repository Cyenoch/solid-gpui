import { useCallback, useMemo, useRef, useState } from "react";
import {
  Image,
  Pressable,
  StdioTransport,
  StyleSheet,
  Text,
  TextInput,
  View,
  VirtualList,
  createAppearanceStore,
  createProcessTerminationHandler,
  createRoot,
  createWindowSizeStore,
  useAppearance,
  useWindowSize,
  type AppearanceStore,
  type KeyEvent,
  type Style,
  type WindowSizeStore,
} from "../src/index";
import { useTheme, type Theme } from "./theme";

export interface TodoItem {
  readonly id: number;
  readonly title: string;
  readonly completed: boolean;
}

export interface TodoAppProps {
  readonly windowSizeStore: WindowSizeStore;
  readonly onFocusNext?: () => void;
  readonly appearanceStore?: AppearanceStore;
}

const AVATAR_SOURCE = new URL("./todo-avatar.svg", import.meta.url).pathname;

const INITIAL_TODOS: readonly TodoItem[] = [
  { id: 1, title: "Review the GPUI renderer API", completed: false },
  { id: 2, title: "Try the todo app with a narrow window", completed: false },
  { id: 3, title: "Write down one developer-experience friction", completed: true },
];
const fallbackAppearanceStore = createAppearanceStore();

function createStyles(theme: Theme) {
  return StyleSheet.create({
    root: {
      flexDirection: "column",
      flexGrow: 1,
      padding: 16,
      gap: 10,
      backgroundColor: theme.canvas,
    },
    header: {
      flexDirection: "row",
      alignItems: "baseline",
      justifyContent: "space-between",
      marginBottom: 4,
    },
    title: {
      fontSize: 26,
      fontWeight: "bold",
      color: theme.text,
    },
    subtitle: {
      fontSize: 13,
      fontStyle: "italic",
      color: theme.textMuted,
      marginLeft: 8,
    },
    composer: {
      flexDirection: "row",
      alignItems: "center",
      gap: 8,
      marginBottom: 2,
    },
    input: {
      flexGrow: 1,
      minWidth: 120,
      padding: 9,
      borderRadius: 7,
      borderWidth: 1,
      borderColor: theme.borderInput,
      backgroundColor: theme.input,
      color: theme.text,
      fontSize: 14,
    },
    addButton: {
      padding: 9,
      borderRadius: 7,
      backgroundColor: theme.accent,
    },
    addButtonLabel: {
      fontWeight: "bold",
      color: theme.onAccent,
    },
    list: {
      flexGrow: 1,
      minHeight: 220,
      overflow: "scroll",
    },
    row: {
      flexDirection: "row",
      alignItems: "center",
      gap: 10,
      padding: 9,
      marginBottom: 8,
      borderRadius: 8,
      backgroundColor: theme.surface,
    },
    avatar: {
      width: 32,
      height: 32,
      borderRadius: 16,
      flexShrink: 0,
    },
    rowBody: {
      flexDirection: "column",
      flexGrow: 1,
      gap: 6,
      minWidth: 0,
    },
    todoTitle: {
      fontSize: 14,
      color: theme.text,
    },
    actions: {
      flexDirection: "row",
      gap: 12,
    },
    actionLabel: {
      fontSize: 12,
      color: theme.accentText,
      textDecoration: "underline",
    },
    deleteLabel: {
      fontSize: 12,
      color: theme.danger,
      textDecoration: "underline",
    },
    editInput: {
      padding: 5,
      borderRadius: 5,
      borderWidth: 1,
      borderColor: theme.focusRing,
      backgroundColor: theme.inputFocus,
      color: theme.text,
      fontSize: 14,
    },
    footer: {
      marginTop: 4,
      padding: 8,
      borderRadius: 6,
      backgroundColor: theme.surfaceMuted,
    },
    footerLabel: {
      fontSize: 12,
      fontStyle: "italic",
      textDecoration: "underline",
      color: theme.textMuted,
    },
    compactHint: {
      marginTop: 0,
      marginBottom: 2,
      fontSize: 12,
      color: theme.textMuted,
    },
    emptyState: {
      marginTop: 16,
      fontStyle: "italic",
      textDecoration: "underline",
      color: theme.textMuted,
    },
  });
}
interface TodoStyles {
  readonly root: Style;
  readonly header: Style;
  readonly title: Style;
  readonly subtitle: Style;
  readonly composer: Style;
  readonly input: Style;
  readonly addButton: Style;
  readonly addButtonLabel: Style;
  readonly list: Style;
  readonly row: Style;
  readonly avatar: Style;
  readonly rowBody: Style;
  readonly todoTitle: Style;
  readonly actions: Style;
  readonly actionLabel: Style;
  readonly deleteLabel: Style;
  readonly editInput: Style;
  readonly footer: Style;
  readonly footerLabel: Style;
  readonly compactHint: Style;
  readonly emptyState: Style;
}

interface TodoRowProps {
  readonly item: TodoItem;
  readonly compact: boolean;
  readonly editing: boolean;
  readonly editDraft: string;
  readonly styles: TodoStyles;
  readonly onToggle: (id: number) => void;
  readonly onDelete: (id: number) => void;
  readonly onBeginEdit: (item: TodoItem) => void;
  readonly onEditDraft: (value: string) => void;
  readonly onCommitEdit: (value: string) => void;
  readonly onCancelEdit: () => void;
}

function TodoRow({
  item,
  compact,
  editing,
  editDraft,
  styles,
  onToggle,
  onDelete,
  onBeginEdit,
  onEditDraft,
  onCommitEdit,
  onCancelEdit,
}: TodoRowProps) {
  const titleStyle: Style = {
    ...styles.todoTitle,
    marginLeft: compact ? 2 : 6,
    fontStyle: item.completed ? "italic" : "normal",
    textDecoration: item.completed ? "lineThrough" : "none",
  };
  return (
    <View style={styles.row} accessibilityRole="generic">
      <Image source={AVATAR_SOURCE} objectFit="contain" style={styles.avatar} accessibilityLabel="Todo owner avatar" />
      <View style={styles.rowBody}>
        {editing ? (
          <TextInput
            style={styles.editInput}
            value={editDraft}
            onChangeText={onEditDraft}
            onSubmitEditing={onCommitEdit}
            onKeyDown={(event) => {
              if (event.key === "Escape" && event.action === "down") onCancelEdit();
            }}
            accessibilityRole="textbox"
            accessibilityLabel={`Edit ${item.title}`}
          />
        ) : (
          <Pressable
            focusable
            onPress={() => onToggle(item.id)}
            accessibilityRole="checkbox"
            accessibilityLabel={item.title}
            accessibilityChecked={item.completed}
          >
            <Text style={titleStyle}>{item.title}</Text>
          </Pressable>
        )}
        <View style={styles.actions}>
          <Pressable
            focusable
            onPress={editing ? () => onCommitEdit(editDraft) : () => onBeginEdit(item)}
            accessibilityRole="button"
            accessibilityLabel={editing ? `Save ${item.title}` : `Edit ${item.title}`}
          >
            <Text style={styles.actionLabel}>{editing ? "Save" : "Edit"}</Text>
          </Pressable>
          <Pressable
            focusable
            onPress={() => onDelete(item.id)}
            accessibilityRole="button"
            accessibilityLabel={`Delete ${item.title}`}
          >
            <Text style={styles.deleteLabel}>Delete</Text>
          </Pressable>
        </View>
      </View>
    </View>
  );
}
export function TodoApp({ windowSizeStore, onFocusNext, appearanceStore = fallbackAppearanceStore }: TodoAppProps) {
  const { width, height } = useWindowSize(windowSizeStore);
  const compact = width < 720;
  const theme = useTheme(useAppearance(appearanceStore));
  const styles = useMemo(() => createStyles(theme), [theme]);
  const [draft, setDraft] = useState("");
  const [todos, setTodos] = useState<readonly TodoItem[]>(INITIAL_TODOS);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editDraft, setEditDraft] = useState("");
  const nextId = useRef(INITIAL_TODOS.length + 1);

  const addTodo = useCallback((text: string) => {
    const title = text.trim();
    if (title.length === 0) return;
    const item: TodoItem = { id: nextId.current, title, completed: false };
    nextId.current += 1;
    setTodos((current) => [item, ...current]);
    setDraft("");
  }, []);

  const toggleTodo = useCallback((id: number) => {
    setTodos((current) => current.map((item) => (item.id === id ? { ...item, completed: !item.completed } : item)));
  }, []);

  const deleteTodo = useCallback((id: number) => {
    setTodos((current) => current.filter((item) => item.id !== id));
    setEditingId((current) => (current === id ? null : current));
  }, []);

  const beginEdit = useCallback((item: TodoItem) => {
    setEditingId(item.id);
    setEditDraft(item.title);
  }, []);

  const commitEdit = useCallback(
    (text: string) => {
      if (editingId === null) return;
      const title = text.trim();
      if (title.length === 0) return;
      setTodos((current) => current.map((item) => (item.id === editingId ? { ...item, title } : item)));
      setEditingId(null);
      setEditDraft("");
    },
    [editingId],
  );
  const cancelEdit = useCallback(() => {
    setEditingId(null);
    setEditDraft("");
  }, []);

  const handleKeyDown = useCallback(
    (event: KeyEvent) => {
      if (event.key === "Tab" && event.action === "down") onFocusNext?.();
    },
    [onFocusNext],
  );

  const renderItem = useCallback(
    (item: TodoItem) => (
      <TodoRow
        item={item}
        compact={compact}
        editing={editingId === item.id}
        editDraft={editDraft}
        styles={styles}
        onToggle={toggleTodo}
        onDelete={deleteTodo}
        onBeginEdit={beginEdit}
        onEditDraft={setEditDraft}
        onCommitEdit={commitEdit}
        onCancelEdit={cancelEdit}
      />
    ),
    [beginEdit, cancelEdit, commitEdit, compact, deleteTodo, editDraft, editingId, styles, toggleTodo],
  );

  const listStyle: Style = {
    ...styles.list,
    height: Math.max(220, Math.min(560, height - (compact ? 235 : 250))),
  };

  return (
    <View style={styles.root} accessibilityRole="generic" accessibilityLabel="Todo list">
      <View style={styles.header}>
        <Text style={styles.title}>Todos</Text>
        <Text style={styles.subtitle}>{todos.length} items</Text>
      </View>
      {compact ? <Text style={styles.compactHint}>Compact layout for a narrow window</Text> : null}
      <View style={styles.composer}>
        <TextInput
          style={styles.input}
          value={draft}
          onChangeText={setDraft}
          onSubmitEditing={addTodo}
          onKeyDown={handleKeyDown}
          placeholder="What needs doing?"
          accessibilityLabel="New todo"
          accessibilityRole="textbox"
        />
        <Pressable
          focusable
          disabled={draft.trim().length === 0}
          style={styles.addButton}
          onPress={() => addTodo(draft)}
          accessibilityRole="button"
          accessibilityLabel="Add todo"
        >
          <Text style={styles.addButtonLabel}>Add</Text>
        </Pressable>
      </View>
      <VirtualList<TodoItem>
        style={listStyle}
        data={todos}
        itemKey={(item) => item.id}
        renderItem={renderItem}
        estimatedItemSize={compact ? 88 : 96}
        overscan={3}
        initialNumToRender={8}
        emptyState={<Text style={styles.emptyState}>No todos yet — add one above.</Text>}
      />
      <View focusable onKeyDown={handleKeyDown} style={styles.footer} accessibilityRole="generic">
        <Text style={styles.footerLabel}>Tab moves through focusable controls; press a todo to complete it.</Text>
      </View>
    </View>
  );
}
const isMain = (import.meta as ImportMeta & { readonly main?: boolean }).main === true;
if (isMain) {
  const windowSizeStore = createWindowSizeStore();
  const appearanceStore = createAppearanceStore();
  const root = createRoot(new StdioTransport(), {
    onWindowResize: (width, height) => windowSizeStore.set(width, height),
    onAppearance: (appearance) => appearanceStore.set(appearance),
    onTransportTermination: createProcessTerminationHandler(),
  });
  root.render(
    <TodoApp
      windowSizeStore={windowSizeStore}
      appearanceStore={appearanceStore}
      onFocusNext={() => void root.focusNext()}
    />,
  );
}
