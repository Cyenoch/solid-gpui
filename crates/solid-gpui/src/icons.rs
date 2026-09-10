//! Bounded, application-owned SVG resources installed before starting a host.
use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};

/// Monochrome icons inherit text color; original SVGs preserve brand colors.
pub enum IconColorMode {
    Monochrome,
    Original,
}
pub struct IconResource {
    pub name: &'static str,
    pub svg: &'static [u8],
    pub color_mode: IconColorMode,
}
struct InstalledIcon {
    svg: &'static [u8],
    image: Option<Arc<gpui::Image>>,
}
static APPLICATION_ICONS: OnceLock<BTreeMap<String, InstalledIcon>> = OnceLock::new();

/// Install an immutable offline icon catalog before starting the runtime.
/// SVGs must be embedded with `include_bytes!`; names use `prefix:name` syntax.
/// Registration validates the complete catalog atomically and may happen only once.
pub fn register_icons(icons: &[IconResource]) -> Result<(), String> {
    if icons.is_empty() || icons.len() > 256 {
        return Err("application icon catalog must contain 1..=256 icons".into());
    }
    let mut catalog = BTreeMap::new();
    for icon in icons {
        let name = icon.name;
        let bytes = icon.svg;
        if !valid_name(name) || gpui_iconify::IconId::from_name(name).is_some() {
            return Err(format!("invalid or reserved application icon name: {name}"));
        }
        validate_svg(bytes).map_err(|error| format!("icon {name}: {error}"))?;
        let image = match icon.color_mode {
            IconColorMode::Monochrome => None,
            IconColorMode::Original => Some(Arc::new(gpui::Image::from_bytes(
                gpui::ImageFormat::Svg,
                bytes.to_vec(),
            ))),
        };
        if catalog
            .insert(name.to_owned(), InstalledIcon { svg: bytes, image })
            .is_some()
        {
            return Err(format!("duplicate application icon: {name}"));
        }
    }
    APPLICATION_ICONS
        .set(catalog)
        .map_err(|_| "application icons already registered".into())
}

fn valid_name(name: &str) -> bool {
    name.len() <= 128
        && name.split(':').count() == 2
        && name.split(':').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}

/// Validate the deliberately small SVG subset accepted for distributed icons.
pub fn validate_svg(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > 65_536 {
        return Err("SVG must contain 1..=65536 bytes".into());
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "SVG must be UTF-8")?;
    let document =
        roxmltree::Document::parse(text).map_err(|error| format!("invalid SVG XML: {error}"))?;
    let root = document.root_element();
    if root.tag_name().name() != "svg" || root.attribute("viewBox").is_none() {
        return Err("SVG requires an svg root and viewBox".into());
    }
    let view_box = root
        .attribute("viewBox")
        .unwrap()
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .map(str::parse::<f32>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "invalid SVG viewBox")?;
    if view_box.len() != 4
        || view_box.iter().any(|v| !v.is_finite())
        || view_box[2] <= 0.
        || view_box[3] <= 0.
        || view_box[2] > 4096.
        || view_box[3] > 4096.
    {
        return Err("SVG viewBox dimensions must be in (0, 4096]".into());
    }
    for key in ["width", "height"] {
        if let Some(value) = root.attribute(key) {
            let length = value
                .strip_suffix("px")
                .unwrap_or(value)
                .parse::<f32>()
                .map_err(|_| format!("invalid SVG {key}"))?;
            if !length.is_finite() || length <= 0. || length > 4096. {
                return Err(format!("SVG {key} must be in (0, 4096]"));
            }
        }
    }
    let mut count = 0;
    for node in document.descendants().filter(|node| node.is_element()) {
        count += 1;
        if count > 1024 {
            return Err("SVG exceeds 1024 elements".into());
        }
        if ![
            "svg",
            "g",
            "path",
            "circle",
            "ellipse",
            "rect",
            "line",
            "polyline",
            "polygon",
            "title",
            "desc",
            "defs",
            "linearGradient",
            "radialGradient",
            "stop",
            "clipPath",
        ]
        .contains(&node.tag_name().name())
        {
            return Err(format!(
                "unsupported SVG element: {}",
                node.tag_name().name()
            ));
        }
        for attribute in node.attributes() {
            if ![
                "viewBox",
                "width",
                "height",
                "x",
                "y",
                "x1",
                "y1",
                "x2",
                "y2",
                "cx",
                "cy",
                "r",
                "rx",
                "ry",
                "d",
                "points",
                "fill",
                "stroke",
                "stroke-width",
                "stroke-linecap",
                "stroke-linejoin",
                "stroke-miterlimit",
                "stroke-dasharray",
                "stroke-dashoffset",
                "fill-rule",
                "clip-rule",
                "opacity",
                "fill-opacity",
                "stroke-opacity",
                "transform",
                "id",
                "offset",
                "stop-color",
                "stop-opacity",
                "gradientUnits",
                "gradientTransform",
                "clip-path",
            ]
            .contains(&attribute.name())
            {
                return Err(format!("unsupported SVG attribute: {}", attribute.name()));
            }
            let value = attribute.value();
            if value.contains("url(")
                && !(value.starts_with("url(#")
                    && value.ends_with(')')
                    && !value.contains(['\"', '\'', ' ', '\n']))
            {
                return Err("SVG external resources are forbidden".into());
            }
        }
    }
    Ok(())
}

pub(crate) fn is_registered(name: &str) -> bool {
    gpui_iconify::IconId::from_name(name).is_some()
        || APPLICATION_ICONS
            .get()
            .is_some_and(|icons| icons.contains_key(name))
}
/// The application icon namespace. Kit assets are intentionally outside this source.
pub struct ApplicationIconAssets;
impl gpui::AssetSource for ApplicationIconAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        if path.starts_with("iconify/") {
            return gpui_iconify::IconAssets.load(path);
        }
        Ok(path
            .strip_prefix("application-icons/")
            .and_then(|name| APPLICATION_ICONS.get()?.get(name))
            .map(|icon| std::borrow::Cow::Borrowed(icon.svg)))
    }
    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        let mut entries = gpui_iconify::IconAssets.list(path)?;
        if let Some(icons) = APPLICATION_ICONS.get() {
            entries.extend(
                icons
                    .keys()
                    .map(|name| format!("application-icons/{name}"))
                    .filter(|key| key.starts_with(path))
                    .map(Into::into),
            );
        }
        Ok(entries)
    }
}

#[cfg(feature = "component-runtime")]
pub(crate) fn monochrome_asset_path(name: &str) -> Result<gpui::SharedString, String> {
    if let Some(icon) = gpui_iconify::IconId::from_name(name) {
        if icon.kind() != gpui_iconify::IconKind::Mono {
            return Err(
                "component icon slots require monochrome icons; use core Icon for original colors"
                    .into(),
            );
        }
        return Ok(icon.cache_key().into());
    }
    if let Some(icon) = APPLICATION_ICONS.get().and_then(|icons| icons.get(name)) {
        if icon.image.is_some() {
            return Err(
                "component icon slots require monochrome icons; use core Icon for original colors"
                    .into(),
            );
        }
        return Ok(format!("application-icons/{name}").into());
    }
    Err(format!(
        "unknown application icon `{name}`; use an installed Iconify name or explicit SVG"
    ))
}

/// Export the exact installed catalog alongside an application's native bindings.
pub fn typescript() -> String {
    let Some(icons) = APPLICATION_ICONS.get() else {
        return String::new();
    };
    let names =
        serde_json::to_string(&icons.keys().collect::<Vec<_>>()).expect("icon names serialize");
    format!(
        "\nimport {{ registerIconNames }} from \"@solid-gpui/core\";\nexport const applicationIcons = registerIconNames({names} as const);\n"
    )
}

pub(crate) fn image(name: &str) -> Option<Arc<gpui::Image>> {
    APPLICATION_ICONS.get()?.get(name)?.image.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distributed_svg_rejects_active_and_external_content() {
        assert!(
            validate_svg(
                br##"<svg viewBox="0 0 24 24"><path d="M0 0L24 24" stroke="currentColor"/></svg>"##
            )
            .is_ok()
        );
        for svg in [
            r#"<svg viewBox="0 0 24 24"><script/></svg>"#,
            r#"<svg viewBox="0 0 24 24"><path onclick="x()"/></svg>"#,
            r#"<svg viewBox="0 0 24 24"><image href="https://example.org/a"/></svg>"#,
        ] {
            assert!(validate_svg(svg.as_bytes()).is_err());
        }
    }
}
