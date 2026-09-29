# Choose a runtime

Solid GPUI offers three native runtime modes sharing the same Solid component
API, native controls, and Rust rendering engine. Choose according to where your
application's services live and how you deliver it. JavaScript runs directly;
JSX/TSX uses [Vite](vite.md) with any runtime.

## Runtime comparison

| | External Bun | Embedded Bun | QuickJS |
| --- | --- | --- | --- |
| Main purpose | Rapid development | Single-executable Bun delivery | Rust services with a Solid UI |
| JavaScript runs in | A child process | A dedicated thread inside the app | A dedicated thread inside the app |
| Files and networking | Bun services or Rust commands | Bun services or Rust commands | Rust commands |
| Transport | Binary stdio | Native frame bridge | Native frame bridge |
| End-user runtime | A Bun executable | Bun/JSC linked into the executable | QuickJS linked into the executable |

Embedded Bun packaging is experimental. Check the authoritative
[platform status](distribution.md#platform-status-and-current-evidence) before
choosing it for delivery.

## Develop with external Bun

Use external Bun for rapid iteration, whether domain logic lives in Bun or Rust.
For your application, follow [Getting started](getting-started.md). In this
repository, run:

```sh
bun run website:native:dev
```

Vite replaces the application inside the existing native window. Preserve
explicit UI state with [`captureState`](capture-state.md); see
[hot reload](hot-reload.md) for activation and native rebuild behavior.

## Package a Bun application with Embedded Bun

Choose Embedded Bun when the delivered application needs Bun services. The
packager links GPUI, Bun/JSC, and the application's declared module graph into
one executable. The destination needs no separate Bun installation or JavaScript
directory.

Use the public CLI `solid-gpui embedded package` or
`packageEmbeddedApplication` from `@solid-gpui/vite/embedded`. Follow
[distribution](distribution.md#embedded-bun-static-applications) for the SDK
checkout, pinned serializer, Cargo inputs, assets, Workers, and qualification
requirements. [Runtime strategy](runtime-strategy.md) explains the native image
and session lifecycle.

## Keep application capabilities in Rust with QuickJS

Choose QuickJS when Rust owns files, networking, domain operations, and long-running
work. Solid still owns signals, event handlers, routing, and presentation state;
Rust services are exposed through [Native Modules](rust-bridge.md).

QuickJS supplies timers, Promises, and the platform primitives needed by the
native router. It has no Bun or Node modules, DOM, or network `fetch`. Bundle
JavaScript dependencies and select `runtime: "quickjs"` in the Vite plugin.
See [Vite integration](vite.md) for configuration and
[QuickJS application reload](hot-reload.md#quickjs-application-reload) for
development against the actual VM. Validate shared UI in QuickJS even if you
normally iterate under Bun.

## Performance and ownership

Every runtime exchanges owned frames with Rust; GPUI owns layout, painting, and
native input. Embedding removes the child-process pipe but retains queues,
encoding, and JS execution costs. Runtime choice alone does not improve scrolling
or frame rate. Use the [performance workflow](performance-analysis.md) to measure
startup, memory, JS work, and input-to-present latency separately.
