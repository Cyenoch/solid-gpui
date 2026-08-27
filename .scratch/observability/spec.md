# Observability

Status: resolved

## Scope

Answer the consumer support questions around the opt-in `REACT_GPUI_TAP`
metadata stream, host diagnostics, and the event-storm performance evidence.
The chosen slice stays deliberately small: aggregate the existing JSONL report,
identify the protocol version in host diagnostics, and document the limits of
what the existing seams can honestly claim. No metrics endpoint, telemetry
framework, OpenTelemetry integration, or runtime thread is introduced.

## Baseline audit

### 1. Runtime metrics and backpressure

Before this slice, the report was already a programmatic CLI
(`scripts/protocol-tap-report.py:155-162` at audit starting point `0c4d2a6`)
with overall duration/frame rate, kind counts, byte summaries, event-type
counts, command-result correlation, and frame interval percentiles. It did not
expose a patch byte rate, a time-bucketed rate view, or a direct malformed-frame
count. The tap metadata itself contains only timestamp, direction, peer, kind,
framed byte count, sequence, and optional event/command fields
(`packages/react-gpui/src/protocol-tap.ts:103-160`; the Rust tap renders the
same metadata at `crates/react-gpui/src/protocol_tap.rs:249-298`). Payloads are
never persisted.

The report now emits `frame_rate_hz`, `patch_bytes_per_second`,
`frames_by_kind`, `frame_bytes`, `byte_histogram`, one-second `timeline`
buckets, `malformed_frames`, and `malformed_records`
(`scripts/protocol-tap-report.py:15-30,77-126,183-230`). A consumer can answer
frames/sec and patch byte rate by running the report, without opening the raw
JSONL. The report remains a local command, not an installed runtime metrics
API.

Backpressure is not observable from the tap. `StdioTransport` keeps private
`backpressured`, pending-frame, and pending-byte state
(`packages/react-gpui/src/transport.ts:162-180`); it records a frame after the
write or enqueue path (`packages/react-gpui/src/transport.ts:217-232`) and
flushes the private queue on `drain` (`packages/react-gpui/src/transport.ts:291-323`).
The process host has its own bounded event queue (32 frames and
`MAX_FRAME_LENGTH` bytes) that returns `WouldBlock`
(`crates/react-gpui/src/transport.rs:112-169`), while the tap records an event
only after a successful writer write (`crates/react-gpui/src/transport.rs:225-258`).
There is no tap record for queue depth, drain, enqueue delay, or backpressure;
manual stderr/runtime instrumentation would be required. The bounded slice
therefore reports this as an explicit limitation instead of guessing from
inter-frame gaps.

### 2. Tap discoverability, aggregates, and overhead

The environment variable was already discoverable in the root debugging quick
reference (`README.md:368-380`), the package README's debugging section
(`packages/react-gpui/README.md:364-384`), and the symptom-first troubleshooting
workflow (`docs/troubleshooting.md:58-90`). The old report invocation was
usable, but its output was mostly broad summaries. The report now makes the
high-value support aggregates explicit: frames by kind, overall and patch byte
rates, fixed one-second frame/byte buckets, byte-size histogram, event subtype
histogram, and malformed input counters.

The current measured overhead claim is one informational microbench documented
in `packages/react-gpui/README.md:400-404`: 10,000 seven-byte snapshot frames,
tap off 18.03 ms versus tap on 33.65 ms, about 1.56 microseconds of incremental
wall time per frame. The later event-storm audit intentionally measured the
tap-disabled renderer path (`.scratch/perf-event-storm/spec.md:9-19`) and its
10,000-node stateful path (`.scratch/perf-event-storm/spec.md:27-37,45-50`),
so it did not revalidate tap-on overhead. The 1.56 microsecond number remains a
historical measured result, not a current guarantee or performance gate.

### 3. Frame-budget visibility and commit timing

The perf-storm evidence is consumer-relevant context but not a live metric:
143-node gallery-sized scroll/drag paths stayed below the 8.33 ms 120 Hz
interval, while 10,000-node stateful paths measured roughly 9 ms at the median
and over budget (`.scratch/perf-event-storm/spec.md:37,45-50`). The audit found
more than 99% of stateful cost at the React reconciler/host commit-diff
boundary, not protocol stages (`.scratch/perf-event-storm/spec.md:27-37`).

There is no per-commit timing log today. `REACT_GPUI_LOG` only parses
`off|error|info|debug` and `host_log` prints diagnostics at those levels
(`crates/react-gpui-host/src/main.rs:44-79`); the startup line is the only
normal info line around runtime creation (`crates/react-gpui-host/src/main.rs:637-645`).
The temporary `REACT_GPUI_PROFILE` probe used during the perf audit was removed
before the final run and is not committed runtime behavior
(`.scratch/perf-event-storm/spec.md:19`). The tap timestamp is taken at frame
metadata write time, after renderer encode/submit, so it cannot provide the
React commit-diff duration. Adding a debug commit timer would require a new
renderer seam and a runtime behavior change; it is rejected from this bounded
slice and recorded as future work.

### 4. Startup diagnostics and protocol version

At the audit starting point (`0c4d2a6`), the host startup line was formatted as
`starting mode={mode:?} entry={} pid={}` and `--version` printed only the package
version (`crates/react-gpui-host/src/main.rs:637-645,860-862` at that
revision). The protocol compatibility work already had a single Rust
`PROTOCOL_VERSION = 3` (`crates/react-gpui/src/protocol.rs:7-10`), a typed Rust
mismatch diagnostic (`crates/react-gpui/src/protocol.rs:1184-1187`), and the
symmetric TypeScript `ProtocolVersionMismatchError`
(`packages/react-gpui/src/protocol.ts:11-22`). Neither host identity line
surfaced that version.

The host now prints `protocol=v3` in both places: the info startup line is
built at `crates/react-gpui-host/src/main.rs:637-645,860-862`, and `--version`
uses `version_line` at `crates/react-gpui-host/src/main.rs:864-870`. The focused
host test locks both formats (`crates/react-gpui-host/src/main.rs:889-903`).
Support can therefore capture host package version, protocol version, runtime
mode, renderer entry, and PID in one startup conversation.

## Decision

1. Keep `scripts/protocol-tap-report.py` as the single local summary command;
   do not add an HTTP endpoint, metrics registry, OpenTelemetry dependency, or
   runtime worker.
2. Add deterministic report aggregates over existing tap records. One-second
   non-empty buckets expose frame and byte rates; `frame_bytes` excludes the
   synthetic `tap_stopped` marker, while the pre-existing broad `bytes` and
   `bytes_by_kind` fields retain all record metadata for continuity.
3. Treat unknown-classified wire records as `malformed_frames`; count parser
   rejected JSON/object/timestamp lines separately as `malformed_records`. This
   is intentionally honest about the tap's payload-free classifier: unknown
   future message kinds are counted with malformed prefixes rather than being
   claimed as fully decoded protocol errors.
4. Include `protocol=v3` in the host startup info line and host `--version`, and
   update candidate/release smoke expectations. This is formatting only and
   does not alter protocol negotiation.
5. Document the report output, version lines, overhead evidence, and missing
   backpressure/commit timing in the root/package/troubleshooting surfaces.

## Rejected alternatives

- **Metrics endpoint or telemetry framework:** rejected as disproportionate to a
  local support diagnostic and explicitly out of scope. Existing JSONL already
  supplies bounded, process-local metadata.
- **Tap-recorded queue depth/backpressure:** rejected because neither adapter's
  queue state is available at the tap seam; inferring pressure from timestamps
  would be misleading. A future explicit transport diagnostic can be designed
  when a consumer contract requires it.
- **Per-commit debug timer now:** rejected because the measured hot path is the
  TypeScript reconciler/host commit-diff boundary and the prior profile probe
  was temporary. Do not add a renderer hook or runtime instrumentation merely
  to produce a speculative number.
- **Changing the tap format or persisting payloads:** rejected. Aggregation is
  pure tooling over the existing bounded metadata and preserves payload privacy.
- **Protocol negotiation:** rejected; v3 remains a lockstep boundary. The host
  version line identifies the current version but does not imply compatibility
  negotiation.

## Verification

- `python3 scripts/protocol-tap-report.test.py` passes one deterministic
  fixture test covering frame/kind counts, frame and patch byte rates, byte
  histogram, event types, one-second timeline, and malformed counters.
- `python3 -m py_compile scripts/protocol-tap-report.py` passes.
- The host unit test covers both `version_line()` and `startup_diagnostic()`.
- Host candidate, embedded-candidate, and release-check scripts now assert the
  intentional `protocol=v3` output change.
- Full `make ci`, `make embedded-bun`, `make host-release-check`, and
  `make host-candidate-smoke`/`make host-embedded-candidate-smoke` remain the
  integration gates for the parent workstream.
