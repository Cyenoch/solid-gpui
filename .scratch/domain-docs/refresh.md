# Domain documentation refresh

## What changed and why

`CONTEXT.md` now covers the vocabulary introduced after the original eight ADRs:
Surface Retirement; Wire Tail Slots; Pointer Coordinates and Pointer-Move
Capability; Drag Capability Bits; Overlay Outside-Dismissal Capture; Close
Policy and Close Request/Resolve; Tab-stop Capability; Host-Owned Input Model
and Host-Owned Undo History; Caret Follow/Scroll Offset; Runtime Resource
Commands; JSC Single-VM Isolation; and the Font Registration Timing Invariant.
Each entry stays terse, names the avoided synonym where useful, and points to
an implementation, protocol section, scratch decision, or ADR.

ADR-0008's implementation references were updated from stale line-numbered
monolith paths to the current split module paths. The changelog records the
refresh under `Changed`.

## New ADRs

These four decisions meet the ADR test: each is difficult to reverse, surprising
without context, and has meaningful alternatives and consequences.

- **ADR-0009 — Async close confirmation across the process boundary:** the host
  holds per-Surface close policy, synchronously vetoes when confirmation is
  required, and uses one request event plus one resolve command rather than
  blocking native code or pretending JavaScript can cancel synchronously.
- **ADR-0010 — Opt in to high-frequency event streams:** pointer movement is a
  per-node capability; absent handlers register no native stream, preserving
  measured performance and keeping pointer move distinct from hover and drag.
- **ADR-0011 — Keep one wire form with optional node tails:** current optional
  suffixes and unambiguous placeholders are allowed, but historical arities and
  default-filling compatibility branches are removed; required fields remain
  required.
- **ADR-0012 — Own input models at the host when GPUI has no primitive:** the
  host owns bounded text/selection/geometry/undo state and emits existing events,
  while unsupported secure/password behavior remains an explicit gap.

## Candidates rejected as ADR material

- **Surface identity and typed close errors:** important vocabulary and public
  lifecycle semantics, but the decision is a focused lifecycle contract already
  represented by `Surface Retirement` in `CONTEXT.md`, `.scratch/surface-lifecycle/`,
  and the protocol/consumer docs; it does not need a fifth ADR beside close
  confirmation.
- **Focus traversal and focus-handle tab stops:** the native capability is
  load-bearing, but it is a direct correction to GPUI's required registration
  seam and is sufficiently captured by the Tab-stop Graph/Capability glossary,
  ADR-0005, and `.scratch/focus-order/`; adding another ADR would split one
  focus decision across records.
- **Overlay outside dismissal:** capture-phase scope is a precise event contract,
  not a durable architectural choice with competing system designs; it remains
  in the glossary and protocol directory with implementation evidence.
- **Pointer down/up coordinates:** required coordinates follow the existing GPUI
  event data and have no meaningful legacy alternative; `.scratch/pointer-coords/`
  and `docs/protocol.md` record the wire rule. The architectural trade-off is
  the separate opt-in stream, which is ADR-0010.
- **Drag capability bits:** independent source/target booleans are a compact
  feature shape, not an enduring cross-boundary decision; the glossary and
  protocol docs capture the contract.
- **Caret-follow viewport behavior:** a local rendering invariant with no wire
  change, documented by `.scratch/caret-follow/reproduction.md` and the
  Host-Owned Input Model ADR rather than a standalone architecture record.
- **File, clipboard, image, and font commands:** bounded command additions and
  platform capability matrices are feature contracts. Their common ownership
  vocabulary is in `Runtime Resource Commands`; no single alternative set needs
  a separate ADR.
- **Window controls and appearance/theme decisions:** platform-specific API
  coverage and application policy, respectively; they are documented in
  `docs/protocol.md`, the package README, and Unreleased notes without a
  durable architectural fork.
- **Selectable Text alone:** it is a host-owned input-model instance, and its
  selection feasibility/review remains in `.scratch/interchange/` and the
  Host-Owned Input Model ADR; a standalone ADR would duplicate that model.
- **VirtualList migration and accessibility tails:** important behavior and wire
  details, but each is a bounded feature/schema extension without the three ADR
  criteria once the current protocol reference is authoritative.
- **Protocol tap metrics and event-storm measurements:** observability and
  evidence, not a chosen system architecture. They remain in the tap/performance
  scratch records and existing docs.
