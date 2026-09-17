# @solid-gpui/vite

Vite development, builds, and tooling for Solid GPUI applications running in Bun or
QuickJS. Install this package with `@solid-gpui/core`, `solid-js`, and Vite 8. None
of the SDK packages are published to a registry yet: from one pinned SDK checkout
run `bun run task sdk-pack <dir>` and install `solid-gpui-core.tgz` and
`solid-gpui-vite.tgz` from that run together. Native development tooling requires
Bun 1.4.2 or newer.

## Project

```ts
import { defineConfig } from "vite";
import { solidGpui } from "@solid-gpui/vite";

export default defineConfig({
  plugins: [solidGpui({ entry: "src/app.tsx", runtime: "bun" })],
});
```

```json
{
  "scripts": {
    "generate": "solid-gpui prepare",
    "check:generated": "solid-gpui prepare --check",
    "doctor": "solid-gpui doctor",
    "typecheck": "tsc --noEmit",
    "dev": "bun --bun vite",
    "test": "solid-gpui test",
    "build": "bun --bun vite build",
    "preview": "solid-gpui preview"
  }
}
```

Run `bun run generate` after installing. It writes `.solid-gpui/tsconfig.json`
(extend it from your own `tsconfig.json`; it carries the `#native` and
`@solid-gpui/core/components` mappings), the bindings file (`native.output`,
default `.generated/native.ts`), and `.solid-gpui/artifacts.json` (the resolved
artifact record). `prepare --check` writes no files and fails when either generated file is stale (it
still builds the host to compare against the real catalog), `prepare --json` prints
the record, and `doctor` reports environment and Cargo
dependency problems. Do not name the script `prepare`: that is an install lifecycle
hook, and installation must not compile a native host. Every command accepts
`--root`, `--config`, and `--mode`.

Vite owns JSX/TSX compilation, aliases, virtual modules, watching, and production
bundling. `runtime` is explicit: `"bun"` preserves Bun and Node imports for Bun to
execute, and `"quickjs"` builds one self-contained ES module and rejects them.
Bun retains its own APIs, including `Bun.file`, `bun:sqlite`, Workers, and Node
builtins. Direct JavaScript applications use core without this package.

## Native hosts

The default native development host is `solid-gpui-host` on `PATH`. Select an
existing executable with `host: { command, args?, output? }`. Its `--export-native`
output supplies `#native` and `@solid-gpui/core/components` in development and
builds; `output` defaults to `.generated/native.ts`. The executable must already
exist and support export; Vite never substitutes a stale SDK catalog. Arguments
are passed before `--export-native`, so an interpreted host can use
`{ command: "bun", args: ["host.ts"] }`. Rust host export takes no extra launch
arguments. `host: false` selects no native contract and leaves the runnable Vite
environment to the caller.

Alternatively, build and watch an application-owned Cargo host:

```ts
solidGpui({
  entry: "src/app.tsx",
  native: { manifestPath: "native/Cargo.toml", bin: "my-app" },
});
```

| Native option | Default | Meaning |
| --- | --- | --- |
| `manifestPath` | — | Host Cargo manifest. Required. |
| `package`, `bin` | — | Cargo selectors, required together for a multi-binary manifest. |
| `features` | — | Cargo features, for example `["quickjs"]`. |
| `output` | `.generated/native.ts` | Bindings destination. |
| `profile` | `"dev"` | Cargo profile; `"debug"` is accepted as an alias. |
| `target` | — | Cargo `--target` triple for a cross build. |
| `locked` | `true` | Pass `--locked`; set `false` only for an intentional first resolution. |
| `check` | `false` | Fail instead of writing when bindings are stale. |
| `watch` | — | Extra files or directories that trigger a native rebuild. |
| `exporter` | — | Host used to export bindings when `target` cannot run on this machine. |

The plugin builds the selected Cargo binary, runs its `--export-native` entrypoint,
and atomically writes the bindings before loading the application. Rust sources,
Cargo manifests and configuration, declared build-script inputs, declared native
assets, and `native.watch` paths rebuild the host and restart the session once per
burst; application modules keep normal Vite HMR. `.solid-gpui/`, `target/`, the
build output directory, and `node_modules/` never trigger rebuilds. The old session
stops before bindings change, and a host rebuild reopens windows and resets
application state. Prepare and development refuse to execute a host whose `target`
is not this machine; pass `exporter` or drop `target`.

Neither `host` nor `native` requires application-specific native modules. A host
that only registers the built-in controls still supplies its own exact catalog.
The default unconfigured `solid-gpui-host` uses the matching SDK catalog.

## Artifacts, preview, and tests

`.solid-gpui/artifacts.json` records `root`, `runtime`, `entry`, `outDir`,
`bindings`, `tsconfig`, the production `bundle` when one exists, the selected
`host`, and the `native` build settings. `outDir` is the configured Vite output
directory and the stable identity of where this build's outputs belong; `prepare`
writes `bundle` as a prediction that `vite build` replaces with the entry it
actually emitted (possibly nested), so read the record after the build. Read it
instead of reconstructing `target/<profile>/<bin>`:

```ts
import { readNativeArtifacts, prepareProject, cargoProfileDirectory, recordedHostExecutable, expectedExecutablePath } from "@solid-gpui/vite/artifacts";
import { previewApplication } from "@solid-gpui/vite/project";
```

`recordedHostExecutable(record)` is the host Cargo actually built (authoritative);
`expectedExecutablePath(record, profile?)` only predicts another profile's location,
so verify it or build that profile before shipping.

`solid-gpui preview` runs the built host against the built bundle recorded there —
no rebuild, no watcher, host arguments after `--`, and the host's exit status.

`solid-gpui test [args...]` delegates to this package's test entrypoint:

```ts
import { runTests } from "@solid-gpui/vite/test";

const exitCode = await runTests({ root: process.cwd(), configFile: "vite.config.ts", args: ["src/app.test.tsx"] });
```

`runTests({ root?, configFile?, args? })` uses the application's own
`vite.config.ts`, so tests see the real JSX transform, aliases, `?inline` assets,
deduped `solid-js`, and the native bindings contract, with `bun:test` semantics and
ordinary single-file invocation.

## Source consumption

Ordinary use resolves to `dist`. To run against SDK sources, add the plugin and pass
the condition on the command line (Bun ignores custom conditions in `bunfig.toml`):

```ts
import { solidGpuiSource } from "@solid-gpui/vite/source";

export default defineConfig({ plugins: [solidGpuiSource(), solidGpui({ entry: "src/app.tsx" })] });
```

```sh
bun --conditions=solid-gpui-source vite
```

The subpath also exports `SOURCE_CONDITION`, `SDK_PACKAGES`, `sdkSource(options?)`,
and `sdkPackage(name, root?)`; `sdkSource()` returns `aliases`, `paths`, `dedupe`,
and `names` for tooling without condition support. TypeScript agrees through the
generated project or `customConditions` with `moduleResolution: "bundler"`.

Bun entries use `StdioTransport`; QuickJS entries use `EmbeddedTransport` and
`runtime: "quickjs"` with a QuickJS-enabled host. Rust applications can start the
same Vite project through `solid_gpui::runtime::vite::Vite`. The lower-level
`startDev({ root?, configFile?, nativeHost? })` export from `@solid-gpui/vite/dev`
returns a Vite server inside a Rust-owned Bun process using protocol stdio;
call `close()` to stop it. Browser hosts use `solidGpui({ target: "web" })`.

See [Vite integration](../../docs/vite.md) for setup, process ownership, native
modules, testing, and runtime limits, and [hot reload](../../docs/hot-reload.md)
for the captured-state and recovery contracts.
