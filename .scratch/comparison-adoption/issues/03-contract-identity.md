# Separate native contract and build identities

Status: ready-for-agent
Execution: claimed
Blocked by: none

Implement spec outcome 4. Refactor NativeModule/NativeView definitions and generation so canonical contract digest describes exported types/slots/events/methods plus explicit semantic version; selected implementation source and exact SDK/build provenance are separate identity with clear diagnostics/checking. Comment/line-ending changes do not masquerade as contract differences. Strict host/application release matching survives; do not simply remove source hashing and weaken admission. Avoid wire schema changes if the generated native descriptor/module seam can own this. If schema changes are necessary, coordinate an explicit proposed delta for ticket 04/integrator rather than overwrite parallel schema work. Clean obsolete with_contract wrappers/callers, use meaningful mutation/version/line-ending tests, regenerate API from source. Synchronize CONTEXT/ADR/rust-bridge/protocol/distribution implications and translations as needed.
