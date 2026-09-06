use crate::IconId;
use gpui::{AssetSource, Result, SharedString};
use std::borrow::Cow;

/// Serves embedded SVG bytes from [`IconId::cache_key`] paths.
///
/// ```ignore
/// gpui_platform::application().with_assets(gpui_iconify::IconAssets)
/// ```
pub struct IconAssets;

impl AssetSource for IconAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(IconId::from_asset_path(path).map(|id| Cow::Borrowed(id.svg())))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(IconId::ALL
            .iter()
            .map(|id| SharedString::from(id.cache_key()))
            .filter(|key| path.is_empty() || key.as_ref().starts_with(path))
            .collect())
    }
}
