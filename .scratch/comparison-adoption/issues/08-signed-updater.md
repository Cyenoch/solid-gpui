# Explicit signed application update capability

Status: ready-for-agent
Execution: implemented and merged; review fixes in progress
Blocked by: none

Implement spec outcome 10 as an application-owned native module/interface. Trusted feed/key and exact platform/channel identity; bounded/cancellable downloads, cryptographic verification before staging/install, path/archive safety, atomic replacement/rollback, and explicit restart semantics. Follow GPUIX signed-updater design without copying code/license blindly. Use meaningful local HTTP/signed artifact fixtures and temporary app directories; never touch the running host/install, credentials, or public releases. Product packaging support from ticket 02 integrates through an explicit manifest/interface documented in delivery. No auto-enable, silent install, permissive unsupported OS path, or fake success. Update relevant docs/translations and an illustrative native-service example. Native dependencies/manifests/lockfiles may need merge reconciliation; run actual compile/tests.
