# Ticket 04 delivery: strict native styles

Branch: `adopt/04-style-contract`. Baseline ancestry: `e879b6625d2590fc5bf1d74c6003e5919b21e92d` confirmed with `git merge-base --is-ancestor`.

## Delivered contract

- One Style interface: physical side > X/Y axis > all-sides shorthand, with zero retained and key order irrelevant. `paddingX/Y`, `margin`, `marginX/Y` lower to existing native side fields.
- Dimension authoring accepts numeric logical pixels, `{ unit: "px" | "rem" | "percent", value }`, or `"auto"` for width/height/min/max/flexBasis. Spacing, border, font and offset fields remain numeric pixels. Removed widthPercent/heightPercent and migrated maintained examples/guides; no old decoder or wrapper.
- Native `aspectRatio`, `overflowX/Y`, and flex basis map to GPUI layout. Per-axis overflow overrides its shorthand. Native rem values use the actual window root font size. Auto preserves native intrinsic sizing.
- Core View/Pressable `hover`, `active`, `focusVisible` accept only backgroundColor, color, borderColor and opacity. Native hit testing/click/focus state owns refinements, under stable node identity. Base < focus-visible < hover < active follows GPUI. Disabled nodes suppress refinements. Reject state layout/transition/nesting and incompatible gradient/edge-color combinations. Extension/native controls retain their own state APIs; core states on them fail explicitly.
- Hover events require an explicit onHoverChange subscription. Native styling alone allocates no JS listener; onPress no longer implicitly subscribes to hover. Focusable Pressable owns a native focus handle independently of listener demand. Removed the prevent-default handler that suppressed native active paint.
- Strict unknown-key checks, immutable nested style recipes, complete style equality for new and pre-existing edge/gradient fields, and pixel-only width/height interpolation across unit changes.

## Canonical wire ownership

Current protocol version: **7**. Current schema SHA-256: **de5fe6c96c85953d91bc06d40e6353f65c2c071e78d696dc1c1a55ed8d344d59**.

`protocol.bop` changes:

- Add LengthUnit enum: Pixels=0, Rems=1, Percent=2, Auto=3; add Length { unit, value }.
- Style fields 1/2/28..31 retain f32 values but require units. Repurpose 55/56 as widthUnit/heightUnit; remove separate percent fields. Add 73..76 min/max units. All present wire dimensions have explicit units; Auto requires value zero. Rust semantic numeric pixels use absent unit metadata as the numeric-pixel representation, and the current encoder always emits its explicit Pixels wire tag.
- Add Style 67 flexBasis; 68/69 overflowX/Y; 70/71/72 hover/active/focusVisible; 77 aspectRatio.
- Add bounded InteractionStyle record with RGBA u32 background/text/border and f32 opacity.
- Add required Node field 15 observesHover; PatchUpdate field 14 follows UPDATE_HOVER=1024 exact presence/mask rules. Updated native semantic/tree state and Rust literal fixtures accordingly. Existing invalid-mask test now uses 2048.
- Generated TypeScript/Rust Bebop, metadata/guard and schema lock regenerated; both producer golden sets now use v7 and cover explicit units/state/hover demand. Older version payloads are rejected.

## Generation for final combined integration

Use these after all native additions are merged; schema belongs to this ticket, native catalog belongs to the combined host:

```sh
export CARGO_BUILD_JOBS=2
export CARGO_TARGET_DIR=/private/var/folders/sl/42r5kc756mj96rlxkglrpr900000gn/T/opencode/solid-gpui-adoption-target
bun run task protocol-codegen
bun scripts/protocol-golden.ts fixtures/protocol
cargo run --locked -p solid-gpui --example protocol_golden -- fixtures/protocol/rust_to_ts.hex
bun scripts/protocol-golden.ts fixtures/protocol --verify
bun run task protocol-codegen-check
bun run task protocol-golden-check
bun scripts/native-codegen.ts
bun scripts/native-codegen.ts --check
```

If integration deliberately changes schema bytes, generate Bebop first, then accept the exact new digest with:

```sh
bunx bebopc --config packages/solid-gpui/src/protocol/bebop.json build
bun scripts/normalize-bebop.ts
bun scripts/protocol-schema-meta.ts packages/solid-gpui/src/protocol/protocol.bop --accept-schema-digest
bun run task protocol-codegen
```

Update protocol English/Chinese digest text to the final lock. No version negotiation is supplied.

## Evidence and synchronization

- Focused producer/wire tests: spacing precedence, explicit unit preservation, strict rejection before publication, refinement-only patches, explicit hover subscription addition/removal, disabled semantics and immutable recipes.
- Producer → wire → native TestAppContext checks: 50% flex basis yields a 96px child in the padded 200px container; Y scroll displaces it by 40px with X unchanged; 5rem at a 20px native root font yields 100px width and automatic 50px height at aspect ratio 2. Native hover/active/focus-visible paint changes use literal expected colors; disabled controls suppress them and styling sends no native interaction events to JS.
- Protocol malformed vectors: missing dimension unit, invalid auto basis value, opacity outside range and historical v6 rejection.
- Guides and their explicit Chinese copies synchronized: native composition, protocol, scrolling, system popovers, GPUI components; SDK README, documentation index, ADR active-version pointers, desktop sample/readmes and website guide text/translations. Website shared buttons now use native state paint; layout preview uses X/Y spacing/flexBasis. Added `fixtures/style-contract.ts` for a five-second real-host sample.
- Maintained docs/comments describe this project's contracts only. Removed external-design narrative from native composition and CONTEXT; factual licensed source metadata remains.

Focused verification results and integration reconciliation are recorded below after final checks. CPU TestPlatform scenes do not establish physical GPU presentation, display dropped frames or platform accessibility behavior. Browser/native host launch results are reported separately. No publication or deployment.
