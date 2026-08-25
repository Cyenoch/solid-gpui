# @react-gpui/core

`@react-gpui/core` is a React custom renderer for the Rust GPUI host. React components, hooks, context, fragments, and JavaScript event closures keep their normal React semantics; each React commit becomes one atomic Commit Batch: a Snapshot bootstrap for a new surface epoch or a Patch afterward.

## JSX

```tsx
import { useCallback, useState } from "react";
import { Pressable, StyleSheet, Text, View, createRoot } from "@react-gpui/core";

const styles = StyleSheet.create({
  root: { flexDirection: "column", gap: 8, padding: 16 },
  button: { padding: 8, backgroundColor: "#2d6cdf" },
  label: { color: "#ffffff" },
});

function Counter() {
  const [count, setCount] = useState(0);
  const increment = useCallback(() => setCount((current) => current + 1), []);
  return (
    <View style={styles.root}>
      <Text>Count: {count}</Text>
      <Pressable style={styles.button} onPress={increment}>
        <Text style={styles.label}>Increment</Text>
      </Pressable>
    </View>
  );
}

const root = createRoot(transport, { surfaceId: 1, epoch: 1 });
root.render(<Counter />);
```

`View`, `Text`, and `Pressable` accept `ref` values that resolve to typed host nodes. `Pressable` accepts `onPress`; callbacks stay in JavaScript and receive a semantic press notification. There is intentionally no synchronous `preventDefault()` because the transport cannot cancel a native action synchronously.

## Styles

`StyleSheet.create` is a validated, frozen style-recipe helper; styles are not CSS. V3 fields are:

- `width`, `height`, `flexGrow`, `padding`, and `gap` — finite, non-negative numbers.
- `flexDirection` — `"row"` or `"column"`.
- `backgroundColor` and `color` — `#RRGGBB` or `#RRGGBBAA`, packed as RGBA `u32` values on the wire.
- `opacity` — a number from `0` through `1`.
- `transition` — `{ durationMs, delayMs?, easing?, properties?, onComplete? }`; easing defaults to `"easeInOut"` and completion callbacks remain JavaScript-side.

Unknown fields, invalid colors, non-finite values, negative numeric fields, invalid easing values, and duplicate transition properties are rejected.

## Transport and framing

`createRoot` accepts a `Transport` with two directions:

- `submit(frame)` sends a complete renderer-to-host frame. The frame is a four-byte little-endian payload length followed by MessagePack bytes.
- `onData(listener)` delivers host-to-renderer bytes. Input may be fragmented or coalesced; the package incrementally decodes frames and enforces the 16 MiB maximum frame size.

Snapshots use `[3,1,surfaceId,epoch,baseRevision,revision,nodes]` for bootstrap; later commits use Patch `[3,3,surfaceId,epoch,baseRevision,revision,operations]` with positional Create, Update, Move, and Delete operations. Node records are `[id,parentId,index,kind,style,text,listenerId,hostProperties,accessibility]`; `hostProperties` is a tagged TextInput or VirtualList tuple. Events use exactly `[3,2,surface,epoch,revision,sequence,node,listener,eventType,payload|null]`; payload tags are TextInput `1`, CommandResult `2`, VisibleRange `3`, and AnimationComplete `4`. Node and listener IDs are `u32` values.

## Commit and event semantics

A completed React commit produces exactly one atomic Commit Batch: a complete Snapshot during bootstrap or one incremental Patch afterward, rather than one transport call per host mutation or prop. Listener IDs remain stable while a host node remains mounted; changing a callback function updates only the JavaScript callback table, while 0↔nonzero listener transitions update the native listener field. TextInput native edits are acknowledged with editSeq and preserve uncontrolled defaults. Press, TextInput, VisibleRange, and AnimationComplete events are semantic notifications, not cancellable browser events.

`TextInput` is controlled with `value`/`onChangeText` or initialized once with `defaultValue`. It also supports `placeholder`, `onSelectionChange`, `onFocus`, `onBlur`, `multiline`, `disabled`, and accessibility props; native edits carry UTF-16 selection/marked ranges and editSeq acknowledgements.

## VirtualList

`VirtualList<T>` keeps its `data`, `itemKey`, and `renderItem` closures in JavaScript:

```tsx
<VirtualList
  data={rows}
  itemKey={(row) => row.id}
  renderItem={(row) => <Text>{row.title}</Text>}
  estimatedItemSize={28}
  overscan={3}
  initialNumToRender={12}
  onEndReached={loadMore}
/>
```

Only the committed visible range is reconciled into host rows, so a 100,000-item array does not produce 100,000 host nodes. A `VirtualListHandle` ref exposes Promise-returning `scrollToIndex(index)` and `scrollToEnd()` methods. Native GPUI uses one persistent fixed-height `uniform_list` scroll handle per node and reports the actual next-frame range with overscan.
The list must have a finite viewport height (for example `style={{ height: 400 }}`) or be inside a parent that supplies a bounded height; `uniform_list` uses that bound to render only visible rows.
## Native animation

`opacity` and `backgroundColor` can transition natively without per-frame JavaScript commits:

```tsx
<View
  style={{
    opacity: visible ? 1 : 0,
    backgroundColor: "#2d6cdf",
    transition: {
      durationMs: 180,
      delayMs: 20,
      easing: "easeOut",
      properties: ["opacity", "backgroundColor"],
      onComplete: (generation) => console.log("completed", generation),
    },
  }}
/>
```

GPUI retains one animation state per node, samples easing and RGBA values on frame ticks, retargets from the current sample, cancels deleted nodes, and honors reduced motion. Completion is emitted once as the tagged AnimationComplete event.


## Local commands

From this directory:

```sh
bun install --frozen-lockfile
bunx tsc --noEmit
bun test
bun run examples/counter.tsx
```

The counter example is intended to run as the renderer child of `react-gpui-host` from the repository root:

```sh
cargo run -p react-gpui-host -- bun run packages/react-gpui/examples/counter.tsx
```
