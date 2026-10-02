# Bounded recorded paint and native live-frame resources

Status: resolved
Execution: integrated and locally qualified; see integration record for platform limits
Blocked by: none

Implement spec outcome 9. Use existing generated NativeView/NativeModule rather than new host kinds/wire changes if possible. A concrete recorded-paint component must replay retained quad/path/text commands natively with explicit bounds/finite-coordinate/resource validation, resize/theme/DPI/clipping/transform semantics and no JS callbacks during native paint. Live-image/media resources must have host-owned opaque identity, bounded binary frame ownership/replacement/clear/dispose and epoch cancellation; never pointers as portable IDs or per-frame data URLs. Implement a useful cross-platform CPU frame path first within this same delivery, optional native acceleration only with honest support. Add a real interactive diagram/live-frame example and key renderer/resource acceptance. Own new component/module files; register in components only as required, regenerate from source. Synchronize docs/translations and website component catalog coverage.

## Final integration

Resolved in the integrated delivery. See [integration record](../integration.md),
[delivery note](../delivery-07.md), and `../qualification/` for
checks, measured counterexamples, and explicit unqualified platforms. Nothing
was published or installed into the user's active application.
