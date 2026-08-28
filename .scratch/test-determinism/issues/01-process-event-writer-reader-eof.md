# 01 — Process event writer closes child after stdin EOF

Status: resolved

## Baseline

`cargo test -p react-gpui process_event_writer_kills_closed_stdin_child_for_reader_eof -- --exact`
was run for 20 iterations before the hardening change: **20 pass, 0 fail**.
The historical failure was therefore **unreproducible under 20 iterations;
hardened by inspection**.

## Change

The process EventWriter now exposes a bounded completion condition that is
signaled after retaining the writer failure and running the child-stop callback.
The regression test waits on that condition before waiting for the commit reader
to return EOF. It still asserts the writer error, `RuntimeStatus::Failed`,
rejection of a later send, and clean shutdown.

## After evidence

The exact focused command was run for 20 iterations after the change:

- **20 pass**
- **0 fail**
- The loop completed in approximately 5.84 seconds.

No sleep-based synchronization was added to the test; both waits are bounded.
