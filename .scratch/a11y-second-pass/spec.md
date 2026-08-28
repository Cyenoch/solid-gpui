# Accessibility second pass

This review closes the accessibility seams available in pinned GPUI 0.2.2 and records the remaining upstream gap. The protocol remains lockstep v3.

## Audit decisions

| Item | Pinned evidence | Decision |
| --- | --- | --- |
| Expanded/popup state | Pinned GPUI exposes `StatefulInteractiveElement::aria_expanded` at `references/zed/crates/gpui/src/elements/div.rs:1339-1343`; `AriaProperties.expanded` is written with `Node::set_expanded` at `references/zed/crates/gpui/src/elements/div.rs:1997-2005,3405-3410`. AccessKit 0.24.1 defines `Expanded` and `Node::set_expanded` at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/accesskit-0.24.1/src/lib.rs:2117-2123`. | **Implemented.** Add `accessibilityExpanded?: boolean`, append it to the accessibility tuple, and apply it through the GPUI builder. The dropdown and gallery triggers set the state. |
| Heading level | Pinned GPUI exposes `aria_level` at `references/zed/crates/gpui/src/elements/div.rs:1396-1400`; `AriaProperties.level` is written with `Node::set_level` at `references/zed/crates/gpui/src/elements/div.rs:2012-2015,3435-3437`. AccessKit 0.24.1 includes `Level` at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/accesskit-0.24.1/src/lib.rs:1992-2006`. | **Implemented.** Add positive `accessibilityLevel?: number`, require the `heading` role, append it after an expanded placeholder when needed, validate as a positive u32, and apply it through `aria_level`. |
| Live regions | AccessKit 0.24.1 defines `Live::{Off,Polite,Assertive}` at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/accesskit-0.24.1/src/lib.rs:2117-2142`, but pinned GPUI's `AriaProperties` at `references/zed/crates/gpui/src/elements/div.rs:1997-2020` has no live field, and its write path at `references/zed/crates/gpui/src/elements/div.rs:3392-3455` has no `set_live` call or public `aria_live` method. | **Upstream-gap.** Do not add a wire field or claim live-region support. Keep async status announcements documented as unavailable until GPUI exposes a public builder/write seam. |
| Label vs description | Pinned GPUI's public builders independently set label and description at `references/zed/crates/gpui/src/elements/div.rs:1271-1283`; the write path independently calls `Node::set_label` and `Node::set_description` at `references/zed/crates/gpui/src/elements/div.rs:3392-3401`. The renderer applies both independently at `crates/react-gpui/src/renderer/paint/accessibility.rs:33-38`. | **Implemented and audited.** Keep both fields distinct in the wire and docs; add source citations to the README/protocol table so this mapping cannot drift. |
| Validation parity | TypeScript prop encoding/validation is in `packages/react-gpui/src/renderer/props.ts:36-45,142-168,319-324`; Rust tuple conversion is in `crates/react-gpui/src/protocol/wire/node.rs:50-91,653-683`; retained-tree validation is in `crates/react-gpui/src/tree/validation.rs:137-171`. | **Implemented.** Both sides accept the same optional tail order, require heading for level, enforce positive u32 level, and preserve old seven-field tuples. Cross-language golden vectors are regenerated; focused protocol/type checks are the verification seam. |

## Wire shape

Accessibility is encoded as `[role,label,description,disabled,checked,selected,value,expanded?,level?]`. The first seven fields remain unchanged. When `level` is present, the renderer emits the eighth placeholder even if `expanded` is absent, preserving positional meaning. A tuple with neither tail remains seven fields; a tuple with only expanded has eight fields. The Rust decoder accepts omitted optional tails and defaults them to `None`.

The public props are available on all accessibility-capable host nodes:

- `accessibilityExpanded?: boolean` — optional expanded/collapsed state.
- `accessibilityLevel?: number` — positive u32, valid only with `accessibilityRole="heading"`.

`accessibilityDisabled` remains retained on the wire but unexpressible through the pinned GPUI public AX builder. Headless `TestPlatform` cannot activate or inspect the AccessKit tree; native-tree verification remains display-backed.
