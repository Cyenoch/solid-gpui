# 01 — Runtime queue and commit timing gaps

Status: future

## Evidence

`StdioTransport` owns private `backpressured`, pending-frame, and pending-byte
state (`packages/react-gpui/src/transport.ts:162-180,217-232,291-323`). The Rust
process adapter separately bounds its event writer queue to 32 frames and
`MAX_FRAME_LENGTH` bytes and returns `WouldBlock` when full
(`crates/react-gpui/src/transport.rs:112-169`). The protocol tap records only
successful framed traffic metadata, not queue state or drain latency
(`crates/react-gpui/src/transport.rs:225-258` and
`packages/react-gpui/src/protocol-tap.ts:103-160`).

`REACT_GPUI_LOG=debug` currently selects the existing host diagnostic levels but
there is no commit timer in the host's normal startup/runtime path
(`crates/react-gpui-host/src/main.rs:44-79,637-645`). The temporary profile probe
used for the event-storm audit was removed before the final run
(`.scratch/perf-event-storm/spec.md:19`).

## Decision

Do not infer backpressure from tap timestamp gaps, and do not add a renderer
commit timer solely for this slice. The event-storm audit attributes the
10,000-node cost to the React reconciler/host commit-diff boundary, while the
143-node path stays below the 120 Hz interval (`.scratch/perf-event-storm/spec.md:27-37,45-50`).
If support requires these metrics later, define an explicit transport/runtime
contract first (queue depth, enqueue rejection, drain latency, and timer scope)
and add focused tests; do not add a metrics endpoint or background thread by
default.
