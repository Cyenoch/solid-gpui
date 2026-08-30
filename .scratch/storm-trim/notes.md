# Event-storm sample-budget verdict

## Decision

**No trim.** The test remains at `DURATION_MS = 1_000` (the full one-second
native-rate sample). The 250 ms candidate reached the desired runtime but did
not retain the full sample's p99 safety margin or variance; the 500 ms candidate
was still over the approximately 8 s runtime target. Therefore neither
candidate satisfies the trim gate. The existing 45 s per-test timeout remains
unchanged because it is not the normal-run binding constraint.

**23.6 s is the minimum stable sample; flake exposure is accepted and
documented.** This is a test-budget verdict, not a production performance
claim and not permission to weaken any assertion.

## What consumes the time

`perf-event-storm.test.tsx` makes 20 fresh-root `drive` calls. `drive` sends
`rateHz * DURATION_MS / 1_000` events and, for paced calls, sleeps to the target
native period. The 12 paced scroll/drag calls cover two tree sizes (143 and
10,000) at 60, 120, and 240 Hz. Four unpaced 10,000-node scroll/drag bursts
cover 120 and 240 Hz. The final four calls are the 10,000-node no-op control
and the 143-node layout, visible-range, and pointer-move streams.

The one-second wall segments dominate the small-tree and tail calls. The
10,000-node stateful calls are more expensive than their target wall segment:
React reconciliation/host commit-diff work makes the 240 Hz paced and burst
rounds run for roughly 1.8--2.0 s each. On baseline run 1, measured `elapsedMs`
sums were:

| Segment | Calls/run | Events/run | Measured elapsed sum |
| --- | ---: | ---: | ---: |
| 143-node paced scroll + drag | 6 | 840 | 5,951.4 ms |
| 10,000-node paced scroll + drag | 6 | 840 | 7,684.1 ms |
| 10,000-node unpaced bursts | 4 | 720 | 5,627.7 ms |
| no-op control | 1 | 240 | 0.4 ms |
| layout + visible-range + pointer-move | 3 | 720 | 2,989.2 ms |
| **sum of drive elapsed values** | **20** | **2,520** | **22,252.8 ms** |

The remaining roughly 1.2 s is root setup/teardown and runner overhead. Thus
the time sink is fixed one-second pacing plus the 10,000-node stateful event
rounds, not a hidden unbounded loop. At the full sample the pointer-move round
specifically sends 240 events and produces 240 commits; its guard is
counter-primary (expected commits and output bytes), with `transport.push`
p50/p99 latency as the secondary `< 100 ms` guard.

## Method

I first ran the restored full test once to print the asserted metrics, then ran
three sample budgets five times each. Every run passed with the existing
assertions unchanged. For each scenario, the table records the five-run p99
mean, sample standard deviation, and observed range. `events` and `commits`
were exact deterministic counters; no threshold or assertion was changed.

- Full: `DURATION_MS = 1_000`; 23.35--23.78 s wall/run (the readiness
  reproduction separately recorded 23.61 s).
- Half: `DURATION_MS = 500`; 12.17--12.45 s wall/run.
- Quarter: `DURATION_MS = 250`; 6.77--6.82 s wall/run.

The count triplets in the table are full/half/quarter. For paced 60/120/240 Hz
calls they are 60/30/15, 120/60/30, and 240/120/60. The unpaced burst rows use
the corresponding 120/60/30 or 240/120/60 triplets. All non-no-op rows retained
the expected one-commit-per-event behavior, except layout and visible-range,
which retained one fewer commit; the no-op row retained zero commits.

## p99 variance table

All values are milliseconds, shown as `mean +/- sample SD (min--max)` over five
runs. The assertion is `p99 < 100 ms`; the full-sample maximum observed p99 was
13.387 ms (7.47x below the threshold). The quarter-sample maximum was 14.115 ms
(7.08x below), so its relative safety margin was smaller, while its pointer-move
p99 SD was 0.793 ms versus 0.227 ms for full and its range widened to
0.789--2.396 ms from 0.762--1.338 ms.

| Scenario | Events full/half/quarter | Commits full/half/quarter | Full p99 | Half p99 | Quarter p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| drag-over / 143 / 60 Hz | 60/30/15 | 60/30/15 | 1.782 +/- 1.113 (1.005--3.742) | 2.179 +/- 1.098 (1.136--3.579) | 1.331 +/- 0.058 (1.283--1.424) |
| drag-over / 143 / 120 Hz | 120/60/30 | 120/60/30 | 1.173 +/- 0.593 (0.763--2.205) | 1.890 +/- 1.266 (0.789--3.527) | 0.720 +/- 0.097 (0.584--0.816) |
| drag-over / 143 / 240 Hz | 240/120/60 | 240/120/60 | 0.861 +/- 0.090 (0.768--0.966) | 1.027 +/- 0.329 (0.696--1.526) | 1.210 +/- 0.745 (0.630--2.453) |
| drag-over / 10,000 / 60 Hz | 60/30/15 | 60/30/15 | 11.684 +/- 1.253 (10.224--13.387) | 11.126 +/- 0.193 (10.920--11.375) | 12.129 +/- 1.575 (10.582--14.115) |
| drag-over / 10,000 / 120 Hz | 120/60/30 | 120/60/30 | 11.121 +/- 0.347 (10.602--11.461) | 9.566 +/- 0.421 (8.998--9.914) | 9.881 +/- 0.646 (9.129--10.906) |
| drag-over / 10,000 / 240 Hz | 240/120/60 | 240/120/60 | 10.252 +/- 0.289 (9.989--10.691) | 10.142 +/- 0.369 (9.773--10.658) | 10.865 +/- 0.440 (10.301--11.354) |
| drag-over-burst / 10,000 / 120 Hz | 120/60/30 | 120/60/30 | 10.404 +/- 0.415 (10.028--11.014) | 10.366 +/- 0.557 (9.835--10.981) | 10.871 +/- 1.164 (9.756--12.438) |
| drag-over-burst / 10,000 / 240 Hz | 240/120/60 | 240/120/60 | 10.168 +/- 0.413 (9.841--10.834) | 10.199 +/- 0.544 (9.395--10.914) | 11.254 +/- 1.064 (10.396--13.082) |
| layout / 143 / 240 Hz | 240/120/60 | 239/119/59 | 0.844 +/- 0.115 (0.669--0.954) | 0.718 +/- 0.121 (0.542--0.850) | 0.506 +/- 0.266 (0.371--0.982) |
| pointer-move / 143 / 240 Hz | 240/120/60 | 240/120/60 | 0.992 +/- 0.227 (0.762--1.338) | 1.439 +/- 0.660 (0.920--2.273) | 1.734 +/- 0.793 (0.789--2.396) |
| scroll / 143 / 60 Hz | 60/30/15 | 60/30/15 | 2.896 +/- 0.688 (2.220--3.984) | 2.906 +/- 0.498 (2.288--3.634) | 3.618 +/- 0.372 (3.189--4.156) |
| scroll / 143 / 120 Hz | 120/60/30 | 120/60/30 | 1.234 +/- 0.232 (1.011--1.595) | 1.306 +/- 0.179 (1.129--1.531) | 2.112 +/- 0.475 (1.581--2.736) |
| scroll / 143 / 240 Hz | 240/120/60 | 240/120/60 | 0.737 +/- 0.065 (0.672--0.810) | 1.652 +/- 0.456 (1.035--2.136) | 1.197 +/- 0.250 (0.910--1.534) |
| scroll / 10,000 / 60 Hz | 60/30/15 | 60/30/15 | 11.488 +/- 0.633 (10.935--12.496) | 12.128 +/- 0.294 (11.711--12.540) | 11.092 +/- 0.470 (10.485--11.660) |
| scroll / 10,000 / 120 Hz | 120/60/30 | 120/60/30 | 10.058 +/- 0.702 (9.400--11.146) | 10.434 +/- 0.670 (9.363--11.022) | 10.253 +/- 0.765 (9.504--11.122) |
| scroll / 10,000 / 240 Hz | 240/120/60 | 240/120/60 | 10.179 +/- 0.693 (9.765--11.384) | 9.802 +/- 0.578 (9.091--10.568) | 10.863 +/- 0.882 (9.779--12.172) |
| scroll-burst / 10,000 / 120 Hz | 120/60/30 | 120/60/30 | 10.117 +/- 0.294 (9.703--10.509) | 10.220 +/- 0.376 (9.868--10.748) | 11.178 +/- 0.571 (10.709--12.132) |
| scroll-burst / 10,000 / 240 Hz | 240/120/60 | 240/120/60 | 10.298 +/- 0.314 (9.878--10.622) | 9.879 +/- 0.189 (9.580--10.097) | 11.140 +/- 0.400 (10.520--11.550) |
| scroll-noop / 10,000 / 240 Hz | 240/120/60 | 0/0/0 | 0.011 +/- 0.008 (0.004--0.024) | 0.011 +/- 0.001 (0.010--0.013) | 0.036 +/- 0.004 (0.031--0.041) |
| visible-range / 143 / 240 Hz | 240/120/60 | 239/119/59 | 0.564 +/- 0.059 (0.525--0.668) | 0.632 +/- 0.094 (0.524--0.745) | 0.703 +/- 0.148 (0.538--0.854) |

## Gate result

- Assertion thresholds were left untouched.
- No candidate met all trim conditions: half duration failed the approximately
  8 s runtime condition; quarter duration failed the same-relative-margin and
  variance condition despite meeting the runtime condition.
- The five full, five half, and five quarter focused runs all passed. The
  restored source has no diff from `HEAD`; the full-suite and repository gates
  are recorded in the delivery report.
