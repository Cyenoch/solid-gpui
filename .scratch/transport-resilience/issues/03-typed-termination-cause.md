# Transport termination causes need programmatic discrimination

Status: ready-for-agent
Type: task

`TransportTerminatedError.cause` is currently `unknown`; the changelog's
shutdown/EOF/non-zero/protocol taxonomy exists only in contextual strings and
optional diagnostic fields. Consumers must parse `.message` to choose recovery
or reporting behavior.

Evidence: `packages/react-gpui/src/transport.ts:7-24` (`cause?: unknown` and
untyped details), `packages/react-gpui/src/transport.ts:64-90` (string-based
termination formatting), and the package public export list
(`packages/react-gpui/src/index.ts:75-93`) which exports the error but no cause
type. Existing termination tests assert message text only
(`packages/react-gpui/tests/transport.test.ts:194-242`).

## Design

Export `TransportTerminationCause` as a discriminated union with honest causes:
`{ kind: "shutdown" }`, `{ kind: "eof" }`, `{ kind: "exit", code }`,
`{ kind: "protocol", detail }`, and `{ kind: "io", detail }`. Store the union
in `TransportTerminatedError.cause` while preserving existing message,
`exitCode`, and `stderrTail` fields. Additive termination details include
`crashReportPath`; add no wire fields and no string-parsing requirement for
consumers. Map a supplied integer exit code to `exit`, stream end/close to
`eof`, protocol decoder failures to `protocol`, and other stream failures to
`io`; explicit disposal/host shutdown uses `shutdown` where a termination error
is needed.

## Acceptance

- TypeScript declarations and API-surface fixtures expose the cause union.
- Focused tests discriminate protocol, EOF, exit, and I/O causes without
  parsing messages, including the crash-report path alongside exit diagnostics.
- Existing termination callback and message behavior remains compatible.
