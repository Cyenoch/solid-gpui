# Pinned native component source

Vendored from [GPUI Kit](https://github.com/longbridge/gpui-kit) at `501c73923280859a5de2b16fe64d4aac960bb040` (0.6.1 plus subsequent changes), under Apache-2.0. This checkout contains `base`, `component`, `component-macros`, `assets`, the `kit` facade, and `fps`; the root workspace resolves GPUI to `gpui-pre 0.3.5` in its lockfile.

Consumers use the `solid-gpui` Rust facade and its generated JavaScript module. This source copy supplies the native state/configuration seams that declarative updates require. It is not a second application framework or a set of JS-painted replacement controls.

## Local changes

- **Theme initialization:** initialize Base before applying the Component theme, so Base initialization cannot erase the configured semantic tokens. The host synchronizes OS appearance and exposes explicit JS theme commands.
- **Dependency scope:** use the existing `web-time` clock and notify 8.2 instead of unmaintained `instant`; syntect belongs only to upstream showcase dev-dependencies. WASM asset HTTP uses upstream Reqwest 0.13.4 with default features disabled, matching the enclosing host adapter without enabling unused native TLS features. These changes remove bincode/instant/rustls-pemfile from the enclosing runtime graph without advisory suppressions. The enclosing repository's root `Cargo.lock` resolves these runtime crates; the vendored lockfile resolves the vendored crates alone — upstream story, shell, webview, LLRT and example entries are pruned, while `reqwest` and `tower-http` remain for the Kit asset and Base HTTP clients. Regenerate it with `cargo metadata --manifest-path vendor/gpui-kit/Cargo.toml` after a dependency change; the previous lock failed `--locked`.
- **Typed composition:** `ComponentChild<T>` carries the host render boundary until the native parent lays out its child. Native containers consume their concrete child types without erasing style, focus, events or identity. Deferred field/sidebar builders enter the child scope before constructing native content.
- **Mutable controls:** input/textarea/editor, OTP, slider, calendar/date picker, choices and rich text can update configuration while retaining their editing/selection entities. Textarea fixed rows can replace auto-grow, determine intrinsic rendered height and survive text/wrap updates; only auto-grow mode derives its rows from content. Asynchronous editor/choice work is invalidated by its current owner/configuration.
- **Virtualized data:** list, table, tree, command, virtual list and message scroller expose the state operations needed for keyed reconciliation, visible-range changes, native search and explicit load completion. Resize and scroll changes preserve native handles and reading anchors.
- **Overlays:** Root has owner/session tokens for dialogs, sheets and notifications. Closing or retiring an old owner cannot dismiss a newer overlay. DialogButtonProps exposes its localized native action footer for ordinary dialogs as well as alerts. Popover/tooltip and menu builders retain their live state. Notification replacement preserves native identity and timer ownership. A dialog popup bounds its height to the room the window leaves above a 24 px bottom gap (never below 160 px) and names itself `dialog-popup` in the debug-bounds map, so a body taller than the window scrolls inside the popup instead of pushing the title and footer off screen.
- **Notification card layout:** a toast is one row — `[icon] [title / message / content] [action]` — with a single 16 px inset on all four sides, 12 px between slots, and no absolutely positioned child. The icon and the action each sit in a slot one body line tall (at least 24 px), so their glyphs centre on the *first* line of wrapped copy; the copy column stretches to the row, so its own inset stays symmetric. The card draws **no** close control: a toast is dismissed by clicking it or by its own timer, and it carries the `notification-card`, `notification-icon`, `notification-copy` and `notification-action` test-only selectors.
- **Menus:** application-menu revisions update AppMenuBar; PopupMenu exposes its weak owner to builders running through a different entity and supports stable submenu adoption/rebuild. NativeMenu exposes combined checked/disabled/icon configuration and disabled submenus. A popup menu's leading item icon draws at its label's own scale (`Size::Medium`, 16 px) instead of one step below it, and names itself `menu-icon` in the debug-bounds map.
- **Settings:** page/group/item keys replace index-based identity. SettingsState exposes selection/search; search and reorder retain the intended page/group. Native input field setters and number options refresh when configuration changes.
- **Pagination:** ellipsis opens a native page-jump input with range validation, instead of allocating every omitted page. Opening a range remains constant work even for `u32::MAX` total pages. Labels are provided in the existing locale set.
- **Charts and plot:** point-scale indexed lookup preserves repeated category positions. Area/Radar styles stay aligned with their series. Per-bar optional fills retain theme defaults. Pie automatic/per-slice radii remain visible. Band scales preserve nonzero range origins; Plot labels honor their font weight.
- **Dock:** DockArea exposes its retained tab groups and refreshes panel metadata without replacing the layout. Closing or moving a pane adopts the source region's measured split sizes before editing, preserving the surviving panes' current proportions. The JS bridge implements the real native Panel traits and restores pane handles through an owner-scoped registry. Upstream removed the freeform tiles canvas; this copy carries no tiles layout, tile state or tile commands.
- **Empty states:** `EmptyHeader` also accepts direct children after its named slots, so the binding composes the header's media, title and description in JSX order instead of setting each typed slot. The rendered order and each part's native layout are unchanged.

The bridge validates serialized data and composition before publication, including numeric/work limits. Native snapshots, render tests and real-window acceptance are separate evidence; passing deterministic GPUI tests alone does not establish GPU/presentation behavior.

`crates/solid-gpui/src/components/` contains the adapters. `docs/gpui-components.md` describes their public JS API, ownership and limits. The pinned upstream inventory and investigation records live under `.scratch/gpui-component-complete/` in the enclosing repository.

## GPUI Kit adoption seams

- Carousel exposes atomic item/selection reconciliation for keyed Solid children.
- Editor exposes directed cursor selections with UTF-8/CRLF, mode and IME validation
  before mutation. Local grapheme editing and controlled-state behavior are preserved.
- FPS exports its headline mode and overlay offset. ComponentHost retains the monitor
  per window, starts in observed mode, and leaves it disabled unless explicitly enabled.
- Shell and Component Shell are not vendored or linked. Solid keeps its own runtime.
- Kit assets serve internal default control glyphs only. Web embeds exactly
  `crates/assets/default-icons.txt`, avoiding the full Lucide catalog and CDN loader.
  Application icon slots resolve through Solid's Iconify/registered SVG catalog.
  Icon exposes shared SVG storage so retained slots do not copy bytes or store
  full style objects in every table/menu data record.
- `kit` is used by the isolated native integration tests so test-support cannot
  accidentally select the Kit facade in production procedural macros.
- Composite empty states are exposed as the six `empty` parts. Upstream's
  `ScrollBounce` (a touch scroll-bounce wrapper for mobile hosts) and the
  `text_view` inline plugin (a native plugin host) are not bound: no SDK element
  wraps them, and the desktop host opts into neither. A `Panel::title_bar` hook
  replaces the removed tiles canvas for a panel that draws its own chrome.
