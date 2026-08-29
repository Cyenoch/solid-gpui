# TypeScript Snapshot Encode Performance

## Scope and method

The TypeScript side was measured against the same mixed-node shape as the native snapshot audit: a root `View`, styled `View`/`Text` nodes, nested `Text` runs, one `TextInput`, one `VirtualList` with visible rows, and filler `View`/`Text`/`Pressable` nodes. The test assembles the wire-node array with `NodeGraph.snapshotNodes()`, encodes the complete V3 frame with `encodeFrame`, and submits it through `MemoryTransport` (including the transport's frame copy). Each size has eight runs after one warm-up pass; percentiles use the existing rank convention (`ceil(N × q) - 1`).

The TypeScript initial commit uses this same path in `RootContainer.commit`: `snapshotNodes()` followed by `encodeFrame([PROTOCOL_VERSION, SNAPSHOT_KIND, ...])` and `transport.submit`.

## Scaling measurements

Captured on the first local run of `bun test tests/snapshot-encode-perf.test.ts`:

| Tree size | Nodes | Construction p50 / p99 (ms) | Encode p50 / p99 (ms) | Total p50 / p99 (ms) |
| ---: | ---: | ---: | ---: | ---: |
| 1,000 | 1,000 | 0.054 / 0.202 | 0.255 / 1.092 | 0.311 / 0.518 |
| 5,000 | 5,000 | 0.190 / 0.224 | 0.867 / 1.605 | 1.080 / 1.986 |
| 20,000 | 20,000 | 0.945 / 1.398 | 2.918 / 3.694 | 3.759 / 5.612 |

The measured 20k total p50 is approximately 3.5x the 5k total p50, while node count is 4x; this is close to linear scaling for this shape. Construction and MessagePack encoding are both small relative to the host and draw costs in the native audit.

## Wire-size measurements

The frame byte count includes the four-byte little-endian length prefix used by the transport. `MAX_FRAME_SIZE` is 16,777,216 payload bytes (16 MiB); percentages below compare the MessagePack payload (`frameBytes - 4`) with that limit.

| Tree size | Frame bytes | Payload bytes | Payload MiB | MAX_FRAME_SIZE |
| ---: | ---: | ---: | ---: | ---: |
| 1,000 | 21,533 | 21,529 | 0.021 | 0.128% |
| 5,000 | 106,885 | 106,881 | 0.102 | 0.637% |
| 20,000 | 429,296 | 429,292 | 0.410 | 2.559% |

A 20k snapshot is therefore about 0.429 MB on the wire (0.410 MiB payload), well below the 16 MiB frame limit. The limit would become relevant only for substantially larger or much more metadata-heavy snapshots.

## Verdict

**TS total encode-and-handoff 3.759 ms p50 (5.612 ms p99) + host snapshot stages approximately 71–78 ms + GPUI first draw approximately 196 ms = approximately 271–278 ms for the measured 20k boot shape; TS encoding is not the boot bottleneck, while the fully visible 20k draw remains dominant.**

## Sanity guard

The test asserts only that the 20k total p50 remains below 150 ms. The first measured p50 was 3.759 ms, so this leaves over 39x headroom for CI machine and runtime variance while still catching an accidental order-of-magnitude regression. No p99 wall-clock threshold is asserted because a single scheduler pause can perturb this short eight-sample run.

## Reproduction

```text
bun test tests/snapshot-encode-perf.test.ts
```
