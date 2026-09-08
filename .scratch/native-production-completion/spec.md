# Native production completion

User scope: address the capability audit with maintainable implementations and key verification. Cross-platform real-desktop qualification is explicitly deferred while Windows 11 is installed in UTM. MIT licensing was committed as 6966603 and remains in place.

Solid owns composition/reactivity; GPUI owns rendering/editing/focus. No historical compatibility wrappers. Optional product services stay app-owned; provide reusable contracts/examples where justified.

- [Per-call native cancellation and deadlines](issues/01-native-cancellation.md)
- [Native accessibility semantics](issues/02-accessibility.md)
- [Application activation and lifecycle](issues/03-application-lifecycle.md)
- [Solid async composition](issues/04-solid-async.md)
- [Native service lifecycle and diagnostics](issues/05-services-diagnostics.md)
- [Native layout and internationalization](issues/06-layout-internationalization.md)
- [Representative workload verification](issues/07-workloads-verification.md)


## Current implementation status

Implemented and tested: request cancellation/deadlines/cooperative context;
native Solid async composition; app-scoped readiness and acknowledged activation,
zero-window/HMR/reopen/quit ownership; bounded real filesystem progress example;
primitive and generated-editor grapheme navigation/deletion; native grid tracks
and spans; application-owned system/reduced/full motion preference with native OS notification subscriptions.

AccessKit disabled/live projection and extended roles are implemented. Actual AT
speech/focus qualification is outstanding. macOS GPUI activation hooks are wired;
Windows/Linux OS activation and cross-process single-instance forwarding remain
implementation gaps, separately from deferred desktop acceptance.

Full bidi caret/selection/wrapped geometry remains implementation work. See bidi-implementation.md for executed failures and
the exact GPUI/platform contract refactor. No RTL alignment workaround is claimed
as visual editing support.

Combined native workloads now pass: 20,000 keyed rows through middle/end scroll,
reverse/append/filter and compact resize, with retained selection and bounded
render counts; a 1000-line multilingual editor through IME composition, parent
updates, resize, delayed stale acknowledgement and undo. These are native
correctness/work-bound tests, not display frame cadence measurements.

Current checks and raw logs are retained alongside this spec. The earlier package
validation (294 Rust / 84 TS) predates the latest implementation. Current focused
checks include 221 Rust library cases including the new workloads/motion reducer, 64 core
TS cases, the vendored cross-chunk grapheme regression, and package type checks.
Image rendering is corrected: HTTP(S) sources reach the HTTP loader, the native
host installs its HTTP client, response reads retain Tokio context, and inline
sources share bounded validation with fallback sources. Packaged macOS Gallery
acceptance passed for remote JPEG, inline SVG, failed-primary fallback, all five
object-fit modes, and height changes. See [Image diagnosis](image-diagnosis.md).
Current Image regression tests, package type checks, native binding consistency,
and workspace Clippy passed. Full regression and final packaging logs use the
`image-final-*` prefix. Earlier counts above are historical snapshots.
Concurrent Web/WASM changes are preserved.
