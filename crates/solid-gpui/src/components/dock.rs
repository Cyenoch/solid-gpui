//! Native docking owns geometry and pane entities; Solid owns each pane's committed content.
use super::icon_source::ComponentIcon;
use super::{
    ButtonVariant, ControlSize,
    dock_layout::*,
    menus::{MenuModel, MenuSpec, validate_menu},
    overlays::Side,
    plot::length,
    primitives::Color,
    scroll_views::ScrollbarVisibility,
};
use crate::native::{
    ComponentDefinition, Event, EventDefinition, NativeChildren, NativeSlot, NativeView,
    ViewCommand,
};
use gpui::{
    App, AppContext, Context, Entity, EntityId, EventEmitter, FocusHandle, Focusable, Global,
    InteractiveElement, IntoElement, ParentElement, Render, SharedString, Styled, Subscription,
    WeakEntity, Window, div, px,
};
use gpui_component::{
    Disableable, Selectable, Sizable,
    button::{Button, ButtonVariants},
    dock::{
        self, BasePanel, DockArea as NativeArea, DockEvent, DockSkin, InsertTarget, NodeId,
        PaneRef, Panel, PanelControl, PanelEvent, PanelId, PanelInfo, PanelState, PanelStyle,
        TileContext, TilesState, TitleStyle, panel_handle,
    },
    menu::PopupMenu,
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    rc::{Rc, Weak},
};
fn yes() -> bool {
    true
}
#[crate::native_type]
#[derive(Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DockToolbarButton {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub icon: Option<ComponentIcon>,
    #[serde(default)]
    pub variant: ButtonVariant,
    #[serde(default)]
    pub size: ControlSize,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub loading: bool,
    #[serde(default)]
    pub selected: bool,
    #[serde(default)]
    pub outline: bool,
    #[serde(default)]
    pub tooltip: Option<String>,
}
#[crate::native_type]
#[derive(Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DockZoomControl {
    None,
    #[default]
    Menu,
    Toolbar,
    Both,
}
#[crate::native_type]
#[derive(Clone, PartialEq)]
pub struct DockTitleStyle {
    pub background: Color,
    pub foreground: Color,
}
#[crate::native_type]
#[derive(Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DockPane {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub tab_name: Option<String>,
    #[serde(default)]
    pub content_slot: Option<usize>,
    #[serde(default)]
    pub title_slot: Option<usize>,
    #[serde(default)]
    pub title_suffix_slot: Option<usize>,
    #[serde(default = "yes")]
    pub visible: bool,
    #[serde(default = "yes")]
    pub closable: bool,
    #[serde(default = "yes")]
    pub zoomable: bool,
    #[serde(default)]
    pub zoom_control: DockZoomControl,
    #[serde(default = "yes")]
    pub inner_padding: bool,
    #[serde(default)]
    pub title_style: Option<DockTitleStyle>,
    #[serde(default)]
    pub toolbar: Vec<DockToolbarButton>,
    #[serde(default)]
    pub menu: MenuSpec,
    #[serde(default)]
    pub data: DockValue,
}
impl DockPane {
    fn validate(&self, children: Option<usize>) -> Result<(), String> {
        name(&self.name)?;
        self.data.validate()?;
        validate_menu(&self.menu, children)?;
        for index in [self.content_slot, self.title_slot, self.title_suffix_slot]
            .into_iter()
            .flatten()
        {
            if children.is_some_and(|n| index >= n) {
                return Err(format!(
                    "pane {} references missing child slot {index}",
                    self.name
                ));
            }
        }
        if self.toolbar.len() > 32 {
            return Err("pane toolbar supports at most 32 buttons".into());
        }
        let mut names = HashSet::new();
        for b in &self.toolbar {
            name(&b.id)?;
            if !names.insert(&b.id) {
                return Err("toolbar button ids must be unique".into());
            }
        }
        Ok(())
    }
}
#[crate::native_type]
#[derive(Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum DockPanelStyle {
    #[default]
    Auto,
    TabBar,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct DockAreaProps {
    pub panes: Vec<DockPane>,
    #[serde(default)]
    pub initial_layout: DockLayoutSpec,
    #[serde(default)]
    pub version: Option<u32>,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub panel_style: DockPanelStyle,
    #[serde(default = "yes")]
    pub toggle_button_visible: bool,
    #[serde(default)]
    pub tiles_scrollbar: Option<ScrollbarVisibility>,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum DockPaneEvent {
    Active { pane: String, active: bool },
    Zoomed { pane: String, zoomed: bool },
    Removed { pane: String },
    Menu { pane: String, id: String },
    MenuDismissed { pane: String },
    Toolbar { pane: String, id: String },
}
#[crate::native_type]
#[derive(Clone)]
pub struct DockLayoutChange {
    pub revision: u32,
}
struct JsPane {
    spec: DockPane,
    data: DockValue,
    content: NativeSlot,
    focus: FocusHandle,
    event: Event<DockPaneEvent>,
    menu: Rc<MenuModel>,
    active: bool,
    zoomed: bool,
}
impl JsPane {
    fn new(
        spec: DockPane,
        content: NativeSlot,
        event: Event<DockPaneEvent>,
        cx: &mut Context<Self>,
    ) -> Self {
        let pane = spec.name.clone();
        let select = event.clone();
        let dismiss_pane = pane.clone();
        let dismiss_event = event.clone();
        let menu = MenuModel::with_callbacks(
            &spec.menu,
            content.clone(),
            Rc::new(move |v| {
                select.emit(DockPaneEvent::Menu {
                    pane: pane.clone(),
                    id: v.id,
                })
            }),
            Rc::new(move || {
                dismiss_event.emit(DockPaneEvent::MenuDismissed {
                    pane: dismiss_pane.clone(),
                })
            }),
        );
        Self {
            data: spec.data.clone(),
            spec,
            content,
            focus: cx.focus_handle(),
            event,
            menu,
            active: false,
            zoomed: false,
        }
    }
    fn sync(&mut self, p: DockPane, window: &mut Window, cx: &mut Context<Self>) {
        if self.spec.menu != p.menu {
            self.menu.replace(&p.menu, window, cx);
        }
        if self.spec.data != p.data {
            self.data = p.data.clone();
        }
        self.spec = p;
        self.menu.notify(cx);
        cx.notify();
    }
}
impl EventEmitter<PanelEvent> for JsPane {}
impl Focusable for JsPane {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl Render for JsPane {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .track_focus(&self.focus)
            .children(self.spec.content_slot.map(|i| self.content.item(i)))
    }
}
impl BasePanel for JsPane {
    fn panel_name(&self) -> &'static str {
        PANEL_NAME
    }
    fn visible(&self, _: &App) -> bool {
        self.spec.visible
    }
    fn closable(&self, _: &App) -> bool {
        self.spec.closable
    }
    fn zoomable(&self, _: &App) -> bool {
        self.spec.zoomable && self.spec.zoom_control != DockZoomControl::None
    }
    fn set_active(&mut self, active: bool, _: &mut Window, _: &mut Context<Self>) {
        self.active = active;
        self.event.emit(DockPaneEvent::Active {
            pane: self.spec.name.clone(),
            active,
        });
    }
    fn set_zoomed(&mut self, zoomed: bool, _: &mut Window, _: &mut Context<Self>) {
        self.zoomed = zoomed;
        self.event.emit(DockPaneEvent::Zoomed {
            pane: self.spec.name.clone(),
            zoomed,
        });
    }
    fn on_removed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active = false;
        self.zoomed = false;
        self.menu.close(window, cx);
        self.event.emit(DockPaneEvent::Removed {
            pane: self.spec.name.clone(),
        });
    }
    fn dump(&self, _: &App) -> PanelState {
        PanelState {
            panel_name: PANEL_NAME.into(),
            children: vec![],
            info: PanelInfo::panel(serde_json::json!({"pane":self.spec.name,"data":self.data})),
        }
    }
}
impl Panel for JsPane {
    fn tab_name(&self, _: &App) -> Option<SharedString> {
        self.spec.tab_name.clone().map(Into::into)
    }
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.spec.title_slot.map_or_else(
            || self.spec.title.clone().into_any_element(),
            |i| self.content.item(i).into_any_element(),
        )
    }
    fn title_suffix(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<impl IntoElement> {
        self.spec.title_suffix_slot.map(|i| self.content.item(i))
    }
    fn title_style(&self, _: &App) -> Option<TitleStyle> {
        self.spec.title_style.as_ref().map(|v| TitleStyle {
            background: v.background.native(),
            foreground: v.foreground.native(),
        })
    }
    fn inner_padding(&self, _: &App) -> bool {
        self.spec.inner_padding
    }
    fn zoom_control(&self, _: &App) -> Option<PanelControl> {
        match self.spec.zoom_control {
            DockZoomControl::None => None,
            DockZoomControl::Menu => Some(PanelControl::Menu),
            DockZoomControl::Toolbar => Some(PanelControl::Toolbar),
            DockZoomControl::Both => Some(PanelControl::Both),
        }
    }
    fn dropdown_menu(
        &mut self,
        menu: PopupMenu,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> PopupMenu {
        self.menu.compile(menu, "", window, cx)
    }
    fn toolbar_buttons(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<Vec<Button>> {
        Some(
            self.spec
                .toolbar
                .iter()
                .map(|s| {
                    let event = self.event.clone();
                    let pane = self.spec.name.clone();
                    let id = s.id.clone();
                    let mut b = Button::new(s.id.clone())
                        .with_variant(s.variant.into())
                        .with_size(s.size)
                        .disabled(s.disabled)
                        .loading(s.loading)
                        .selected(s.selected)
                        .on_click(move |_, _, _| {
                            event.emit(DockPaneEvent::Toolbar {
                                pane: pane.clone(),
                                id: id.clone(),
                            })
                        });
                    if s.outline {
                        b = b.outline();
                    }
                    if let Some(v) = &s.label {
                        b = b.label(v.clone());
                    }
                    if let Some(v) = &s.icon {
                        b = b.icon(v.native());
                    }
                    if let Some(v) = &s.tooltip {
                        b = b.tooltip(v.clone());
                    }
                    b
                })
                .collect(),
        )
    }
}
type PaneRegistry = RefCell<BTreeMap<String, WeakEntity<JsPane>>>;
#[derive(Default)]
struct DockOwners(HashMap<EntityId, Weak<PaneRegistry>>);
impl Global for DockOwners {}
fn register(cx: &mut App) {
    if cx.try_global::<DockOwners>().is_some() {
        return;
    }
    cx.set_global(DockOwners::default());
    dock::register_panel(cx, PANEL_NAME, |context, _, cx| {
        let owner = cx
            .global::<DockOwners>()
            .0
            .get(&context.dock_area().entity_id())
            .and_then(Weak::upgrade)
            .expect("validated live JS dock owner");
        let PanelInfo::Panel(info) = context.info() else {
            unreachable!("validated pane state")
        };
        let name = info["pane"].as_str().expect("validated pane name");
        let pane = owner
            .borrow()
            .get(name)
            .and_then(WeakEntity::upgrade)
            .expect("validated committed pane");
        panel_handle(pane)
    });
}
#[crate::native_type]
pub struct DockPaneRequest {
    pub pane: String,
}
#[crate::native_type]
#[serde(rename_all = "camelCase")]
pub struct DockAddPane {
    pub pane: String,
    #[serde(default)]
    pub region: DockRegion,
    #[serde(default)]
    pub size: Option<f32>,
    #[serde(default)]
    pub bounds: Option<DockBounds>,
}
#[crate::native_type]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum DockMoveTarget {
    Tabs {
        anchor: String,
        #[serde(default)]
        index: Option<usize>,
        #[serde(default = "yes")]
        activate: bool,
    },
    Split {
        anchor: String,
        side: Side,
        #[serde(default)]
        size: Option<f32>,
    },
    Tile {
        region: DockRegion,
        bounds: DockBounds,
    },
}
#[crate::native_type]
pub struct DockMovePane {
    pub pane: String,
    pub target: DockMoveTarget,
}
#[crate::native_type]
pub struct DockRegionRequest {
    pub region: DockRegion,
}
#[crate::native_type]
pub struct DockResize {
    pub region: DockRegion,
    pub size: f32,
}
#[crate::native_type]
pub struct DockCollapsible {
    pub region: DockRegion,
    pub collapsible: bool,
}
#[crate::native_type]
pub struct DockSetTileBounds {
    pub pane: String,
    pub bounds: DockBounds,
}
struct DockArea {
    props: DockAreaProps,
    children: NativeChildren,
    area: Entity<NativeArea>,
    skin: Rc<DockSkin>,
    panes: BTreeMap<String, Entity<JsPane>>,
    registry: Rc<PaneRegistry>,
    event: Event<DockPaneEvent>,
    revision: u32,
    pending_layout: bool,
    _subscription: Subscription,
}
impl DockArea {
    fn known(&self) -> HashSet<&str> {
        self.panes.keys().map(String::as_str).collect()
    }
    fn pane(&self, name: &str) -> Result<Entity<JsPane>, String> {
        self.panes
            .get(name)
            .cloned()
            .ok_or_else(|| format!("pane is not configured: {name}"))
    }
    fn location(&self, pane: PanelId, cx: &App) -> Result<(DockRegion, NodeId), String> {
        for region in [
            DockRegion::Center,
            DockRegion::Left,
            DockRegion::Right,
            DockRegion::Bottom,
        ] {
            if let Some(node) = self
                .area
                .read(cx)
                .layout(region.into())
                .and_then(|tree| tree.find_panel_node(pane))
            {
                return Ok((region, node));
            }
        }
        Err("pane is not in the dock layout".into())
    }
    fn tiles(&self, pane: &str, cx: &App) -> Result<(Entity<TilesState>, TileContext), String> {
        let id = PanelId::from(self.pane(pane)?.entity_id());
        let (_, node) = self.location(id, cx)?;
        let state = self
            .area
            .read(cx)
            .tiles_state(node)
            .ok_or("pane is not on a tiles canvas")?;
        let tile = state
            .read(cx)
            .tiles(cx)
            .into_iter()
            .find(|t| t.panel_id() == id)
            .ok_or("tile is no longer present")?;
        Ok((state, tile))
    }
    fn canvas(&self, region: DockRegion, cx: &App) -> Result<NodeId, String> {
        let tree = self
            .area
            .read(cx)
            .layout(region.into())
            .ok_or("dock region does not exist")?;
        let mut found = None;
        for node in tree.node_ids() {
            if matches!(tree.find_node(node).unwrap().kind(), PaneRef::Tiles { .. }) {
                if found.is_some() {
                    return Err(
                        "region contains multiple tile canvases; address a pane instead".into(),
                    );
                }
                found = Some(node);
            }
        }
        found.ok_or("region has no tiles canvas".into())
    }
    fn replace_layout(
        &mut self,
        layout: DockLayoutSpec,
        version: Option<u32>,
        data: Option<BTreeMap<String, DockValue>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        layout.validate(Some(&self.known()))?;
        if let Some(data) = &data {
            for (name, v) in data {
                self.pane(name)?;
                v.validate()?;
            }
        }
        if let Some(data) = data {
            for (name, v) in data {
                self.panes[&name].update(cx, |pane, _| pane.data = v);
            }
        }
        let data = self
            .panes
            .iter()
            .map(|(name, pane)| (name.clone(), pane.read(cx).data.clone()))
            .collect();
        self.area.update(cx, |area, cx| {
            area.load(layout.native(version, &data), window, cx)
                .map_err(|e| e.to_string())?;
            for (region, dock) in layout.regions() {
                if let Some(dock) = dock {
                    area.set_dock_collapsible(region.into(), dock.collapsible, window, cx);
                }
            }
            Ok::<_, String>(())
        })
    }
    fn snapshot(&self, cx: &App) -> Result<DockSnapshot, String> {
        let native = self.area.read(cx).dump(cx);
        let mut data = BTreeMap::new();
        let center = DockNode::from_native(&native.center, &mut data)?;
        let mut region =
            |state: Option<dock::DockState>| -> Result<Option<DockRegionLayout>, String> {
                state
                    .map(|s| {
                        Ok(DockRegionLayout {
                            layout: DockNode::from_native(s.panel(), &mut data)?,
                            size: s.size().as_f32(),
                            open: s.open(),
                            collapsible: self.area.read(cx).is_dock_collapsible(s.placement()),
                        })
                    })
                    .transpose()
            };
        let layout = DockLayoutSpec {
            center,
            left: region(native.left_dock)?,
            right: region(native.right_dock)?,
            bottom: region(native.bottom_dock)?,
        };
        Ok(DockSnapshot {
            version: native
                .version
                .map(|v| u32::try_from(v).expect("JS dock version")),
            layout,
            data,
        })
    }
    fn sync_panes(&mut self, props: &DockAreaProps, window: &mut Window, cx: &mut Context<Self>) {
        let live = props
            .panes
            .iter()
            .map(|p| p.name.as_str())
            .collect::<HashSet<_>>();
        let removed = self
            .panes
            .keys()
            .filter(|name| !live.contains(name.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        for name in removed {
            let pane = self.panes.remove(&name).unwrap();
            pane.update(cx, |p, cx| p.menu.close(window, cx));
            self.area
                .update(cx, |a, cx| a.remove_panel(pane, window, cx));
        }
        for p in &props.panes {
            if let Some(pane) = self.panes.get(&p.name) {
                if pane.read(cx).spec != *p {
                    pane.update(cx, |pane, cx| pane.sync(p.clone(), window, cx));
                } else {
                    pane.update(cx, |pane, cx| {
                        pane.menu.notify(cx);
                        cx.notify();
                    });
                }
            } else {
                let pane = cx.new(|cx| {
                    JsPane::new(p.clone(), self.children.content(), self.event.clone(), cx)
                });
                self.panes.insert(p.name.clone(), pane);
            }
        }
        *self.registry.borrow_mut() = self
            .panes
            .iter()
            .map(|(n, p)| (n.clone(), p.downgrade()))
            .collect();
        self.area.update(cx, |a, cx| a.refresh_panels(window, cx));
    }
    fn apply_skin(&self, p: &DockAreaProps, cx: &mut App) {
        self.skin.set_panel_style(
            match p.panel_style {
                DockPanelStyle::Auto => PanelStyle::Auto,
                DockPanelStyle::TabBar => PanelStyle::TabBar,
            },
            cx,
        );
        self.skin
            .set_toggle_button_visible(p.toggle_button_visible, cx);
        self.skin
            .set_tiles_scrollbar_mode(p.tiles_scrollbar.map(Into::into), cx);
    }
}
impl NativeView for DockArea {
    type Props = DockAreaProps;
    type Event = DockPaneEvent;
    fn accepts_children() -> bool {
        true
    }
    fn event_name() -> &'static str {
        "paneEvent"
    }
    fn additional_events() -> Vec<EventDefinition> {
        vec![EventDefinition::new::<DockLayoutChange>("layoutChange")]
    }
    fn validate_props(p: &Self::Props) -> Result<(), String> {
        if p.panes.len() > 128 {
            return Err("a dock supports at most 128 configured panes".into());
        }
        let mut names = HashSet::new();
        for pane in &p.panes {
            pane.validate(None)?;
            if !names.insert(pane.name.as_str()) {
                return Err("dock pane names must be unique".into());
            }
        }
        p.initial_layout.validate(Some(&names))
    }
    fn validate_children(p: &Self::Props, c: &crate::ExtensionChildSummary) -> Result<(), String> {
        for pane in &p.panes {
            pane.validate(Some(c.content_count()))?;
        }
        Ok(())
    }
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        children: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        register(cx);
        let (area, skin) = DockSkin::dock_area(
            format!("solid-dock-{}", cx.entity_id()),
            props.version.map(|v| v as usize),
            window,
            cx,
        );
        let registry = Rc::new(RefCell::new(BTreeMap::new()));
        cx.global_mut::<DockOwners>()
            .0
            .insert(area.entity_id(), Rc::downgrade(&registry));
        let subscription = cx.subscribe(&area, |this, _, e, cx| {
            if matches!(e, DockEvent::LayoutChanged) && !this.pending_layout {
                this.pending_layout = true;
                let weak = cx.weak_entity();
                cx.defer(move |cx| {
                    let _ = weak.update(cx, |this, _| {
                        this.pending_layout = false;
                        this.revision = this
                            .revision
                            .checked_add(1)
                            .expect("dock revision exhausted");
                        this.event.related::<DockLayoutChange>("layoutChange").emit(
                            DockLayoutChange {
                                revision: this.revision,
                            },
                        );
                    });
                });
            }
        });
        let mut this = Self {
            props: props.clone(),
            children,
            area,
            skin,
            registry,
            event,
            panes: BTreeMap::new(),
            revision: 0,
            pending_layout: false,
            _subscription: subscription,
        };
        this.sync_panes(&props, window, cx);
        this.apply_skin(&props, cx);
        this.replace_layout(
            props.initial_layout.clone(),
            props.version,
            None,
            window,
            cx,
        )
        .expect("validated initial dock layout");
        this.area
            .update(cx, |a, cx| a.set_locked(props.locked, window, cx));
        this
    }
    fn update(&mut self, p: Self::Props, window: &mut Window, cx: &mut Context<Self>) {
        self.sync_panes(&p, window, cx);
        self.apply_skin(&p, cx);
        self.area.update(cx, |a, cx| {
            a.set_locked(p.locked, window, cx);
            if p.version != self.props.version {
                a.set_version(p.version.map(|v| v as usize), cx);
            }
        });
        self.props = p;
        cx.notify();
    }
    fn unmount(&mut self, window: &mut Window, cx: &mut App) {
        cx.global_mut::<DockOwners>()
            .0
            .remove(&self.area.entity_id());
        for pane in self.panes.values() {
            pane.update(cx, |p, cx| p.menu.close(window, cx));
        }
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("dump", |v, _: (), _, cx| v.snapshot(cx)),
            ViewCommand::new("load", |v, p: DockSnapshot, w, cx| {
                v.replace_layout(p.layout, p.version, Some(p.data), w, cx)
            }),
            ViewCommand::new("replaceLayout", |v, p: DockLayoutSpec, w, cx| {
                let version = v.area.read(cx).version().map(|n| n as u32);
                v.replace_layout(p, version, None, w, cx)
            }),
            ViewCommand::new("addPane", |v, p: DockAddPane, w, cx| {
                let pane = v.pane(&p.pane)?;
                if let Some(n) = p.size {
                    length(n)?;
                }
                if let Some(b) = p.bounds {
                    b.validate()?;
                    v.canvas(p.region, cx)?;
                }
                if v.location(PanelId::from(pane.entity_id()), cx).is_ok() {
                    return Err("pane is already in the layout; use movePane".into());
                }
                v.area.update(cx, |a, cx| match p.bounds {
                    Some(bounds) => {
                        a.add_tile_view(panel_handle(pane), p.region.into(), bounds.native(), w, cx)
                    }
                    None => {
                        a.add_panel_view(panel_handle(pane), p.region.into(), p.size.map(px), w, cx)
                    }
                });
                Ok(())
            }),
            ViewCommand::new("removePane", |v, p: DockPaneRequest, w, cx| {
                let pane = v.pane(&p.pane)?;
                v.location(PanelId::from(pane.entity_id()), cx)?;
                v.area.update(cx, |a, cx| a.remove_panel(pane, w, cx));
                Ok(())
            }),
            ViewCommand::new("movePane", |v, p: DockMovePane, w, cx| {
                let id = PanelId::from(v.pane(&p.pane)?.entity_id());
                v.location(id, cx)?;
                let target = match p.target {
                    DockMoveTarget::Tabs {
                        anchor,
                        index,
                        activate,
                    } => {
                        let (_, node) =
                            v.location(PanelId::from(v.pane(&anchor)?.entity_id()), cx)?;
                        let group = v
                            .area
                            .read(cx)
                            .tab_group(node)
                            .ok_or("target is not a tab group")?;
                        if index.is_some_and(|i| i > group.read(cx).panels().len()) {
                            return Err("tab insertion index is out of range".into());
                        }
                        InsertTarget::Tabs {
                            node,
                            ix: index,
                            activate,
                        }
                    }
                    DockMoveTarget::Split { anchor, side, size } => {
                        if anchor == p.pane {
                            return Err("a pane cannot split against itself".into());
                        }
                        if let Some(n) = size {
                            length(n)?;
                        }
                        let (_, node) =
                            v.location(PanelId::from(v.pane(&anchor)?.entity_id()), cx)?;
                        InsertTarget::Split {
                            node,
                            placement: side.into(),
                            size: size.map(px),
                        }
                    }
                    DockMoveTarget::Tile { region, bounds } => {
                        bounds.validate()?;
                        InsertTarget::Tile {
                            node: v.canvas(region, cx)?,
                            bounds: bounds.native(),
                        }
                    }
                };
                v.area.update(cx, |a, cx| a.move_panel(id, target, w, cx));
                Ok(())
            }),
            ViewCommand::new("selectTab", |v, p: DockPaneRequest, w, cx| {
                let id = PanelId::from(v.pane(&p.pane)?.entity_id());
                let (_, node) = v.location(id, cx)?;
                let group = v
                    .area
                    .read(cx)
                    .tab_group(node)
                    .ok_or("pane is not in a tab group")?;
                let index = group
                    .read(cx)
                    .panels()
                    .iter()
                    .position(|p| p.panel_id(cx) == id)
                    .ok_or("pane is missing from its tab group")?;
                group.update(cx, |g, cx| g.select_tab(index, w, cx));
                Ok(())
            }),
            ViewCommand::new("zoomPane", |v, p: DockPaneRequest, w, cx| {
                let pane = v.pane(&p.pane)?;
                if !pane.read(cx).zoomable(cx) || !pane.read(cx).visible(cx) {
                    return Err("pane must be visible and allow zoom".into());
                }
                let id = PanelId::from(pane.entity_id());
                let (_, node) = v.location(id, cx)?;
                if let Some(group) = v.area.read(cx).tab_group(node) {
                    let index = group
                        .read(cx)
                        .panels()
                        .iter()
                        .position(|p| p.panel_id(cx) == id)
                        .expect("current group member");
                    group.update(cx, |g, cx| g.select_tab(index, w, cx));
                    v.area.update(cx, |a, cx| a.set_zoomed_in(node, w, cx));
                } else {
                    let (state, _) = v.tiles(&p.pane, cx)?;
                    if state.read(cx).zoomed_tile() != Some(id) {
                        state.update(cx, |s, cx| s.toggle_zoom(id, w, cx));
                    }
                }
                Ok(())
            }),
            ViewCommand::new("zoomOut", |v, _: (), w, cx| {
                v.area.update(cx, |a, cx| a.set_zoomed_out(w, cx));
                Ok(())
            }),
            ViewCommand::new("toggleDock", |v, p: DockRegionRequest, w, cx| {
                if p.region == DockRegion::Center || !v.area.read(cx).has_dock(p.region.into()) {
                    return Err("toggleDock needs an existing side or bottom dock".into());
                }
                v.area
                    .update(cx, |a, cx| a.toggle_dock(p.region.into(), w, cx));
                Ok(())
            }),
            ViewCommand::new("resizeDock", |v, p: DockResize, w, cx| {
                length(p.size)?;
                if !v.area.read(cx).has_dock(p.region.into()) {
                    return Err("dock region does not exist".into());
                }
                v.area.update(cx, |a, cx| {
                    a.set_dock_size(p.region.into(), px(p.size), w, cx)
                });
                Ok(())
            }),
            ViewCommand::new("setDockCollapsible", |v, p: DockCollapsible, w, cx| {
                if !v.area.read(cx).has_dock(p.region.into()) {
                    return Err("dock region does not exist".into());
                }
                v.area.update(cx, |a, cx| {
                    a.set_dock_collapsible(p.region.into(), p.collapsible, w, cx)
                });
                Ok(())
            }),
            ViewCommand::new("setTileBounds", |v, p: DockSetTileBounds, _, cx| {
                p.bounds.validate()?;
                let (state, tile) = v.tiles(&p.pane, cx)?;
                if state.update(cx, |s, cx| {
                    s.set_bounds(tile.panel_id(), p.bounds.native(), cx)
                }) {
                    Ok(())
                } else {
                    Err("tile is being moved or resized".into())
                }
            }),
            ViewCommand::new("bringTileToFront", |v, p: DockPaneRequest, w, cx| {
                let (_, tile) = v.tiles(&p.pane, cx)?;
                tile.bring_to_front(w, cx);
                Ok(())
            }),
            ViewCommand::new("undoTiles", |v, p: DockPaneRequest, _, cx| {
                let (state, _) = v.tiles(&p.pane, cx)?;
                state.update(cx, |s, cx| s.undo(cx));
                Ok(())
            }),
            ViewCommand::new("redoTiles", |v, p: DockPaneRequest, _, cx| {
                let (state, _) = v.tiles(&p.pane, cx)?;
                state.update(cx, |s, cx| s.redo(cx));
                Ok(())
            }),
        ]
    }
}
impl Render for DockArea {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.area.clone()
    }
}
pub(super) fn definition() -> ComponentDefinition {
    ComponentDefinition::view::<DockArea>("DockArea").with_contract(concat!(
        include_str!("dock.rs"),
        include_str!("dock_layout.rs")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EventPayload, components::test_support::Fixture};
    use gpui::{TestAppContext, point, size};
    fn p<T: serde::de::DeserializeOwned>(v: &str) -> T {
        crate::native::decode_json(v.as_bytes()).unwrap()
    }
    fn events(f: &Fixture<DockArea>) -> Vec<DockPaneEvent> {
        let mut result = vec![];
        while let Some(e) = f.runtime.take_event().unwrap() {
            if let EventPayload::Extension {
                event_id: 1,
                fields,
                ..
            } = e.payload
            {
                let crate::protocol::ExtensionValue::Bytes(bytes) = &fields[0].value else {
                    panic!("typed event")
                };
                result.push(crate::native::decode_json(bytes).unwrap());
            }
        }
        result
    }
    #[gpui::test]
    fn dock_moves_restore_and_metadata_keep_pane_entities_and_tile_history(
        cx: &mut TestAppContext,
    ) {
        let props: DockAreaProps = p(r#"{"panes":[
            {"name":"a","title":"Alpha","data":{"counter":7},"menu":{"items":[{"kind":"item","id":"first","label":"First"},{"kind":"item","id":"second","label":"Second"}]}},
            {"name":"b","title":"Beta"},{"name":"c","title":"Gamma"}],
            "initialLayout":{"center":{"kind":"tabs","panes":["a","b"]},"right":{"layout":{"kind":"tiles","panes":[{"pane":"c","bounds":{"x":10,"y":10,"width":200,"height":150}}]},"size":240}}}"#);
        DockArea::validate_props(&props).unwrap();
        let f = Fixture::<DockArea>::new(props, cx);
        cx.run_until_parked();
        let (a, b, c, group) = f.update(cx, |v, _, cx| {
            let a = v.panes["a"].clone();
            let b = v.panes["b"].clone();
            let c = v.panes["c"].clone();
            let (_, group) = v.location(PanelId::from(a.entity_id()), cx).unwrap();
            (a, b, c, group)
        });
        events(&f);
        let menu = f.update(cx, |_, w, cx| {
            PopupMenu::build(w, cx, |menu, w, cx| {
                a.update(cx, |a, cx| a.dropdown_menu(menu, w, cx))
            })
        });
        f.update(cx, |v, w, cx| {
            menu.update(cx, |m, cx| m.select_index(Some(1), cx));
            let mut props = v.props.clone();
            props.panes.reverse();
            props.panes[2].title = "Renamed".into();
            props.panes[2].menu.items.swap(0, 1);
            v.update(props, w, cx);
            assert_eq!(a.entity_id(), v.panes["a"].entity_id());
            assert_eq!(b.entity_id(), v.panes["b"].entity_id());
            assert_eq!(c.entity_id(), v.panes["c"].entity_id());
            assert_eq!(
                group,
                v.location(PanelId::from(a.entity_id()), cx).unwrap().1
            );
            assert_eq!(menu.read(cx).selected_index(), Some(0));
            let node = v.canvas(DockRegion::Right, cx).unwrap();
            v.area.update(cx, |a, cx| {
                a.move_panel(
                    PanelId::from(b.entity_id()),
                    InsertTarget::Tile {
                        node,
                        bounds: gpui::Bounds::new(
                            point(px(30.), px(50.)),
                            size(px(220.), px(140.)),
                        ),
                    },
                    w,
                    cx,
                )
            });
        });
        cx.run_until_parked();
        assert!(
            !events(&f)
                .iter()
                .any(|e| matches!(e, DockPaneEvent::Removed { .. }))
        );
        f.update(cx, |v, _, cx| {
            let (state, tile) = v.tiles("b", cx).unwrap();
            assert!(state.update(cx, |s, cx| {
                s.set_bounds(
                    tile.panel_id(),
                    DockBounds {
                        x: 80.,
                        y: 90.,
                        width: 250.,
                        height: 180.,
                    }
                    .native(),
                    cx,
                )
            }));
        });
        cx.run_until_parked();
        f.update(cx, |v, _, cx| {
            assert_eq!(v.tiles("b", cx).unwrap().1.bounds().origin.x, px(80.));
            let (state, _) = v.tiles("b", cx).unwrap();
            state.update(cx, |s, cx| s.undo(cx));
        });
        cx.run_until_parked();
        f.update(cx, |v, _, cx| {
            assert_eq!(v.tiles("b", cx).unwrap().1.bounds().origin.x, px(30.));
            let (state, _) = v.tiles("b", cx).unwrap();
            state.update(cx, |s, cx| s.redo(cx));
        });
        cx.run_until_parked();
        f.update(cx, |v, w, cx| {
            let saved = v.snapshot(cx).unwrap();
            let json = crate::native::encode_json(&saved).unwrap();
            let saved: DockSnapshot = crate::native::decode_json(&json).unwrap();
            v.replace_layout(saved.layout, saved.version, Some(saved.data), w, cx)
                .unwrap();
            assert_eq!(a.entity_id(), v.panes["a"].entity_id());
            assert_eq!(b.entity_id(), v.panes["b"].entity_id());
            assert_eq!(v.tiles("b", cx).unwrap().1.bounds().origin.x, px(80.));
            assert_eq!(a.read(cx).data, p::<DockValue>(r#"{"counter":7}"#));
            let before = crate::native::encode_json(&v.snapshot(cx).unwrap()).unwrap();
            let bad: DockLayoutSpec = p(r#"{"center":{"kind":"tabs","panes":["missing"]}}"#);
            assert!(v.replace_layout(bad, None, None, w, cx).is_err());
            assert_eq!(
                before,
                crate::native::encode_json(&v.snapshot(cx).unwrap()).unwrap()
            );
        });
        cx.run_until_parked();
        assert!(
            !events(&f)
                .iter()
                .any(|e| matches!(e, DockPaneEvent::Removed { .. }))
        );
        cx.update_window(f.window.into(), |_, w, cx| w.draw(cx).clear(cx))
            .unwrap();
        f.update(cx, |v, w, cx| {
            v.area.update(cx, |a, cx| a.remove_panel(b.clone(), w, cx))
        });
        cx.run_until_parked();
        assert_eq!(
            events(&f)
                .iter()
                .filter(|e| matches!(e,DockPaneEvent::Removed {pane} if pane=="b"))
                .count(),
            1
        );
        f.update(cx, |v, w, cx| {
            v.update(v.props.clone(), w, cx);
            assert!(v.location(PanelId::from(b.entity_id()), cx).is_err());
        });
    }
}
