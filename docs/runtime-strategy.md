# Runtime strategy

The three runtime modes have distinct product roles. These roles describe the
intended development and delivery workflow; they do not imply that every release
pipeline is already implemented.

## Positioning

| Runtime          | Primary role                                                     | Application responsibilities                                                                                                  |
| ---------------- | ---------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| External Bun     | Rapid iteration and development, including Vite hot reload       | Bun may own domain logic and services, or call Rust Native Modules.                                                           |
| Embedded Bun     | Final production packaging for applications using Bun services   | Keep the Bun application model while running Bun/JSC inside the native host.                                                  |
| Embedded QuickJS | UI runtime for applications whose main capabilities live in Rust | JSX/TSX owns UI composition, interaction, and reactive presentation state; Rust owns domain capabilities and system services. |

UI state is still real application code: signals, event handlers, routing, local
validation, and presentation calculations belong in JSX/TSX where appropriate.
The QuickJS role does not mean static markup. It means that files, networking,
long-running domain work, and other principal capabilities are implemented in
Rust and exposed through Native Modules instead of recreating Bun/Node services
inside QuickJS.

All three modes use the same Solid universal renderer and native contracts.
GPUI owns native windows, input, layout, and painting in every mode. Application
ownership and process topology are separate decisions, as recorded in
[ADR-0017](adr/0017-runtime-engines.md).

## Intended workflow and current support

For a Bun-based application, develop with external Bun and Vite, then package
with Embedded Bun. Embedding must be qualified using the actual embedded runtime;
passing tests under the external Bun executable does not qualify a differently
pinned embedded Bun build or all of its service APIs.

For a Rust-led application, ship the UI through QuickJS. External Bun and Vite
can provide rapid UI iteration today, but the shared UI must also run in the real
QuickJS VM to catch unsupported dependencies and platform assumptions. QuickJS
application reload is available through `bun run quickjs:dev` or the
`solid-gpui-quickjs-dev` application launcher.

| Area                     | Current implementation                                                                                                                             | Remaining work                                                                                                               |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| External Bun development | Default host mode; Vite dependency reload, explicit state transfer, and failure recovery have integration coverage.                                | Keep this fast development path as runtime internals evolve.                                                                 |
| Embedded Bun production  | Optional `embedded-bun` feature; macOS-only embedding, bounded transport, lifecycle tests, and a recorded native Gallery interaction run.          | An equivalent final application packaging/signing/dependency-validation pipeline, and qualification on each intended target. |
| QuickJS production       | Optional `quickjs` feature; a self-contained ESM bundle can be embedded in the executable. The current Gallery packaging workflow uses this route. | Complete release qualification for each target desktop; candidate archives are not proof of native desktop correctness.      |
| QuickJS hot reload       | Development rebuilds replace the QuickJS VM while retaining the native host. Production evaluates one bundle per runtime.                          | Use explicit state and setup-owned Surface identities; native/config changes require restart.                                |

The current [distribution guide](distribution.md) describes QuickJS Gallery
packages. It is not evidence that Embedded Bun packaging is complete. The
[embedded Gallery acceptance record](../.scratch/native-authoring-research/gallery-qualification.md)
proves selected interaction and shutdown behavior, not release packaging or
performance.

## Communication baseline

| Mode         | Current transport                                                                | Ownership boundary                                                            |
| ------------ | -------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| External Bun | Length-prefixed binary protocol over child stdin/stdout through `StdioTransport` | Owned bytes cross process boundaries.                                         |
| Embedded Bun | In-process bounded byte queues through `EmbeddedTransport`                       | Owned bytes cross the runtime and GPUI threads; this is not an OS stdio pipe. |
| QuickJS      | In-process bounded byte queues through `EmbeddedTransport`                       | Owned bytes cross the QuickJS worker and GPUI threads.                        |

No mode directly shares live JS objects, Solid owners, closures, or GPUI handles
between JavaScript and the GPUI thread. Engine-local Rust bindings are a different
concept from sharing VM values across threads. Generated native clients already
provide typed operations across this boundary.

The authoritative transport invariants remain in [the protocol guide](protocol.md)
and [ADR-0017](adr/0017-runtime-engines.md). Research may recommend changes, but
an unimplemented recommendation does not silently replace these contracts.

## Performance interpretation

Embedded Bun removes the separate Bun process, but it still initializes and runs
Bun/JSC. In-process transport still has queueing, synchronization, encoding, and
decoding costs. It does not imply zero-copy object access or a higher native FPS.
QuickJS avoids shipping Bun services, but its interpreter must still execute the
UI's JavaScript. Native layout and painting remain shared GPUI work.

Embedded Bun's first build also compiles a pinned, patched Bun native library.
A reusable build directory can reduce repeat work; it does not skip version and
patch validation. This build cost is separate from application startup time.
See [ADR-0002](adr/0002-embedded-bun-runtime.md).

No controlled three-runtime performance comparison is established by these
roles. Compare the same UI, workload, release profile, and native host with the
[performance workflow](performance-analysis.md), recording startup-to-content,
total process memory, JS work, transport cost, and native input-to-present
separately.

## Design research

The research records separate implementation facts from proposals and cite
primary sources:

- [Communication and ownership](../.scratch/runtime-strategy/transport-research.md).
- [QuickJS hot reload and Vite integration](../.scratch/runtime-strategy/quickjs-hot-reload.md).

The recommended communication direction keeps framed stdio for external Bun and
uses one explicit native frame bridge
for both embedded engines. Pressure must reach the renderer scheduler; changing
only an adapter or adding another queue would not solve bounded production.
Directly sharing live JS/GPUI objects is not the recommendation. Bulk buffer
ownership transfer is a separate optimization requiring measurements and
explicit memory accounting.

QuickJS reload implements external Bun/Vite tooling plus
whole-bundle replacement in a fresh QuickJS Runtime. A stable Rust supervisor
owns epochs, state handoff, candidate validation, activation, and retirement,
while preserving native windows and Rust services. Vite need not execute inside
QuickJS. Reuse the current QuickJS build policy first; the existing Vite SSR
configuration is not a QuickJS target. A custom ModuleRunner is a later option
only if measured rebuild/evaluation costs justify its complexity.

The reports preserve the research evidence and alternatives. The current bridge
and whole-bundle QuickJS reload are implemented; ModuleRunner and bulk-buffer
ownership transfer remain measurement-gated alternatives. See the maintained
[reload workflow](hot-reload.md#quickjs-application-reload).
