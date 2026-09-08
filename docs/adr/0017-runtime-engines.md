# Bun and QuickJS share one native host contract

Status: accepted, 2026-09-07.

Updated: 2026-09-07 to make development and production positioning explicit.

Support two application ownership models: Rust-led applications keep domain
state and services in Rust; Bun-led applications keep them primarily in Bun.
SolidJS composes the UI in either case, and Rust always owns GPUI rendering,
native input, layout, and painting. Application ownership is independent of
whether JavaScript runs in another process or inside the host.

Bun supports both ownership models with its application services. QuickJS is
an optional lightweight embedded UI runtime for Rust-led applications: it runs
a self-contained ESM bundle, schedules UI work, and reaches Rust services
through generated native command clients. Adding Node/Bun service emulation to
QuickJS would duplicate a full application runtime and obscure this boundary.

The product workflow assigns external Bun to rapid development and Vite hot
reload, and Embedded Bun to final production packaging for applications that
need Bun services. QuickJS targets applications whose main capabilities live in
Rust, with JSX/TSX limited to UI composition, interaction, and reactive
presentation state. These are intended roles, not restrictions on which
executable can evaluate a production bundle.

The intended Embedded Bun release role is not a claim of completed packaging:
its embedding remains macOS-only and the current Gallery package uses QuickJS.
See [runtime strategy](../runtime-strategy.md) for current support and remaining
release work. This clarification changes workflow guidance, not the runtime
transport contract or the ownership distinction above.

All engines retain the same bounded byte transport contract, atomic Commit
Batches, Native Events, Surface lifetimes, and Native Modules. Applications
select their transport explicitly; `mountApplication` owns the connection
created by its factory and retains it across application reloads. Runtime
selection must not silently substitute engines or permit GPUI/JavaScript
objects to cross thread boundaries.

Compilation is separate from runtime selection. As specified in
[ADR-0018](0018-vite-application-toolchain.md), Vite runs the shared official
Solid/Oxc universal transform during development and bundling; direct JS needs
no bundler. The pinned
compiler emits the renderer helper contract exercised against the existing
Solid 1 runtime; choosing QuickJS does not require another JSX transform.

This extends [ADR-0002](0002-embedded-bun-runtime.md) with an additional engine
while retaining its Bun embedding decision. [ADR-0015](0015-vite-bun-native-hot-reload.md)
continues to define Vite HMR under Bun. Embedded Bun remains macOS-specific;
the new QuickJS integration does not imply cross-platform release qualification
or preservation of native instances across engine restarts.
