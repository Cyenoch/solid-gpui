# Gallery package verification

Date: 2026-09-07. Platform: macOS, `aarch64-apple-darwin`.

## Candidate artifact

- Command: `bun run task gallery-package`
- Archive: [solid-gpui-gallery-0.2.0-aarch64-apple-darwin.zip](../../dist/gallery/solid-gpui-gallery-0.2.0-aarch64-apple-darwin.zip)
- Size: 12,582,298 bytes.
- SHA-256: `81a061bea9b8e76763018c49da8964342c5777a9fb6f39debbdaf1ff7b1c2f3a`
- Final release compilation: 2 minutes 39 seconds; the task exited successfully.

The task regenerated both SDK and Gallery native bindings, rebuilt the TypeScript
packages, bundled the Gallery with source maps omitted, and embedded it in the
release executable. The final candidate includes the first-Snapshot refresh and
ordered startup command fixes discovered during native application verification.

## Passed checks

- Locked release build for the build machine's explicit Rust host target.
- `otool -L`: no non-system dynamic libraries; dependency inventory packaged.
- `plutil -lint`: application metadata valid.
- Ad hoc `codesign` and `codesign --verify --strict`: passed.
- Archive extraction into an operating-system temporary directory outside the
  checkout, with SHA-256 verification of all packaged files.
- Extracted executable `--version`: `solid-gpui-gallery 0.2.0`.
- Extracted executable `--check-bundle` with empty `PATH`: passed. This consumes
  ordered Snapshots and Patches under one 15-second deadline, validates their
  tree and native contracts, and requires content from both native providers.
  It does not accept the router's small initial loading tree as success.

Self-check output:

```text
Gallery bundle tree and native contracts verified; runtime shut down cleanly
```

The source-map/bundling tests passed: 2 tests, 15 assertions. Tooling, Gallery,
and Vite Gallery TypeScript checks passed during implementation. The existing
QuickJS counter roundtrip test now starts from owned source after deleting its
original file; the root agent runs that test in the final Rust gate.

## Native startup observations

Final-candidate startup trials are recorded as
`package-final-trial{1,2,3}.ax.txt` and matching screenshots in this directory
(PIDs 31884, 32719, and 33586). All three untouched initial accessibility trees
showed the full Overview/workspace controls, the title `Solid GPUI — Component
Workbench`, and no performance monitor. The first and third captures retained
the expected 800×633 geometry. Later external desktop interaction changed the
second trial's screenshot and is not used as startup evidence. All three
processes were stopped after observation.

The final build was not used for an interaction smoke after those launches
because it exited before a fresh accessibility query; earlier candidate-build
interaction evidence is retained separately and is not attributed to this
archive. Run records contain only purpose-specific environment overrides, not a
dump of the process environment.

## Qualification limits

Windows/Linux native packaging jobs are implemented but were not executed on
those platforms in this macOS session. Their desktop/GPU/runtime prerequisites
still need platform qualification. This macOS package has an ad hoc signature;
publisher signing and notarization remain release steps. The included
third-party notices are an inventory, not a complete license-text collection.
