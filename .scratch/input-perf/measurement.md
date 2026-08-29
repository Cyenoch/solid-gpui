# TextInput Typing Performance Audit

## Method

The established headless GPUI renderer test harness measures 32 individual
character insertions after one warm draw. Each insertion dispatches one native
character input and records the elapsed time for the resulting input event and
redraw. Test-only counters cover input content extraction, run construction, and
text shaping; test-only timers attribute their total cost across the 32 edits.
Percentiles use the rank convention already used by the rich-text audit.

## Measurements

| Document | Layout | p50 / p99 per keystroke (ms) | Content assemblies | Run assemblies | Shape calls |
| ---: | --- | ---: | ---: | ---: | ---: |
| 100 chars | single-line | 0.208 / 0.541 | 33 | 33 | 33 |
| 1,000 chars | single-line | 0.343 / 0.441 | 33 | 33 | 33 |
| 10,000 chars | single-line | 1.252 / 1.586 | 33 | 33 | 33 |
| 1,000 chars | multiline/wrapped | 0.240 / 0.267 | 33 | 33 | 33 |

The count of 33 includes the warm draw and the 32 measured keystrokes. Thus,
content extraction, run construction, and shaping each occur once per draw and
once per keystroke. The unrelated style-patch scenario also performs the
expected redraw and records bounded input-path calls (2 total including the
warm draw), without a text patch.

## Attribution

| Document | Content extraction total (ms) | Run construction total (ms) | Shape total (ms) | Shape share |
| ---: | ---: | ---: | ---: | ---: |
| 100 chars | 0.015 | 0.004 | 0.399 | 95.5% |
| 1,000 chars | 0.013 | 0.003 | 1.345 | 98.8% |
| 10,000 chars | 0.015 | 0.003 | 7.303 | 99.8% |
| 1,000 chars multiline/wrapped | 0.006 | 0.002 | 2.500 | 99.7% |

The host-side content and run work is effectively flat at these sizes and is
negligible relative to platform shaping. Single-line shaping grows with the
whole changed line: the 10,000-character case is about 3.6x the 1,000-character
p50 and 3.6x its shape total. Wrapped 1,000-character input follows the pinned
platform's wrapped layout path and does not show the same single-line growth.

## Verdict and boundary

**No host-side defect; guard-only.** `UPDATE_TEXT` patches are scoped to the
text-input host property, `NativeInputState` updates the text and selection, and
the next GPUI prepaint intentionally rebuilds the display string/run and asks
the pinned text system to shape the current content. The measured host
reassembly work is not the source of the typing cost, so adding a host cache
would not address the observed jank.

At 10,000 characters, single-line input is above the comfortable ~0.3 ms p99
per-keystroke share used by this audit, but attribution identifies the pinned
platform's whole-line layout miss as the cause. **Wontfix in this host:**
without wrapping, a changed single line has inherent O(n) shaping per keystroke
because its LineLayoutCache key changes with the complete line. This is a
pinned platform-layer boundary, not an avoidable flatten/run-assembly defect.
Wrapped input is already handled by the platform's wrapped layout cache path;
the host reassembly cost remains negligible.

The regression guard intentionally asserts bounded content/run/shape call
counts per keystroke and emits p50/p99 plus attribution values, so a future
host-side reassembly regression is visible even if the platform shaping cost
remains unchanged.
