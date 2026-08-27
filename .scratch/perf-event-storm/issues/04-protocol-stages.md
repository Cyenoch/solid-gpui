# 04 — Patch build, encode, and submit

**Status: resolved-no-gap.**

## Evidence

Temporary profiling around the one-operation patch path measured the following p50/p99 timings:

| Event | Build | Encode | Submit | Total |
| --- | ---: | ---: | ---: | ---: |
| Scroll | 0.014 / 0.055 ms | 0.014 / 0.087 ms | 0.002 / 0.011 ms | **0.029 / 0.143 ms** |
| Drag-over | 0.007 / 0.020 ms | 0.004 / 0.037 ms | 0.001 / 0.004 ms | **0.013 / 0.056 ms** |

The probe was deleted before the final run. The final Rust smoke test encoded 10,000 events per family within the 2-second guardrail:

| Event | Payload | Framed bytes | 10,000 encodes |
| --- | ---: | ---: | ---: |
| Scroll | 34 B | 38 B | 20.715 ms |
| Drag-over | 20 B | 24 B | 11.654 ms |
| Visible range | 14 B | 18 B | 8.330 ms |
| Layout | 31 B | 35 B | 10.193 ms |

## Resolution

A protocol optimization is not supported by the evidence. The complete measured TypeScript patch stages are at most **0.143 ms p99**, while stateful 10,000-node event delivery is roughly 9 ms at p50. The dominant cost is outside patch construction, encoding, and submission.
