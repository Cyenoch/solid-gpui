# Rust components and JavaScript calls

Use one Rust dependency, `solid-gpui`, and one npm dependency,
`@solid-gpui/core`. Rust declarations define the contract; the host exports
components, events, instance refs, and Promise clients. Applications do not
need handwritten field IDs, JSON codecs, TypeScript interfaces, or separate
schema and code-generation packages.

## Minimal application

Application `Cargo.toml`:

```toml
[dependencies]
solid-gpui = { path = "path/to/solid-gpui/crates/solid-gpui", features = ["gpui-component"] }
```

`src/main.rs`:

```rust
use solid_gpui::native_module;

#[native_module(name = "my-app")]
mod app {
    #[command]
    fn greet(name: String) -> Result<String, String> {
        if name.trim().is_empty() {
            return Err("Name is required".into());
        }
        Ok(format!("Hello, {name}!"))
    }
}

fn main() {
    solid_gpui::run(app::native_module());
}
```

An `#[command] async fn` also generates a Promise client. The entire function
runs on the host's shared Tokio runtime, including timer and I/O drivers, so
it can use Tokio networking, timers, and ecosystem libraries directly.
Synchronous commands run in Tokio's blocking pool. Ordinary GPUI work belongs
in the foreground lifecycle or instance methods of a NativeView.

The runtime is created lazily for the host's module collection; exporting
TypeScript starts no threads. Each Surface allows at most 32 in-flight requests,
and the module collection allows at most 128. Excess requests fail immediately.
Surface closure, epoch replacement, and Root disposal cancel managed asynchronous
calls. Tokio panics become request errors. A synchronous blocking function
that has already started cannot be forcibly interrupted and occupies capacity
until it finishes. Long tasks that require cancellation must cooperate;
detached child tasks are not automatically cancelled with their parent call.

## Request cancellation and deadlines

Generated client methods accept a second `NativeCallOptions` argument:

```tsx
const controller = new AbortController();
const result = native.greet({ name: "Ada" }, {
  signal: controller.signal,
  timeoutMs: 5_000,
});
controller.abort();
await result;
```

Cancellation rejects with the signal's reason; a deadline rejects with a
`TimeoutError`. An already-aborted signal sends no request. A deadline is a
non-negative integer up to 2,147,483,647 milliseconds. Calls without a request
DTO accept `undefined` as their first argument. Both successful completion and
cancellation release the JavaScript listener and timer. Cancellation targets
one request in its originating Surface and epoch; late results are discarded.

An optional Rust `NativeCallContext` parameter is injected by the command macro
and does not enter the generated request DTO:

```rust
#[command]
fn scan(paths: Vec<String>, context: solid_gpui::native::NativeCallContext)
    -> Result<u32, String>
{
    let mut completed = 0;
    for path in paths {
        context.check_cancelled()?;
        std::fs::metadata(path).map_err(|error| error.to_string())?;
        completed += 1;
    }
    Ok(completed)
}
```

Blocking work should check between bounded units of work. Its admission slot
remains occupied until it actually returns. `context.cancelled().await` can
coordinate app-owned child work; ordinary async command futures are also
aborted by the executor. Cancellation does not undo completed side effects.
Direct `CommandDefinition` registrations receive `(request, context)`.

The host handles `--export-native` before starting its window or runtime:

```sh
cargo run --manifest-path native/Cargo.toml -- --export-native > src/native.ts
```

Capture the current Surface's client during Solid component initialization:

```tsx
import { Button } from "@solid-gpui/core/components";
import { useNative } from "./native";

function Page() {
  const native = useNative();
  return (
    <Button
      label="Greet"
      onPress={async () => {
        console.error(await native.greet({ name: "Ada" }));
      }}
    />
  );
}
```

Use `createClient(root)` outside a component. Do not call `useNative()` inside
an asynchronous callback: it must capture the Solid owner during component
initialization. Diagnostics go to stderr; stdout carries protocol frames.

## Custom Rust components

Define a function inside the same `#[native_module]`:

```rust
#[component(children = false)]
fn badge(
    label: String,
    #[prop(default)] highlighted: bool,
    cx: &mut solid_gpui::native::ElementContext,
) -> impl solid_gpui::gpui::IntoElement {
    use solid_gpui::gpui::{div, rgb, InteractiveElement, ParentElement, Styled};
    div().id(cx.id())
        .text_color(if highlighted { rgb(0x60a5fa) } else { rgb(0xffffff) })
        .child(label)
}
```

The generated component supports `<Badge label="Ready" highlighted />`.
Rust parameters become camelCase properties. `#[prop(default)]` uses Rust's
`Default`; `#[prop(default = expression)]` supplies an explicit default.
Declare custom DTOs with `#[native_type]`. Nested structs, `Vec`, `Option`,
symmetric serde rename/tag/content attributes, and enums are supported.
Independent TypeScript overrides, flattening, skipped fields, and asymmetric
input/output serialization rules are rejected.

An `on_press: Event<()>` parameter generates `onPress?: () => void`;
`on_change: Event<MyChange>` generates a typed callback. `event.emit(value)`
sends a Native Event and skips encoding when no listener is subscribed.
Install high-frequency GPUI handlers only when `event.is_subscribed()`.
Events do not provide synchronous JavaScript return values.

Components allow JavaScript children by default. Place them in native layout
with `cx.children()`. Declare `children = false` for a component that does not
consume children so the host rejects invalid nesting before publishing its
tree. Interactive descendants need distinct GPUI IDs derived from `cx.id()`;
do not use a global fixed ID across instances.

## Stateful components and instance methods

Annotate `impl NativeView for Editor` with `#[component]`. Define `Props` and
`Event`, implement `mount` and `update`, and implement GPUI `Render`. The host
creates an Entity when the node is first committed and retains it across
updates. Removal, type replacement, or epoch replacement releases the Entity
and its owned subscriptions and Tasks. Declare instance methods that require
a foreground Window/Context with `ViewCommand::new` to generate typed refs.
Use module-level `#[command]` for asynchronous application operations.

`NativeView::Event` can be a tagged enum, expressing multiple changes through
one typed event stream. Rust owns the delegates and transient state of complex
editors, tree tables, and virtualized lists; each internal state change does not
need a corresponding JavaScript property. Use batch DTOs or Rust-owned models
for large datasets and high-frequency input to avoid per-item round trips.

Import built-in components from the main package's subpath:

```tsx
import { Button, Input, type InputRef } from "@solid-gpui/core/components";
import { createSignal } from "@solid-gpui/core/runtime";

function Editor() {
  const [value, setValue] = createSignal("");
  let editor: InputRef | undefined;
  return (
    <>
      <Input
        value={value()}
        onChange={(event) => setValue(event.value)}
        ref={(ref) => {
          editor = ref;
        }}
      />
      <Button label="Focus" onPress={() => editor?.focus()} />
    </>
  );
}
```

`Input` uses the real gpui-component InputState. Returning the same value
preserves selection, scrolling, and undo history. A controlled `onChange`
should update `value` synchronously, with asynchronous validation following
the update. If an asynchronous result determines whether to replace text,
use `defaultValue` to keep ownership in Rust and explicitly call
`ref.replaceValue(text)`. Following upstream replacement semantics, that
operation ends composition and updates undo history. `defaultValue` applies
only at mount.

Method calls wait for the current Solid transaction to produce a complete
Patch. Unmount sets the ref to `undefined` and rejects pending calls. Calls
across Surfaces, stale epochs, mismatched contract digests, and invalid targets
produce explicit errors.

## Data limits and generation

Each DTO is limited to 1 MiB and must contain plain JSON data. Cycles,
non-finite numbers, integers outside the safe range, BigInt, class instances,
unknown fields, and invalid enum values are rejected. JavaScript callbacks,
GPUI Entities, and thread objects do not cross runtime boundaries. Put domain
constraints in the Rust DTO's `Deserialize` implementation, such as the
built-in Percentage type's 0–100 validation.

Run `bun run task native-codegen` to generate core component and website
bindings, and `bun run task native-codegen-check` to detect drift. Commit the
generated files for editor support and validation; do not edit them by hand.
Vite's `native` option invokes the same host exporter and provides the `#native`
alias. The separate `@solid-gpui/vite` package builds the exact Cargo artifact
that exports the bindings. Rust-owned Vite development exports from the running
host. Direct Bun JS can import those generated bindings without a bundler;
the Native Contract is unchanged. Rust changes require rebuilding and restarting
the host. See [Vite integration](vite.md).

For a runnable example, `examples/website/native/src/lib.rs` declares both
`BuildBadge` and `analyze_workspace`; website TSX uses the generated component
and Promise client. See [ADR-0016](adr/0016-rust-owned-native-modules.md) for the
design rationale.
