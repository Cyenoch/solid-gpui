# Surface lifecycle audit

## Goal

Audit the multi-surface lifecycle from native window creation through event routing,
close, renderer failure, and attempted reopen. Confirm whether each surface is
isolated for ordinary failures while shared-runtime failures still tear down the
runtime deliberately.

## Scope and invariants

- Every snapshot, patch, command, and event carries `surface_id` and `epoch`.
- A root accepts only frames for its own surface and epoch, with monotonic
  revision/event-sequence checks.
- Native close emits `EVENT_SURFACE_CLOSED` before removing that surface's
  registry entries.
- A native close or explicit root unmount retires the TypeScript surface id;
  changing the epoch does not authorize revival.
- A render/reconciliation error is root-local. Malformed protocol input,
  transport failure, or renderer-process termination remains shared-runtime
  fatal behavior.
- Focus and blur are observed per native window and routed with the owning
  surface identity.
- The application example demonstrates the supported open/register/render/close
  path; it does not attempt same-id reopen.

## Status map

| Area | Status | Evidence | Result |
| --- | --- | --- | --- |
| Per-surface vs shared-runtime isolation | resolved | `issues/01-isolation.md` | Root-local render errors remain recoverable and do not affect sibling roots; shared transport/runtime failures intentionally tear down all roots. |
| Close ordering, teardown, and reopen identity | implemented | `issues/02-close-ordering.md` | Native close ordering is preserved; TS host now tracks retired ids and raises `SurfaceIdReusedError` for explicit reuse. |
| Cross-surface focus and activation | resolved | `issues/03-focus-routing.md` | Real headless dispatch proves focus on surface 1, blur on activation of surface 2, and focus on surface 2 remain correctly identified. |
| Epoch validation and generation | resolved | `issues/04-epoch-validation.md` | Epoch is validated as identity/generation and cannot bypass retired-id protection. |
| Multi-surface example coverage | resolved | `issues/05-example-coverage.md` | Existing example covers supported lifecycle usage without adding a misleading reopen flow. |

## Implemented contract changes

- Public `SurfaceClosedError` carries `surfaceId` and is used for pending
  command rejection and public operations after native close or unmount.
- Public `SurfaceIdReusedError` carries `surfaceId` and rejects explicit
  registration of a retired id, including when a new epoch is supplied.
- `createSurfaceHost` retires ids on either native close or explicit unmount;
  auto-allocation remains monotonic and skips active/retired ids.
- Wire framing remains unchanged (`protocol=v3`); these are lifecycle and
  public-error semantics only.

## Verification

- Renderer suite, run in isolation: `66 pass, 0 fail, 288 expect calls`.
  Intentional invalid-style tests print their expected validation diagnostics.
  The earlier failure observed only in a combined/parallel invocation is
  classified as test-runner contamination because the isolated rerun passed.
- Headless native focus/blur roundtrip runs through `TestAppContext` and passes.
- Focused surface-host lifecycle tests and TypeScript package typecheck pass.
- Final repository gates and commit hashes are recorded by the integration
  owner in the handoff.