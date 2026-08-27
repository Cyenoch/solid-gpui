# TestApp facade for consumer behavior tests

## Evidence and audit

- `render` creates `MemoryTransport`, mounts synchronously, and exposes decoded
  commits plus complete submitted frames (`packages/react-gpui-dev/src/testing.ts:399-420`).
- The current node locator requires a host kind and predicate, then reports only
  `testing node not found: <kind>` (`packages/react-gpui-dev/src/testing.ts:427-434`).
- Press, key, text change, submit, and virtual-list visible-range notifications
  already have helpers that build protocol frames (`packages/react-gpui-dev/src/testing.ts:436-462`).
- `dispatchFrame` is the only escape hatch for events without helpers
  (`packages/react-gpui-dev/src/testing.ts:493-495`).
- Therefore consumers currently need raw frame knowledge for hover (event 11),
  focus/blur (events 4/5), pointer/multi-click (event 10), scroll (event 12),
  drag over/drop (event 20), pointer-down-outside (event 22), and other native
  notifications. Press/key/input/submit/visible-range do not require raw frames.

## Interface

`renderTestApp(element, options?)` returns a `TestApp` wrapping the existing
`render` result. Labels are the app-level identity: `app.node(labelOrPredicate)`
locates the latest retained node by accessibility label or predicate, and
`app.text(value)` locates exact rendered text on a RawText node or its Text
parent. Both throw an explicit locator error naming the query when no node
matches.

The facade exposes `root`, `frames`, `commits`, `unmount`, and the existing raw
`dispatchFrame` escape hatch. `press`, `hover`, `key`, `input`, `submit`,
`scroll`, `dragOver`, `drop`, `pointerDownOutside`, `focus`, `blur`, and
`commandResult` resolve a locator as needed, construct the canonical protocol
payload, dispatch through the wrapped `MemoryTransport`, and return the latest
decoded commit. `key` defaults modifiers to `[]`; scroll defaults to pixel
units at origin; focus/blur inject renderer-side native-notification events
appropriate to the node (including text-input state), not imperative native
focus commands. Command results infer the most recent captured request when no
request id is supplied.

This is ergonomics over the real seam, not a fake event system: every event
method reaches the same `Root` receive/dispatch path through `MemoryTransport`
that production uses after host decoding.

## Honest boundary

TypeScript `TestApp` assertions cover renderer semantics: state transitions,
patches, retained-node lookup, and event routing. Host-owned semantics cannot be
proven by this facade, including drag preview geometry, painted quads, hit
testing, and real wheel scrolling. Those assertions remain display-backed in
Rust `HeadlessSurface`/VisualTestContext suites; the README states this split
rather than simulating host behavior.

## Key tests and docs

- A single realistic flow filters a list, presses the filtered item, and checks
  the resulting state patch through `TestApp`.
- Locator coverage checks accessibility labels, exact text, and the explicit
  not-found error contract.
- The dev README gets a Bun-test recipe using the new facade and a boundary
  paragraph; getting-started only points to that section.
- Regenerate `fixtures/api-surface.dev.txt` after exports change and verify the
  generator is idempotent.
