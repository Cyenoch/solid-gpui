# ADR-0005: Reuse native press semantics for keyboard activation

- **Status:** Accepted
- **Date:** 2026-08-25

## Context

Interactive host nodes need one meaning for pointer activation and keyboard
activation. GPUI already owns focus handles, tab-stop traversal, click
recognition, and disabled state. React receives a semantic press notification
rather than a browser DOM event. Creating a second keyboard-only path in the
host would make focusable Pressables behave differently from pointer-clicked
Pressables and would make disabled handling easy to get wrong.

## Decision

Keyboard activation of a focusable, enabled Pressable reuses GPUI's native
click/press semantics. The host must not manufacture an independent protocol
`Press` event from a key handler. Disabled Pressables are not keyboard
activation targets and must not emit a press; focus traversal continues through
the native tab-stop graph and `Root.focusNext()`/`Root.focusPrev()` commands.
## Scope clarification

The no-fabrication rule above is about the pointer surface: pointer handling must not invent an `Event::press` outside GPUI's native click/press dispatch. It does not forbid the separate, bounded keyboard-accessibility path for interactive Text runs. As defined by [ADR-0013](0013-interactive-text-runs.md), a focused nested run may emit the existing `Event::press` for one unmodified, non-held `Enter` keydown. That event is keyboard activation equivalence, not fabricated pointer activation; `Space`, modified or held `Enter`, and unfocused runs remain non-activating.

The resulting JavaScript callback remains the same semantic `onPress` path as
pointer activation. Key notifications remain available for applications that
need to observe key identity, but observing a key is not a second activation
mechanism.

## Alternatives rejected

- **Have a Rust key handler directly emit `Event::press` for a Pressable:** rejected because a pointer click and a keyboard activation could both reach the callback for one native action, and the two paths would drift in sequence, disabled, or focus semantics. The bounded focused-run exception is defined separately by ADR-0013.
- **Have JavaScript synthesize `onPress` from `onKeyDown`:** rejected because
  it adds transport latency, duplicates native activation policy, and makes
  every application reimplement Enter/Space and repeat filtering.
- **Copy GPUI `ButtonLike` behavior wholesale:** rejected because React GPUI
  has a smaller semantic Pressable contract and must preserve its own
  accessibility/disabled validation rather than expose unrelated GPUI widget
  internals.
- **Let disabled nodes remain in the activation path and only suppress the
  callback:** rejected because a disabled control must be removed from the
  actionable focus/activation semantics, not merely hide a JavaScript side
  effect after native work has occurred.

## Consequences

- Pointer and keyboard activation share one observable press contract and one
  listener identity.
- Native tests must prove enabled activation, disabled suppression, focus
  traversal, and no duplicate press when a key and click arrive together.
- Applications can still observe key events, but keyboard shortcuts that are
  not control activation remain an explicit application concern.
