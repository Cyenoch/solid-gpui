# Reference application

Reference Studio is a shared native application workload in the website Showcase
and the desktop application example. Open `/showcase/reference-studio` or
`/showcase/reference-studio-history` in Gallery Desktop. The desktop example's
Studio link opens `/studio/timeline`, with `/studio/history` as its second view.
Both use `examples/website/src/showcase/ReferenceStudio.tsx` and its adjacent
`reference/state.ts` and `reference/model.ts`; the Showcase's copied source reads
those same modules through Vite's raw imports.

## Source identity and adaptations

The fixture manifest `src/showcase/reference/provenance.json` records immutable
source commits, Git blob and SHA-256 hashes, source mappings, and adaptations.
The app code and data are original. Source identity checks and native interaction
qualification are separate checks.

Run from the repository root:

```sh
bun examples/website/scripts/check-reference-provenance.ts
bun --conditions=browser test examples/website/tests
```

For offline verification, pass the GPUIX and gpuix-solid checkout paths in that
order. The checker reads the pinned Git objects rather than mutable checkout
files. HTTP verification fetches only immutable commit URLs, with a bounded
timeout, and rejects either hash mismatch.

Clip editing uses controlled title/start/duration fields, native drag/drop,
explicit time-window and zoom controls, and GPUI-owned vertical scroll. Every
track header and clip grid shares one virtualized row. The history is a
deterministic in-memory review dataset with variable paragraph counts.
Overlapping clips occupy separate lanes inside a measured track row, so moving
a clip onto another clip's time range preserves individual pointer targets.
Lane packing includes the 24-pixel minimum hit target and recomputes when
viewport width or zoom changes.
Recorded paint and host-owned media compose through the `preview` slot using
generated NativeView contracts; that component owns cancellation and release.

## Interaction and ownership

The app starts with 240 tracks, 480 clips, and 10,000 review entries. Native
`VirtualList` requests bound Solid row owners to the viewport with overscan 2;
initial mounts are 8 tracks and 5 history entries. Data storage and search are
O(dataset size); rendering and row-owner memory depend on the visible range.
Search is memoized by the controlled query. Pointer hover does no app work.
Native drag/drop performs one immutable edit on completion.

Drag a clip onto another track to move it while keeping its time and ID. Drag a
track header onto another track to insert it before that track. The inspector
edits a selected clip's Unicode title and finite timing values. Placements clamp
to the 120-second project and durations remain at least one second. Post review
notes from the multiline composer; posting inserts a unique note, clears the
search, and asks the native list to show its first row. Select review paragraphs
and use the platform copy shortcut, or use Copy review to copy the selected
review's complete text through the root's clipboard command.

Timeline and History change the proportions of retained panes. Narrow windows
give the active pane the available width and keep the inactive pane mounted at
zero width. Inspector opens the retained detail pane. Returning to a wide window
keeps input models, scroll handles, and business selection. Navigating away
disposes row owners and releases handle references. No timers, per-frame JS
callbacks, or global subscriptions are installed by this app.

## Acceptance

`tests/reference-studio.fixture.tsx` covers controlled edits, bounded row owners,
filtering at the last row, retained Host Node identity through view/size changes,
post/copy behavior, and cleanup through the public semantic TestHost. It does
not establish native geometry, content displacement, drag hit testing, selection
painting, clipboard integration, or physical presentation.

`tests/reference-studio.native.ts` is the executable native interaction workload.
Its driver must wrap the opt-in native acceptance API and return actual painted
bounds/identity and native state. It rejects unsupported required capabilities.
Run serially after builds with a five-minute deadline; never substitute semantic
visible-range events for native scrolling. The adjacent acceptance manifest
specifies locators and invariants for harness integration.

Qualification requires nonempty painted rows, bounded owners after real wheel
displacement, last-row reachability, Unicode edit/save, native clip move and
track reorder followed by another click, pane/input identity and offset
retention across routes and wide/narrow/wide resizes, selection/copy across
separate paragraphs, and cleanup after surface closure. Capture the same native
window where supported. Record unsupported capture or platform input explicitly.
Native test rendering and OS-injected physical input are separate evidence
categories. No display FPS or latency claim follows from semantic tests or builds.

The workload also reads the generated selection service and compares its text
with the native clipboard, searches committed text and selects a matching
revision, then clears highlights. Preview checks require painted recording/frame
bounds and acknowledged retained CPU pixels after advancing the native frame.
Before closing, it starts a partial native upload, cancels staging while retaining
the displayed frame, and clears the frame. Native `getState` acknowledgements
check retained bytes at each transition. Cleanup reports actual row-owner,
Surface, window, and popup counts; per-resource teardown is covered by the native
owner lifecycle tests. The preview aborts any in-flight call when it unmounts.

`mountReferenceStudio()` exposes `ready`, resolved after the first router load.
Await it before window/root commands. Native qualification uses the acceptance
resize event so window geometry and the application size store change together.

Run the driver with Bun's `browser` condition so external Solid imports use the
client reactive runtime:

```sh
bun --conditions=browser examples/website/scripts/qualify-reference.ts /absolute/path/to/website-acceptance deterministic .scratch/reference-acceptance
```
