# Rust-to-JavaScript binding research and low-overhead authoring

Research dates: 2026-09-05 through 2026-09-06. Sources were official documentation, maintainer source, and the Rust Reference. Designs are proposals, not existing repository APIs, and no runnable prototype was built. This appendix includes an independent alternative; the [combined report](report.md) records the final recommendation.

## Conclusion

Adopt the established pattern of ordinary Rust declarations plus annotations producing one metadata model that generates both Rust dispatch and TypeScript bindings. Preserve byte-only transport, Solid ownership, and GPUI foreground execution. Existing tools supply parts of this design, but none directly integrates Solid JSX, GPUI Entity lifecycles, and this transport protocol. That is an engineering inference from the sources and architectural constraints, not an upstream claim.

| Technology                | Capability established by primary sources                                                                                | Reusable idea                                                                              | Capability not established                                                                                 |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------- |
| napi-rs                   | Rust structs/impls export JavaScript classes, accessors, and TypeScript; async fn becomes Promise; thread-safe callbacks | Annotation experience, separate value/identity objects, generated proxies/error conversion | Arbitrary gpui-component builder export, cross-thread GPUI state, or JavaScript GC as Surface lifecycle    |
| wasm-bindgen              | Rust exports become JavaScript classes; reference parameters, TypeScript definitions, Future/Promise conversion          | One declaration source, generator pipeline, explicit ownership                             | Unchanged native GPUI rendering in Wasm or replacement of the native byte bridge                           |
| UniFFI                    | Proc macros/UDL generate Rust scaffolding and foreign proxies; Arc objects and async FFI                                 | Metadata IR, value/object separation, explicit release                                     | Official ready-to-use embedded Bun backend or foreground-only GPUI objects as Send+Sync interfaces         |
| Tauri 2 + tauri-specta v2 | Commands, state, channels; one builder registers commands/events and exports TypeScript                                  | Context injection, shared registration/export, distinct requests/events                    | Direct substitution into a WebView-free host or command bindings supplying JSX instance lifecycles         |
| Specta v2                 | Type derives form a dependency graph; TypeScript export and separate serialization-format interpretation                 | Candidate internal generation model reducing handwritten TypeScript                        | Reflection over arbitrary third-party semantics, automatic GPUI adapters/children/events, or binary codecs |

## 1. napi-rs: learn from the author experience

`#[napi] struct` exports a JavaScript class backed by an identity-bearing Rust value; `#[napi(object)]` exports copied records. Constructors, public fields, accessors, methods, and TypeScript declarations can be generated while native internals remain private. Rust values normally drop with JavaScript garbage collection. This demonstrates generatable glue, but GC lifetime differs from explicit UI unmount. [Class](https://napi.rs/docs/concepts/class)

Async exports require features and normally use napi-rs's Tokio runtime, returning Promises. `async fn(&mut self)` explicitly requires unsafe because JavaScript still owns the object. Promise conversion alone does not solve UI-object concurrency. [async fn](https://napi.rs/docs/concepts/async-fn)

`napi_env`, `napi_value`, and `napi_ref` cannot be accessed arbitrarily across threads. `ThreadsafeFunction` schedules callbacks into JavaScript, supports bounded queues, and reports queue-full in nonblocking mode. Even with N-API, this project would still need foreground dispatch, event backpressure, and owner-release design. [`ThreadsafeFunction`](https://napi.rs/docs/concepts/threadsafe-function)

Bun officially loads `.node` modules and its documentation claimed about 95% Node-API support at the research date. That percentage does not qualify the embedded Bun build, target platform, or required API subset and cannot justify changing the bridge by itself. [Bun Node-API](https://bun.sh/docs/runtime/node-api)

## 2. wasm-bindgen: generated bindings are not arbitrary native reflection

Exported Rust structs map to JavaScript classes with distinct `T/&T/&mut T` handling. Public Copy fields generate accessors; non-Copy fields require cloning annotations or explicit accessors. Exported functions cannot declare their own generic parameters. These are supported explicit export contracts. [Exported Rust Types](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/exported-rust-types.html)

`#[wasm_bindgen] async fn` returns a Promise, and helpers convert Rust Futures and JavaScript Promises. TypeScript declarations are generated by default with per-item opt-out. Reuse the build-time multi-artifact workflow without changing GPUI's native execution target. [Promises and Futures](https://wasm-bindgen.github.io/wasm-bindgen/reference/js-promises-and-rust-futures.html), [TypeScript generation](https://wasm-bindgen.github.io/wasm-bindgen/reference/attributes/on-rust-exports/skip_typescript.html)

## 3. UniFFI: learn from lifetimes and generation IR without copying its thread model

UniFFI accepts Rust proc macros or UDL and generates both sides of the bridge. Fully supported official languages primarily include Kotlin, Swift, and Python; other bindings require checking the specific third-party backend. Do not describe it as an official ready-to-use Bun binding. [UniFFI guide](https://mozilla.github.io/uniffi-rs/latest/)

Objects use Arc and foreign-language proxies; interfaces require Send+Sync and do not expose exclusive `&mut self` borrows. This suits shared application objects, but adding Arc/Mutex cannot make foreground-only GPUI objects valid cross-thread UI resources. [Interfaces, Objects and Traits](https://mozilla.github.io/uniffi-rs/latest/types/interfaces.html)

UniFFI async does not mandate one Rust runtime: the foreign side drives RustFuture poll/complete/free, while runtime dependencies still need integration. Exporting async fn and deciding its thread/executor are separate problems. [Async overview](https://mozilla.github.io/uniffi-rs/latest/internals/async-overview.html)

## 4. Tauri 2 and Specta: abstractions closer to this bridge

Tauri command annotations expose parameters, results, errors, and async execution while injecting State/Window context. GPUI can adopt the rule that context is not a JavaScript argument. [Calling Rust](https://v2.tauri.app/develop/calling-rust/)

Tauri state has application-owned lifetime and shared mutation through interior mutability. It does not replace each mounted native component's Entity lifecycle. [State Management](https://v2.tauri.app/develop/state-management/)

Tauri channels explicitly serve ordered streams, while global events are unsuitable for large data. This supports distinct semantics for commands, component events, and persistent streams even over one byte transport. [Calling the Frontend / Channels](https://v2.tauri.app/develop/calling-frontend/)

tauri-specta v2 Builder uses one command collection for TypeScript export and invoke_handler; typed events can also register/export. At research time, maintainer source marked v2 beta and examples pinned `=2.0.0-rc.25`. `docs.rs/tauri-specta/latest` pointed to stable 1.0.2, so v1 examples must not be mixed in. Main-branch references establish source declarations, not release availability. [Source documentation](https://raw.githubusercontent.com/specta-rs/tauri-specta/main/src/lib.rs), [version mapping](https://github.com/specta-rs/tauri-specta), [stable latest](https://docs.rs/tauri-specta/latest/tauri_specta/)

Specta registers root types and collects dependencies. Its guide specified specta 2.0.0-rc.25, specta-serde 0.0.12, and specta-typescript 0.0.12 at the research date. It supplies type export, not a runtime bridge. Verify integers, Option/defaults, enums, input/output differences, and byte arrays against actual wire shapes. [Specta TypeScript](https://docs.rs/specta-typescript/latest/specta_typescript/)

Structural export metadata is not general Rust reflection. Proc macros receive and emit token streams; a third-party type name does not reveal every impl method or its domain semantics. Known-syntax generation is feasible; automatic understanding of every gpui-component builder and interaction is not. This follows from the macro input model. [Rust Reference: Procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html)

### Type-tool recommendation

Use **Specta v2 as the first prototype candidate for a unified internal type graph**, without promising immediate replacement of ts-rs. The ts-rs 12/separate FieldType findings come from the main code review, not an independent full inspection in this appendix. Nested DTOs, input/output wire shapes, and framework export fit Specta's graph plus separate serde interpretation better than simply replacing a TypeScript string generator. v2 was still an RC: pin it exactly and assess upgrade cost. [TypeScript usage](https://docs.rs/specta-typescript/latest/specta_typescript/), [serde-aware phased types](https://raw.githubusercontent.com/specta-rs/tauri-specta/main/src/lib.rs)

Prototype acceptance requires a shared Rust DTO containing nested structs/enums, Option/default, rename, bytes, and large integers whose JavaScript input type, Rust decoding/encoding, and JavaScript output type agree exactly. Component props and command DTOs use the same constrained type set. **TypeScript export is not runtime codec generation:** Specta owns type metadata, serde owns Rust serialization, and the framework still supplies matching JavaScript codecs and special subscription-handle fields. If ExtensionValue only supports scalar lists, extend/refactor its wire shape rather than claiming nested support through declarations alone.

If consistency cannot be established, retain ts-rs temporarily and land authoring/lifecycle simplification separately. Avoid changing type tooling, byte encoding, and lifecycles simultaneously without attributable acceptance. Keeping ts-rs limits migration scope; it does not justify permanently separate DTO and FieldType truths.

## 5. Independent alternative: ordinary Rust functions export components

This third option minimizes author learning cost through a deep interface: developers see parameters, events, and ordinary GPUI construction, while one module implements protocol and lifetime machinery. The following is a design sketch, not supported syntax.

```rust
#[native_component]
fn Checkbox(
    #[prop(default = false)] checked: bool,
    #[prop(default = false)] disabled: bool,
    on_change: Event<bool>,
    scope: &mut NativeScope,
) -> impl IntoElement {
    gpui_component::checkbox::Checkbox::new(scope.id())
        .checked(checked)
        .disabled(disabled)
        .on_click(move |next, _, _| on_change.emit(*next))
}
```

The macro generates prop types, event descriptions, dispatch/decoding, and JSX exports from the same function parameters. `scope` is injected in Rust. `Event<bool>` is a Rust event endpoint; the JavaScript function remains in its Solid owner. The framework must define `Event::emit` queue/error policy; the sketch does not imply transport errors can be ignored.

```tsx
import { Checkbox } from "native:app";
<Checkbox checked={enabled()} onChange={setEnabled} />;
```

`native:app` is a proposed build-plugin virtual module. Authors define functions in the application's Rust crate; one explicit export set drives host registration and codegen without application-specific schema/codegen/npm packages. Library authors put component adapters in one integration crate with one installation entrypoint for its exports.

### Reducing handwritten third-party builder mappings

Start with ordinary functions so Rust checks builder calls. Declare each prop's wire type once; rendering retains only actual mappings such as `.checked(checked)`. Decoding, subscription checks, sink lookup, emission wrappers, registry entries, and TypeScript wrappers should be generated or shared rather than repeated per function. The inspected [`render_checkbox`](../../crates/solid-gpui-gpui-component/src/lib.rs) combines those responsibilities and is a concrete automation target.

Add optional builder-mapping macros only after dozens of controls demonstrate identical patterns. Examples: `checked: bool = false => .checked`, `compact: bool = false => if_true(.compact)`, `size: ControlSize => .with_size(convert_size)`. One declaration generates parameters and builder chains. Keep this an integration-crate implementation tool, not another DSL every custom component author must learn.

Semantics still requiring explicit human decisions include:

- Call `.compact()` only for true, while `.disabled(false)` sets a value explicitly. Decide whether Option skips a setter or clears existing state.
- Define conversion from native `on_click` to `onChange(bool)`; not every GPUI event or Window/Context can cross the wire.
- Define children/slots, variant/size conversion, mount-only properties, and controlled versus uncontrolled ownership.
- Define Input/editor creation and updates, IME/selection protection, and subscription/task destruction.

Macros should not parse third-party source to guess semantics or expose every generic builder method. Export the finite component contract applications need.

### Stateful components and execution

Functions alone cannot retain InputState. The proposed `NativeScope` represents one mounted instance with persistent slots owning GPUI Entities, subscriptions, and tasks. Explicit stable keys such as `scope.entity("editor", ...)` create once until unmount. Entity creation/update/release stays on the GPUI foreground; UI Context does not enter workers or cross await borrows. This requires a real new runtime interface, not just proc macros suggesting lifetimes exist.

Do not write every prop back on every render. Input commands, edits, controlled values, and revisions need explicit ordering. Slot keys/types remain stable. Complex views may retain an existing Render Entity and let ordinary Rust manage internals. Avoid mandatory lifecycle boilerplate for every component and identity inferred from call order.

JavaScript owners manage reactive props/callbacks; native instances own Entities/tasks. Unmount and reload epochs invalidate old commands/events. Updating one instance invalidates its native view. Transport carries only values, instance/event handles, and versions. Do not adopt N-API GC timing or block for synchronous JavaScript callbacks.

### Package counts and real constraints

Application authors need their existing Rust crate plus one JavaScript runtime package; generated files/virtual modules are build artifacts, not new packages. Integration maintainers can put exports, native construction, initialization, and codegen metadata in one crate.

A public Rust facade can re-export proc macros, but macro implementation still requires a separate proc-macro crate. This Rust constraint prevents promising a single physical crate. Codegen can live behind a facade tooling/build feature or binary without a separate crate family per provider. [Rust Reference](https://doc.rust-lang.org/reference/procedural-macros.html)

The framework must first implement reliable NativeScope, metadata, and generation. A minimal prototype needs both a builder control and a truly stateful Input, checking retained Entity identity on prop updates, resource release on unmount, and old-epoch event rejection. Progress alone cannot establish complete simplified integration.
