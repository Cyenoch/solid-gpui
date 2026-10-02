//! Native toolbar navigation around independently owned Solid controls.
use super::ControlSize;

#[crate::native_module(name = "gpui-component", version = "1.0.0")]
mod exports {
    use super::*;
    use crate::native::ElementContext;
    use gpui::{IntoElement, ParentElement, Styled, prelude::FluentBuilder};
    use gpui_component::{Sizable, toolbar};

    #[component]
    fn toolbar(
        #[prop(default = ControlSize::Small)] size: ControlSize,
        #[prop(default)] disabled: bool,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        toolbar::Toolbar::new(cx.id())
            .with_size(size)
            .disabled(disabled)
            .contents(cx.children())
    }

    #[component]
    fn toolbar_group(
        #[prop(default)] label: Option<String>,
        #[prop(default = ControlSize::Small)] size: ControlSize,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        let mut group = toolbar::ToolbarGroup::new(cx.id())
            .with_size(size)
            .when_some(label, |group, label| group.label(label));
        group.extend(cx.children());
        group
    }
}
pub(super) use exports::native_module;
