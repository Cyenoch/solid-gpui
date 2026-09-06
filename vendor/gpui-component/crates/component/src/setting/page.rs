use std::rc::Rc;

use gpui::{
    AnyElement, App, Entity, InteractiveElement as _, IntoElement, ListAlignment, ListState,
    ParentElement as _, SharedString, StyleRefinement, Styled, Window, div, list,
    prelude::FluentBuilder as _, px,
};
use rust_i18n::t;

use crate::{
    ActiveTheme, ComponentChild, Icon, IconName, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    label::Label,
    scroll::ScrollableElement,
    setting::{RenderOptions, SettingGroup, settings::SettingsState},
    v_flex,
};

/// A setting page that can contain multiple setting groups.
#[derive(Clone)]
pub struct SettingPage {
    pub(super) key: SharedString,
    pub(super) icon: Option<Icon>,
    resettable: bool,
    pub(super) default_open: bool,
    pub(super) title: SharedString,
    pub(super) title_suffix: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>>,
    pub(super) description: Option<SharedString>,
    pub(super) groups: Vec<ComponentChild<SettingGroup>>,
    pub(super) header_style: StyleRefinement,
}

impl SettingPage {
    pub fn new(key: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            icon: None,
            resettable: true,
            default_open: false,
            title: title.into(),
            title_suffix: None,
            description: None,
            groups: Vec::new(),
            header_style: StyleRefinement::default(),
        }
    }

    pub fn key(&self) -> &SharedString {
        &self.key
    }
    pub fn has_group(&self, key: &str) -> bool {
        self.groups.iter().any(|g| g.key.as_ref() == key)
    }

    /// Set the title of the setting page.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    /// Set a custom element to render after the title in the page header.
    ///
    /// For example, an info icon button that opens the help documentation.
    pub fn title_suffix<F, E>(mut self, suffix: F) -> Self
    where
        E: IntoElement,
        F: Fn(&mut Window, &mut App) -> E + 'static,
    {
        self.title_suffix = Some(Rc::new(move |window, cx| {
            suffix(window, cx).into_any_element()
        }));
        self
    }

    /// Set the icon of the setting page.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the description of the setting page, default is None.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the default open state of the setting page, default is false.
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    /// Set whether the setting page is resettable, default is true.
    ///
    /// If true and the items in this page has changed, the reset button will appear.
    pub fn resettable(mut self, resettable: bool) -> Self {
        self.resettable = resettable;
        self
    }

    /// Add a setting group to the page.
    pub fn group(mut self, group: impl Into<ComponentChild<SettingGroup>>) -> Self {
        self.groups.push(group.into());
        self
    }

    /// Add multiple setting groups to the page.
    pub fn groups<I, G>(mut self, groups: I) -> Self
    where
        I: IntoIterator<Item = G>,
        G: Into<ComponentChild<SettingGroup>>,
    {
        self.groups.extend(groups.into_iter().map(Into::into));
        self
    }

    /// Set the style refinement for the header of the setting page.
    pub fn header_style(mut self, style: &StyleRefinement) -> Self {
        self.header_style = style.clone();
        self
    }

    fn is_resettable(&self, cx: &App) -> bool {
        self.resettable && self.groups.iter().any(|group| group.is_resettable(cx))
    }

    fn reset_all(&self, window: &mut Window, cx: &mut App) {
        for group in &self.groups {
            group.reset(window, cx);
        }
    }

    pub(super) fn render(
        &self,
        ix: usize,
        state: &Entity<SettingsState>,
        options: &RenderOptions,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement + use<> {
        let search_input = state.read(cx).search_input.clone();
        let query = search_input.read(cx).value();
        let groups = self
            .groups
            .iter()
            .filter(|group| group.is_match(&query, cx))
            .cloned()
            .collect::<Vec<_>>();
        let groups_count = groups.len();

        let keys = groups.iter().map(|g| g.key.clone()).collect::<Vec<_>>();
        let list_state = window
            .use_keyed_state(
                SharedString::from(format!("list-state:{}", self.key)),
                cx,
                |_, _| GroupListState {
                    list: ListState::new(groups_count, ListAlignment::Top, px(100.)),
                    keys: keys.clone(),
                },
            )
            .update(cx, |state, _| {
                if state.keys != keys {
                    let mut anchor = state.list.logical_scroll_top();
                    let old_key = state.keys.get(anchor.item_ix);
                    let next_ix = old_key.and_then(|key| keys.iter().position(|k| k == key));
                    state.list.reset(groups_count);
                    if let Some(ix) = next_ix {
                        anchor.item_ix = ix;
                        state.list.scroll_to(anchor);
                    }
                    state.keys = keys;
                }
                state.list.clone()
            });

        let deferred_scroll_group_ix = state
            .read(cx)
            .deferred_scroll_group
            .as_ref()
            .and_then(|key| groups.iter().position(|g| &g.key == key));
        if let Some(ix) = deferred_scroll_group_ix {
            state.update(cx, |state, _| {
                state.deferred_scroll_group = None;
            });
            list_state.scroll_to_reveal_item(ix);
        }

        v_flex()
            .id(self.key.clone())
            .size_full()
            .child(
                v_flex()
                    .p_4()
                    .gap_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .refine_style(&self.header_style)
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(self.title.clone())
                                    .when_some(self.title_suffix.clone(), |this, suffix| {
                                        this.child(suffix(window, cx))
                                    }),
                            )
                            .when(self.is_resettable(cx), |this| {
                                this.child(
                                    Button::new("reset")
                                        .icon(IconName::Undo2)
                                        .ghost()
                                        .small()
                                        .tooltip(t!("Settings.Reset All"))
                                        .on_click({
                                            let page = self.clone();
                                            move |_, window, cx| {
                                                page.reset_all(window, cx);
                                            }
                                        }),
                                )
                            }),
                    )
                    .when_some(self.description.clone(), |this, description| {
                        this.child(
                            Label::new(description)
                                .text_sm()
                                .text_color(cx.theme().muted_foreground),
                        )
                    }),
            )
            .child(
                div()
                    .px_4()
                    .relative()
                    .flex_1()
                    .w_full()
                    .child(
                        list(list_state.clone(), {
                            let query = query.clone();
                            let options = *options;
                            move |group_ix, _, _| {
                                let group = groups[group_ix].clone();
                                let query = query.clone();
                                group.py_4().render_deferred(move |group, window, cx| {
                                    group.render(
                                        &query,
                                        &options.with_page_ix(ix).with_group_ix(group_ix),
                                        window,
                                        cx,
                                    )
                                })
                            }
                        })
                        .size_full(),
                    )
                    .vertical_scrollbar(&list_state),
            )
    }
}

struct GroupListState {
    list: ListState,
    keys: Vec<SharedString>,
}
