# Application activation and lifecycle

Status: ready-for-agent

## Acceptance

Bounded typed activation events, URL/document/reopen routing, and explicit last-window/quit policy with lifecycle tests.

## Comments

Implementation and verification evidence will be recorded here.


Implemented application-scoped configuration and acknowledgement, bounded launch/reopen/open-URL events, zero-window transport ownership, fresh Surface allocation, HMR replay and explicit quit. Rust host and TypeScript lifecycle tests pass. Windows/Linux OS callback ingress and cross-process single-instance delivery remain separate work.

The host now tracks the application root Surface independently from auxiliary
windows. Closing the app root while an auxiliary window remains allocates a fresh
root on activation, retains the auxiliary, and routes subsequent activations to
the new root. Reopen/open-URL delivery brings that window forward. Solid keeps the
application transport after root closure under both last-window policies; the host
is authoritative about whether the last native window has closed. Native and
TypeScript regressions cover these cases.
