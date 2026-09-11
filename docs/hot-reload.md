# Native development and hot reload

External Bun is the rapid iteration runtime. Embedded Bun is the intended
production packaging runtime for Bun-based applications; QuickJS is the UI
runtime for Rust-led applications. See [runtime strategy](runtime-strategy.md)
for the distinction between these roles and current delivery support.

Vite 8 handles the module graph, file watching, HMR, and production bundling.
The plugin compiles universal JSX with the official Oxc-based
`@solidjs/compiler` 2.0.0-rc.6 and lowers TypeScript with `oxc-transform` 0.148.0.
The application runtime remains Solid 1.9.15. In the external Bun workflow,
Bun executes Vite's ModuleRunner and the application JavaScript in the
host-owned child. Vite is the only application bundler; Bun and QuickJS execute
JavaScript. See [Vite integration](vite.md) for direct JS, JSX/TSX, native modules,
and Rust-owned startup. The GPUI host keeps its native window and receives
a new epoch's Snapshot over the existing stdio connection. This is a native
application module environment; it does not require HTML, a DOM, or a WebView.

## Repository development

`examples/website` owns the shared website and native application. Its desktop
entrypoint and Vite configuration run Components and Showcase in a native window.

- `bun run website:native:dev`: build the required packages and start the host with Vite/Bun.
- `bun run --cwd examples/website build:native`: produce `examples/website/dist-native/main.native.js`.
- `target/debug/website-host bun --conditions=browser examples/website/dist-native/main.native.js`: run the bundle after building the host.

`website:native:dev` and `quickjs:dev` prepare the JavaScript tooling first and
leave native compilation to Vite's managed session, so an initial Rust error
does not terminate the watcher before it starts.

Saving application code remounts it in the existing window. With `native`
configured, Rust changes automatically rebuild the host and its bindings, then
open a fresh application session. Vite configuration changes restart its
environment. Protocol schema changes still require the protocol generation and
package build workflow before restarting development.

## Application integration

Install `@solid-gpui/core` and Vite 8, then create a configuration:

```ts
import { defineConfig } from "vite";
import { solidGpui } from "@solid-gpui/vite";

export default defineConfig({
  plugins: [solidGpui({ entry: "src/app.tsx" })],
});
```

Run `bun --bun vite` to launch the configured host and Bun module runner. Use
`native` to build an application-owned Rust host and generate `#native`, or `host`
to select an existing executable. Rust applications can launch the same config
through `solid_gpui::runtime::vite::Vite`. See [Vite integration](vite.md).

```tsx
import { mountApplication, Text } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { StdioTransport } from "@solid-gpui/core/stdio";

mountApplication<number>({
  transport: () => new StdioTransport(),
  hotKey: import.meta.hot ? import.meta.url : undefined,
  setup(previous = 0) {
    const [count] = createSignal(previous);
    return {
      render: () => <Text>{count()}</Text>,
      captureState: () => count(),
    };
  },
});
```

The plugin injects HMR acceptance into the configured entrypoint. Add
`/// <reference types="vite/client" />` to declare `import.meta.hot`.
For development aliases to library source, see
`examples/website/vite.native.config.ts`; applications usually use package exports.

## Managed development sessions

When Vite launches the host, the development server survives compilation,
initial application loading, and host failures. Diagnostics remain in the
terminal; fix the error and save a source file to start a fresh session. Closing
the application window also leaves the watcher available for the next edit.
Use Ctrl+C to stop development. Failed sessions are not restarted in a loop.

Configure `native` to watch `.rs` files (including `build.rs`), Cargo manifests
and lockfiles, and workspace `.cargo/config` or `.cargo/config.toml`. Cargo
metadata includes local path dependencies outside the frontend directory. A
native change stops the previous runtime before building and exporting bindings;
new bindings are never hot-updated into the previous host. Rapid edits supersede
the pending attempt. Shutting down also cancels Cargo's compiler and build-script
processes. Changes to build-script asset inputs are not automatically watched;
save a Rust source file to request that rebuild.
Cargo's resolved output directory is excluded from input watching, including
custom target directories inside a source package.

`#native` and `@solid-gpui/core/components` resolve to the configured host's
generated bindings. Motion imports use that same component catalog. Do not edit
generated bindings or retain a separate component alias pointing at an older SDK.
A host rebuild closes and reopens native windows; Rust services, component state,
and HMR checkpoints do not survive a host process replacement.

An explicit `host: { command }` remains an externally built executable: Vite can
restart it after a source edit but does not infer a Cargo project. Direct
Rust-owned startup, `host: false`, and caller-provided Vite environments keep
their existing lifecycle ownership. Start Vite with `native` when it should own
native rebuilds and recovery. Production builds and direct host launches still
exit on errors.

The host continues to reject invalid commits and terminate the affected runtime.
Subsequent Patches may depend on a rejected revision, so recovery starts a fresh
session rather than dropping a frame. See [native contract diagnostics](troubleshooting.md#native-contract-mismatch).

## State and failure boundaries

Start with [Preserve UI state with captureState](capture-state.md) for a complete
application example and guidance on restoring forms after native data loads.

Reloading remounts the application. It does not automatically preserve each
component's signals. Under Bun, `captureState` returns structured-cloneable data
that the next generation receives in `setup`. QuickJS uses the stricter
[JSON contract](#captured-state). Capture explicit application data such as the
route, selected item, or window size; component state, native input, and scroll
caches are recreated. Do not retain Solid owners, Root or router instances, functions, or
native resources across generations.

After the candidate's setup, render, and first-frame preparation succeed, the
previous owner and root are disposed and a new epoch is published for the same
surface ID. Syntax errors and synchronous setup/render errors leave the previous
page mounted; fixing the error and saving again retries the replacement. With a
Vite-managed host, an initial failure leaves development watching for another edit
even when no previous page exists.

`onMount` runs after commit. Its errors and asynchronous side effects are outside
the rollback boundary; I/O failures also have no rollback guarantee. Top-level
application side effects occur outside candidate preparation. Create resources
inside `setup` or components and release them with `onCleanup`. Late asynchronous
results should check disposal before recreating timers.

The required transport factory creates the application-owned connection once.
It survives reloads and closes on final disposal, including when it is a custom
transport. Replacing a generation invalidates its previous handle
for the same `hotKey`. Reopening a retired native surface after manual disposal
is outside the HMR lifecycle.

## File-based routing

For file-based routes, add `solidGpuiRouter()` from `@solid-gpui/router/vite`
before `solidGpui()`. It generates the typed route tree before the application is
loaded and updates it when routes are added, edited, renamed, or removed. See the
[Router guide](router.md) for setup and navigation patterns.

## Production bundles

Configure `solidGpui({ entry: "src/app.tsx", runtime: "bun" })` and build with Vite:

```sh
bun --bun vite build
```

A Bun entrypoint uses `StdioTransport`, as above. Run its bundle with an existing
host and `bun --conditions=browser dist/app.js`. Embedded Bun uses `EmbeddedTransport` and requires the `embedded-bun` feature.
Give it a separate entrypoint importing the shared application composition.

For a Rust-led application using QuickJS, create a separate entrypoint with the
embedded transport and shared application components:

```tsx
import { mountApplication } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";
import { App } from "./app";

mountApplication({
  transport: () => new EmbeddedTransport(),
  setup: () => ({ render: () => <App /> }),
});
```

Build a self-contained ESM bundle, then launch the QuickJS-enabled host:

```sh
bun --bun vite build # solidGpui({ entry: "src/quickjs.tsx", runtime: "quickjs" })
cargo run -p solid-gpui --features quickjs --bin solid-gpui-host -- --runtime quickjs dist/app.js
```

The generic host command runs core host components. Applications using custom
Native Modules or the optional gpui-component integration launch their own host
with those modules/features enabled.

QuickJS has no ambient Bun/Node services and does not load an application's
external packages at runtime. Bundle JavaScript dependencies and put services
such as files and networking in Rust Native Modules. Timers and native command
Promises support UI work; this is not a browser or a full Bun environment.
The build initializes URL/search, event, and cancellation primitives needed by
the native router. Its `Response` is deliberately bodyless and exists for route
redirect metadata; non-null bodies are rejected. It does not provide Fetch body
methods or network `fetch`. `AbortSignal.any` combines local cancellation sources,
including `AbortSignal.timeout`, and preserves the first cancellation reason.
Nested combinations settle before source listeners run, so reentrant cancellation
and stopped event propagation cannot change their result. Pending combinations
are retained by their sources and unlinked from every source when cancelled;
abort operation-owned controllers during cleanup instead of accumulating pending
combinations on an application-wide controller.
Vite module HMR executes under external Bun. QuickJS development uses the
separate whole-application reload workflow below; production still executes one
built entry without development tooling.

## QuickJS application reload

Use the actual QuickJS engine while iterating on a Rust-led UI:

```sh
bun run quickjs:dev # Counter demo in the actual QuickJS engine
```

For another application, configure `solidGpui({ entry, runtime: "quickjs", native })`
with a QuickJS-enabled host and run `bun --bun vite`. The entry uses
`EmbeddedTransport` and `mountApplication`. Vite's build watcher applies the same
plugins, aliases, and QuickJS build policy as production. A loopback development
connection carries bounded bundles separately from the UI protocol. No browser
client, WebSocket, fetch, or ModuleRunner is installed inside QuickJS.

### Application build configuration

In the consuming application's **workspace-root** `Cargo.toml`, keep the
interpreter optimized during development:

```toml
[profile.dev.package.rquickjs-sys]
opt-level = 3
```

This repository already sets that profile, but Cargo ignores profile settings
in dependencies. An unoptimized interpreter can exhaust the same 2 MiB JS stack
on a nested route that succeeds with the optimized interpreter. See
[QuickJS stack diagnosis](troubleshooting.md#quickjs-overflows-the-stack-on-a-nested-route)
before changing layouts or increasing the runtime's stack limit. Rebuild and
restart the native host after changing the profile.

After changing interpreter dependencies, verify that this profile still matches
the package resolved in the consuming workspace's Cargo.lock. A Cargo warning
that a profile package spec matches no packages means that override is inactive.
Vite can still report `ready` while the UI overflows during initialization; see
[blank QuickJS windows](troubleshooting.md#vite-is-ready-but-the-quickjs-window-is-blank).

### Generation lifecycle

A successful build creates a fresh VM while keeping the Rust host, windows, and
services alive. The old VM pauses at a microtask checkpoint and exports explicit
`captureState` data. The host validates every candidate Surface and application
configuration before switching routing on the foreground thread. Old epoch input
and replies cannot invoke the new generation. Native caches and component-local
signals remount. Current window observations are sent again after replacement.
Lifecycle requests use a bounded FIFO: a later capture cannot overwrite an
activation that the VM has not processed yet. Exhausting the control queue is
reported explicitly rather than silently dropping a lifecycle step.

### Captured state

QuickJS captured state must be acyclic JSON data: plain objects, dense arrays,
strings, booleans, finite numbers, and null. Accessors, hidden properties, symbols,
functions, non-plain objects, sparse arrays, negative zero, and non-finite numbers
are rejected. The state budget is 1 MiB, 100,000 visited values, and 64 nesting
levels. An absent captured value is distinct from an explicit null. Bun's
same-VM HMR still uses its structured-clone state contract.

Return a dedicated state object rather than the entire live session. For example,
if a session's `userId` can be `undefined`, choose its persisted meaning explicitly:

```ts
type ReloadState = { session: { userId: string | null } };

const captureState = (): ReloadState => ({
  session: { userId: session.userId ?? null },
});
```

Use `null` when it means “no user”, or omit an optional object property when it
means “not supplied”. Array elements must contain JSON values; neither holes nor
`undefined` elements are accepted. Returning `undefined` from `captureState`
itself means no captured state, and the next `setup` receives `undefined`;
returning `null` preserves `null`. Do not use a `JSON.stringify`/`JSON.parse`
round trip to sanitize a live session: it silently drops or changes unsupported
values before validation can report them.

Validation errors identify the field, for example
`$.state[0].session.userId: undefined is not JSON`. `state[0]` is the captured
application value inside the handoff envelope. Capture runs in the **old VM**,
before the candidate is created. A capture failure keeps that VM interactive;
fix its live state or restart after correcting `captureState`. Editing only the
candidate cannot replace an old capture function that always returns invalid data.

### Activation and recovery

Candidate preparation accepts initial Snapshots and application configuration.
Perform native effects in the application's `onMount`, after activation. The
candidate cannot run arbitrary native commands while being staged. A valid
loading view can be the first frame; route loading may finish after activation.
Preparation has a three-second deadline. A candidate that fails or becomes stale
is discarded, and the previous generation resumes. Rebuild requests are latest
wins; only one pending bundle and one candidate are retained.

A candidate must describe exactly the currently open persistent Surface set. Auxiliary roots
created during setup must use stable explicit Surface IDs and share the connection;
their default epoch comes from the host generation. Register their cleanup in the
application owner and include their UI state in `captureState`. Opening/closing
native windows or changing the window set during staging rejects that candidate
instead of partially replacing the application. Zero-window keep-alive reload
preserves the application connection and activation acknowledgement sequence.

`SystemPopover` Surfaces are transient: the old generation closes them, and the
new controlled state recreates them after activation. Their native IDs are not
part of the candidate's persistent Surface set. Keep shared form state above the
content factory and include it in `captureState`.

After activation, asynchronous errors and native side effects are not rolled
back. A failed development VM leaves the last native tree visible; a later edit
can recover using the last captured UI state and current host activation metadata.
That state is a checkpoint, not a promise to preserve changes made after it.
Rust/native-contract changes replace the host when `native` is configured;
Vite configuration changes restart development. Neither path preserves a VM
checkpoint across host replacement.

The `applied` diagnostic confirms activation, not completion of asynchronous
route rendering. Verify the restored page's actual content and an interaction
after activation; a loading Snapshot or persistent navigation label is insufficient.

### Verify application reload

Run acceptance checks with the consuming application's native build profile and
actual QuickJS host. Rebuild the TypeScript packages first when the application
resolves their compiled `dist` exports.

1. Start directly at a nested route and verify its page-specific content and an
   interaction. Also navigate to it from another page; these initialization paths
   can create different amounts of the component tree at once.
2. Change data covered by `captureState`, edit a TSX dependency, and wait for the
   restored page to finish loading. Check the route, captured values, and an
   interaction in the new generation.
3. Introduce a syntax error or synchronous preparation error. Verify that the
   previous generation remains interactive, then fix the error and reload again.
   Repeat saves to exercise replacement ordering and recovery.
4. Inspect diagnostics after activation for asynchronous route or native-command
   failures. Validate these outcomes separately from the `applied` acknowledgement.

## External Bun operational constraints

- Solid's universal renderer requires the client reactive implementation. Configure `browser` in both Vite SSR `conditions` and `externalConditions`, and launch Bun with that condition.
- Use Vite ModuleRunner HMR. Setting `server.hmr: false` also disables server-side HMR. Do not layer Bun `--hot` or a browser HMR client onto it.
- Reserve stdout for length-prefixed protocol bytes. Send Vite and application diagnostics to stderr with `console.error`.
- Exclude the native `target` directory from watching. Disable automatic dependency discovery, which has no HTML entrypoint to inspect and could scan embedded Bun vendor test data.
- Integration verification must save a TSX dependency, introduce syntax and render failures, recover, decode the entire stdout stream, and check epochs, explicit state, and owner cleanup. Transform-only tests cannot establish working HMR.

Key tests: `scripts/hot-reload.test.ts` and
`packages/solid-gpui/tests/application.test.ts`.
References: [Vite framework Environment API](https://vite.dev/guide/api-environment-frameworks)
and [Vite runtime API](https://vite.dev/guide/api-environment-runtimes).
