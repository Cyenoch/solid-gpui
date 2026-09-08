export interface Task {
  id: number;
  title: string;
  done: boolean;
}
export function addTask(tasks: readonly Task[], title: string): Task[] {
  const trimmed = title.trim();
  if (!trimmed) return [...tasks];
  return [...tasks, { id: Math.max(0, ...tasks.map((task) => task.id)) + 1, title: trimmed, done: false }];
}
export function toggleTask(tasks: readonly Task[], id: number, done: boolean): Task[] {
  return tasks.map((task) => (task.id === id ? { ...task, done } : task));
}
