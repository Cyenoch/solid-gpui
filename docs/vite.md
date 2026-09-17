# Vite integration

Solid GPUI supports two authoring paths: write JavaScript and run it directly,
or write JSX/TSX and use Vite to compile it. Bun and QuickJS are execution
runtimes. Vite is the only supported application bundler. There is no direct
TSX launcher, built-in bundler, or Bun compile path.

`@solid-gpui/vite` is the tooling package. It provides this Vite plugin, the
`solid-gpui` CLI (`prepare`, `preview`, `doctor`, `test`), the
`@solid-gpui/vite/test` runner, and the `@solid-gpui/vite/artifacts` and
`@solid-gpui/vite/project` helpers. [Getting started](getting-started.md) walks the
full sequence; this guide covers the option surface and the advanced paths.

Three different things are produced along the way, and only the third is a
deliverable:

| Artifact              | Produced by                                         | Contains                                                                                                                                                                                            |
| --------------------- | --------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Bindings**          | `solid-gpui prepare` (`bun run generate`)           | The host's exported component catalog, commands, and types, plus the generated TypeScript project.                                                                                                  |
| **Bundle**            | `bun --bun vite build`                              | One JavaScript entry module for the host to execute. The build also prepares the configured native host (an incremental Cargo build), but it emits no native code into the bundle and no installer. |
| **Native executable** | Cargo, via `native` in the plugin or your own build | The GPUI host that renders the bundle.                                                                                                                                                              |
| **Distributable**     | Your packaging script                               | Executable plus bundle, assets, licences, and signature. See [distribution](distribution.md).                                                                                                       |

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

## Project setup

`@solid-gpui/core` and `solid-js`, plus `@solid-gpui/vite` and Vite 8 as
development dependencies. The packages are not on a public registry yet: build
matching tarballs from one pinned SDK checkout with `bun run task sdk-pack <dir>`
and install `solid-gpui-core.tgz` / `solid-gpui-vite.tgz` from that run (see
[Getting started](getting-started.md#1-install)). Install by name only once the
packages are published; today that resolves nothing. Native development tooling runs
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
Bun. `runtime` is explicit and is never inferred or converted: `"bun"` keeps Bun
and Node imports as runtime imports, while `"quickjs"` builds one self-contained
ES module and rejects them.

`solidGpui()` also sets `resolve.dedupe: ["solid-js"]`, so the SDK and your
application share one Solid instance; do not add another `solid-js` copy or a
hand-written dedupe entry. `solidGpuiSource()` is exported from both
`@solid-gpui/vite/source` and the package root.

Add the scripts once and run the generator after installing dependencies. Do not
name a `package.json` script `prepare`: Bun runs a root package's `prepare` script
during install, which must not compile a native host.

```json
{
  "scripts": {
    "generate": "solid-gpui prepare",
    "check:generated": "solid-gpui prepare --check",
    "doctor": "solid-gpui doctor",
    "dev": "bun --bun vite",
    "build": "bun --bun vite build",
    "preview": "solid-gpui preview",
    "test": "solid-gpui test"
  }
}
```

`prepare` writes three generated files: `.solid-gpui/tsconfig.json` (the TypeScript
project that maps `#native` and `@solid-gpui/core/components` to the bindings the
plugin actually selects), the bindings file (`native.output`, default
`.generated/native.ts`), and `.solid-gpui/artifacts.json` (the resolved artifact
record). Keep `.solid-gpui/` out of version control; the bindings file may be
committed, in which case `solid-gpui prepare --check` is the CI freshness gate.
`prepare --check` writes nothing (it still resolves and builds the host to compare
against the real catalog) and fails when either file is stale; `prepare --json`
prints the artifact record.
Project commands accept `--root` and `--config`. `prepare` and `preview` accept
`--mode` (default `production`); `test --mode` defaults to Vite's `development`
mode. `doctor` does not support selecting a Vite mode. For `preview` and `test`,
arguments after `--` belong to the host or Bun, not to this CLI.

Your own `tsconfig.json` only extends the generated project:

```json
{
  "extends": "./.solid-gpui/tsconfig.json",
  "compilerOptions": {
    "strict": true,
    "jsx": "preserve",
    "jsxImportSource": "@solid-gpui/core"
  },
  "include": ["src", "vite.config.ts"]
}
```

Do not hand-maintain `paths`, `baseUrl`, or `customConditions` for the SDK: the
generated project and the Vite aliases come from one resolution. Add Vite's client
types when using `import.meta.hot`.

`solid-gpui doctor` reports environment and dependency problems: the active Cargo
dependencies, profiles, and `[patch.crates-io]` requirements your workspace root
must carry, plus unsupported runtime or target combinations. Cargo ignores profile
settings declared by a dependency, so a consuming workspace root must declare them
itself; see [application build configuration](hot-reload.md#application-build-configuration)
and [troubleshooting](troubleshooting.md).

## JSX and TSX with Vite

```sh
bun --bun vite
bun --bun vite build
bun run preview          # run the built host against the built bundle
```

`vite` launches the default `solid-gpui-host` executable. For an existing custom
host, set `host: { command: "./target/debug/my-host", args: [], output: ".generated/native.ts" }`.
Vite runs that command with `--export-native` before loading component imports,
then resolves `#native` and `@solid-gpui/core/components` to the exported file.
`command` is launched with Vite's root as its working directory, so a relative
executable path resolves against that root. `output` defaults to
`.generated/native.ts` relative to Vite's root. This applies
even when the application declares no custom native module: the built-in controls
must still use the running host's exact catalog. The executable must implement
`--export-native` and use Solid GPUI's host entrypoint or Rust `Vite` helper to
accept its managed renderer. It is not downloaded or compiled implicitly.

`host.args` precede `--export-native`, allowing an interpreted exporter such as
`{ command: "bun", args: ["host.ts"] }`. Rust host export takes no extra launch
arguments. The unconfigured default host uses the SDK catalog generated from
the standard `solid-gpui-host`; keep both on the same revision. `host: false`
selects no native contract and leaves its resolution to the caller.

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
external when they must ship beside the application. A build also prepares the
configured native host, so a change that a `generate` would rebuild is compiled here
too; with `native` unconfigured it does not build a host at all. Build success does
not package native libraries, sign an application, or install the selected runtime;
[distribution](distribution.md) owns that work.

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
Unchanged output is not rewritten.

### Native build options

| Option           | Default                | Meaning                                                                                |
| ---------------- | ---------------------- | -------------------------------------------------------------------------------------- |
| `manifestPath`   | —                      | Cargo manifest of the host. Required.                                                  |
| `package`, `bin` | —                      | Cargo selectors; required together when a manifest has several binaries.               |
| `features`       | —                      | Cargo features, for example `["quickjs"]`.                                             |
| `output`         | `.generated/native.ts` | Bindings destination, relative to Vite's root.                                         |
| `profile`        | `"dev"`                | Cargo profile; `"debug"` is accepted as an alias.                                      |
| `target`         | —                      | Cargo `--target` triple for a cross build.                                             |
| `locked`         | `true`                 | Pass `--locked`. Set `false` only for an intentional first resolution.                 |
| `check`          | `false`                | Verification mode: fail instead of writing when bindings are stale.                    |
| `watch`          | —                      | Extra files or directories that trigger a native rebuild.                              |
| `exporter`       | —                      | Host executable used to export bindings when `target` is not runnable on this machine. |

`prepare` and development need a host for the current machine. When `target` names
a platform or architecture other than the build host, both refuse to execute it and
tell you to drop `target` for the prepare/development host or to pass `exporter`;
`--export-native` is never run on a foreign target. Cross-compiled artifacts are
built by Cargo for packaging, and their paths come from the artifact record.

Use either `native` or `host` to select an executable. Both export bindings from
that executable; `native` additionally builds and watches its Cargo project.
It does not require declaring an application-specific native module. Use
`native` for an application-owned Rust host, including one that only registers
the built-in component module. The first Rust build cannot depend on the JS
bundle whose bindings it is about to export. During development, Rust source, Cargo manifest,
lockfile, and workspace Cargo configuration edits rebuild the host and bindings
automatically, including local path dependencies. `native.watch` adds files or
directories that Cargo cannot infer. Vite stops the old runtime before publishing
the new bindings, then launches a fresh host. Compilation or
application startup failures leave the watcher running; fix and save to retry.
Closing the native window leaves development watching too; Ctrl+C stops it.
See [managed development sessions](hot-reload.md#managed-development-sessions)
for watched inputs, state loss on host replacement, and lifecycle ownership.

Native module calls are independent of the bundler. A direct Bun JS application
can import the host's exported `native.ts` because Bun loads TypeScript modules;
it needs no Vite process or `#native` alias. `createClient(root)` and `useNative()`
use the same identities, command dispatch, cancellation, and validation in both
authoring paths. See [Rust bridge](rust-bridge.md).

## Artifacts and project resolution

`.solid-gpui/artifacts.json` is the authoritative record: `root`, `runtime`,
`entry`, `outDir`, `bindings`, `tsconfig`, `bundle` when one exists, the selected
`host` (`command`, `args`, `output`), and the `native` build settings
(`manifestPath`,
`package`, `bin`, `features`, `profile`, `profileDirectory`, `target`, `locked`,
`targetDirectory`, `executable`). `outDir` is the configured Vite build output
directory (absolute) and is the stable identity of where this build's outputs
belong; `bundle` always lives inside it. `profileDirectory` maps `dev` → `debug`,
`release` → `release`, and any other profile to its own name; `target` is the
effective Cargo target, including an implicit `CARGO_BUILD_TARGET` or
`.cargo/config.toml` `build.target`. `prepare` writes `bundle` as a prediction from
your Vite configuration (a string `entryFileNames` pattern keeps its placeholders)
and the production build replaces it with the entry Vite actually emitted, which may
be nested (an output function, or a pattern such as `assets/[name].js`), so read the
record after `vite build` when a packaging script or `preview` needs the real path —
never derive it from the config. Reading requires `root`, `runtime`, `entry`,
`outDir`, and `tsconfig`; a record written by another version or corrupted fails with
a message telling you to re-run `solid-gpui prepare`.
Read it instead of reconstructing `target/<profile>/<bin>` or the bundle path:

```ts
import { readNativeArtifacts, prepareProject } from "@solid-gpui/vite/artifacts";

const record = await readNativeArtifacts(process.cwd()); // or: (await prepareProject()).artifacts
const executable = record?.native?.executable ?? record?.host?.command;
```

`prepareProject` resolves the project and returns the same record as `artifacts`;
`cargoProfileDirectory(profile)` maps a Cargo profile name to its output directory
(`dev` → `debug`, `release` → `release`, otherwise the profile name).
`recordedHostExecutable(record)` returns the host Cargo actually built — the
authoritative path for launching or packaging the profile you built — while
`expectedExecutablePath(record, profile?)` only predicts where another profile's
executable would be; verify it, or run that profile's build, before shipping it.
`previewApplication({ root?, configFile?, mode?, args?, env? })` from
`@solid-gpui/vite/project` runs the built host against the built bundle without
rebuilding, which is what the `solid-gpui preview` CLI command and the `preview`
script do.
Use the same `--mode` for build and preview when the config selects its entry or
native Cargo profile by mode, for example `vite build --mode release` followed by
`solid-gpui preview --mode release`. Preview resolves that mode before checking
the recorded artifacts; it never silently substitutes another profile.

## Testing

`solid-gpui test` delegates to `@solid-gpui/vite/test`:

```sh
bun run test                       # whole suite
bun run test src/app.test.tsx      # one file
```

```ts
import { runTests } from "@solid-gpui/vite/test";

const exitCode = await runTests({ root: process.cwd(), configFile: "vite.config.ts", args: ["src/app.test.tsx"] });
```

`runTests({ root?, configFile?, mode?, args? })` resolves to the process exit code and
uses the application's own `vite.config.ts`: the real JSX transform, aliases,
`?inline` assets, deduped `solid-js`, and the native bindings contract. `args` are
forwarded verbatim to `bun test`, so file filters and every `bun:test` flag work.
The runner adds the `browser` condition itself, so you never pass
`--conditions=browser` for tests. One Bun process and one client Solid runtime
serve the whole suite, and that same runtime is what the packaged application
loads, so a test and the application share one reactive graph. Bun's test runner
keeps its normal semantics, so ordinary single-file tests and `bun:test` APIs work
unchanged. No test needs to
import a private `dist` module, copy the compiler, or build its own Vite pipeline.

Tests run in Bun and preserve its real `process.env`, including mutations and
environment inherited by subprocesses, even when the application targets QuickJS.
This test-only override does not expose Node/Bun capabilities or environment
variables to a production QuickJS bundle. `mode` selects the application's Vite
config, aliases, definitions and `.env.<mode>` loading; it does not select another
test runtime or add a `NODE_ENV` value. Omitting it keeps Vite's `development`
default. Use `solid-gpui test --mode staging` or `runTests({ mode: "staging" })`.

### Inspect renderer output without private protocol imports

`TestHost` from `@solid-gpui/core/testing` inspects a `MemoryTransport` and replays
its Snapshot and Patch frames into an ordered, detached tree view:

```tsx
import { expect, test } from "bun:test";
import { createRoot, Pressable, Text } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { TestHost } from "@solid-gpui/core/testing";

test("press updates the committed text", () => {
  const host = new TestHost();
  const root = createRoot(host.transport, { surfaceId: 1 });
  try {
    root.render(() => {
      const [count, setCount] = createSignal(0);
      return (
        <Pressable onPress={() => setCount(count() + 1)}>
          <Text>{count()}</Text>
        </Pressable>
      );
    });
    const button = host.surface(1)!.nodes.find((node) => node.kind === "Pressable")!;
    host.dispatch(button, { type: "press" });
    expect(host.surface(1)!.nodes.some((node) => node.text === "1")).toBe(true);
  } finally {
    root.unmount();
  }
});
```

| Interface                                      | Behavior                                                                                                                                                                                                                                                                              |
| ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `new TestHost(transport?)`                     | Uses the supplied `MemoryTransport`, including already-submitted frames, or creates one.                                                                                                                                                                                              |
| `surface(id)`                                  | Returns the latest committed Surface, or `undefined` before its first Snapshot. `nodes` is preorder, including the synthetic root; each node exposes kind, parent/ordered child IDs, text, input value, placeholder, accessibility label and tooltip. Previous views are not mutated. |
| `commits`                                      | Ordered Snapshot/Patch metadata: type, Surface, epoch and revision. Wire tags and update masks stay private.                                                                                                                                                                          |
| `dispatch(node, event)`                        | Sends `press`, `focus`, `blur`, `input` (`text`, optional UTF-8-byte selection offsets), or `native` (`eventId`, JSON `value`). The captured node's revision and epoch are retained, so stale-event behavior remains testable.                                                        |
| `nativeProps(node)`                            | Decodes the JSON DTO props of a `createNativeComponent` node.                                                                                                                                                                                                                         |
| `nativeCalls`                                  | Observed module-function and component-method requests: Surface, epoch, node/request IDs, module identity, function ID and opaque `args`. Use the public `decodeJson` from `@solid-gpui/core/native` for generated DTO calls.                                                         |
| `reply(call, bytes)` / `reject(call, message)` | Settles that exact request through the real event path; JSON DTO replies use `encodeJson(value)`. Responses can arrive out of order; a request cannot be answered twice or through another TestHost.                                                                                  |

Reads consume only already-submitted frames. Await your application's scheduled
work before inspecting a later commit (a synchronous signal update outside event
dispatch normally needs `await Promise.resolve()`). Native client calls also defer
submission until the current Solid batch completes. The helper owns injected
event sequences; do not mix it with manually encoded events or clear
`transport.submitted`. Use a fresh host per test and unmount roots in cleanup.
It does not calculate native styles/layout, paint pixels, execute Rust handlers,
or emulate platform services; those still require a real host.

## Consume packages from source

Installed packages resolve to their built `dist` output by default, and ordinary
use needs nothing else. To debug or iterate on SDK internals, opt in explicitly:

```ts
import { defineConfig } from "vite";
import { solidGpui } from "@solid-gpui/vite";
import { solidGpuiSource } from "@solid-gpui/vite/source";

export default defineConfig({
  plugins: [solidGpuiSource(), solidGpui({ entry: "src/app.tsx" })],
});
```

```sh
bun --conditions=solid-gpui-source vite      # run, test, or build with source resolution
```

Bun does not accept custom conditions from `bunfig.toml` (verified on 1.4.2), so
pass `--conditions=solid-gpui-source` on the command line, for example
`bun --conditions=solid-gpui-source test`. TypeScript agrees through either
`"extends": "./.solid-gpui/tsconfig.json"` (the generated project already carries
the source `paths`) or
`"customConditions": ["solid-gpui-source"]` with
`"moduleResolution": "bundler"` or `"nodenext"`.

`@solid-gpui/vite/source` exports `SOURCE_CONDITION`, `SDK_PACKAGES`,
`solidGpuiSource(options?)` (the plugin), `sdkSource(options?)`, and
`sdkPackage(name, root?)`. `sdkSource({ root?, packages?, exclude? })` returns the
resolved table — `aliases` (longest-first, ready for `resolve.alias`), `paths`
(specifier to absolute source file, the same table `prepare` writes into
`.solid-gpui/tsconfig.json`), `dedupe`, and `names` — for tooling that cannot use
conditions at all. `@solid-gpui/core/components` is deliberately excluded from the
source mappings because the plugin remaps it to the selected host's generated
bindings; a web-only application that has no native host passes `exclude: []`.

Keep source mode opt-in: a packed consumer and a source consumer must still share
one reactive graph, and ordinary use stays `dist`-based.

## Rust-owned Vite development

A Rust application can own the Vite/Bun runtime directly:

```rust
use solid_gpui::runtime::vite::Vite;

let runtime = Vite::new("path/to/frontend")
    .config_file("vite.config.ts")
    .spawn()?;
solid_gpui::run_application(app::native_module, runtime);
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
bun run preview          # or: <host> --runtime quickjs <bundle>
```

Vite builds one self-contained ESM module, initializes the explicit QuickJS UI
platform, and rejects Bun/Node imports, remaining dynamic imports, and external
assets. Use inline assets or host-managed files. QuickJS has no ambient Bun API,
Node services, DOM, or network `fetch`. Put those services in native modules. The
runtime selection is a build decision: nothing converts a Bun bundle into a QuickJS
bundle or substitutes one runtime for the other.

A DOM-free QuickJS application type-checks against the published ambient types
instead of hand-writing its own declarations:

```json
{
  "compilerOptions": {
    "lib": ["ES2024"],
    "types": ["@solid-gpui/core/quickjs"]
  }
}
```

The entry declares the facilities the engine actually provides — timers
(`setTimeout`, `setInterval`, `clearTimeout`, `clearInterval`, `queueMicrotask`),
`performance.now()`, `console`, UTF-8-only `TextEncoder`/`TextDecoder`, `self`,
`URL`/`URLSearchParams`/`DOMException`/`Event`/`EventTarget`/`AbortController`/
`AbortSignal`/`Headers`/`Response` (the bodyless native-router redirect response),
`import.meta.url`, and `declare module "*?inline"` with a default `string` export
for inlined assets. It deliberately declares no `fetch`/`Request`, filesystem or
socket API, no `requestAnimationFrame`, no `import.meta.hot`, and no Node or Bun
API, so an unavailable capability is a compile error rather than a runtime throw.
`types: []` plus a hand-written `.d.ts` remains possible, but it loses that check.
The prepared `.solid-gpui/tsconfig.json` already carries this entry.

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
