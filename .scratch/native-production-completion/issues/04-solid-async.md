# Solid async composition

Status: ready-for-agent
Resolution: implemented and verified

## Acceptance

Run resource/Suspense/ErrorBoundary/lazy/transition pending, retry, and disposal fixtures in Bun and QuickJS; fix demonstrated defects.

## Comments

Implementation and verification evidence will be recorded here.

Implemented native-typed Solid control flow. Fixed empty-commit rollback destroying detached Suspense content. Production Bun and real QuickJS composition fixtures pass, including retry and late completion disposal; see ../async-bun.log and ../async-vm.log.
