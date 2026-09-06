use super::overlays::Side;
use crate::native::NativeSlot;
use gpui::{
    App, InteractiveElement, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement,
    Window, div,
};
use gpui_component::Selectable;

#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PopupAnchor {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    LeftCenter,
    RightCenter,
}
impl From<PopupAnchor> for gpui::Anchor {
    fn from(v: PopupAnchor) -> Self {
        match v {
            PopupAnchor::TopLeft => Self::TopLeft,
            PopupAnchor::TopCenter => Self::TopCenter,
            PopupAnchor::TopRight => Self::TopRight,
            PopupAnchor::BottomLeft => Self::BottomLeft,
            PopupAnchor::BottomCenter => Self::BottomCenter,
            PopupAnchor::BottomRight => Self::BottomRight,
            PopupAnchor::LeftCenter => Self::LeftCenter,
            PopupAnchor::RightCenter => Self::RightCenter,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum PopupMouseButton {
    #[default]
    Left,
    Right,
    Middle,
}
impl From<PopupMouseButton> for gpui::MouseButton {
    fn from(v: PopupMouseButton) -> Self {
        match v {
            PopupMouseButton::Left => Self::Left,
            PopupMouseButton::Right => Self::Right,
            PopupMouseButton::Middle => Self::Middle,
        }
    }
}

#[derive(IntoElement)]
struct PopupTrigger {
    content: NativeSlot,
    selected: bool,
}
impl Selectable for PopupTrigger {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    fn is_selected(&self) -> bool {
        self.selected
    }
}
impl RenderOnce for PopupTrigger {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .id("trigger")
            .aria_expanded(self.selected)
            .child(self.content)
    }
}
#[crate::native_module(name = "gpui-component")]
mod exports {
    use super::*;
    use crate::native::{ElementContext, Event};
    use gpui::{Styled, prelude::FluentBuilder};
    use gpui_component::tooltip::ManagedTooltipExt;
    use std::time::Duration;
    #[component]
    fn popover(
        trigger: NativeSlot,
        #[prop(default)] open: Option<bool>,
        #[prop(default)] default_open: bool,
        #[prop(default)] anchor: PopupAnchor,
        #[prop(default)] mouse_button: PopupMouseButton,
        #[prop(default = true)] appearance: bool,
        #[prop(default = true)] overlay_closable: bool,
        on_open_change: Event<bool>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        let content = cx.content();
        gpui_component::popover::Popover::new(cx.id())
            .anchor(gpui::Anchor::from(anchor))
            .mouse_button(mouse_button.into())
            .default_open(default_open)
            .appearance(appearance)
            .overlay_closable(overlay_closable)
            .trigger(PopupTrigger {
                content: trigger,
                selected: false,
            })
            .content(move |_, _, _| content.clone())
            .when_some(open, |p, open| p.open(open))
            .on_open_change(move |v, _, _| on_open_change.emit(*v))
    }
    #[component]
    fn hover_card(
        trigger: NativeSlot,
        #[prop(default = PopupAnchor::TopCenter)] anchor: PopupAnchor,
        #[prop(default = 600)] open_delay_ms: u32,
        #[prop(default = 300)] close_delay_ms: u32,
        #[prop(default = true)] appearance: bool,
        on_open_change: Event<bool>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        let content = cx.content();
        gpui_component::hover_card::HoverCard::new(cx.id())
            .anchor(gpui::Anchor::from(anchor))
            .open_delay(Duration::from_millis(open_delay_ms as u64))
            .close_delay(Duration::from_millis(close_delay_ms as u64))
            .appearance(appearance)
            .trigger(trigger)
            .content(move |_, _, _| content.clone())
            .on_open_change(move |v, _, _| on_open_change.emit(*v))
    }
    #[component]
    fn tooltip(
        trigger: NativeSlot,
        #[prop(default)] text: Option<String>,
        #[prop(default)] key_binding: Option<super::super::extra_elements::KeyStroke>,
        #[prop(default = Side::Top)] placement: Side,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        let content = cx.content();
        div()
            .id(cx.id())
            .child(trigger)
            .managed_tooltip_at(placement.into(), move |window, cx| {
                let tip = if let Some(text) = &text {
                    gpui_component::tooltip::Tooltip::new(text.clone())
                } else {
                    let content = content.clone();
                    gpui_component::tooltip::Tooltip::element(move |_, _| content.clone())
                };
                tip.key_binding(
                    key_binding
                        .as_ref()
                        .map(|v| gpui_component::kbd::Kbd::new(v.native())),
                )
                .build(window, cx)
            })
    }
}
pub(super) use exports::native_module;
