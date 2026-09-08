# Website component audit — 2026-09-08

## Coverage

The inventory is in [manifest.json](manifest.json): 86 canonical component pages
and 216 example variants, all reviewed in the production GPUI WASM preview shell
at 1280 px. All 86 main previews and 58 layout-sensitive variants were also
reviewed at 390 px. These are visual coverage counts, not claims that every
possible interaction or platform behavior was tested.

Browser interaction checks exercised Accordion, Collapsible, Button, Checkbox,
Switch, Toggle groups, Slider, Rating, Pagination, TabBar, Sidebar, Tree,
Input, Editor, Textarea, OtpInput, NumberInput, Select, Combobox, ColorPicker,
DatePicker, Dialog, Sheet, DropdownMenu, ContextMenu, Notification,
Tooltip, HoverCard, ResizablePanelGroup, FocusTrap, Form, Scrollable, and
VirtualList. Checks included actual state changes, text entry, popup content,
focus returning from the last control to the first, panel dragging, and nested
scroll ownership. Tooltip removal was checked both after display and during its
pending display delay.

AppMenuBar, NativeMenu, TitleBar, and WindowBorder expose desktop-only platform
behavior. Their browser notices were reviewed; this audit does not claim OS menu
or window-management interaction coverage. Desktop JavaScript bundling and native
Rust tests passed.

## Corrections

- Shared structural patch planning now uses published sibling order. Previously,
  inserting rows at final indexes before deleting old rows reordered retained
  VirtualList content. The planner appends new nodes, moves retained descendants
  out of removed parents, removes old roots, then reconciles only changed sibling
  positions. It reuses the existing transaction journal and adds no persistent
  mirror tree. Rust applies actual TypeScript-generated patches in regression
  tests for forward/reverse/jump scrolling and retained-child reparenting.

- Tooltip and Collapsible examples now provide real named trigger slots.
- Accordion examples own their open state and respond to change events.
- Form renders explicit labels independently of placeholder indentation, and
  hidden fields leave no layout space.
- The shared bounds observer measures the trigger itself. Popup corner mapping
  now attaches to the opposite trigger edge, keeping HoverCard/Popover content
  outside the trigger. Tooltip requests observe their trigger lifetime so that
  navigating away clears displayed and pending content.
- Message compositions use avatar/header/footer slots and correctly inheriting
  text; filled Tag variants no longer put white labels on white backgrounds.
- Sidebar examples have bounded height, valid bundled icons, active selection,
  and a working collapsed presentation.
- Resizable panels have visible boundaries and wrapping text. PlotTooltip
  examples reserve their positioning area instead of covering adjacent code.
- TextView receives actual Markdown newlines. Clipboard, Scrollable, and
  FocusTrap examples now contain enough content to demonstrate their behavior.
- Nested scrolling is handled in the shared GPUI Div/List implementations: a
  viewport consumes a wheel only when it moves, and otherwise hands it outward.
- Maple Mono v7.9 standard TTF (no CN/NF additions) is embedded in the shared
  component initialization for Web and desktop. Code blocks, line numbers,
  inline code, type signatures, raw Markdown, and native theme mono consumers
  use the same family. Four font files report version 7.900 and identical
  600-unit ASCII advances. Upstream OFL and provenance are bundled.

## Verification and reproduction

`bun --conditions=browser test packages/solid-gpui/tests`: 72 tests passed.
Core and website TypeScript checks passed.

`bun run --cwd examples/website test`: 6 tests passed, including type checking
every catalog composition and rendering without DOM globals. Contract checks
cover trigger slots, filled Tag text, quoted escape sequences, and icon paths.

`cargo test -p solid-gpui --features gpui-component --lib`: 229 tests passed.
Key regressions reproduce explicit-label rendering, zero-height hidden fields,
trigger bounds, popup edge mapping, and nested scroll handoff. The field
visibility and popup position checks failed before the shared fixes.

`bun run --cwd examples/website build` and `build:native` passed. The Web build
includes the Rust WASM host and TypeScript checking. It retains Vite's existing
large-chunk warning.

The repeatable preview harness is documented in
`examples/website/tests/README.md`. Local visual evidence is in
`/tmp/website-preview-audit/`: `main-*`, `variants-*`, `narrow-main-*`,
`narrow-variants-*`, and `interaction-*` contact sheets/screenshots. A blank
capture during a development-server restart was recaptured before review.

Final standalone Chrome/WebGPU checks confirmed complete navigation population,
ordered rows after scrolling, retained sidebar position when opening Editor, and
English/Chinese Editor pages at 390 px. Evidence is in `final-production-*.png`.
The collaborative preview session became unreliable late in the audit; final
full-page checks therefore used a fresh isolated Chrome session, not those failed
captures.
