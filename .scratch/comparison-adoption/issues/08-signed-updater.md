# Explicit signed application update capability

Status: resolved
Execution: integrated and locally qualified; see integration record for platform limits
Blocked by: none

Implement spec outcome 10 as an application-owned native module/interface. Trusted feed/key and exact platform/channel identity; bounded/cancellable downloads, cryptographic verification before staging/install, path/archive safety, atomic replacement/rollback, and explicit restart semantics. Keep legal provenance for any copied code. Use meaningful local HTTP/signed artifact fixtures and temporary app directories; never touch the running host/install, credentials, or public releases. Product packaging support from ticket 02 integrates through an explicit manifest/interface documented in delivery. No auto-enable, silent install, permissive unsupported OS path, or fake success. Update relevant docs/translations and an illustrative native-service example. Native dependencies/manifests/lockfiles may need merge reconciliation; run actual compile/tests.

## Final integration

Resolved in the integrated delivery. See [integration record](../integration.md),
[delivery note](../delivery-08.md), and `../qualification/` for
checks, measured counterexamples, and explicit unqualified platforms. Nothing
was published or installed into the user's active application.
