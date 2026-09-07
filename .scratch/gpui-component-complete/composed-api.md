# gpui-component composition and Plot integration research

Research baseline: `gpui-component` checkout `928c3eb776a3d733d9b771f7dea27a6a79242ced`, 2026-09-06. This is implementation research, not product changes or compiled skeletons. `C/` and `B/` refer to `crates/component/src/` and `crates/base/src/` in that checkout; linked source entrypoints appear below. `NativeView` and renderer files were changing in the main task. This report first defines required interfaces, then sketches actual upstream calls.

## 1. Establish native ownership boundaries first

The initially inspected `NativeView` exposed only `mount(props,event,window,cx)` and `update(props,window,cx)`. `ComponentDefinition::view` declared `children:false` and `ViewInstance::render` discarded its child iterator. `with_children(true)` only relaxed validation, without giving retained views reusable subtrees. This composition gap cannot be fixed by adding `ParentElement` to each component. [Repository `NativeView`](../../crates/solid-gpui/src/native/component.rs)

The working tree subsequently introduced `ExtensionChildren`/`ExtensionContent`. They retain a weak SolidRoot and event route, rebuild AnyElements from the **currently committed Rust Host Tree**, and check route/surface/epoch. That approach serves the closures below. Do not capture a frame's `Vec<AnyElement>` and `take()` it inside `Fn`: native callbacks may run over many later frames. `ExtensionContent` is a cloneable deferred render wrapper suitable for chart-label closures without `App` access. [Native subtree projection](../../crates/solid-gpui/src/renderer/extensions.rs)

Skeleton conventions:

```rust
// Obtained from a validated Host Tree, not an arbitrary forgeable JavaScript ID.
type Slot = solid_gpui::ExtensionContent;

#[derive(Clone)]
struct Slots {
    content: Slot,
    trigger: Option<Slot>,
    title: Option<Slot>,
    description: Option<Slot>,
    footer: Option<Slot>,
}

// Every Props/Event/Command DTO uses the Deserialize/Serialize/TS contract.
// Commit validation checks optional values, ranges, and reference integrity.
// The generator maps named slots to host groups; Rust receives no JSX/Solid owners/JS closures.
```

Slots belong to component instances, not global node IDs. Revoke them on node removal, type replacement, and epoch retirement so retained overlays cannot revive old nodes. Generate slot-group indices from contracts and validate their structure; Dialog cannot name any arbitrary Host Node as content. Child commits must notify native slot owners/root so retained Entities cannot reuse stale frames because their own dirty flags stayed unchanged.

Native event closures only emit DTOs. Synchronous getters and boolean decisions read Rust DTO/state only. Dialog on_ok→bool cannot wait synchronously for JavaScript: `closeOnOk` can return directly, while async validation uses `closeOnOk:false`, emits `okRequested {requestId}`, and later receives `resolve({requestId,close})`. Reject stale requests, closed layers, and revoked owners.

## 2. Which components can be wrapped independently

| Type                                                                     | Actual shape in 928c3eb                                                                                                          | Integration                                                                                 |
| ------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| DialogContent/Header/Title/Description/Footer/Action/Close               | `RenderOnce + ParentElement`, `new()`                                                                                            | Independent element wrappers with current-frame children                                    |
| Dialog / AlertDialog / Sheet                                             | `RenderOnce`; `Root` owns overlay builders/focus/layers                                                                          | Retained owner, slots, and **owner-aware overlay APIs**, described next                     |
| Popover / HoverCard                                                      | `RenderOnce`; native `Fn` `content` and base keyed element state                                                                 | Stateless wrapper with stable ID/reusable slot; retain focus/state when commands require it |
| Tooltip                                                                  | `Render`; build(window,cx)→AnyView; trigger uses `ManagedTooltipExt`                                                             | Wrapper plus tooltip-content slot; ordinary children alone are insufficient                 |
| PopupMenuItem / PopupMenu                                                | Specific enum builder / `Entity<PopupMenu>`                                                                                      | Compile `MenuSpec` DTO into native builders; custom items reference slots                   |
| ContextMenu                                                              | `ContextMenu<E>` wraps `InteractiveElement + ParentElement + Styled`; menu is private                                            | Use context_menu(builder), not private `ContextMenu::new(...).menu(...)`                    |
| DropdownButton                                                           | button(Button), not `AnyElement`                                                                                                 | Typed `ButtonSpec` plus dropdown_menu(builder); arbitrary JSX cannot replace native Button  |
| AppMenuBar                                                               | AppMenuBar::new(cx)→Entity; reads global `OwnedMenu` and reloads                                                                 | Retained view plus application menu model, not arbitrary local popup children               |
| DockArea                                                                 | `Entity<DockArea>`; `gpui-base` owns layout/state, `DockSkin` supplies appearance                                                | Retained dock owner, typed layout DTO, and native `JsPane` Entities                         |
| TabPanel / Tiles                                                         | **No public constructors with these names in this revision**; actual APIs are `DockLayout::tabs/tiles`, `TabGroup`, `TilesState` | DockLayout variants, not invented `TabPanel::new`/`Tiles::new`                              |
| Settings / `SettingPage` / `SettingGroup` / `SettingItem` / SettingField | Settings accepts pages, pages accept groups, groups accept items; descriptors are not ordinary render elements                   | Typed DTOs or a dedicated typed-child tree compiled to native builders, not AnyElement      |
| Seven Charts                                                             | Generic `IntoPlot` types with Rust `Fn` field accessors                                                                          | Chart-specific DTOs converted to fixed Rust data and native accessors                       |
| PlotAxis/Grid/PlotLabel/Line/Area/Bar/RadialLine/Arc                     | Mostly paint-only types without ordinary IntoElement                                                                             | One `Plot` renderer constructs typed primitives; `child(PlotAxis)` is invalid               |
| Scale/Pie/Stack/Sankey layout                                            | Mathematical objects: `tick/arcs/series/layout`                                                                                  | Typed computation commands or internal Plot functions, not visual components                |

Sources: [Dialog module][dialog], [PopupMenu][menu], [Dock exports][dock], [Settings][settings], [Plot trait][plot].

## 3. Dialog, AlertDialog, and Sheet: lifecycle before appearance

### 3.1 Public API boundaries in the inspected revision

`Dialog::new(&mut App)` supports `trigger(IntoElement)`, content(Fn(DialogContent,&mut Window,&mut App)→DialogContent), `title`, `footer`, `button_props`, `on_ok`/`on_cancel`/`on_close`, `close_button`, `margin_top`, `width`/`w`, `max_w`, `overlay`, `overlay_closable`, `keyboard`, Styled, and ParentElement. `header` is `pub(crate)` and unavailable externally. [Dialog][dialog]

`AlertDialog::new(&mut App)` supports `confirm`, `trigger`, `content`, `footer`, `icon`, `title`, `description`, `button_props`, `width`, `show_cancel`, `close_button`, `keyboard`, and `on_ok/cancel/close`. Trigger/`content` mode asserts against `icon/title/description/button_props`, so those DTO modes must be exclusive. `overlay_closable` is deprecated and ineffective: AlertDialog never closes from its backdrop. Do not expose a writable JavaScript prop with no effect. [AlertDialog][alert-dialog]

`Sheet::new(&mut Window,&mut App)` supports `title`, `footer`, `size(DefiniteLength)`, `resizable`, `overlay`, `overlay_closable`, `on_close`, Styled, and ParentElement. Placement is crate-private and must be selected through `WindowExt::open_sheet_at(Placement,...)`. `SheetSettings.margin_top` is global Root configuration. [Sheet][sheet]

`AlertDialog::on_close` also has a source defect: it writes `self.base.button_props.on_close`, but imperative `build_surface` overwrites base button props from `self.button_props`. The trigger branch copies on_close separately; the imperative branch does not. Copy that callback in `build_surface` or have owned Root report closure centrally before integration, otherwise `open_alert_dialog(...on_close(...))` loses its callback. [AlertDialog][alert-dialog]

`WindowExt` exposes `open_dialog(builder)`, `open_alert_dialog(builder)`, `close_dialog()` for the top layer, `close_all_dialogs()`, and singleton `open_sheet_at`/`close_sheet`. It has no owner/token for removing a specific layer. [`WindowExt`][window-ext]

Do not implement controlled `open` by returning `Dialog::new(cx)` directly from `NativeView::render`. `RenderOnce` uses `Root.active_dialogs.len()` for topmost status and Root for closing; `open` stores stable focus handles and selection scope in Root. A detached Dialog cannot preserve equivalent focus traps or stacking. [Dialog render][dialog-render], [Root overlays][root]

Weak slots alone are insufficient: after a Dialog opens and its JSX owner unmounts, Root's retained closure can still draw an empty modal/backdrop and hold focus. Nested owners, non-top layer closure, and multiple SolidRoots cannot assume the last layer belongs to a given owner.

### 3.2 Required owner APIs and DTOs

These APIs are **minimal lifecycle capabilities to add to gpui-component Root**, not capabilities already present in that revision:

```rust
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct OverlayOwner { surface: u64, epoch: u64, node: u32, generation: u64 }

enum ModalKind { Dialog, Alert }
struct OverlayToken { owner: OverlayOwner, serial: u64 }

// Root owns owner/token, builder, focus, and selection_scope.
// open/update/close return explicit results; never close_all for one JS owner.
fn open_dialog_owned(owner: OverlayOwner, builder: DialogBuilder) -> OverlayToken;
fn update_dialog_owned(token: &OverlayToken, builder: DialogBuilder) -> Result<(), Retired>;
fn close_dialog_owned(token: &OverlayToken) -> Result<(), Retired>;
fn open_sheet_owned(owner: OverlayOwner, placement: Placement, builder: SheetBuilder)
    -> OverlayToken;
fn update_sheet_owned(token: &OverlayToken, placement: Placement, builder: SheetBuilder)
    -> Result<(), Retired>;
fn close_sheet_owned(token: &OverlayToken) -> Result<(), Retired>;
```

Removing a middle dialog updates the next layer's `previous_focused_handle` to the removed layer's predecessor; restore focus only when removing the top layer. Builder replacement preserves focus_handle, selection_scope, and layer identity. Replacing a singleton Sheet emits `closed {reason:"replaced"}` to its old owner and retires its token. Surface disposal removes only that Surface's tokens. Implement this inside Root without reflecting on or copying private vectors.

Proposed JavaScript contract:

```ts
type DialogProps = {
  open: boolean;
  title?: string;
  width?: number;
  maxWidth?: number;
  marginTop?: number;
  closeButton?: boolean;
  overlay?: boolean;
  overlayClosable?: boolean;
  keyboard?: boolean;
  okText?: string;
  cancelText?: string;
  showCancel?: boolean;
  okVariant?: ButtonVariant;
  cancelVariant?: ButtonVariant;
  closeOnOk: boolean;
  closeOnCancel: boolean;
  // JSX slots: trigger?, content, title?, footer?
};
type AlertDialogProps = {
  open: boolean;
  width?: number;
  closeButton?: boolean;
  keyboard?: boolean;
  contentMode: "standard" | "custom";
  title?: string;
  description?: string;
  icon?: IconSpec;
  showCancel?: boolean;
  okText?: string;
  cancelText?: string;
  okVariant?: ButtonVariant;
  cancelVariant?: ButtonVariant;
  closeOnOk: boolean;
  closeOnCancel: boolean;
  // Standard content and a custom content slot are mutually exclusive.
};
type SheetProps = {
  open: boolean;
  placement: "left" | "right" | "top" | "bottom";
  size: { unit: "px"; value: number } | { unit: "relative"; value: number };
  title?: string;
  resizable?: boolean;
  overlay?: boolean;
  overlayClosable?: boolean;
  // content/title/footer JSX slots
};
type ModalEvent =
  | { kind: "openRequested" }
  | { kind: "okRequested" | "cancelRequested"; requestId: number }
  | { kind: "closed"; reason: "ok" | "cancel" | "dismiss" | "programmatic" | "replaced" };
```

### 3.3 Native builder skeleton

Here `p` contains validated Rust props with defaults applied, and NativeView owns `event`. Closures read the current `Rc<RefCell<DialogModel>>` rather than capturing only initial props.

```rust
#[derive(Clone)]
struct EventSequence(Rc<Cell<u32>>);
impl EventSequence {
    fn next(&self) -> u32 {
        let next=self.0.get().checked_add(1).expect("native instance sequence exhausted");
        self.0.set(next);
        next
    }
}

fn build_dialog(d: Dialog, model: &DialogModel) -> Dialog {
    let p = &model.props;
    let ok = model.event.clone();
    let cancel = model.event.clone();
    let close = model.event.clone();
    let close_ok = p.close_on_ok;
    let close_cancel = p.close_on_cancel;
    let content = model.slots.content.clone();
    let request_ids = model.request_ids.clone();
    let cancel_ids = request_ids.clone();
    let mut d = d.width(px(p.width))
        .close_button(p.close_button).overlay(p.overlay)
        .overlay_closable(p.overlay_closable).keyboard(p.keyboard)
        .button_props(DialogButtonProps::default()
            .ok_text(p.ok_text.clone()).cancel_text(p.cancel_text.clone())
            .ok_variant(p.ok_variant.native()).cancel_variant(p.cancel_variant.native())
            .show_cancel(p.show_cancel))
        .content(move |c, _, _| c.child(content.clone()))
        .on_ok(move |_, _, _| {
            ok.emit(ModalEvent::OkRequested { request_id: request_ids.next() });
            close_ok
        })
        .on_cancel(move |_, _, _| {
            cancel.emit(ModalEvent::CancelRequested { request_id: cancel_ids.next() });
            close_cancel
        })
        .on_close(move |_, _, _| close.emit(ModalEvent::Closed {
            reason: CloseReason::Dismiss,
        }));
    if let Some(width) = p.max_width { d = d.max_w(px(width)); }
    if let Some(top) = p.margin_top { d = d.margin_top(px(top)); }
    if let Some(title) = &model.slots.title { d = d.title(title.clone()); }
    else if let Some(title) = &p.title { d = d.title(title.clone()); }
    if let Some(footer) = &model.slots.footer { d = d.footer(footer.clone()); }
    d
}

fn build_alert(a: AlertDialog, model: &AlertModel) -> AlertDialog {
    let p = &model.props;
    let ok = model.event.clone();
    let cancel = model.event.clone();
    let close = model.event.clone();
    let close_ok = p.close_on_ok;
    let close_cancel = p.close_on_cancel;
    let ok_ids=model.request_ids.clone();
    let cancel_ids=ok_ids.clone();
    let mut a = a.width(px(p.width)).close_button(p.close_button).keyboard(p.keyboard);
    match &p.content {
        AlertContent::Standard { title, description, icon, buttons } => {
            a = a.title(title.clone()).description(description.clone())
                .button_props(buttons.native());
            if let Some(icon) = icon { a = a.icon(icon.native()); }
            a = a.child(model.slots.content.clone());
        }
        AlertContent::Custom => {
            let slot = model.slots.content.clone();
            a = a.content(move |c, _, _| c.child(slot.clone()));
        }
    }
    if let Some(footer) = &model.slots.footer { a = a.footer(footer.clone()); }
    // Install callbacks after button_props so standard button settings cannot overwrite them.
    a.on_ok(move |_,_,_| { ok.emit(ModalEvent::OkRequested {request_id:ok_ids.next()});close_ok })
        .on_cancel(move |_,_,_| { cancel.emit(ModalEvent::CancelRequested {request_id:cancel_ids.next()});close_cancel })
        .on_close(move |_,_,_| close.emit(ModalEvent::Closed {reason:CloseReason::Dismiss}))
}

fn build_sheet(s: Sheet, model: &SheetModel) -> Sheet {
    let p = &model.props;
    let event = model.event.clone();
    let mut s = s.size(p.size.native()).resizable(p.resizable)
        .overlay(p.overlay).overlay_closable(p.overlay_closable)
        .child(model.slots.content.clone())
        .on_close(move |_, _, _| event.emit(ModalEvent::Closed { reason: CloseReason::Dismiss }));
    if let Some(title) = &model.slots.title { s = s.title(title.clone()); }
    else if let Some(title) = &p.title { s = s.title(title.clone()); }
    if let Some(footer) = &model.slots.footer { s = s.footer(footer.clone()); }
    s
}
```

Give each native Dialog/Alert owner an `EventSequence` request counter. The sketch's generic on_close can express only dismissal; the final owned Root should report precise `CloseReason` once, removing duplicate builder-close events. Record synchronous ok/cancel before returning true, record resolve's programmatic/action reason, and emit one closed after Root closes. This also avoids the inspected imperative AlertDialog callback defect.

`mount/update` retains model, slots, and an optional token. false→true opens through the owned API; `true→true` replaces the builder; `true→false` closes that token. `resolve` accepts only the current pending request. Unmount/window closure/epoch retirement explicitly disposes tokens; Rust `Drop` cannot assume Window/App access. `Render` draws only the trigger slot. Native trigger clicks emit `openRequested` or open immediately in explicit uncontrolled mode. Full Dialog integration requires the lifecycle API to exist first.

## 4. Popover / HoverCard / Tooltip

`Popover::trigger` requires `Selectable + IntoElement`; a `Slot` wrapper does not automatically implement Selectable. Provide a native selectable trigger whose selected state affects agreed wrapper style/accessibility while retaining JavaScript children. Arbitrary triggers need not be restricted to Button DTOs. [Popover][popover]

```rust
#[derive(IntoElement)]
struct SlotTrigger { slot: Slot, selected: bool }
impl Selectable for SlotTrigger {
    fn selected(mut self, selected: bool) -> Self { self.selected = selected; self }
    fn is_selected(&self) -> bool { self.selected }
}
impl RenderOnce for SlotTrigger {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().child(self.slot) // Map selected to the host's selection semantics/style.
    }
}

fn popover(p: &PopoverProps, slots: &Slots, event: Event<OpenEvent>, id: ElementId)
    -> Popover
{
    let content = slots.content.clone();
    let mut out = Popover::new(id)
        .anchor(p.anchor.native()).mouse_button(p.mouse_button.native())
        .default_open(p.default_open).appearance(p.appearance)
        .overlay_closable(p.overlay_closable)
        .trigger(SlotTrigger { slot: slots.trigger.clone().unwrap(), selected: false })
        .content(move |_, _, _| content.clone())
        .on_open_change(move |open, _, _| event.emit(OpenEvent { open: *open }));
    if let Some(open) = p.open { out = out.open(open); }
    out
}

fn hover_card(p: &HoverCardProps, slots: &Slots, event: Event<OpenEvent>, id: ElementId)
    -> HoverCard
{
    let content = slots.content.clone();
    HoverCard::new(id).anchor(p.anchor.native())
        .open_delay(Duration::from_millis(p.open_delay_ms))
        .close_delay(Duration::from_millis(p.close_delay_ms))
        .appearance(p.appearance).trigger(slots.trigger.clone().unwrap())
        .content(move |_, _, _| content.clone())
        .on_open_change(move |open, _, _| event.emit(OpenEvent { open: *open }))
}

fn tooltip(p: &TooltipProps, slots: &Slots, id: ElementId) -> impl IntoElement {
    let content = slots.content.clone();
    let text = p.text.clone();
    let kbd = p.key_binding.clone();
    div().id(id).child(slots.trigger.clone().unwrap())
        .managed_tooltip_at(p.placement.native(), move |window, cx| {
            let mut tip = match &text {
                Some(text) => Tooltip::new(text.clone()),
                None => {
                    let content = content.clone();
                    Tooltip::element(move |_, _| content.clone())
                }
            };
            if let Some(kbd) = &kbd { tip = tip.key_binding(Some(kbd.native())); }
            tip.build(window, cx)
        })
}
```

Popover props include eight `anchor` positions, `mouseButton`, `defaultOpen`, optional controlled open, `appearance`, `overlayClosable`, and trigger style. Retain `FocusHandle` natively rather than exposing it to JavaScript. HoverCard supports only `openDelay/closeDelay/anchor/appearance/onOpenChange`; it has no controlled open setter. Tooltip `action(&dyn Action, Option<&str>)` would require a registered native Action string/DTO, not an arbitrary JavaScript function. [HoverCard][hover-card], [Tooltip][tooltip]

## 5. Menu / ContextMenu / DropdownButton / AppMenuBar

### 5.1 Complete menu DTO and native compiler

```ts
type MenuSpec = {
  items: MenuItem[];
  minWidth?: number;
  maxWidth?: number;
  maxHeight?: number;
  scrollable: boolean;
  checkSide: "left" | "right";
  externalLinkIcon: boolean;
};
type MenuItem =
  | { kind: "separator" }
  | { kind: "label"; label: string }
  | {
      kind: "item";
      id: string;
      label: string;
      icon?: IconSpec;
      disabled?: boolean;
      checked?: boolean;
      action?: NativeActionSpec;
    }
  | { kind: "link"; label: string; href: string; icon?: IconSpec; disabled?: boolean }
  | {
      kind: "element";
      id: string;
      slot: number;
      icon?: IconSpec;
      disabled?: boolean;
      checked?: boolean;
      action?: NativeActionSpec;
    }
  | { kind: "submenu"; label: string; icon?: IconSpec; disabled?: boolean; menu: MenuSpec };
```

A `scrollable` parent cannot contain submenus; reject this explicit upstream restriction during DTO validation. `PopupMenuItem::submenu` directly accepting an Entity is valid: `PopupMenu::render` from line 1390 fills parent_menu/priority for it. This supports disabled submenus and Dock hooks with only `Context<JsPane>`. Reading `submenu_with_icon` alone must not lead to a false claim that this entrypoint lacks parent linkage. [PopupMenu][menu], [submenu render linkage][menu-linking]

```rust
// MenuSink is Rc<dyn Fn(MenuEvent)> in Rust, forwarding the same builder's events
// into a Menu NativeView Event<MenuEvent> or Dock Event<DockBridgeEvent>.
// Both call Event::emit only; neither invokes JavaScript callbacks directly.
type MenuSink = Rc<dyn Fn(MenuEvent)>;
fn build_menu(mut menu: PopupMenu, spec: &MenuSpec, slots: &[Slot],
              event: &MenuSink, window: &mut Window,
              cx: &mut App) -> PopupMenu {
    menu = menu.scrollable(spec.scrollable).check_side(spec.check_side.native())
        .external_link_icon(spec.external_link_icon);
    if let Some(v) = spec.min_width { menu = menu.min_w(px(v)); }
    if let Some(v) = spec.max_width { menu = menu.max_w(px(v)); }
    if let Some(v) = spec.max_height { menu = menu.max_h(px(v)); }
    for item in &spec.items {
        match item {
            MenuItem::Separator => { menu = menu.separator(); }
            MenuItem::Label { label } => { menu = menu.label(label.clone()); }
            MenuItem::Submenu { label, icon, disabled, menu: nested } => {
                let spec = nested.clone();
                let slots = slots.to_vec();
                let event = event.clone();
                let submenu = PopupMenu::build(window,cx,move |menu,window,cx| {
                    build_menu(menu,&spec,&slots,&event,window,cx)
                });
                let mut item = PopupMenuItem::submenu(label.clone(),submenu).disabled(*disabled);
                if let Some(icon) = icon { item = item.icon(icon.native()); }
                menu = menu.item(item);
            }
            item => {
                let mut native = match item {
                    MenuItem::Item { label, .. } => PopupMenuItem::new(label.clone()),
                    MenuItem::Link { label, href, .. } => PopupMenuItem::link(label.clone(), href.clone()),
                    MenuItem::Element { slot, .. } => {
                        let slot = slots[*slot].clone();
                        PopupMenuItem::element(move |_, _| slot.clone())
                    }
                    _ => unreachable!(),
                };
                if let Some(icon) = item.icon() { native = native.icon(icon.native()); }
                native = native.disabled(item.disabled()).checked(item.checked());
                if let Some(action) = item.action() { native = native.action(action.native()); }
                // link() already opens URLs natively; do not replace it with event-only behavior.
                if let Some(id) = item.event_id() {
                    let id = id.to_owned();
                    let event = event.clone();
                    native = native.on_click(move |_, _, _| event(MenuEvent::Selected { id: id.clone() }));
                }
                menu = menu.item(native);
            }
        }
    }
    menu
}
```

`PopupMenu` can be retained: mount with `PopupMenu::build(window,cx,...)`, update through `entity.update(cx, |m,cx| m.rebuild(window,cx,...))`, and preserve focus/parent/priority identity. Retain its `DismissEvent` `Subscription` in the view for automatic unsubscription on drop. Prop updates must not discard an open menu Entity. [`PopupMenu`::rebuild][menu-rebuild]

```rust
// ContextMenu::menu is private; this is the public entrypoint.
let context_menu = div().id(native_id).child(content_slot)
    .context_menu(move |m, w, cx| build_menu(m, &spec, &slots, &event, w, cx));

// Compile ButtonSpec for DropdownButton; AnyElement cannot be cast into Button.
let dropdown = DropdownButton::new(native_id)
    .button(build_button(&p.button))
    .disabled(p.disabled)
    .dropdown_menu_with_anchor(p.anchor.native(), move |m,w,cx| {
        build_menu(m, &spec, &slots, &event, w, cx)
    });

// Ordinary Button dropdown mode can emit open-state changes.
let dropdown_menu = build_button(&p.button)
    .dropdown_menu_with_anchor(p.anchor.native(), move |m,w,cx| build_menu(m,&spec,&slots,&event,w,cx))
    .on_open_change(move |open,_,_| open_event.emit(OpenEvent { open:*open }));
```

`DropdownButton` forwards variant/size/selected/outline/disabled through `ButtonVariants/Sizable/Selectable/Disableable`. Its split-button primary action belongs to ButtonSpec; the right side opens the menu. [`DropdownButton`][dropdown-button], [DropdownMenu][dropdown-menu], [ContextMenu][context-menu]

Updating an open menu requires more than replacing the DropdownMenu builder closure: `DropdownMenuState.menu` is reused until Dismiss. Builders passed to dropdown_menu/context_menu can register `cx.weak_entity()` in bridge `live_menus`; prop updates `rebuild` through those weak handles and native notification redraws current items. Track menu-model revision in NativeView so unrelated child/style commits do not `rebuild` menus. Direct retained PopupMenu already owns its Entity and needs no extra live-handle registry. [DropdownMenu cache][dropdown-menu]

### 5.2 AppMenuBar mirrors the application menu

`AppMenuBar::new(cx)` calls `reload` and reads `GlobalState::global(cx).app_menus()` as `OwnedMenu`. Update `GlobalState::global_mut(cx).set_app_menus(Vec<OwnedMenu>)` before `bar.update(cx, |bar,cx| bar.reload(cx))`. Menus are application-scoped; local wrappers must not compete to replace globals every render. [AppMenuBar][app-menu], [Base GlobalState][global-state]

Use the existing application-menu command/client as sole menu owner. Compile `NativeActionSpec` into registered Actions whose main-UI-root handlers forward item IDs to the correct JavaScript event endpoint. The retained adapter creates its bar at mount, reloads on menu revision changes, renders `bar.clone()`, and unsubscribes owned action routes on disposal. macOS native menus and Windows/Linux `AppMenuBar` can share OwnedMenu; PopupMenu custom-element slots are not OwnedMenu entries.

## 6. DockArea, TabGroup, and Tiles: real Pane traits and layout APIs

This revision moves behavior into `gpui-base::dock`. `gpui_component::dock::Panel` is an appearance trait extending `BasePanel`; appearance alone is insufficient. `BasePanel` requires EventEmitter<PanelEvent> + Render + Focusable. Its only method without a default is panel_name(&self)→&'static str, but complete integration also covers visibility, close/zoom permission, active/zoomed/added/removed lifecycle, and dump. [Component Panel][panel], [Base Panel][base-panel]

```rust
struct JsPane {
    spec: PaneSpec,
    content: Slot,
    title: Option<Slot>,
    title_suffix: Option<Slot>,
    focus: FocusHandle,
    group: Option<WeakEntity<TabGroup>>,
    event: Event<DockBridgeEvent>,
    menu_slots: Vec<Slot>,
}
impl EventEmitter<PanelEvent> for JsPane {}
impl Focusable for JsPane {
    fn focus_handle(&self, _: &App) -> FocusHandle { self.focus.clone() }
}
impl Render for JsPane {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().track_focus(&self.focus).child(self.content.clone())
    }
}
impl BasePanel for JsPane {
    fn panel_name(&self) -> &'static str { "solid-gpui-pane" }
    fn visible(&self, _: &App) -> bool { self.spec.visible }
    fn closable(&self, _: &App) -> bool { self.spec.closable }
    fn zoomable(&self, _: &App) -> bool { self.spec.zoomable }
    fn set_active(&mut self, active: bool, _: &mut Window, _: &mut Context<Self>) {
        self.event.emit(DockBridgeEvent::Active { pane: self.spec.id.clone(), active });
    }
    fn set_zoomed(&mut self, zoomed: bool, _: &mut Window, _: &mut Context<Self>) {
        self.event.emit(DockBridgeEvent::Zoomed { pane: self.spec.id.clone(), zoomed });
    }
    fn on_added_to(&mut self, group: WeakEntity<TabGroup>, _: &mut Window, _: &mut Context<Self>) {
        self.group = Some(group);
    }
    fn on_removed(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.group = None;
        self.event.emit(DockBridgeEvent::Removed { pane: self.spec.id.clone() });
    }
    fn dump(&self, _: &App) -> PanelState {
        let mut state = PanelState::new("solid-gpui-pane");
        state.info = PanelInfo::panel(serde_json::json!({
            "id": self.spec.id,
            "data": self.spec.persisted_data,
        }));
        state
    }
}
impl Panel for JsPane {
    fn tab_name(&self, _: &App) -> Option<SharedString> { self.spec.tab_name.clone().map(Into::into) }
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        match &self.title {
            Some(slot) => slot.clone().into_any_element(),
            None => self.spec.title.clone().into_any_element(),
        }
    }
    fn title_suffix(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<impl IntoElement> {
        self.title_suffix.clone()
    }
    fn title_style(&self, _: &App) -> Option<TitleStyle> {
        self.spec.title_style.as_ref().map(|v| TitleStyle {
            background: v.background.native(), foreground: v.foreground.native(),
        })
    }
    fn toolbar_buttons(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<Vec<Button>> {
        Some(self.spec.toolbar.iter().map(build_button).collect())
    }
    fn dropdown_menu(&mut self, menu: PopupMenu, window: &mut Window, cx: &mut Context<Self>) -> PopupMenu {
        let event=self.event.clone();
        let pane=self.spec.id.clone();
        let sink:MenuSink=Rc::new(move |menu| event.emit(DockBridgeEvent::Menu {pane:pane.clone(),menu}));
        build_menu(menu,&self.spec.chrome_menu,&self.menu_slots,&sink,window,cx)
    }
    fn zoom_control(&self, _: &App) -> Option<PanelControl> { self.spec.zoom_control.map(|v| v.native()) }
    fn inner_padding(&self, _: &App) -> bool { self.spec.inner_padding }
}
```

Integrate through `panel_handle(entity)`, which returns a base object-safe handle backed by `PanelHandle`. Passing `.panel(entity)` directly prevents `DockSkin` from downcasting to the appearance `PanelHandle`, losing titles/toolbars. Use `.panel_view(panel_handle(pane),cx)` and `.tile_view(...)`. [`PanelHandle`][panel], [DockLayout][dock-layout]

```ts
type DockLayoutSpec =
  | { kind: "split"; axis: "horizontal" | "vertical"; children: { layout: DockLayoutSpec; size?: number }[] }
  | { kind: "tabs"; panes: string[]; activeIndex: number }
  | { kind: "tiles"; panes: { id: string; bounds: { x: number; y: number; width: number; height: number } }[] };
type PaneSpec = {
  id: string;
  title: string;
  tabName?: string;
  visible: boolean;
  closable: boolean;
  zoomable: boolean;
  zoomControl?: "menu" | "toolbar" | "both";
  innerPadding: boolean;
  contentSlot: number;
  titleSlot?: number;
  titleSuffixSlot?: number;
  toolbar: ButtonSpec[];
  chromeMenu: MenuSpec;
  persistedData: JsonValue;
};
type DockProps = {
  id: string;
  version?: number;
  panes: PaneSpec[];
  initialLayout: { center: DockLayoutSpec; left?: DockSpec; right?: DockSpec; bottom?: DockSpec };
  locked: boolean;
  panelStyle: "auto" | "tabBar";
  toggleButtonVisible: boolean;
  // Native code owns ordinary layout interaction; reset through replaceLayout.
};
```

```rust
fn layout(spec: &DockLayoutSpec, panes: &BTreeMap<String, Entity<JsPane>>, cx: &App) -> DockLayout {
    match spec {
        DockLayoutSpec::Split { axis, children } => {
            let mut out = match axis { AxisSpec::Horizontal => DockLayout::h_split(), AxisSpec::Vertical => DockLayout::v_split() };
            for child in children { out = out.child(layout(&child.layout,panes,cx),child.size.map(px)); }
            out
        }
        DockLayoutSpec::Tabs { panes: ids, active_index } => {
            let mut out = DockLayout::tabs().active_index(*active_index);
            for id in ids { out = out.panel_view(panel_handle(panes[id].clone()),cx); }
            out
        }
        DockLayoutSpec::Tiles { panes: tiles } => {
            let mut out = DockLayout::tiles();
            for tile in tiles { out = out.tile_view(panel_handle(panes[&tile.id].clone()),tile.bounds.native(),cx); }
            out
        }
    }
}

// Mount and retain area, skin, panes, and Subscription.
let (area, skin) = DockSkin::dock_area(p.id.clone(), p.version, window, cx);
area.update(cx, |area,cx| {
    area.set_center(layout(&p.initial_layout.center,&panes,cx),window,cx);
    for (placement,dock) in p.initial_layout.docks() {
        area.set_dock(placement,layout(&dock.layout,&panes,cx),window,cx);
        area.set_dock_size(placement,px(dock.size),window,cx);
        area.set_dock_collapsible(placement,dock.collapsible,window,cx);
        if area.is_dock_open(placement) != dock.open { area.toggle_dock(placement,window,cx); }
    }
    area.set_locked(p.locked,window,cx);
});
skin.set_panel_style(p.panel_style.native(),cx);
skin.set_toggle_button_visible(p.toggle_button_visible,cx);
```

`set_dock_size` and `set_dock_collapsible` accept `(placement,value,window,&mut Context<DockArea>)`. Do not repeatedly call `set_center` for ordinary Pane props: it replaces the layout and sends `on_removed` to old panes. Diff props by pane ID, `update` retained Entities, and remove them through `area.remove_panel(entity,window,cx)`. Only explicit `replaceLayout/load` resets geometry. Validate unique IDs, existing layout references, single placement per pane, split/tabs/tiles child types, and sizes. [DockArea][dock-area]

Commands: `dump`, `load`, `replaceLayout`, `addPane`, `removePane`, `movePane`, `splitAt`, `toggleDock`, `resizeDock`, `setDockCollapsible`, `zoomInGroup`, `zoomOut`, `selectTab`. `selectTab` resolves stable pane IDs to current group indices through `JsPane.group` and `TabGroup::select_tab`. Do not expose raw `NodeId/PanelId` for persistent JSON; use view-scoped opaque tokens or resolve current nodes by pane IDs. Tiles supports move/resize/bringToFront/zoom/`undo/redo`. `TilesState::new` is private, so external `undo/redo` needs explicit DockArea access/forwarding, not fabricated Entities. [TilesState][tiles-state]

`DockAreaState` has serde but no TypeScript derive. Define recursive DTOs mapping `PanelState {panel_name,children,info}` and `PanelInfo::{Stack{sizes,axis},Tabs{active_index},Panel(JsonValue),Tiles{metas}}`, rather than hiding structure in a JSON string. Register one fixed `solid-gpui-pane` builder, associate its bridge through `PanelBuildContext.dock_area()`, and resolve `info.id` to a committed slot of the same owner. Never `Box::leak` arbitrary JavaScript pane IDs into `&'static str`. Reject unknown/missing-slot IDs or request asynchronous restoration before loading; upstream unknown-panel placeholders cannot create JavaScript owners. [Dock state][dock-state], [Panel registry][panel-registry]

`DockEvent::LayoutChanged` fires at every tile drag step. JavaScript may receive one coalesced `layoutChanged {sequence}` per native tick, with full dumps via async commands or debounced snapshots. Avoid serializing the whole dock tree on every mouse move. DragDrop payloads must be bridge-created typed `AnyDrag` values, with `DropTarget` converted to a validated enum. [DockEvent][dock-area]

## 7. Settings: typed hierarchy and native getters/setters

`Settings::new(id)` supports `pages`, `sidebar_width`, `sidebar_size_range`, `with_group_variant`, `sidebar_style`, `header_style`, `default_selected_index(SelectIndex{page_ix,group_ix})`, and `Sizable`. Private `SettingsState` retains search/selection through window keyed state without public controlled setters/events. Native sidebar/search behavior can be preserved, but JavaScript selection/search control requires an upstream state API, not changing IDs to simulate updates. [Settings][settings]

`SettingPage::new(title)` supports title/title_suffix closures, icon, description, default_open, resettable, groups, and header_style. Group supports title/description/items/Styled. Item offers `new(title, AnySettingField)` or render(Fn(&RenderOptions,&mut Window,&mut App)→E), plus keywords/disabled/description/layout. A custom item's disabled flag is **only passed through RenderOptions**; it does not automatically disable arbitrary inner JavaScript controls. [SettingPage][setting-page], [SettingGroup][setting-group], [SettingItem][setting-item]

```ts
type SettingsSpec = {
  pages: SettingPageSpec[];
  sidebarWidth: number;
  sidebarMinWidth: number;
  sidebarMaxWidth: number;
  groupVariant: GroupBoxVariant;
  size: Size;
  defaultSelectedIndex: { pageIx: number; groupIx?: number };
};
type SettingPageSpec = {
  id: string;
  title: string;
  description?: string;
  icon?: IconSpec;
  titleSuffixSlot?: number;
  defaultOpen: boolean;
  resettable: boolean;
  groups: SettingGroupSpec[];
};
type SettingGroupSpec = { id: string; title?: string; description?: string; items: SettingItemSpec[] };
type SettingItemSpec =
  | {
      kind: "field";
      id: string;
      title: string;
      description?: string;
      keywords: string[];
      disabled: boolean;
      layout: "horizontal" | "vertical";
      field: SettingFieldSpec;
    }
  | {
      kind: "element";
      id: string;
      slot: number;
      keywords: string[];
      disabled: boolean;
      dirty: boolean;
      resettable: boolean;
    };
type SettingFieldSpec =
  | { kind: "switch" | "checkbox"; value: boolean; defaultValue?: boolean }
  | { kind: "input"; value: string; defaultValue?: string }
  | { kind: "number"; value: number; defaultValue?: number; min: number; max: number; step: number }
  | {
      kind: "dropdown";
      value: string;
      defaultValue?: string;
      scrollable: boolean;
      options: { value: string; label: string }[];
    }
  | { kind: "element"; slot: number; dirty: boolean; resettable: boolean };
type SettingsEvent =
  | { kind: "change"; id: string; value: boolean | string | number; sequence: number }
  | { kind: "resetRequested"; id: string };
```

Retain `BTreeMap<ItemId,Value>` in `SettingsModel`. Native getters read it; setters update it before emitting. Emitting without updating makes the next getter return stale values and controls revert. Controlled JavaScript updates need sequence/ack rules to stop late commits overwriting newer native edits. [SettingField][setting-field]

```rust
// One Settings NativeView owns value_cell; update reconciles by item ID without cross-instance sharing.
// NativeNotify retains a WeakEntity and notifies only a live Settings owner.
type NativeNotify=Rc<dyn Fn(&mut App)>;
fn bool_field(kind: BoolKind, cell: Rc<Cell<bool>>, event: Event<SettingsEvent>, id: String,
              sequence:EventSequence,notify:NativeNotify)
    -> SettingField<bool>
{
    let get = cell.clone();
    let setter = move |value: bool, cx: &mut App| {
        cell.set(value);
        notify(cx);
        event.emit(SettingsEvent::Change { id:id.clone(), value:Value::Bool(value), sequence:sequence.next() });
    };
    match kind {
        BoolKind::Switch => SettingField::switch(move |_| get.get(),setter),
        BoolKind::Checkbox => SettingField::checkbox(move |_| get.get(),setter),
    }
}

fn string_field(spec: &StringFieldSpec, cell: Rc<RefCell<SharedString>>,
                event: Event<SettingsEvent>, id: String,sequence:EventSequence,notify:NativeNotify) -> SettingField<SharedString> {
    let get = cell.clone();
    let setter = move |value: SharedString, cx: &mut App| {
        *cell.borrow_mut() = value.clone();
        notify(cx);
        event.emit(SettingsEvent::Change { id:id.clone(),value:Value::String(value.to_string()),sequence:sequence.next() });
    };
    let mut field = match spec {
        StringFieldSpec::Input { .. } => SettingField::input(move |_| get.borrow().clone(),setter),
        StringFieldSpec::Dropdown { options,scrollable,.. } => {
            let options = options.iter().map(|o| (o.value.clone().into(),o.label.clone().into())).collect();
            if *scrollable { SettingField::scrollable_dropdown(options,move |_| get.borrow().clone(),setter) }
            else { SettingField::dropdown(options,move |_| get.borrow().clone(),setter) }
        }
    };
    if let Some(default) = spec.default_value() { field = field.default_value(SharedString::from(default)); }
    field
}

fn number_field(spec: &NumberSpec, cell: Rc<Cell<f64>>, event: Event<SettingsEvent>,id:String,
                sequence:EventSequence,notify:NativeNotify)
    -> SettingField<f64>
{
    let get = cell.clone();
    let mut field = SettingField::number_input(NumberFieldOptions {
        min:spec.min,max:spec.max,step:spec.step,
    },move |_| get.get(),move |value,cx| {
        cell.set(value);
        notify(cx);
        event.emit(SettingsEvent::Change { id:id.clone(),value:Value::Number(value),sequence:sequence.next() });
    });
    if let Some(default) = spec.default_value { field = field.default_value(default); }
    field
}

fn custom_field(slot: Slot, dirty: Rc<Cell<bool>>, event: Event<SettingsEvent>,id:String)
    -> SettingField<SharedString>
{
    // As with upstream custom rendering, pass disabled to the actual control.
    // A JavaScript SettingsItem helper can provide context from the same disabled prop
    // and set child-control props before commit; opacity does not disable controls.
    SettingField::render(move |_,_,_| slot.clone()).on_reset(move |_| dirty.get(),move |_,_| event.emit(SettingsEvent::ResetRequested { id:id.clone() }))
}

fn settings(spec: &SettingsSpec, model: &SettingsModel, id: ElementId) -> Settings {
    Settings::new(id).sidebar_width(px(spec.sidebar_width))
        .sidebar_size_range(px(spec.sidebar_min_width)..px(spec.sidebar_max_width))
        .with_group_variant(spec.group_variant.native()).with_size(spec.size.native())
        .default_selected_index(SelectIndex { page_ix:spec.default_selected_index.page_ix,group_ix:spec.default_selected_index.group_ix })
        .pages(spec.pages.iter().map(|p| {
            let mut page = SettingPage::new(p.title.clone()).default_open(p.default_open).resettable(p.resettable);
            if let Some(icon) = &p.icon { page = page.icon(icon.native()); }
            if let Some(description) = &p.description { page = page.description(description.clone()); }
            if let Some(slot) = model.title_suffix_slot(p) { page = page.title_suffix(move |_,_| slot.clone()); }
            page.groups(p.groups.iter().map(|g| {
                let mut group = SettingGroup::new();
                if let Some(title) = &g.title { group = group.title(title.clone()); }
                if let Some(description) = &g.description { group = group.description(description.clone()); }
                group.items(g.items.iter().map(|item| model.build_item(item)))
            }))
        }))
}
```

`model.build_item` dispatches switch/checkbox to `SettingItem::new(title,bool_field(...).default_value(...))`, input/dropdown to string_field, number to number_field, element fields to custom_field, and full rows to `SettingItem::render(...).on_reset(...)`. Apply keywords/disabled to all, and description/layout only to field items. Changes use the native owner's `EventSequence`; WeakEntity notifications call `entity.update(cx,|_,cx|cx.notify())`. Updating a cell alone does not trigger GPUI rendering.

`SettingFieldElement` has only render_field(&self,&RenderOptions,&mut Window,&mut App)→Self::Element with Element: IntoElement+'static. Ordinary `Fn` already has a blanket implementation; custom fields need no new trait object each. [`SettingFieldElement`][setting-element]

`RenderOptions` also carries `page_ix/group_ix/item_ix/size/group_variant/layout/disabled`. Supply JavaScript-known size/disabled/indices through component context; native code handles width-dependent container layout. Exposing runtime `RenderOptions.layout` to JavaScript requires an event and later commit, not synchronous JSX evaluation during native render. Applying opacity cannot stand in for disabled semantics.

Structural changes need care: built-in input state keys use page/group/item **indices**. Inserting/reordering SettingsSpec must not transfer old editing state to another item. Lossless arbitrary reordering requires stable upstream item keys; changing the whole Settings ID would only hide the identity problem. [Number-field state][setting-number]

## 8. Seven charts: fixed data with complete builder coverage

`LineChart/AreaChart/BarChart/CandlestickChart` X/B types are discrete labels, defaulting to `String/SharedString`. `Sealed` Y/V permits `f64` and optional rust_decimal Decimal; use `f64` in the bridge. No JavaScript functions cross: project application fields into typed rows when creating props. The shared label helper calculates `% tick_margin`, so **reject zero during commit validation**. [Chart exports/helper][chart], [sealed scale types][scale-sealed]

```rust
#[derive(Clone)]
struct XY { x: String, y: f64 }
#[derive(Clone)]
struct MultiRow { label:String, values:Vec<f64>, label_slot:Option<Slot> }
#[derive(Clone)]
struct BarRow { band:String, value:f64, label:String, fill:Background }
#[derive(Clone)]
struct Candle { x:String, open:f64, high:f64, low:f64, close:f64 }
#[derive(Clone)]
struct Slice { value:f32, color:Hsla, label:String, leader:Hsla,
               inner_radius:Option<f32>, outer_radius:Option<f32> }
#[derive(Clone)]
struct FlowNode { label:String,color:Hsla,value_label:Option<String>,labels:Vec<FlowLabel> }
```

DTOs need no Rust-only Hsla/Background/Slot: encode ColorSpec/`BackgroundSpec` and slot indices, then validate/compile at mount into native data. `BackgroundSpec` should cover solid and two-stop linear gradients with angle. Bounds-dependent gradients use an explicit `barLocal/chartRange` enum and upstream `chart_to_bar` inside Rust. No unknown JSON fields, `function source`, or `eval` are needed.

```rust
fn line_chart(p: &LineProps,id:ElementId) -> AnyElement {
    let mut c = LineChart::new(p.data.clone()).x(|d:&XY| d.x.clone()).y(|d:&XY|d.y)
        .name(p.name.clone()).stroke(p.stroke.native()).tick_margin(p.tick_margin)
        .x_axis(p.x_axis).grid(p.grid);
    c = match p.curve { Curve::Natural=>c.natural(),Curve::Linear=>c.linear(),Curve::StepAfter=>c.step_after() };
    if p.dot { c=c.dot(); }
    if p.interactive { c=c.id(id); }
    c.into_any_element()
}

fn area_chart(p:&AreaProps,id:ElementId)->AnyElement {
    let mut c = AreaChart::new(p.data.clone()).x(|d:&MultiRow|d.label.clone())
        .tick_margin(p.tick_margin).x_axis(p.x_axis).grid(p.grid);
    for (index,s) in p.series.iter().enumerate() {
        c=c.y(move |d:&MultiRow|d.values[index]).name(s.name.clone())
            .stroke(s.stroke.native()).fill(s.fill.native());
        c=match s.curve { Curve::Natural=>c.natural(),Curve::Linear=>c.linear(),Curve::StepAfter=>c.step_after() };
    }
    if p.interactive { c=c.id(id); }
    c.into_any_element()
}

fn bar_chart(p:&BarProps,id:ElementId)->AnyElement {
    let mut c=BarChart::new(p.data.clone()).band(|d:&BarRow|d.band.clone()).value(|d:&BarRow|d.value)
        .name(p.name.clone()).label(|d:&BarRow|d.label.clone())
        .label_axis(p.label_axis).value_axis(p.value_axis).value_tick_count(p.value_tick_count)
        .tick_margin(p.tick_margin).grid(p.grid).alignment(p.alignment.native())
        .corner_radii(p.corner_radii.native());
    c=match &p.gradient {
        None=>c.fill(|d:&BarRow,_,_,_|d.fill),
        Some(spec)=>{
            let spec=spec.clone();
            c.fill_gradient(move |_,range,to_bar| {
                let (start,end)=match spec.coordinates {
                    GradientCoordinates::BarLocal=>(spec.start,spec.end),
                    GradientCoordinates::ChartRange=>(to_bar(*range.start()),to_bar(*range.end())),
                };
                [linear_color_stop(spec.color0.native(),start),linear_color_stop(spec.color1.native(),end)]
            })
        }
    };
    if p.interactive { c=c.id(id); }
    c.into_any_element()
}

fn candle_chart(p:&CandleProps)->AnyElement {
    CandlestickChart::new(p.data.clone()).x(|d:&Candle|d.x.clone())
        .open(|d:&Candle|d.open).high(|d:&Candle|d.high).low(|d:&Candle|d.low).close(|d:&Candle|d.close)
        .tick_margin(p.tick_margin).body_width_ratio(p.body_width_ratio)
        .x_axis(p.x_axis).grid(p.grid).into_any_element()
}

fn pie_chart(p:&PieProps)->AnyElement {
    let mut c=PieChart::new(p.data.clone()).value(|d:&Slice|d.value).color(|d:&Slice|d.color)
        .inner_radius(p.inner_radius).outer_radius(p.outer_radius).pad_angle(p.pad_angle)
        .label_color(p.label_color.native()).label_gap(p.label_gap);
    if p.labels { c=c.label(|d:&Slice|d.label.clone().into()).label_line_color(|d:&Slice|d.leader); }
    // Per-slice radius mode requires validated radii for every slice.
    if p.per_slice_radii {
        c=c.inner_radius_fn(|a|a.data.inner_radius.unwrap())
            .outer_radius_fn(|a|a.data.outer_radius.unwrap());
    }
    c.into_any_element()
}

fn radar_chart(p:&RadarProps,id:ElementId)->AnyElement {
    let mut c=RadarChart::new(p.data.clone())
        .label(|d:&MultiRow|match &d.label_slot {
            Some(slot)=>RadarLabel::Element(slot.clone().into_any_element()),
            None=>RadarLabel::Text(d.label.clone().into()),
        }).label_color(p.label_color.native()).label_gap(p.label_gap)
        .outer_radius(p.outer_radius).grid(p.grid).grid_levels(p.grid_levels);
    for (index,s) in p.series.iter().enumerate() {
        c=c.value(move |d:&MultiRow|d.values[index]).name(s.name.clone())
            .stroke(s.stroke.native()).fill(s.fill.native());
    }
    if let Some(max)=p.max_value { c=c.max_value(max); }
    if p.dot { c=c.dot(); }
    if p.interactive { c=c.id(id); }
    c.into_any_element()
}

fn sankey_chart(p:&SankeyProps)->AnyElement {
    let links=p.links.iter().map(|l|SankeyLink::new(l.source,l.target,l.value));
    let mut c=SankeyChart::new(p.nodes.clone(),links)
        .node_width(p.node_width).node_padding(p.node_padding).node_align(p.align.native())
        .iterations(p.iterations).value_scale(p.value_scale.native())
        .node_corner_radius(px(p.node_corner_radius)).node_color(|d:&FlowNode|d.color)
        .node_label(|d:&FlowNode|d.label.clone().into())
        .link_opacity(p.link_opacity).min_link_width(p.min_link_width).label_gap(p.label_gap);
    if p.show_value_labels {
        let format=p.value_format.clone();
        c=c.value_label(move |d:&FlowNode,v|d.value_label.clone()
            .unwrap_or_else(||format.format(v)).into());
    }
    if p.custom_labels {
        c=c.labels(|d:&FlowNode,_|d.labels.iter().map(|l| {
            SankeyLabel::new(l.text.clone()).color(l.color).font_size(l.font_size)
        }).collect());
    }
    c.into_any_element()
}
```

Each renderer's p includes its builder capabilities. Give each chart a corresponding DTO instead of an arbitrary property dictionary. Area/Radar series `name/stroke/fill/curve` values and row lengths must align with the series array. Missing series cannot be filled with zeros without changing domain data. Line/Area/Bar/Radar need stable IDs for native hover tooltips. Candlestick/Pie/Sankey expose no interactive ID/tooltip setter in this revision. [Line][line], [Area][area], [Bar][bar], [Candle][candle], [Pie][pie], [Radar][radar], [Sankey][sankey]

Radar's Fn(&T)→RadarLabel receives no Window/App, so `Slot.clone().into_any_element()` is the correct deferred entrypoint. Native prepaint later lays out the label subtree; label cannot call JavaScript synchronously. `Plot::prepaint` handles label layout, while `paint` may not perform layout. [Plot trait][plot]

Charts have no persistent Entity state: retain stable Host Node IDs and rebuild lightweight plots for prop/theme changes through `IntoPlot` layout/paint. Large datasets can compile during mount/update and use `Rc<Row>` to reduce String cloning, without retransmitting all data on mouse moves. Sankey topology/relaxation depends on data and bounds; any cache key must include data revision, bounds, nodeWidth/padding/align/iterations/valueScale, and text styles.

Validate finite numbers, tickMargin≥1, bounded dimensions, nonnegative pie values/radii with inner≤outer, OHLC low≤min(open,close)≤max(open,close)≤high, valid Sankey source/target, nonnegative finite flow, and acyclic graphs with concrete topology errors. Bound iterations so user DTOs cannot cause arbitrary relaxation work in paint. [Sankey mathematics][sankey-math]

## 9. Complete Plot primitive coverage

`Plot` requires `paint(&mut self,Bounds<Pixels>,&mut Window,&mut App)` and may override prepaint()→Vec<AnyElement>, id()→Option<ElementId>, tooltip_state, and tooltip. `#[derive(IntoPlot)]` supplies IntoElement. Primitives are not ordinary JSX children. Expose `Plot { primitives: PlotPrimitive[] }` or typed declarations such as `Plot.Line` compiled into that DTO; both syntaxes converge on one native painter. [`Plot`][plot]

Complete representable DTO variants:

```ts
type PlotPrimitive =
  | {
      kind: "axis";
      x?: number;
      y?: number;
      xAxis: boolean;
      yAxis: boolean;
      xLabels: AxisTextSpec[];
      yLabels: AxisTextSpec[];
      xLabelSide: "start" | "end";
      yLabelSide: "start" | "end";
      stroke: ColorSpec;
    }
  | { kind: "grid"; x: number[]; y: number[]; stroke: ColorSpec; dash: number[] }
  | { kind: "labels"; items: TextSpec[] }
  | {
      kind: "line";
      points: PointSpec[];
      stroke: BackgroundSpec;
      strokeWidth: number;
      curve: Curve;
      dots: boolean;
      dotSize: number;
      dotFill: ColorSpec;
      dotStroke?: ColorSpec;
    }
  | { kind: "area"; points: PointSpec[]; baseline: number; fill: BackgroundSpec; stroke: BackgroundSpec; curve: Curve }
  | {
      kind: "bar";
      rows: { cross: number | null; base: number; value: number | null; fill: BackgroundSpec; labels: TextSpec[] }[];
      alignment: "top" | "bottom" | "left" | "right";
      bandWidth: number;
      cornerRadii: CornersSpec;
    }
  | {
      kind: "radialLine";
      points: { angle: number | null; radius: number | null }[];
      closed: boolean;
      fill: BackgroundSpec;
      stroke: BackgroundSpec;
      strokeWidth: number;
      dots: boolean;
      dotSize: number;
      dotFill: ColorSpec;
      dotStroke?: ColorSpec;
    }
  | {
      kind: "arc";
      startAngle: number;
      endAngle: number;
      padAngle: number;
      innerRadius: number;
      outerRadius: number;
      fill: ColorSpec;
    };
type PointSpec = { x: number | null; y: number | null };
type TextSpec = {
  text: string;
  x: number;
  y: number;
  color: ColorSpec;
  fontSize: number;
  fontWeight: number;
  align: "left" | "center" | "right";
};
type AxisTextSpec = {
  text: string;
  tick: number;
  color: ColorSpec;
  fontSize: number;
  align: "left" | "center" | "right";
};
```

`Line/Area/RadialLine` accessors use `Option<f32>`; DTO null means a missing point, never zero. Configure Axis x/y and labelSide **before x/y_label**, which computes text position immediately. [Axis][axis], [Shapes][shapes]

```rust
#[derive(IntoPlot)]
struct NativePlot { primitives:Vec<NativePrimitive> }
impl Plot for NativePlot {
    fn paint(&mut self,bounds:Bounds<Pixels>,window:&mut Window,cx:&mut App) {
        for primitive in &self.primitives {
            match primitive {
                NativePrimitive::Axis(p)=>{
                    let mut a=PlotAxis::new().stroke(p.stroke).x_axis(p.x_axis).y_axis(p.y_axis)
                        .x_label_side(p.x_label_side).y_label_side(p.y_label_side);
                    if let Some(x)=p.x { a=a.x(x); }
                    if let Some(y)=p.y { a=a.y(y); }
                    a.x_label(p.x_labels.iter().map(axis_text)).y_label(p.y_labels.iter().map(axis_text))
                        .paint(&bounds,window,cx);
                }
                NativePrimitive::Grid(p)=>Grid::new().x(p.x.clone()).y(p.y.clone()).stroke(p.stroke)
                    .dash_array(&p.dash).paint(&bounds,window),
                NativePrimitive::Labels(p)=>PlotLabel::new(p.iter().map(plot_text).collect()).paint(&bounds,window,cx),
                NativePrimitive::Line(p)=>{
                    let mut s=Line::new().data(p.points.clone()).x(|d:&PointSpec|d.x).y(|d:&PointSpec|d.y)
                        .stroke(p.stroke).stroke_width(p.stroke_width).stroke_style(p.curve)
                        .dot_size(p.dot_size).dot_fill_color(p.dot_fill);
                    if p.dots { s=s.dot(); }
                    if let Some(c)=p.dot_stroke { s=s.dot_stroke_color(c); }
                    s.paint(&bounds,window);
                }
                NativePrimitive::Area(p)=>Area::new().data(p.points.clone()).x(|d:&PointSpec|d.x)
                    .y0(p.baseline).y1(|d:&PointSpec|d.y).fill(p.fill).stroke(p.stroke)
                    .stroke_style(p.curve).paint(&bounds,window),
                NativePrimitive::Bar(p)=>Bar::new().data(p.rows.clone()).alignment(p.alignment)
                    .cross(|d:&NativeBarRow|d.cross).base(|d:&NativeBarRow|d.base)
                    .value(|d:&NativeBarRow|d.value).band_width(p.band_width)
                    .fill(|d:&NativeBarRow,_,_|d.fill).label(|d:&NativeBarRow,origin|{
                        // The label callback origin is the bar anchor; DTO text coordinates are relative to it.
                        d.labels.iter().map(|l|plot_text_at(l,origin)).collect()
                    }).corner_radii(p.corner_radii).paint(&bounds,window,cx),
                NativePrimitive::RadialLine(p)=>{
                    let mut s=RadialLine::new().data(p.points.clone())
                        .angle(|d:&RadialPoint,_|d.angle).radius(|d:&RadialPoint,_|d.radius)
                        .fill(p.fill).stroke(p.stroke).stroke_width(p.stroke_width)
                        .dot_size(p.dot_size).dot_fill_color(p.dot_fill);
                    if p.closed { s=s.closed(); }
                    if p.dots { s=s.dot(); }
                    if let Some(c)=p.dot_stroke { s=s.dot_stroke_color(c); }
                    s.paint(&bounds,window);
                }
                NativePrimitive::Arc(p)=>{
                    let arc_data=ArcData {data:&(),index:0,value:0.,start_angle:p.start_angle,end_angle:p.end_angle,pad_angle:p.pad_angle};
                    Arc::new().inner_radius(p.inner_radius).outer_radius(p.outer_radius)
                        .paint(&arc_data,p.fill,None,None,&bounds,window);
                }
            }
        }
    }
}
```

`Plot tooltip` DTOs can map `tooltip::Tooltip::new(cursor,within)` with title/row/gap/cross_line/dots/appearance/children, `CrossLine::new(point)` with band/horizontal/both/height/width/span/h_span, and `Dot::new(point)` with size/stroke/fill. All are RenderOnce and can wrap as elements, with explicitly plot-local coordinates and overlay-only usage. Custom tooltip hit testing uses native index/geometry DTOs, never synchronous JavaScript. [`Plot tooltip`][plot-tooltip]

Expose computation objects as typed ViewCommands or module functions returning reusable data:

```rust
// Scale domain/range, ticks, and nearest are computation, not empty visual JSX components.
let linear=ScaleLinear::new(domain_f64,range_f32);
let ticks:Vec<Option<f32>>=values.iter().map(|v|linear.tick(v)).collect();
let point=ScalePoint::new(domain_strings,range_f32);
let nearest=point.least_index(cursor);
let band=ScaleBand::new(domain_strings,range_f32).padding_inner(inner).padding_outer(outer);
let width=band.band_width();
let ordinal=ScaleOrdinal::new(domain_strings,range_colors);
let color=ordinal.map(&value); // unknown(...) can be configured explicitly.

let pie=Pie::new().value(|d:&f32|*d).start_angle(start).end_angle(end).pad_angle(pad);
let arcs=pie.arcs(&values); // Return index/value/startAngle/endAngle/padAngle DTOs without &'a T.

let stacked=Stack::new().data(rows).keys(keys)
    .value(|row:&BTreeMap<String,f32>,key|row.get(key).copied()).series();
// DTO: [{key,index,points:[{y0,y1,data:row}]}]。

let sankey=Sankey::new().node_width(width).node_padding(padding).node_align(align)
    .iterations(iterations).value_scale(scale).size(canvas_w,canvas_h);
let graph=sankey.layout(node_count,&links)?; // Preserve typed graph-validation errors.
```

`Sankey::topology(node_count,&links)` can be cached separately from `layout_from(graph)`. sankey_link_path(&SankeyNodeLayout,&SankeyNodeLayout,&SankeyLinkLayout,min_width:f32,origin:Point<Pixels>)→Option<Path<Pixels>> returns a native Path that does not cross the wire; send graph geometry only. `Plot::polygon` likewise returns a native painter helper, not a serializable component. [Scale][scale], [Pie/Stack/Arc][shapes], [Sankey mathematics][sankey-math]

Computation result DTOs can preserve the complete upstream public data:

```ts
type ArcDataDto = { index: number; value: number; startAngle: number; endAngle: number; padAngle: number };
type StackDto<T> = { key: string; index: number; points: { y0: number; y1: number; data: T }[] }[];
type SankeyGraphDto = {
  nodes: {
    index: number;
    value: number;
    depth: number;
    height: number;
    layer: number;
    x0: number;
    x1: number;
    y0: number;
    y1: number;
    sourceLinks: number[];
    targetLinks: number[];
  }[];
  links: {
    index: number;
    source: number;
    target: number;
    value: number;
    y0: number;
    y1: number;
    width: number;
    sourceWidth: number;
    targetWidth: number;
  }[];
};
type SankeyErrorDto = { kind: "missingNode"; index: number } | { kind: "circularLink" };
type PlotTooltipDto = {
  cursor: { x: number; y: number };
  within: { width: number; height: number };
  title?: string;
  gap: number;
  appearance: boolean;
  rows: { color: ColorSpec; label: string; value: string }[];
  crossLine?: {
    point: { x: number; y: number };
    direction: "vertical" | "horizontal" | "both";
    bandThickness?: number;
    verticalSpan?: { start: number; length: number };
    horizontalSpan?: { start: number; length: number };
  };
  dots: { point: { x: number; y: number }; size: number; stroke: ColorSpec; fill: ColorSpec }[];
};
```

Sankey graph `value` is in layout `value`-space; with `valueScale:"sqrt"`, it is not raw domain flow. Original links retain raw values, and chart `value_label` receives raw throughput. Name and document these separately instead of displaying scaled graph.`value` as domain flow. [Sankey mathematics][sankey-math]

## 10. Essential acceptance cases

1. Open Dialog A then B. Unmounting A preserves B and its closure/focus restoration; retiring the Surface leaves no backdrop or focus trap. Stale async resolve from A cannot close B.
2. Update a Popover's JavaScript content signal while open and verify new text across multiple frames, then close/reopen. Retained closures cannot render old children after owner removal.
3. Exercise submenu left/right/Escape navigation, disabled items, and correct focus after selection. Open-menu prop updates preserve PopupMenu identity.
4. Drag Dock tabs into splits/tiles while preserving native Entities and JavaScript child state. dump/load includes pane IDs/data/tile bounds. Closing emits Removed; moving does not.
5. Exercise custom Settings disabled/reset/async acknowledgement, editing after search, and identity through index changes.
6. Use real data and native screenshots for all seven charts, correct Area/Radar series alignment, Radar JSX labels, and Bar gradients in both coordinate spaces. Reject tickMargin=0 before commit and return concrete Sankey cycle/invalid-link errors.

These are key checks for new bridge behavior, not mirrored tests for every builder setter. UI/focus/painting acceptance requires a real native surface; this report did not execute those checks.

## Source index

[dialog]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/dialog/dialog.rs:237
[dialog-render]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/dialog/dialog.rs:480
[alert-dialog]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/dialog/alert_dialog.rs:68
[sheet]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/sheet.rs:41
[root]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/root.rs:239
[window-ext]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/window_ext.rs:12
[popover]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/popover.rs:109
[hover-card]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/hover_card.rs:18
[tooltip]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/tooltip.rs:34
[menu]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/menu/popup_menu.rs:32
[menu-linking]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/menu/popup_menu.rs:1386
[menu-rebuild]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/menu/popup_menu.rs:727
[context-menu]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/menu/context_menu.rs:16
[dropdown-button]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/button/dropdown_button.rs:25
[dropdown-menu]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/menu/dropdown_menu.rs:11
[app-menu]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/menu/app_menu_bar.rs:27
[global-state]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/global_state.rs:88
[dock]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/dock/mod.rs:53
[panel]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/dock/panel.rs:73
[base-panel]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/dock/panel.rs:21
[dock-layout]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/dock/layout/builder.rs:23
[dock-area]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/dock/dock_area.rs:39
[dock-state]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/dock/state.rs:10
[panel-registry]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/dock/registry.rs:111
[tiles-state]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/dock/tiles_state.rs:116
[settings]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/setting/settings.rs:34
[setting-page]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/setting/page.rs:22
[setting-group]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/setting/group.rs:16
[setting-item]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/setting/item.rs:22
[setting-field]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/setting/fields/mod.rs:138
[setting-element]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/setting/fields/element.rs:9
[setting-number]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/setting/fields/number.rs:73
[chart]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/chart/mod.rs:9
[line]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/chart/line_chart.rs:23
[area]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/chart/area_chart.rs:23
[bar]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/chart/bar_chart.rs:31
[candle]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/chart/candlestick_chart.rs:18
[pie]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/chart/pie_chart.rs:21
[radar]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/chart/radar_chart.rs:74
[sankey]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/chart/sankey_chart.rs:74
[plot]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/plot/mod.rs:23
[scale]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/plot/scale.rs:1
[scale-sealed]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/plot/scale/sealed.rs:1
[axis]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/plot/axis.rs:67
[shapes]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/plot/shape.rs:1
[sankey-math]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/plot/shape/sankey.rs:190
[plot-tooltip]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/plot/tooltip.rs:10
