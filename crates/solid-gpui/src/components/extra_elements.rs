use super::{ControlSize, primitives::Color};
use crate::native::{Deserialize, Serialize, TS};
#[derive(Clone, Debug, Serialize, TS)]
pub struct KeyStroke(String);
impl KeyStroke {
    pub(super) fn native(&self) -> gpui::Keystroke {
        gpui::Keystroke::parse(&self.0).expect("validated keystroke")
    }
}
impl<'de> Deserialize<'de> for KeyStroke {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        gpui::Keystroke::parse(&s).map_err(serde::de::Error::custom)?;
        Ok(Self(s))
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum MarkerVariant {
    #[default]
    Plain,
    Separator,
    Border,
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum MarkerLoadingStyle {
    #[default]
    Spinner,
    Shimmer,
}
#[crate::native_module(name = "gpui-component")]
mod exports {
    use super::*;
    use crate::native::{ElementContext, NativeSlot};
    use gpui::{IntoElement, ParentElement, Styled, prelude::FluentBuilder};
    use gpui_component::{Disableable, Selectable, Sizable};
    #[component(children = false)]
    pub fn kbd(
        stroke: KeyStroke,
        #[prop(default = true)] appearance: bool,
        #[prop(default)] outline: bool,
    ) -> impl IntoElement + Styled {
        gpui_component::kbd::Kbd::new(
            gpui::Keystroke::parse(&stroke.0).expect("validated keystroke"),
        )
        .appearance(appearance)
        .when(outline, |v| v.outline())
    }
    #[component(children = false)]
    pub fn caret(
        #[prop(default)] size: ControlSize,
        #[prop(default)] color: Option<Color>,
    ) -> impl IntoElement {
        gpui_component::select::Caret::new(size.into())
            .when_some(color, |v, c| v.text_color(c.native()))
    }
    #[component]
    pub fn searchable_list_item_element(
        #[prop(default)] checked: bool,
        #[prop(default)] selected: bool,
        #[prop(default)] disabled: bool,
        #[prop(default)] size: ControlSize,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        let gpui::ElementId::Integer(id) = cx.id() else {
            unreachable!("native node identity")
        };
        gpui_component::searchable_list::SearchableListItemElement::new(id as usize)
            .checked(checked)
            .selected(selected)
            .disabled(disabled)
            .with_size(size)
            .children(cx.children())
    }
    #[component]
    pub fn marker(
        #[prop(default)] variant: MarkerVariant,
        #[prop(default)] loading: bool,
        #[prop(default)] loading_style: MarkerLoadingStyle,
        #[prop(default)] status: bool,
        icon: NativeSlot,
        content: NativeSlot,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        use gpui_component::marker as m;
        m::Marker::new()
            .id(cx.id())
            .with_variant(match variant {
                MarkerVariant::Plain => m::MarkerVariant::Plain,
                MarkerVariant::Separator => m::MarkerVariant::Separator,
                MarkerVariant::Border => m::MarkerVariant::Border,
            })
            .loading(loading)
            .with_loading_style(match loading_style {
                MarkerLoadingStyle::Spinner => m::MarkerLoadingStyle::Spinner,
                MarkerLoadingStyle::Shimmer => m::MarkerLoadingStyle::Shimmer,
            })
            .when(status, |v| v.role(gpui::Role::Status))
            .when(!icon.is_empty(), |v| {
                v.icon(m::MarkerIcon::new().child(icon))
            })
            .when(!content.is_empty(), |v| {
                v.content(m::MarkerContent::new().child(content))
            })
            .children(cx.children())
    }
    #[component]
    pub fn marker_icon(cx: &mut ElementContext) -> impl IntoElement + Styled {
        gpui_component::marker::MarkerIcon::new().children(cx.children())
    }
    #[component]
    pub fn marker_content(
        #[prop(default)] text: Option<String>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        gpui_component::marker::MarkerContent::new()
            .when_some(text, |v, s| v.text(s))
            .children(cx.children())
    }
    #[component(children = false)]
    pub fn text(value: String) -> impl IntoElement {
        gpui_component::text::Text::from(value)
    }
}
pub use exports::native_module;
