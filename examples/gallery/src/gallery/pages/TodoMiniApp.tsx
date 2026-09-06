import { Icon, Text, TextInput, View } from "@solid-gpui/core";
import { createMemo, createSignal } from "@solid-gpui/core/runtime";
import { useGallery } from "../context";
import type { SolidChild } from "../types";
import { Badge, Button, Card, Checkbox, DenseRow, ResponsiveRow, SectionHeader } from "../components/ui";

interface TodoItem {
  readonly id: number;
  readonly text: string;
  readonly completed: boolean;
  readonly priority: "High" | "Medium" | "Low";
  readonly category: string;
}

export function TodoMiniApp(): SolidChild {
  const { theme, showStatus } = useGallery();

  const [todos, setTodos] = createSignal<TodoItem[]>([
    { id: 1, text: "Explore Solid GPUI Gallery features", completed: true, priority: "High", category: "Learning" },
    { id: 2, text: "Build native desktop app", completed: false, priority: "High", category: "Core" },
    {
      id: 3,
      text: "Integrate fine-grained reactive signals",
      completed: false,
      priority: "Medium",
      category: "Design",
    },
    { id: 4, text: "Optimize virtual list rendering", completed: false, priority: "Low", category: "Performance" },
  ]);

  const [newTodoText, setNewTodoText] = createSignal("");
  const [newPriority, setNewPriority] = createSignal<"High" | "Medium" | "Low">("Medium");
  const [filter, setFilter] = createSignal<"all" | "active" | "completed">("all");

  const filteredTodos = createMemo(() => {
    const f = filter();
    const list = todos();
    if (f === "active") return list.filter((t) => !t.completed);
    if (f === "completed") return list.filter((t) => t.completed);
    return list;
  });

  const activeCount = createMemo(() => todos().filter((t) => !t.completed).length);
  const completedCount = createMemo(() => todos().filter((t) => t.completed).length);

  const addTodo = () => {
    const text = newTodoText().trim();
    if (!text) return;
    const item: TodoItem = {
      id: Date.now(),
      text,
      completed: false,
      priority: newPriority(),
      category: "General",
    };
    setTodos((prev) => [item, ...prev]);
    setNewTodoText("");
    showStatus(`Added task "${text}"`, "success");
  };

  const toggleTodo = (id: number) => {
    setTodos((prev) => prev.map((item) => (item.id === id ? { ...item, completed: !item.completed } : item)));
  };

  const deleteTodo = (id: number) => {
    setTodos((prev) => prev.filter((item) => item.id !== id));
    showStatus("Task deleted", "info");
  };

  const clearCompleted = () => {
    setTodos((prev) => prev.filter((item) => !item.completed));
    showStatus("Cleared completed tasks", "info");
  };

  return (
    <View style={{ gap: 20, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Mini-App: Todo Studio"
        tag="Interactive Application"
        description="Task state, filtering, list updates, and checkbox interaction."
      />

      {/* Main Todo Container */}
      <Card
        title="Task Management Board"
        description={`Total: ${todos().length} tasks • ${activeCount()} active • ${completedCount()} completed`}
        actions={
          <View style={{ flexDirection: "row", gap: 8 }}>
            {completedCount() > 0 ? (
              <Button size="sm" variant="ghost" onPress={clearCompleted}>
                Clear Completed
              </Button>
            ) : null}
          </View>
        }
      >
        {/* Add Todo Input Bar */}
        <DenseRow
          gap={8}
          style={{ backgroundColor: theme().bgHover, padding: 10, borderRadius: 6, alignItems: "center" }}
        >
          <TextInput
            style={{
              flexGrow: 1,
              alignSelf: "stretch",
              minWidth: 0,
              flexShrink: 1,
              backgroundColor: theme().bgInput,
              borderWidth: 1,
              borderColor: theme().border,
              borderRadius: 6,
              padding: 8,
              color: theme().textPrimary,
              fontSize: 13,
            }}
            placeholder="What needs to be done? Type and press enter..."
            value={newTodoText()}
            onChangeText={(text) => setNewTodoText(text)}
            onKeyDown={(e) => {
              if (e.key === "Enter") addTodo();
            }}
          />

          {/* Priority selector */}
          <View style={{ flexDirection: "row", gap: 4 }}>
            <Button
              size="sm"
              variant={newPriority() === "High" ? "danger" : "secondary"}
              onPress={() => setNewPriority("High")}
            >
              High
            </Button>
            <Button
              size="sm"
              variant={newPriority() === "Medium" ? "primary" : "secondary"}
              onPress={() => setNewPriority("Medium")}
            >
              Med
            </Button>
            <Button
              size="sm"
              variant={newPriority() === "Low" ? "outline" : "secondary"}
              onPress={() => setNewPriority("Low")}
            >
              Low
            </Button>
          </View>
          <Button variant="primary" onPress={addTodo} disabled={!newTodoText().trim()}>
            Add Task
          </Button>
        </DenseRow>

        {/* Filter Tabs */}
        <DenseRow gap={6} style={{ justifyContent: "flex-start", alignItems: "center", padding: 4 }}>
          <DenseRow gap={6}>
            <Button size="sm" variant={filter() === "all" ? "primary" : "ghost"} onPress={() => setFilter("all")}>
              {`All (${todos().length})`}
            </Button>
            <Button size="sm" variant={filter() === "active" ? "primary" : "ghost"} onPress={() => setFilter("active")}>
              {`Active (${activeCount()})`}
            </Button>
            <Button
              size="sm"
              variant={filter() === "completed" ? "primary" : "ghost"}
              onPress={() => setFilter("completed")}
            >
              {`Completed (${completedCount()})`}
            </Button>
          </DenseRow>
          <Text style={{ color: theme().textMuted, fontSize: 12 }}>{`${activeCount()} items left`}</Text>
        </DenseRow>

        {/* Tasks List */}
        <View style={{ gap: 6 }}>
          {filteredTodos().length === 0 ? (
            <View style={{ padding: 32, alignItems: "center", justifyContent: "center" }}>
              <Text style={{ color: theme().textMuted, fontSize: 13, fontStyle: "italic" }}>
                No tasks found in this view.
              </Text>
            </View>
          ) : (
            filteredTodos().map((item) => {
              let priorityVariant: "danger" | "warning" | "neutral" = "neutral";
              if (item.priority === "High") priorityVariant = "danger";
              else if (item.priority === "Medium") priorityVariant = "warning";

              return (
                <ResponsiveRow
                  gap={8}
                  grow={[1, 0]}
                  style={{
                    justifyContent: "space-between",
                    alignItems: "center",
                    padding: 8,
                    backgroundColor: item.completed ? theme().bgHover : theme().bgCard,
                    borderRadius: 6,
                    borderWidth: 1,
                    borderColor: theme().borderMuted,
                  }}
                >
                  <View
                    style={{
                      flexDirection: "row",
                      alignItems: "center",
                      gap: 12,
                      minWidth: 0,
                      flexGrow: 1,
                      flexShrink: 1,
                    }}
                  >
                    <Checkbox checked={item.completed} onChange={() => toggleTodo(item.id)} />
                    <Text
                      style={{
                        color: item.completed ? theme().textMuted : theme().textPrimary,
                        fontSize: 14,
                        minWidth: 0,
                        flexShrink: 1,
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        textDecoration: item.completed ? ("lineThrough" as const) : ("none" as const),
                        fontWeight: item.completed ? ("normal" as const) : ("medium" as const),
                      }}
                    >
                      {item.text}
                    </Text>
                  </View>
                  <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
                    <Badge label={item.priority} variant={priorityVariant} size="sm" />
                    <Button
                      variant="ghost"
                      size="sm"
                      icon="lucide:x"
                      accessibilityLabel="Delete task"
                      onPress={() => deleteTodo(item.id)}
                    />
                  </View>
                </ResponsiveRow>
              );
            })
          )}
        </View>
      </Card>
    </View>
  );
}
