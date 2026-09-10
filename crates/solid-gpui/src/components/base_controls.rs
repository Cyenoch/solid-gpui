//! Unstyled controls preserve Base interaction and accessibility with application-owned content.
#[crate::native_type]
#[derive(Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum BaseCheckState {
    #[default]
    Unchecked,
    Checked,
    Indeterminate,
}
impl From<BaseCheckState> for gpui_base::CheckboxState {
    fn from(value: BaseCheckState) -> Self {
        match value {
            BaseCheckState::Unchecked => Self::Unchecked,
            BaseCheckState::Checked => Self::Checked,
            BaseCheckState::Indeterminate => Self::Indeterminate,
        }
    }
}
impl From<gpui_base::CheckboxState> for BaseCheckState {
    fn from(value: gpui_base::CheckboxState) -> Self {
        match value {
            gpui_base::CheckboxState::Unchecked => Self::Unchecked,
            gpui_base::CheckboxState::Checked => Self::Checked,
            gpui_base::CheckboxState::Indeterminate => Self::Indeterminate,
        }
    }
}
#[crate::native_module(name = "gpui-base")]
mod controls {
    use super::*;
    use crate::native::{ElementContext, Event};
    use gpui::{IntoElement, ParentElement, Styled, prelude::FluentBuilder};

    #[component]
    fn base_button(
        accessibility_label: String,
        #[prop(default)] disabled: bool,
        #[prop(default)] selected: bool,
        #[prop(default)] tab_index: i32,
        #[prop(default = true)] tab_stop: bool,
        on_press: Event<()>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        gpui_base::Button::new(cx.id())
            .accessibility_label(accessibility_label)
            .disabled(disabled)
            .selected(selected)
            .tab_index(tab_index as isize)
            .tab_stop(tab_stop)
            .when(on_press.is_subscribed(), |button| {
                button.on_click(move |_, _, _| on_press.emit(()))
            })
            .children(cx.children())
    }
    #[component]
    fn base_checkbox(
        accessibility_label: String,
        #[prop(default)] state: BaseCheckState,
        #[prop(default)] disabled: bool,
        #[prop(default)] tab_index: i32,
        #[prop(default = true)] tab_stop: bool,
        on_change: Event<BaseCheckState>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        gpui_base::Checkbox::new(cx.id())
            .accessibility_label(accessibility_label)
            .state(state.into())
            .disabled(disabled)
            .tab_index(tab_index as isize)
            .tab_stop(tab_stop)
            .when(on_change.is_subscribed(), |checkbox| {
                checkbox.on_change(move |state, _, _, _| on_change.emit(state.into()))
            })
            .children(cx.children())
    }
    #[component]
    fn base_switch(
        accessibility_label: String,
        #[prop(default)] checked: bool,
        #[prop(default)] disabled: bool,
        #[prop(default)] tab_index: i32,
        #[prop(default = true)] tab_stop: bool,
        on_change: Event<bool>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        gpui_base::Switch::new(cx.id())
            .accessibility_label(accessibility_label)
            .checked(checked)
            .disabled(disabled)
            .tab_index(tab_index as isize)
            .tab_stop(tab_stop)
            .when(on_change.is_subscribed(), |switch| {
                switch.on_change(move |checked, _, _, _| on_change.emit(checked))
            })
            .children(cx.children())
    }
    #[component]
    fn base_toggle(
        accessibility_label: String,
        #[prop(default)] pressed: bool,
        #[prop(default)] disabled: bool,
        #[prop(default)] tab_index: i32,
        #[prop(default = true)] tab_stop: bool,
        on_change: Event<bool>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        gpui_base::Toggle::new(cx.id())
            .accessibility_label(accessibility_label)
            .pressed(pressed)
            .disabled(disabled)
            .tab_index(tab_index as isize)
            .tab_stop(tab_stop)
            .when(on_change.is_subscribed(), |toggle| {
                toggle.on_change(move |pressed, _, _, _| on_change.emit(pressed))
            })
            .children(cx.children())
    }
}
pub(super) use controls::native_module;
