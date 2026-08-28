# 03 — Embedded candidate smoke fixed timeout

Status: resolved

## Baseline

The historical first smoke run failed because the helper treated the fixed
five-second `process.communicate(timeout=5.0)` lifetime as the success window:

```text
react-gpui-host: embedded smoke press sent=true, commits=1, status=None
embedded candidate timed out after 5.010s
```

A subsequent five-run baseline loop passed 5/5, so the timing failure was
intermittent rather than reproduced on every run.

## Change

The helper now polls the existing info diagnostic for
`embedded smoke press sent=true, commits>=1` with a bounded ten-second deadline.
Once a committed Snapshot is observed, it terminates the intentionally
long-lived candidate and retains the expected exit-124, startup, version, and
help assertions. Child stderr is kept in a separate file while polling so the
parent's redirected stderr does not interfere with observation.

## After evidence

`bash scripts/host-embedded-candidate-smoke.sh` was run five times after the
change:

- **5 pass**
- **0 fail**
- Each run observed `commits=1`; observed intentional timeout messages ranged
  from approximately 4.46s to 4.74s.

The script's shell syntax also passed `bash -n`.
