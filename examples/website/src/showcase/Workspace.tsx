import { View, VirtualList, Icon } from "@solid-gpui/core";
import * as N from "@solid-gpui/core/components";
import { createMemo, createSignal } from "@solid-gpui/core/runtime";
import { Button, Copy, colors } from "../ui";
import { t } from "../i18n";
import { addTask, toggleTask, type Task } from "./model";
const card = {
  padding: 24,
  gap: 20,
  borderWidth: 1,
  borderColor: colors.line,
  borderRadius: 12,
  backgroundColor: colors.panel,
  minWidth: 0,
  flexShrink: 0,
} as const;
export function createWorkspaceState() {
  const [tasks, setTasks] = createSignal<Task[]>([
    { id: 1, title: "Explore the component library", done: true },
    { id: 2, title: "Sketch the first screen", done: false },
    { id: 3, title: "Share a working prototype", done: false },
  ]);
  const [draft, setDraft] = createSignal("");
  const [filter, setFilter] = createSignal("All");
  return { tasks, setTasks, draft, setDraft, filter, setFilter };
}
export function Workspace(props: { state: ReturnType<typeof createWorkspaceState> }) {
  const { tasks, setTasks, draft, setDraft, filter, setFilter } = props.state;
  const completed = () => tasks().filter((task) => task.done).length;
  const visible = createMemo(() =>
    tasks().filter((task) => filter() === "All" || task.done === (filter() === "Completed")),
  );
  const submit = () => {
    if (!draft().trim()) return;
    setTasks(addTask(tasks(), draft()));
    setDraft("");
  };
  return (
    <View style={card}>
      <View style={{ flexDirection: "row", alignItems: "center", gap: 10 }}>
        <Icon name="lucide:check" size={20} color={colors.text} />
        <Copy size={20}>A little progress, every day.</Copy>
      </View>
      <Copy color={colors.muted}>Your next project starts here.</Copy>
      <N.Progress value={tasks().length ? (completed() / tasks().length) * 100 : 0} />
      <Copy size={13} color={colors.muted}>{`${completed()} / ${tasks().length} ${t("completed")}`}</Copy>
      <N.Input value={draft()} onChange={(event) => setDraft(event.value)} placeholder={t("What needs doing?")} />
      <N.Button label={t("Add task")} variant="primary" disabled={!draft().trim()} onPress={submit} />
      <View style={{ flexDirection: "row", gap: 6 }}>
        {["All", "Active", "Completed"].map((label) => (
          <Button compact ghost active={filter() === label} onPress={() => setFilter(label)}>
            {label}
          </Button>
        ))}
      </View>
      <VirtualList
        data={visible()}
        itemKey={(task) => task.id}
        estimatedItemSize={58}
        style={{ height: 280 }}
        renderItem={(task) => (
          <View
            style={{
              padding: 10,
              gap: 8,
              flexDirection: "row",
              alignItems: "center",
              borderWidth: 1,
              borderRadius: 6,
              borderColor: colors.line,
            }}
          >
            <View style={{ flexGrow: 1, minWidth: 0 }}>
              <N.Checkbox
                label={t(task.title)}
                checked={task.done}
                onChange={(done) => setTasks(toggleTask(tasks(), task.id, done))}
              />
            </View>
            <Button ghost compact onPress={() => setTasks(tasks().filter((item) => item.id !== task.id))}>
              Remove
            </Button>
          </View>
        )}
        emptyState={<Copy color={colors.muted}>Nothing here. Enjoy the clear list.</Copy>}
      />
    </View>
  );
}
