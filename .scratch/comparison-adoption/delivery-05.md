# Ticket 05: native regions and immutable style ownership

Branch: `adopt/05-native-performance`. Baseline ancestry: `e879b662` confirmed.

## Design

Automatic native region owners retain static View/Text/raw-text trees with pixel
width/height, no intrinsic min/max/growth, zero shrink and hidden overflow.
No public cache hint or wire change is added. Capability summaries reconcile
after atomic validation; accepted content edits notify the owning Entity.
Removal, snapshots, epoch reset and live capability additions release owners.
Geometry/text/clipping dependencies use GPUI's cache key; opacity/interactive/
animated ancestors, active accessibility and popup anchors use live paths.
Native controls, listeners, extensions, media, images, scrolling and selectable
text remain live. In-flight animation dependencies are checked each frame.

Work bounds: mount summary O(nodes); patches visit changed summaries/ancestors
and affected child lists, stopping equal summaries. Candidate/owner checks are
O(region count × ancestor depth), independent of static descendant count.
Unchanged region construction is O(1), while GPUI still replays its scene.
Misses and intrinsic content perform required native construction/layout.
Owner holds a weak root; root holds owners and releases them on removal/reset.

StoredNode uses `Option<Arc<Style>>`. Journals/animation source styles clone Arc;
equal writes retain the allocation; sampled frame styles are independent.
No interner, stale wrapper or style-reference wire protocol ships.

## Tests and evidence

- Counter update next to 10/1,000 rows draws `Count: 12` and bounds primitive work.
  The regression failed before implementation (26 constructions for 10 rows).
- Retained text edit reaches paint; inherited geometry changes and repeated
  560/1280/560/800 widths preserve definite region dimensions; deletion releases.
- Native typing preserves visible edits and avoids static sibling reconstruction
  after the intentional first keyboard-modality full refresh.
- Hit-tested button after unrelated revisions emits revision 5/listener 66.
- Adding native input releases region; deleting input restores it; animated
  ancestor remains live. Intrinsic text widening reflows its neighbor.
- Native wheel displaces region content. Async image reaches native atlas beside
  retained static content. Style test covers equal writes, journal ownership,
  rejection/rollback and release after collapse.

Focused checks before integration: six region tests, async image test, style
ownership test passed; memory region fixture passed 256 verified updates at
500/2,500/10,000 rows. Website content/Markdown: 3 passed, 1,995 assertions.
Native GPU/presentation timing and physical input acceptance are intentionally
serialized by integrator after parallel compilation; no timing claim here.

## Allocation experiment

Real NodeStore/transaction probe: `crates/solid-gpui/examples/style-storage-profile.rs`.
Inputs prepared before counting; final revision/text/style checked; mount,
text journals, equal styles, unique drag styles and collapse counted.
Pre-v7 experiment: Style=496 bytes; StoredNode 784→296 bytes. Allocation results:

| Phase | Inline baseline | Immutable shared candidate |
| --- | ---: | ---: |
| compact mount allocated bytes | 29,976,344 | 19,105,560 |
| 1,000 text journals allocated bytes | 3,592,232 | 1,640,232 |
| 1,000 equal styles allocated bytes | 3,292,000 | 1,340,000 |
| 1,000 compact drag allocated bytes | 3,292,000 | 1,852,000 |
| compact mount allocations | 100,047 | 110,047 |
| heap text journal allocations | 18,000 | 16,000 |
| heap equal style allocations | 11,000 | 7,000 |
| heap drag allocations | 11,000 | 8,000 |

Compact unique drag adds one allocation per changed style. Heap-bearing styles
remove font/shadow copies. No global retained table; final owner releases data.
Raw files: `style-storage-{baseline,shared,interned}-05.jsonl`.
A 64-bucket/four-way weak collision-confirming interner additionally reduced
mount bytes to 13,986,104, but compact/heap drag elapsed increased from shared
3,784/3,908µs to 4,606/4,840µs in this diagnostic sample. It was removed entirely
from production. Concurrent machine load prevents a controlled CPU claim;
allocation counts are the evidence for retained immutable sharing.

## Measurement commands

Use `CARGO_BUILD_JOBS=2` and the agreed shared CARGO_TARGET_DIR for compilation.
Build/copy baseline (`e879b662`) and candidate profile binaries separately,
finish compilation, exclude test-support in feature graph, then run serially:

```sh
cargo build --locked -p solid-gpui --bin solid-gpui-profile --features frame-profile
cargo tree --locked -p solid-gpui --features frame-profile -e normal,build,features
bun scripts/native-region-compare.ts /absolute/baseline-profile /absolute/candidate-profile .scratch/native-region-comparison-05
cargo build --locked -p solid-gpui --example style-storage-profile
/absolute/target/debug/examples/style-storage-profile
```

Comparison script never compiles, refuses an existing directory, records hashes,
disables HUD, runs A/B/B/A with 60s limit per run. Copy the current fixture to the
baseline checkout for identical `--regions` content. Report construction, CPU
draw and input-to-present separately. `--hold` native fixture must check visible
counter, native typing/press, final row, actual scroll, narrow/wide geometry.
Record viewport 800×600, scale/display/refresh rate and background load.
Use acceptance seam for painted content; wire assertions alone cannot qualify.

## Documentation and integration

Synchronized performance guide and explicit Chinese copy, native composition
and Chinese copy, docs index, website README and Signals chapter/translation.
Website consumes authoritative guides directly. No generated contract output.
Maintained content describes current functionality. Retain legal/provenance metadata.

Merge notes: reconcile acceptance observations across region replay; never clear
the shared observations before cached content skips paint. Media/extensions stay
live by capability classification. Ticket04 pseudo-style refinements must make
stateful styled nodes/ancestors live; axis-scroll fields must participate in
capability summaries. Ticket06 renderer-wide selection must disable scene reuse
for its live selection/search geometry. Focused combined checks follow merge.

Merged integration `5205bb4f` into this branch, resolving the one paint seam
conflict by retaining both test paint probes and production acceptance observe.
Added scene-generation-owned observation replay; the existing per-frame clear
remains and each cached region replays its captured painted nodes. Cache misses
replace captures from actual paint; deletion releases captures. The combined
`native-acceptance` regression passes painted text/bounds reuse, fresh edits,
bounded primitive construction and removal.

## Final integration measurement

`qualification/performance/` retains native correctness screenshots/reports,
feature lists, executable hashes, workload hashes, the single control override,
raw A/B/B/A logs, current allocations and construction summaries. Production
binaries use release/thin-LTO, GPUI 0.3.7, `frame-profile`, no test support,
matching temporary macOS app contexts, HUD off and scale 2. All compilation
ended before measurement. The shared 500-row workload passes actual Metal
paint, Count 360->361, hit-tested click, Unicode editing, native wheel
displacement, row 500 reachability and repeated resize.

Ordinary/occluded submission runs had insufficient sustained draws and were
rejected. Continuous native resize supplies a valid construction counterexample,
not display cadence. Root and region counters/totals are now separate; cleanup
removes owned temporary bundles on success or failure. Before the final policy,
candidate aggregate construction cost exceeded the live control. Viewport-change
frames now bypass missed region wrappers and stable frames resume reuse; eight
focused native region regressions pass.

Final A/B/B/A root construction means: control 0.562/0.417 ms, candidate
0.568/0.582 ms; candidate additionally reconstructed 13/8 regions after warmup.
Construction totals: control 180.933/137.178 ms, candidate 194.700/190.468 ms.
The runs show no resize speed improvement and significant control drift. Do
not report a pooled p95, CPU draw, display FPS or physical latency from these
construction samples. Retained stable-viewport work bounds remain established
by deterministic construction/paint/input regressions, not this counterexample.

Final v7 allocation probe: Style 600 bytes, StoredNode 304; compact mount
20,407,704 allocated bytes; 1,000 text journals 1,672,232; equal styles
1,372,000; unique drag styles 1,988,000. Collapse allocated zero and released
19,003,336 bytes. This current ownership attribution is not a new inline baseline
or a physical presentation metric. No global interner ships.

Computer-use could not bind the production window; its failed hold measurement
is not accepted timing. Shared-workload GPU tests establish content correctness,
but physical OS input, IME, idle presentation, Windows/Linux, and display
performance remain unqualified. Final source suites/Clippy passed after policy
changes; generation and local paired delivery are requalified by integration.
