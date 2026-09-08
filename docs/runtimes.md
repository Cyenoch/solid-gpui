# Choose a runtime

Author JavaScript directly, or compile JSX/TSX with Vite. Runtime selection is
independent of this choice. Bun retains its native APIs; QuickJS requires
self-contained JS and Rust services. See [Vite integration](vite.md).

Solid GPUI offers three native runtime modes. They share the same Solid component
API, native controls, and Rust rendering engine. Choose where application
capabilities live and how you want to develop and deliver the application.

## Runtime comparison

|                                | External Bun                  | Embedded Bun                              | QuickJS                                                                             |
| ------------------------------ | ----------------------------- | ----------------------------------------- | ----------------------------------------------------------------------------------- |
| Main purpose                   | Rapid development             | Production packaging for Bun applications | Rust capabilities with JSX/TSX UI                                                   |
| JavaScript runs in             | A child process               | A dedicated thread inside the native app  | A dedicated thread inside the native app                                            |
| Files and networking           | Bun services or Rust commands | Bun services or Rust commands             | Rust commands                                                                       |
| Native UI                      | GPUI                          | GPUI                                      | GPUI                                                                                |
| Transport                      | Binary stdio                  | Native frame bridge                       | Native frame bridge                                                                 |
| End-user runtime               | A Bun executable              | Embedded Bun/JSC library                  | QuickJS embedded in the executable                                                  |
| Current platform qualification | Check the target desktop      | macOS embedding                           | Candidate packages for macOS, Linux, and Windows; desktop qualification is separate |

## Develop with external Bun

Use external Bun for the shortest iteration loop. In this repository:

```sh
bun run website:native:dev
```

Vite watches application dependencies and replaces the application inside the
existing native window. Keep state such as the current route and search query
through `captureState`; component-local signals and native editing caches remount.
See [hot reload](hot-reload.md).

Bun may own the application's business logic and services, or it may call Rust
Native Modules. Production code can use the same application composition with an
Embedded Bun entrypoint.

## Package a Bun application with Embedded Bun

Embedded Bun is the production direction when the application needs Bun services.
It uses `EmbeddedTransport`, and the host must enable the `embedded-bun` Cargo
feature. The runtime runs in the native process; it does not share mutable JS
objects with the GPUI thread.

The first build compiles a pinned, patched Bun/JSC library and can be substantially
more work than a UI rebuild. Reuse the supported build cache for later builds.
Embedding currently targets macOS. The website's existing final archive pipeline
uses QuickJS; do not mistake that pipeline for Embedded Bun release qualification.
See [distribution](distribution.md) and [runtime strategy](runtime-strategy.md).

## Keep application capabilities in Rust with QuickJS

Use QuickJS when Rust owns files, networking, domain operations, and long-running
work. JSX/TSX still owns signals, event handlers, routing, and presentation state.
Expose Rust operations through [generated Native Modules](rust-bridge.md).

QuickJS has timers, Promises, and the platform primitives needed by the native
router. It does not provide Bun, Node modules, a DOM, or network `fetch`. Bundle
JavaScript dependencies rather than loading external packages at runtime.

```tsx
import { mountApplication, Text } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";

mountApplication({
  transport: () => new EmbeddedTransport(),
  setup: () => ({ render: () => <Text>Hello from Rust and Solid</Text> }),
});
```

Build a production entry:

```sh
bun --bun vite build # solidGpui({ entry: "src/app.tsx", runtime: "quickjs" })
```

For development against the actual QuickJS engine, build your native host with
`quickjs` enabled and the
[interpreter development profile](hot-reload.md#application-build-configuration)
in your application's workspace-root manifest, then run:

```sh
bun --bun vite
```

The external development tool watches and bundles code. The host prepares a fresh
QuickJS VM and validates the replacement before switching the native application.
Rust services and native windows remain alive. Explicit captured state crosses
generations as bounded JSON data. Rust, protocol, and build configuration changes
require restarting the development command. See [the reload lifecycle](hot-reload.md).

## Performance and ownership

Embedded execution avoids the child-process pipe, but queueing and binary encoding
still have costs. Both embedded engines exchange owned frames with Rust; native
commands provide typed results without exposing GPUI handles to JavaScript.
Pressure pauses event-driven production until the native consumer catches up;
all pending queues remain bounded.

Changing runtime does not automatically improve scrolling or frame rate. GPUI
still performs native layout and painting, and some scrolling never enters JS.
Measure startup, total memory, JS computation, and input-to-present separately
using the [performance guide](performance-analysis.md).
