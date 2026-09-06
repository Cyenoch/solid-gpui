# Embedded Bun source build is outside ordinary candidate coverage

Status: ready-for-human
Type: task

The embedded Bun path compiles the pinned Bun/JSC source graph. It is not part
of ordinary `bun run ci` or the process-runtime host candidate archive. The
separate `bun run task embedded-check` gate and path-filtered embedded workflow
own this coverage.

Evidence: `scripts/tasks.ts`, `.github/workflows/embedded-bun.yml`, and
`crates/solid-gpui-bun/tests/embedded_counter.rs`.

## Comments

The separation avoids compiling Bun for every ordinary commit while retaining
an explicit release gate for changes to the embedded adapter, Bun patch, Solid
package, or toolchain inputs.

The gate builds the Solid renderer bundle, checks and tests the host with the
`embedded-bun` feature, and runs the adapter integration test. That integration
test starts the bundled counter, decodes its protocol-v3 Snapshot, routes a
press event, observes the resulting commit, and requires a clean runtime
shutdown. The embedded runtime has one event/commit bridge; it does not carry a
second watch or hot-reload protocol.
