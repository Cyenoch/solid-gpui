# Pinned host tooltips

Status: resolved
Type: feature

`View` and `Pressable` accept a bounded `tooltip?: string` prop. The value is validated as a non-empty string of at most 256 UTF-8 bytes, carried in host-properties tag 5 as `[5,text]`, and rendered by the host using pinned GPUI's `Interactivity::tooltip`. The host constructs a minimal text-only tooltip view rather than depending on Zed's `crates/ui` package. GPUI owns hover tracking, viewport fitting, and the documented default 500 ms show delay.

Evidence: `references/zed/crates/gpui/src/elements/div.rs:662-675,1603-1611,3516-3571`, `references/zed/crates/gpui/src/app.rs:2990-3004`, and `references/zed/crates/gpui/src/window.rs:3202-3267`.

Headless protocol and wiring tests cover validation and tuple round trips. A display-backed smoke run is required to verify actual hover timing and placement; headless GPUI does not paint native tooltip windows.
