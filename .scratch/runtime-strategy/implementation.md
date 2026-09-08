# Runtime implementation and validation

Date: 2026-09-07. Scope: runtime documentation, embedded frame transport, and QuickJS application reload. Existing unrelated workspace changes are excluded.

## Implemented

- Website reference navigation exposes **Choose a runtime** and **Runtime strategy**, backed by `docs/runtimes.md` and `docs/runtime-strategy.md`.
- External Bun retains binary stdio. Both embedded engines use the explicit `EmbeddedTransport` native frame bridge. Embedded Bun no longer replaces `process.stdin` or `process.stdout` for renderer traffic.
- Transport submission reports accepted-but-pressured frames with `false`; drain resumes bounded router staging and event dispatch. Candidate application publication also respects pressure.
- QuickJS development uses an external Vite watcher and the shared Bun build policy. The native host stages a fresh VM, captures bounded explicit JSON state, validates every open surface and application configuration, then switches generations on the foreground thread.
- Candidate staging uses its hard aggregate capacity without a soft-watermark pause midway through the transaction. Normal pressure applies after activation.
- Invalid builds, invalid candidate trees, and entry module exceptions (including throws after mounting) preserve the active generation. An active VM failure can recover on edit from its most recent capture checkpoint. Native effects belong in `onMount`; external side effects are not transactional.
- `bun run quickjs:dev` launches the counter fixture; application projects use `solid-gpui-quickjs-dev <entry.tsx> <native-host>` through their local package binary. The website Gallery has separate Vite content plugins, so its desktop build is not presented as a plain Bun bundle.

## Automated evidence

- Browser-conditioned renderer/application/transport tests: 40 passed, including candidate pressure ordering and strict state serialization.
- `cargo test -p solid-gpui --features quickjs runtime:: --lib`: 10 passed. Real QuickJS runs cover state handoff, rejected candidates, supersession, recovery, cancellation, platform APIs, and candidate staging above the soft watermark.
- `cargo test -p solid-gpui --features quickjs,test-support generation_preflight --lib`: passed, including multiwindow rejection without partial mutation and zero-window activation.
- `cargo test -p solid-gpui --features embedded-bun --test host_embedded_counter -- --nocapture`: passed with the actual embedded Bun/JSC library, covering frame traffic, pressure, HTTP, workers, timers, and shutdown.
- Core, tooling, and fixture TypeScript checks passed. QuickJS library Clippy passed with warnings denied.
- Website TypeScript and production builds passed; existing Vite configuration and generated WASM warnings remain.
- Development-tool integration against a temporary TCP host verified imported dependency edits, build-error rejection, recovery after fixing the dependency, and cleanup on child exit. Vite browser WebSocket/HMR servers are disabled for this launcher.

## Interactive evidence

The native counter ran in a real macOS window: click to 1, save and retain 1, introduce a syntax error and continue clicking the old generation to 2, then fix the entry and display `Reloaded: 2` in the same window. Closing the window ended the development process successfully. This smoke preceded the final pressure regressions; those were subsequently checked with real-VM automated tests.

The production website runtime page was opened in Chrome and visually checked for the runtime matrix, readable content, and navigation entries.

## Boundaries

No three-engine performance benchmark, component-local Fast Refresh, zero-copy transfer, or new Embedded Bun final packaging/signing pipeline is claimed. Embedded Bun remains the production direction for Bun-dependent applications, with current platform and release qualification explicitly documented. Rust and native contract changes require rebuilding/restarting the host.
