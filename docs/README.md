# Solid GPUI documentation

The [GitHub Pages website](https://cyenoch.github.io/solid-gpui/) hosts interactive
guides, components, and Showcase apps. Read the guides below or
[run the website locally](../examples/website/README.md).

Use this order:

1. [`../README.md`](../README.md) — project overview, quick start, and a small TSX example.
2. [`../CONTEXT.md`](../CONTEXT.md) — shared domain vocabulary and invariants.
3. [`getting-started.md`](getting-started.md) — build and run a Solid application.
4. [`protocol.md`](protocol.md) — authoritative framed Bebop v5 contract, bounded decoding, and generated binding workflow.
5. [`troubleshooting.md`](troubleshooting.md) — runtime and build failures, including blank QuickJS windows after Vite reports ready and stale Cargo optimization profiles.
6. [`rust-bridge.md`](rust-bridge.md) — export ordinary Rust logic and native components with generated types.
7. [`hot-reload.md`](hot-reload.md) — Bun HMR and QuickJS application reload, automatic native rebuilds, persistent development sessions, explicit state contracts, and activation/recovery verification.
8. [`gpui-components.md`](gpui-components.md) — GPUI Kit controls, native motion, editor selections, and application APIs.
9. [`performance-analysis.md`](performance-analysis.md) — repeatable performance measurement, native commit profiling, and acceptance.
10. [`scroll-performance.md`](scroll-performance.md) — native layout contracts, regressions, and recorded measurements.
11. [`keyboard-and-menus.md`](keyboard-and-menus.md) — platform shortcuts, window scope, and system menus.
12. [`distribution.md`](distribution.md) — verified application bundles, native dependencies, and signing for macOS, Windows, and Linux.
13. [`adr/`](adr/) — architectural decisions.

See [Vite integration](vite.md) for direct JS, JSX/TSX development and builds,
Bun APIs, native module generation, and Rust-owned Vite startup.

See [Preserve UI state with captureState](capture-state.md) for a complete
`setup(previous)` example, QuickJS JSON constraints, draft restoration, and the
difference between UI reload and native host replacement.

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
- [Iconify](iconify.md) — embedded icons, size and color, reactive usage, and application icon registration.
- [System popovers](system-popover.md) — owned native popup Surfaces, shared context, lifecycle, multi-display placement, and platform qualification limits.
- [Native presentation research](../.scratch/native-presentation/spec.md) — design baseline and remaining SwiftUI/AppKit embedding stages; current delivery evidence is tracked separately.

- [GPUI Kit source](../references/gpui-kit/) — pinned [upstream](https://github.com/longbridge/gpui-kit) source used for implementation comparison.
- [Zed GPUI source](../references/zed/crates/gpui/) — pinned GPUI implementation reference.
