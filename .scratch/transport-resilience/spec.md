# Transport resilience

This effort hardens the JavaScript/native process boundary without adding a
resynchronization protocol or silently recovering a failed stream. It covers
host crash diagnostics, malformed host-to-renderer frames, typed termination
causes, pending-command rejection, and the documented restart pattern.

## Current-state audit

- The host panic hook writes a versioned report under
  `REACT_GPUI_CRASH_DIR` (or the system temporary directory), but
  `write_crash_report` discards the generated path and the hook emits no
  machine-readable path marker (`crates/react-gpui-host/src/main.rs:82-145`).
  `ProcessAdapter` inherits child stderr instead of capturing it
  (`crates/react-gpui/src/transport.rs:422-443`), so a renderer-side
  `StdioTransport` can only surface a report path if the host prints one and
  the process wrapper forwards stderr as an input error diagnostic.
- `TransportTerminatedError.cause` is currently `unknown`; termination details
  only carry optional `exitCode` and a bounded `stderrTail`
  (`packages/react-gpui/src/transport.ts:7-24,68-90`). Consumers cannot branch
  on shutdown/EOF/exit/protocol failure without parsing the message.
- `FrameDecoder.push` fails only for an oversized declared length and resets its
  framing buffer (`packages/react-gpui/src/protocol.ts:378-428`). MessagePack
  decode failures are intentionally contained as `null`
  (`packages/react-gpui/src/protocol.ts:430-436`), and `decodeEvent` returns
  `null` for malformed event frames (`packages/react-gpui/src/protocol.ts:761-797`).
  Both receive paths silently discard that result or any decoder exception:
  `RootContainer.receive` (`packages/react-gpui/src/renderer/root-container.ts:767-777`)
  and `SurfaceHostImpl.receive` (`packages/react-gpui/src/surface-host.ts:158-170`).
  This leaves the transport endpoint accepting later sends while the shared
  protocol state is no longer provable.
- Transport termination itself already fans out to pending commands on a root
  (`packages/react-gpui/src/renderer/root-container.ts:142-150`) and to every
  routed root (`packages/react-gpui/src/surface-host.ts:173-191`). Existing
  tests cover surface close, unmount, host disposal, and a two-root termination
  sample (`packages/react-gpui/tests/renderer.test.tsx:1902-1942`,
  `packages/react-gpui/tests/surface-host.test.ts:176-206`), but the transport
  death test does not exercise every root command family.
- The existing protocol fuzz tests assert decoder containment/no panic, not
  transport survival; changing transport behavior after a decoder returns
  `null` does not conflict with that scope (`packages/react-gpui/tests/protocol-fuzz.test.ts:101-127`).

## Chosen designs

1. **Crash report bridge.** After a successful panic report write, the host
   prints one stable stderr line, `react-gpui-host: crash report: <path>`.
   The JavaScript transport retains the bounded stderr tail, extracts the last
   marker line, exposes it as `crashReportPath`, and includes it in the
   contextual message. No new wire event is invented: the panic is host-fatal,
   stderr is already the process diagnostic channel, and the existing stream
   wrapper is the only seam that can correlate the report with exit status.
2. **Fail-fast malformed frames.** A framed payload that cannot decode to a
   valid event, or a framing decoder failure, creates a
   `TransportTerminatedError` with `{ kind: "protocol", detail }`. Receive paths
   mark their root/host transport endpoint terminated and reject all pending
   commands. Shared `SurfaceHost` roots receive the same terminal error. Later
   sends reject; no payload is skipped and no resynchronization is attempted.
   Direct `FrameDecoder` fuzz behavior remains a contained decoder/no-panic
   contract.
3. **Typed cause.** Export `TransportTerminationCause` as a discriminated
   union: shutdown, EOF, non-zero exit (with integer code), protocol failure
   (with detail), and I/O failure (with detail). `TransportTerminatedError.cause`
   stores this union; `crashReportPath` is an additive diagnostic detail. Raw
   stream errors are represented honestly as I/O causes unless an exit code is
   present. Existing message text, exit code, and stderr tail remain available.
4. **Restart.** A terminated `Transport`/`Root` is not reusable. The package
   troubleshooting section documents supervising the child process, creating a
   fresh `StdioTransport` and `createRoot` after the child closes, and rendering
   the application again. No global root/transport state or retry timer is
   introduced.

No new ADR is needed: these choices apply ADR-0008's existing fail-fast
transport/protocol seam and keep crash diagnostics on the existing stderr
channel.
