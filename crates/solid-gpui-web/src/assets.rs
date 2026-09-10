use gpui::{AssetSource, Result, SharedString};
use std::borrow::Cow;
include!(concat!(env!("OUT_DIR"), "/icons.rs"));

pub struct WebAssets;
impl AssetSource for WebAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some((_, bytes)) = COMPONENT_ICONS.iter().find(|(name, _)| *name == path) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        solid_gpui::icons::ApplicationIconAssets.load(path)
    }
    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut entries = solid_gpui::icons::ApplicationIconAssets.list(path)?;
        entries.extend(
            COMPONENT_ICONS
                .iter()
                .filter(|(name, _)| name.starts_with(path))
                .map(|(name, _)| SharedString::from(*name)),
        );
        Ok(entries)
    }
}
