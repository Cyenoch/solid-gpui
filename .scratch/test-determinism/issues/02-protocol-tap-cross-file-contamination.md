# 02 — Protocol tap state leaks across tests

Status: resolved

## Baseline

The isolated renderer test file passed 10/10 runs. In one Bun process with
`--parallel=1 --no-isolate --randomize`, the combined
`tests/renderer.test.tsx tests/protocol-tap.test.ts` loop failed 10/20 seeds.
The failure was in `protocol tap > records complete frames across
fragmented/coalesced input without payload bytes`: its temporary tap file was
missing (`ENOENT`). An earlier invalid-path tap had permanently disabled later
opens through module-global failure flags.

## Change

`ProtocolTap.fromEnv()` no longer uses module-global `openFailureDisabled` or
`openFailureWarned` state. A failed open is local to that tap instance, so a
later transport in the same Bun process attempts its configured path normally.
A regression test creates a failed tap, then a valid tap, and verifies one
record is written to the valid path.

## Focused after evidence

`bun test tests/protocol-tap.test.ts --timeout=10000`:

- **5 pass**
- **0 fail**
- Includes the failure-then-valid-follow-up regression.

## Randomized after evidence

The required combined command was run for seeds 1 through 20 after the
selectable tuple repair:

`bun test tests/renderer.test.tsx tests/protocol-tap.test.ts --parallel=1 --no-isolate --randomize --seed=<seed> --timeout=10000`

- **20/20 clean runs**; **0 fail**
- **No compile failures**
- No run reproduced the original protocol-tap `ENOENT` failure.

After the later shared close/tooltip fixes restored window observation handler
assignments and the malformed command validator guard, the explicit full
12-file suite was also run once with seed 1:

- **112 pass**, **0 fail**, **53,226 expects**
- Files: renderer, protocol-tap, surface-host, todo, perf-budget,
  protocol-golden, protocol-fuzz, perf-event-storm, transport, stress,
  api-surface, and appearance tests.

The earlier baseline had 10/20 failures, all at the protocol-tap `ENOENT`
test; no clean after run reproduced that contamination failure.
