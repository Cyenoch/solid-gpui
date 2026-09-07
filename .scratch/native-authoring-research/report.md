# Rust native component integration: a smaller interface with complete capabilities

Research dates: 2026-09-05 through 2026-09-06. This document preserves the original design exploration. Implementation subsequently completed and [ADR-0016](../../docs/adr/0016-rust-owned-native-modules.md) was adopted. For the current compilable API, see [Rust components and JavaScript calls](../../docs/rust-bridge.md); for runtime implementation and actual acceptance, see the [final audit](runtime-audit.md). The sketches below are not current API documentation.

Baseline: HEAD `4698b0e0b188c34b599ad0275717d17a329c79af` with substantial existing uncommitted changes. Findings describe inspected files, not necessarily that commit alone. Actual dependencies were `gpui-pre 0.3.3`, `gpui-component 0.6.0 @ 928c3eb776a3d733d9b771f7dea27a6a79242ced`, and `ts-rs 12.0.1`. The different Shell implementation in `references/gpui-component` does not establish this bridge's capabilities.

## Recommendation

Use **Rust source declarations, binding export from the same host, and native instance lifecycles**.

Applications define components and functions in one Rust module. One development command compiles the host, generates a local `#native` TypeScript module, and runs the application. JavaScript uses generated JSX and a typed client; the framework owns property codecs, identity, registration, events, commands, host configuration, and generation.

Ordinary components use functions; complex components use familiar GPUI `Entity<T>`/`Render`. Both share the native component contract and transport. gpui-component becomes an optional built-in integration whose common control adapters are implemented once by the framework.

Target one framework Rust dependency and one framework npm package. The application's Rust crate and locally generated file are not additional published packages. `solid-js`, Vite, and domain libraries remain dependencies as needed. Internally, retain Rust's required proc-macro crate and optional low-level Bun build crate.

The gain is more than merged directories or different macro syntax: authors no longer assemble protocols, and Input/editor/list gain actual native state ownership. No ready-made library was found that replaces the whole Solid/GPUI bridge while solving these problems. Existing systems offer individual mechanisms to reuse or learn from.

## 1. Sources of current complexity

| Inspected design                                                                                                   | Author burden or capability gap                                                                | Proposed change                                                                                    |
| ------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `declarations.rs` names props/adapter/decode/render/entry symbols, numeric IDs, versions, field IDs, and event IDs | Authors must understand generator internals and protocol identity                              | Derive internal symbols and deterministic identity from exported Rust items                        |
| Separate schema, provider renderer, host, and TypeScript package                                                   | One control change crosses several configurations and generators                               | Use the same app/module declaration for compilation, registration, and export                      |
| Components use `FieldType`; ordinary functions use serde and ts-rs                                                 | Separate type capabilities, contracts, and generators; component data limited to scalars/bytes | Unify exports while distinguishing components, service functions, and instance commands internally |
| `native-codegen.ts` hard-codes Workbench exporter and Gallery paths                                                | Example structure becomes framework machinery                                                  | Export actual host capabilities; configure the output location once per application                |
| `gpui-component-codegen.ts` mixes contract validation, TypeScript templates, and extensive shared runtime text     | Runtime copies and maintenance locations multiply                                              | Put shared runtime in the npm package and generate only application types and compact descriptors  |
| `ExtensionAdapter` offers only `validate/render`                                                                   | No mount/update/disposal contract for Entities, subscriptions, or tasks                        | Let the framework own isolated native instance state                                               |
| Render context lacks Window/App/Context; commit primarily updates root                                             | A TypeScript wrapper alone cannot integrate InputState                                         | Apply native instances on the window foreground                                                    |
| Provider host registers WorkbenchApi by default                                                                    | Gallery domain logic enters the generic host                                                   | Move application functions into Gallery's own module                                               |

Source: [component declarations](../../crates/solid-gpui-gpui-component-schema/src/declarations.rs), [adapter macros](../../crates/solid-gpui-bridge-schema/src/adapters.rs), [native rendering](../../crates/solid-gpui-gpui-component/src/lib.rs), [Extension runtime](../../crates/solid-gpui/src/renderer/extensions.rs), [function bridge](../../crates/solid-gpui-bridge/src/lib.rs), [function generator](../../scripts/native-codegen.ts), [component generator](../../scripts/gpui-component-codegen.ts), [provider host](../../crates/solid-gpui-gpui-component-host/src/lib.rs).

Preserve the existing system's useful properties: reactive Solid getters, atomic Snapshot/Patch validation, Surface/epoch isolation, listener generations, and bounded byte transport. The framework should hide this complexity without removing those capabilities.

### A case that disproves automatic setters as a complete solution

In the pinned gpui-component, `Input::new` requires `&Entity<InputState>`, and `InputState::new` requires Window and entity Context. Crucially, `set_value` resets selection, LSP state, scrolling, and undo history.

Automatically mapping props.value to state.set_value on every controlled acknowledgement would break native editing. Type reflection alone cannot distinguish acknowledgement of user input from intentional programmatic replacement.

Primary source: [Input construction](https://github.com/longbridge/gpui-component/blob/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/component/src/input/input.rs#L167), [set_value](https://github.com/longbridge/gpui-component/blob/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/base/src/input/base/state.rs#L834), [InputState construction](https://github.com/longbridge/gpui-component/blob/928c3eb776a3d733d9b771f7dea27a6a79242ced/crates/base/src/input/base/state.rs#L4978). These findings came from the actual Cargo checkout rather than webpage summaries.

The repository's [input.rs](../../crates/solid-gpui/src/renderer/input.rs) already has `edit_seq/ack_edit_seq` and marked-text protection; extract and reuse that controlled synchronization model.

## 2. Lessons from existing systems

See [binding-sources.md](binding-sources.md) for primary sources and version caveats.

| System                      | Useful mechanism                                                                           | Applicability                                                                                                                 |
| --------------------------- | ------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| napi-rs                     | Rust annotations derive invocation wrappers, TypeScript types, and class interfaces        | Learn from the author experience; Node-API/env/async execution does not own GPUI UI threads, Surfaces, or atomic tree commits |
| Tauri + Specta/tauri-specta | One Rust registration generates dispatch and typed clients through a structural type graph | Closest to ordinary command authoring; reuse the pattern without treating Tauri runtime as a GPUI component runtime           |
| UniFFI                      | Rust export metadata, object lifetimes, and cross-language binding generation              | Reuse unified-contract ideas; its thread constraints and language backends do not model GPUI Entity semantics                 |
| wasm-bindgen                | Rust types/annotations generate JavaScript glue and hide ABI details                       | Designed for Wasm, not the existing native GPUI host                                                                          |
| React Native Fabric         | Native component props, events, commands, and persistent Host Views                        | Learn from capability separation and instances without adopting TypeScript-first specs, platform glue, or JSI pointers        |
| flutter_rust_bridge         | Opaque Rust capabilities and automated bindings                                            | Reuse typed-handle ideas; handles here require Surface/epoch ownership and cannot carry raw pointers across processes         |

Fabric distinguishes rendering, commit, and mount while retaining native-only state. This supports including instance management in component bindings, but does not prove this repository's implementation or performance. [Render, Commit, Mount](https://reactnative.dev/architecture/render-pipeline)

Fabric generates commands for specific native views, supporting explicit instance command targets. [Native Commands](https://reactnative.dev/docs/the-new-architecture/fabric-component-native-commands)

flutter_rust_bridge's automatic opaque types address Dart/Rust smart pointers. They suggest controlled object proxies, not arbitrary cross-thread GPUI access. [RustAutoOpaque](https://cjycode.com/flutter_rust_bridge/guides/types/arbitrary/rust-auto-opaque/overview)

## 3. Three independent designs

| Design                                   | Interface                                                   | Depth / locality                                                                                          | Tradeoff                                                                                        |
| ---------------------------------------- | ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| A: Automatic application-module exports  | app/module macro, component/function annotations, `#native` | One Rust declaration defines actual host and JavaScript behavior without registering every function twice | Multi-file/cross-crate exports need explicit composition, not fictional automatic discovery     |
| B: Native entities first                 | Typed props, mount/update, GPUI Render, commands            | Complete editor/list/input support with state and behavior together                                       | Requiring lifecycle boilerplate for every Progress would add complexity again                   |
| C: Automatic third-party builder mapping | Restricted mapping declarations generate setter/event glue  | Convenient for integrating ordinary library controls in bulk                                              | Signatures cannot infer controlled state, ownership, synchronous callbacks, or thread semantics |

Use A as the public entrypoint and B as its stateful capability. Reserve C for mechanical framework-internal mappings with established semantics. Do not first invent a DSL covering every GPUI builder or require descriptor/registry/decoder interfaces for every control.

The source appendix also explores keyed `NativeScope` state. Do not make render-time `scope.entity(...)` the default stateful interface: it can obscure creation/update side effects and commit timing. Prefer explicit mount/update with native Render. Any later functional convenience API must use the same instance lifecycle instead of introducing another state system.

## 4. Intended author experience

All examples below are target API sketches. The annotations and `#native` were not compilable in the investigated repository.

### Ordinary components and functions

```rust
#[solid_gpui::app(components = "gpui-component")]
mod app {
    use solid_gpui::prelude::*;

    #[component]
    pub fn greeting(
        name: String,
        on_press: Event<()>,
        cx: &mut ElementContext,
    ) -> impl IntoElement {
        gpui_component::button::Button::new(cx.id())
            .label(format!("Hello, {name}"))
            .on_click(move |_, _, _| { on_press.emit(()); })
    }

    #[command]
    pub fn greet(name: String) -> String {
        format!("Hello, {name}")
    }
}
```

```tsx
import { Greeting, useNative } from "#native";

export function Page() {
  const native = useNative();
  return <Greeting name="Ada" onPress={async () => console.log(await native.greet("Ada"))} />;
}
```

Conventions: component names default to PascalCase, properties/functions to camelCase; overrides are exceptional. Ordinary parameters become props/requests, `Event<T>` becomes a JavaScript callback, and the framework injects context. Callbacks and contexts are not serialized. Simple values need no separate DTO; nested structs/enums use one `NativeType` derive. Annotate optionality, defaults, ranges, and mount-only behavior only when meaningful.

`useNative()` captures the Surface client during Solid setup; later async callbacks use that bound client instead of guessing a current window globally. Ownerless code uses an explicit root client. Disposed Surfaces and retired epochs reject calls clearly.

A module macro gathers its actual contained exports without global proc-macro mutation or scanning all dependency source. Cross-file modules provide generated descriptors for one application composition step; cross-crate libraries install a module explicitly once. Duplicate export names fail at build time. Respect actual cfg/features so disabled components never enter TypeScript exports.

### Stateful components

Stateful components retain ordinary GPUI code while declaring initial props, updates, and methods. This sketch omits application logic to show responsibilities:

```rust
#[component]
impl Editor {
    fn mount(props: &EditorProps, events: Events<EditorEvent>,
             window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Create input/document Entities once and own their subscriptions/tasks.
    }

    fn update(&mut self, props: &EditorProps,
              window: &mut Window, cx: &mut Context<Self>) {
        // Apply changed domain properties using controlled-edit synchronization.
    }

    #[command]
    fn focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Focus the actual input Entity on the foreground.
    }
}

impl Render for Editor {
    // Render persistent Entities without recreating state, subscriptions, or tasks.
}
```

JavaScript receives `<Editor ... ref={...} />`, `EditorRef`, and `await editor.focus()`. A ref is a lifetime-bound remote capability, not a GPUI pointer. Complex components define their update semantics; the framework generates serialization, registration, lifecycle scheduling, events, and method proxies.

## 5. Required framework internals

### One export model, three execution targets

One internal contract describes types, props/defaults/constraints, events, slots, commands, identity, and limits. Generate Rust glue and TypeScript facades from it while retaining distinct execution rules:

- Component creation/updates/instance methods run on the window foreground with GPUI Context access.
- Ordinary computation runs in the background without captured window Entities and returns JavaScript Promises.
- Async services need an explicitly supported executor and cancellation policy. `async fn` does not provide a Tokio reactor automatically; configure one runtime for services that need it, not one implicit runtime per command.

Investigate Specta's structural type graph first; the appendix records options and version risks. TypeScript strings are not reversible schemas, and type derives are not runtime decoders. Whatever the internal library, authors see one NativeType interface. Verify serde rename, tagged enums, Option/default, and input/output differences against actual transport.

Retain the Bebop envelope and bounded bytes. Scalars can keep compact ExtensionField encoding; richer nested DTOs need explicit generated codecs and depth/size limits, not unbounded JSON/any. Determine complex DTO wire changes in a prototype without casually replacing an already-measured protocol. Keep large document/table models in Rust and send semantic changes rather than whole models on every keystroke.

### Identity and generated artifacts

Stop manually maintaining numeric IDs. Deterministically assign compact IDs from module namespaces and canonical export names; include mappings, types, defaults, constraints, events, slots, and commands in a complete digest. Allocation must not depend on source-read or linker ordering. Reject mismatched artifacts without permissive legacy compatibility.

Remove manual tombstone and entry-version bookkeeping while preserving machine-checked contract identity. Any temporarily retained wire fields are generator-owned, outside author APIs. Removing wire fields requires explicit protocol and ADR updates, not merely deleting JavaScript properties. Opaque custom-validation semantics need explicit contract policy; arbitrary Rust function-body hashes are not stable type contracts.

### Native instances and atomic commits

The host identifies instances by `(surface, epoch, node, incarnation, component contract)` and owns Entities, subscriptions, tasks, and typed props.

1. Validate all candidate Commit Batch topology, contracts, props, and slots into typed updates without changing live Entities.
2. Publish and apply valid updates on the window foreground. Mount only new instances and update only changed data; moves and sibling changes do not remount.
3. Render from typed props without repeating wire decoding or registry construction every frame.
4. On removal, type replacement, epoch replacement, or window closure, revoke events/refs first, then release instances, subscriptions, and tasks through RAII.

The host commit entrypoint also needs a foreground update path with Window access. Enlarging render context alone does not provide this.

Treat validated mount/update as infallible UI transformations; external I/O errors become component state or typed errors. Do not promise rollback of arbitrary Rust panics, file writes, or application side effects. Future fallible mounting would require candidate-instance staging and cleanup, not silently weakened batch validation.

### Event and ref revocation

`ExtensionEventSink` fixes node/listener identity but reads current epoch/revision from shared Cells at emission. Retaining it unchanged in subscriptions could label old-instance callbacks with a new epoch.

Emitters must capture immutable surface/epoch/incarnation and support revocation. Valid commits update live listener bindings; event creation freezes revision/listener generation so queued events cannot be relabeled. Existing current/previous rules handle older events. Post-unmount callbacks cannot impersonate new instances. Events produced during commit require explicit post-commit ordering.

Instance commands carry the same lifetime identity and reject before mount, after unmount, across epochs, or across Surfaces. Commands cannot overtake pending creation/prop commits: wait for the relevant revision, then execute in foreground order. Revalidate target lifetime when background results return.

### Input, children, and lists

- Input: acknowledging the same value preserves selection/IME/undo; sequences and acknowledgements stop old JavaScript values overwriting newer edits. Programmatic reset is explicit.
- Slots: preserve Host Node identity and Solid Owner Tree. Build fresh AnyElements per render, without retaining previous-frame elements or JavaScript closures. Child changes invalidate retained parents correctly.
- Ordinary children may initially reuse the ordered tree. Named slots require real schema/runtime work; `Vec<AnyElement>` alone does not implement them.
- Lists: retain native ListState and create JavaScript row owners for the committed visible range. GPUI's synchronous render-item callback cannot wait for cross-process JavaScript.
- Install high-frequency handlers only for subscribers. Verify unsubscribe, listener replacement, and Patches, not only initial Snapshots.

## 6. Simplifying the actual build workflow

```text
One dev/build command
  → compile the host containing the app/module contract
  → run Host --export-native before GPUI/Bun/application initialization
  → atomically write .generated/native.ts and types/manifest
  → resolve #native to that file for JSX transform/typecheck/bundling
  → run JavaScript with the same host
```

Authors write no separate exporter binary and do not manually sequence three codegen commands. TypeScript editors read real generated files rather than modules understood only by Vite. Avoid rewriting unchanged output and triggering unnecessary reloads.

Rejected approaches: proc macros writing TypeScript directly, build.rs recursively running its own crate through Cargo, inferring Rust decoders from d.ts, or discovering host/JavaScript contract mismatches only after startup.

Rust proc macros require a separate proc-macro crate and operate on token streams; they cannot reflect on every dependency type and method. [Rust Reference](https://doc.rust-lang.org/reference/procedural-macros.html)

build.rs runs before its package compiles, so generating a crate from that same crate's compiled registry creates an invalid build order. [Cargo Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)

State the cost: first export compiles the complete host and is heavier than the lightweight schema crate. Incremental compilation and binding caches can reduce subsequent cost without an unmeasured speed claim. Frontend-only development may use pre-generated contracts matching the host; CI still checks the actual host. Cache keys include target, features, build inputs, and generator version.

A cross-compiled host may not run on the build machine. Prefer native CI per platform for bindings/manifests. Cross-build release workflows need a target runner or verifiable target contract artifact; a macOS export cannot automatically represent Windows cfg. This is a real cost of same-host export.

Production bundles may be application resources. If embedding them in the binary is required, use explicit second-stage packaging. The first-stage host must not depend on the JavaScript bundle generated from its own bindings, which would create a cycle. Recheck the contract in stage two.

TSX-only HMR retains the existing new-epoch mechanism. Rust edits rebuild, re-export, and restart the host; failures surface errors rather than mixing an old host with new bindings. An unchanged digest does not make Rust implementation hot replacement possible.

## 7. Reducing package count

| Existing crate/package             | Proposed destination                                                          |
| ---------------------------------- | ----------------------------------------------------------------------------- |
| `solid-gpui`                       | Sole public framework Rust crate with internal responsibility modules         |
| `solid-gpui-host`                  | `solid-gpui::host` and standard startup                                       |
| `solid-gpui-bridge`                | `solid-gpui::native`                                                          |
| `solid-gpui-bridge-schema`         | Contract runtime in native; syntax handling in internal macros                |
| `solid-gpui-gpui-component-schema` | Remove separate schema crate; export metadata from actual components          |
| `solid-gpui-gpui-component`        | Optional main-crate feature/module                                            |
| `solid-gpui-gpui-component-host`   | Remove separate host; configure integration Root/overlays/init                |
| `solid-gpui-workbench-api`         | Gallery's own Rust module                                                     |
| `solid-gpui-bun`                   | Adapter in the main crate; necessary low-level FFI/build in internal bun-sys  |
| New `solid-gpui-macros`            | Separate proc-macro crate re-exported by the main crate                       |
| npm core, gpui-component, vite     | One framework npm package exposing runtime, components, and vite/dev subpaths |
| `#native`                          | Application-generated local file without package.json or publication          |

The existing solid-gpui-bun→solid-gpui dependency would cycle if the main crate directly depended on it. Separate low-level FFI from RuntimeAdapter before reversing dependency direction; a re-export alone cannot solve this.

Applications without gpui-component should not compile that optional dependency. Enabling integration installs the actual Root, overlays, theme, and keybindings. Integrations needing different window roots require an explicit unique root policy, not assumed automatic composition. Configure it once inside the framework without making ordinary authors implement HostProfile.

Runtime entrypoints must not statically import build tools such as `/vite`. Manage Vite as an optional build-time peer. Multi-window theme/provider scope still depends on actual host support and cannot be inferred from npm exports.

`gpui-iconify` and `gpui-performance` have independent GPUI uses and need not be merged merely to reduce a count. Reduce packages applications must manage without putting the entire workspace into one file.

## 8. What to remove and retain

Automate and remove author-managed numeric identities, decode/render/adapter symbol lists, duplicate types, registration matches, separate schemas/exporters, manual TypeScript generation entrypoints, and copied runtime templates.

Retain or centralize actual GPUI rendering, third-party event conversion, native ownership, controlled input policy, domain/range constraints, and host-root integration. Authors do not repeat mappings for controls already integrated by the framework.

Do not generate every third-party public method indiscriminately. Builder children, synchronous render callbacks, input setters, asynchronous work, and window commands have different execution semantics. Automate simple fields with explicit meaning without turning necessary contracts into hidden runtime failures.

## 9. Minimal validation capable of disproving the design

Research does not establish a working framework. Implement these vertical examples before generating the whole library:

1. **Progress + Button:** Add only component declarations/source, without handwritten schema/TypeScript/registry changes. Signal updates and listener replacement/removal exercise Snapshot/Patch.
2. **Real gpui-component Input:** Mount once; preserve selection/IME/undo across equal-value acknowledgements, delayed acknowledgements during typing, and moves. Verify explicit reset semantics and real-window input/focus.
3. **Instance disposal races:** Exercise deletion, HMR, node-ID reuse, late events/background results/ref commands. Old instances never invoke new callbacks; Promises settle and tasks/subscriptions release.
4. **Atomic validation:** Mix valid nodes and invalid props/slots in one batch without partial publication, mounting, or old-state mutation.
5. **Native lists/child content:** Exercise visible ranges, filtering/reorder/resize, and slot updates. Construction follows visible ranges without synchronous JavaScript callbacks or rebuilding the whole list.
6. **Clean generation and release:** One command succeeds without generated files; actual host types and capabilities match. Cover cfg/features, JavaScript HMR, process/embedded modes, and two-stage production packaging.

Record independent author edit locations, direct dependencies, clean/incremental build time, decode/mount counts per prop update, and input-to-present behavior. Claim performance gains only from matched actual workloads; fast macro expansion or codec unit tests do not establish native experience.

Only `bun run task gpui-component-codegen-check` ran during this research, exiting 0. It establishes consistency of the old artifacts, not viability of this proposal. No product code or unnecessary tests were added for the research.

## 10. Migration order and existing decisions

First replace both authoring paths with application modules and actual-host export, migrating the four existing controls and Workbench. Then establish Input instance lifecycles and emitter/ref revocation, verify lists/slots and complex DTOs, and remove old schema/host/bridge packages and compatibility entrypoints while consolidating npm packages. These are implementation stages, not permanent parallel interfaces.

Do not wait for all controls before evaluating the design. Controlled Input semantics, foreground instance commits, clean builds, and instance revocation are prerequisites.

- Consistent with ADR-0001 atomic Commit Batches and ADR-0012 Host-Owned Input Models.
- Consistent with ADR-0015 Rust restarts and new-epoch cleanup; native Entity HMR retention is not promised.
- Retain ADR-0014 Bebop and strict validation. Removing entry versions, changing Extension values, or adding slots/instance commands requires explicit canonical schema, golden-vector, and decision updates.
- This proposal does not modify accepted ADRs. Record the final decision after implementation and acceptance.

Key source and existing documentation: [CONTEXT](../../CONTEXT.md), [Rust bridge](../../docs/rust-bridge.md), [ADR-0001](../../docs/adr/0001-batch-renderer-commits-into-gpui.md), [ADR-0012](../../docs/adr/0012-host-owned-input-models.md), [ADR-0014](../../docs/adr/0014-bebop-v5-generated-wire-protocol.md), [ADR-0015](../../docs/adr/0015-vite-bun-native-hot-reload.md).
