# Vite integration

Solid GPUI supports two authoring paths: write JavaScript and run it directly,
or write JSX/TSX and use Vite to compile it. Bun and QuickJS are execution
runtimes. Vite is the only supported application bundler. There is no direct
TSX launcher, built-in bundler, or Bun compile path.

## JavaScript without a bundler

Use `createComponent` and reactive getters instead of JSX:

```js
import { mountApplication, Text } from "@solid-gpui/core";
import { createComponent, createSignal } from "@solid-gpui/core/runtime";
import { StdioTransport } from "@solid-gpui/core/stdio";

mountApplication({
  transport: () => new StdioTransport(),
  setup() {
    const [message] = createSignal("Hello from JavaScript");
    return { render: () => createComponent(Text, { children: message }) };
  },
});
```

Save this as `app.js`, install the SDK and Solid, and let a native host execute it:

```sh
solid-gpui-host bun --conditions=browser app.js
```

The host executable must already be installed or built; use its explicit path
when it is not on `PATH`. It owns the Bun child and protocol stdio. The `browser`
condition selects Solid's reactive implementation; it does not install a DOM or
disable Bun APIs. Bun can resolve npm imports and use `Bun.file`, `bun:sqlite`,
Node builtins, Workers, and other APIs supplied by the selected Bun version.

QuickJS can also execute JavaScript directly through `QuickJsAdapter::start`
or `QuickJsAdapter::from_source`. Its current loader requires one self-contained
ES module: it cannot resolve npm packages or external imports. Direct execution
does not compile or silently bundle that input. Use Vite when dependencies need
bundling. QuickJS entries use `EmbeddedTransport` and a `quickjs`-enabled host.

## JSX and TSX with Vite

Install `@solid-gpui/core` and `solid-js`, plus `@solid-gpui/vite` and Vite 8
as development dependencies. Native development tooling runs
under Bun 1.4.2 or newer. Set `"type": "module"` in `package.json` and create
`vite.config.ts`:

```ts
import { defineConfig } from "vite";
import { solidGpui } from "@solid-gpui/vite";

export default defineConfig({
  plugins: [solidGpui({ entry: "src/app.tsx", runtime: "bun" })],
});
```

The entry explicitly calls `mountApplication`, using `StdioTransport` for external
Bun. Set TypeScript `jsx` to `preserve` and `jsxImportSource` to `@solid-gpui/core`.
Add Vite's client types when using `import.meta.hot`.

```sh
bun --bun vite
bun --bun vite build
solid-gpui-host bun --conditions=browser dist/app.js
```

`vite` launches the default `solid-gpui-host` executable. For an existing custom
host, set `host: { command: "/path/to/my-host", args: [] }`. The host must use
Solid GPUI's host entrypoint or Rust `Vite` helper to accept its managed renderer.
The executable is not downloaded or compiled implicitly.

Vite owns configuration, aliases, virtual modules, transforms, module watching,
and builds. Bun development executes Vite's ModuleRunner in the host-owned Bun
child; a separate authenticated loopback channel carries modules and HMR. Native
frames stay on stdio. Console output in that child goes to stderr. Entry HMR
acceptance is injected automatically; explicit application state preservation is
described in [hot reload](hot-reload.md). Standard console diagnostics, including
`console.dir` and `console.table`, go to stderr. Raw stdin/stdout APIs (including
`Bun.stdin`, `Bun.stdout`, and `console.write`) belong to the native protocol; use
stderr or application-owned files and sockets for other I/O.

Production builds bundle JavaScript dependencies by default and preserve Bun/Node
builtin imports for Bun to execute. Vite `ssr.external` can keep selected packages
external when they must ship beside the application. Build success does not package
native libraries, sign an application, or install the selected runtime.

## Application-owned native modules

Register Rust commands and components with `#[native_module]` and expose the host's
`--export-native` entrypoint. Then configure the same executable in Vite:

```ts
solidGpui({
  entry: "src/app.tsx",
  runtime: "bun",
  native: {
    manifestPath: "native/Cargo.toml",
    bin: "my-app",
    output: ".generated/native.ts",
  },
});
```

`manifestPath`, `entry`, and `output` resolve relative to Vite's root. `package`
and `features` are optional Cargo selectors. Cargo's artifact messages determine
the executable path, including custom target directories; ambiguous binaries are
rejected. Vite invokes that executable's exporter, atomically writes bindings,
and resolves both `#native` and `@solid-gpui/core/components` to that output before
loading the application. This also keeps Motion on the running host's catalog.
Unchanged output is not rewritten. Configure TypeScript `paths` for `#native`
to the same generated file; use its generated types when changing native APIs.

Use either `native` or `host` to select an executable. `native` already selects
and builds the host. The first Rust build cannot depend on the JS bundle whose
bindings it is about to export. During development, Rust source, Cargo manifest,
lockfile, and workspace Cargo configuration edits rebuild the host and bindings
automatically, including local path dependencies. Vite stops the old runtime
before publishing the new bindings, then launches a fresh host. Compilation or
application startup failures leave the watcher running; fix and save to retry.
Closing the native window leaves development watching too; Ctrl+C stops it.
See [managed development sessions](hot-reload.md#managed-development-sessions)
for watched inputs, state loss on host replacement, and lifecycle ownership.

Native module calls are independent of the bundler. A direct Bun JS application
can import the host's exported `native.ts` because Bun loads TypeScript modules;
it needs no Vite process or `#native` alias. `createClient(root)` and `useNative()`
use the same identities, command dispatch, cancellation, and validation in both
authoring paths. See [Rust bridge](rust-bridge.md).

## Rust-owned Vite development

A Rust application can own the Vite/Bun runtime directly:

```rust
use solid_gpui::runtime::vite::Vite;

let runtime = Vite::new("path/to/frontend")
    .config_file("vite.config.ts")
    .spawn()?;
solid_gpui::run_application(app::native_module(), runtime);
```

Omit `config_file` for Vite's normal config discovery. The frontend directory
must have the npm dependencies installed. `command()` returns a regular
`std::process::Command` when the application needs to configure its environment.
The helper selects Bun's client resolution condition and protocol-safe logging.
It exports bindings from the current native executable instead of recursively
building another host. Handle `--export-native` before starting this runtime.

When launched directly by Rust, that Rust process owns its own lifetime and must
be rebuilt/restarted externally. For automatic native rebuilds and persistent
failure recovery, launch the application through Vite with `native` configured.

When Vite launches that Rust application, the same helper attaches to the existing
Vite module channel. It does not start a second Vite server. This helper returns
a Bun `ProcessAdapter`; it is not a QuickJS adapter. Direct production JS continues
to use `ProcessAdapter`, `QuickJsAdapter`, or the embedded Bun adapter.

For an application-owned Bun test runner, use `host: false` and import modules
through Vite's runnable `ssr` environment. This disables automatic host startup.
An explicit Vite `dev.createEnvironment` also remains under the caller's control.

## QuickJS development and builds

Select `runtime: "quickjs"`, use an `EmbeddedTransport` entry, and select a host
compiled with the `quickjs` Cargo feature. With native modules, add
`features: ["quickjs"]` to the `native` options. The commands remain:

```sh
bun --bun vite
bun --bun vite build
my-host --runtime quickjs dist/app.js
```

Vite builds one self-contained ESM module, initializes the explicit QuickJS UI
platform, and rejects Bun/Node imports, remaining dynamic imports, and external
assets. Use inline assets or host-managed files. QuickJS has no ambient Bun API,
Node services, DOM, or network `fetch`. Put those services in native modules.

Development uses Vite's build watcher, including configured plugins and aliases,
then sends successful bundles to the existing Rust generation supervisor. It
replaces the VM while preserving Rust services and native windows; failed builds
leave the running application intact. Host profiles may wrap SolidRoot with
component providers; reload validation uses the retained renderer entity.
There is no ModuleRunner inside QuickJS
and no second bundler. Keep the consuming workspace's optimized interpreter
profile and [captured-state contract](hot-reload.md#captured-state).

For the experimental browser host, use `solidGpui({ target: "web" })`. This applies
the same universal JSX transform while leaving Vite's HTML, client environment,
and browser build configuration in place. See [Web host](web.md).
