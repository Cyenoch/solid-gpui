use gpui::Pixels;

/// Logical icon size in pixels. Named constants are conveniences; any positive finite value is valid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IconSize(f32);

impl IconSize {
    /// 12px.
    pub const X_SMALL: Self = Self(12.0);
    /// 14px.
    pub const SMALL: Self = Self(14.0);
    /// 16px.
    pub const MEDIUM: Self = Self(16.0);
    /// 20px.
    pub const LARGE: Self = Self(20.0);
    /// 24px.
    pub const X_LARGE: Self = Self(24.0);
    /// 32px.
    pub const LOGO: Self = Self(32.0);

    /// Exact logical size. Non-finite and non-positive values do not paint.
    pub const fn new(pixels: f32) -> Self {
        Self(pixels)
    }

    /// Logical pixel length.
    pub const fn px(self) -> f32 {
        self.0
    }

    /// Size that GPUI can paint (positive and finite).
    pub fn paintable_px(self) -> Option<f32> {
        let pixels = self.0;
        (pixels.is_finite() && pixels > 0.0).then_some(pixels)
    }
}

impl Default for IconSize {
    fn default() -> Self {
        Self::MEDIUM
    }
}

impl From<Pixels> for IconSize {
    fn from(pixels: Pixels) -> Self {
        Self::new(pixels.as_f32())
    }
}

impl From<f32> for IconSize {
    fn from(pixels: f32) -> Self {
        Self::new(pixels)
    }
}

#[cfg(test)]
mod tests {
    use super::IconSize;
    use gpui::px;

    #[test]
    fn from_pixels_keeps_exact_value() {
        assert_eq!(IconSize::from(px(23.)).px(), 23.0);
        assert_eq!(IconSize::new(18.5).px(), 18.5);
    }

    #[test]
    fn named_constants_match_documented_px() {
        assert_eq!(IconSize::MEDIUM.px(), 16.0);
        assert_eq!(IconSize::LOGO.px(), 32.0);
    }

    #[test]
    fn non_positive_is_not_paintable() {
        assert!(IconSize::new(0.0).paintable_px().is_none());
        assert!(IconSize::new(-4.0).paintable_px().is_none());
        assert!(IconSize::new(f32::NAN).paintable_px().is_none());
    }
}
