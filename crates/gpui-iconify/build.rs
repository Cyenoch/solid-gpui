#[path = "src/emit.rs"]
mod emit;
#[path = "src/iconify/mod.rs"]
mod iconify;

use emit::{emit_icon_id, emit_icon_macro};
use iconify::{
    CodegenError, EmbeddedIcon, embed_preset, ensure_allowlist_known, ensure_catalog,
    ensure_preset_enabled, load_allowlist, load_palette_flag, preset_feature_enabled,
};
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=allowlist.toml");
    println!("cargo:rerun-if-changed=vendor");
    println!("cargo:rerun-if-changed=src/iconify");
    println!("cargo:rerun-if-changed=src/emit.rs");
    if let Err(error) = generate() {
        eprintln!("gpui-iconify build failed: {error}");
        std::process::exit(1);
    }
}

fn generate() -> Result<(), CodegenError> {
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR")
            .map_err(|error| CodegenError::Io(format!("CARGO_MANIFEST_DIR: {error}")))?,
    );
    let out_dir = PathBuf::from(
        env::var("OUT_DIR").map_err(|error| CodegenError::Io(format!("OUT_DIR: {error}")))?,
    );
    let vendor_root = manifest_dir.join("vendor/iconify");
    let allowlist = load_allowlist(&manifest_dir.join("allowlist.toml"))?;
    ensure_allowlist_known(&allowlist)?;
    let mut icons = Vec::new();
    for (prefix, names) in allowlist {
        let enabled = preset_feature_enabled(&prefix);
        if prefix == "lucide" {
            ensure_preset_enabled(&prefix, enabled)?;
        }
        if !enabled {
            continue;
        }
        let palette = load_palette_flag(&vendor_root, &prefix)?;
        icons.extend(embed_preset(&vendor_root, &prefix, &names, palette)?);
    }
    ensure_catalog(&icons)?;
    icons.sort_by(|left, right| left.variant.cmp(&right.variant));
    write_generated(&out_dir, &icons)
}

fn write_generated(out_dir: &Path, icons: &[EmbeddedIcon]) -> Result<(), CodegenError> {
    std::fs::write(out_dir.join("icon_id.rs"), emit_icon_id(icons))
        .map_err(|error| CodegenError::Io(error.to_string()))?;
    std::fs::write(out_dir.join("icon_macro.rs"), emit_icon_macro(icons))
        .map_err(|error| CodegenError::Io(error.to_string()))?;
    Ok(())
}
