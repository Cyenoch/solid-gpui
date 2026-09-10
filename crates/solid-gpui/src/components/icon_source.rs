//! Application icons use Iconify identity or explicit SVG, never Kit's internal asset paths.
use crate::native::{Deserialize, Serialize, TS};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(untagged)]
pub enum ComponentIconSource {
    Name(String),
    Svg { svg: String },
}
#[derive(Clone, TS)]
#[ts(as = "ComponentIconSource")]
pub struct ComponentIcon {
    source: ComponentIconSource,
    #[ts(skip)]
    native: PreparedIcon,
}
#[derive(Clone)]
enum PreparedIcon {
    Path(gpui::SharedString),
    Svg(std::sync::Arc<[u8]>),
}
impl Serialize for ComponentIcon {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.source.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for ComponentIcon {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let source = ComponentIconSource::deserialize(deserializer)?;
        let native = match &source {
            ComponentIconSource::Name(name) => {
                let path =
                    crate::icons::monochrome_asset_path(name).map_err(serde::de::Error::custom)?;
                PreparedIcon::Path(path)
            }
            ComponentIconSource::Svg { svg } => {
                crate::icons::validate_svg(svg.as_bytes()).map_err(serde::de::Error::custom)?;
                PreparedIcon::Svg(std::sync::Arc::from(svg.as_bytes()))
            }
        };
        Ok(Self { source, native })
    }
}
impl ComponentIcon {
    pub(super) fn native(&self) -> gpui_component::Icon {
        match &self.native {
            PreparedIcon::Path(path) => gpui_component::Icon::default().path(path.clone()),
            PreparedIcon::Svg(bytes) => gpui_component::Icon::default().shared_data(bytes.clone()),
        }
    }
}
impl PartialEq for ComponentIcon {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
    }
}
impl std::fmt::Debug for ComponentIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn application_icon_sources_cannot_reach_kit_or_unregistered_assets() {
        for name in [
            "icons/check.svg",
            "iconify/lucide/check.svg",
            "lucide:not-installed",
        ] {
            assert!(
                crate::native::decode_json::<ComponentIcon>(&serde_json::to_vec(name).unwrap())
                    .is_err()
            );
        }
        let icon = crate::native::decode_json::<ComponentIcon>(br#""lucide:check""#).unwrap();
        assert_eq!(serde_json::to_value(icon).unwrap(), "lucide:check");
        assert!(
            crate::native::decode_json::<ComponentIcon>(
                br#"{"svg":"<svg viewBox=\"0 0 24 24\"><script/></svg>"}"#
            )
            .is_err()
        );
        assert!(
            crate::native::decode_json::<ComponentIcon>(
                br#"{"svg":"<svg viewBox=\"0 0 24 24\"><path d=\"M0 0L24 24\"/></svg>"}"#
            )
            .is_ok()
        );
    }
}
