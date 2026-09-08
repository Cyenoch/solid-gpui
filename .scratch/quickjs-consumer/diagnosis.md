# QuickJS consumer reload investigation

Date: 2026-09-08. Environment: macOS arm64, Rust 1.98.1, Bun 1.4.2,
`rquickjs-sys` 0.12.2. Solid GPUI base: `a9b98bb`.

## Consumer build profile changes the outcome

The adjacent RMCL application already keeps a dedicated `captureSession` DTO,
omitting optional `undefined` fields. Its saved real-VM reproduction lives at
`../rmcl/.scratch/quickjs-reload/reload_tests.rs` relative to this repository.
RMCL currently uses Bun; this investigation used an isolated copy of its UI with
`EmbeddedTransport`, preserving its pinned JavaScript packages and layout.

The saved tests were temporarily adapted to the Solid GPUI integration-test host.
They publish actual Snapshot/Patch frames, navigate through native events, reload
the bundle, reject a syntax-error candidate, and navigate/reload again. A separate
probe started directly at `/settings`, asserting the actual player-name field.

| Interpreter build | Download reload and recovery | Settings reload | Direct settings startup |
| --- | --- | --- | --- |
| `opt-level = 0` | Pass | Stack overflow in `createMemo` / `Field` children | Same stack overflow |
| `opt-level = 3` | Pass | Pass | Pass |

The comparison changed only Cargo's `profile.dev.package.rquickjs-sys.opt-level`
via `--config`; the JS stack remained 2 MiB and the worker stack 8 MiB. Returning
to level 3 passed all three probes again. No layout change or stack increase was
needed. Cold-route failure disproves a requirement for prior state capture or VM
replacement to trigger the failure.

Solid GPUI's root manifest already selects level 3. Cargo does not inherit this
profile into consuming workspaces. The application's root manifest must select
the interpreter profile explicitly. See the [authoritative Cargo rule](https://doc.rust-lang.org/cargo/reference/profiles.html)
and the updated [reload guide](../../docs/hot-reload.md).

The temporary integration harness and copied bundle are removed after diagnosis;
RMCL retains its original reproducible test and updated setup instructions. Its
settings assertion now waits for the form's player-name field, rather than a
category label also present in navigation. This is not a desktop, native-command,
or complete application qualification, and RMCL's runtime/dependency pins remain
unchanged.

## A separate lifecycle ordering race

`QuickJsAdapter` stored control operations in one `Option<Control>`. Calling
`resume(true)` followed immediately by `capture()` could overwrite the unprocessed
activation. A real-VM test observed `{"activated":false}` after this sequence.
This can affect consecutive reloads independently of page complexity.

Control operations now use a four-entry FIFO. Capture preserves earlier
activation, and exhaustion reports an explicit error; failure to enqueue a
required resume/activation fails the VM instead of losing that operation.

Regression command:

```sh
cargo test -p solid-gpui --features quickjs runtime::quickjs::tests::immediate_capture_observes_activation --lib
```

The test failed before the change and passes after it, exercising 32 fresh VMs
to cover the scheduling race.

## State contract and diagnostics

The project's strict codec, not QuickJS's native JSON implementation, rejects
nested `undefined`. Top-level absent capture is encoded as an empty state array;
explicit `null` is retained. Errors now report the offending field and value
category without printing state values. The previous array property-count check
also admitted a sparse array whose extra property compensated for the hole;
canonical index validation closes that data-loss gap.

The real-VM regression rejects a session field, proves the old VM remains
interactive, captures explicit `null` after a native event, and restores it in
a fresh VM. The source API comment now distinguishes Bun's structured-clone
contract from QuickJS's JSON contract.

```sh
bun --conditions=browser test packages/solid-gpui/tests/transport-pressure.test.ts
cargo test -p solid-gpui --features quickjs runtime::quickjs::tests::reload_capture_reports_invalid_session_field_and_preserves_the_live_vm --lib
```

Capture runs in the old VM before candidate launch. Correcting only new code
cannot replace an old capture function that always returns invalid data. The
diagnostic now explains that live state must be corrected or the host restarted.

## Qualification boundaries

- `applied` acknowledges activation. Async route initialization can still fail
  afterward; inspect actual restored content and an interaction.
- A simple default-route launch is weaker than direct startup at a nested route.
- SDK source tests and production fixtures may exercise different code if the
  package's `dist` has not been rebuilt. Build the package before real-VM tests.
- Passing the producer workspace does not qualify a consumer's build profile.
- The new diagnostics and FIFO fix are local upstream changes; RMCL's pinned
  package does not acquire them until a coordinated dependency update.

## Final verification

- Runtime suite: 12 passed, including real QuickJS state handoff and activation
  ordering. The ordering regression failed before the FIFO change.
- Application/state/Bun HMR tests: 8 passed. The original codec serialized the
  sparse-array counterexample to `[null]`; the corrected codec rejects its extra
  property explicitly.
- RMCL isolated application probes: 3 passed with the optimized interpreter,
  including direct settings startup and full settings/download reload recovery.
  The unoptimized comparison intentionally reproduces settings failures.
- Website suite: 6 passed; content/highlight verification passed again after the
  final guide changes. Website typecheck and embedded build passed.
- SDK declarations and fixture types regenerated/checked; targeted formatting
  and Git whitespace checks passed.

No native window, platform-specific UI interaction, complete RMCL native-service
round trip, or full workspace qualification was performed. The existing Cargo
future-compatibility notice for `block` and Vite's future config-loader warning
remain outside this change.
