//! Composable empty states. The application decides when to show one and owns its actions.

/// Visual treatment for the media slot of an `Empty`.
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EmptyMediaVariant {
    #[default]
    Default,
    Icon,
}
impl From<EmptyMediaVariant> for gpui_component::empty::EmptyMediaVariant {
    fn from(v: EmptyMediaVariant) -> Self {
        match v {
            EmptyMediaVariant::Default => Self::Default,
            EmptyMediaVariant::Icon => Self::Icon,
        }
    }
}

#[crate::native_module(name = "gpui-component")]
mod exports {
    use super::*;
    use crate::native::ElementContext;
    use gpui::{IntoElement, ParentElement, Styled};
    use gpui_component::empty;

    #[component]
    pub fn empty(cx: &mut ElementContext) -> impl IntoElement + Styled {
        empty::Empty::new().children(cx.children())
    }
    #[component]
    pub fn empty_header(cx: &mut ElementContext) -> impl IntoElement + Styled {
        empty::EmptyHeader::new().children(cx.children())
    }
    #[component]
    pub fn empty_media(
        #[prop(default)] variant: EmptyMediaVariant,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        empty::EmptyMedia::new()
            .with_variant(variant.into())
            .children(cx.children())
    }
    #[component]
    pub fn empty_title(cx: &mut ElementContext) -> impl IntoElement + Styled {
        empty::EmptyTitle::new().children(cx.children())
    }
    #[component]
    pub fn empty_description(cx: &mut ElementContext) -> impl IntoElement + Styled {
        empty::EmptyDescription::new().children(cx.children())
    }
    #[component]
    pub fn empty_content(cx: &mut ElementContext) -> impl IntoElement + Styled {
        empty::EmptyContent::new().children(cx.children())
    }
}
pub(super) use exports::native_module;
