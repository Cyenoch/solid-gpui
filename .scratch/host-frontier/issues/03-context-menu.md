# Context-menu boundary

Status: resolved
Type: decision

No context-menu API is added. The pinned GPUI `WindowKind::AnchoredPopup` path is a generic popup mechanism and is explicitly rejected on macOS; the pinned platform surface has no `NSMenu`/`popUpContextMenu` bridge. The sanctioned app-level pattern is the existing in-window overlay composition: right-pointer event, position state, an overlay `View`, and `onPointerDownOutside` dismissal. This is the right-click variant of the existing dropdown pattern and keeps menu contents, hit testing, and dismissal in application-owned SolidJS state.

Evidence: pinned GPUI platform popup support and macOS rejection in `references/zed/crates/gpui/src/platform.rs:653-657,2051-2057`, plus the existing overlay implementation in `crates/solid-gpui/src/renderer/paint/overlay.rs`.
