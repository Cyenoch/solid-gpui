# @solid-gpui/vite

Vite development and builds for Solid GPUI applications running in Bun or QuickJS.
Install this package with `@solid-gpui/core`, `solid-js`, and Vite 8. Native
development tooling requires Bun 1.4.2 or newer.

```ts
import { defineConfig } from "vite";
import { solidGpui } from "@solid-gpui/vite";

export default defineConfig({
  plugins: [solidGpui({ entry: "src/app.tsx", runtime: "bun" })],
});
```

```sh
bun --bun vite
bun --bun vite build
```

Vite owns JSX/TSX compilation, aliases, virtual modules, watching, and production
bundling. Bun retains its own APIs, including `Bun.file`, `bun:sqlite`, Workers,
and Node builtins. Direct JavaScript applications use core without this package.

The default native development host is `solid-gpui-host` on `PATH`. Select an
existing executable with `host: { command, args }`, or build an application-owned
Rust host and generate its native bindings:

```ts
solidGpui({
  entry: "src/app.tsx",
  native: { manifestPath: "native/Cargo.toml", bin: "my-app" },
});
```

The plugin builds the selected Cargo binary, runs its `--export-native` entrypoint,
and atomically writes `.generated/native.ts` before loading the application.
`package`, `features`, and `output` are optional native settings. `#native` and
`@solid-gpui/core/components` resolve to that output; configure TypeScript's
`paths` for `#native` to match. Rust source and Cargo configuration edits rebuild
the host and bindings automatically, including local path dependencies. The old
session stops before bindings change. Compilation and application failures keep
Vite watching: fix the error and save to retry. Host replacement reopens windows
and resets application state. Ctrl+C stops development and its build processes.
Native commands and components use the same contract with direct JS.

Bun entries use `StdioTransport`; QuickJS entries use `EmbeddedTransport` and
`runtime: "quickjs"` with a QuickJS-enabled host. QuickJS builds produce one
self-contained ES module and reject unavailable Bun/Node imports and external
assets. Development feeds successful Vite watch builds to the Rust generation
supervisor, retaining native windows and Rust services.

Rust applications can start the same Vite project through
`solid_gpui::runtime::vite::Vite`. The lower-level
`startDev({ root?, configFile?, nativeHost? })` export from `@solid-gpui/vite/dev`
returns a Vite server inside a Rust-owned Bun process using protocol stdio;
call `close()` to stop it. For Bun, `host: false` leaves the runnable Vite environment
under the JS caller's control. Browser hosts use `solidGpui({ target: "web" })`.

See [Vite integration](../../docs/vite.md) for setup, process ownership, native
modules, and runtime limits, and [hot reload](../../docs/hot-reload.md) for the
captured-state and recovery contracts.
