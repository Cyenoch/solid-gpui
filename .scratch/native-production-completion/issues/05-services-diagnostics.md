# Native service lifecycle and diagnostics

Status: ready-for-agent

## Acceptance

Provide a bounded native progress/service example and actionable command/application diagnostics without logging payloads.

## Comments

Implementation and verification evidence will be recorded here.

Implemented structured NativeCommandError identity without retaining argument payloads; documented trace correlation. The bounded native progress/service example remains outstanding.


Implemented the generated WorkspaceScan native view and Gallery directory picker/progress/cancel UI. Real scanning is bounded to two workers, 100,000 entries, 4096 pending directories and a single coalesced progress slot at 20 Hz. Workers cancel on replacement/unmount and retain admission until they exit. Real directory/symlink/cancellation test passes.
