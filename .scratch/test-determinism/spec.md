# Test determinism

Status: resolved

## Scope

Eliminate three gate flakes without weakening their observable contracts:

1. The process transport writer must stop a child whose stdin has closed and
   allow the commit reader to observe EOF.
2. A failed optional protocol-tap path must not poison later tap instances in
   the same Bun process.
3. Embedded candidate smoke must wait for the existing committed Snapshot
   diagnostic instead of treating a fixed five-second process lifetime as the
   success condition.

## Decisions

- The transport test waits on an explicit EventWriter completion condition,
  bounded by a finite deadline, then waits for reader EOF. It retains checks
  for the writer error, failed runtime status, rejected later sends, and clean
  shutdown. The failure was **unreproducible under 20 iterations; hardened by
  inspection** before this change.
- ProtocolTap opening failure is instance-local. Every `fromEnv()` call gets a
  fresh attempt; a failed instance does not disable future transports. A test
  exercises failure followed by a valid tap in one Bun process.
- The embedded smoke helper polls the existing `commits>=1` info diagnostic for
  a bounded ten-second deadline, then terminates the intentionally long-lived
  candidate and preserves the existing exit-124 and diagnostic assertions.

## Verification contract

The exact after stress loops are recorded in the issue files in this
 directory. The parent workstream owns full `make ci` and `make embedded-bun`
 integration gates after concurrent host/renderer edits settle.
