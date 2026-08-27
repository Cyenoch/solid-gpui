# @react-gpui/dev

`@react-gpui/dev` provides the Fast Refresh transform, stable component-family proxies, and last-good-tree refresh session used while developing React GPUI entries.

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](./LICENSE)

[Getting started](../../docs/getting-started.md) · [Protocol](../../docs/protocol.md) · [Contributing](../../CONTRIBUTING.md)

## Public API

```ts
import {
  FastRefreshSession,
  createFastRefreshSession,
  importWithRefresh,
  installFastRefreshTransform,
  performReactRefresh,
  render,
  transformRefreshSource,
  watchModule,
  type CommandResultOptions,
  type RefreshLoader,
  type RefreshModule,
  type RefreshResult,
  type RefreshRoot,
  type RenderResult,
  type TestNodeHandle,
} from "@react-gpui/dev";
```

`installFastRefreshTransform(sourceRoot)` installs the Bun loader for the selected source tree. `FastRefreshSession` keeps a stable root mounted while compatible modules update; failed transforms leave the last-good tree in place. The package requires React 19 and runs under Bun 1.4 or newer. The headless testing API keeps `@react-gpui/core` external and expects the consumer test project to provide it.

## Testing your app

### Headless component tests

`render` creates a real core `Root` backed by `MemoryTransport`; it does not
open a native window. The returned `frames` are complete length-prefixed
submissions, while `commits()` decodes their MessagePack payloads for
positional assertions. Use it from a Bun test and inject native events through
the returned helpers:

```tsx
import React from "react";
import { expect, test } from "bun:test";
import { Pressable, Text } from "@react-gpui/core";
import { render } from "@react-gpui/dev";

test("a button can be rendered and pressed without a display", () => {
  let presses = 0;
  function Counter() {
    return (
      <Pressable accessibilityLabel="increment" onPress={() => (presses += 1)}>
        <Text>increment</Text>
      </Pressable>
    );
  }

  const testView = render(<Counter />, { surfaceId: 7, epoch: 1 });
  const button = testView.node("Pressable", (node) => node.accessibility?.[1] === "increment");
  testView.press(button);
  expect(presses).toBe(1);
  expect(testView.commits()[0]).toEqual(expect.arrayContaining([3, 1, 7, 1]));
  testView.unmount();
});
```

Use `node(kind, predicate?)` to locate the latest retained Host Node. Event
helpers `press`, `key`, `input`, `submit`, and `visibleRange` inject real
protocol frames through the core `MemoryTransport`; `commandResult(requestId,
options?)` completes a captured command and can infer its command/node fields.
For unusual events, `dispatchFrame(rawEvent)` accepts one complete framed
event. `unmount()` is idempotent. Render errors are not swallowed: a consumer
Error Boundary can recover, while an unbounded render error is thrown to the
test.

For advanced assertions, `RenderOptions` controls the explicit surface/epoch
and window callbacks; `RenderResult` exposes the real `Root`, frames, decoded
commits, and raw dispatch. `TestNode`, `TestNodeHandle`, and
`TestNodePredicate` provide typed retained-node predicates, while
`CommandResultOptions` describes inferred command acknowledgements.

The testing API is intentionally a development dependency. It shares the
consumer's core and React installation rather than bundling another renderer
runtime.

## Example

[`examples/counter.tsx`](examples/counter.tsx) is a minimal Fast Refresh entry
that can be used while developing a core renderer surface.

## Local development

From this directory:

```sh
bun install --frozen-lockfile
bun run format
bun run typecheck
bun run test
bun run build
bun pm pack --dry-run
```

The package build writes only `dist/index.js` and declaration files used by the package export. Source, tests, and examples remain development-only files.
