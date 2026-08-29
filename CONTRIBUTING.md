# Contributing to React GPUI

This is the contributor map: it gives a new contributor an order of operations
and points at the authorities that own the details. When this file and a deeper
source disagree, follow the deeper source and update this map rather than
copying the contract here.

## Read this first

1. Read [`AGENTS.md`](AGENTS.md) for the repository's non-negotiable principles.
2. Read [`CONTEXT.md`](CONTEXT.md) for the glossary and the names of the
   protocol, surface, runtime, and native-host seams.
3. Start with the [documentation index](docs/README.md) to choose the relevant
   architecture decision records and [agent conventions](docs/agents/) for your
   area.
4. Use [`docs/getting-started.md`](docs/getting-started.md) for the consumer
   path, [`docs/protocol.md`](docs/protocol.md) for wire details, and
   [`docs/troubleshooting.md`](docs/troubleshooting.md) for symptom-first
   diagnosis.

Keep the [root README](README.md) as the workspace map and public overview;
this guide adds contributor order, not a second product reference.

## Toolchain and setup

The pinned versions are authoritative in
[`docs/getting-started.md`](docs/getting-started.md), `.bun-version`, and
`rust-toolchain.toml`:

- Bun `1.4.0`
- Rust `1.97.1`

Verify them before package or Rust work:

```sh
bun --version
rustc --version
```

Each package has a checked-in lockfile. Install with frozen locks from the
repository root's package directories before running package commands:

```sh
(cd packages/react-gpui && bun install --frozen-lockfile)
(cd packages/react-gpui-dev && bun install --frozen-lockfile)
```

The package and consumer setup details belong to
[`docs/getting-started.md`](docs/getting-started.md), not this map.

## Verification ladder

Run the narrowest useful check while iterating, then climb the ladder before
handing off. Timings below are recent macOS ARM evidence, not guarantees:
cache state and concurrent work change wall time. The full measurements and
counts are in [`gate timing evidence`](.scratch/gate-perf/timing.md) and the
[`release-readiness evidence`](.scratch/release-productionization/release-readiness.md).

### 1. Focused checks, then the ordinary gate

`make ci` is the ordinary contributor gate and the local entry point used by
the CI workflows. A recent fresh run took about **79 s**; uncontended warm
runs were about **51–54 s**, while a cold or contended run can take roughly
2–3 minutes. It proves all of the following in one ordered recipe:

- Rust formatting;
- locked workspace check, strict Clippy (`-D warnings`), and workspace tests;
- both Bun packages' formatting, typechecking, tests, and builds; and
- package archive contents plus an external consumer install/runtime/type smoke.

The package smoke is intentionally part of `bun-ci`; it is not enough to test
source imports when the published artifact is the consumer contract.

**Focused green is not CI green.** A focused Rust or Bun test can pass while
the separate Clippy invocation rejects a warning, or while the Rust module
boundary test rejects an accidental dependency/export. In particular,
`crates/react-gpui/tests/module_boundaries.rs` keeps renderer, paint, protocol,
and tree child modules private and allowlists intentional `super::` edges and
public items. A change that passes a focused behavior test still needs
`make ci` to prove those architectural locks and all-target Clippy.

### 2. Embedded runtime gate when its inputs matter

Run `make embedded-bun` when touching the embedded adapter, Bun/JSC build
inputs, host runtime selection, or a path covered by the embedded workflow;
run it before calling an embedded change release-ready. It is intentionally
outside `make ci` because it builds the pinned Bun/JSC source graph. The gate
checks the embedded host feature, runs the isolated startup matrix (gallery,
text input, VirtualList, and notes), checks Fast Refresh lifecycle behavior,
and tests the embedded adapter. Recent warm evidence is approximately **4–9
s** (3.97 s in the latest readiness run; 8.68 s after the matrix was added).
It proves transport/startup/lifecycle behavior, not display-backed painting,
IME placement, picker UI, or asynchronous file-command completion.

### 3. Candidate and release smokes

Use these after `make ci` for a macOS ARM candidate, not for every edit:

| Target | When | What it proves | Recent time |
| --- | --- | --- | ---: |
| `make host-release-bundle` | Need a staged process-host archive | Builds the release host and writes the deterministic archive shape used by the check. | Not separately timed in the cited evidence. |
| `make host-release-check` | Before distributing a process-host candidate; also runs in ordinary CI as an independent job | Builds, archives twice, requires equal archive SHA-256 values, checks the allowlist/checksums/executable, and validates `--help`/`--version` without a display. | Included in candidate evidence; no isolated timing recorded. |
| `make host-candidate-smoke` | Process-runtime candidate handoff | Extracts the archive, runs the counter through the extracted host, observes a Snapshot, checks expected timeout `124`, startup diagnostics, help, and version. | **35.05 s** |
| `make host-embedded-candidate-smoke` | Embedded candidate handoff, after the embedded gate | Builds the embedded release host, runs the counter from a fresh directory, observes an embedded Snapshot/press commit, checks expected timeout `124`, startup diagnostics, help, and version. | **19.85 s** |

Candidate smokes do not claim display-backed GUI pixels or native picker,
IME, cursor, menu, or clipboard behavior. The scripts' timeout `124` is an
expected successful rehearsal outcome, not a failed gate. The candidate
workflow definitions are [`host-release-candidate.yml`](.github/workflows/host-release-candidate.yml)
and [`host-embedded-candidate.yml`](.github/workflows/host-embedded-candidate.yml).

### Make target map

These are all targets in the root [`Makefile`](Makefile). Use the leaf targets
for a narrow loop; use `ci`, the embedded gate, and candidate smokes according
to the ladder above.

| Target | Does / when to use |
| --- | --- |
| `ci` | Runs `rust-format`, `rust-check`, and `bun-ci`; ordinary handoff gate. |
| `rust-format` | `cargo fmt --all -- --check`; run for Rust-only edits or formatting diagnosis. |
| `rust-check` | Locked workspace `cargo check`, all-target strict Clippy, and workspace tests; run for Rust iteration, then still run `ci`. |
| `bun-install` | Frozen installs for both package lockfiles; use after checkout or lockfile changes. |
| `bun-format` | Prettier checks for both packages; use for TypeScript formatting diagnosis. |
| `bun-typecheck` | Builds both packages, then runs both TypeScript typechecks; build-first ordering is intentional. |
| `bun-test` | Runs both package test suites; use for focused package behavior iteration. |
| `bun-build` | Builds both package distributions; use when exports or generated `dist/` artifacts matter. |
| `bun-ci` | Runs Bun formatting, typecheck, tests, and package smoke; the Bun half of `ci`. |
| `bun-pack-smoke` | Packs both packages, checks allowlisted contents/licenses, installs a temporary consumer, and runs runtime/type smoke; use for packaging/export changes. |
| `protocol-golden-generate` | Regenerates bidirectional protocol fixtures with the Rust example and Bun script; use only for intentional wire-contract changes, then review bytes and semantics. |
| `api-surface-generate` | Builds packages and regenerates checked-in public name/kind snapshots; use only after an intentional export change and review the diff. |
| `embedded-bun` | Locked embedded host check, isolated representative examples/Fast Refresh, and embedded adapter tests; use for embedded-runtime inputs and release readiness. |
| `host-release-bundle` | Creates the process-host archive under `dist/`; use to inspect a staged candidate. |
| `host-release-check` | Checks deterministic archive output, contents, checksums, executable permissions, help, and version; use before distribution. |
| `host-candidate-smoke` | Rehearses an extracted process candidate against a user renderer entry; use for candidate handoff. |
| `host-embedded-candidate-smoke` | Rehearses an extracted-style embedded release binary; use for embedded candidate handoff. |
| `soak-smoke` | Runs the release host against the stress example (default 60 s), records protocol taps and RSS samples, and checks ordering/paired frame counts/leak thresholds; use when changing transport, lifecycle, or long-run behavior. |
| `release-prep` | `make release-prep VERSION=x.y.z` synchronizes Cargo/package versions and locks after requiring a matching changelog section; use only for release preparation, never as publication. |

Do not regenerate fixtures to hide an unexpected diff. The authoritative wire
and export contracts are [`docs/protocol.md`](docs/protocol.md) and the
checked-in API fixtures described in [`README.md`](README.md).

## Conventions that protect the contract

### Commits and changelog

Use Conventional Commit subjects, in the lowercase style already established
by recent history (`docs:`, `fix(dx):`, `test(protocol):`, `perf(renderer):`,
and similar): `type(scope): imperative summary` when a scope helps. Keep one
logical change per commit and make the subject say what changed.

For user-visible work, put bullets **inside** `## [Unreleased]`, under the
matching `### Added`, `### Fixed`, or `### Changed` subsection in
[`CHANGELOG.md`](CHANGELOG.md). The four-times-corrected section-placement
lesson is simple: an entry under the wrong heading—or a top-level bullet
outside `[Unreleased]`—is misfiled. For example, a new capability belongs
under `## [Unreleased]` → `### Added`, not beside those headings. Release
versioning and lock synchronization are owned by `release-prep`; it does not
create changelog sections.

### Tests, fallback, and legacy forms

[`AGENTS.md`](AGENTS.md) is the authority here:

- “Only keep the key test cases. Meaningless testing is prohibited.” Add a
  test for an observable contract, boundary, invariant, or transition—not a
  test that merely repeats implementation shape.
- “Avoid excessive fallback.” Preserve explicit failure boundaries rather than
  hiding an invalid state with a speculative fallback.
- During development, “don't keep any historical wrappers.” Do not retain a
  legacy dual-form decoder, compatibility alias, or old data structure merely
  to avoid a clean cutover. Protocol evolution rules and accepted exceptions
  are recorded in [`docs/protocol.md`](docs/protocol.md) and the relevant ADR.

## Local process

### Issues and triage

The issue tracker is local Markdown, not a remote service. Follow
[`docs/agents/issue-tracker.md`](docs/agents/issue-tracker.md): one feature per
`.scratch/<feature-slug>/`, a `spec.md`, one numbered file per implementation
issue under `issues/`, a `Status:` line, and appended comments. Wayfinder maps,
blocking, claiming, and resolving rules are described there too.

Use only the canonical labels from
[`docs/agents/triage-labels.md`](docs/agents/triage-labels.md):
`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, and
`wontfix`. The label meanings and the mapping from triage role to tracker
string live in that table.

### ADRs

Offer an ADR only when all three conditions hold: the decision is hard to
reverse, surprising without context, and the result of a real trade-off. The
numbering and minimal format are specified by the
[ADR format](.agents/skills/domain-modeling/ADR-FORMAT.md). Read ADRs that
touch the seam you are changing; do not reopen an accepted decision silently.
[`ADR-0013`](docs/adr/0013-interactive-text-runs.md) is the current exemplar:
it records context, the decision, rejected alternatives, pinned constraints,
and consequences for one non-obvious native/protocol boundary. A small,
obvious, reversible implementation choice does not need an ADR.

### Evidence

Scratch evidence is intentionally separate from product docs. The repository
ignores `.scratch/` by default (see [`.gitignore`](.gitignore)); a measurement
or release record that must be shared is added explicitly with `git add -f`.
Include the command, environment/date, sample size or workload, numeric
measurements, and a bounded verdict—do not report a vague “looks good.” Add a
row to [`EVIDENCE.md`](.scratch/EVIDENCE.md) so six-month-later readers can
find the source, key number, and freshness (`historical`, `living`, or
`in-flight`). Existing timing and release records show the expected level of
numeric detail.

## Architecture wayfinding

- [`CONTEXT.md`](CONTEXT.md) is the vocabulary authority. Use its terms rather
  than inventing synonyms; it also links each term to code, protocol sections,
  scratch evidence, or ADRs.
- [`docs/protocol.md`](docs/protocol.md) owns field-level wire details and
  evolution rules; [`docs/getting-started.md`](docs/getting-started.md) owns
  the consumer setup/composition path; [`docs/troubleshooting.md`](docs/troubleshooting.md)
  owns symptom → diagnosis → repair.
- [`docs/adr/`](docs/adr/) records hard-to-reverse protocol, runtime, and
  native-boundary decisions.
- [`EVIDENCE.md`](.scratch/EVIDENCE.md) is the evidence index; the individual
  scratch files are the measurements and decision records.
- For pinned GPUI capabilities and true upstream gaps, start with the
  [`upstream dependency assessment`](.scratch/release-productionization/upstream-dependencies.md)
  rather than assuming a missing project API is an upstream limitation.
- Use [`docs/README.md`](docs/README.md) as the reading-order index for product,
  reference, architecture, contribution, and evidence documentation.

The root [README workspace map](README.md#workspace-map) points here for this
workflow; keep both pointers short and repair links when files move.
