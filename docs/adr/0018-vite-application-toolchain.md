# Vite owns application compilation

Status: accepted, 2026-09-08.

Applications either execute JavaScript directly or compile JSX/TSX with Vite.
Bun and QuickJS are execution runtimes; Bun applications retain Bun APIs. A
separate `@solid-gpui/vite` package owns compilation, development environments,
and native binding preparation so direct-JS users do not install a compiler.
This replaces the standalone bundler and TSX launchers, avoiding multiple config,
plugin, resolution, and watch pipelines that diverge as applications grow.

Bun development uses Vite's ModuleRunner in a native-host-owned process. QuickJS
development uses Vite watch builds and the existing Rust generation supervisor;
its restricted module loader still consumes one self-contained JS module.
Native Modules keep one host-exported contract in both authoring paths. Rust can
start the same Vite project, or attach when Vite already owns development, without
recursively building or launching another host.

This refines [ADR-0015](0015-vite-bun-native-hot-reload.md) and
[ADR-0017](0017-runtime-engines.md). See [Vite integration](../vite.md) for the
supported configuration and process lifetimes.
