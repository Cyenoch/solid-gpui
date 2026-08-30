# Kill resilience

## Teardown semantics

- `packages/react-gpui/src/transport.ts:162-215` constructs `StdioTransport` over process stdio. The input stream's `end` and `close` events both call `terminate(..., { kind: "eof" })`; termination is idempotent, detaches listeners, clears pending frames, and notifies `onTermination` listeners. `createProcessTerminationHandler` at `:146-150` prints the typed error and exits with status 1.
- The renderer examples use `createProcessTerminationHandler()` in `createRoot` (for example `packages/react-gpui/examples/counter.tsx:40-47` and `gallery.tsx:646-655`). No keep-alive timer exists in either example. The `stress.tsx` interval is cleared by its effect cleanup and is not part of this verification.
- `crates/react-gpui/src/transport.rs:364-461` owns the process child and polls its `Child` with `try_wait`; `RuntimeStatus::Exited` includes exit code/signal. `ProcessAdapter::recv_commit` reads framed stdout at `:535-542`.
- `crates/react-gpui/src/renderer/commit_reader.rs:20-99` runs the blocking commit reader on a dedicated thread. EOF becomes `ReaderMessage::Terminated`; a failure status calls `fatal_runtime_failure`, which logs the status, requests runtime shutdown, and exits 1.
- The host's equivalent reader is `crates/react-gpui-host/src/main.rs:723-795`. On `Terminated`, it closes all surfaces and calls `fatal_runtime_failure` for failure statuses. A renderer SIGKILL therefore reaches the typed `RuntimeStatus::Exited { signal: 9 }` path.
- The process host captures the renderer stdin pipe in `ProcessAdapter::spawn` (`crates/react-gpui/src/transport.rs:473-505`). When the host is SIGKILLed, the OS closes that pipe; Bun's `process.stdin` emits `end`/`close`, and the transport termination callback exits the renderer.

## Verification command

```sh
make kill-resilience
```

The independent `scripts/kill-resilience.sh` target deliberately does not alter the normal `examples-smoke` path. It builds the release host, launches each selected example in a fresh scratch working directory and session/process group, records host/renderer PIDs, and verifies each target's cwd, command, and pgid immediately before every signal. It defaults to `counter.tsx,gallery.tsx`; override with `KILL_RESILIENCE_ENTRIES` for a narrower run. Watchdogs default to 10 seconds for startup and 5 seconds for post-kill observation.

## Observed run

Command:

```text
KILL_RESILIENCE_STARTUP_TIMEOUT_SECONDS=10 KILL_RESILIENCE_KILL_TIMEOUT_SECONDS=3 bash scripts/kill-resilience.sh
```

Observed output:

| Entry | Kill target | Observed outcome | Latency | Verdict |
| --- | --- | --- | ---: | --- |
| counter | host PID (`SIGKILL`) | renderer gone; stdin EOF cause | 0.058 s | PASS |
| counter | Bun renderer PID (`SIGKILL`) | host exited 1; typed `renderer runtime terminated unexpectedly` / `terminated by signal 9` diagnostic | 0.011 s | PASS |
| counter | process group (`SIGKILL`) | host and renderer gone together; 2 verified scratch members | 0.008 s | PASS |
| gallery | host PID (`SIGKILL`) | renderer gone; stdin EOF cause | 0.058 s | PASS |
| gallery | Bun renderer PID (`SIGKILL`) | host exited 1; typed `renderer runtime terminated unexpectedly` / `terminated by signal 9` diagnostic | 0.011 s | PASS |
| gallery | process group (`SIGKILL`) | host and renderer gone together; 2 verified scratch members | 0.008 s | PASS |

The first host-PID run measured 0.057 s for counter; the repeated two-entry run above is the recorded matrix.

## Verdict

- Host single-PID force-kill: **clean**. The renderer exits promptly after stdin EOF; no orphan was observed.
- Renderer single-PID force-kill: **clean**. Host exits promptly through its existing typed child-death path with status 1.
- Process-group force-kill: **clean**. Both verified scratch processes are removed together; this is behaviorally different from single-PID kill because it does not exercise pipe-driven renderer cleanup.
- No production transport fix was needed. The existing seam is unconditional: stdin `end`/`close` enters transport termination, and the process termination handler exits. The verification is kept out of `make ci` because it launches display-backed examples and is a separate bounded resilience smoke target.
