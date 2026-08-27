# Crash report path must cross the process boundary

Status: resolved
Type: task

The host already writes a versioned panic report, but the generated path is not
reported to the renderer-side transport. A production consumer therefore sees
only an exit/stderr diagnostic and must search the configured crash directory.

Evidence: `crates/react-gpui-host/src/main.rs:82-145` (panic hook and report
writer discard the path), `crates/react-gpui/src/transport.rs:422-443`
(`ProcessAdapter` inherits child stderr), and
`packages/react-gpui/src/transport.ts:68-90` (only exit code/stderr tail are
retained).

## Design

After `write_crash_report` flushes successfully, emit exactly one stable stderr
line: `react-gpui-host: crash report: <path>`. Keep the existing panic output and
versioned report contents. Parse the last matching line from the bounded stderr
tail and expose `crashReportPath` on `TransportTerminatedError` and its
termination details. Include the path in the human-readable message. Do not
add a wire event: stderr is the existing host diagnostic channel and the panic
is host-fatal, so no protocol recovery or extra event ordering is appropriate.

## Acceptance

- A host panic report test proves the writer returns the generated path and the
  hook emits the marker after a successful write.
- A renderer-side transport termination test with non-zero exit diagnostics and
  the marker proves `error.exitCode`, `error.stderrTail`, and
  `error.crashReportPath` survive together and the path appears in the message.
- An unwritable crash directory still reports the existing write failure and
  does not claim a path.

## Comments

- Implemented in `crates/react-gpui-host/src/main.rs:89-149`: a successful
  report write now emits `react-gpui-host: crash report: <path>` after flush;
  the existing panic output and versioned report are unchanged.
- Implemented in `packages/react-gpui/src/transport.ts:95-141`: the bounded
  stderr tail extracts the marker into `crashReportPath` and includes it in the
  termination message. `packages/react-gpui/tests/transport.test.ts:229-247`
  proves the marker, exit code `23`, tail bound, typed exit cause, and path.
- `cargo test -p react-gpui-host panic_hook_writes_crash_report_with_location
  --locked -- --nocapture` observed the marker and report path; the focused
  TypeScript transport suite passes. A panic hook write failure still emits
  only the existing write diagnostic because no path exists.
