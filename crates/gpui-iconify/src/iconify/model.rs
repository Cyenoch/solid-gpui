use serde::Deserialize;
use std::collections::BTreeMap;

/// How a resolved body should be painted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaintKind {
    /// `currentColor` body; GPUI tints an alpha mask.
    Mono,
    /// Hard-coded colors; paint as an RGBA frame.
    Palette,
    /// Both `currentColor` and hex; paint as palette and ignore tint.
    Mixed,
}

/// Fully resolved Iconify icon ready to wrap as SVG.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedIcon {
    /// Collection prefix.
    pub prefix: String,
    /// Icon or alias name.
    pub name: String,
    /// Inner SVG markup (no `<svg>` wrapper).
    pub body: String,
    /// viewBox x.
    pub left: f64,
    /// viewBox y.
    pub top: f64,
    /// viewBox width.
    pub width: f64,
    /// viewBox height.
    pub height: f64,
    /// Horizontal flip after alias merge.
    pub h_flip: bool,
    /// Vertical flip after alias merge.
    pub v_flip: bool,
    /// Quarter-turns after alias merge, in `0..4`.
    pub rotate: i32,
    /// `hidden: true` on the resolved record.
    pub hidden: bool,
    /// Paint classification.
    pub kind: PaintKind,
}

impl ResolvedIcon {
    /// Iconify identity (`prefix:name`).
    pub fn iconify_name(&self) -> String {
        format!("{}:{}", self.prefix, self.name)
    }

    /// Asset path that is not a URI scheme (so `img()` does not HTTP-fetch it).
    pub fn cache_key(&self) -> String {
        format!("iconify/{}/{}.svg", self.prefix, self.name)
    }
}

/// One icon selected for embedding.
#[derive(Clone, Debug)]
pub struct EmbeddedIcon {
    /// Resolved Iconify record.
    pub resolved: ResolvedIcon,
    /// Complete `<svg>` document.
    pub svg: String,
    /// Generated enum variant ident.
    pub variant: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AllowlistFile {
    #[serde(flatten)]
    pub sets: BTreeMap<String, AllowlistSet>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AllowlistSet {
    pub icons: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IconifyInfo {
    #[serde(default)]
    pub palette: bool,
}

#[derive(Debug, Deserialize)]
pub struct IconifySet {
    pub prefix: String,
    #[serde(default)]
    pub left: Option<f64>,
    #[serde(default)]
    pub top: Option<f64>,
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub height: Option<f64>,
    #[serde(default)]
    pub icons: BTreeMap<String, IconifyIcon>,
    #[serde(default)]
    pub aliases: BTreeMap<String, IconifyAlias>,
}

#[derive(Debug, Deserialize)]
pub struct IconifyIcon {
    pub body: String,
    #[serde(flatten)]
    pub props: IconLayer,
}

#[derive(Debug, Deserialize)]
pub struct IconifyAlias {
    pub parent: String,
    #[serde(flatten)]
    pub props: IconLayer,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct IconLayer {
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub left: Option<f64>,
    #[serde(default)]
    pub top: Option<f64>,
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub height: Option<f64>,
    #[serde(default, rename = "hFlip")]
    pub h_flip: Option<bool>,
    #[serde(default, rename = "vFlip")]
    pub v_flip: Option<bool>,
    #[serde(default)]
    pub rotate: Option<i32>,
    #[serde(default)]
    pub hidden: Option<bool>,
}
