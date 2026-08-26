# Style slot gap matrix

This is a read-only style-surface comparison for the pinned GPUI revision and
this repository's current 42-slot style tuple. It records evidence and scope
decisions; it does not make an implementation claim.

## Evidence baseline

The pinned GPUI `Style` contains layout, spacing, border, paint, text, cursor,
opacity, and grid fields in `references/zed/crates/gpui/src/style.rs:180-321`.
Its defaults are in `references/zed/crates/gpui/src/style.rs:771-820`.
`TextStyle` is nested in the style and contains the text fields listed at
`references/zed/crates/gpui/src/style.rs:435-483`. Four-side `Edges` are
`references/zed/crates/gpui/src/geometry.rs:1747-1759`; four-corner `Corners`
are `references/zed/crates/gpui/src/geometry.rs:2252-2267`; grid placement is
`references/zed/crates/gpui/src/geometry.rs:3791-3810`.

GPUI's relevant enum/value semantics are visible at
`references/zed/crates/gpui/src/style.rs:337-345` (visibility),
`references/zed/crates/gpui/src/style.rs:1126-1141` (display),
`references/zed/crates/gpui/src/style.rs:1193-1223` (overflow), and
`references/zed/crates/gpui/src/scene.rs:594-603` (solid/dashed border style).

## Current 42 slots, aligned to GPUI

The Rust protocol `Style` has 42 fields at
`crates/react-gpui/src/protocol.rs:397-441`. Their positional wire order and
Rust decode mapping are at `crates/react-gpui/src/protocol/wire/node.rs:228-273`
and `crates/react-gpui/src/protocol/wire/node.rs:277-322`. The TypeScript
`Style` keys are at `packages/react-gpui/src/style.ts:52-95`, and the emitted
42-position array is at `packages/react-gpui/src/style.ts:583-634`.

| Slot | TypeScript key | Rust protocol field | GPUI target and present coverage | Evidence |
| ---: | --- | --- | --- | --- |
| 0 | `width` | `width` | `Style::size.width`; finite non-negative px only | `wire/node.rs:230-232`; `style.ts:583-585`; `renderer/paint.rs:1151-1153`; GPUI `style.rs:229-240` |
| 1 | `height` | `height` | `Style::size.height`; finite non-negative px only | `wire/node.rs:230-232`; `style.ts:584-585`; `renderer/paint.rs:1154-1156`; GPUI `style.rs:229-240` |
| 2 | `flexDirection` | `flex_direction` | `Style::flex_direction`; row, column, and reverse variants | `wire/node.rs:230-234`; `style.ts:586-594`; `renderer/paint.rs:1176-1184`; GPUI `style.rs:266-270,1160-1190` |
| 3 | `flexGrow` | `flex_grow` | `Style::flex_grow` | `wire/node.rs:233-234`; `style.ts:595`; `renderer/paint.rs:1185-1187`; GPUI `style.rs:271-275` |
| 4 | `padding` | `padding` | `Style::padding`; one px value copied to all four edges | `wire/node.rs:234-235`; `style.ts:595-597`; `renderer/paint.rs:1188-1190`; GPUI `geometry.rs:1747-1759` |
| 5 | `gap` | `gap` | `Style::gap`; one px value copied to row and column gaps | `wire/node.rs:235-236`; `style.ts:597`; `renderer/paint.rs:1191-1193`; GPUI `style.rs:262-264`, `gpui_macros/src/styles.rs:905-920` |
| 6 | `backgroundColor` | `background_rgba` | `Style::background` as one solid RGBA fill | `wire/node.rs:236-237`; `style.ts:598`; `renderer/paint.rs:1283-1285`; GPUI `style.rs:278-280` |
| 7 | `color` | `color_rgba` | `TextStyle::color`; GPUI text color cascades to children | `wire/node.rs:237-238`; `style.ts:599`; `renderer/paint.rs:1300-1302`; GPUI `styled.rs:511-516` |
| 8 | `opacity` | `opacity` | `Style::opacity`, finite `0..1` | `wire/node.rs:238-240`; `style.ts:600`; `renderer/paint.rs:1303-1305`; GPUI `style.rs:301-302` |
| 9 | `transition` | `transition` | Bridge-owned animation state for opacity/background/width/height; not a GPUI `Style` field | `wire/node.rs:239-240`; `style.ts:601`; `renderer/animation.rs:35-82`; `docs/protocol.md:244` |
| 10 | `justifyContent` | `justify_content` | `Style::justify_content`; six mapped values | `wire/node.rs:240-242`; `style.ts:602`; `renderer/paint.rs:1221-1231`; GPUI `style.rs:253-264` |
| 11 | `alignItems` | `align_items` | `Style::align_items`; five mapped values | `wire/node.rs:241-243`; `style.ts:603`; `renderer/paint.rs:1232-1241`; GPUI `style.rs:253-259` |
| 12 | `borderRadius` | `border_radius` | `Style::corner_radii`; one px value copied to all four corners | `wire/node.rs:242-244`; `style.ts:604`; `renderer/paint.rs:1254-1256`; GPUI `geometry.rs:2252-2267` |
| 13 | `borderWidth` | `border_width` | `Style::border_widths`; one px value copied to all four sides | `wire/node.rs:243-245`; `style.ts:605`; `renderer/paint.rs:1257-1259`; GPUI `geometry.rs:1747-1759` |
| 14 | `borderColor` | `border_color_rgba` | `Style::border_color`; one color for the border | `wire/node.rs:244-246`; `style.ts:606`; `renderer/paint.rs:1260-1262`; GPUI `style.rs:281-282` |
| 15 | `fontSize` | `font_size` | `TextStyle::font_size`; positive px only | `wire/node.rs:245-247`; `style.ts:607`; `renderer/paint.rs:1382-1384`; GPUI `style.rs:451-452` |
| 16 | `fontWeight` | `font_weight` | `TextStyle::font_weight`; only 400/500/600/700/900 | `wire/node.rs:246-248`; `style.ts:608`; `renderer/paint.rs:1385-1393`; GPUI `text_system.rs:883-977` |
| 17 | `overflow` | `overflow` | `Style::overflow.x/y`; one value copied to both axes, no `Clip` value | `wire/node.rs:247-249`; `style.ts:609`; `renderer/paint.rs:1263-1281`; GPUI `style.rs:187-220,1208-1223` |
| 18 | `lineClamp` | `line_clamp` | `TextStyle::line_clamp`, integer `1..100` | `wire/node.rs:248-250`; `style.ts:610`; `renderer/paint.rs:1365-1370`; GPUI `style.rs:481-482` |
| 19 | `textOverflow` | `text_overflow` | `TextStyle::text_overflow`; clip or end ellipsis only | `wire/node.rs:249-251`; `style.ts:611`; `renderer/paint.rs:1372-1380`; GPUI `style.rs:405-418` |
| 20 | `marginTop` | `margin_top` | `Style::margin.top`; px value only | `wire/node.rs:250-252`; `style.ts:612`; `renderer/paint.rs:1194-1196`; GPUI `geometry.rs:1747-1759` |
| 21 | `marginRight` | `margin_right` | `Style::margin.right`; px value only | `wire/node.rs:251-253`; `style.ts:613`; `renderer/paint.rs:1197-1199`; GPUI `geometry.rs:1747-1759` |
| 22 | `marginBottom` | `margin_bottom` | `Style::margin.bottom`; px value only | `wire/node.rs:252-254`; `style.ts:614`; `renderer/paint.rs:1200-1202`; GPUI `geometry.rs:1747-1759` |
| 23 | `marginLeft` | `margin_left` | `Style::margin.left`; px value only | `wire/node.rs:253-255`; `style.ts:615`; `renderer/paint.rs:1203-1205`; GPUI `geometry.rs:1747-1759` |
| 24 | `fontStyle` | `font_style` | `TextStyle::font_style`; normal or italic only | `wire/node.rs:254-256`; `style.ts:616`; `renderer/paint.rs:1336-1341`; GPUI `text_system.rs:967-977` |
| 25 | `textDecoration` | `text_decoration` | `TextStyle` underline/strikethrough basic enum | `wire/node.rs:255-257`; `style.ts:617`; `renderer/paint.rs:1343-1353`; GPUI `style.rs:463-470,824-843` |
| 26 | `lineHeight` | `line_height` | `TextStyle::line_height`; px only | `wire/node.rs:256-258`; `style.ts:618`; `renderer/paint.rs:1354-1356`; GPUI `style.rs:453-455` |
| 27 | `minWidth` | `min_width` | `Style::min_size.width`; px only | `wire/node.rs:257-259`; `style.ts:619`; `renderer/paint.rs:1206-1208`; GPUI `style.rs:233-240` |
| 28 | `maxWidth` | `max_width` | `Style::max_size.width`; px only | `wire/node.rs:258-260`; `style.ts:620`; `renderer/paint.rs:1209-1211`; GPUI `style.rs:237-240` |
| 29 | `minHeight` | `min_height` | `Style::min_size.height`; px only | `wire/node.rs:259-261`; `style.ts:621`; `renderer/paint.rs:1212-1214`; GPUI `style.rs:233-240` |
| 30 | `maxHeight` | `max_height` | `Style::max_size.height`; px only | `wire/node.rs:260-262`; `style.ts:622`; `renderer/paint.rs:1215-1217`; GPUI `style.rs:237-240` |
| 31 | `flexShrink` | `flex_shrink` | `Style::flex_shrink` | `wire/node.rs:261-263`; `style.ts:623`; `renderer/paint.rs:1218-1220`; GPUI `style.rs:273-276` |
| 32 | `alignSelf` | `align_self` | `Style::align_self`; seven mapped values | `wire/node.rs:262-264`; `style.ts:624`; `renderer/paint.rs:1242-1252`; GPUI `style.rs:255-258` |
| 33 | `position` | `position` | `Style::position`; relative or absolute | `wire/node.rs:263-265`; `style.ts:625`; `renderer/paint.rs:1157-1163`; GPUI `style.rs:222-227,1225-1246` |
| 34 | `left` | `left` | `Style::inset.left`; finite px offset | `wire/node.rs:264-266`; `style.ts:626`; `renderer/paint.rs:1164-1166`; GPUI `geometry.rs:1747-1759` |
| 35 | `top` | `top` | `Style::inset.top`; finite px offset | `wire/node.rs:265-267`; `style.ts:627`; `renderer/paint.rs:1167-1169`; GPUI `geometry.rs:1747-1759` |
| 36 | `right` | `right` | `Style::inset.right`; finite px offset | `wire/node.rs:266-268`; `style.ts:628`; `renderer/paint.rs:1170-1172`; GPUI `geometry.rs:1747-1759` |
| 37 | `bottom` | `bottom` | `Style::inset.bottom`; finite px offset | `wire/node.rs:267-269`; `style.ts:629`; `renderer/paint.rs:1173-1175`; GPUI `geometry.rs:1747-1759` |
| 38 | `cursor` | `cursor` | `Style::mouse_cursor`; codes map aliases and omit single-direction resize variants | `wire/node.rs:268-270`; `style.ts:630`; `renderer/paint.rs:1306-1326`; GPUI `platform.rs:2216-2302` |
| 39 | `textAlign` | `text_align` | `TextStyle::text_align`; physical left/center/right | `wire/node.rs:269-271`; `style.ts:631`; `renderer/paint.rs:1357-1363`; GPUI `style.rs:421-433` |
| 40 | `boxShadow` | `box_shadows` | `Style::box_shadow`; one or two shadows only | `wire/node.rs:271-272`; `style.ts:632`; `renderer/paint.rs:1286-1299`; GPUI `style.rs:347-360` |
| 41 | `fontFamily` | `font_family` | `TextStyle::font_family`; configured GPUI fallback still applies | `wire/node.rs:272-273`; `style.ts:633`; `renderer/paint.rs:1331-1335`; GPUI `styled.rs:707-736` |

## Difference A: GPUI fields with no bridge field

The following 19 GPUI fields have no positional bridge field. `corner_radii`,
`gap`, `padding`, `border_widths`, and related narrower cases are listed under
C because the current tuple represents a smaller subset rather than having no
field at all.

| A field | GPUI semantics | Current disposition | Evidence |
| --- | --- | --- | --- |
| `display` | Block, Flex, Grid, or None layout strategy | **B1**: expose block/grid/none; flex remains the existing default | `references/zed/crates/gpui/src/style.rs:180-185,1126-1141`; `references/zed/crates/gpui/src/styled.rs:36-61` |
| `visibility` | Visible or Hidden; Hidden still occupies layout space | **B1**: expose visible/hidden (`invisible` builder name) | `references/zed/crates/gpui/src/style.rs:337-345`; `references/zed/crates/gpui_macros/src/styles.rs:50-66` |
| `scrollbar_width` | Space reserved for scrollbars when overflow scrolls | **Deferred** to scroll-domain round | `references/zed/crates/gpui/src/style.rs:187-194,1193-1203`; `references/zed/crates/gpui/src/styled.rs:64-70` |
| `allow_concurrent_scroll` | Allow x and y scrolling together | **Deferred** to scroll-domain round | `references/zed/crates/gpui/src/style.rs:191-220`; `references/zed/crates/gpui/src/elements/div.rs:3197-3237` |
| `restrict_scroll_to_axis` | Keep wheel input on its dominant scroll axis | **Deferred** to scroll-domain round | `references/zed/crates/gpui/src/style.rs:195-220`; `references/zed/crates/gpui/src/elements/div.rs:1478-1484,3209-3230` |
| `aspect_ratio` | Preferred width/height ratio | **B1** | `references/zed/crates/gpui/src/style.rs:229-240`; `references/zed/crates/gpui/src/styled.rs:475-487` |
| `align_content` | Cross-axis distribution of flex/grid content | **B1** | `references/zed/crates/gpui/src/style.rs:253-261`; `references/zed/crates/gpui/src/styled.rs:416-473` |
| `flex_wrap` | NoWrap, Wrap, or WrapReverse | **B1**: expose nowrap/wrap | `references/zed/crates/gpui/src/style.rs:266-270,1143-1157`; `references/zed/crates/gpui/src/styled.rs:264-283` |
| `flex_basis` | Initial flex-item size, including auto/relative lengths | **B1** | `references/zed/crates/gpui/src/style.rs:271-274`; `references/zed/crates/gpui/src/styled.rs:215-220` |
| `border_style` | Solid or dashed border stroke | **B1** | `references/zed/crates/gpui/src/style.rs:281-285`; `references/zed/crates/gpui/src/scene.rs:594-603` |
| `text.background_color` | Text-run background, distinct from element fill | **Deferred** to avoid `text_bg`/`backgroundColor` confusion | `references/zed/crates/gpui/src/style.rs:435-464`; `references/zed/crates/gpui/src/styled.rs:527-532` |
| `text.font_features` | OpenType font feature settings | **Deferred** as low frequency | `references/zed/crates/gpui/src/style.rs:442-447`; `references/zed/crates/gpui/src/styled.rs:713-716` |
| `text.font_fallbacks` | Explicit ordered font fallback list | **Deferred** as low frequency | `references/zed/crates/gpui/src/style.rs:448-450`; `references/zed/crates/gpui/src/styled.rs:719-735` |
| `text.white_space` | Normal wrapping or nowrap text behavior | **B1**: expose nowrap | `references/zed/crates/gpui/src/style.rs:472-474`; `references/zed/crates/gpui/src/styled.rs:73-85` |
| `grid_cols` | Repeated grid column template and min-content/max-content mode | **Deferred** until display grid has landed; separate wire design | `references/zed/crates/gpui/src/style.rs:304-307,169-175`; `references/zed/crates/gpui/src/styled.rs:751-777` |
| `grid_rows` | Repeated grid row template and min-content/max-content mode | **Deferred** until display grid has landed; separate wire design | `references/zed/crates/gpui/src/style.rs:308-311`; `references/zed/crates/gpui/src/styled.rs:779-805` |
| `grid_location` | Row/column line or span placement | **Deferred** until display grid has landed; separate wire design | `references/zed/crates/gpui/src/style.rs:312-313`; `references/zed/crates/gpui/src/geometry.rs:3791-3810`; `references/zed/crates/gpui/src/styled.rs:807-888` |
| `debug` | Debug outline on the hovered element; debug builds only | **Internal/debug-only; not a production style candidate** | `references/zed/crates/gpui/src/style.rs:315-321`; `references/zed/crates/gpui/src/styled.rs:891-896` |
| `debug_below` | Debug outline propagated below an element; debug builds only | **Internal/debug-only; not a production style candidate** | `references/zed/crates/gpui/src/style.rs:315-321`; `references/zed/crates/gpui/src/styled.rs:898-903` |

A is therefore 19 field omissions, not a bounded four-slot change. The
conditional per-slot wire/validation/renderer cost estimate is not applicable.

### B1 selected implementation batch

The eight selected production-facing GPUI-native fields are:

1. `display`: block/grid/none; flex remains the current default.
2. `visibility`: visible/hidden, exposed through the existing GPUI
   `visible`/`invisible` semantics.
3. `flexWrap`: nowrap/wrap.
4. `flexBasis`.
5. `aspectRatio`.
6. `alignContent`.
7. `borderStyle`: solid/dashed.
8. `whiteSpace`: nowrap (normal remains the default).

This batch is a scope decision; the GPUI field and builder evidence is in the
A table above. No C-class expansion is included in B1.

### A-field use in Zed

The following A fields have concrete pinned-Zed use. These are use sites, not
just proof that a GPUI method exists:

| Field | Zed use evidence |
| --- | --- |
| `display` | `.flex()` and `.grid()` in `references/zed/crates/gpui/examples/grid_layout.rs:41-53`; `.hidden()` in `references/zed/crates/diagnostics/src/items.rs:32-34` |
| `visibility` | `.invisible()` and conditional `.visible()` in `references/zed/crates/agent_ui/src/agent_panel.rs:6279-6292` |
| `restrict_scroll_to_axis` | Horizontal code scroll uses it in `references/zed/crates/agent_ui/src/conversation_view/thread_view.rs:403-414` |
| `flex_wrap` | Search/selector layout uses `.flex_wrap()` in `references/zed/crates/agent_ui/src/agent_diff.rs:1248-1250` |
| `flex_basis` | Split editor panes use fractional `.flex_basis(...)` in `references/zed/crates/editor/src/split_editor_view.rs:217-231`; conversation panes use it at `references/zed/crates/agent_ui/src/conversation_view/thread_view.rs:3133-3137` |
| `aspect_ratio` | GPUI image layout derives an intrinsic ratio at `references/zed/crates/gpui/src/elements/img.rs:348-352`; explicit ratio regression uses `.aspect_square()` at `references/zed/crates/gpui/src/elements/img.rs:901-914` |
| `align_content` | Window example uses `.content_center()` at `references/zed/crates/gpui/examples/window.rs:104-107`; title bar uses `.content_stretch()` at `references/zed/crates/platform_title_bar/src/platform_title_bar.rs:286-288` |
| `border_style` | Failed message cards use `.border_dashed()` at `references/zed/crates/agent_ui/src/conversation_view/thread_view.rs:4265-4271` |
| `text.background_color` | Text-level selection/preview uses `.text_bg()` at `references/zed/crates/language_tools/src/highlights_tree_view.rs:589-604` |
| `text.font_features` / `text.font_fallbacks` | UI font construction sets both in `references/zed/crates/agent_ui/src/agent_registry_ui.rs:268-273` |
| `text.white_space` | Code blocks use `.whitespace_nowrap()` at `references/zed/crates/agent_ui/src/conversation_view/thread_view.rs:394-414` |
| `grid_cols` / `grid_rows` / `grid_location` | GPUI grid example uses columns, rows, and row/column spans at `references/zed/crates/gpui/examples/grid_layout.rs:50-58`; Git graph uses `.grid_cols(1)` at `references/zed/crates/git_ui/src/git_graph.rs:3690-3694` |
| `scrollbar_width` | No pinned app call to the GPUI `Styled::scrollbar_width` builder was found; editor scrollbar-width values at `references/zed/crates/editor/src/editor.rs:525-526,11049-11050` are editor-owned state, not this GPUI `Style` field |
| `allow_concurrent_scroll` | No pinned app use was found; the GPUI event path reads the field at `references/zed/crates/gpui/src/elements/div.rs:3197-3237` |
| `debug` / `debug_below` | No pinned app `.debug()`/`.debug_below()` use was found; only the cfg-gated GPUI builders are at `references/zed/crates/gpui/src/styled.rs:891-903` |

## Difference B: common RN/React properties absent from GPUI Style

These are representative production RN/React style names that have no
corresponding field in the pinned GPUI `Style`/`TextStyle`. They are not A
because the GPUI API itself cannot carry them.

| RN/React property | GPUI status and archive disposition | Evidence |
| --- | --- | --- |
| `zIndex` | No GPUI `Style` field; native overlap follows subtree/deferred order. Keep archived as upstream-unavailable. | `references/zed/crates/gpui/src/style.rs:180-321`; `docs/protocol.md:286-290` |
| `order` | No GPUI flex-item order field. Keep archived as upstream-unavailable. | `references/zed/crates/gpui/src/style.rs:266-276`; complete field list `style.rs:180-321` |
| `direction` / `writingDirection` | No container/text direction field. Explicit RTL base direction remains its own upstream gap. | `references/zed/crates/gpui/src/style.rs:435-483`; `docs/getting-started.md:420-421` |
| `letterSpacing` | No `TextStyle` letter-spacing field or shaping adjustment. Keep the existing upstream-gap archive. | `references/zed/crates/gpui/src/style.rs:435-483`; `docs/getting-started.md:409-411` |
| `textAlignVertical` | No vertical text-alignment field in `TextStyle`. | `references/zed/crates/gpui/src/style.rs:435-483` |
| `includeFontPadding` | No font-padding field in `TextStyle`. | `references/zed/crates/gpui/src/style.rs:435-483` |
| `textShadowColor` / `textShadowOffset` / `textShadowRadius` | No text-level shadow fields. Top-level GPUI `box_shadow` is an element shadow, not text-shadow semantics. | `references/zed/crates/gpui/src/style.rs:291-299,435-483`; `references/zed/crates/gpui/src/styled.rs:527-532` |
| `elevation` | No elevation field in GPUI `Style`; keep archived as platform-specific RN surface. | `references/zed/crates/gpui/src/style.rs:180-321` |
| `backfaceVisibility` | No backface-visibility field in GPUI `Style`. | `references/zed/crates/gpui/src/style.rs:180-321` |
| `opacity` | Not B: it is already bridged as slot 8. | `crates/react-gpui/src/protocol.rs:412-415`; `crates/react-gpui/src/protocol/wire/node.rs:238-240`; `crates/react-gpui/src/renderer/paint.rs:1303-1305` |

## Difference C: existing slots with narrower or different semantics

C means a slot exists, but it does not express the full GPUI field or the
usual RN/React semantic. These are intentionally outside B1.

| Current slot/property | Difference | Evidence |
| --- | --- | --- |
| `width`, `height`, `minWidth`, `maxWidth`, `minHeight`, `maxHeight` | Bridge accepts px numbers; GPUI `Length` also carries relative/auto lengths. | `packages/react-gpui/src/style.ts:52-95,195-199`; `references/zed/crates/gpui/src/style.rs:229-240`; `references/zed/crates/gpui/src/geometry.rs:3500-3788` |
| `padding` | One number fills all four sides; GPUI uses `Edges<DefiniteLength>`. | `crates/react-gpui/src/renderer/paint.rs:1188-1190`; `references/zed/crates/gpui/src/style.rs:242-251`; `references/zed/crates/gpui/src/geometry.rs:1747-1759` |
| `gap` | One number fills both axes; GPUI has `gap.width`/`gap.height` and `gap_x`/`gap_y`. | `crates/react-gpui/src/renderer/paint.rs:1191-1193`; `references/zed/crates/gpui_macros/src/styles.rs:905-920` |
| `overflow` | One value fills both axes and omits GPUI `Clip`; GPUI stores `Point<Overflow>`. | `crates/react-gpui/src/renderer/paint.rs:1263-1281`; `references/zed/crates/gpui/src/style.rs:187-220,1208-1223` |
| `borderWidth` | One number fills all sides; GPUI exposes four `border_widths` entries and per-side builders. | `crates/react-gpui/src/renderer/paint.rs:1257-1259`; `references/zed/crates/gpui_macros/src/styles.rs:1278-1325` |
| `borderRadius` | One number fills all corners; GPUI exposes four `corner_radii` entries and per-corner builders. | `crates/react-gpui/src/renderer/paint.rs:1254-1256`; `references/zed/crates/gpui_macros/src/styles.rs:1161-1224` |
| `backgroundColor` | Solid top-level `Fill` only; GPUI `Fill` also represents richer background forms. It is not `TextStyle::background_color`. | `crates/react-gpui/src/renderer/paint.rs:1283-1285`; `references/zed/crates/gpui/src/style.rs:278-280,435-464`; `references/zed/crates/gpui/src/styled.rs:527-532` |
| `fontSize` | Bridge is positive px; GPUI uses `AbsoluteLength`, including rems. | `crates/react-gpui/src/renderer/paint.rs:1382-1384`; `references/zed/crates/gpui/src/style.rs:451-452,493-504` |
| `lineHeight` | Bridge is px; GPUI uses `DefiniteLength`, including relative values. | `crates/react-gpui/src/renderer/paint.rs:1354-1356`; `references/zed/crates/gpui/src/style.rs:453-455` |
| `fontWeight` | Bridge offers five named weights; GPUI `FontWeight` supports the 100..900 numeric domain. | `packages/react-gpui/src/style.ts:546-557`; `references/zed/crates/gpui/src/text_system.rs:883-977` |
| `fontStyle` | Bridge omits GPUI `Oblique`; it exposes normal/italic. | `crates/react-gpui/src/renderer/paint.rs:1336-1341`; `references/zed/crates/gpui/src/text_system.rs:967-977` |
| `textDecoration` | Bridge exposes basic none/underline/line-through; GPUI underline carries thickness/color/wavy and strikethrough carries thickness/color. | `crates/react-gpui/src/renderer/paint.rs:1343-1353`; `references/zed/crates/gpui/src/style.rs:463-470,824-843` |
| `textOverflow` | Bridge exposes clip/end ellipsis; GPUI also has start and middle truncation. | `crates/react-gpui/src/renderer/paint.rs:1372-1380`; `references/zed/crates/gpui/src/style.rs:405-418` |
| `boxShadow` | Bridge limits one or two shadows; GPUI stores an unrestricted `Vec<BoxShadow>`. | `crates/react-gpui/src/protocol/wire/node.rs:194-224`; `references/zed/crates/gpui/src/style.rs:347-360` |
| `cursor` | Bridge maps aliases and combines some values; GPUI also has single-direction resize cursors. | `crates/react-gpui/src/renderer/paint.rs:1306-1326`; `references/zed/crates/gpui/src/platform.rs:2216-2302` |
| `textAlign` | Left/center/right are physical alignment values; no logical start/end direction semantics. | `packages/react-gpui/src/style.ts:23-24,631`; `references/zed/crates/gpui/src/style.rs:421-433`; `docs/protocol.md:274` |
| `marginTop` / `marginRight` / `marginBottom` / `marginLeft` | Four sides exist, but bridge only accepts finite px values while GPUI `Length` also supports auto/relative values. | `crates/react-gpui/src/protocol.rs:419-422`; `references/zed/crates/gpui/src/geometry.rs:1747-1759,3500-3788` |
| `flex` shorthand | RN shorthand is not a GPUI field; GPUI exposes independent grow/shrink/basis and convenience combinations. | `references/zed/crates/gpui/src/styled.rs:179-219`; complete GPUI fields `references/zed/crates/gpui/src/style.rs:266-276` |
| `transform` | Generic RN transform is not a GPUI `Style` field. GPUI's available `Transformation` is SVG-only, supports scale/translate/rotate, and does not affect layout/hitboxes. Keep archived as a semantic mismatch. | `references/zed/crates/gpui/src/elements/svg.rs:64-67,212-273`; `docs/getting-started.md:417-419` |
| `transition` | Bridge animation is intentionally limited to four properties and is not a GPUI Style field. | `crates/react-gpui/src/renderer/animation.rs:14-82`; `docs/protocol.md:244` |

## Archive and scope outcome

`zIndex`, `order`, and generic `transform` remain documented upstream or
semantic gaps; no new bridge slot is inferred from their RN names. C-class
expansions (four-way padding, per-axis overflow, per-side borders, per-corner
radii, and the full font-weight domain) are worthwhile but are explicitly
outside B1. Grid fields wait for a display-grid round; font feature/fallback
fields wait for frequency justification; scroll details wait for a scroll
round; text-level background waits to avoid a same-name semantic mistake.
