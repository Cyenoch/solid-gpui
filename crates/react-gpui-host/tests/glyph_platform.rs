#![cfg(target_os = "macos")]

use gpui::{IsZero, Point, RenderGlyphParams, font, px};

/// Proves the native platform text path can produce a real glyph bitmap.
///
/// The production host must compile `gpui_macos` with `font-kit`; otherwise the
/// platform falls back to `NoopTextSystem`, whose glyph raster bounds are zero
/// and whose bitmap is empty even though text layout/underlines still run.
#[test]
fn native_platform_glyph_raster_probe() {
    let platform = gpui_platform::current_platform(false);

    let text_system = platform.text_system();
    let font_id = text_system
        .font_id(&font(".SystemUIFont"))
        .expect("system UI font should resolve");
    let glyph_id = text_system
        .glyph_for_char(font_id, 'G')
        .expect("system UI font should map G to a glyph");
    let params = RenderGlyphParams {
        font_id,
        glyph_id,
        font_size: px(20.0),
        subpixel_variant: Point { x: 0, y: 0 },
        scale_factor: 2.0,
        is_emoji: false,
        subpixel_rendering: false,
        dilation: 0,
    };
    let raster_bounds = text_system
        .glyph_raster_bounds(&params)
        .expect("native text system should compute glyph raster bounds");
    let (raster_size, bitmap) = text_system
        .rasterize_glyph(&params, raster_bounds)
        .expect("native text system should rasterize a glyph");

    assert!(
        !raster_bounds.is_zero(),
        "glyph raster bounds are zero: {raster_bounds:?}"
    );
    assert!(
        !raster_size.is_zero(),
        "glyph raster size is zero: {raster_size:?}"
    );
    assert!(
        bitmap.iter().any(|alpha| *alpha != 0),
        "glyph bitmap has no covered pixels: {} bytes",
        bitmap.len()
    );
}

/// The mocked test context still resolves metrics for the same text. This
/// intentionally remains green when the native raster probe is red: it
/// isolates the failure to glyph filling rather than font lookup or shaping.
#[test]
fn test_context_resolves_same_glyph_metrics() {
    let context = gpui::TestAppContext::single();
    let text_system = context.text_system();
    let font_id = text_system.resolve_font(&font(".SystemUIFont"));
    let bounds = text_system
        .typographic_bounds(font_id, px(20.0), 'G')
        .expect("TestAppContext should resolve probe glyph metrics");
    assert!(
        !bounds.is_zero(),
        "TestAppContext should return nonzero probe glyph metrics"
    );
}
