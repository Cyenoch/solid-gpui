# Clipboard read events

Status: resolved
Verdict: future

## Question

Does pinned GPUI expose a clipboard-change observer worth forwarding as a
JavaScript event?

## Evidence

- The public GPUI platform interface exposes pull reads/writes and an async
  read fallback, but no observer: `references/zed/crates/gpui/src/platform.rs:310-321`.
- X11's `data_changed` condition variable is private selection bookkeeping at
  `references/zed/crates/gpui_linux/src/linux/x11/clipboard.rs:185-194,255-275`;
  it is not an application callback and has no cross-platform equivalent in
  the public seam.

## Decision and action

Keep `getClipboardText()` pull-only. Do not add a speculative subscription or
notification event without a concrete consumer and a cross-platform ownership,
lifecycle, and throttling contract. Revisit when demand identifies those
requirements.
