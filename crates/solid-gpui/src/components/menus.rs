//! Menu data is compiled once per revision; open menus retain native entities.
use super::{ButtonVariant, ControlSize, popups::PopupAnchor};
use crate::native::{
    ComponentDefinition, Event, EventDefinition, NativeChildren, NativeSlot, NativeView, TS,
    ViewCommand,
};
use gpui::{
    App, Context, DismissEvent, Entity, Focusable, InteractiveElement, IntoElement, ParentElement,
    Render, Subscription, WeakEntity, Window, div, prelude::FluentBuilder, px,
};
use gpui_component::{
    Disableable, Selectable, Sizable,
    button::{Button, ButtonVariants, DropdownButton as NativeSplit},
    menu::{ContextMenuExt, DropdownMenu as _, PopupMenu as NativeMenu, PopupMenuItem},
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[crate::native_type]
#[derive(Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MenuItem {
    Separator,
    Label {
        label: String,
    },
    Item {
        id: String,
        label: String,
        #[serde(default)]
        icon: Option<String>,
        #[serde(default)]
        disabled: bool,
        #[serde(default)]
        checked: bool,
    },
    Link {
        id: String,
        label: String,
        href: String,
        #[serde(default)]
        icon: Option<String>,
        #[serde(default)]
        disabled: bool,
    },
    Element {
        id: String,
        index: usize,
        #[serde(default)]
        icon: Option<String>,
        #[serde(default)]
        disabled: bool,
        #[serde(default)]
        checked: bool,
    },
    Submenu {
        id: String,
        label: String,
        menu: Box<MenuSpec>,
        #[serde(default)]
        icon: Option<String>,
        #[serde(default)]
        disabled: bool,
    },
}
impl MenuItem {
    fn key(&self) -> Option<&str> {
        match self {
            Self::Item { id, .. }
            | Self::Link { id, .. }
            | Self::Element { id, .. }
            | Self::Submenu { id, .. } => Some(id),
            _ => None,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MenuCheckSide {
    #[default]
    Left,
    Right,
}
#[crate::native_type]
#[derive(Clone, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct MenuSpec {
    pub items: Vec<MenuItem>,
    pub min_width: f32,
    pub max_width: f32,
    pub max_height: f32,
    pub scrollable: bool,
    pub check_side: MenuCheckSide,
    pub external_link_icon: bool,
}
impl Default for MenuSpec {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            min_width: 120.,
            max_width: 500.,
            max_height: 450.,
            scrollable: false,
            check_side: MenuCheckSide::Left,
            external_link_icon: true,
        }
    }
}
#[crate::native_type]
#[derive(Clone)]
pub struct MenuSelection {
    pub id: String,
}

pub(super) fn validate_menu(menu: &MenuSpec, child_count: Option<usize>) -> Result<(), String> {
    fn visit(
        menu: &MenuSpec,
        depth: usize,
        count: &mut usize,
        ids: &mut HashSet<String>,
        children: Option<usize>,
    ) -> Result<(), String> {
        *count += menu.items.len();
        if depth > 16 || *count > 1024 {
            return Err("menus support at most 1024 items and 16 submenu levels".into());
        }
        if ![menu.min_width, menu.max_width, menu.max_height]
            .iter()
            .all(|v| v.is_finite() && (1.0..=1_000_000.0).contains(v))
            || menu.min_width > menu.max_width
        {
            return Err("menu dimensions require 1 <= minWidth <= maxWidth <= 1000000 and a positive maxHeight".into());
        }
        for item in &menu.items {
            if let Some(id) = item.key()
                && (id.is_empty() || !ids.insert(id.to_owned()))
            {
                return Err(
                    "menu item ids must be non-empty and unique across the menu tree".into(),
                );
            }
            match item {
                MenuItem::Submenu { menu: nested, .. } => {
                    if menu.scrollable {
                        return Err("native scrollable menus do not support submenus".into());
                    }
                    visit(nested, depth + 1, count, ids, children)?;
                }
                MenuItem::Element { index, .. }
                    if children.is_some_and(|count| *index >= count) =>
                {
                    return Err("custom menu item index is outside the supplied children".into());
                }
                _ => {}
            }
        }
        Ok(())
    }
    visit(menu, 0, &mut 0, &mut HashSet::new(), child_count)
}
fn catalog(spec: &MenuSpec) -> HashMap<String, Rc<MenuSpec>> {
    fn visit(id: &str, spec: &MenuSpec, result: &mut HashMap<String, Rc<MenuSpec>>) {
        result.insert(id.to_owned(), Rc::new(spec.clone()));
        for item in &spec.items {
            if let MenuItem::Submenu { id, menu, .. } = item {
                visit(id, menu, result);
            }
        }
    }
    let mut result = HashMap::new();
    visit("", spec, &mut result);
    result
}
struct LiveMenu {
    view: WeakEntity<NativeMenu>,
    _dismiss: Option<Subscription>,
}
pub(super) struct MenuModel {
    specs: RefCell<HashMap<String, Rc<MenuSpec>>>,
    live: RefCell<HashMap<String, LiveMenu>>,
    content: NativeSlot,
    select: Rc<dyn Fn(MenuSelection)>,
    dismiss: Rc<dyn Fn()>,
}
impl MenuModel {
    fn new(spec: &MenuSpec, content: NativeSlot, event: Event<MenuSelection>) -> Rc<Self> {
        let dismiss = event.related::<()>("dismiss");
        Self::with_callbacks(
            spec,
            content,
            Rc::new(move |v| event.emit(v)),
            Rc::new(move || dismiss.emit(())),
        )
    }
    pub(super) fn with_callbacks(
        spec: &MenuSpec,
        content: NativeSlot,
        select: Rc<dyn Fn(MenuSelection)>,
        dismiss: Rc<dyn Fn()>,
    ) -> Rc<Self> {
        Rc::new(Self {
            specs: RefCell::new(catalog(spec)),
            live: RefCell::new(HashMap::new()),
            content,
            select,
            dismiss,
        })
    }
    pub(super) fn compile(
        self: &Rc<Self>,
        mut menu: NativeMenu,
        key: &str,
        window: &mut Window,
        cx: &mut App,
    ) -> NativeMenu {
        let spec = self.specs.borrow()[key].clone();
        if !self
            .live
            .borrow()
            .get(key)
            .is_some_and(|m| m.view.entity_id() == menu.owner().entity_id())
        {
            let subscription = key.is_empty().then(|| {
                let dismiss = self.dismiss.clone();
                cx.subscribe(
                    &menu.owner().upgrade().expect("live menu builder"),
                    move |_, _: &DismissEvent, _| dismiss(),
                )
            });
            self.live.borrow_mut().insert(
                key.to_owned(),
                LiveMenu {
                    view: menu.owner(),
                    _dismiss: subscription,
                },
            );
        }
        menu = menu
            .min_w(px(spec.min_width))
            .max_w(px(spec.max_width))
            .max_h(px(spec.max_height))
            .scrollable(spec.scrollable)
            .external_link_icon(spec.external_link_icon)
            .check_side(match spec.check_side {
                MenuCheckSide::Left => gpui_component::Side::Left,
                MenuCheckSide::Right => gpui_component::Side::Right,
            });
        for item in &spec.items {
            let (mut native, icon, disabled, checked) = match item {
                MenuItem::Separator => {
                    menu = menu.item(PopupMenuItem::separator());
                    continue;
                }
                MenuItem::Label { label } => {
                    menu = menu.item(PopupMenuItem::label(label.clone()));
                    continue;
                }
                MenuItem::Item {
                    label,
                    icon,
                    disabled,
                    checked,
                    ..
                } => (PopupMenuItem::new(label.clone()), icon, *disabled, *checked),
                MenuItem::Link {
                    label,
                    href,
                    icon,
                    disabled,
                    ..
                } => (
                    PopupMenuItem::link(label.clone(), href.clone()),
                    icon,
                    *disabled,
                    false,
                ),
                MenuItem::Element {
                    index,
                    icon,
                    disabled,
                    checked,
                    ..
                } => {
                    let slot = self.content.item(*index);
                    (
                        PopupMenuItem::element(move |_, _| slot.clone()),
                        icon,
                        *disabled,
                        *checked,
                    )
                }
                MenuItem::Submenu {
                    id,
                    label,
                    icon,
                    disabled,
                    ..
                } => {
                    let existing = self.live.borrow().get(id).and_then(|m| m.view.upgrade());
                    let submenu = existing.unwrap_or_else(|| {
                        let model = self.clone();
                        let id = id.clone();
                        NativeMenu::build(window, cx, move |menu, window, cx| {
                            model.compile(menu, &id, window, cx)
                        })
                    });
                    menu.adopt_submenu(&submenu, cx);
                    (
                        PopupMenuItem::submenu(label.clone(), submenu),
                        icon,
                        *disabled,
                        false,
                    )
                }
            };
            native = native.disabled(disabled).checked(checked);
            if let Some(path) = icon {
                native = native.icon(gpui_component::Icon::default().path(path.clone()));
            }
            if !matches!(item, MenuItem::Submenu { .. }) {
                let id = item.key().unwrap().to_owned();
                let href = match item {
                    MenuItem::Link { href, .. } => Some(href.clone()),
                    _ => None,
                };
                let select = self.select.clone();
                native = native.on_click(move |_, _, cx| {
                    if let Some(href) = &href {
                        cx.open_url(href);
                    }
                    select(MenuSelection { id: id.clone() });
                });
            }
            menu = menu.item(native);
        }
        menu
    }
    pub(super) fn replace(self: &Rc<Self>, spec: &MenuSpec, window: &mut Window, cx: &mut App) {
        let old = self.specs.replace(catalog(spec));
        let live: Vec<_> = self
            .live
            .borrow()
            .iter()
            .filter_map(|(id, m)| m.view.upgrade().map(|m| (id.clone(), m)))
            .collect();
        let root = self.live.borrow().get("").and_then(|m| m.view.upgrade());
        let removed_focus = live.iter().any(|(id, menu)| {
            !self.specs.borrow().contains_key(id)
                && menu.focus_handle(cx).contains_focused(window, cx)
        });
        if removed_focus && let Some(root) = root {
            root.focus_handle(cx).focus(window, cx);
        }
        self.live
            .borrow_mut()
            .retain(|id, _| self.specs.borrow().contains_key(id));
        for (id, menu) in live {
            let Some(spec) = self.specs.borrow().get(&id).cloned() else {
                continue;
            };
            let selected = menu
                .read(cx)
                .selected_index()
                .and_then(|ix| old.get(&id)?.items.get(ix)?.key())
                .map(str::to_owned);
            let next_index = selected
                .and_then(|key| spec.items.iter().position(|item| item.key() == Some(&key)));
            menu.update(cx, |menu, cx| {
                menu.rebuild(window, cx, |menu, window, cx| {
                    self.compile(menu, &id, window, cx)
                });
                menu.select_index(next_index, cx);
            });
        }
    }
    pub(super) fn notify(&self, cx: &mut App) {
        let menus: Vec<_> = self
            .live
            .borrow()
            .values()
            .filter_map(|m| m.view.upgrade())
            .collect();
        for menu in menus {
            menu.update(cx, |_, cx| cx.notify());
        }
    }
    pub(super) fn close(&self, window: &mut Window, cx: &mut App) {
        let root = self.live.borrow().get("").and_then(|m| m.view.upgrade());
        if let Some(root) = root {
            root.update(cx, |menu, cx| menu.close(window, cx));
        }
    }
}
#[crate::native_type]
#[derive(Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct PopupMenuProps {
    pub menu: MenuSpec,
}
#[crate::native_type]
#[derive(Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ContextMenuProps {
    pub menu: MenuSpec,
}
macro_rules! dropdown_props {
    ($name:ident) => {
        #[crate::native_type]
        #[derive(Clone, Default)]
        #[serde(default, rename_all = "camelCase")]
        pub struct $name {
            pub menu: MenuSpec,
            pub label: Option<String>,
            pub icon: Option<String>,
            pub variant: ButtonVariant,
            pub size: ControlSize,
            pub disabled: bool,
            pub loading: bool,
            pub selected: bool,
            pub outline: bool,
            pub anchor: PopupAnchor,
        }
    };
}
dropdown_props!(DropdownMenuProps);
dropdown_props!(DropdownButtonProps);
trait MenuKind: Sized + 'static {
    type Props: Clone + serde::de::DeserializeOwned + TS + 'static;
    const INLINE: bool = false;
    const SPLIT: bool = false;
    const OPEN_CHANGE: bool = false;
    fn spec(props: &Self::Props) -> &MenuSpec;
    fn render(
        view: &MenuView<Self>,
        window: &mut Window,
        cx: &mut Context<MenuView<Self>>,
    ) -> gpui::AnyElement;
}
struct PopupMenu;
struct ContextMenu;
struct DropdownMenu;
struct DropdownButton;
struct MenuView<M: MenuKind> {
    event: Event<MenuSelection>,
    props: M::Props,
    children: NativeChildren,
    model: Rc<MenuModel>,
    inline: Option<Entity<NativeMenu>>,
}
impl MenuKind for PopupMenu {
    type Props = PopupMenuProps;
    const INLINE: bool = true;
    fn spec(p: &Self::Props) -> &MenuSpec {
        &p.menu
    }
    fn render(
        view: &MenuView<Self>,
        _: &mut Window,
        _: &mut Context<MenuView<Self>>,
    ) -> gpui::AnyElement {
        view.inline.as_ref().unwrap().clone().into_any_element()
    }
}
impl MenuKind for ContextMenu {
    type Props = ContextMenuProps;
    fn spec(p: &Self::Props) -> &MenuSpec {
        &p.menu
    }
    fn render(
        view: &MenuView<Self>,
        _: &mut Window,
        _: &mut Context<MenuView<Self>>,
    ) -> gpui::AnyElement {
        let model = view.model.clone();
        div()
            .id("trigger")
            .child(view.children.slot("trigger"))
            .context_menu(move |menu, window, cx| model.compile(menu, "", window, cx))
            .into_any_element()
    }
}
macro_rules! dropdown_kind {
    ($kind:ident, $props:ident, $split:expr) => {
        impl MenuKind for $kind {
            type Props = $props;
            const SPLIT: bool = $split;
            const OPEN_CHANGE: bool = !$split;
            fn spec(p: &Self::Props) -> &MenuSpec {
                &p.menu
            }
            fn render(
                view: &MenuView<Self>,
                _: &mut Window,
                _: &mut Context<MenuView<Self>>,
            ) -> gpui::AnyElement {
                let p = &view.props;
                let model = view.model.clone();
                let event = view.event.clone();
                let button = Button::new("trigger")
                    .with_variant(p.variant.into())
                    .with_size(p.size)
                    .disabled(p.disabled)
                    .loading(p.loading)
                    .selected(p.selected)
                    .when(p.outline, |b| b.outline())
                    .when_some(p.label.clone(), |b, label| b.label(label))
                    .when_some(p.icon.clone(), |b, path| {
                        b.icon(gpui_component::Icon::default().path(path))
                    })
                    .child(view.children.slot("trigger"));
                if $split {
                    NativeSplit::new("split")
                        .button(
                            button.on_click(move |_, _, _| event.related::<()>("press").emit(())),
                        )
                        .disabled(p.disabled)
                        .dropdown_menu_with_anchor(
                            gpui::Anchor::from(p.anchor),
                            move |menu, window, cx| model.compile(menu, "", window, cx),
                        )
                        .into_any_element()
                } else {
                    button
                        .dropdown_menu_with_anchor(
                            gpui::Anchor::from(p.anchor),
                            move |menu, window, cx| model.compile(menu, "", window, cx),
                        )
                        .on_open_change(move |open, _, _| {
                            event.related::<bool>("openChange").emit(*open)
                        })
                        .into_any_element()
                }
            }
        }
    };
}
dropdown_kind!(DropdownMenu, DropdownMenuProps, false);
dropdown_kind!(DropdownButton, DropdownButtonProps, true);
impl<M: MenuKind> NativeView for MenuView<M> {
    type Props = M::Props;
    type Event = MenuSelection;
    fn event_name() -> &'static str {
        "select"
    }
    fn accepts_children() -> bool {
        true
    }
    fn slots() -> &'static [&'static str] {
        if M::INLINE { &[] } else { &["trigger"] }
    }
    fn additional_events() -> Vec<EventDefinition> {
        let mut events = vec![EventDefinition::new::<()>("dismiss")];
        if M::SPLIT {
            events.push(EventDefinition::new::<()>("press"));
        }
        // Only the native DropdownMenu exposes an open-change callback.
        if M::OPEN_CHANGE {
            events.push(EventDefinition::new::<bool>("openChange"));
        }
        events
    }
    fn validate_props(p: &Self::Props) -> Result<(), String> {
        validate_menu(M::spec(p), None)
    }
    fn validate_children(
        p: &Self::Props,
        children: &crate::ExtensionChildSummary,
    ) -> Result<(), String> {
        validate_menu(M::spec(p), Some(children.content_count()))
    }
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        children: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let model = MenuModel::new(M::spec(&props), children.content(), event.clone());
        let inline = M::INLINE.then(|| {
            NativeMenu::build(window, cx, |menu, window, cx| {
                model.compile(menu, "", window, cx)
            })
        });
        Self {
            event,
            props,
            children,
            model,
            inline,
        }
    }
    fn update(&mut self, props: Self::Props, window: &mut Window, cx: &mut Context<Self>) {
        if M::spec(&props) != M::spec(&self.props) {
            self.model.replace(M::spec(&props), window, cx);
        } else {
            self.model.notify(cx);
        }
        self.props = props;
        cx.notify();
    }
    fn unmount(&mut self, window: &mut Window, cx: &mut App) {
        self.model.close(window, cx);
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("dismiss", |view: &mut Self, _: (), window, cx| {
                view.model.close(window, cx);
                Ok(())
            }),
            ViewCommand::new("focus", |view: &mut Self, _: (), window, cx| {
                let menu = view
                    .model
                    .live
                    .borrow()
                    .get("")
                    .and_then(|m| m.view.upgrade())
                    .ok_or("menu is not open")?;
                menu.focus_handle(cx).focus(window, cx);
                Ok(())
            }),
        ]
    }
}
impl<M: MenuKind> Render for MenuView<M> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        M::render(self, window, cx)
    }
}
pub(super) fn definitions() -> Vec<ComponentDefinition> {
    vec![
        ComponentDefinition::view::<MenuView<PopupMenu>>("PopupMenu"),
        ComponentDefinition::view::<MenuView<ContextMenu>>("ContextMenu"),
        ComponentDefinition::view::<MenuView<DropdownMenu>>("DropdownMenu"),
        ComponentDefinition::view::<MenuView<DropdownButton>>("DropdownButton"),
        ComponentDefinition::view::<AppMenuBar>("AppMenuBar"),
    ]
    .into_iter()
    .map(|d| d.with_contract(include_str!("menus.rs")))
    .collect()
}

#[crate::native_type]
#[derive(Default)]
pub struct AppMenuBarProps {}
struct AppMenuBar {
    native: Entity<gpui_component::menu::AppMenuBar>,
}
impl NativeView for AppMenuBar {
    type Props = AppMenuBarProps;
    type Event = ();
    fn mount(
        _: Self::Props,
        _: Event<()>,
        _: NativeChildren,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            native: gpui_component::menu::AppMenuBar::new(cx),
        }
    }
    fn update(&mut self, _: Self::Props, _: &mut Window, _: &mut Context<Self>) {}
}
impl Render for AppMenuBar {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.native.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_support::Fixture;
    fn item(id: &str) -> MenuItem {
        MenuItem::Item {
            id: id.into(),
            label: id.into(),
            icon: None,
            disabled: false,
            checked: false,
        }
    }
    #[gpui::test]
    fn open_menu_reconciles_keyboard_cursor_and_submenus_by_key(cx: &mut gpui::TestAppContext) {
        let spec = MenuSpec {
            items: vec![
                MenuItem::Label {
                    label: "Group".into(),
                },
                item("a"),
                MenuItem::Submenu {
                    id: "nested".into(),
                    label: "Nested".into(),
                    menu: Box::new(MenuSpec {
                        items: vec![item("inside")],
                        ..Default::default()
                    }),
                    icon: None,
                    disabled: false,
                },
            ],
            ..Default::default()
        };
        let fixture = Fixture::<MenuView<PopupMenu>>::new(PopupMenuProps { menu: spec }, cx);
        fixture.update(cx, |view, window, cx| {
            let menu = view.inline.clone().unwrap();
            let submenu = view.model.live.borrow()["nested"].view.upgrade().unwrap();
            menu.update(cx, |menu, cx| menu.select_index(Some(1), cx));
            let mut p = view.props.clone();
            p.menu.items.swap(1, 2);
            view.update(p.clone(), window, cx);
            assert_eq!(menu.entity_id(), view.inline.as_ref().unwrap().entity_id());
            assert_eq!(menu.read(cx).selected_index(), Some(2));
            assert_eq!(
                submenu.entity_id(),
                view.model.live.borrow()["nested"].view.entity_id()
            );
            submenu.focus_handle(cx).focus(window, cx);
            p.menu.items.remove(1);
            view.update(p.clone(), window, cx);
            assert!(menu.focus_handle(cx).is_focused(window));
            assert_eq!(menu.read(cx).selected_index(), Some(1));
            let MenuItem::Item { disabled, .. } = &mut p.menu.items[1] else {
                unreachable!()
            };
            *disabled = true;
            view.update(p, window, cx);
            assert_eq!(menu.read(cx).selected_index(), None);
        });
    }
}
