# Crash report path must cross the process boundary

Status: ready-for-agent
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
