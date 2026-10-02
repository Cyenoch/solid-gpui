# Strict style authoring and native interaction refinements

Status: resolved
Execution: integrated and locally qualified; see integration record for platform limits
Blocked by: none

Implement spec outcome 7. Own Style authoring/normalization and canonical protocol schema/style generators/native style mapping. Add useful X/Y spacing, explicit supported length/overflow/flex-basis semantics where appropriate, and declarative hover/active/focus-visible refinements evaluated natively. Define precedence and reject unsupported tokens/units; no CSS cascade/fake DOM. Build one coherent small authoring interface rather than parallel wrappers. Preserve layout/text and native control contracts. This ticket owns wire changes and generated protocol/golden updates; coordinate conflicts during integration. Add key authoring→wire→native behavior tests, actual examples, docs/translations and website examples. No unsupported no-op API.

## Final integration

Resolved in the integrated delivery. See [integration record](../integration.md),
[delivery note](../delivery-04.md), and `../qualification/` for
checks, measured counterexamples, and explicit unqualified platforms. Nothing
was published or installed into the user's active application.
