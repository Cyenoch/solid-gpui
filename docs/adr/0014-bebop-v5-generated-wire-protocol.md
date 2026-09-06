# ADR-0014: Select Bebop v5 as the generated wire protocol

- **Status:** Accepted
- **Date:** 2026-09-01
- **Accepted:** 2026-09-01

## Context

The TypeScript renderer and Rust host previously shared the MessagePack v3 wire
protocol. The next protocol must preserve one semantic protocol boundary across
both languages while reducing the actual bidirectional CPU and allocation cost
of snapshots, patches, commands, and event streams. This is a hard-to-reverse
wire decision, so the choice is based on the final comparison rather than on a
codec-only or one-direction benchmark.
The accepted v5 schema also defines the provider-neutral `Extension` seam
alongside the `Icon` host node. An Extension is selected by the exact catalog
identity tuple `(providerId, catalogDigest, entryId, entryVersion)`; this
decision records the protocol and validation seam, not an Extension-specific
benchmark claim.

The final five-process comparison used five fresh processes per candidate, 20
warm-up rounds, 120 measured rounds, and the actual directions A+B and C+D:
TypeScript semantic encode plus Rust frame decode/materialization, and Rust
Event encode plus TypeScript Event decode/materialization. With scenario weights
of snapshot 30%, patch 25%, Event 30%, and binary command 15%, the performance
composite is:

```
CPU_GM^0.70 * requested-allocation-bytes_GM^0.15
  * additional-peak-live_GM^0.15
```

Bebop is the sole migration candidate that passes the required composite ratio
of at most `0.85` (ratio `0.7964`, a `20.36%` benefit) and the Event p95 guard.
Its Event p95 ratio is `0.7626x` for CPU and `0.7250x` for wall time. Protobuf
reaches only `1.98%` composite benefit and regresses Event p95; FlatBuffers
reaches only `7.62%` composite benefit. MessagePack is the current baseline,
not a migration candidate. The final evidence and calculation inputs are
[the consolidated evaluation](../../.scratch/protocol-benchmark/comparison/evaluation.md)
and the [final comparison result](../../.scratch/protocol-benchmark/comparison/results/comparison-final.json)
(with the [rendered report](../../.scratch/protocol-benchmark/comparison/results/comparison-final.md)).

## Acceptance evidence

The post-cutover production gate was rerun against the optimized production
tree with five fresh processes, 20 warm-up rounds, 120 measured rounds, and
the actual A/B/C/D directions. The [fresh machine-readable result](../../.scratch/protocol-benchmark/bebop/post-cutover/results/post-cutover-final.json)
and [rendered report](../../.scratch/protocol-benchmark/bebop/post-cutover/results/post-cutover-final.md)
supersede the prior failed Event-p95 run.

- Weighted composite: `0.1566x` (limit `<=0.85`).
- Weighted requested allocation bytes: `0.0527x`.
- Weighted additional peak live: `0.0104x` (limit `<=1.00x`).
- Event p95 C+D wall time: `0.8067x` (limit `<=1.10x`).
- Event p95 C+D CPU time: `0.7590x` (limit `<=1.10x`).

The conformance run covered 20,000 Snapshot nodes, 2,048 Patch operations, and
10,000 ordered Events; the recorded digest and payload hash matched across the
fresh producer/consumer checks.

## Decision

Select **Bebop v5** as the accepted wire protocol. The protocol version is
exactly `5`; this is a clean cutover decision, not a proposal to keep two
active decoders.

- Maintain one canonical `.bop` schema as the only wire-schema source of truth.
  Generate the TypeScript and Rust bindings from it, check the generated outputs
  in or otherwise make their exact inputs reproducible, and keep all generated
  types behind the semantic protocol seam. The rest of the renderer and host
  continue to use domain values rather than generated wire types.
- Keep full domain validation handwritten at the semantic boundary. Generated
  Bebop decoding supplies structural parsing; it does not replace validation of
  protocol version, body and union ownership, revisions, topology, capability
  negotiation, resource limits, or publication invariants.
- Include the provider-neutral `Extension` host node in the v5 seam. Resolve
  its exact `(providerId, catalogDigest, entryId, entryVersion)` identity to a
  registered adapter; the default `NoExtensions` registry rejects an unknown
  identity transactionally before publication.
- Cut over directly to v5. Do not add a dual decoder, permissive v4 compatibility
  branch, generic fallback codec, or second wire schema.

## Production acceptance gates

All four gates have been demonstrated against the production protocol and the
fresh post-cutover result is recorded above:

1. **Bounded decoding:** guard the framed input and every repeated, string,
   bytes, and nested value before generated decoding can allocate from an
   attacker-controlled count or length. Reject malformed frames, excessive
   depth or counts, unknown body/union values, and disallowed unknown fields;
   then run the handwritten domain, topology, revision, capability, and
   publication validators.
2. **Reusable Event streaming:** implement a per-connection reusable framed
   Event buffer/writer (for example, a reusable Bebop view or Rust `Vec` with
   in-place frame sizing) and stream or chunk-flush Events without a per-Event
   buffer allocation and copy. The weighted additional peak-live ratio must be
   no worse than the MessagePack baseline (`<=1.00x`).
3. **Full schema and conformance:** extend the single `.bop` schema and both
   generated bindings to every production command, command result, and Event
   family, and cover the complete patch and style/resource surface. Add
   cross-language golden vectors and malformed/resource-limit cases for every
   family; require exact v5 semantics on both sides.
4. **Performance recheck:** rerun the full actual-direction comparison after
   the guards, complete schema, and reusable Event writer are in place. Accept
   only if the performance composite remains `<=0.85` and Event p95 remains
   `<=1.10x` of MessagePack in both CPU and wall-time guard measurements.

The gates are now satisfied; no legacy decoder or fallback path is shipped.
Future wire changes require an explicit versioned decision and fresh
cross-language evidence.

## Alternatives rejected

- **Retain the previous wire protocol:** rejected as the migration choice because
  it offers no migration benefit. It remains only a historical benchmark
  baseline, not a runtime fallback.
- **Protobuf v4:** rejected despite typed generated bindings and lower
  allocation-byte ratios because its actual-direction composite benefit is
  below the 15% requirement and its Event p95 exceeds the 10% regression guard.
  Its TypeScript generator also depends on host `protoc` in the spike.
- **FlatBuffers v4:** rejected because its composite benefit is below the 15%
  requirement. The spike also leaves TypeScript without a generated verifier,
  and the candidate's event and payload sizes regress materially despite an
  Event p95 latency pass.

## Consequences

Bebop introduces a schema/code-generation toolchain and a larger generated
source surface than the current codec, so code generation must be pinned,
hash-checked, and kept out of the normal runtime build. The production adapter
owns the semantic-to-generated mapping, structural guard, domain validation,
and framing; this keeps the wire choice replaceable without leaking generated
Bebop types into the public API.

The schema digest is pinned in
`packages/solid-gpui/src/protocol/schema-lock.json`
(`e0fcd0e7b6c78ce18dce79c5d5d54be7e6fe27a0c717d4b88132ca13f0c2e3b3` for
`protocol.bop`), so checked code generation and semantic adapters share one
verified input.
Rust protocol messages now use typed `CommandMeta`/
`CommandOperation` and `EventMeta`/`EventPayload` DTOs; `Event::event_kind()`
and closed style enums keep union and optional-enum ownership explicit.

The host ownership seam is split between `NativeStateRegistry` and its bounded
`CommitPump`. The TypeScript seam is split between `SurfaceRouter`, `HostTree`,
`CommandClient`, and the `HostKind` facts module. These boundaries keep native
state, tree publication, command completion, frame routing, and host facts
separate without exposing generated records to renderer callers.

The v5 Extension seam keeps provider behavior behind the neutral protocol:
`(providerId, catalogDigest, entryId, entryVersion)` is the exact catalog
identity, while adapters own provider validation and rendering. The empty
`NoExtensions` registry rejects an unknown identity before candidate
publication. The performance evidence above concerns the protocol migration
scenarios recorded there and makes no Extension-specific benchmark claim.

The accepted production result improves the weighted CPU/allocation composite
and satisfies the Event latency and additional peak-live guards despite payload
growth relative to the historical baseline: Snapshot `1.4577x`, Patch `1.7470x`,
Event `2.3247x`, and Command `1.0008x`. The decision accepts those transport-size
regressions for the CPU, allocation, and latency gains; transport size should be
monitored as the protocol and production fixture surface evolve.
