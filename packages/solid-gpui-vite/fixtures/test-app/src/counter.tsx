import { Text, View } from "@solid-gpui/core";
import { For, Show, createSignal, onMount } from "@solid-gpui/core/runtime";

export interface CounterProps {
  /** Owned by the caller, so a host or a test drives the very signal the renderer reads. */
  readonly count: () => number;
  readonly history?: readonly number[];
}

export function Counter(props: CounterProps) {
  const [status, setStatus] = createSignal("starting");
  onMount(() => setStatus("ready"));
  return (
    <View style={{ gap: 4, padding: 8 }}>
      <Text>{`count ${props.count()}`}</Text>
      <Show when={props.count() >= 42} fallback={<Text>waiting</Text>}>
        <Text>{status()}</Text>
      </Show>
      <For each={props.history ?? []}>{(value: number) => <Text>{`seen ${value}`}</Text>}</For>
    </View>
  );
}
