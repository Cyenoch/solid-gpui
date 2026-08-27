# 02 — Tap report and host identity ergonomics

Status: resolved

## Baseline

`REACT_GPUI_TAP` was documented with separate host/renderer paths and a
metadata-only privacy boundary (`docs/troubleshooting.md:58-90`,
`packages/react-gpui/README.md:364-384`). The existing report had broad frame,
byte, event, command-result, and interval summaries but did not directly show
a patch byte rate, time-bucketed rates, or malformed counters
(`scripts/protocol-tap-report.py:118-152,155-162` at audit starting point
`0c4d2a6`).

The host startup line and `--version` output did not include protocol identity,
although protocol v3 mismatch errors already carried both versions
(`crates/react-gpui/src/protocol.rs:7-10,1184-1187`; the pre-change host
format was at `crates/react-gpui-host/src/main.rs:637-645,860-862`).

## Resolution

- Extend the existing report command over unchanged JSONL records with
  `frames_by_kind`, `frame_bytes`, overall and patch byte rates, fixed
  one-second frame/byte `timeline` buckets, byte-size histogram,
  `malformed_frames`, and `malformed_records`.
- Keep payloads, the 64 MiB cap, and per-process file behavior unchanged.
- Add `protocol=v3` to the info startup diagnostic and `--version`; update
  candidate/release smoke expectations and focused host coverage.
- Document the aggregate names, the no-backpressure/no-commit-timing boundary,
  and the historical 1.56 microsecond tap-on overhead claim. The event-storm
  measurements were tap-disabled and therefore do not refresh that claim
  (`.scratch/perf-event-storm/spec.md:9-19,27-37`).

## Explicit non-goals

No metrics endpoint, OpenTelemetry, payload dump, runtime queue telemetry, or
new thread is part of this issue.
