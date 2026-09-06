use crate::{
    ComponentChild, IconName, Sizable, Size, StyledExt,
    group_box::GroupBoxVariant,
    h_resizable,
    input::{Input, InputEvent, InputState},
    resizable_panel,
    setting::{SettingGroup, SettingPage},
    sidebar::{Sidebar, SidebarMenu, SidebarMenuItem},
};
use gpui::{
    App, AppContext as _, Axis, Context, ElementId, Entity, EventEmitter, IntoElement,
    ParentElement as _, Pixels, RenderOnce, SharedString, StyleRefinement, Styled, Subscription,
    Window, container_query, div, prelude::FluentBuilder as _, px, relative,
};
use rust_i18n::t;
use std::ops::Range;
const STACKED_LAYOUT_MAX_WIDTH: Pixels = px(480.);

/// A selection addresses stable page/group keys, independent of filtering and order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingSelection {
    pub page: SharedString,
    pub group: Option<SharedString>,
}
#[derive(Clone)]
pub enum SettingsEvent {
    SelectionChanged(SettingSelection),
    SearchChanged(SharedString),
}
/// Retained search and navigation state, owned by a Settings instance.
pub struct SettingsState {
    preferred: Option<SettingSelection>,
    selected: Option<SettingSelection>,
    pub(super) deferred_scroll_group: Option<SharedString>,
    pub(super) search_input: Entity<InputState>,
    _subscription: Subscription,
}
impl EventEmitter<SettingsEvent> for SettingsState {}
impl SettingsState {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input =
            cx.new(|cx| InputState::new(window, cx).placeholder(t!("Settings.search_placeholder")));
        let subscription = cx.subscribe(&search_input, |_, input, event, cx| {
            if matches!(event, InputEvent::Change) {
                cx.emit(SettingsEvent::SearchChanged(input.read(cx).value()));
                cx.notify();
            }
        });
        Self {
            preferred: None,
            selected: None,
            deferred_scroll_group: None,
            search_input,
            _subscription: subscription,
        }
    }
    pub fn selected(&self) -> Option<&SettingSelection> {
        self.selected.as_ref()
    }
    pub fn query(&self, cx: &App) -> SharedString {
        self.search_input.read(cx).value()
    }
    pub fn set_query(
        &mut self,
        query: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.search_input
            .update(cx, |input, cx| input.set_value(query, window, cx));
        cx.notify();
    }
    pub fn select(&mut self, selection: SettingSelection, cx: &mut Context<Self>) {
        if self.preferred.as_ref() == Some(&selection) && self.selected.as_ref() == Some(&selection)
        {
            return;
        }
        self.deferred_scroll_group = selection.group.clone();
        self.preferred = Some(selection.clone());
        self.selected = Some(selection.clone());
        cx.emit(SettingsEvent::SelectionChanged(selection));
        cx.notify();
    }
    fn reconcile(
        &mut self,
        pages: &[ComponentChild<SettingPage>],
        filtered: &[ComponentChild<SettingPage>],
    ) {
        if let Some(selection) = &mut self.preferred {
            match pages.iter().find(|page| page.key == selection.page) {
                None => self.preferred = None,
                Some(page) => {
                    if selection
                        .group
                        .as_ref()
                        .is_some_and(|key| !page.groups.iter().any(|g| &g.key == key))
                    {
                        selection.group = None;
                    }
                }
            }
        }
        let visible = self
            .preferred
            .as_ref()
            .and_then(|selection| {
                filtered
                    .iter()
                    .find(|page| page.key == selection.page)
                    .map(|page| SettingSelection {
                        page: selection.page.clone(),
                        group: selection
                            .group
                            .as_ref()
                            .filter(|key| page.groups.iter().any(|g| &g.key == *key))
                            .cloned(),
                    })
            })
            .or_else(|| {
                filtered.first().map(|page| SettingSelection {
                    page: page.key.clone(),
                    group: None,
                })
            });
        if self.preferred.is_none() {
            self.preferred = visible.clone();
        }
        self.selected = visible;
    }
}
/// Settings → SettingPage → SettingGroup → SettingItem → field.
#[derive(IntoElement)]
pub struct Settings {
    id: ElementId,
    pages: Vec<ComponentChild<SettingPage>>,
    state: Option<Entity<SettingsState>>,
    group_variant: GroupBoxVariant,
    size: Size,
    sidebar_width: Pixels,
    sidebar_size_range: Range<Pixels>,
    sidebar_style: StyleRefinement,
    header_style: StyleRefinement,
}
impl Settings {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            pages: vec![],
            state: None,
            group_variant: GroupBoxVariant::default(),
            size: Size::default(),
            sidebar_width: px(250.),
            sidebar_size_range: px(160.)..px(360.),
            sidebar_style: StyleRefinement::default(),
            header_style: StyleRefinement::default(),
        }
    }
    pub fn with_state(mut self, state: &Entity<SettingsState>) -> Self {
        self.state = Some(state.clone());
        self
    }
    pub fn sidebar_width(mut self, width: impl Into<Pixels>) -> Self {
        self.sidebar_width = width.into();
        self
    }
    pub fn sidebar_size_range(mut self, range: impl Into<Range<Pixels>>) -> Self {
        self.sidebar_size_range = range.into();
        self
    }
    pub fn page(mut self, page: impl Into<ComponentChild<SettingPage>>) -> Self {
        self.pages.push(page.into());
        self
    }
    pub fn pages<I, P>(mut self, pages: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<ComponentChild<SettingPage>>,
    {
        self.pages.extend(pages.into_iter().map(Into::into));
        self
    }
    pub fn with_group_variant(mut self, variant: GroupBoxVariant) -> Self {
        self.group_variant = variant;
        self
    }
    pub fn sidebar_style(mut self, style: &StyleRefinement) -> Self {
        self.sidebar_style = style.clone();
        self
    }
    pub fn header_style(mut self, style: &StyleRefinement) -> Self {
        self.header_style = style.clone();
        self
    }
    fn filtered_pages(&self, query: &str, cx: &App) -> Vec<ComponentChild<SettingPage>> {
        self.pages
            .iter()
            .filter_map(|page| {
                let groups: Vec<ComponentChild<SettingGroup>> = page
                    .groups
                    .iter()
                    .filter_map(|group| {
                        let mut group = group.clone();
                        group.items.retain(|item| item.is_match(query, cx));
                        (!group.items.is_empty()).then_some(group)
                    })
                    .collect();
                let mut page = page.clone();
                page.groups = groups;
                (!page.groups.is_empty()).then_some(page)
            })
            .collect()
    }
    fn render_active_page(
        &self,
        state: &Entity<SettingsState>,
        pages: &[ComponentChild<SettingPage>],
        options: RenderOptions,
        cx: &App,
    ) -> gpui::AnyElement {
        let selected = state.read(cx).selected();
        if let Some((ix, page)) = pages
            .iter()
            .enumerate()
            .find(|(_, page)| selected.is_some_and(|s| s.page == page.key))
        {
            let state = state.clone();
            return page.clone().render_deferred(move |page, window, cx| {
                page.render(ix, &state, &options, window, cx)
            });
        }
        div().into_any_element()
    }
    fn render_sidebar(
        &self,
        state: &Entity<SettingsState>,
        pages: &[ComponentChild<SettingPage>],
        cx: &App,
    ) -> impl IntoElement + use<> {
        let selected = state.read(cx).selected().cloned();
        Sidebar::new("settings-sidebar")
            .w(relative(1.))
            .border_0()
            .refine_style(&self.sidebar_style)
            .collapsible(false)
            .collapsed(false)
            .header(
                div()
                    .w_full()
                    .refine_style(&self.header_style)
                    .child(Input::new(&state.read(cx).search_input).prefix(IconName::Search)),
            )
            .child(SidebarMenu::new().children(pages.iter().map(|page| {
                let page_key = page.key.clone();
                let active = selected
                    .as_ref()
                    .is_some_and(|s| s.page == page.key && s.group.is_none());
                SidebarMenuItem::new(page.title.clone())
                    .id(page.key.clone())
                    .click_to_open(true)
                    .when_some(page.icon.clone(), |this, icon| this.icon(icon))
                    .default_open(page.default_open)
                    .active(active)
                    .on_click({
                        let state = state.clone();
                        let page_key = page_key.clone();
                        move |_, _, cx| {
                            state.update(cx, |s, cx| {
                                s.select(
                                    SettingSelection {
                                        page: page_key.clone(),
                                        group: None,
                                    },
                                    cx,
                                )
                            });
                        }
                    })
                    .when(page.groups.len() > 1, |this| {
                        this.children(page.groups.iter().filter(|g| g.title.is_some()).map(
                            |group| {
                                let selection = SettingSelection {
                                    page: page_key.clone(),
                                    group: Some(group.key.clone()),
                                };
                                SidebarMenuItem::new(group.title.clone().unwrap_or_default())
                                    .id(group.key.clone())
                                    .active(selected.as_ref() == Some(&selection))
                                    .on_click({
                                        let state = state.clone();
                                        move |_, _, cx| {
                                            state.update(cx, |s, cx| {
                                                s.select(selection.clone(), cx)
                                            });
                                        }
                                    })
                            },
                        ))
                    })
            })))
    }
}
impl Sizable for Settings {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}
impl RenderOnce for Settings {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self
            .state
            .clone()
            .unwrap_or_else(|| window.use_keyed_state(self.id.clone(), cx, SettingsState::new));
        let query = state.read(cx).query(cx);
        let filtered = self.filtered_pages(&query, cx);
        state.update(cx, |s, _| s.reconcile(&self.pages, &filtered));
        let options = RenderOptions::new()
            .with_size(self.size)
            .with_group_variant(self.group_variant);
        let sidebar = self.render_sidebar(&state, &filtered, cx);
        h_resizable(self.id.clone())
            .child(
                resizable_panel("settings-sidebar")
                    .size(self.sidebar_width)
                    .size_range(self.sidebar_size_range.clone())
                    .child(sidebar),
            )
            .child(resizable_panel("settings-content").child(container_query(
                move |size, _, cx| {
                    let options = options.with_layout(if size.width <= STACKED_LAYOUT_MAX_WIDTH {
                        Axis::Vertical
                    } else {
                        Axis::Horizontal
                    });
                    self.render_active_page(&state, &filtered, options, cx)
                },
            )))
    }
}
/// Options for rendering setting item.
///
/// The fields are private and reached through the methods below, so that a new
/// one can be added without breaking the item renderers. The setters take
/// `self` by value, so a nested renderer narrows a copy of its parent options:
///
/// ```ignore
/// item.render_item(&options.with_item_ix(item_ix), window, cx)
/// ```
#[derive(Clone, Copy)]
pub struct RenderOptions {
    page_ix: usize,
    group_ix: usize,
    item_ix: usize,
    size: Size,
    group_variant: GroupBoxVariant,
    layout: Axis,
    disabled: bool,
}

impl RenderOptions {
    pub fn new() -> Self {
        Self {
            page_ix: 0,
            group_ix: 0,
            item_ix: 0,
            size: Size::default(),
            group_variant: GroupBoxVariant::default(),
            layout: Axis::Horizontal,
            disabled: false,
        }
    }

    pub fn with_page_ix(mut self, page_ix: usize) -> Self {
        self.page_ix = page_ix;
        self
    }

    pub fn with_group_ix(mut self, group_ix: usize) -> Self {
        self.group_ix = group_ix;
        self
    }

    pub fn with_item_ix(mut self, item_ix: usize) -> Self {
        self.item_ix = item_ix;
        self
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn with_group_variant(mut self, group_variant: GroupBoxVariant) -> Self {
        self.group_variant = group_variant;
        self
    }

    pub fn with_layout(mut self, layout: Axis) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn page_ix(&self) -> usize {
        self.page_ix
    }

    pub fn group_ix(&self) -> usize {
        self.group_ix
    }

    pub fn item_ix(&self) -> usize {
        self.item_ix
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn group_variant(&self) -> GroupBoxVariant {
        self.group_variant
    }

    pub fn layout(&self) -> Axis {
        self.layout
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Root,
        setting::{SettingField, SettingItem},
    };
    use gpui::{Render, TestAppContext};
    use std::{cell::RefCell, collections::HashMap, rc::Rc};
    type Evidence = Rc<RefCell<HashMap<String, gpui::EntityId>>>;
    struct SettingsView {
        state: Entity<SettingsState>,
        reversed: bool,
        evidence: Evidence,
    }
    impl Render for SettingsView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let names = if self.reversed {
                ["beta", "alpha"]
            } else {
                ["alpha", "beta"]
            };
            let groups = names.map(|name| {
                let evidence = self.evidence.clone();
                SettingGroup::new(name).title(name).item(SettingItem::new(
                    "field",
                    name,
                    SettingField::render(move |_, window, cx| {
                        let state = window.use_keyed_state("field-state", cx, |_, _| ());
                        evidence.borrow_mut().insert(name.into(), state.entity_id());
                        div().h(px(30.)).child(name)
                    }),
                ))
            });
            Settings::new("settings")
                .with_state(&self.state)
                .page(SettingPage::new("general", "General").groups(groups))
                .page(SettingPage::new("advanced", "Advanced").group(
                    SettingGroup::new("other").item(SettingItem::new(
                        "other",
                        "other",
                        SettingField::render(|_, _, _| div().child("other")),
                    )),
                ))
        }
    }
    #[gpui::test]
    fn search_and_reorder_preserve_selection_and_field_identity(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let evidence = Evidence::default();
        let (root, cx) = cx.add_window_view(|window, cx| {
            let state = cx.new(|cx| SettingsState::new(window, cx));
            let view = cx.new(|_| SettingsView {
                state,
                reversed: false,
                evidence: evidence.clone(),
            });
            Root::new(view, window, cx)
        });
        let view = root.read_with(cx, |root, _| {
            root.view().clone().downcast::<SettingsView>().unwrap()
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let initial = evidence.borrow().clone();
        assert_eq!(initial.len(), 2);
        assert_ne!(initial["alpha"], initial["beta"]);
        view.update(cx, |view, cx| {
            view.reversed = true;
            cx.notify();
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(*evidence.borrow(), initial);
        let state = view.read_with(cx, |view, _| view.state.clone());
        cx.update(|window, cx| {
            state.update(cx, |state, cx| {
                state.select(
                    SettingSelection {
                        page: "general".into(),
                        group: Some("alpha".into()),
                    },
                    cx,
                );
                state.set_query("beta", window, cx);
            })
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(
            state.read_with(cx, |s, _| s.selected().cloned()),
            Some(SettingSelection {
                page: "general".into(),
                group: None
            })
        );
        assert_eq!(evidence.borrow()["beta"], initial["beta"]);
        cx.update(|window, cx| state.update(cx, |state, cx| state.set_query("other", window, cx)));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(
            state.read_with(cx, |s, _| s.selected().unwrap().page.clone()),
            "advanced"
        );
        cx.update(|window, cx| state.update(cx, |state, cx| state.set_query("", window, cx)));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(
            state.read_with(cx, |s, _| s.selected().cloned()),
            Some(SettingSelection {
                page: "general".into(),
                group: Some("alpha".into())
            })
        );
    }
}
