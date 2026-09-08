# Vite bundles; Bun executes native HMR

Status: accepted

Use Vite's runnable server environment under Bun for development. Transform Solid JSX
with `generate: universal`. Retain the native host and its byte transport; an application
replacement uses a new protocol epoch on the existing surface after candidate rendering succeeds.

`mountApplication` owns the Solid setup and renderer lifetimes. Explicit cloneable state
crosses generations; component owners and native caches do not. This avoids assuming
module replacement preserves renderer identity or cleans up effects automatically.

For Bun execution, production uses a bundle without a development server.
[ADR-0017](0017-runtime-engines.md) clarifies the intended Embedded Bun release
role and the separate QuickJS production target; this ADR describes the
external Bun development path.
The embedded one-evaluation runtime in ADR 0002 is unchanged; this development path uses
an external Bun process. Rust/config changes require restart.

See [the maintained workflow and failure boundaries](../hot-reload.md).
