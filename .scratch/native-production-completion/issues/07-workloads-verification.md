# Representative workload verification

Status: ready-for-agent

## Acceptance

Validate keyed large-data updates and native editing invariants, preserve performance work bounds; run appropriate final gates.

## Comments

Implementation and verification evidence will be recorded here.


Passed native 20,000-row list workload through scroll/reorder/append/filter/resize with selection identity and render-count bounds; passed 1000-line editor workload through IME/parent update/resize/stale echo/undo. These establish correctness and CPU work bounds, not presentation timing. Final integrated gates and macOS UI qualification remain.
