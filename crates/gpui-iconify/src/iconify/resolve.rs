use super::error::CodegenError;
use super::model::{IconLayer, IconifySet, PaintKind, ResolvedIcon};
use std::collections::BTreeSet;

/// Parses an Iconify `icons.json` document.
pub fn parse_iconify_json(bytes: &[u8]) -> Result<IconifySet, CodegenError> {
    serde_json::from_slice(bytes).map_err(|error| CodegenError::Json(error.to_string()))
}

/// Parses Iconify `info.json` and returns `info.palette`.
pub fn parse_info_palette(bytes: &[u8]) -> Result<bool, CodegenError> {
    let info: super::model::IconifyInfo =
        serde_json::from_slice(bytes).map_err(|error| CodegenError::Json(error.to_string()))?;
    Ok(info.palette)
}

/// Resolves one icon or alias against a parsed set.
pub fn resolve_icon(
    set: &IconifySet,
    name: &str,
    palette: bool,
) -> Result<ResolvedIcon, CodegenError> {
    let chain = parent_chain(set, name)?;
    let mut acc = IconLayer::default();
    for layer in &chain {
        acc = merge_icon_data(layer, &acc);
    }
    let defaults = IconLayer {
        left: set.left,
        top: set.top,
        width: set.width,
        height: set.height,
        ..IconLayer::default()
    };
    let merged = merge_icon_data(&defaults, &acc);
    let body = merged
        .body
        .clone()
        .ok_or_else(|| CodegenError::MissingIcon {
            prefix: set.prefix.clone(),
            name: name.to_string(),
        })?;
    Ok(ResolvedIcon {
        prefix: set.prefix.clone(),
        name: name.to_string(),
        kind: classify_kind(palette, &body),
        body,
        left: merged.left.unwrap_or(0.0),
        top: merged.top.unwrap_or(0.0),
        width: merged.width.unwrap_or(16.0),
        height: merged.height.unwrap_or(16.0),
        h_flip: merged.h_flip.unwrap_or(false),
        v_flip: merged.v_flip.unwrap_or(false),
        rotate: normalize_rotate(merged.rotate.unwrap_or(0)),
        hidden: merged.hidden.unwrap_or(false),
    })
}

/// Classifies a body using collection `palette` plus hex/`currentColor` hints.
pub fn classify_kind(palette: bool, body: &str) -> PaintKind {
    let has_current = body.contains("currentColor");
    if palette {
        PaintKind::Palette
    } else if has_current && has_hex_color(body) {
        PaintKind::Mixed
    } else {
        PaintKind::Mono
    }
}

fn parent_chain(set: &IconifySet, name: &str) -> Result<Vec<IconLayer>, CodegenError> {
    let mut chain = Vec::new();
    let mut current = name.to_string();
    let mut seen = BTreeSet::new();
    loop {
        if !seen.insert(current.clone()) {
            return Err(CodegenError::AliasCycle { name: current });
        }
        if let Some(icon) = set.icons.get(&current) {
            let mut layer = icon.props.clone();
            layer.body = Some(icon.body.clone());
            chain.push(layer);
            return Ok(chain);
        }
        if let Some(alias) = set.aliases.get(&current) {
            chain.push(alias.props.clone());
            current = alias.parent.clone();
            continue;
        }
        if chain.is_empty() {
            return Err(CodegenError::MissingIcon {
                prefix: set.prefix.clone(),
                name: name.to_string(),
            });
        }
        return Err(CodegenError::MissingParent {
            name: name.to_string(),
            parent: current,
        });
    }
}

fn merge_icon_data(parent: &IconLayer, child: &IconLayer) -> IconLayer {
    IconLayer {
        body: child.body.clone().or_else(|| parent.body.clone()),
        left: child.left.or(parent.left),
        top: child.top.or(parent.top),
        width: child.width.or(parent.width),
        height: child.height.or(parent.height),
        h_flip: Some(parent.h_flip.unwrap_or(false) != child.h_flip.unwrap_or(false)),
        v_flip: Some(parent.v_flip.unwrap_or(false) != child.v_flip.unwrap_or(false)),
        rotate: Some(normalize_rotate(
            parent.rotate.unwrap_or(0) + child.rotate.unwrap_or(0),
        )),
        hidden: child.hidden.or(parent.hidden),
    }
}

pub(crate) fn normalize_rotate(rotate: i32) -> i32 {
    let mut value = rotate % 4;
    if value < 0 {
        value += 4;
    }
    value
}

fn has_hex_color(body: &str) -> bool {
    let bytes = body.as_bytes();
    let mut index = 0;
    while index + 3 < bytes.len() {
        if bytes[index] == b'#'
            && bytes[index + 1].is_ascii_hexdigit()
            && bytes[index + 2].is_ascii_hexdigit()
            && bytes[index + 3].is_ascii_hexdigit()
        {
            return true;
        }
        index += 1;
    }
    false
}
