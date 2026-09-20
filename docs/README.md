# Solid GPUI documentation

The [GitHub Pages website](https://cyenoch.github.io/solid-gpui/) hosts interactive
guides, components, and Showcase apps. Read the guides below or
[run the website locally](../examples/website/README.md).

Use this order:

1. [`../README.md`](../README.md) — project overview and a small TSX example.
2. [`getting-started.md`](getting-started.md) — the authoritative external-consumer sequence: install, prepare, typecheck, develop, test, build, preview, plus the Cargo requirements your workspace root must carry. The SDK is not on a public registry yet, so it starts by packing tarballs from one pinned checkout.
3. [`../CONTEXT.md`](../CONTEXT.md) — shared domain vocabulary and invariants.
4. [`vite.md`](vite.md) — plugin options, artifact lookup, the published test runner, source consumption, and native module generation.
5. [`protocol.md`](protocol.md) — authoritative framed Bebop v6 contract, bounded decoding, and generated binding workflow.
6. [`troubleshooting.md`](troubleshooting.md) — stale bindings, install-hook mistakes, cross-target refusals, and runtime failures.
7. [`rust-bridge.md`](rust-bridge.md) — export Rust logic and native components, configure desktop hosts, windows, and titlebars.
8. [`hot-reload.md`](hot-reload.md) — Bun HMR and QuickJS application reload, Rust rebuild watching, persistent development sessions, explicit state contracts, and activation/recovery verification.
9. [`gpui-components.md`](gpui-components.md) — GPUI Kit controls, application themes, native motion, editor selections, and application APIs.
10. [`performance-analysis.md`](performance-analysis.md) — repeatable performance measurement, native commit profiling, and acceptance.
11. [`scroll-performance.md`](scroll-performance.md) — native layout contracts, regressions, and recorded measurements.
12. [`keyboard-and-menus.md`](keyboard-and-menus.md) — platform shortcuts, window scope, and system menus.
13. [`distribution.md`](distribution.md) — bundle versus native executable versus distributable, verified and experimental target capability, and signing.
14. [`adr/`](adr/) — architectural decisions.

See [Choose a runtime](runtimes.md) for the development and delivery paths,
including the experimental Embedded Bun packager, and
[runtime strategy](runtime-strategy.md) for external Bun development, Embedded Bun
production packaging, Rust-led QuickJS applications, and the communication and
QuickJS hot reload research. See
[native UI composition](native-composition.md) for layout, painting, async control
flow, and accessibility.

See [Preserve UI state with captureState](capture-state.md) for a complete
`setup(previous)` example, QuickJS JSON constraints, draft restoration, and the
difference between UI reload and native host replacement.

Run the [desktop application example](../examples/desktop-app/README.md) to see
window configuration, themes, scrolling, and Rust services working together.
Its README links to the authoritative topic guides instead of duplicating them.

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
