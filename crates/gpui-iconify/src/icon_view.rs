use crate::{IconId, IconKind, IconSize, palette};
use gpui::{
    App, Hsla, IntoElement, Radians, RenderOnce, Styled, Transformation, Window,
    prelude::FluentBuilder as _, px, svg,
};

/// `RenderOnce` view over an [`IconId`]. Size is exact; color is a GPU tint for mono icons.
#[derive(Clone, Copy, IntoElement)]
#[must_use]
pub struct IconView {
    id: IconId,
    size: IconSize,
    color: Option<Hsla>,
    rotation: Option<Radians>,
}

impl IconView {
    /// Wraps an already-embedded icon identity.
    pub fn new(id: IconId) -> Self {
        Self {
            id,
            size: IconSize::MEDIUM,
            color: None,
            rotation: None,
        }
    }

    /// Sets the logical size. Any positive finite pixel value is used as-is.
    pub fn with_size(mut self, size: impl Into<IconSize>) -> Self {
        self.size = size.into();
        self
    }

    /// Tint for [`IconKind::Mono`]. Ignored for palette and mixed icons.
    pub fn text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// GPU rotation for [`IconKind::Mono`]. Ignored for palette and mixed icons.
    pub fn rotate(mut self, radians: impl Into<Radians>) -> Self {
        self.rotation = Some(radians.into());
        self
    }

    /// Embedded identity.
    pub fn id(self) -> IconId {
        self.id
    }

    /// Logical size used for layout and the atlas / raster key.
    pub fn size(self) -> IconSize {
        self.size
    }
}

impl RenderOnce for IconView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Some(logical) = self.size.paintable_px() else {
            return gpui::div().into_any_element();
        };
        match self.id.kind() {
            IconKind::Mono => self.render_mono(window, logical).into_any_element(),
            IconKind::Palette | IconKind::Mixed => {
                palette::paint(self.id, logical, window, cx).into_any_element()
            }
        }
    }
}

impl IconView {
    fn render_mono(self, window: &mut Window, logical: f32) -> impl IntoElement {
        let color = self.color.unwrap_or_else(|| window.text_style().color);
        svg()
            .flex_none()
            .size(px(logical))
            .path(self.id.cache_key())
            .text_color(color)
            .when_some(self.rotation, |this, radians| {
                this.with_transformation(Transformation::rotate(radians))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::IconView;
    use crate::IconId;
    use gpui::px;

    #[test]
    fn builder_keeps_identity_and_exact_pixels() {
        let view = IconView::new(IconId::LucidePlay).with_size(px(23.));
        assert_eq!(view.id(), IconId::LucidePlay);
        assert_eq!(view.size().px(), 23.0);
    }
}
