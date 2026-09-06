# ADR-0009: Async close confirmation across the process boundary

- **Status:** Accepted
- **Date:** 2026-08-29

## Context

GPUI asks whether a native window may close through a synchronous
`Window::on_window_should_close` callback. Solid application confirmation, however, runs
through a `RuntimeAdapter`: the host must emit a Native Event, wait for a
renderer callback or Promise, and receive a later Surface Command. A process
transport cannot block that native callback waiting for JavaScript, and the
same constraint is useful to preserve for the embedded adapter.

The close path also has per-Surface lifecycle state. The host must distinguish a
second native close attempt while a decision is pending from a new request, and
an eventual allowed close must not re-enter the callback that just vetoed it.
The relevant seams are `crates/solid-gpui/src/host/mod.rs`,
`crates/solid-gpui/src/protocol.rs`,
`crates/solid-gpui/src/protocol/wire/event.rs`,
`crates/solid-gpui/src/protocol/wire/command.rs`, and
`packages/solid-gpui/src/renderer/root-container.ts`.

## Decision

- Keep `ClosePolicy` in the host, per Surface, with permissive `allow` as the
  default and `require-confirmation` as the asynchronous policy.
- Under `allow`, the native callback returns `true` and the ordinary close
  lifecycle proceeds. Under `require-confirmation`, the callback immediately
  returns `false`, records one pending request ID, and emits the root-scoped
  `EVENT_CLOSE_REQUESTED` Native Event.
- JavaScript resolves that request with the root-only
  `resolveCloseRequest(requestId, allow)` Surface Command. A denial clears the
  pending request and leaves the native window open. An approval clears it,
  acknowledges the command, and schedules the host's `window.remove_window`
  path so the approved close does not re-enter the veto callback.
- Only one request may be pending per Surface. Repeated native attempts are
  vetoed without duplicate events. Unknown, stale, and already-resolved IDs are
  acknowledged or ignored without affecting another request.
- Policy changes, transport shutdown, and Surface teardown clear pending close
  state. Application quit and Wayland layer-shell teardown remain outside this
  per-window callback contract.

## Alternatives rejected

- **Block the native callback until JavaScript answers:** rejected because the
  Runtime Adapter is asynchronous and a process transport cannot synchronously
  call into JavaScript; blocking would also risk stalling the native executor.
- **Let the native close proceed and undo it if JavaScript denies:** rejected
  because the window may already be destroyed, and denial would no longer be a
  reliable close policy.
- **Have JavaScript own a global close flag:** rejected because policy and
  pending request identity belong to individual native windows, while a shared
  flag would mix Surfaces and make routing ambiguous.
- **Emit a new request for every native attempt:** rejected because repeated
  attempts while a decision is pending do not represent independent user
  decisions and would create races between stale approvals and newer requests.
- **Approve by invoking the ordinary close callback again:** rejected because
  it would re-enter the veto path and could produce an approval loop; direct
  removal after clearing the pending state is the unambiguous host operation.

## Consequences

- Close confirmation is intentionally asynchronous; Solid GPUI does not claim
  a synchronous `preventDefault()` API for native close.
- The host owns the short-lived policy and request state, while JavaScript owns
  the confirmation UI and decision. The wire gains one event and one command,
  but no blocking or callback transport is required.
- Consumers must handle a request at most once and treat a closed or retired
  Surface as terminal. The lifecycle and public error behavior remain governed
  by `packages/solid-gpui/src/surface-host.ts` and the existing Surface
  retirement contract.
- Any future confirmation of application-level quit or layer-shell teardown
  needs a separate native callback boundary and a new decision; it is not an
  extension of this per-window policy by implication.
