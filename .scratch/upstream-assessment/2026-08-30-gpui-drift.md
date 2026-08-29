# GPUI upstream drift assessment

Date: 2026-08-30

## Scope and references

This assessment compares the workspace's pinned Zed/GPUI source snapshot with the newest published GPUI line and the current Zed release/main source. The workspace requests GPUI `0.2.2` and patches it to Zed revision `6805d952f9f3d702f760aa11b1547df8a625fa16` (2026-08-24). The newest published GPUI tag available in the checkout is `gpui-v0.2.2` (`69e2130295c2649963eb639fc70b4f2ee8ea1624`, 2025-10-21). Stable Zed `v1.17.2` is the current released Zed comparison point (`c8e44cfa7bda9b2e22c8d6934d78969352e7f61a`, 2026-08-26); `origin/main` is `1662f5f3f6497c5f80830ccdca1edfd1fc0c6c6a` (2026-08-29).

The release source is the GPUI crate published as part of `v1.17.2`; the `gpui-v0.2.2` tag is included where the published-crate boundary matters. Main-only APIs are explicitly not counted as released unblocks. Existing project specs remain authoritative for already-resolved renderer decisions; this file does not reopen them.

## Verdict summary

| Boundary | Released GPUI (`gpui-v0.2.2` / `v1.17.2`) | Current pinned state | `origin/main` | Verdict / action |
| --- | --- | --- | --- | --- |
| Per-run `fontSize` / `lineHeight`, `TextRun`, `shape_line` / `shape_text` | `shape_line` and `shape_text` each take one paragraph `font_size`; `TextRun` has length, font, color, background, underline, and strikethrough only. | Same one-size paragraph API; React rejects nested run typography. | Same signatures and fields (line numbers shifted only). | **BLOCKED upstream in released API.** No issue: existing `.scratch/rich-text` boundary is resolved; keep paragraph-level typography. |
| RTL rendering | Backend shaping uses cosmic-text/unicode-bidi and emits positioned glyphs. | Supported for automatic/mixed rendering by pinned WGPU backend. | Still supported. | **SUPPORTED for rendering; no issue.** |
| Explicit RTL/base direction | No `TextStyle`/`TextRun` direction field and no shape parameter. | No caller-selected base direction. | No new direction field found. | **BLOCKED upstream.** Keep out of protocol; do not confuse automatic RTL rendering with an explicit direction contract. |
| Bidi-aware caret/hit geometry | `LineLayout` exposes raw glyph positions/indices and scans them for `index_for_x`/`x_for_index`; no bidi levels/affinity. | Same. | Same public geometry shape. | **BLOCKED upstream.** Existing text/link boundaries remain honest; no simulated per-glyph cursor. |
| `letterSpacing` | No `TextStyle` field or shaping adjustment. | Unsupported. | No released API; no main API found. | **BLOCKED upstream.** No issue or renderer approximation. |
| Secure/obscured text input | Released `InputHandler` exposes selection, composition, replacement, and geometry but no `TextInputConfiguration`/secure-obscuring semantic. | Renderer has no secure/password primitive and intentionally does not mask. | `TextInputConfiguration` and forwarding APIs were added by main-only commits `8fc1a8a0c6` (2026-08-26) and `d84e5d4992` (2026-08-27), but the type covers autocorrect/capitalization/suggestions/action, not password obscuring. | **BLOCKED for secure/password semantics; main-only for general input configuration.** Existing `.scratch/text-input-completeness/issues/04-secure.md` remains the record; no duplicate issue. |
| Image fallback | `StyledImage::with_fallback` and `with_loading` exist in released GPUI. | Renderer already exposes `fallbackSource`; no JS loader event. | Same fallback APIs. | **SUPPORTED and already implemented.** `Image.onError` is a separate upstream gap; no issue because fallback is resolved and callback has no release seam. |
| Image error callback | No `on_error`/loader error callback in released GPUI image API. | No event. | No released unblock found. | **BLOCKED upstream.** Keep `Image.onError` out of protocol. |
| `aria_live` / live regions | Released `AriaProperties` has labels, descriptions, state/value, orientation, level and set metadata, but no live field; writer has no `Node::set_live`. | No live-region wire or paint path. | Same omission in main. | **BLOCKED upstream.** Existing `.scratch/a11y-second-pass/issues/02-live-regions.md` is resolved; no duplicate. |
| Window-position setter | Released `Window` reads global bounds and can activate/minimize/resize, but no generic runtime position setter. | `openSurface` restores size/options only and accepts centered creation. | Main adds no generic runtime position setter. | **BLOCKED upstream.** Existing `.scratch/window-controls/issues/02-position-boundary.md` is resolved; no duplicate. |
| X11 clipboard image writes | Released X11 `write_to_clipboard` calls `set_text`; a private `set_image` helper exists but is unused. Wayland similarly writes text only. | Protocol rejects X11/Wayland image operations as `platform-unsupported`. | Main source retains the platform split; no portable write contract was found. | **BLOCKED / platform-uneven.** Existing `.scratch/interchange/issues/01-clipboard-images.md` is implemented with bounded macOS/Windows semantics; do not reopen as duplicate. |
| Per-glyph/per-range cursor support | Released `InteractiveText` uses clickable ranges for hit testing but calls `Window::set_cursor_style` on the containing hitbox. `LineLayout` has no cursor-region API. | React scopes node cursor handling around listener-bearing runs, but no reliable per-run/per-glyph native hover region. | No released precision API found. | **BOUNDED only; not an unblock.** Existing `.scratch/link-affordance/issues/01-per-run-cursor.md` records the failed precision experiment; no issue. |
| Fixed `block` release | Stable Zed and main still declare `block = "0.1"` and lock `block 0.1.6`; no official replacement release or upstream fix was found. | Apple dependency chain still reaches the warning. | Same `0.1.6` lock. | **BLOCKED / human dependency decision.** Existing `.scratch/release-productionization/issues/02-block-future-incompat.md` remains `ready-for-human`; do not file a duplicate issue. |

## Exact API evidence

### Text sizing and runs

Published GPUI's `TextSystem::shape_line` is at `crates/gpui/src/text_system.rs:365-371` in `gpui-v0.2.2` and `:397-403` in the pinned/current source; it accepts one `font_size` followed by styled runs. Published `shape_text` is `:409-416` in `gpui-v0.2.2` and `:509-516` in the pinned source, again with one paragraph `font_size`. Published `TextRun` is `:731-746`; the pinned equivalent is `:985-1000`. The struct has no run-level size or line-height field. The release `TextStyle` has paragraph `font_size` and `line_height` at `crates/gpui/src/style.rs:367-371` in `gpui-v0.2.2` (and the same fields at `v1.17.2:crates/gpui/src/style.rs:367-371`).

The stable release additionally exposes low-level layout primitives useful for bounded host-owned work: `LineLayout::index_for_x`, `closest_index_for_x`, `x_for_index`, and `font_id_for_index` at `v1.17.2:crates/gpui/src/text_system/line_layout.rs:56-125`, plus `TextLayout` position mapping in `v1.17.2:crates/gpui/src/elements/text.rs:829-884`. These are not a per-glyph cursor-region or bidi-affinity contract.

### Image fallback and interactive ranges

Released `StyledImage` stores `loading` and `fallback` builders and provides `with_fallback` / `with_loading` at `gpui-v0.2.2:crates/gpui/src/elements/img.rs:127-174` (same API in `v1.17.2:crates/gpui/src/elements/img.rs:128-176`). There is no error callback in that API. Released `InteractiveText` maps a hit index to clickable ranges and sets `CursorStyle::PointingHand` on the parent hitbox at `gpui-v0.2.2:crates/gpui/src/elements/text.rs:770-805`.

### Accessibility

Stable `AriaProperties` is listed at `v1.17.2:crates/gpui/src/elements/div.rs:1997-2020`; `write_a11y_info` writes the available fields at `:3392-3460`. Neither contains a live property or `set_live` call. Main's corresponding property list begins at `origin/main:crates/gpui/src/elements/div.rs:2004-2027` and remains live-free; its writer begins at `:3399`.

### Input and window APIs

Released `InputHandler` begins at `gpui-v0.2.2:crates/gpui/src/platform.rs:995` and provides native text selection/composition/replacement/geometry methods, but no `TextInputConfiguration`. Main introduces `TextInputConfiguration` at `origin/main:crates/gpui/src/platform.rs:1881-1944`; the feature landed after stable in commits `8fc1a8a0c6f40b9bbf7d71997138866a78563486` and `d84e5d4992bd445b8af8eeaf1717f4cf9c5d5a54`, so it is main-only and still does not constitute secure/password support.

Released and main `PlatformWindow` expose `bounds`, `window_bounds`, `content_size`, `resize`, activation, and minimize but no set-position method (`origin/main:crates/gpui/src/platform.rs:815-850`; `origin/main:crates/gpui/src/window.rs:2453-2455,5806-5819`). `WindowOptions.window_bounds` is a creation-time option at `origin/main:crates/gpui/src/platform.rs:1946-1953`; it is not a runtime setter.

### Clipboard and `block`

The published X11 adapter writes both primary and regular clipboard through `set_text` at `gpui-v0.2.2:crates/gpui/src/platform/linux/x11/client.rs:1555-1562`. Its private helper maps image formats to MIME atoms at `gpui-v0.2.2:crates/gpui/src/platform/linux/x11/clipboard.rs:991-1010`, but that helper does not make the public adapter image-capable. The workspace therefore keeps the documented `platform-unsupported` result for X11/Wayland rather than silently converting image bytes to text.

Both stable and main continue to declare `block = "0.1"` (`v1.17.2:Cargo.toml:561-562`; `origin/main:Cargo.toml:567-568`) and lock `block` at `0.1.6` with checksum `0d8c1fef690941d3e7788d328517591fecc684c084084702d6ff1641e993699a` (`v1.17.2:Cargo.lock:2329-2333`; `origin/main:Cargo.lock:2300-2304`). No official fixed release is present in these refs.

## Pin-to-release delta and upgrade impact

The pinned revision is newer than the published GPUI tag but still shares the `0.2.2` package version. Comparing pinned to stable Zed shows no useful released seam for the requested hard boundaries: the relevant text, image, accessibility, window, and platform APIs remain materially the same. Stable/main contain broad internal GPUI changes, including extraction of platform crates and line-layout history, but those changes do not add per-run typography, explicit base direction, letter spacing, secure obscuring, live regions, a runtime position setter, or portable Linux image writes.

The nearest released text change with a concrete reusable shape is the line-layout split/paint work in commit `2936989f1b7a15aaf7131b0a3c17961d706fdbf5` (2026-08-19), which adds `LineLayout::split_at` and low-level paint methods in current main (`origin/main:crates/gpui/src/text_system/line_layout.rs:128-188`; `origin/main:crates/gpui/src/text_system/line.rs:204-258`). It is not a per-run font-size or cursor API and does not change this project's verdicts.

Upgrading from the pinned revision to a future release would therefore require a full GPUI/platform graph review rather than a version-only edit. For this release, keep the current lockstep protocol and the explicit boundaries in the existing specs. The only main-only candidate discovered here is general text-input configuration; it should be reconsidered only after GPUI publishes it, and it does not unblock secure/password text.

## Issue disposition

No new issue files are created by this assessment. Every requested boundary is either already implemented/resolved in this repository, remains an upstream/platform blocker already recorded by an existing resolved or human-triage issue, or is main-only and therefore not a released unblock. Creating duplicate tickets would obscure the existing ownership and status decisions.

## Recommendation

Do not change the GPUI pin or protocol for these boundaries in the current release. Keep the existing renderer behavior: paragraph-level text sizing, automatic bidi rendering without an explicit base-direction promise, visual image fallback without `onError`, bounded macOS/Windows clipboard images, parent-hitbox interactive cursor behavior, and no secure-input masking. Re-audit when a published GPUI release provides a concrete API and platform semantics; then open one narrowly scoped issue per genuinely released, unimplemented capability.
