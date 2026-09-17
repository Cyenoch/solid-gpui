# Contributing to Solid GPUI

## Setup

Required tools:

- Rust toolchain from `rust-toolchain.toml`
- Bun from `.bun-version`
- cargo-deny `0.20.2`

```sh
bun install --frozen-lockfile
bun run check
```

## Task system

`scripts/tasks.ts` is the only development and release task graph. The root
`package.json` exposes common aliases; advanced commands use
`bun run task <command>`. Keep one root `bun.lock` and do not add package-local
locks or duplicate package scripts. `bun run task sdk-pack <output>` builds and
packs all four packages once when you need installable tarballs of the SDK, and
`bun run task native-codegen` is the explicit native-binding generation step
(`package-build` is JavaScript-only).

Use `Bun.spawn()` with argument arrays for portable subprocesses and Bun APIs
such as `Bun.Archive` when they replace platform utilities directly. Keep Bash
only inside substantive platform-specific release or stress helpers. Add new
commands to the Commander CLI instead of creating a second task runner.

## Ownership map

- `packages/solid-gpui/src/renderer.ts` and `renderer/host-config.ts`: Solid owner and host-mutation boundary.
- `packages/solid-gpui/src/renderer/root-container.ts`: transactional Snapshot/Patch production and commands.
- `packages/solid-gpui/src/protocol/`: semantic DTOs, Bebop codec, schema-derived guard, and checked generated TypeScript bindings.
- `crates/solid-gpui/src/protocol.rs`: semantic protocol types, framing, and Rust wire entrypoints.
- `crates/solid-gpui/src/protocol/{guard.rs,wire/adapter.rs}`: bounded structural guard and generated-to-semantic adapter.
- `crates/solid-gpui/src/tree.rs`: validated retained native tree.
- `crates/solid-gpui/src/host`: application and multi-surface lifecycle.
- `crates/solid-gpui/src/runtime/embedded.rs` and `crates/solid-gpui-bun-sys`: optional embedded runtime.

Do not introduce a second tree protocol, compatibility wrapper, browser abstraction, or synchronous cross-runtime callback. Refactor the owning seam instead.

## Focused checks

```sh
bun run task package-typecheck
bun run task package-test
bun run task rust-check
```

`package-test` covers the TypeScript packages, including the Vite plugin and its
QuickJS cases. For an interactive desktop loop, run
`bun run --cwd examples/desktop-app dev`, or `bun run quickjs:dev` for the
fixture counter under the real QuickJS engine. For renderer
changes, prove a signal produces a Patch. For host changes, run the relevant
process or embedded counter path. Add only tests that protect an observable
contract or invariant.

## Protocol changes

A wire change requires synchronized TypeScript and Rust models, the canonical
`packages/solid-gpui/src/protocol/protocol.bop` schema, generated bindings,
schema-derived guards, validation, golden fixtures, and `docs/protocol.md`.
Regenerate the checked bindings before refreshing the fixtures:

```sh
bun run task protocol-codegen-check
bun run task protocol-golden-check
```

`protocol-golden-check` regenerates both directions, verifies malformed vectors,
and fails if the committed fixtures drift. Never edit generated bindings by hand.
Never add legacy decoding or dual-form output unless the current specification
explicitly requires it.

## Documentation

Keep `README.md`, `CONTEXT.md`, and `docs/protocol.md` aligned with the code. Architectural decisions belong in `docs/adr/`; issue work lives under `.scratch/` according to `docs/agents/issue-tracker.md`.

`docs/getting-started.md` is the authoritative external-consumer sequence:
install, prepare, typecheck, develop, test, build, and preview. Keep advanced
integration, runtime, and packaging detail in its own guide and link it instead
of extending the happy path. Do not document a command that the task graph or a
published CLI does not implement.

English documents are authoritative. Every guide published on the website needs
a `.zh-CN.md` copy with a localized level-one heading; the website loads
`docs/*.md` directly through `examples/website/src/documentation.ts` and fails
without the translation. Update both copies in the same change, and keep the
navigation labels in `examples/website/src/Docs.tsx` and their Chinese
translations in `examples/website/src/locale.zh-CN.ts` in step.
