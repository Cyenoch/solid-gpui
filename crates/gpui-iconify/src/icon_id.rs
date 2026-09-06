/// How an embedded icon should be painted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IconKind {
    /// Single-color `currentColor` body. GPUI stores an alpha mask and tints it.
    Mono,
    /// Hard-coded colors. Painted as an RGBA image; `.text_color()` is ignored.
    Palette,
    /// Both `currentColor` and hex in one body. Painted like [`IconKind::Palette`].
    Mixed,
}

include!(concat!(env!("OUT_DIR"), "/icon_id.rs"));

impl IconId {
    /// Slice of every compiled icon. Same as [`IconId::ALL`].
    pub const fn all() -> &'static [Self] {
        Self::ALL
    }
}

impl std::fmt::Display for IconId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_iconify_name())
    }
}

#[cfg(test)]
mod tests {
    use super::{IconId, IconKind};

    #[test]
    fn from_name_accepts_only_iconify_identity() {
        assert_eq!(IconId::from_name("lucide:play"), Some(IconId::LucidePlay));
        assert_eq!(IconId::from_name("lucide:not-a-real-icon"), None);
        assert_eq!(IconId::from_name("iconify/lucide/play.svg"), None);
    }

    #[test]
    fn from_asset_path_accepts_only_cache_keys() {
        assert_eq!(
            IconId::from_asset_path("iconify/lucide/play.svg"),
            Some(IconId::LucidePlay)
        );
        assert_eq!(IconId::from_asset_path("lucide:play"), None);
    }

    #[test]
    fn lucide_play_is_mono() {
        let id = IconId::LucidePlay;
        assert_eq!(id.as_iconify_name(), "lucide:play");
        assert_eq!(id.cache_key(), "iconify/lucide/play.svg");
        assert_eq!(id.kind(), IconKind::Mono);
        let svg = core::str::from_utf8(id.svg());
        assert!(svg.is_ok());
        if let Ok(svg) = svg {
            assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox="));
        }
    }

    #[test]
    fn alias_home_keeps_requested_name() {
        assert_eq!(IconId::from_name("lucide:home"), Some(IconId::LucideHome));
        assert_eq!(IconId::LucideHome.svg(), IconId::LucideHouse.svg());
    }

    #[cfg(feature = "swatch")]
    #[test]
    fn swatch_is_palette() {
        assert_eq!(IconId::SwatchGrid.kind(), IconKind::Palette);
    }
}
