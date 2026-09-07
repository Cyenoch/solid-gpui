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

A module's identity comes from its namespace. Its contract digest derives from
exported types, component/event/method metadata, and declarations visible to the
macro. It detects differences between registered and generated contracts; it is
not a digest of the entire executable. Rust implementation changes require a
host rebuild. Instance methods execute after a complete Solid transaction has
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
