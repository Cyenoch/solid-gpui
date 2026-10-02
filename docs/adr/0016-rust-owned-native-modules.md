# Rust-owned native modules

Application components and commands are declared by the Rust modules linked
into the actual host. That host exports TypeScript with `--export-native`.
`solid-gpui` supplies the runtime, host, annotations, and optional gpui-component
integration; `@solid-gpui/core` supplies the JavaScript runtime, generated
components, and Vite tools. An application owns its Rust crate without separate
packages for schemas, exporters, adapters, or JavaScript facades.

The design retains bounded Bebop transport and carries strict JSON DTOs in the
Extension/InvokeNative bytes fields. Serde and ts-rs derive the types. This
supports nested structures, enums, default properties, and asynchronous
application functions through the same interface in process and embedded modes.
Direct JSC/GPUI object exposure was rejected because threading, lifetimes, and
process mode cannot share that model. A separate IDL was rejected because it
would split Rust implementations from their declarations again.

Stateless component functions return GPUI Elements. Stateful components
implement NativeView, with the host owning their Entity, Subscription, and Task
lifetimes. Properties update at commit; painting does not repeatedly parse DTOs.
Native inputs retain editing state, and returning the same value does not call
upstream `set_value`. Controlled values return synchronously; asynchronous
application processing can run separately.

A module's identity comes from its namespace. Its canonical contract digest derives
from exported DTO syntax, props, slots, child rules, events, methods, controlled
bindings, and explicit behavioral semantic versions. DTO documentation is still
exported but does not affect this digest. Every module declares an explicit
`version = "major.minor.patch"`; direct registrations supply the same version to
`ModuleDefinition::new`. Changes to defaults, validation, lifecycle, or other
observable behavior require a version bump even when types stay the same.

A separate build digest locks the contract to normalized selected implementation
sources, the exact SDK version, and SDK source/dependency provenance. Generated
native props and invocation arguments carry an `SGN` format-2 envelope containing
the 32-byte build digest before their strict JSON DTO. Admission rejects missing
envelopes and mismatched builds before decoding DTOs, publishing a component tree,
or running a command. Events and results remain strict JSON. The existing Bebop
byte fields carry this envelope; no canonical wire field or historical decoder is
needed. LF/CRLF sources normalize to one build identity; comments can change build
provenance without masquerading as public-contract changes. `with_implementation`
records selected sources and replaces the old `with_contract` API.

The build identity is portable source provenance, not an executable hash: it
does not certify toolchains, target-specific artifacts, signing, or every
application dependency. Applications must include additional implementation inputs
when their module depends on them, and distribution still pairs exact release
artifacts. Host exporters expose `nativeIdentity` metadata for inspection and
release checking. Rust implementation changes require a host rebuild and binding
regeneration. Instance methods execute after a complete Solid transaction has
committed, with node identity, epoch, and revision validation. Unmount revokes
event routes and rejects pending JavaScript instance calls.

Asynchronous application commands run on a Tokio runtime shared by the module
collection; synchronous commands enter its blocking pool. The GPUI executor
does not replace Tokio's timer and I/O drivers. Bounded capacity and cancellable
tasks tie requests to the Surface lifecycle. Results return to GPUI before
entering the JavaScript event loop as messages. Runtime destruction starts
background shutdown so the UI thread does not wait for application threads.
Already-running blocking functions and application-detached child tasks have
no automatic forced-cancellation guarantee.
