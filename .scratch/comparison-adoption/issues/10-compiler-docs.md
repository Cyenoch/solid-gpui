# Public canonical compiler and accurate maintenance provenance

Status: ready-for-agent
Blocked by: none

Implement spec outcomes 3 and 12 plus clear documentation drift. Export a reusable compiler-only interface from @solid-gpui/vite (or smallest coherent existing package seam), using the one current universal transform, source-map composition and stable runtime ABI. Vite must consume the same implementation; no duplicate compiler/runtimes or compatibility wrapper. Include meaningful source-map/reactivity/ref/packed-export checks. Ticket 02 owns CLI/project, so keep compiler/package exports/build entry edits narrow and record conflicts. Fix README website status, inaccurate vendor GPUI provenance with actual source evidence, and macOS direct-library vs experimental Windows packager wording. Maintain patch inventory and prepare worthwhile general fixes as local reviewable patches if useful; do not create upstream comments/PRs. Synchronize related guides/translations/website snippets, keeping capability state authoritative. Other tickets own their topical guide edits and integrator reconciles.
