# gpui-component 组合组件与 Plot 接入研究

研究基线：`gpui-component` checkout `928c3eb776a3d733d9b771f7dea27a6a79242ced`，2026-09-06。本文仅为实现研究，没有修改产品实现，也没有把骨架标为编译通过。以下 `C/`、`B/` 分别指此 checkout 的 `crates/component/src/`、`crates/base/src/`；文末给出可点击的源码入口。`NativeView` 与 renderer 文件在主任务中正在修改，下面先明确需要满足的接口，再给实际上游调用骨架。

## 1. 必须先落实的原生边界

最初读到的 `NativeView` 只有 `mount(props,event,window,cx)`、`update(props,window,cx)`；`ComponentDefinition::view` 声明 `children:false`，`ViewInstance::render` 直接丢弃 child iterator。`with_children(true)` 只放开 validator，不能让 retained view 获得可复用的子树。这是组合组件接入缺口，不是各组件自己加 `ParentElement` 就能修好。[本仓库 NativeView](../../crates/solid-gpui/src/native/component.rs)

随后工作树已出现 `ExtensionChildren` / `ExtensionContent`：它们保存 weak SolidRoot 与 event route，读取**当前已提交的 Rust host tree**重新构建 `AnyElement`，并检查 route、surface、epoch。这个方向可以供下列 closure 使用。不能捕获某一帧 `Vec<AnyElement>` 再从 `Fn` 内 `take()`；native callback 可能在以后的许多帧调用。`ExtensionContent` 自身是可 clone 的 render wrapper，适合交给不带 `App` 参数的 chart label closure。[原生子树投影](../../crates/solid-gpui/src/renderer/extensions.rs)

本研究骨架约定：

```rust
// 来自已验证 host 子树。这里不是任意可伪造的 JS 数字句柄。
type Slot = solid_gpui::ExtensionContent;

#[derive(Clone)]
struct Slots {
    content: Slot,
    trigger: Option<Slot>,
    title: Option<Slot>,
    description: Option<Slot>,
    footer: Option<Slot>,
}

// 所有 Props/Event/Command DTO 使用 Deserialize/Serialize/TS 生成契约。
// 所有可选数据必须在 commit validator 中验证类型、范围和引用完整性。
// 将具名 slot 变为 host group 是生成器的职责；Rust 不接收 JSX/Solid owner/JS closure。
```

Slot 必須与组件实例而非全局 node ID 绑定。主节点移除、类型替换、epoch 退休时 revoke；保留引用的 overlay 不能恢复旧节点。Slot group 索引必须从契约中生成，并验证其结构；不能允许 Dialog 把任意宿主节点作为 content。局部 children commit 后应通知保存该 slot 的原生 owner / 根层，防止 retained `Entity` 在自己的 dirty 标记未改变时复用旧帧。

Native event closure 只 `event.emit(Dto)`；所有同步 getter / bool 决策只读取 Rust DTO/state。比如 Dialog 的 `on_ok -> bool` 不能同步等待 JS：显式 `closeOnOk` 可以直接返回；异步业务校验用 `closeOnOk:false`，先发 `okRequested {requestId}`，然后 JS 异步调用 `resolve({requestId,close})`。过期请求、已关闭层、owner revoke 都要拒绝。

## 2. 组件分类：哪些能独立包装，哪些不能

| 类型 | 928c3eb 的真实形式 | 接入形式 |
|---|---|---|
| DialogContent/Header/Title/Description/Footer/Action/Close | `RenderOnce + ParentElement`，`new()` | 可以逐一独立 element wrapper，传当帧 children |
| Dialog / AlertDialog / Sheet | `RenderOnce`，真实 overlay 由 `Root` 保存 builder/focus/layer | retained owner + slots + **有 owner 的 overlay API**；见下一节 |
| Popover / HoverCard | `RenderOnce`，`content` 为 native `Fn`，state 由 base keyed element 保存 | stateless render wrapper 加稳定 ID、复用 slot；需要命令时 retained view 保存 focus/state |
| Tooltip | `Render`，`build(window,cx) -> AnyView`；trigger 使用 `ManagedTooltipExt` | wrapper + tooltip content slot，不能只把 Tooltip 当常规 children |
| PopupMenuItem / PopupMenu | 特定 enum builder / `Entity<PopupMenu>` | `MenuSpec` DTO 编译为原生 builder；custom element item 引用 slot |
| ContextMenu | `ContextMenu<E>` 包在 `InteractiveElement + ParentElement + Styled` 上；`.menu` 是私有方法 | 使用 `.context_menu(builder)` 扩展，不能从外部 `ContextMenu::new(...).menu(...)` |
| DropdownButton | `.button(Button)`，不是 `AnyElement` | `ButtonSpec` typed prop，再 `.dropdown_menu(builder)`；任意 JSX children 不能替代 native Button 类型 |
| AppMenuBar | `AppMenuBar::new(cx) -> Entity`，读取应用 `OwnedMenu` global，`reload` | retained view + 应用菜单模型；不是局部任意 popup menu children |
| DockArea | `Entity<DockArea>`，状态/布局来自 `gpui-base`；用 `DockSkin` 装外观 | retained dock owner、typed layout DTO、native `JsPane` entities |
| TabPanel / Tiles | **这个 revision 没有同名公开组件构造器**；真实公开类型是 `DockLayout::tabs/tiles`、`TabGroup`、`TilesState` | 作为 DockLayout 的变体暴露；不要虚构 `TabPanel::new` 或 `Tiles::new` |
| Settings / SettingPage / SettingGroup / SettingItem / SettingField | Settings 收 `SettingPage`，Page 收 `SettingGroup`，Group 收 `SettingItem`；后三者不是普通 render elements | 一组 typed DTO；或者专用 typed-child tree 在宿主编译成 builder，不能走 AnyElement |
| 7 Charts | `IntoPlot` 的泛型结构体，字段 accessor 为 Rust `Fn` | 每种专用 DTO→固定 Rust datum→纯 native accessor |
| PlotAxis/Grid/PlotLabel/Line/Area/Bar/RadialLine/Arc | 多数只有 `.paint`，不实现普通 IntoElement | 统一 `Plot` renderer 包一层，以 typed primitive DTO 构建；不能直接 `child(PlotAxis)` |
| Scale/Pie/Stack/Sankey layout | 数学对象；`tick/arcs/series/layout` | typed data computation commands 或 Plot 内部纯函数；不是视觉组件 |

来源：[Dialog 模块][dialog]、[PopupMenu][menu]、[Dock 导出][dock]、[Settings][settings]、[Plot trait][plot]。

## 3. Dialog / AlertDialog / Sheet：完整 lifecycle 先于外观

### 3.1 当前公开 API 的硬边界

`Dialog::new(&mut App)`；builder：`trigger(IntoElement)`, `content(Fn(DialogContent,&mut Window,&mut App)->DialogContent)`, `title`, `footer`, `button_props`, `on_ok`, `on_cancel`, `on_close`, `close_button`, `margin_top`, `width`（也有同义 `w`）, `max_w`, `overlay`, `overlay_closable`, `keyboard`，以及 Styled/ParentElement。`header` 是 `pub(crate)`，外部不要调用。[Dialog][dialog]

`AlertDialog::new(&mut App)`；`confirm`, `trigger`, `content`, `footer`, `icon`, `title`, `description`, `button_props`, `width`, `show_cancel`, `close_button`, `keyboard`, `on_ok/cancel/close`。`icon/title/description/button_props` 在 trigger/content 模式有 `debug_assert_no_trigger`；两种 DTO 模式应互斥。`overlay_closable` 已 deprecated 且不生效：AlertDialog 永不通过 backdrop 关闭。不能暴露一个看似可写但永不生效的 JS prop。[AlertDialog][alert-dialog]

`Sheet::new(&mut Window,&mut App)`；`title`, `footer`, `size(DefiniteLength)`, `resizable`, `overlay`, `overlay_closable`, `on_close`、Styled/ParentElement。placement 本身是 crate-private 字段，必须通过 `WindowExt::open_sheet_at(Placement,...)` 选择。`SheetSettings.margin_top` 是 root 全局配置。[Sheet][sheet]

`AlertDialog::on_close` 还有当前源码缺陷：它写 `self.base.button_props.on_close`，但 imperative `build_surface` 又以 `self.button_props` 覆盖 base button props；trigger 分支专门复制了 on_close，imperative 分支没有。实现前应在 `build_surface` 复制该 callback，或让 owned Root 统一报告 close lifecycle；否则 `open_alert_dialog(...on_close(...))` 回调会丢失。[AlertDialog][alert-dialog]

`WindowExt` 只公开 `open_dialog(builder)`、`open_alert_dialog(builder)`、`close_dialog()`（pop 最后一层）、`close_all_dialogs()`，以及单例 `open_sheet_at` / `close_sheet`。没有移除指定 layer 的 owner/token。[WindowExt][window-ext]

不能为了 controlled `open` 直接在 `NativeView::render` 返回 `Dialog::new(cx)`：其 `RenderOnce` 读取 `Root.active_dialogs.len()` 决定 topmost，并通过 root 执行 close；Root 在 open 时保存稳定 focus handle 和 selection scope。脱离 Root 的 dialog 没有等价 focus trap / stack 行为。[Dialog render][dialog-render]、[Root overlay][root]

仅 weak slot 还不够：触发 Dialog 后卸载 JSX owner，Root 保留的 closure 仍可能绘制空 modal/backdrop 并占有 focus。处理嵌套 owner、关闭非顶部层、同时多个 SolidRoot，不能猜当前最后一个就是自己的层。

### 3.2 所需 owner API 与 DTO

下面 API 是**需要添加到 gpui-component Root 的最小真实生命周期能力**，并非该 revision 已提供：

```rust
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct OverlayOwner { surface: u64, epoch: u64, node: u32, generation: u64 }

enum ModalKind { Dialog, Alert }
struct OverlayToken { owner: OverlayOwner, serial: u64 }

// Root 保存 owner/token + builder + focus + selection_scope。
// open/update/close 都返回明确结果，不用 close_all 来清理单个 JS owner。
fn open_dialog_owned(owner: OverlayOwner, builder: DialogBuilder) -> OverlayToken;
fn update_dialog_owned(token: &OverlayToken, builder: DialogBuilder) -> Result<(), Retired>;
fn close_dialog_owned(token: &OverlayToken) -> Result<(), Retired>;
fn open_sheet_owned(owner: OverlayOwner, placement: Placement, builder: SheetBuilder)
    -> OverlayToken;
fn update_sheet_owned(token: &OverlayToken, placement: Placement, builder: SheetBuilder)
    -> Result<(), Retired>;
fn close_sheet_owned(token: &OverlayToken) -> Result<(), Retired>;
```

Root 内删除中间 dialog 时，更新下一层 `previous_focused_handle` 为被删层的 predecessor；只在删除 top layer 时恢复 focus。替换 builder 保留 focus_handle/selection_scope/layer identity。单例 Sheet 被另一 owner 替换时发 `closed {reason:"replaced"}` 给旧 owner，并令旧 token 退休。Surface dispose 清除此 surface 的 token，不影响其他 surface。这个功能应在 Root 实现，不用外部反射或复制其私有 Vec。

JS contract 建议：

```ts
type DialogProps = {
  open: boolean; title?: string; width?: number; maxWidth?: number;
  marginTop?: number; closeButton?: boolean; overlay?: boolean;
  overlayClosable?: boolean; keyboard?: boolean;
  okText?: string; cancelText?: string; showCancel?: boolean;
  okVariant?: ButtonVariant; cancelVariant?: ButtonVariant;
  closeOnOk: boolean; closeOnCancel: boolean;
  // JSX slots: trigger?, content, title?, footer?
};
type AlertDialogProps = {
  open: boolean; width?: number; closeButton?: boolean; keyboard?: boolean;
  contentMode: "standard" | "custom";
  title?: string; description?: string; icon?: IconSpec;
  showCancel?: boolean; okText?: string; cancelText?: string;
  okVariant?: ButtonVariant; cancelVariant?: ButtonVariant;
  closeOnOk: boolean; closeOnCancel: boolean;
  // standard 和 custom content slot 不可混用
};
type SheetProps = {
  open: boolean; placement: "left" | "right" | "top" | "bottom";
  size: {unit:"px"; value:number} | {unit:"relative"; value:number};
  title?: string; resizable?: boolean; overlay?: boolean; overlayClosable?: boolean;
  // content/title/footer JSX slots
};
type ModalEvent =
 | {kind:"openRequested"}
 | {kind:"okRequested" | "cancelRequested"; requestId:number}
 | {kind:"closed"; reason:"ok"|"cancel"|"dismiss"|"programmatic"|"replaced"};
```

### 3.3 Native builder 骨架

下面 `p` 是经默认值填充、字段验证后的 Rust props；`event` 由 NativeView 保存。closure 读取最新 `Rc<RefCell<DialogModel>>`，不要只捕获首次 props。

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
    // 在 button_props 配置之后设置 callbacks，避免被标准按钮配置覆盖。
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

Dialog/Alert 的 request counter 为每个 native owner 创建一个 `EventSequence`。上面 builder 的通用 on_close 只能表达 dismiss；最终实现应由 owned Root 统一报告精确 `CloseReason`，删除重复 builder close 事件：同步 ok/cancel 返回 true 前记录该原因，resolve 命令记录 programmatic/对应 action 原因，Root 执行关闭后只发一次 closed。这同时绕开当前 imperative AlertDialog 丢 close callback 的上游缺陷。

`mount/update`：保存 model、slots、optional token；`open:false→true` 调 owned open，`true→true` 更新 builder，`true→false` close token。`resolve` 只接受当前 pending request。unmount/window close/epoch 退休时 native adapter 显式 dispose token；不能依赖 Rust `Drop` 取得 Window/App。`Render` 只绘制 trigger slot；native trigger click emits `openRequested` 或由明确 uncontrolled 模式立刻 open。生命周期方法的 owner API 尚未实现时，Dialog 全功能接入不能宣称完成。

## 4. Popover / HoverCard / Tooltip

`Popover::trigger` 要求 `Selectable + IntoElement`；`Slot` 的 wrapper 并不自动实现 Selectable。可以做一个有意义的 native selectable trigger shim，选中态写入 wrapper 样式或 ARIA，内部保留 JS children，不需要传一个原生 Button DTO 才能使用任意 trigger。[Popover][popover]

```rust
#[derive(IntoElement)]
struct SlotTrigger { slot: Slot, selected: bool }
impl Selectable for SlotTrigger {
    fn selected(mut self, selected: bool) -> Self { self.selected = selected; self }
    fn is_selected(&self) -> bool { self.selected }
}
impl RenderOnce for SlotTrigger {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().child(self.slot) // 将 selected 映射为宿主约定的选中语义/样式。
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

Popover props 覆盖 `anchor`（8 个 Anchor 位置）、`mouseButton`、`defaultOpen`、`open?`、`appearance`、`overlayClosable`、trigger style；focus track 使用 native view 自己保存 `FocusHandle`，不要把 handle 传给 JS。HoverCard 只有 `openDelay/closeDelay/anchor/appearance/onOpenChange`；当前组件无 controlled `.open`，不要生成假 prop。Tooltip `action(&dyn Action, Option<&str>)` 如需暴露，应是预注册 native Action 的字符串/DTO 编译，不是任意 JS函数。[HoverCard][hover-card]、[Tooltip][tooltip]

## 5. Menu / ContextMenu / DropdownButton / AppMenuBar

### 5.1 完整菜单 DTO 与原生编译器

```ts
type MenuSpec = {
  items: MenuItem[]; minWidth?: number; maxWidth?: number; maxHeight?: number;
  scrollable: boolean; checkSide: "left"|"right"; externalLinkIcon: boolean;
};
type MenuItem =
 | {kind:"separator"}
 | {kind:"label"; label:string}
 | {kind:"item"; id:string; label:string; icon?:IconSpec; disabled?:boolean; checked?:boolean; action?:NativeActionSpec}
 | {kind:"link"; label:string; href:string; icon?:IconSpec; disabled?:boolean}
 | {kind:"element"; id:string; slot:number; icon?:IconSpec; disabled?:boolean; checked?:boolean; action?:NativeActionSpec}
 | {kind:"submenu"; label:string; icon?:IconSpec; disabled?:boolean; menu:MenuSpec};
```

`scrollable` parent 不支持 submenu，这是源码明确限制，应在 DTO 校验时拒绝这个组合。`PopupMenuItem::submenu` 直接塞 Entity 是可用的公共组合入口：`PopupMenu::render` 1390 起为这种路径补上 parent_menu/priority，因此可以同时支持 disabled submenu，以及只持有 `Context<JsPane>` 的 dock hook。不能只读 `submenu_with_icon` 的 constructor 就误判此入口丢失 parent linking。[PopupMenu][menu]、[submenu render linking][menu-linking]

```rust
// MenuSink 是一个 Rust Rc<dyn Fn(MenuEvent)>，可将相同 builder 的事件提升到
// Menu NativeView Event<MenuEvent> 或 Dock NativeView Event<DockBridgeEvent>。
// 两种都只调用 Event::emit，不是 JS callback。
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
                // link() 已有原生 open_url handler，不覆盖为只发事件。
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

`PopupMenu` 本身可以 retained：mount `PopupMenu::build(window,cx,...)`，update `entity.update(cx, |m,cx| m.rebuild(window,cx,...))`；保持 focus/parent/priority identity。订阅 `DismissEvent` 并保留 `Subscription` 在 view 中，drop 自动退订。不要每次 JS props 更新都丢弃当前打开菜单实体。[PopupMenu::rebuild][menu-rebuild]

```rust
// ContextMenu::menu 是私有；这是正确公共入口。
let context_menu = div().id(native_id).child(content_slot)
    .context_menu(move |m, w, cx| build_menu(m, &spec, &slots, &event, w, cx));

// DropdownButton 必须编译 ButtonSpec，无法把一个 AnyElement 强转成 Button。
let dropdown = DropdownButton::new(native_id)
    .button(build_button(&p.button))
    .disabled(p.disabled)
    .dropdown_menu_with_anchor(p.anchor.native(), move |m,w,cx| {
        build_menu(m, &spec, &slots, &event, w, cx)
    });

// 普通 Button dropdown 模式可发 open change。
let dropdown_menu = build_button(&p.button)
    .dropdown_menu_with_anchor(p.anchor.native(), move |m,w,cx| build_menu(m,&spec,&slots,&event,w,cx))
    .on_open_change(move |open,_,_| open_event.emit(OpenEvent { open:*open }));
```

`DropdownButton` 的 variant/size/selected/outline/disabled 通过其 `ButtonVariants/Sizable/Selectable/Disableable` 实现；split button 左侧独立主 action 留在 ButtonSpec，右侧只打开 menu。[DropdownButton][dropdown-button]、[DropdownMenu][dropdown-menu]、[ContextMenu][context-menu]

打开期间更新菜单不能只替换 DropdownMenu builder closure：其内置 `DropdownMenuState.menu` 在打开期间复用，只有 Dismiss 才清空。可在传给 `.dropdown_menu`/`.context_menu` 的 builder 中保存 `cx.weak_entity()` 到 bridge 的 `live_menus`，props update 时沿这些 weak handles 执行 `rebuild`；原生通知后绘制当前项。NativeView 持有菜单 model 的 revision，以免不相关 child/style commit 都重建 menu。对于直接展示的 PopupMenu retained view，本身就有 entity，不需要这层 live handle 注册。[DropdownMenu 缓存][dropdown-menu]

### 5.2 AppMenuBar 是应用菜单镜像

`AppMenuBar::new(cx)` 调 `reload`，读取 `GlobalState::global(cx).app_menus()` 的 `OwnedMenu`。更新需要先 `GlobalState::global_mut(cx).set_app_menus(Vec<OwnedMenu>)`，再 `bar.update(cx, |bar,cx| bar.reload(cx))`。菜单为 application scope，不能让多个局部 wrapper 每次 render 无条件争抢全局菜单。[AppMenuBar][app-menu]、[Base GlobalState][global-state]

用现有应用菜单 command/client 作为唯一 menu owner；每个 item `NativeActionSpec` 编译为注册的 `Action`，action handler 在主 UI root 上把 item ID 发给对应 JS event endpoint。`AppMenuBar` retained adapter：mount 创建 bar；update 检测菜单 revision 后 reload；render 返回 `bar.clone()`；dispose 退订自有 action routes。macOS 原生菜单与 Windows/Linux AppMenuBar 可共享 OwnedMenu 模型，但不是把 PopupMenu custom element slots 传给 OwnedMenu。

## 6. DockArea、TabGroup、Tiles：真实 Pane trait 与布局 API

该 revision 将行为移入 `gpui-base::dock`。`gpui_component::dock::Panel` 是 appearance trait，extends `BasePanel`；只 impl appearance 不够。`BasePanel: EventEmitter<PanelEvent> + Render + Focusable`。唯一无默认值的 `BasePanel` 方法为 `panel_name(&self)->&'static str`；完整适配需要覆盖可见性、关闭/zoom 权限、active/zoomed/added/removed lifecycle 和 dump。[组件 Panel][panel]、[Base Panel][base-panel]

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

通过 `panel_handle(entity)`（返回 base object-safe handle，内部是 `PanelHandle`）接入；如果直接 `.panel(entity)`，`DockSkin` 无法 downcast 回 appearance PanelHandle，标题/toolbar 等失效。这里应使用 `.panel_view(panel_handle(pane),cx)` 与 `.tile_view(...)`。[PanelHandle][panel]、[DockLayout][dock-layout]

```ts
type DockLayoutSpec =
 | {kind:"split"; axis:"horizontal"|"vertical"; children:{layout:DockLayoutSpec; size?:number}[]}
 | {kind:"tabs"; panes:string[]; activeIndex:number}
 | {kind:"tiles"; panes:{id:string; bounds:{x:number;y:number;width:number;height:number}}[]};
type PaneSpec = {
  id:string; title:string; tabName?:string; visible:boolean; closable:boolean;
  zoomable:boolean; zoomControl?:"menu"|"toolbar"|"both"; innerPadding:boolean;
  contentSlot:number; titleSlot?:number; titleSuffixSlot?:number;
  toolbar:ButtonSpec[]; chromeMenu:MenuSpec; persistedData:JsonValue;
};
type DockProps = {
  id:string; version?:number; panes:PaneSpec[];
  initialLayout:{center:DockLayoutSpec;left?:DockSpec;right?:DockSpec;bottom?:DockSpec};
  locked:boolean; panelStyle:"auto"|"tabBar"; toggleButtonVisible:boolean;
  // layout 的日常交互由原生拥有；重置布局走 replaceLayout command。
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

// mount，保留 area + skin + panes + Subscription。
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

`set_dock_size` 与 `set_dock_collapsible` 均接受 `(placement,value,window,&mut Context<DockArea>)`。不要通过反复 `set_center` 同步普通 Pane props：它表示 wholesale replace，会给旧 pane 发 `on_removed`，并重建 layout。`update` 按 pane ID diff 更新保留的 Entity，删除的实体用 `area.remove_panel(entity,window,cx)`；仅显式 `replaceLayout/load` 才重置几何。ID 不可重复、所有 layout pane 引用必须存在、单 pane 不得出现在两个节点、split/tabs/tiles 的子类型和尺寸必须验证。[DockArea][dock-area]

命令完整面：`dump`, `load`, `replaceLayout`, `addPane`, `removePane`, `movePane`, `splitAt`, `toggleDock`, `resizeDock`, `setDockCollapsible`, `zoomInGroup`, `zoomOut`, `selectTab`。`selectTab` 可经 `JsPane.group` 找 `TabGroup::select_tab`，稳定 pane ID 先解析成当前 group index。不要把 raw `NodeId/PanelId` 裸暴露给长期持久化 JSON；使用 view-scoped opaque token 或通过 pane ID 找当前节点。Tiles 自带 move/resize/bringToFront/zoom 与 `undo/redo`；`TilesState::new` 不公开，若要外部命令调 undo/redo 需要 DockArea 的明确 accessor/forward 方法，不能捏造 Entity。[TilesState][tiles-state]

持久化 `DockAreaState` 自带 serde，但不带 TS derive。桥接应定义递归 DTO，映射 `PanelState {panel_name,children,info}` 与 `PanelInfo::{Stack{sizes,axis},Tabs{active_index},Panel(JsonValue),Tiles{metas}}`，不要仅返回 JSON 字符串掩盖结构。注册一个固定名称 `solid-gpui-pane` 的 `register_panel` builder，通过 `PanelBuildContext.dock_area()` 关联所属 native bridge，再通过 `info.id` 找同一 owner 的已提交 slot。不能以任意 JS pane ID `Box::leak` 构造 `&'static str`。加载未知或无 slot ID 应拒绝/异步 `restoreRequested` 后再加载，不能把上游保留 unknown panel 的占位逻辑误当 JS owner 创建能力。[Dock state][dock-state]、[Panel registry][panel-registry]

`DockEvent::LayoutChanged` 每次 tile drag step 都发；对 JS 可在一 native tick 合并成一次 `layoutChanged {sequence}`，完整 dump 通过异步 command 获取或 debounced snapshot。不要每个 mouse move JSON 序列化整棵 dock tree。若业务要 DragDrop，`AnyDrag` 必须限定为桥接自己创建的 typed payload，映射 `DropTarget` 为 validated enum。[DockEvent][dock-area]

## 7. Settings：typed hierarchy，原生 getter/setter

`Settings::new(id)` 支持 `pages`, `sidebar_width`, `sidebar_size_range`, `with_group_variant`, `sidebar_style`, `header_style`, `default_selected_index(SelectIndex{page_ix,group_ix})`, `Sizable`。Settings 内部 `SettingsState` 非公开，search/selected state 以 window keyed state 持久化，没有公开受控选页/search setter/event。因此现有 API 能完整保留原生 sidebar/search 行为；若要 JS 控制 selection/search，需要上游公开 state interface，不能重复构造 ID 达到“更新”。[Settings][settings]

`SettingPage::new(title)` 有 title/title_suffix closure、icon、description、default_open、resettable、groups、header_style。Group 有 title/description/items/Styled。Item 分 `new(title, AnySettingField)` 与 `render(Fn(&RenderOptions,&mut Window,&mut App)->E)`，还可 keywords/disabled/description/layout；自定义 render item 的 disabled **只通过 RenderOptions 传递给自定义渲染器**，不会自动禁掉内部任意 JS 控件。[SettingPage][setting-page]、[SettingGroup][setting-group]、[SettingItem][setting-item]

```ts
type SettingsSpec = {
  pages: SettingPageSpec[]; sidebarWidth:number; sidebarMinWidth:number; sidebarMaxWidth:number;
  groupVariant:GroupBoxVariant; size:Size;
  defaultSelectedIndex:{pageIx:number;groupIx?:number};
};
type SettingPageSpec = {id:string; title:string; description?:string; icon?:IconSpec;
  titleSuffixSlot?:number; defaultOpen:boolean; resettable:boolean; groups:SettingGroupSpec[]};
type SettingGroupSpec = {id:string; title?:string; description?:string; items:SettingItemSpec[]};
type SettingItemSpec =
 | {kind:"field";id:string;title:string;description?:string;keywords:string[];disabled:boolean;layout:"horizontal"|"vertical";field:SettingFieldSpec}
 | {kind:"element";id:string;slot:number;keywords:string[];disabled:boolean;dirty:boolean;resettable:boolean};
type SettingFieldSpec =
 | {kind:"switch"|"checkbox";value:boolean;defaultValue?:boolean}
 | {kind:"input";value:string;defaultValue?:string}
 | {kind:"number";value:number;defaultValue?:number;min:number;max:number;step:number}
 | {kind:"dropdown";value:string;defaultValue?:string;scrollable:boolean;options:{value:string;label:string}[]}
 | {kind:"element";slot:number;dirty:boolean;resettable:boolean};
type SettingsEvent = {kind:"change";id:string;value:boolean|string|number;sequence:number}
 | {kind:"resetRequested";id:string};
```

使用 retained `SettingsModel` 保存 `BTreeMap<ItemId,Value>`，native getter 读 model，setter 先更新原生模型再 emit。不能 setter 只发 JS event 不本地更新，否则下一帧 getter 仍返回旧值，出现控件回弹。JS controlled 更新需要 sequence/ack 规则，避免迟到 commit 覆盖新 native 编辑。[SettingField][setting-field]

```rust
// value_cell 由单个 Settings NativeView 持有，update 按 item ID 调和，不跨实例共享。
// NativeNotify 捕获 settings owner 的 WeakEntity，只通知仍然存活的 owner。
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
    // 与上游 custom render 的责任相同：disabled 必须已传给实际控件。
    // JS SettingsItem convenience wrapper 可从同一 disabled prop 创建 context，
    // 在提交前设置 custom child control props；这里不把 opacity 当禁用。
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

`model.build_item` 的实际分派为：switch/checkbox→`SettingItem::new(title,bool_field(...).default_value(...))`；input/dropdown→string_field；number→number_field；element field→custom_field；整行 element→`SettingItem::render(...).on_reset(...)`。每一种都附加 keywords/disabled，只有 field item 附加 description/layout。change 事件使用 Settings native owner 的 `EventSequence`，notify closure 使用其 WeakEntity 执行 `entity.update(cx,|_,cx|cx.notify())`；普通 cell 改值本身不会触发 GPUI render。

`SettingFieldElement` 的唯一方法是 `render_field(&self,&RenderOptions,&mut Window,&mut App)->Self::Element`，associated `Element:IntoElement+'static`。普通 `Fn` 已有 blanket impl，无须为每个 custom field 造额外 trait object。[SettingFieldElement][setting-element]

`RenderOptions` 还携带 `page_ix/group_ix/item_ix/size/group_variant/layout/disabled`。custom JSX children 若需要这些值，应把 JS 已知的 size/disabled/index 放在组件 context 中，原生仅处理按实际宽度变化的容器布局；要把运行时 `RenderOptions.layout` 暴露回 JS，则使用明确事件/下一 commit 更新，不可在 native render 同步求 JSX。原生 renderer 不应假装对子树设 opacity 就实现了 disabled。

结构更新还需注意：内置 input state 的 key 基于 page/group/item **索引**。动态插入/重新排序 SettingsSpec 时，必须确认输入 entity 没把旧 item 的编辑态转给新 item；需要上游 stable item key API 才能无损支持任意重排。不能靠每次换整个 Settings ID 隐藏 identity 问题。[number field state][setting-number]

## 8. 七种 Chart：固定 datum，支持全部 builder 功能

`LineChart/AreaChart/BarChart/CandlestickChart` 的 X/B 是离散标签类型；默认固定为 `String/SharedString`。Y/V 的 `Sealed` 仅允许 `f64` 和可选 rust_decimal Decimal，默认桥接 `f64`。它们不接受 JS 函数；JS 在创建 props 时把想要的业务字段投影成 typed rows。`tick_margin` 在共享 label helper 做 `% tick_margin`，因此 **0 必须在 commit 校验时拒绝**。[Chart exports/helper][chart]、[Scale sealed][scale-sealed]

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

DTO 不必包含 Rust-only Hsla/Background/Slot；wire 为 ColorSpec/BackgroundSpec、slot index，经 validate/mount 编译成上述 Datum。统一 `BackgroundSpec` 至少覆盖 solid、linear gradient 两个 color stop、angle；bounds-dependent gradient 用一个明确枚举 `barLocal/chartRange`，在 Rust 闭包中用上游 `chart_to_bar` 计算。`unknown JSON field` / `function source` / `eval` 都不需要。

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
    // 若启用 per-slice 半径，DTO 校验必须补全全部 slice 的半径值。
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

每个 renderer 的 p 已包含对应 builder 表示的全部功能，建议 JS DTO 分别与其字段一一对应，不弄一个能接收任意 chart 属性的大字典。Area/Radar series 数组的 `name/stroke/fill/curve` 必须按索引齐全填入，每行 values 长度等于 series 数量。不能把缺失 series 填零，这会改变业务数据。Line/Area/Bar/Radar 只有设稳定 `.id` 才有 native hover tooltip；Candlestick/Pie/Sankey 当前没有公开 interactive ID/tooltip setter，不能宣称原生已有。[Line][line]、[Area][area]、[Bar][bar]、[Candle][candle]、[Pie][pie]、[Radar][radar]、[Sankey][sankey]

Radar custom label 的 `Fn(&T)->RadarLabel` 没有 Window/App，所以 `Slot.clone().into_any_element()` 是正确延迟入口：后续 native prepaint 再布局 label 子树；不能在 `.label` 中同步调 JS。源文件明确 `Plot::prepaint` 布局标签，`paint` 不允许进行 layout。[Plot trait][plot]

更新：Charts 无长期 Entity state，保留 stable host ID；props 或 native theme 改变时重建相应 lightweight plot，renderer 由 `IntoPlot` 管布局/paint。大型数据可在 mount/update 编译 datum，用 `Rc<Row>` 传递减轻每帧 String clone，但不要在每个 mouse move 重新跨线程传整份数据。Sankey 的布局 topology/relaxation 随数据与 bounds 变化，若需要缓存，key 至少包含 data revision、bounds、nodeWidth/padding/align/iterations/valueScale 与文本样式。

输入检查：所有数值 finite；tickMargin≥1；图表尺寸有界；pie 非负 value/半径且 inner≤outer；OHLC low≤min(open,close)≤max(open,close)≤high；Sankey source/target 有效、nonnegative finite value、graph acyclic（Sankey::topology 返回具体错误）；iterations 有边界，禁止用户 DTO 让 paint 任意多次 relaxation。[Sankey math][sankey-math]

## 9. Plot 原语完整覆盖方式

`Plot` 至少实现 `paint(&mut self,Bounds<Pixels>,&mut Window,&mut App)`；可覆写 `prepaint()->Vec<AnyElement>`, `id()->Option<ElementId>`, `tooltip_state(...)`, `tooltip(...)`。`#[derive(IntoPlot)]` 提供 IntoElement。原语不是传统 JSX children，可暴露 `Plot { primitives: PlotPrimitive[] }`，也可生成 `Plot.Line` 等 typed child declarations，由 wrapper 编译为该 DTO。两种 TS 写法必须汇合为同一原生 painter。[Plot][plot]

完整可表示的 DTO 变体：

```ts
type PlotPrimitive =
 | {kind:"axis";x?:number;y?:number;xAxis:boolean;yAxis:boolean;
    xLabels:AxisTextSpec[];yLabels:AxisTextSpec[];xLabelSide:"start"|"end";yLabelSide:"start"|"end";stroke:ColorSpec}
 | {kind:"grid";x:number[];y:number[];stroke:ColorSpec;dash:number[]}
 | {kind:"labels";items:TextSpec[]}
 | {kind:"line";points:PointSpec[];stroke:BackgroundSpec;strokeWidth:number;curve:Curve;
    dots:boolean;dotSize:number;dotFill:ColorSpec;dotStroke?:ColorSpec}
 | {kind:"area";points:PointSpec[];baseline:number;fill:BackgroundSpec;stroke:BackgroundSpec;curve:Curve}
 | {kind:"bar";rows:{cross:number|null;base:number;value:number|null;fill:BackgroundSpec;labels:TextSpec[]}[];
    alignment:"top"|"bottom"|"left"|"right";bandWidth:number;cornerRadii:CornersSpec}
 | {kind:"radialLine";points:{angle:number|null;radius:number|null}[];closed:boolean;fill:BackgroundSpec;
    stroke:BackgroundSpec;strokeWidth:number;dots:boolean;dotSize:number;dotFill:ColorSpec;dotStroke?:ColorSpec}
 | {kind:"arc";startAngle:number;endAngle:number;padAngle:number;innerRadius:number;outerRadius:number;fill:ColorSpec};
type PointSpec={x:number|null;y:number|null};
type TextSpec={text:string;x:number;y:number;color:ColorSpec;fontSize:number;fontWeight:number;align:"left"|"center"|"right"};
type AxisTextSpec={text:string;tick:number;color:ColorSpec;fontSize:number;align:"left"|"center"|"right"};
```

`Line/Area/RadialLine` 的 accessor 用 `Option<f32>`；DTO 里的 null 明确表示缺失点。不要把 null 变成 0。Axis 的 builder **先 x/y 和 labelSide，再 x/y_label**，后者调用当时就计算文本位置。[Axis][axis]、[Shapes][shapes]

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
                        // label callback 的 origin 是该 bar 的 anchor；DTO 文本坐标是相对此 anchor。
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

`Plot tooltip` 的独立视觉 DTO 可映射原生 `tooltip::Tooltip::new(cursor,within)`→title/row/gap/cross_line/dots/appearance/children；`CrossLine::new(point)`→band/horizontal/both/height/width/span/h_span；`Dot::new(point)`→size/stroke/fill。这些都实现 RenderOnce，可包普通 element，但需明确坐标为 plot-local、仅在 overlay 中使用。自定义 tooltip 的 native hit-test 用 index/几何 DTO，不可同步调用 JS。[Plot tooltip][plot-tooltip]

计算对象可用 typed `ViewCommand` 或 module functions，返回数据并由 JS 重用：

```rust
// Scale 的 domain/range、tick、nearest 是计算，不应生成一个空屏幕 JSX 组件。
let linear=ScaleLinear::new(domain_f64,range_f32);
let ticks:Vec<Option<f32>>=values.iter().map(|v|linear.tick(v)).collect();
let point=ScalePoint::new(domain_strings,range_f32);
let nearest=point.least_index(cursor);
let band=ScaleBand::new(domain_strings,range_f32).padding_inner(inner).padding_outer(outer);
let width=band.band_width();
let ordinal=ScaleOrdinal::new(domain_strings,range_colors);
let color=ordinal.map(&value); // unknown(...) 可显式配置。

let pie=Pie::new().value(|d:&f32|*d).start_angle(start).end_angle(end).pad_angle(pad);
let arcs=pie.arcs(&values); // 返回 DTO index/value/startAngle/endAngle/padAngle，不携带 &'a T。

let stacked=Stack::new().data(rows).keys(keys)
    .value(|row:&BTreeMap<String,f32>,key|row.get(key).copied()).series();
// DTO: [{key,index,points:[{y0,y1,data:row}]}]。

let sankey=Sankey::new().node_width(width).node_padding(padding).node_align(align)
    .iterations(iterations).value_scale(scale).size(canvas_w,canvas_h);
let graph=sankey.layout(node_count,&links)?; // typed error，不吞 graph validation failure。
```

`Sankey::topology(node_count,&links)` 可与 `layout_from(graph)` 分离缓存拓扑；`sankey_link_path(&SankeyNodeLayout,&SankeyNodeLayout,&SankeyLinkLayout,min_width:f32,origin:Point<Pixels>)->Option<Path<Pixels>>` 返回 native Path，不跨 wire；wire 只传 graph node/link geometry。`Plot::polygon` 也返回 native Path，只能作为 native painter 的 helper，不是可序列化组件。[Scale][scale]、[Pie/Stack/Arc][shapes]、[Sankey math][sankey-math]

这些计算命令的返回 DTO 可以完整保留上游公开数据：

```ts
type ArcDataDto={index:number;value:number;startAngle:number;endAngle:number;padAngle:number};
type StackDto<T>={key:string;index:number;points:{y0:number;y1:number;data:T}[]}[];
type SankeyGraphDto={
  nodes:{index:number;value:number;depth:number;height:number;layer:number;
    x0:number;x1:number;y0:number;y1:number;sourceLinks:number[];targetLinks:number[]}[];
  links:{index:number;source:number;target:number;value:number;y0:number;y1:number;
    width:number;sourceWidth:number;targetWidth:number}[];
};
type SankeyErrorDto={kind:"missingNode";index:number}|{kind:"circularLink"};
type PlotTooltipDto={
  cursor:{x:number;y:number};within:{width:number;height:number};title?:string;gap:number;appearance:boolean;
  rows:{color:ColorSpec;label:string;value:string}[];
  crossLine?:{point:{x:number;y:number};direction:"vertical"|"horizontal"|"both";
    bandThickness?:number;verticalSpan?:{start:number;length:number};horizontalSpan?:{start:number;length:number}};
  dots:{point:{x:number;y:number};size:number;stroke:ColorSpec;fill:ColorSpec}[];
};
```

Sankey graph `value` 是 layout value-space；`valueScale:"sqrt"` 时并非 raw business flow。原始 links 保留原值，图表 `value_label` callback 得到 raw throughput；DTO 应分别命名/说明，不能把 scaled graph.value 显示为业务流量。[Sankey math][sankey-math]

## 10. 必留验收案例

1. Dialog A→Dialog B；卸载 A 时 B 保留且能正常关闭/恢复 focus；再退休 surface 时没有遮罩/focus trap 残留。异步 ok 的过期 resolve 不得关闭 B。
2. Popover 正打开时 content 的 JS signal 更新，连续多帧仍可见新文本；关闭后复开；owner 删除后 retained closure 不能绘制旧 child。
3. Popup submenu keyboard 左右/escape、disabled item、点击 item 回到正确 focus；打开状态 props 更新沿用 PopupMenu identity。
4. Dock tabs drag 到 split/tiles，pane native Entity 与 JS child 状态保留；dump/load 包含 pane ID/data/tiles bounds；关闭 pane 发 Removed，移动 pane 不发 Removed。
5. Settings 自定义 disabled、reset、异步值 ack；搜索后编辑，以及索引变化时不会编辑错误 item。
6. 七种 chart 各一个真实数据/native screenshot；Area/Radar 多 series 索引正确；Radar label JSX；Bar 双坐标 gradient；tickMargin=0 在 commit 前明确拒绝，Sankey cycle/invalid link 有具体错误。

这些是桥接新增行为的关键测试。没有为每个 builder setter 单独造镜像测试。UI/焦点/绘制验收需要真实 native surface，本文没有执行它们。

## 源码索引

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
