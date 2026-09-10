# Shell plugin-panel implementation recipe

Date: 2026-09-10. Source baseline:
`05433bd8e9e75af2f3aa508141b78ba21bfa6261`.
This is an implementation recipe derived from source, not implemented or compiled
by this research task. It supplements [the adoption report](research.md).

## Proposed public contract and owners

Add an optional native module exposing `ShellPanel` with a host-configured
`pluginId`, a bounded plain-data `context`, and typed `onMessage`/`onStatus`
events. Provide `refresh` and `getState` ref commands. Rust configures plugin
directories, data directories, allowed host functions, and policy. JSX selects
an installed plugin ID; it does not grant filesystem paths or runtime authority.
Initially enforce one mounted instance per plugin ID in the application because
upstream `PluginManager` keys loaded plugins by that ID.

The application owns one `Rc<ShellRuntime>` and plugin catalog/manager. Each
mounted `ShellPanel: NativeView` owns its `Entity<ScriptView>` clone, context DTO,
event endpoint, and plugin mount lease. Its `unmount` removes owner-specific
overlays, drops its view clone, and unloads its plugin. A replacement `pluginId`
performs the same teardown before loading the new identity; ordinary context
updates preserve native state. The native view cannot be held strongly by one
of its own host-module closures.

Use `ModuleDefinition::new(..., vec![ComponentDefinition::view::<ShellPanel>(...)], ...)`
and the existing `NativeModules` generator. `NativeView::mount` cannot return a
`Result`; malformed props should fail pure validation, while loading failures
become an explicit failed status and visible error with structured details.
Do not evaluate JS during pure commit validation. Prepare local plugin files and
dependency resolution before the foreground mount; keep the initial recipe
free of Git dependencies so mounting does not fetch source.
[NativeView, Event and mount/release hooks](../../crates/solid-gpui/src/native/component.rs),
[module registration/code generation](../../crates/solid-gpui/src/native/module.rs).

## Existing methods to use

Initialize Shell once after the host's Component/Base initialization, then create
one default Shell runtime with the actual component catalog:

```rust
gpui_shell::init(cx);
let runtime = gpui_shell::ShellRuntime::new_with_components(
    cx,
    gpui_component_shell::components()?,
)?;
let mut manager = gpui_shell::plugin::PluginManager::new(plugin_directories)
    .with_data_home(plugin_data_directory);
for manifest in manager.discover() {
    register_manifest_or_report(manifest);
}
```

`register_manifest_or_report` denotes application code. The other calls exist
upstream. `discover()` returns individual results and does not evaluate plugins.
The default runtime is needed because Shell's overlay-view constructor currently
uses `ShellRuntime::global(cx)`; `new_isolated_with_components` alone does not
install that registration. The global runtime slot is weak and refuses a second
live default, so the application must retain the `Rc`.
[`new_with_components`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/engine/quickjs/mod.rs#L1503),
[`discover`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/plugin.rs#L1416),
[overlay constructor](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/engine/quickjs/overlay.rs#L365).

The current mounting calls are:

```rust
manager.load(&runtime, plugin_id, authorize_manifest, window, cx)?;
let view = manager.plugin(plugin_id).expect("loaded plugin").view().clone();
// Render the retained view with .child(view.clone()).
```

`authorize_manifest` currently returns only `bool`. This is sufficient to approve
the exact manifest capability request, but **insufficient to attach host methods**;
the required change is described next. `Plugin::view()` returns a real retained
GPUI entity, so it can be the rendered child of the native adapter without
serializing Shell's private element arena through Solid's protocol.
[`PluginManager::load`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/plugin.rs#L1472),
[`Plugin::view`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/plugin.rs#L1179).

When native application data changes, update the adapter's plain-data context
and call `view.update(cx, |view, cx| view.refresh(cx))`. A bare `cx.notify()`
only repaints the previous script snapshot. A host function must enqueue its
event or request a later refresh, never synchronously reenter either JS engine.
[`ScriptView::refresh`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/view.rs#L159),
[`HostModule::function`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/host_modules.rs#L484).

## Three required embedding seams

### 1. Host-built policy during plugin loading

`load_plugin` constructs `Policy::default()` with manifest capabilities and
storage. `Policy::default()` is a fresh empty policy, not the thread-local
default. Therefore `export_module()` cannot populate the modules of a plugin
loaded by `PluginManager`. The useful public builder
`Policy::with_host_module` currently has no public path into this loader.
[`load_plugin`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/plugin.rs#L1192),
[`Policy::new`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/policy.rs#L112),
[`with_host_module`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/policy.rs#L171).

Refactor the loader's authorization seam to produce a host-built `Policy` (or a
typed grant containing capabilities and host modules), with errors propagated
before JS evaluation. Pass the inert manifest and resolved plugin/data paths to
the host callback. Keep application identity and storage placement loader-owned;
do not overwrite host-selected capabilities with the manifest afterwards.
Thread that policy into the existing private `load_view_with_policy`, retaining
its failure cancellation. This is a targeted source change, not an existing API
to pretend is available. Avoid temporarily swapping global policies as an
embedding workaround.
[`load_view_with_policy`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/plugin.rs#L1235).

The smallest useful grant is `Capabilities::new()` plus one module such as
`"solid-host"` containing `context(): PanelContext` and
`emit(message: PanelMessage): void`. Bind `emit` to the mounted
`Event<PanelMessage>` and `context` to an adapter-owned DTO; validate payloads at
that boundary. `HostModule::declarations` checks exported names, not complete
signature correctness. Long-running host operations belong in
`async_function`'s `Send` future, with plain copied data.
[`HostModule` builders](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/host_modules.rs#L445),
[`async_function`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/host_modules.rs#L547).

Use `"capabilities": {"storage": false}` in a minimal plugin manifest.
Omitting the manifest capability block grants storage by default. Source/module
loading from the configured plugin root is distinct from granting runtime `fs`
access. Optional network/filesystem/clipboard grants should be constructed by
the Rust host only when the application needs them.
[Manifest capability behavior and tests](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/plugin.rs#L2514).

### 2. A real window overlay host with plugin ownership

Solid's window root is `gpui_component::Root`. Shell's
`ShellRoot::update` and `tooltip_overlay` use
`window.root::<ShellRoot>()`; placing a `ShellRoot` inside a panel does not fix
that lookup. Shell overlay APIs explicitly fail when the root is wrong.
[Local window host](../../crates/solid-gpui/src/components/host.rs),
[`ShellRoot::update` and tooltip lookup](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/root.rs#L260),
[explicit overlay failure](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/engine/quickjs/overlay.rs#L339).

Refactor Shell overlay lookup to an explicitly installed per-window host with a
weak lifecycle registration. Keep Component `Root` outermost and install one
Shell overlay host for the window, not one per panel. Compose layers once and
retain the existing Component overlay owner/session rules. Alternatively route
Shell overlay operations through a host-provided overlay implementation using
those existing rules; either approach needs actual source support.

Lookup is only half the change: `ActiveDialog`, `ActiveSheet`, and toast IDs
currently lack plugin owner identity, and `close_dialog` pops the global top
dialog. Add owner identity from the calling policy, enforce ownership for
mutation, and release only that owner's overlays on plugin unload. Preserve
focus restoration and other plugins' active UI. Apply the same ownership to
Component Shell's window effects before advertising multi-plugin modal support.
Also share the host FPS visibility policy rather than creating another HUD.
[Shell overlay storage and operations](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/root.rs#L133),
[Component Shell window effects](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/component-shell/src/shell/window_effects/mod.rs).

### 3. Keep application theme ownership in Solid's host

`ScriptView::render` synchronizes Base theme tokens, so plugins can read the
application's existing theme. However Shell's `set_theme` directly mutates the
global Base theme and refreshes all windows, without a capability check; it can
diverge from the host's Component theme. Introduce an explicit embedding policy
that denies script theme mutation or delegates it to the host's authoritative
appearance operation. Do not initialize a separate per-panel theme or silently
allow one plugin to restyle unrelated windows.
[`ScriptView::render`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/view.rs#L287),
[`set_theme`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/engine/quickjs/theme_api.rs#L105).

## Unload and one critical integration test

`PluginManager::unload` calls `Plugin::shutdown`, which releases its application
generation and cancels policy tasks. Dropping the manager also shuts down loaded
plugins. `ScriptView::drop` releases its root application generation. Drop all
adapter/overlay view clones during unmount and avoid retaining a policy or host
closure after its native owner is removed. A per-plugin registry must be revoked
with its owner; calling global `clear_exported_modules` from one panel would
revoke unrelated applications' global modules instead.
[`unload`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/plugin.rs#L1511),
[`shutdown`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/plugin.rs#L1183),
[`Drop for PluginManager`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/plugin.rs#L1558).

The key integration test should mount a real native `ShellPanel` through a
committed Solid host tree, loading an actual temporary `gpui-shell.json` and
`main.js`. A fixture can import `context`/`emit` from `solid-host`, render a
host-provided value, emit one initial message, retain an `InputState`, and start
an asynchronous host operation whose completion the test controls with a
channel. The script emits a second message only after that promise resolves.

Assert the rendered snapshot/actual frame contains the host value and the first
typed event reaches the matching Solid root. Remove the panel through the normal
commit path, drain release work, then complete the pending host operation.
Assert the continuation emits nothing, the manager has no loaded plugin, and a
weak view handle cannot upgrade. This proves source evaluation, host calls,
event routing, and unload cancellation in one scenario without arbitrary sleeps
or duplicating upstream sampler tests. Public code cannot inspect Shell's private
`entities()`/task counts; use observable events and weak ownership instead.
[Upstream real-source host-call test](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/tests/host_api.rs#L135),
[upstream unload/entity test](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/plugin.rs#L2005).

The ownership refactors additionally need one two-plugin acceptance scenario:
both open overlays; unloading A preserves B's overlay/focus, A's module cannot
call B's methods, and an attempted plugin theme mutation cannot bypass the host.
Those are separate consequences of the new embedding contract.

## Cargo and platform qualification

Solid currently requests crates.io `rquickjs = "=0.12.2"`. Shell at this revision
aliases `quickjs-jit` and `quickjs-jit-runtime` 0.12.7 from Git commit
`9a83a7f9cdc178a0ab5d7b29c3c9227b0dcb8620`, and its standard-library facade is
from commit `605da483611a3548edb8c33fdff602e3f5f42076`. Shell also needs the root
Cargo patch for LLRT's `rquickjs` dependencies to its shipped compatibility
adapter. A patch supplying version 0.12.7 does not satisfy Solid's exact 0.12.2
requirement.
[Solid manifest](../../crates/solid-gpui/Cargo.toml),
[Shell dependencies](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/Cargo.toml),
[upstream dependency adapter](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/rquickjs-compat/Cargo.toml).

Unify Solid's Rust binding dependency onto the exact same `quickjs-jit` source
and revision before linking Shell and the embedded runtime together. Continue
to create separate VM instances; shared dependency identity does not mean shared
application state. Both existing `rquickjs-sys` 0.12.2 and `quickjs-jit-sys`
compile a native `libquickjs.a`; leaving both implementations in the same binary
creates an unresolved symbol/ABI collision risk, not a safe engine boundary.
This risk was identified from source; this research did not attempt the link.
[JIT binding manifest](https://github.com/longbridge/quickjs-jit/blob/9a83a7f9cdc178a0ab5d7b29c3c9227b0dcb8620/Cargo.toml),
[native build script](https://github.com/longbridge/quickjs-jit/blob/9a83a7f9cdc178a0ab5d7b29c3c9227b0dcb8620/sys/build.rs#L528).

Keep Shell optional and desktop-only until its platform builds are qualified.
It enables Base inspector reflection and has native network/process/runtime
dependencies. Run the existing embedded QuickJS startup/hot-reload/teardown
tests after dependency unification. Shell debug builds deliberately set JIT
thresholds to `u32::MAX` because upstream observed backend crashes under
short-lived Linux/Windows debug test workloads; release tiering is different,
so a debug test does not establish release JIT acceptance.
[`shell_jit_config`](https://github.com/longbridge/gpui-kit/blob/05433bd8e9e75af2f3aa508141b78ba21bfa6261/crates/shell/src/engine/quickjs/mod.rs#L4970).

This research changed only this internal recipe. It does not add a public SDK
export or a website capability claim. When implementation lands, generate its
native bindings and synchronize the Rust bridge/runtime guides, both published
language copies, component usage/availability notes, and website checks. This
file's local links and immutable source paths were checked; execution/build
verification belongs to the implementation task.

## Appendix: delegate Shell overlays to the existing Component Root

This follow-up inspects the current local vendor, including Solid's owned-overlay
seams. It supersedes the earlier alternative of adding a second ShellRoot.
The adapter should keep one native modal/focus/notification/tooltip stack in the
existing Component Root. The methods below are **proposed Shell interfaces**;
the Component methods cited in the mapping already exist locally.

### Minimal object-safe service

Store `Rc<dyn OverlayHost>` in the plugin's policy. Each instance captures one
native mount owner, its generation, a weak Component Root, and its WindowId.
No caller supplies an owner string. All mutations validate that the lease is
live and the current window matches before touching Root.

```rust
pub trait OverlayHost {
    fn open_dialog(&self, request: HostedDialog, window: &mut Window, cx: &mut App)
        -> anyhow::Result<u32>;
    fn close_dialog(&self, window: &mut Window, cx: &mut App)
        -> anyhow::Result<bool>;
    fn close_all_dialogs(&self, window: &mut Window, cx: &mut App)
        -> anyhow::Result<u32>;
    fn has_active_dialog(&self, window: &Window, cx: &App)
        -> anyhow::Result<bool>;
    fn open_sheet(&self, request: HostedSheet, window: &mut Window, cx: &mut App)
        -> anyhow::Result<()>;
    fn close_sheet(&self, window: &mut Window, cx: &mut App)
        -> anyhow::Result<bool>;
    fn has_active_sheet(&self, window: &Window, cx: &App)
        -> anyhow::Result<bool>;
    fn push_toast(&self, request: HostedToast, window: &mut Window, cx: &mut App)
        -> anyhow::Result<()>;
    fn remove_toast(&self, id: &str, window: &mut Window, cx: &mut App)
        -> anyhow::Result<bool>;
    fn clear_toasts(&self, window: &mut Window, cx: &mut App)
        -> anyhow::Result<()>;
    fn tooltip_overlay(&self, window: &Window, cx: &App)
        -> anyhow::Result<Entity<gpui_base::TooltipOverlay>>;
    fn release_owner(&self, window: &mut Window, cx: &mut App);
}
```

Keep request types independent of Component: `HostedDialog` carries optional
`AnyView` content, title/description, plain-versus-alert presentation,
`DialogOptions`, optional footer actions, and owner-bound close/action callbacks.
`HostedSheet` carries content, placement, optional title, and close callback.
`HostedToast` carries the existing Shell `ToastRequest` plus optional native
click/close callbacks. These fields preserve the currently exposed Component
Shell title, AlertDialog, cancel-button, and callback behavior; an `AnyView`-only
request would lose those controls. A small retained factory view can adapt
`ComponentElementFactory::build` into `AnyView` content without sharing JS values.

Shell's raw `window.open_dialog` uses plain presentation and no synthetic close
button/footer. Its return depth and query/close operations refer to the calling
owner's active dialogs. Component Shell uses the same service with its explicit
native presentation fields. Carry phase checks in the JS binding and lifecycle
checks in the service. Expose the captured service to Component Shell through
`ComponentEventEffects::overlay_host()` or a similarly explicit generation-bound
accessor; it must resolve inside the existing `effects.event(...)` scope.
[Existing effect scope](../../vendor/gpui-kit/crates/shell/src/component_registry.rs),
[current Component Shell presentation fields](../../vendor/gpui-kit/crates/component-shell/src/shell/window_effects/mod.rs).

### Exact Component delegation and lifecycle

The existing owned methods are:

```rust
Root::open_dialog_owned(
    &mut self,
    build: impl Fn(Dialog, &mut Window, &mut App) -> Dialog + 'static,
    on_closed: impl Fn(OverlayCloseReason, &mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut Context<Root>,
) -> OverlayToken;

Root::open_alert_dialog_owned(/* same shape, using AlertDialog */) -> OverlayToken;

Root::open_sheet_owned(
    &mut self,
    placement: Placement,
    build: impl Fn(Sheet, &mut Window, &mut App) -> Sheet + 'static,
    on_closed: impl Fn(OverlayCloseReason, &mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut Context<Root>,
) -> OverlayToken;
```

Retain an owner-local `Vec<OverlayToken>` and an optional sheet token. Tokens
are cloneable and expose `is_open()`, `close(window, cx) -> bool`,
`refresh(cx)`, and `set_sheet_placement(placement, cx)`. Remove closed tokens
when querying/counting. Closing a token identifies its exact native overlay;
Root already repairs focus restoration when a middle dialog is removed. Never
use `WindowExt::close_dialog`, `close_all_dialogs`, or `close_sheet` from a
plugin because those target window-wide state.
[Owned overlay implementation](../../vendor/gpui-kit/crates/component/src/root/overlay.rs),
[unscoped WindowExt operations](../../vendor/gpui-kit/crates/component/src/window_ext.rs).

For a plain Shell dialog, map `DialogOptions::is_escape_dismissable()` to
`Dialog::keyboard(bool)` and `is_backdrop_dismissable()` to
`overlay_closable(bool)`; suppress the extra close button and place the retained
content in `.content(...)`. Styled requests preserve their title, footer and
callbacks. Deliver close callbacks from one chosen point rather than both
native `.on_close` and the ownership callback.
[Dialog setters](../../vendor/gpui-kit/crates/component/src/dialog/dialog.rs).

Preserve Shell's existing overlay refresh contract: its private
`rebuild_script_overlay(content: &AnyView, cx: &mut App)` downcasts content to
`ScriptView` and calls `invalidate()` before the root renders that content.
The Component adapter's dialog/sheet builder must perform the same invalidation
for script overlay views before yielding the child. This schedules no extra
frame but lets `window.refresh()` reevaluate the overlay's content function;
mounting an ordinary cached `AnyView` without this hook freezes captured state.
Native Component factory views retain their normal rendering behavior.
[Existing Shell refresh hook](../../vendor/gpui-kit/crates/shell/src/root.rs).

**Sheet ownership needs an opening guard.** `open_sheet_owned` replaces the
window's single sheet, even if another owner opened it. If
`window.has_active_sheet(cx)` is true while this lease has no open sheet token,
refuse the new request before mutation. Replacing this owner's own sheet is
valid. Otherwise B could replace A despite having no token that can close A.
[Replacement semantics](../../vendor/gpui-kit/crates/component/src/root/overlay.rs).

For toasts, use
`Notification::id1::<ShellOwnedNotification>(owner_generation_and_toast_key)`
and retain those exact keys. `Root::remove_notification1::<ShellOwnedNotification>`
closes only a key; do not call type-wide remove or global clear. Force
`delivery(NotificationDelivery::InApp)` unless the host explicitly grants OS
delivery, because an unset delivery inherits application theme settings.
`Root::notification` is public, and `NotificationList::notification1`/`revise`
can find/update a live keyed notification when that is the desired operation.
[Root notification methods](../../vendor/gpui-kit/crates/component/src/root.rs),
[Notification keys/delivery/revision](../../vendor/gpui-kit/crates/component/src/notification.rs).

Shell's numeric/null toast timeout cannot be represented by the current native
`autohide: bool`: `NotificationList::push` and `revise` use a fixed five seconds.
Add a native `Option<Duration>` timeout seam and forward it to the existing
Base `ToastOptions`; do not silently round every script timeout to five seconds
or add a competing per-toast JS timer. A dismissed notification remains mounted
during its exit transition, so adapter state must mark it closing immediately
to preserve `remove_toast`'s true-once return value. Close callbacks must check
the generation before touching a replacement session.
[Native notification timing](../../vendor/gpui-kit/crates/component/src/notification.rs),
[Shell toast contract](../../vendor/gpui-kit/crates/shell/src/root.rs).

For tooltips, narrowly expose Component `Root::tooltip_overlay(&Window, &App)`
(currently `pub(crate)`) and return that entity through the service. Shell's
tooltip materializer must capture its service during materialization, retain
a per-trigger keyed `Entity<()>`, and call
`TooltipRequest::new(...).owned_by(owner.downgrade())`. Base already observes
that owner's release and cancels pending/visible tooltips. Unload should drop
the owner's trigger handles, not call shared `TooltipOverlay::hide` and erase
another plugin's tooltip. Existing Component managed tooltips already use this
owned request mechanism and need no duplicate Shell tooltip layer.
[Component managed tooltips](../../vendor/gpui-kit/crates/component/src/tooltip.rs),
[Base request ownership/cancellation](../../vendor/gpui-kit/crates/base/src/tooltip.rs).

`release_owner` is idempotent: revoke the lease first, take its tokens/keys out
of interior storage, then close them and unload the plugin generation. Do not
hold a `RefCell` borrow across token close or Root mutation: `on_closed` runs
synchronously and may update bookkeeping. Revocation suppresses delayed native
callbacks during notification exit. Window destruction should release the weak
root and retained mount state without trying to mutate a different window.

### Bypass inventory to patch

| Current source | Required routing/guard |
| --- | --- |
| Shell `engine/quickjs/overlay.rs` | Replace every `with_root` dispatch for dialog/sheet/toast queries and mutations with the calling policy's service; retain phase guards. FPS show/hide/query use application services, not modal storage. |
| Shell `materialize/components/tooltip.rs` | Replace `ShellRoot::tooltip_overlay` lookup and unowned requests with the policy service and trigger ownership. A missing service is an explicit unsupported operation, not a silent absent tooltip. |
| Component Shell `shell/window_effects/mod.rs` | All four direct paths bypass ownership: `window.open_dialog`, `open_alert_dialog`, `open_sheet_at`, and `push_notification`. Replace inside the existing event transaction; preserve its callback generation and once-per-event checks. Native notification IDs currently contain only the trigger ID and collide across plugins. |
| Component Shell `shell/lifecycle/menu.rs` | `ComponentAppEffects::replace` directly calls `cx.set_menus` and restores a saved global menu on cleanup. A plugin can overwrite another's menus, and an old cleanup can restore obsolete state. Route through host-owned app-menu contributions or reject the application menu effect for embedded plugins. This is separate from ordinary popup/context menus. |
| Shell `engine/quickjs/theme_api.rs` | Gate/delegate `set_theme` before `Theme::global_mut`; the host updates Component and its Base projection together. Reading tokens stays available. |
| Shell `materialize/components/fps.rs` | Direct `gpui_fps::fps_monitor(window, cx)` is another path beyond show/hide globals. Reject this element under a policy without HUD ownership or route it through the single host monitor service. Guarding only `overlay.rs` leaves this bypass. |
| Shell `engine/quickjs/window_api.rs` | `window.set_rem_size` directly changes the shared window typography and bypasses theme ownership. Gate/delegate it with other host-owned appearance/window mutations. `window.refresh` should refresh the owner's overlay script views as required by its public contract, not create a second root. |
| Plugin release path and policy | Call `release_owner` before retiring callback/application state, with the owner already revoked. Ensure cloned policy/handler handles cannot reopen UI after unload. |

Sources:
[Shell overlay bindings](../../vendor/gpui-kit/crates/shell/src/engine/quickjs/overlay.rs),
[Shell tooltip materializer](../../vendor/gpui-kit/crates/shell/src/materialize/components/tooltip.rs),
[Component Shell window effects](../../vendor/gpui-kit/crates/component-shell/src/shell/window_effects/mod.rs),
[application menus](../../vendor/gpui-kit/crates/component-shell/src/shell/lifecycle/menu.rs),
[theme mutation](../../vendor/gpui-kit/crates/shell/src/engine/quickjs/theme_api.rs),
[inline HUD](../../vendor/gpui-kit/crates/shell/src/materialize/components/fps.rs),
[window mutations](../../vendor/gpui-kit/crates/shell/src/engine/quickjs/window_api.rs).

Component Shell's `shell/lifecycle/tooltip.rs` uses a real Component Button's
managed tooltip, so it already uses the shared Component Root and trigger
lifetime. Popover/HoverCard/DropdownMenu are local native elements rather than
`WindowExt` modal operations; keep their existing retained lifecycle instead of
routing all floating UI through the modal adapter.
[Component tooltip binding](../../vendor/gpui-kit/crates/component-shell/src/shell/lifecycle/tooltip.rs),
[local overlay adapters](../../vendor/gpui-kit/crates/component-shell/src/shell/overlays/mod.rs).

Verify the integration with two owners in the real Component Root: A opens a
dialog, B opens one above it, unloading A preserves B and its focus; A cannot
replace B's sheet; identically named toasts remain distinct; removing A's tooltip
trigger cannot dismiss B's current tooltip. Exercise both raw Shell window APIs
and Component Shell trigger controls so routing one path cannot mask another.
This appendix changes internal research only; its local source links were
checked, while runtime tests remain the implementation task's responsibility.
