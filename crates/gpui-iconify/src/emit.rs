//! Source emission for `build.rs`. Not compiled into the library.

use crate::iconify::{EmbeddedIcon, PaintKind};

/// Emits `IconId` and its inherent methods.
pub fn emit_icon_id(icons: &[EmbeddedIcon]) -> String {
    let mut source = String::new();
    source.push_str("/// Embedded Iconify identity. Only allowlisted icons exist.\n");
    source.push_str("#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]\n");
    source.push_str("#[non_exhaustive]\n");
    source.push_str("pub enum IconId {\n");
    for icon in icons {
        source.push_str("    /// `");
        source.push_str(&icon.resolved.iconify_name());
        source.push_str("`\n    ");
        source.push_str(&icon.variant);
        source.push_str(",\n");
    }
    source.push_str("}\n\nimpl IconId {\n");
    emit_all_const(&mut source, icons);
    emit_const_match(
        &mut source,
        icons,
        "as_iconify_name",
        "Iconify `prefix:name`.",
        "&'static str",
        |icon| format!("\"{}\"", icon.resolved.iconify_name()),
    );
    emit_const_match(
        &mut source,
        icons,
        "cache_key",
        "Asset path used by [`crate::IconAssets`] and GPUI's SVG atlas.",
        "&'static str",
        |icon| format!("\"{}\"", icon.resolved.cache_key()),
    );
    emit_kind_match(&mut source, icons);
    emit_intrinsic(&mut source, icons);
    emit_svg_match(&mut source, icons);
    emit_lookup(
        &mut source,
        icons,
        "from_name",
        "Resolves an Iconify `prefix:name` already compiled into this crate.",
        |icon| icon.resolved.iconify_name(),
    );
    emit_lookup(
        &mut source,
        icons,
        "from_asset_path",
        "Resolves a [`IconId::cache_key`] path.",
        |icon| icon.resolved.cache_key(),
    );
    source.push_str("}\n");
    source
}

/// Emits `icon_id!` and `icon!`. Unknown literals fail compilation.
pub fn emit_icon_macro(icons: &[EmbeddedIcon]) -> String {
    let mut source = String::new();
    emit_literal_macro(
        &mut source,
        "icon_id",
        "/// Compiles an Iconify name literal into [`IconId`].\n///\n/// Unknown names fail compilation.\n",
        icons,
        |_| true,
        |icon| format!("$crate::IconId::{}", icon.variant),
    );
    source.push('\n');
    emit_literal_macro(
        &mut source,
        "icon",
        "/// Compiles an Iconify name literal into [`IconView`].\n///\n/// Unknown names fail compilation.\n",
        icons,
        |_| true,
        |icon| format!("$crate::IconView::new($crate::IconId::{})", icon.variant),
    );
    source
}

fn emit_literal_macro(
    source: &mut String,
    name: &str,
    docs: &str,
    icons: &[EmbeddedIcon],
    include: impl Fn(&EmbeddedIcon) -> bool,
    expansion: impl Fn(&EmbeddedIcon) -> String,
) {
    source.push_str(docs);
    source.push_str("#[macro_export]\nmacro_rules! ");
    source.push_str(name);
    source.push_str(" {\n");
    for icon in icons.iter().filter(|icon| include(icon)) {
        source.push_str("    (\"");
        source.push_str(&icon.resolved.iconify_name());
        source.push_str("\") => {\n        ");
        source.push_str(&expansion(icon));
        source.push_str("\n    };\n");
    }
    source.push_str(
        "    ($name:literal) => {\n        compile_error!(concat!(\"unknown or not-embedded icon: \", $name))\n    };\n}\n",
    );
}

fn emit_all_const(source: &mut String, icons: &[EmbeddedIcon]) {
    source.push_str("    /// Every icon compiled into this crate.\n");
    source.push_str("    pub const ALL: &'static [IconId] = &[\n");
    for icon in icons {
        source.push_str("        IconId::");
        source.push_str(&icon.variant);
        source.push_str(",\n");
    }
    source.push_str("    ];\n\n");
}

fn emit_const_match(
    source: &mut String,
    icons: &[EmbeddedIcon],
    method: &str,
    doc: &str,
    ty: &str,
    value: impl Fn(&EmbeddedIcon) -> String,
) {
    source.push_str("    /// ");
    source.push_str(doc);
    source.push_str("\n    pub const fn ");
    source.push_str(method);
    source.push_str("(self) -> ");
    source.push_str(ty);
    source.push_str(" {\n        match self {\n");
    for icon in icons {
        source.push_str("            IconId::");
        source.push_str(&icon.variant);
        source.push_str(" => ");
        source.push_str(&value(icon));
        source.push_str(",\n");
    }
    source.push_str("        }\n    }\n\n");
}

fn emit_kind_match(source: &mut String, icons: &[EmbeddedIcon]) {
    emit_const_match(
        source,
        icons,
        "kind",
        "Paint path for this icon.",
        "IconKind",
        |icon| match icon.resolved.kind {
            PaintKind::Mono => "IconKind::Mono".to_string(),
            PaintKind::Palette => "IconKind::Palette".to_string(),
            PaintKind::Mixed => "IconKind::Mixed".to_string(),
        },
    );
}

fn emit_intrinsic(source: &mut String, icons: &[EmbeddedIcon]) {
    emit_const_match(
        source,
        icons,
        "intrinsic_px",
        "Iconify viewBox width used to scale palette rasters.",
        "f32",
        |icon| format!("{}_f32", icon.resolved.width.max(1.0)),
    );
}

fn emit_svg_match(source: &mut String, icons: &[EmbeddedIcon]) {
    source.push_str("    /// Complete SVG document bytes. `'static` and size-free.\n");
    source.push_str("    pub const fn svg(self) -> &'static [u8] {\n        match self {\n");
    for icon in icons {
        source.push_str("            IconId::");
        source.push_str(&icon.variant);
        source.push_str(" => ");
        source.push_str(&rust_raw_byte_string(&icon.svg));
        source.push_str(",\n");
    }
    source.push_str("        }\n    }\n\n");
}

fn emit_lookup(
    source: &mut String,
    icons: &[EmbeddedIcon],
    method: &str,
    doc: &str,
    key: impl Fn(&EmbeddedIcon) -> String,
) {
    source.push_str("    /// ");
    source.push_str(doc);
    source.push_str("\n    pub fn ");
    source.push_str(method);
    source.push_str("(name: &str) -> Option<Self> {\n        match name {\n");
    for icon in icons {
        source.push_str("            \"");
        source.push_str(&key(icon));
        source.push_str("\" => Some(IconId::");
        source.push_str(&icon.variant);
        source.push_str("),\n");
    }
    source.push_str("            _ => None,\n        }\n    }\n\n");
}

fn rust_raw_byte_string(value: &str) -> String {
    let mut hashes = 1;
    loop {
        let closer = format!("\"{}", "#".repeat(hashes));
        if !value.contains(&closer) {
            let delim = "#".repeat(hashes);
            return format!("br{delim}\"{value}\"{delim}");
        }
        hashes += 1;
    }
}
