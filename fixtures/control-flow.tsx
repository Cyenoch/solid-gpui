/**
 * Compile fixture for the public type contract of Solid control flow over native
 * JSX. `package-typecheck` compiles this file and never mounts it; the
 * `@ts-expect-error` cases pin the native element boundary.
 */
import { Text, View, type SolidChild } from "@solid-gpui/core";
import { For, Index, Match, Show, Switch, createSignal } from "@solid-gpui/core/runtime";

export function ControlFlowFixture(props: { rows: string[]; selected: string | null; fallback: SolidChild }) {
  const [dense, setDense] = createSignal(false);
  return (
    <View style={{ flexDirection: "column", gap: 8 }}>
      <For each={props.rows} fallback={<Text>No settings</Text>}>
        {(row, index) => (
          <Text onPress={() => setDense((value) => !value)}>
            {index()}: {row}
          </Text>
        )}
      </For>
      <For each={props.rows}>{(row) => <Text>{row}</Text>}</For>
      <Index each={props.rows}>{(row, index) => <Text>{`${index} ${row()}`}</Text>}</Index>
      <Show when={props.selected} fallback={props.fallback}>
        {(selected) => <Text>{selected()}</Text>}
      </Show>
      <Show when={props.selected} keyed>
        {(selected) => <Text>{selected}</Text>}
      </Show>
      <Switch fallback={<Text>Unknown</Text>}>
        <Match when={props.rows.length > 0}>
          <Text>{dense() ? "Dense" : "Comfortable"}</Text>
        </Match>
        <Match when={props.selected} keyed>
          {(selected) => <Text>{selected}</Text>}
        </Match>
      </Switch>
      {/* @ts-expect-error item types come from `each`, not from a DOM element */}
      <For each={props.rows}>{(row) => <Text>{row.missing}</Text>}</For>
      {/* @ts-expect-error native children are renderer children, never DOM nodes */}
      <Show when={props.selected} fallback={document.createElement("div")}>
        <Text>Selected</Text>
      </Show>
      {/* @ts-expect-error unknown native props are rejected before the host sees them */}
      <Text unknownProp="x">Text</Text>
    </View>
  );
}
