use super::error::CodegenError;
use super::model::{AllowlistFile, EmbeddedIcon};
use super::resolve::{parse_iconify_json, parse_info_palette, resolve_icon};
use super::svg::{icon_to_svg, variant_ident};
use std::collections::BTreeMap;
use std::path::Path;

/// Loads `allowlist.toml`.
pub fn load_allowlist(path: &Path) -> Result<BTreeMap<String, Vec<String>>, CodegenError> {
    let text =
        std::fs::read_to_string(path).map_err(|error| CodegenError::Io(error.to_string()))?;
    let parsed: AllowlistFile =
        toml::from_str(&text).map_err(|error| CodegenError::Toml(error.to_string()))?;
    Ok(parsed
        .sets
        .into_iter()
        .map(|(prefix, set)| (prefix, set.icons))
        .collect())
}

/// Reads `info.json` when present; missing file means mono.
pub fn load_palette_flag(vendor_root: &Path, prefix: &str) -> Result<bool, CodegenError> {
    let path = vendor_root.join(prefix).join("info.json");
    if !path.exists() {
        return Ok(false);
    }
    let bytes = std::fs::read(path).map_err(|error| CodegenError::Io(error.to_string()))?;
    parse_info_palette(&bytes)
}

/// Loads one vendor preset and resolves the requested names.
pub fn embed_preset(
    vendor_root: &Path,
    prefix: &str,
    names: &[String],
    palette: bool,
) -> Result<Vec<EmbeddedIcon>, CodegenError> {
    let json = std::fs::read(vendor_root.join(prefix).join("icons.json"))
        .map_err(|error| CodegenError::Io(error.to_string()))?;
    let set = parse_iconify_json(&json)?;
    let mut embedded = Vec::new();
    for name in names {
        let resolved = resolve_icon(&set, name, palette)?;
        if resolved.hidden {
            return Err(CodegenError::HiddenIcon {
                prefix: prefix.to_string(),
                name: name.clone(),
            });
        }
        let variant = variant_ident(prefix, name);
        let svg = icon_to_svg(&resolved);
        embedded.push(EmbeddedIcon {
            resolved,
            svg,
            variant,
        });
    }
    Ok(embedded)
}

/// Whether a prefix names one of the vendored presets.
pub fn ensure_preset_known(prefix: &str) -> Result<(), CodegenError> {
    if matches!(prefix, "lucide" | "swatch") {
        Ok(())
    } else {
        Err(CodegenError::UnknownPreset {
            prefix: prefix.to_string(),
        })
    }
}

/// Rejects allowlist entries whose prefixes are not vendored presets.
pub fn ensure_allowlist_known(
    allowlist: &BTreeMap<String, Vec<String>>,
) -> Result<(), CodegenError> {
    for prefix in allowlist.keys() {
        ensure_preset_known(prefix)?;
    }
    Ok(())
}

/// Feature flag for a vendored prefix.
pub fn preset_feature_enabled(prefix: &str) -> bool {
    match prefix {
        "lucide" => std::env::var_os("CARGO_FEATURE_LUCIDE").is_some(),
        "swatch" => std::env::var_os("CARGO_FEATURE_SWATCH").is_some(),
        _ => false,
    }
}

/// Errors when a listed preset's feature is off.
pub fn ensure_preset_enabled(prefix: &str, enabled: bool) -> Result<(), CodegenError> {
    if enabled {
        Ok(())
    } else {
        Err(CodegenError::DisabledPreset {
            prefix: prefix.to_string(),
        })
    }
}

/// Errors when the embed list is empty.
pub fn ensure_catalog(icons: &[EmbeddedIcon]) -> Result<(), CodegenError> {
    if icons.is_empty() {
        Err(CodegenError::EmptyCatalog)
    } else {
        Ok(())
    }
}
