# Documentation index

Use this reading order to find the repository's product overview, contracts, architecture decisions, contributor workflow, and verification evidence.

## Start here

- [Root README](../README.md) — public overview, quick start, workspace map, and supported-surface boundaries.
- [Getting started](getting-started.md) — pinned toolchain, installation, host commands, and the first consumer application.
- [Troubleshooting](troubleshooting.md) — symptom-first diagnosis and repair for setup, runtime, protocol, and package failures.

## Reference

- [Protocol reference](protocol.md) — framed protocol-v3 messages, validation, ownership, fixtures, and evolution rules.

## Understanding the architecture

- [CONTEXT.md glossary](../CONTEXT.md) — shared vocabulary for surfaces, commits, events, runtime adapters, native state, and protocol seams.
- [ADR-0001: Batch React commits into GPUI](adr/0001-batch-react-commits-into-gpui.md) — make each React commit one atomic Snapshot or Patch across the runtime boundary.
- [ADR-0002: Embed the Bun runtime on a dedicated thread](adr/0002-embedded-bun-runtime.md) — isolate Bun/JSC on its runtime thread and exchange only bounded wire bytes.
- [ADR-0003: Use a three-layer cross-language golden vector contract](adr/0003-cross-language-golden-vector-contract.md) — lock producer bytes, cross-language meaning, and permitted numeric forms.
- [ADR-0004: Keep resource and text-input length limits in their native units](adr/0004-dual-length-semantics.md) — measure resources in UTF-8 bytes and TextInput limits in UTF-16 code units.
- [ADR-0005: Reuse native press semantics for keyboard activation](adr/0005-keyboard-activation-reuses-press-semantics.md) — keep keyboard activation aligned with native Pressable behavior and traversal.
- [ADR-0006: Append an optional typed value to CommandResult](adr/0006-optional-command-result-value.md) — extend acknowledgements with one validated optional result tail.
- [ADR-0007: Keep Image sources as explicit host-resolved paths](adr/0007-explicit-image-path-source.md) — keep image sources bounded paths and degrade locally through an optional fallback.
- [ADR-0008: Apply fail-fast and bounded degradation at different seams](adr/0008-error-handling-philosophy.md) — fail fast for contract failures while containing resource and backpressure failures.
- [ADR-0009: Async close confirmation across the process boundary](adr/0009-async-close-confirmation.md) — make per-surface close confirmation asynchronous without blocking native callbacks.
- [ADR-0010: Opt-in high-frequency event streams](adr/0010-opt-in-high-frequency-event-streams.md) — register and transport pointer movement only for nodes that request it.
- [ADR-0011: Single-form wire with optional node tails](adr/0011-single-form-wire-optional-tails.md) — define one current tuple shape and reject historical compatibility forms.
- [ADR-0012: Host-owned input models over upstream gaps](adr/0012-host-owned-input-models.md) — keep bounded TextInput editing, selection, geometry, and history in the host.
- [ADR-0013: Model interactive text as one shaped paragraph with clickable runs](adr/0013-interactive-text-runs.md) — shape rich text as one paragraph while retaining nested run styles and interaction.

## Contributing

- [CONTRIBUTING.md](../CONTRIBUTING.md) — contributor order, toolchain, verification ladder, conventions, and architecture wayfinding.
- [Domain conventions](agents/domain.md) — vocabulary and domain-modeling guidance.
- [Issue tracker](agents/issue-tracker.md) — local issue structure, status, ownership, and resolution workflow.
- [Triage labels](agents/triage-labels.md) — canonical labels and their meanings.

## Evidence & verification

- [Tracked evidence index](../.scratch/EVIDENCE.md) — pointers to measurements, release records, decisions, and freshness classifications.
