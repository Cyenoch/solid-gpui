# Posting checklist

> - [ ] Post this issue in `zed-industries/zed` (the upstream repository).
> - [ ] Before posting, verify the dependency edges, package licenses, and source line numbers at the current `main`; update the revisions/citations if they have moved.
> - [ ] Confirm that the exact GPUI package and target graph relevant to the report still resolves `ztracing` (including build-dependencies and `sum_tree`).
> - [ ] Keep the report as a question about intended licensing/relicensing direction; do not characterize it as a legal conclusion.
> - [ ] Do not include links or identifying details for the private, unpublished downstream repository.

# Issue title

Apache-licensed `gpui` depends on GPL-3.0-or-later `ztracing`

# Issue body

## Summary

Could you clarify whether the `gpui` dependency on `ztracing` is intentional as part of the relicensing effort?

At the Zed revision we currently consume (`6805d952f9f3d702f760aa11b1547df8a625fa16`), `gpui` declares `license = "Apache-2.0"` in `crates/gpui/Cargo.toml:1-10`, while its ordinary dependency table includes `ztracing.workspace = true` at `crates/gpui/Cargo.toml:107`. The `ztracing` manifest declares `license = "GPL-3.0-or-later"` at `crates/ztracing/Cargo.toml:1-6`. It also has ordinary dependencies on `zlog` and `ztracing_macro` (`crates/ztracing/Cargo.toml:15-21`), whose manifests likewise declare `GPL-3.0-or-later`.

The same GPUI dependency and tracing attributes were present in the `origin/main` snapshot checked at `1662f5f3f6497c5f80830ccdca1edfd1fc0c6c6a` (2026-08-29). The direct GPUI edge and SVG instrumentation were introduced in commit `00cba838ad4e0be4b6176438551b72b2d512e9f8` (2026-08-05), according to our local history review.

## Where the edge is used

The direct GPUI edge is used by instrumentation on SVG operations:

```rust
// crates/gpui/src/svg_renderer.rs:190
#[ztracing::instrument(skip_all)]
pub fn parse_svg(...)

// crates/gpui/src/svg_renderer.rs:196
#[ztracing::instrument(skip_all)]
pub fn render_parsed(...)
```

There is also a second ordinary path through `sum_tree`, which is a GPUI dependency (`crates/gpui/Cargo.toml:92`) and independently declares `ztracing.workspace = true` in `crates/sum_tree/Cargo.toml:16-21`. The source imports and uses the attribute in:

- `crates/sum_tree/src/cursor.rs:4` (`use ztracing::instrument`), with attributes on `prev` (`:214-216`), `seek` (`:405-408`), `seek_forward` (`:416-423`), and `seek_internal` (`:462-465`).
- `crates/sum_tree/src/sum_tree.rs:13` (`use ztracing::instrument`), with attributes on `find_exact` (`:396-400`), `find` (`:424-426`), and `find_with_prev` (`:509-512`).

In a downstream target-qualified graph, these ordinary edges resolve the GPL-3.0-or-later `ztracing` package and its `zlog`/`ztracing_macro` dependencies into binaries that link the affected GPUI graph. Consequently, downstream consumers starting from Apache-licensed `gpui` have to treat the resulting binary as GPL-linked for their distribution review, rather than being able to evaluate `gpui`'s Apache metadata in isolation.

## Why we are asking

We are not presuming that the dependency is impermissible, nor are we making a legal determination. We would appreciate clarification on one of these points:

1. Is the GPL-3.0-or-later tracing dependency intended for GPUI and its downstream consumers?
2. If not, is removing or replacing this dependency planned as part of the relicensing effort?
3. If it is intended, could the GPUI documentation or package metadata make the resulting downstream dependency/licensing relationship easier to discover?

The question matters to distributors because an Apache-licensed top-level package can still bring differently licensed code into a linked binary through ordinary dependencies. Our downstream publication is blocked while the exact dependency terms and required distribution treatment are reviewed.

## Neutral workaround used for investigation

For a narrow, target-specific engineering experiment, we replaced the `ztracing` package through a local Cargo source patch with an independently authored no-op proc-macro crate. The stub exported only the consumer-facing `ztracing::instrument` attribute, returned each annotated item unchanged, and had no `zlog`, `ztracing_macro`, or other dependency. This was a local workaround for an unpublished/private downstream project, not a proposed upstream change or a legal conclusion; it does not address the broader Zed workspace's runtime tracing APIs or other licensing findings.

## References

- Pinned GPUI manifest: `crates/gpui/Cargo.toml:1-10,46-107`
- Pinned `ztracing` manifest: `crates/ztracing/Cargo.toml:1-21`
- Pinned `sum_tree` manifest: `crates/sum_tree/Cargo.toml:1-21`
- Pinned SVG use: `crates/gpui/src/svg_renderer.rs:189-197`
- Pinned `sum_tree` uses: `crates/sum_tree/src/cursor.rs:1-5,214-216,405-428,462-470`; `crates/sum_tree/src/sum_tree.rs:11-13,396-426,509-512`
- Downstream audit evidence was recorded against the pinned revision and the checked `origin/main` snapshot; the downstream repository is private and unpublished.

Thank you for clarifying the intended direction and for maintaining GPUI.
