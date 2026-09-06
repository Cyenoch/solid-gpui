#[path = "../src/iconify/mod.rs"]
mod iconify;

use iconify::resolve::{classify_kind, parse_iconify_json, resolve_icon};
use iconify::svg::{icon_to_svg, variant_ident};
use iconify::{
    CodegenError, EmbeddedIcon, PaintKind, embed_preset, ensure_allowlist_known, ensure_catalog,
    ensure_preset_enabled, ensure_preset_known, load_allowlist, load_palette_flag,
    preset_feature_enabled,
};

#[test]
fn alias_hflip_merges_onto_parent_body() {
    let parsed = parse_iconify_json(include_str!("fixtures/iconify/alias-flip.json").as_bytes());
    assert!(parsed.is_ok());
    if let Ok(set) = parsed {
        let right = resolve_icon(&set, "arrow-right", false);
        assert!(right.is_ok());
        if let Ok(right) = right {
            assert!(right.h_flip);
            assert!(icon_to_svg(&right).contains("scale(-1 1)"));
        }
    }
}

#[test]
fn double_hflip_cancels() {
    let parsed = parse_iconify_json(include_str!("fixtures/iconify/alias-flip.json").as_bytes());
    assert!(parsed.is_ok());
    if let Ok(set) = parsed {
        let again = resolve_icon(&set, "arrow-left-again", false);
        assert!(again.is_ok());
        if let Ok(again) = again {
            assert!(!again.h_flip);
        }
    }
}

#[test]
fn rotate_adds_modulo_four() {
    let parsed = parse_iconify_json(include_str!("fixtures/iconify/alias-rotate.json").as_bytes());
    assert!(parsed.is_ok());
    if let Ok(set) = parsed {
        let turned = resolve_icon(&set, "turned", false);
        assert!(turned.is_ok());
        if let Ok(turned) = turned {
            assert_eq!(turned.rotate, 3);
        }
    }
}

#[test]
fn alias_dimension_overrides_parent() {
    let parsed = parse_iconify_json(include_str!("fixtures/iconify/alias-dims.json").as_bytes());
    assert!(parsed.is_ok());
    if let Ok(set) = parsed {
        let large = resolve_icon(&set, "house-32", false);
        assert!(large.is_ok());
        if let Ok(large) = large {
            assert_eq!(large.width, 32.0);
            assert_eq!(large.left, -4.0);
        }
    }
}

#[test]
fn collection_defaults_fill_missing_viewbox() {
    let parsed = parse_iconify_json(include_str!("fixtures/iconify/defaults.json").as_bytes());
    assert!(parsed.is_ok());
    if let Ok(set) = parsed {
        let home = resolve_icon(&set, "home", false);
        assert!(home.is_ok());
        if let Ok(home) = home {
            assert_eq!(home.width, 24.0);
            assert_eq!(home.cache_key(), "iconify/mdi/home.svg");
        }
    }
}

#[test]
fn implicit_viewbox_is_sixteen() {
    let parsed = parse_iconify_json(include_str!("fixtures/iconify/bare.json").as_bytes());
    assert!(parsed.is_ok());
    if let Ok(set) = parsed {
        let mark = resolve_icon(&set, "mark", false);
        assert!(mark.is_ok());
        if let Ok(mark) = mark {
            assert_eq!(mark.width, 16.0);
        }
    }
}

#[test]
fn hidden_icon_is_flagged() {
    let parsed = parse_iconify_json(include_str!("fixtures/iconify/hidden.json").as_bytes());
    assert!(parsed.is_ok());
    if let Ok(set) = parsed {
        let gone = resolve_icon(&set, "gone", false);
        assert!(gone.is_ok());
        if let Ok(gone) = gone {
            assert!(gone.hidden);
        }
    }
}

#[test]
fn missing_and_cycle_are_errors() {
    let parsed = parse_iconify_json(include_str!("fixtures/iconify/cycle.json").as_bytes());
    assert!(parsed.is_ok());
    if let Ok(set) = parsed {
        assert!(matches!(
            resolve_icon(&set, "nope", false),
            Err(CodegenError::MissingIcon { name, .. }) if name == "nope"
        ));
        assert!(matches!(
            resolve_icon(&set, "loop-a", false),
            Err(CodegenError::AliasCycle { .. })
        ));
    }
}

#[test]
fn mixed_body_is_detected_when_collection_is_mono() {
    assert_eq!(
        classify_kind(
            false,
            r##"<path fill="currentColor"/><path fill="#ff00aa"/>"##
        ),
        PaintKind::Mixed
    );
    assert_eq!(
        classify_kind(true, r#"<path fill="currentColor"/>"#),
        PaintKind::Palette
    );
}

#[test]
fn svg_omits_layout_width_height() {
    let parsed = parse_iconify_json(include_str!("fixtures/iconify/defaults.json").as_bytes());
    assert!(parsed.is_ok());
    if let Ok(set) = parsed {
        let home = resolve_icon(&set, "home", false);
        assert!(home.is_ok());
        if let Ok(home) = home {
            let svg = icon_to_svg(&home);
            let open_end = svg.find('>').unwrap_or(0);
            assert!(!svg[..open_end].contains(" width="));
        }
    }
}

#[test]
fn variant_ident_is_prefix_plus_pascal_name() {
    assert_eq!(
        variant_ident("lucide", "triangle-alert"),
        "LucideTriangleAlert"
    );
}

#[test]
fn ensure_helpers_surface_build_failures() {
    assert!(ensure_preset_enabled("lucide", true).is_ok());
    assert!(matches!(
        ensure_preset_enabled("lucide", false),
        Err(CodegenError::DisabledPreset { .. })
    ));
    assert!(matches!(
        ensure_preset_known("missing-prefix"),
        Err(CodegenError::UnknownPreset { prefix }) if prefix == "missing-prefix"
    ));
    assert!(matches!(
        ensure_catalog(&[]),
        Err(CodegenError::EmptyCatalog)
    ));
}

#[test]
fn unknown_allowlist_prefix_fails_generation_validation() {
    let mut allowlist = std::collections::BTreeMap::new();
    allowlist.insert("missing-prefix".to_owned(), Vec::new());

    assert!(matches!(
        ensure_allowlist_known(&allowlist),
        Err(CodegenError::UnknownPreset { prefix }) if prefix == "missing-prefix"
    ));
}

#[test]
fn vendor_allowlist_embeds_lucide_play() {
    assert!(!preset_feature_enabled("missing-prefix"));
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let allowlist = load_allowlist(&root.join("allowlist.toml"));
    assert!(allowlist.is_ok());
    if let Ok(map) = allowlist {
        let names = map.get("lucide");
        assert!(names.is_some());
        if let Some(names) = names {
            let vendor = root.join("vendor/iconify");
            assert!(matches!(load_palette_flag(&vendor, "lucide"), Ok(false)));
            let embedded = embed_preset(&vendor, "lucide", names, false);
            assert!(embedded.is_ok());
            if let Ok(embedded) = embedded {
                let play: Option<&EmbeddedIcon> =
                    embedded.iter().find(|icon| icon.variant == "LucidePlay");
                assert!(play.is_some());
            }
        }
    }
}
