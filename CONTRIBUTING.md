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
locks or duplicate package scripts.

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

Run `bun run example` for the interactive native counter or
`bun run example:smoke` for its bounded automated startup check. For renderer
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
