# @solid-gpui/core/vite

Vite universal JSX plugin and Bun module runner for native Solid GPUI applications.
Exports `solidGpui({ entry })` and `startDev(entry, configFile?)` (from `@solid-gpui/core/vite/dev`).
The `solid-gpui-dev` executable runs under Bun with Solid's browser resolution condition.
The Vite plugin, Bun preload, and `solid-gpui-build` share the official
Oxc-based Solid universal transform; application execution still uses Solid 1.

For standalone bundles, run
`solid-gpui-build --runtime <bun|quickjs> <entry.tsx> <output.js>`.
QuickJS output is one self-contained ESM module; its entry uses
`EmbeddedTransport` from `@solid-gpui/core/embedded`. Bun entries use
`StdioTransport` from `@solid-gpui/core/stdio`.

For an application with a Rust host, enable native exports in the same plugin:

```ts
solidGpui({
  entry: "src/app.tsx",
  native: { manifestPath: "native/Cargo.toml", bin: "my-app" },
});
```

Before loading the JS entry or bundling, this compiles the host and runs
`--export-native`, then atomically generates `.generated/native.ts` and resolves
`#native` to it. `package`, `features` and `output` are optional native settings.
Point TypeScript's `paths` entry for `#native` at that same generated file.
Rust changes require restarting the host and development command; TSX-only
changes retain the existing HMR flow. The first Rust build must not depend on
the JavaScript bundle that needs these bindings.

See [native hot reload guide](../../../../docs/hot-reload.md) for host startup, state preservation,
production bundling, failure boundaries, and diagnostic rules. Requires Vite 8 and Bun 1.4.2+.
