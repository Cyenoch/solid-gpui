# Vite survives native development session failures

Vite owns a persistent development supervisor for the hosts it launches. A
native source change stops the previous session, rebuilds and exports one host,
then starts that host with its matching bindings; errors leave the watcher
available for the next edit. This keeps the strict commit rejection in
[ADR-0008](0008-error-handling-philosophy.md) without making an application error
terminate the developer's workflow.

The supervisor does not resume rejected Patch streams, retry unchanged failing
binaries, or preserve native windows and services across Rust process replacement.
`#native`, SDK component imports, and Motion share the selected host's exported
catalog, avoiding mixed contracts during rebuilding. Direct Rust-owned launches
and caller-owned environments retain their own process lifecycle. This refines
the restart ownership in [ADR-0015](0015-vite-bun-native-hot-reload.md) and
[ADR-0018](0018-vite-application-toolchain.md); implementation and verification
contracts are in [the development guide](../hot-reload.md#managed-development-sessions).
