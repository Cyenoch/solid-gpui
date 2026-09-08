# Solid GPUI documentation

The [GitHub Pages website](https://cyenoch.github.io/solid-gpui/) hosts interactive
guides, components, and Showcase apps. Until its first deployment, use the guides
below or [run the website locally](../examples/website/README.md).

Use this order:

1. [`../README.md`](../README.md) — project overview, quick start, and a small TSX example.
2. [`../CONTEXT.md`](../CONTEXT.md) — shared domain vocabulary and invariants.
3. [`getting-started.md`](getting-started.md) — build and run a Solid application.
4. [`protocol.md`](protocol.md) — authoritative framed Bebop v5 contract, bounded decoding, and generated binding workflow.
5. [`troubleshooting.md`](troubleshooting.md) — runtime and build failures.
6. [`rust-bridge.md`](rust-bridge.md) — export ordinary Rust logic and native components with generated types.
7. [`hot-reload.md`](hot-reload.md) — Bun HMR and QuickJS application reload, consumer build profiles, explicit state contracts, and activation/recovery verification.
8. [`gpui-components.md`](gpui-components.md) — generated native controls and application APIs.
9. [`performance-analysis.md`](performance-analysis.md) — repeatable performance measurement and acceptance.
10. [`scroll-performance.md`](scroll-performance.md) — native layout contracts, regressions, and recorded measurements.
11. [`keyboard-and-menus.md`](keyboard-and-menus.md) — platform shortcuts, window scope, and system menus.
12. [`distribution.md`](distribution.md) — verified application bundles, native dependencies, and signing for macOS, Windows, and Linux.
13. [`adr/`](adr/) — architectural decisions.

Start with [Choose a runtime](runtimes.md) for the development and delivery paths.

See also [runtime strategy](runtime-strategy.md) for external Bun development,
Embedded Bun production packaging, Rust-led QuickJS applications, and the
communication and QuickJS hot reload research. See
[native async composition and accessibility](native-composition.md).

See [native application migration](native-migration.md) for application-owned
runtimes, window profiles, titlebars, theme overrides, edge styles, and offline icons.

References:

- [Project brand assets](../assets/branding/README.md) — approved icon and generated Web and desktop formats.
- [Continuous integration](ci.md) — workflow triggers, dependency caches, audits, and manual release qualification.
- [Router](router.md) — file-based routing setup, native navigation, and TanStack references.
- [Routing and shared application state](../packages/solid-gpui-router/README.md) — surface navigation and application scope.
- [Shiki syntax highlighting](shiki.md) — Bun-backed native code blocks and build-time highlighting.
- [Native presentation research](../.scratch/native-presentation/spec.md) — proposed system popovers and SwiftUI composition, current backend gaps, and implementation acceptance gates; these capabilities are not implemented.

- [GPUI Component Shell](https://longbridge.github.io/gpui-component/shell/) — host-owned rendering and script snapshot reference.
- [GPUI Component source](../references/gpui-component/) — checked-in source used for implementation comparison.
- [Zed GPUI source](../references/zed/crates/gpui/) — pinned GPUI implementation reference.
