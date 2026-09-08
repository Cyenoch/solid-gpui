# Per-call native cancellation and deadlines

Status: ready-for-agent
Resolution: implemented and verified

## Acceptance

Generated clients accept cancellation options; aborted work is isolated, settles once, and async work releases admission. Blocking work has cooperative cancellation.

## Comments

Implementation and verification evidence will be recorded here.

Implemented per-call signal/deadline propagation, canonical cancellation command, isolated Rust task retirement, and injected cooperative NativeCallContext. Key executor, real GPUI cancellation, macro, generated contract, and TypeScript command tests pass. See ../executor-tests.log, ../native-contracts.log, ../cancellation-rust.log, and ../cancellation-ts.log.
