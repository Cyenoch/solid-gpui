# Ticket 09: reference application and native workload

Integrated branch: `integrate/comparison-adoption`; owner merge `3a02bd19`.

## Application and ownership

Website Showcase and the desktop example share Reference Studio: 240 tracks,
480 clips, and 10,000 review entries. Solid owns immutable business edits,
controlled input, routing and bounded row owners. GPUI owns layout, measured
virtualization, drag/drop, editing, scroll state, cross-element text selection,
clipboard and native preview resources. Pinned fixture Git/SHA-256 provenance
remains separate from original application source.

Clip lanes use actual rendered intervals, including the 24-pixel minimum target,
and recompute with width/zoom. Track rows grow to preserve hit targets after
overlapping drops. The router exposes first-route readiness. The native driver
requires Bun's client Solid runtime and actual viewport feedback.

## Native qualification

Deterministic and macOS Metal workloads passed against the opt-in production
acceptance seam. They exercise Unicode edit/save and subsequent click; clip
parent ownership and track reorder; native wheel displacement; last-row access
and filtering; wide/narrow/wide responsive geometry and retained pane/input IDs;
scroll retention; cross-paragraph selection/copy with generated-service equality;
search/select/clear; note post/copy; and zero row-owner/Surface/window/popup
counts after closure. Peak row ownership was 20, below the manifest limit of 40.

Preview checks observe native frame sequence advancement and retained bytes.
Partial staging is cancelled before unmount: 9,216 -> 18,432 -> 9,216 bytes,
then explicit clear returns 0. The component aborts outstanding calls on unmount.
Per-resource drop is covered by native owner lifecycle tests rather than
manufactured counts derived from the Surface count.

Reports and a visually inspected Metal screenshot live under
`qualification/reference/`. The screenshot shows actual timeline/history,
recorded graphics and CPU frame pixels. Final reruns after review tightening
passed in 1,311 ms (deterministic) and 4,414 ms (Metal), with peak 20 row owners.
The final WASM/browser Showcase and authoritative guide also rendered without
console errors; `qualification/reference/reference-website.png` records that
browser check.

## Synchronization and limits

Authoritative reference-application guide and Chinese copy, website README,
shared source/provenance and acceptance manifest are synchronized. Website
catalog/navigation/runtime suite: eight passed. Business lane/reorder test and
command lifecycle regressions passed.

Evidence is native test rendering, not OS-injected physical input. No physical
IME/trackpad, display FPS/latency, Windows/Linux, or release qualification is
claimed. Deterministic capture is explicitly unsupported.
