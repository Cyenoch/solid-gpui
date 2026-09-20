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
    solid_gpui::run(app::native_module);
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

## Desktop host configuration

`solid_gpui::run_application(module_factory, runtime)` runs an application-owned runtime
with the framework's default profile; the host owns shutdown and joins the runtime
when the application quits.
`solid_gpui::run_application_with_profile(profile_factory, runtime)` runs an
application-owned profile and runtime without parsing host CLI arguments or
creating a runtime. Both reuse the same host runner as `run`: commit admission,
native events, protocol handlers, surface ownership, overlays, close handling,
and final runtime shutdown stay framework-owned.

Host entrypoints own platform startup. Pass a factory that constructs the profile
on the application thread, because a profile may contain non-`Send` GPUI state;
the factory and its captures must be `Send + 'static`. Do not create a custom
thread or add a platform-specific wrapper around the runner.

On Windows the runner reserves 16 MiB of stack for the application thread when
the calling thread has less, instead of relying on the executable's usual 1 MiB
initial-thread reservation; layout and paint recurse through the native element
tree. Other platforms run inline, so macOS keeps AppKit on the real main thread.
`SOLID_GPUI_APP_STACK_BYTES` overrides the Windows reservation for measurement
and `SOLID_GPUI_LOG=info` reports the actual application-thread stack; an invalid
budget fails startup. A reservation is not a promise that every deeply nested
application fits it. Keep the application's Cargo optimization profiles aligned
with [development guidance](hot-reload.md#application-build-configuration), and
follow [Windows host overflows its stack](troubleshooting.md#windows-host-overflows-its-stack)
when measuring a budget.

```rust
use solid_gpui::{gpui::*, components::host::ComponentHost};

let profile = || ComponentHost::new(vec![
    solid_gpui::components::native_module(),
    app::native_module(),
])
.with_window_options(|_, cx| {
    let mut options = gpui_component::TitleBar::window_options();
    options.window_bounds = Some(WindowBounds::Windowed(Bounds::centered(
        None, size(px(1100.), px(720.)), cx,
    )));
    options.window_min_size = Some(size(px(960.), px(640.)));
    options.titlebar.as_mut().unwrap().traffic_light_position =
        Some(point(px(16.), px(17.)));
    options
})
.with_performance_monitor(false);
solid_gpui::run_application_with_profile(profile, runtime);
```

The performance monitor is **off in every build** by default. An application
opts in with `with_performance_monitor(true)`; environment variables do not
override that policy.

Use `ComponentHost::with_initialize(|cx| { ... })` for application-specific
initialization after the component theme exists and before the first window
opens. This avoids reimplementing and forwarding the whole `HostProfile` trait
just to configure first-frame native state. A product choosing reduced motion,
for example, calls `solid_gpui::motion::set(MotionMode::Reduced, cx)` in that hook. Disabling
motion is an application policy, not a stack-overflow fix.

`useNative().setApplicationTheme` applies application colors, typography, and
control metrics at runtime; see
[Application theme overrides](gpui-components.md#application-theme-overrides).
Icons are registered before the runtime starts; see
[Add application icons](iconify.md#add-application-icons). The runnable host,
window configuration, and titlebar composition are in the
[desktop application example](../examples/desktop-app/README.md).

### Window options and titlebar

`ComponentHost::with_window_options` configures native window defaults before
renderer open-surface overrides; a custom profile implements
`HostProfile::window_options` directly. The callback runs for each opened
surface, and it receives the options the renderer requested, so retain them when
you only want to change some fields. Explicit open-surface title, kind,
resizability, and minimum size are applied after the callback.

`gpui_component::TitleBar::window_options()` supplies transparent macOS titlebar
options and `app_owns_titlebar_drag: true`. Its traffic-light position is the
top-left of the close button; 17 px centers a 14 px button in a 48 px titlebar.
The host does not add another Solid titlebar, so render exactly one:

```tsx
<TitleBar
  style={{
    height: 48,
    padding: 0,
    paddingLeft: 88,
    paddingRight: 16,
    backgroundColor: "#131217",
    borderWidth: 0,
    borderBottomWidth: 1,
    borderColor: "#2C2B33",
  }}
>
  <View style={{ flexDirection: "row", flexGrow: 1, alignItems: "center" }}>
    <Text>Application</Text>
    <Input placeholder="Search" style={{ width: 160 }} />
  </View>
</TitleBar>
```

Style refinements replace the TitleBar's native defaults, including its left
padding. Fullscreen adds no additional padding. The native TitleBar owns
blank-area dragging and calls macOS `titlebar_double_click`, respecting the
system preference. Controls that claim mouse-down do not initiate titlebar drag
or double-click zoom; core `Pressable` also claims that native default action.
An application that customizes the titlebar adds the pinned `gpui-component`
dependency alongside `solid-gpui`.

## Request cancellation and deadlines

Generated client methods accept a second `NativeCallOptions` argument:

```tsx
const controller = new AbortController();
const result = native.greet(
  { name: "Ada" },
  {
    signal: controller.signal,
    timeoutMs: 5_000,
  },
);
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

Native binding export rejects TypeScript `any` and `bigint` in DTO types,
including nested objects, arrays, unions, generic arguments, and template-literal
type interpolations. Documentation such as “The pending request, if any.”,
property names such as `any` or `bigint`, and string or template-literal text
do not trigger this check and remain in the generated bindings. Use bounded
Rust values or explicit strings for unsupported value types.

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

`NativeView::controlled()` declares `ControlledBinding::value_props`, a static
list of alternative controlled data properties sharing one edit-sequence event
and acknowledgement property. The generated binding subscribes internally when
any listed property is present, even without an application callback. Input uses
`["value", "content"]` so atomic token snapshots follow the same acknowledgement
path as plain text. This metadata is generated; rebuild every host catalog after
changing it rather than hand-editing generated TypeScript.

Scrollable native views may expose `NativeView::scroll_viewport()` by returning
a clone of an owned `native::ScrollViewport` (backed by `ScrollHandle` or
`ListState`). The host publishes this capability after native commit reconciliation;
`NativeChildren::content().scroll_viewport()` resolves exactly one direct child
without rendering rows or re-borrowing the root during commands. A decorator
retains `ScrollViewport::decorate()`'s lease; the view omits its own scrollbar
while `is_decorated()` is true. Drop the lease when content or orientation changes
or the decorator unmounts. Handles stay in Rust and do not enter generated DTOs.

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

Optional object members whose value is `undefined` are omitted recursively from
native component props and command DTOs. For example,
`{ items: [{ key: "mode", label: "Mode", description: undefined }] }` encodes
without `description`; application-specific sanitizing wrappers are unnecessary.
Undefined array elements and sparse arrays are rejected, not converted to
`null`. This does not relax QuickJS's separate
[captured-state contract](capture-state.md).

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
