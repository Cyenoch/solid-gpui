//! Component-layer traits for the shared concrete child boundary.
use crate::{Sizable, Size, sidebar::SidebarItem};
use gpui::{App, ElementId, IntoElement, RenderOnce, Window};
pub use gpui_base::ComponentChild;
impl<T: Sizable> Sizable for ComponentChild<T> {
    fn with_size(self, size: impl Into<Size>) -> Self {
        self.map_native(|v| v.with_size(size))
    }
}
#[derive(IntoElement)]
struct DeferredSidebarItem<T: SidebarItem + 'static> {
    native: T,
    id: ElementId,
}
impl<T: SidebarItem + 'static> RenderOnce for DeferredSidebarItem<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.native.render(self.id, window, cx)
    }
}
impl<T: SidebarItem + 'static> SidebarItem for ComponentChild<T> {
    fn render(self, id: impl Into<ElementId>, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.map_native(|native| DeferredSidebarItem {
            native,
            id: id.into(),
        })
    }
}
