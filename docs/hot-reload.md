# Native development with Vite and Bun

Vite 8 handles the module graph, file watching, HMR, and production bundling.
The plugin compiles universal JSX with the official Oxc-based
`@solidjs/compiler` 2.0.0-rc.6 and lowers TypeScript with `oxc-transform` 0.148.0.
The application runtime remains Solid 1.9.15. Bun executes Vite's RunnableDevEnvironment/ModuleRunner
and the application JavaScript. The GPUI host keeps its native window and receives
a new epoch's Snapshot over the existing stdio connection. This is a native
application module environment; it does not require HTML, a DOM, or a WebView.

## Repository development

`examples/gallery` is the standalone Bun Gallery project. It owns the shared
pages, state, and application mount. `examples/gallery-vite` supplies the Vite
configuration and entrypoint, importing the application through its workspace
package. Vite dependencies belong to the Vite Gallery project; the workspace
root owns installation and shared tasks. Run `bun run gallery` for direct Bun
execution, or `bun run dev` from either project directory.

- `bun run gallery:vite`: build the required packages and start the host with the Vite/Bun Gallery.
- `bun run --cwd examples/gallery-vite build`: produce `examples/gallery-vite/dist/main.js` with Vite.
- `target/debug/gallery-host bun --conditions=browser examples/gallery-vite/dist/main.js`: run the production bundle after building the host.

Saving application code remounts it in the existing window. Changes to Rust,
the protocol schema, generated native APIs, or Vite configuration require
restarting the development command. Rust changes also require rebuilding the host.

## Application integration

Install `@solid-gpui/core` and Vite 8, then create a configuration:

```ts
import { defineConfig } from "vite";
import { solidGpui } from "@solid-gpui/core/vite";

export default defineConfig({
  plugins: [solidGpui({ entry: "src/app.tsx" })],
});
```

Have the existing native host launch `node_modules/.bin/solid-gpui-dev src/app.tsx`.
This entrypoint uses Bun with the `browser` condition and loads the Vite
configuration from the current directory. A second argument can select a
configuration file. A custom Bun launcher can call
`startDev(entry, configFile?)` from `@solid-gpui/core/vite/dev`; its return value
provides Vite shutdown. The plugin publishes TypeScript source for execution in Bun.

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
`examples/gallery-vite/vite.config.ts`; applications usually use package exports.

## State and failure boundaries

Reloading remounts the application. It does not automatically preserve each
component's signals. `captureState` returns structured-cloneable data that the
next generation receives in `setup`. The Gallery preserves its route, theme,
search, and window size; component state, native input, and scroll caches are
recreated. Do not retain Solid owners, Root or router instances, functions, or
native resources across generations.

After the candidate's setup, render, and first-frame preparation succeed, the
previous owner and root are disposed and a new epoch is published for the same
surface ID. Syntax errors and synchronous setup/render errors leave the previous
page mounted; fixing the error and saving again retries the replacement. A failed
initial launch closes Vite and exits because no previous page exists.

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

## Production bundles

`solid-gpui-build` uses the same Solid/Oxc transform as Vite and the Bun preload.
It runs under Bun during the build; the compiler does not run inside QuickJS.
Select the target explicitly:

```sh
node_modules/.bin/solid-gpui-build --runtime bun src/app.tsx dist/app.js
```

A Bun entrypoint uses `StdioTransport`, as above. Run its bundle with an existing
host and `bun --conditions=browser dist/app.js`. Embedded Bun uses the same
stdio-shaped transport and requires the `embedded-bun` feature.

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
node_modules/.bin/solid-gpui-build --runtime quickjs src/quickjs.tsx dist/app.js
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
Vite HMR continues to execute under Bun. QuickJS runs the built entry; replacing
its bundle requires restarting that runtime.

## Operational constraints

- Solid's universal renderer requires the client reactive implementation. Configure `browser` in both Vite SSR `conditions` and `externalConditions`, and launch Bun with that condition.
- Use Vite ModuleRunner HMR. Setting `server.hmr: false` also disables server-side HMR. Do not layer Bun `--hot` or a browser HMR client onto it.
- Reserve stdout for length-prefixed protocol bytes. Send Vite and application diagnostics to stderr with `console.error`.
- Exclude the native `target` directory from watching. Disable automatic dependency discovery, which has no HTML entrypoint to inspect and could scan embedded Bun vendor test data.
- Integration verification must save a TSX dependency, introduce syntax and render failures, recover, decode the entire stdout stream, and check epochs, explicit state, and owner cleanup. Transform-only tests cannot establish working HMR.

Key tests: `scripts/hot-reload.test.ts` and
`packages/solid-gpui/tests/application.test.ts`.
References: [Vite framework Environment API](https://vite.dev/guide/api-environment-frameworks)
and [Vite runtime API](https://vite.dev/guide/api-environment-runtimes).
