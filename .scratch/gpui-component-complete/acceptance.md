# Native component integration acceptance

Workspace baseline: `1bf74901244d8af95d7d547de5a0fb5ce362b2fc` plus this task's changes. Upstream component source is pinned at `928c3eb776a3d733d9b771f7dea27a6a79242ced`.

The [implementation matrix](implementation-coverage.md) maps all 138 ordinary rendering interfaces, plus composition descriptors and native services. The generated SDK exports 144 JSX components/descriptors and 10 native functions. Root, notification stacks and scroll machinery belong to their concrete host owners.

## Checks

- `bun run ci`: package build, generated native contract check, formatting, TypeScript, package tests, packed-package consumer checks, Rust workspace check, strict Clippy, workspace tests, advisory scans and generated third-party notices.
- Vendored library tests: 773 gpui-base and 423 gpui-component tests passed using the root target directory.
- Key added regression checks cover live native identity, controlled input/IME state, modal ownership, menu routing, settings composition, dock persistence and native plotting. Fixed textarea rows are checked against actual drawn input bounds. Settings inputs must paint inside their page at widths 800 and 1280.
- The renderer removal regression covers nested deletion, moving an existing subtree through a newly created parent, and a child moved out of a deleted parent.
- `repro-gallery.ts` previously failed during the eighth search/theme/route cycle with a repeated descendant deletion. The corrected renderer completes 80 cycles and validates 701 frames. It derives disjoint removals from the transaction's published ancestry.

## Actual macOS window

Built `gallery-host` in the debug profile and ran the actual Gallery entry through Bun. The performance monitor was disabled. Initial screenshots were 800 × 633; zoomed screenshots were 1304 × 768. These checks establish visible behavior, not GPU timing or production frame-rate acceptance.

| Surface | Observed result |
| --- | --- |
| Input and choices | Text input and Unicode paste update JS; switch/checkbox share controlled state; Select returns the chosen language. |
| Textarea and appearance | Fixed three-row height is visible; multiline Unicode paste works; native controls and application colors follow dark/light changes. This is not a physical IME composition test. |
| Date and pagination | Calendar selected 2026-09-16 and returned it to JS. Pagination's ellipsis accepted page 1000000000 within a 4294967295-page range without enumerating the range. |
| Menus and overlays | Actual macOS menu selection reaches JS. Dialog input, Escape and OK work; Sheet input and programmatic close work; in-app notification appears and expires. |
| Settings | Page/group/item/field composition draws; name and notification edits reach JS; reset restores both; native search selects Editor and shows its Language setting. |
| Dock | Input survives tab switches, zoom, save/restore and zoom-out. Native pane and layout commands return to JS. Pointer drag/resize did not produce a conclusive CUA movement sample; retain physical interaction acceptance for those gestures. |
| Data views | List and virtualized table rows draw in the native window. Tree's icon/label stacking was traced to ListItem's inner content wrapper and corrected with a horizontal child container. Final window reinspection of that last layout change was unavailable: CUA returned `cgWindowNotFound` after reconnect/relaunch, while the process remained alive with no logged renderer error. The final code passed full CI. |
| Charts and Plot | Line, Area, Bar, Candlestick, Pie, Radar and Sankey draw. Native hover tooltips appear on Line/Area. Plot primitives draw and the scale command returns ticks, nearest index and bandwidth. Native scrolling reaches the lower charts. |

The window checks found and resolved missing component icon assets, theme initialization order, fixed textarea rows being overwritten by content updates, ordinary Dialog's absent default footer, a redundant layout wrapper hiding Settings content, Tree row composition, duplicate default-slot construction and overlapping renderer deletions. The full API mapping is separate from exhaustive per-control manual acceptance.

## Evidence from this run

- `/tmp/solid-all-components-delivery-ci.log`: full CI after the final Tree row layout change (exit 0).
- `/tmp/solid-ui-vendor-final-tests.log`: both vendored native library suites.
- `/tmp/solid-settings-native-layout-test.log`: actual composed Settings field layouts.
- `/tmp/solid-textarea-height-test.log`: native three/six-row measurement.
- `/tmp/solid-renderer-deletion-regression.log`: key renderer and native JS checks.
- `/tmp/solid-gallery-tree-repro.log` and `/tmp/solid-gallery-tree-repro-fixed.log`: reproducible deletion failure and corrected sequence.
- `/tmp/solid-native-ui-commits-controls-charts.bin` and `/tmp/solid-native-ui-commits-settings-dock.bin`: captured native-window commit streams.

Non-fatal tooling notices remain for the upstream `block 0.1.6` future-compatibility report and Babel peer-dependency ranges in the packed-consumer smoke. Advisory scans passed; no new ignore rules were introduced.
