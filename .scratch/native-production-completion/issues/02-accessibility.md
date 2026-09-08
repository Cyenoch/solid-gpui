# Native accessibility semantics

Status: ready-for-agent

## Acceptance

Add live and disabled semantics through the actual GPUI Element accessibility seam, preserve identity/actions, and test the resulting AccessKit nodes.

## Comments

Implementation and verification evidence will be recorded here.

Implemented extended semantic roles, live-region validation/projection, and disabled state on the existing AccessKit node. Added independent live-priority update regression and Gallery status semantics. Native/TS projection passes. Actual assistive-technology speech/focus qualification is still outstanding.
