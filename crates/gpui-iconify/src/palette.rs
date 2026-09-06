use crate::IconId;
use gpui::{
    App, Asset, ImageCacheError, IntoElement, ParentElement, RenderImage, SMOOTH_SVG_SCALE_FACTOR,
    Styled, Window, div, img, px,
};
use std::sync::Arc;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct PaletteKey {
    id: IconId,
    device_px: u32,
}

enum PaletteAsset<T, E> {
    Ready(T),
    Loading,
    Failed(E),
}

fn classify_asset<T, E>(asset: Option<Result<T, E>>) -> PaletteAsset<T, E> {
    match asset {
        Some(Ok(image)) => PaletteAsset::Ready(image),
        Some(Err(error)) => PaletteAsset::Failed(error),
        None => PaletteAsset::Loading,
    }
}

fn raster_fallback(logical_px: f32) -> impl IntoElement {
    div()
        .flex_none()
        .size(px(logical_px))
        .flex()
        .items_center()
        .justify_center()
        .child("!")
}

enum PaletteRaster {}

impl Asset for PaletteRaster {
    type Source = PaletteKey;
    type Output = Result<Arc<RenderImage>, ImageCacheError>;

    fn load(
        source: Self::Source,
        cx: &mut App,
    ) -> impl Future<Output = Self::Output> + Send + 'static {
        let renderer = cx.svg_renderer();
        let bytes = source.id.svg();
        let intrinsic = source.id.intrinsic_px().max(1.0);
        let device = source.device_px.max(1) as f32;
        let scale = device / (SMOOTH_SVG_SCALE_FACTOR * intrinsic);
        async move { Ok(renderer.render_single_frame(bytes, scale)?) }
    }
}

pub(crate) fn paint(
    id: IconId,
    logical_px: f32,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let device = (logical_px * window.scale_factor())
        .round()
        .clamp(1.0, 4096.0) as u32;
    let sized = div().flex_none().size(px(logical_px));
    match classify_asset(window.use_asset::<PaletteRaster>(
        &PaletteKey {
            id,
            device_px: device,
        },
        cx,
    )) {
        PaletteAsset::Ready(image) => sized
            .child(img(image).size(px(logical_px)))
            .into_any_element(),
        PaletteAsset::Failed(error) => {
            tracing::warn!(icon = %id, ?error, "failed to rasterize palette icon");
            raster_fallback(logical_px).into_any_element()
        }
        PaletteAsset::Loading => sized.into_any_element(),
    }
}

#[cfg(test)]
mod tests {
    use super::{PaletteAsset, classify_asset};

    #[test]
    fn raster_error_is_distinct_from_cache_miss() {
        let loading: PaletteAsset<(), ()> = classify_asset(None);
        let failed: PaletteAsset<(), ()> = classify_asset(Some(Err(())));

        assert!(matches!(loading, PaletteAsset::Loading));
        assert!(matches!(failed, PaletteAsset::Failed(())));
    }
}
