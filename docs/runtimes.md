# Choose a runtime

Author JavaScript directly, or compile JSX/TSX with Vite. Runtime selection is
independent of this choice. Bun retains its native APIs; QuickJS requires
self-contained JS and Rust services. See [Vite integration](vite.md).

Solid GPUI offers three native runtime modes. They share the same Solid component
API, native controls, and Rust rendering engine. Choose where application
capabilities live and how you want to develop and deliver the application.

## Runtime comparison

|                                | External Bun                  | Embedded Bun                                        | QuickJS                                           |
| ------------------------------ | ----------------------------- | --------------------------------------------------- | ------------------------------------------------- |
| Main purpose                   | Rapid development             | Production packaging for Bun applications           | Rust capabilities with JSX/TSX UI                 |
| JavaScript runs in             | A child process               | A dedicated thread inside the native app            | A dedicated thread inside the native app          |
| Files and networking           | Bun services or Rust commands | Bun services or Rust commands                       | Rust commands                                     |
| Native UI                      | GPUI                          | GPUI                                                | GPUI                                              |
| Transport                      | Binary stdio                  | Native frame bridge                                 | Native frame bridge                               |
| End-user runtime               | A Bun executable              | One self-contained executable carrying Bun/JSC      | QuickJS embedded in the executable                |

Packaging commands, prerequisites, and per-platform status have one source of
truth: [distribution](distribution.md#embedded-bun-static-applications) and its
[platform status](distribution.md#platform-status-and-current-evidence). Windows
packaging is experimental and Linux stops at native preparation, so a passing
local build is not release acceptance.

## Develop with external Bun

Use external Bun for the shortest iteration loop. In this repository:

```sh
bun run website:native:dev
```

For your own application, follow [Getting started](getting-started.md): install the
packages, run `bun run generate`, then develop with `bun --bun vite`. Vite watches
application dependencies and replaces the application inside the
existing native window. Keep state such as the current route and search query
through `captureState`; component-local signals and native editing caches remount.
See [hot reload](hot-reload.md).

Bun may own the application's business logic and services, or it may call Rust
Native Modules. The packaged application keeps the same composition.

## Package a Bun application with Embedded Bun

Embedded Bun is the delivery path when the application needs Bun services. It
produces one application executable containing the GPUI host, the Bun/JSC
runtime, and the serialized application with its declared assets and Worker
entries. The product contract has no sidecars: it needs no Bun or Node
installation and no JavaScript tree or `node_modules` beside the executable.

```sh
solid-gpui embedded package \
  --entry <Vite-built JS> \
  --bun <pinned Bun executable> \
  --output <application> \
  --manifest <application Cargo.toml> --package <application crate>
```

`solid-gpui embedded package` comes from `@solid-gpui/vite`. The library form is
`packageEmbeddedApplication({ sdkRoot, entry, output, bun, application, assets, workers, ... })`
from `@solid-gpui/vite/embedded`; it takes an explicit `sdkRoot` naming the SDK
checkout that owns the pinned Bun/Rust backend, so a consumer never imports
repository-private files or copies the toolchain. This repository runs the same
driver as `bun run embedded:package`. `application.manifest`/`application.package`
select an application-owned Cargo manifest and crate, `application.features` adds
its features, and `application.main` replaces the generated Rust entry. The result
reports the executable and its digests, the Rust triple and graph target, the
typed `entry` identity with `role: "application"`, and every worker identity, so a
packaging script never re-derives them.

The packager takes the Vite-built entry, serializes it with a Bun executable
matching the pinned revision, and links it with a native Bun graph built against
prebuilt WebKit/JSC archives. Vite remains the only application compiler, and Bun
and Node built-ins stay runtime imports. The pinned serializer is a build-time
tool: it is never shipped and never needed on the destination machine. The first
build compiles Bun's native graph and is substantially heavier than a UI rebuild;
prebuilt WebKit/JSC archives avoid rebuilding JavaScriptCore, and a supported
build cache plus the extracted native manifest avoid repeating the work.

Declare application Workers and resources with `--workers` and `--assets`.
Arbitrary paths opened on the destination filesystem are not automatically
included; embed those application resources or provision them explicitly.

The host starts such an application with
`EmbeddedBunAdapter::start_packaged(entry)`, where `entry` is the virtual graph
key the packager emitted — distinct from `start(path)`, which still loads a file
from disk for development. A packaged session that cannot find its graph fails
closed instead of evaluating some other file. The application uses
`EmbeddedTransport` and never shares mutable JS objects with the GPUI thread. An
application reports its own exit code with `completeEmbedded(code)` from
`@solid-gpui/core/embedded` (guarded by `supportsEmbeddedCompletion()`), which the
host exposes as `EmbeddedBunAdapter::result()`; the full code survives even though
the VM exit status is only a byte.

Embedded packaging is experimental. The
[platform status table](distribution.md#platform-status-and-current-evidence) is
the only qualification evidence: no target is currently claimed as supported or
verified, and an unsupported triple fails early with the supported matrix.
`--check-bundle` is a startup smoke test only: it starts two sessions and
requires their initial Snapshots. It covers no input, assets, Workers, or
graphics, and passing it is not platform qualification. Architecture, deployment
restrictions, and remaining work are in
[runtime strategy](runtime-strategy.md).

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
all pending queues remain bounded. Loading the application from the executable
image does not skip parsing or evaluating it, and it does not make the runtime
free.

Changing runtime does not automatically improve scrolling or frame rate. GPUI
still performs native layout and painting, and some scrolling never enters JS.
Measure startup, total memory, JS computation, and input-to-present separately
using the [performance guide](performance-analysis.md).
