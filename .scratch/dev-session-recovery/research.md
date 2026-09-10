# Development session recovery research

Researched: 2026-09-10. Scope: desktop development source edits and process
lifecycle, using official documentation and first-party source. Source findings
below are static inspection, not reproduced runtime experiments. Pinned upstream
branch snapshots are implementation evidence, not claims about every released
version.

## Tauri v2

- **Documented:** frontend edits update the WebView when the selected frontend
  tooling supports it. `tauri dev` watches `src-tauri` and dependent workspace
  crates, rebuilding and restarting the application after changes. `--no-watch`
  disables this. [Develop guide](https://v2.tauri.app/develop/#reacting-to-source-code-changes)
  (page updated 2026-07-22; accessed 2026-09-10).
- **Source inference:** Rust changes do not preserve the existing application
  window throughout compilation. The watcher kills and waits for the previous
  child before starting the next Cargo run.
  [Watcher implementation](https://github.com/tauri-apps/tauri/blob/6eda56cd820fd47f90da191cbef42902d5eb6b2b/crates/tauri-cli/src/interface/rust.rs#L510-L598).
- **Source inference:** ordinary Cargo compilation failures are classified as
  `CompilationFailed`; default watch mode keeps the CLI alive, so another save can
  rebuild. This does not imply every crash is recoverable: the examined desktop
  runner classifies other unplanned exits as `NormalExit`, which terminates the
  CLI. `--no-watch` and `--exit-on-panic` also alter the exit policy.
  [Exit classification](https://github.com/tauri-apps/tauri/blob/6eda56cd820fd47f90da191cbef42902d5eb6b2b/crates/tauri-cli/src/interface/rust/desktop.rs#L118-L141),
  [CLI exit policy](https://github.com/tauri-apps/tauri/blob/6eda56cd820fd47f90da191cbef42902d5eb6b2b/crates/tauri-cli/src/dev.rs#L302-L310).

## Electron tooling

Electron development behavior depends on the launcher and bundler configuration.
Two concrete implementations differ:

| Tool | Renderer edits | Main process edits | Preload edits |
| --- | --- | --- | --- |
| electron-vite with `--watch` | Vite HMR | Rebuild and restart Electron | Rebuild and reload renderer |
| Electron Forge Vite plugin, examined source | Vite HMR | Rebuild; automatic restart currently disabled, terminal `rs` restarts | Rebuild and reload renderer |

- **Documented:** electron-vite implements the three behaviors in the first row.
  Main/preload watching requires `--watch` or `build.watch`; the CLI option defaults
  to false. [HMR and hot reloading](https://electron-vite.org/guide/hmr-and-hot-reloading),
  [CLI options](https://electron-vite.org/guide/cli) (accessed 2026-09-10).
- **Source inference:** electron-vite's main rebuild callback removes the old
  child's exit listeners, kills it, and starts another child; preload sends a
  `full-reload` message. Those controlled restarts keep the development tooling
  running. Ordinary child closure invokes `process.exit`, so a genuine process
  crash is not universally contained by this toolkit.
  [Development server](https://github.com/alex8088/electron-vite/blob/608461c04cf74cbf27b47f9916e10e225cbfe8f1/src/server.ts),
  [Electron launcher](https://github.com/alex8088/electron-vite/blob/608461c04cf74cbf27b47f9916e10e225cbfe8f1/src/electron.ts).
- **Documented/source inference:** Forge documents renderer HMR. Its examined
  Vite plugin enables build watching and preload reload, but the main auto-restart
  call is commented out with a reference to issue #3380. Forge core advertises
  terminal `rs` and handles the controlled restart.
  [Forge Vite documentation](https://www.electronforge.io/config/plugins/vite),
  [Vite reload plugin](https://github.com/electron/forge/blob/6c9b951efe50b70960b5b22e173409831909e6f1/packages/plugin/vite/src/config/vite.base.config.ts#L96-L110),
  [Forge restart handler](https://github.com/electron/forge/blob/6c9b951efe50b70960b5b22e173409831909e6f1/packages/api/core/src/api/start.ts#L288-L304).
- **Source inference:** Forge Vite logs subsequent watcher build errors without
  explicitly shutting down the watchers in that error path. A child exit that is
  not marked as a controlled restart closes watchers and servers and exits the
  parent. Do not equate a compile diagnostic with a native/main-process crash.
  [Build-error and child-exit handling](https://github.com/electron/forge/blob/6c9b951efe50b70960b5b22e173409831909e6f1/packages/plugin/vite/src/VitePlugin.ts).

## Implication for solid-gpui

Local inspection in this investigation found that commit routing failure calls
`fatal_runtime_failure`, which shuts down the runtime and exits the host; Vite
then closes its server when the host child exits. The current design deliberately
does not discard an invalid incremental commit and continue applying subsequent
patches. See [error-handling ADR](../../docs/adr/0008-error-handling-philosophy.md)
and [development workflow](../../docs/hot-reload.md). QuickJS candidate rejection
already retains the previous application generation in the cases described by
that guide; Rust/native-contract changes require a host restart.

**Design proposal, not implemented:** retain strict session rejection, while
making the outer development supervisor persist through recoverable build and
session failures. Rust changes should trigger build, binding generation, and a
controlled host restart. Compilation errors should remain visible while the
watcher waits for another edit. A rejected contract should report component and
host identity in readable terms and move the supervisor into a failed-session
state; a corrected rebuild starts a fresh session. Repeating the same failing
binary indefinitely would not be useful recovery. Preserving the old native
window during Rust binary replacement would require additional architecture and
is not implied by this proposal or by Tauri's behavior.

## Documentation and verification scope

Reviewed [documentation index](../../docs/README.md) and
[website README](../../examples/website/README.md). This private research note
changes no runtime behavior or published capability. The existing development
workflow remains accurate; the website imports `docs/*.md`, not this note. No
published guide, translation, navigation, or generated API update is required.
Validation is limited to source/doc inspection and Markdown/link review; no
upstream application experiment or website build was run for this note.

## Implementation follow-up

The proposal was subsequently implemented in this workspace. The maintained
contract is now [managed development sessions](../../docs/hot-reload.md#managed-development-sessions)
and [ADR-0019](../../docs/adr/0019-persistent-native-development-sessions.md).
The investigation above records the pre-change baseline.

Vite-managed Bun and QuickJS sessions now survive build/startup/host failure,
watch local Rust dependencies, and replace the host with matching generated
bindings. Component imports and Motion share that export. Superseded builds and
shutdown terminate the compiler process group, including build scripts. Normal
application exit leaves an informational watcher message. Host commit rejection
remains strict; `ExtensionRegistry::resolve` returns its concrete error so module
and catalog diagnostics survive composition.

Verification on macOS:

- Three development integration tests cover missing executables, initial Rust
  and JS errors, external path dependencies, matching component/native bindings,
  host crashes, normal exits, obsolete builds, and process cleanup during build.
  The two runtime recovery scenarios also passed three consecutive runs.
- The native bridge suite passed seven tests; renderer tests passed 94 tests
  with one existing ignored test. The Presence lifecycle test passed.
- Package build, TypeScript checks, related Markdown links, and formatter checks
  passed. Native SDK/website bindings were regenerated from their actual hosts.
- The website build and all six website tests passed, including documentation
  highlighting, catalog type checking, and native preview navigation.
- The repository `quickjs:dev` and `website:native:dev` commands were started
  with a deliberately failing `RUSTC_WRAPPER`. Both reached Vite's watching
  state instead of exiting during bootstrap. The Rust-owned Vite lifecycle test
  still passed. A custom Cargo target directory containing generated Rust files
  is excluded from rebuild inputs.
- The actual `website-host` applied its initial epoch. Terminating that child
  left the same Vite process watching; touching a watched Rust source rebuilt
  and started a new host which applied its initial epoch. The development
  processes were stopped after the check.

The standalone native executable could not be attached through the available
CUA app inventory (`Invalid app`), so no screenshot or manual window interaction
is claimed. Windows and Linux process cleanup were not executed here.
