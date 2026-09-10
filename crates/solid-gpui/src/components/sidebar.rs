use super::icon_source::ComponentIcon;
use crate::native::{ElementContext, Event, NativeChild, NativeItems, NativeSlot};
use gpui::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, StyleRefinement, Styled, Window,
    prelude::FluentBuilder,
};
use gpui_component::{Collapsible, StyledExt, sidebar as native};
use native::SidebarItem;
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum SidebarSide {
    #[default]
    Left,
    Right,
}
impl From<SidebarSide> for gpui_component::Side {
    fn from(v: SidebarSide) -> Self {
        match v {
            SidebarSide::Left => Self::Left,
            SidebarSide::Right => Self::Right,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum SidebarCollapse {
    #[default]
    Icon,
    Offcanvas,
    None,
}
#[derive(Clone, IntoElement)]
pub struct SidebarPart {
    id: ElementId,
    kind: SidebarKind,
}
#[derive(Clone)]
enum SidebarKind {
    Menu(native::SidebarMenu),
    Group(
        native::SidebarGroup<NativeChild<SidebarPart>>,
        StyleRefinement,
    ),
}
impl Collapsible for SidebarPart {
    fn is_collapsed(&self) -> bool {
        match &self.kind {
            SidebarKind::Menu(v) => v.is_collapsed(),
            SidebarKind::Group(v, _) => v.is_collapsed(),
        }
    }
    fn collapsed(mut self, value: bool) -> Self {
        self.kind = match self.kind {
            SidebarKind::Menu(v) => SidebarKind::Menu(v.collapsed(value)),
            SidebarKind::Group(v, s) => SidebarKind::Group(v.collapsed(value), s),
        };
        self
    }
}
impl SidebarItem for SidebarPart {
    fn render(
        self,
        _: impl Into<ElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        match self.kind {
            SidebarKind::Menu(v) => SidebarItem::render(v, self.id, window, cx).into_any_element(),
            SidebarKind::Group(v, s) => gpui::div()
                .refine_style(&s)
                .child(SidebarItem::render(v, self.id, window, cx))
                .into_any_element(),
        }
    }
}
impl Styled for SidebarPart {
    fn style(&mut self) -> &mut StyleRefinement {
        match &mut self.kind {
            SidebarKind::Menu(v) => v.style(),
            SidebarKind::Group(_, s) => s,
        }
    }
}
impl RenderOnce for SidebarPart {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        SidebarItem::render(self, "standalone", window, cx)
    }
}
#[derive(Clone, IntoElement)]
pub struct SidebarEntry(native::SidebarMenuItem);
impl RenderOnce for SidebarEntry {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        SidebarItem::render(self.0, "standalone", window, cx)
    }
}
impl Styled for SidebarEntry {
    fn style(&mut self) -> &mut StyleRefinement {
        self.0.style()
    }
}
#[crate::native_module(name = "gpui-component")]
mod exports {
    use super::*;
    #[component]
    pub fn sidebar(
        #[prop(default)] side: SidebarSide,
        #[prop(default)] collapsed: bool,
        #[prop(default)] collapsible: SidebarCollapse,
        header: NativeSlot,
        footer: NativeSlot,
        children: NativeItems<SidebarPart>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        native::Sidebar::new(cx.id())
            .side(side.into())
            .collapsed(collapsed)
            .collapsible(match collapsible {
                SidebarCollapse::Icon => native::SidebarCollapsible::Icon,
                SidebarCollapse::Offcanvas => native::SidebarCollapsible::Offcanvas,
                SidebarCollapse::None => native::SidebarCollapsible::None,
            })
            .when(!header.is_empty(), |v| v.header(header))
            .when(!footer.is_empty(), |v| v.footer(footer))
            .children(children)
    }
    #[component]
    pub fn sidebar_group(
        label: String,
        children: NativeItems<SidebarPart>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        SidebarPart {
            id: cx.id(),
            kind: SidebarKind::Group(
                native::SidebarGroup::new(label).children(children),
                StyleRefinement::default(),
            ),
        }
    }
    #[component]
    pub fn sidebar_menu(
        children: NativeItems<SidebarEntry>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        SidebarPart {
            id: cx.id(),
            kind: SidebarKind::Menu(
                native::SidebarMenu::new()
                    .children(children.into_iter().map(|c| c.map_native(|c| c.0))),
            ),
        }
    }
    #[component]
    pub fn sidebar_menu_item(
        label: String,
        #[prop(default)] icon: Option<ComponentIcon>,
        #[prop(default)] active: bool,
        #[prop(default)] disabled: bool,
        #[prop(default)] default_open: bool,
        #[prop(default)] click_to_open: bool,
        #[prop(default)] click_to_toggle: bool,
        suffix: NativeSlot,
        children: NativeItems<SidebarEntry>,
        on_press: Event<()>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        let item = native::SidebarMenuItem::new(label)
            .id(cx.id())
            .active(active)
            .disable(disabled)
            .default_open(default_open)
            .click_to_open(click_to_open)
            .click_to_toggle(click_to_toggle)
            .when_some(icon, |v, path| v.icon(path.native()))
            .when(!suffix.is_empty(), |v| v.suffix(move |_, _| suffix.clone()))
            .on_click(move |_, _, _| on_press.emit(()))
            .children(children.into_iter().map(|c| c.map_native(|c| c.0)));
        SidebarEntry(item)
    }
    #[component(children = false)]
    pub fn sidebar_toggle_button(
        #[prop(default)] side: SidebarSide,
        #[prop(default)] collapsed: bool,
        on_press: Event<()>,
    ) -> impl IntoElement {
        native::SidebarToggleButton::new()
            .side(side.into())
            .collapsed(collapsed)
            .on_click(move |_, _, _| on_press.emit(()))
    }
    #[component]
    pub fn sidebar_header(cx: &mut ElementContext) -> impl IntoElement + Styled {
        native::SidebarHeader::new().children(cx.children())
    }
    #[component]
    pub fn sidebar_footer(cx: &mut ElementContext) -> impl IntoElement + Styled {
        native::SidebarFooter::new().children(cx.children())
    }
}
pub use exports::native_module;
