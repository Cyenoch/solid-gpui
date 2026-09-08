use super::support::*;
use crate::protocol::{ExtensionField, ExtensionProperties, ExtensionValue};
use crate::tree::KIND_EXTENSION;

#[test]
fn synthetic_view_root_is_required_and_empty_unmount_is_valid() {
    let mut store = NodeStore::default();
    store.apply_snapshot(synthetic_root(1)).unwrap();
    assert_eq!(store.root().unwrap().id, 1);
    assert_eq!(store.len(), 1);

    let mut empty = NodeStore::default();
    let error = empty
        .apply_snapshot(Snapshot::new(7, 3, 0, 1, Vec::new()))
        .unwrap_err();
    assert_eq!(error, TreeError::RootCount(0));

    let wrong_id = root_snapshot(1, vec![Node::new(2, 0, 0, KIND_VIEW)]);
    assert!(matches!(
        NodeStore::default().apply_snapshot(wrong_id),
        Err(TreeError::InvalidSyntheticRoot { .. })
    ));

    let wrong_kind = root_snapshot(1, vec![Node::new(1, 0, 0, KIND_TEXT)]);
    assert!(matches!(
        NodeStore::default().apply_snapshot(wrong_kind),
        Err(TreeError::InvalidSyntheticRoot { .. })
    ));
}

#[test]
fn accessibility_properties_validate_and_keep_stable_ids() {
    let mut label = Node::new(1, 0, 0, KIND_VIEW);
    label.accessibility = Some(AccessibilityProperties {
        role: 2,
        label: Some("First name".into()),
        description: Some("Controlled input".into()),
        disabled: false,
        checked: None,
        selected: Some(true),
        value: Some("A".into()),
        expanded: Some(false),
        level: None,
        live: None,
    });
    let store = {
        let mut store = NodeStore::default();
        store.apply_snapshot(root_snapshot(1, vec![label])).unwrap();
        store
    };
    assert_eq!(
        store.get(1).unwrap().accessibility_id.as_ref(),
        "solid-gpui-node-1"
    );
    assert_eq!(
        store
            .get(1)
            .unwrap()
            .accessibility
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("First name")
    );

    let mut invalid = Node::new(1, 0, 0, KIND_VIEW);
    invalid.accessibility = Some(AccessibilityProperties {
        role: 2,
        label: None,
        description: None,
        disabled: false,
        checked: Some(true),
        selected: None,
        value: None,
        expanded: None,
        level: None,
        live: None,
    });
    assert!(matches!(
        NodeStore::default().apply_snapshot(root_snapshot(1, vec![invalid])),
        Err(TreeError::InvalidProperties { .. })
    ));
}

#[test]
fn accessibility_patch_updates_validate_role_and_checked_constraints() {
    let mut store = NodeStore::default();
    store
        .apply_snapshot(synthetic_root(1))
        .expect("initial root");
    let invalid = AccessibilityProperties {
        role: 2,
        label: None,
        description: None,
        disabled: false,
        checked: Some(true),
        selected: None,
        value: None,
        expanded: None,
        level: None,
        live: None,
    };
    let patch = Patch::new(
        7,
        3,
        1,
        2,
        vec![PatchOperation::Update {
            id: 1,
            mask: UPDATE_ACCESSIBILITY,
            style: None,
            text: None,
            listener_id: 0,
            host_properties: None,
            accessibility: Some(invalid),
            focusable: false,
            selectable: false,
            tooltip: None,
            accepts_pointer_move: false,
        }],
    );
    assert!(matches!(
        store.apply_patch(patch),
        Err(TreeError::InvalidPatchOperation { .. })
    ));
    assert!(store.get(1).unwrap().accessibility.is_none());
}

#[test]
fn accessibility_patch_updates_a_valid_label() {
    let mut store = NodeStore::default();
    store
        .apply_snapshot(synthetic_root(1))
        .expect("initial root");
    let patch = Patch::new(
        7,
        3,
        1,
        2,
        vec![PatchOperation::Update {
            id: 1,
            mask: UPDATE_ACCESSIBILITY,
            style: None,
            text: None,
            listener_id: 0,
            host_properties: None,
            accessibility: Some(AccessibilityProperties {
                role: 1,
                label: Some("Updated label".into()),
                description: None,
                disabled: false,
                checked: None,
                selected: None,
                value: None,
                expanded: Some(true),
                level: None,
                live: None,
            }),
            focusable: false,
            selectable: false,
            tooltip: None,
            accepts_pointer_move: false,
        }],
    );
    store
        .apply_patch(patch)
        .expect("valid accessibility label update");
    assert_eq!(
        store
            .get(1)
            .unwrap()
            .accessibility
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("Updated label")
    );
}

#[test]
fn accessibility_patch_can_clear_existing_properties() {
    let mut node = Node::new(1, 0, 0, KIND_VIEW);
    node.accessibility = Some(AccessibilityProperties {
        role: 1,
        label: Some("before".into()),
        description: None,
        disabled: false,
        checked: None,
        selected: None,
        value: None,
        expanded: None,
        level: None,
        live: None,
    });
    let mut store = NodeStore::default();
    store
        .apply_snapshot(root_snapshot(1, vec![node]))
        .expect("initial accessibility");
    store
        .apply_patch(Patch::new(
            7,
            3,
            1,
            2,
            vec![PatchOperation::Update {
                id: 1,
                mask: UPDATE_ACCESSIBILITY,
                style: None,
                text: None,
                listener_id: 0,
                host_properties: None,
                accessibility: None,
                focusable: false,
                selectable: false,
                tooltip: None,
                accepts_pointer_move: false,
            }],
        ))
        .expect("accessibility removal");
    assert!(store.get(1).unwrap().accessibility.is_none());
}

#[test]
fn invalid_revision_is_rejected_and_last_good_tree_is_retained() {
    let mut store = NodeStore::default();
    store.apply_snapshot(synthetic_root(1)).unwrap();

    let stale = Snapshot::new(7, 3, 0, 2, vec![Node::new(1, 0, 0, KIND_VIEW)]);
    assert!(matches!(
        store.apply_snapshot(stale),
        Err(TreeError::BaseRevisionMismatch {
            base_revision: 0,
            current_revision: 1
        })
    ));
    assert_eq!(store.revision(), 1);
    assert_eq!(store.len(), 1);

    let invalid_order = Snapshot::new(7, 3, 2, 2, vec![Node::new(1, 0, 0, KIND_VIEW)]);
    assert!(matches!(
        store.apply_snapshot(invalid_order),
        Err(TreeError::InvalidRevisionOrder { .. })
    ));
    assert_eq!(store.revision(), 1);
}

#[test]
fn duplicate_orphan_and_parent_after_child_trees_are_rejected() {
    let duplicate = root_snapshot(
        1,
        vec![
            Node::new(1, 0, 0, KIND_VIEW),
            view_node(2, 1, 0),
            view_node(2, 1, 1),
        ],
    );
    assert!(matches!(
        NodeStore::default().apply_snapshot(duplicate),
        Err(TreeError::DuplicateId(2))
    ));

    let orphan = root_snapshot(1, vec![Node::new(1, 0, 0, KIND_VIEW), view_node(2, 99, 0)]);
    assert!(matches!(
        NodeStore::default().apply_snapshot(orphan),
        Err(TreeError::MissingParent { parent_id: 99, .. })
    ));

    // This is a cycle in the parent relation; parent-before-child rejects it
    // at the first node rather than allowing a cycle into the candidate store.
    let cycle = root_snapshot(
        1,
        vec![
            Node::new(1, 0, 0, KIND_VIEW),
            view_node(2, 3, 0),
            view_node(3, 2, 0),
        ],
    );
    assert!(matches!(
        NodeStore::default().apply_snapshot(cycle),
        Err(TreeError::MissingParent { parent_id: 3, .. })
    ));
}

#[test]
fn text_containment_listener_and_child_indexes_are_validated() {
    let improper_text = root_snapshot(
        1,
        vec![
            Node::new(1, 0, 0, KIND_VIEW),
            Node::new(2, 1, 0, KIND_TEXT),
            view_node(3, 2, 0),
        ],
    );
    assert!(matches!(
        NodeStore::default().apply_snapshot(improper_text),
        Err(TreeError::InvalidChild { .. })
    ));

    let mut raw = Node::new(3, 1, 0, KIND_RAW_TEXT);
    raw.text = Some("orphan text".into());
    let raw_outside_text = root_snapshot(1, vec![Node::new(1, 0, 0, KIND_VIEW), raw]);
    assert!(matches!(
        NodeStore::default().apply_snapshot(raw_outside_text),
        Err(TreeError::InvalidChild { .. })
    ));

    let mut first_text = Node::new(3, 2, 0, KIND_RAW_TEXT);
    first_text.text = Some("Count: ".into());
    let mut second_text = Node::new(4, 2, 1, KIND_RAW_TEXT);
    second_text.text = Some("0".into());
    let mut text_store = NodeStore::default();
    text_store
        .apply_snapshot(root_snapshot(
            1,
            vec![
                Node::new(1, 0, 0, KIND_VIEW),
                Node::new(2, 1, 0, KIND_TEXT),
                first_text,
                second_text,
            ],
        ))
        .unwrap();
    assert_eq!(
        text_store.get(2).unwrap().text_content.as_deref(),
        Some("Count: 0")
    );

    let mut pointer_listener_on_view = view_node(2, 1, 0);
    pointer_listener_on_view.listener_id = 9;
    let pointer_listener_snapshot = root_snapshot(
        1,
        vec![Node::new(1, 0, 0, KIND_VIEW), pointer_listener_on_view],
    );
    NodeStore::default()
        .apply_snapshot(pointer_listener_snapshot)
        .unwrap();
    let mut valid_text_listener = Node::new(2, 1, 0, KIND_TEXT);
    valid_text_listener.listener_id = 9;
    NodeStore::default()
        .apply_snapshot(root_snapshot(
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), valid_text_listener],
        ))
        .expect("Text listener is valid");

    let non_contiguous = root_snapshot(
        1,
        vec![
            Node::new(1, 0, 0, KIND_VIEW),
            view_node(2, 1, 0),
            view_node(3, 1, 2),
        ],
    );
    assert!(matches!(
        NodeStore::default().apply_snapshot(non_contiguous),
        Err(TreeError::NonContiguousChildIndex { .. })
    ));
}

#[test]
fn style_values_must_be_finite_and_non_negative() {
    for (index, style) in [
        Style {
            width: Some(-1.0),
            ..Style::default()
        },
        Style {
            height: Some(f32::NAN),
            ..Style::default()
        },
        Style {
            padding: Some(f32::INFINITY),
            ..Style::default()
        },
        Style {
            gap: Some(-0.1),
            ..Style::default()
        },
        Style {
            flex_grow: Some(-1.0),
            ..Style::default()
        },
        Style {
            border_radius: Some(-1.0),
            ..Style::default()
        },
        Style {
            font_size: Some(0.0),
            ..Style::default()
        },
        Style {
            line_clamp: Some(0),
            ..Style::default()
        },
        Style {
            line_clamp: Some(101),
            ..Style::default()
        },
        Style {
            margin_top: Some(-1.0),
            ..Style::default()
        },
        Style {
            line_height: Some(f32::NAN),
            ..Style::default()
        },
        Style {
            min_width: Some(f32::INFINITY),
            ..Style::default()
        },
        Style {
            flex_shrink: Some(-1.0),
            ..Style::default()
        },
        Style {
            box_shadows: Some(vec![
                crate::BoxShadow {
                    offset_x: 0.0,
                    offset_y: 0.0,
                    blur_radius: 1.0,
                    spread_radius: 0.0,
                    color_rgba: 0,
                    inset: false,
                },
                crate::BoxShadow {
                    offset_x: 1.0,
                    offset_y: 1.0,
                    blur_radius: 1.0,
                    spread_radius: 0.0,
                    color_rgba: 0,
                    inset: false,
                },
                crate::BoxShadow {
                    offset_x: 2.0,
                    offset_y: 2.0,
                    blur_radius: 1.0,
                    spread_radius: 0.0,
                    color_rgba: 0,
                    inset: false,
                },
            ]),
            ..Style::default()
        },
        Style {
            font_family: Some("x".repeat(65)),
            ..Style::default()
        },
    ]
    .into_iter()
    .enumerate()
    {
        let mut root = Node::new(1, 0, 0, KIND_VIEW);
        root.style = Some(style);
        assert!(
            matches!(
                NodeStore::default().apply_snapshot(root_snapshot(1, vec![root])),
                Err(TreeError::InvalidStyle { .. })
            ),
            "style index {index} unexpectedly accepted"
        );
    }
    let mut valid = Node::new(1, 0, 0, KIND_VIEW);
    valid.style = Some(Style {
        width: Some(10.0),
        height: Some(20.0),
        flex_direction: Some(FlexDirectionCode::RowReverse),
        flex_grow: Some(0.5),
        padding: Some(4.0),
        gap: Some(2.0),
        background_rgba: Some(0xff00ffff),
        justify_content: Some(JustifyContentCode::SpaceBetween),
        align_items: Some(AlignItemsCode::Baseline),
        border_radius: Some(3.0),
        border_width: Some(2.0),
        border_color_rgba: Some(0x11223344),
        font_size: Some(14.0),
        font_weight: Some(FontWeightCode::Semibold),
        color_rgba: Some(0xffffffff),
        opacity: None,
        transition: None,
        overflow: Some(OverflowCode::Scroll),
        line_clamp: Some(3),
        text_overflow: Some(TextOverflowCode::Ellipsis),
        margin_top: Some(1.0),
        margin_right: Some(2.0),
        margin_bottom: Some(3.0),
        margin_left: Some(4.0),
        font_style: Some(FontStyleCode::Italic),
        text_decoration: Some(TextDecorationCode::LineThrough),
        line_height: Some(18.0),
        min_width: Some(4.0),
        max_width: Some(400.0),
        min_height: Some(4.0),
        max_height: Some(200.0),
        flex_shrink: Some(1.0),
        align_self: Some(AlignSelfCode::Center),
        position: Some(PositionCode::Absolute),
        left: Some(-8.0),
        top: Some(4.0),
        right: None,
        bottom: Some(6.0),
        cursor: None,
        text_align: Some(TextAlignCode::Right),
        box_shadows: None,
        font_family: None,
        grid_columns: None,
        grid_rows: None,
        grid_column_span: Some(2),
        grid_row_span: Some(1),
        ..Style::default()
    });
    NodeStore::default()
        .apply_snapshot(root_snapshot(1, vec![valid]))
        .unwrap();
    let mut overlay = Node::new(1, 0, 0, KIND_VIEW);
    overlay.style = Some(Style {
        position: Some(PositionCode::Overlay),
        left: Some(4.0),
        top: Some(8.0),
        width: Some(40.0),
        height: Some(20.0),
        ..Style::default()
    });
    NodeStore::default()
        .apply_snapshot(root_snapshot(1, vec![overlay]))
        .unwrap();
}

#[test]
fn style_wire_round_trips_layout_border_and_text_fields() {
    let mut node = Node::new(1, 0, 0, KIND_VIEW);
    node.style = Some(Style {
        justify_content: Some(JustifyContentCode::SpaceEvenly),
        align_items: Some(AlignItemsCode::Stretch),
        border_radius: Some(8.0),
        border_width: Some(2.0),
        border_color_rgba: Some(0x12345678),
        font_size: Some(16.0),
        font_weight: Some(FontWeightCode::Heavy),
        overflow: Some(OverflowCode::Scroll),
        line_clamp: Some(3),
        text_overflow: Some(TextOverflowCode::Ellipsis),
        margin_top: Some(1.0),
        margin_right: Some(2.0),
        margin_bottom: Some(3.0),
        margin_left: Some(4.0),
        font_style: Some(FontStyleCode::Italic),
        text_decoration: Some(TextDecorationCode::LineThrough),
        line_height: Some(18.0),
        min_width: Some(4.0),
        max_width: Some(400.0),
        min_height: Some(4.0),
        max_height: Some(200.0),
        flex_shrink: Some(0.5),
        align_self: Some(AlignSelfCode::Center),
        position: Some(PositionCode::Absolute),
        left: Some(-8.0),
        top: Some(4.0),
        right: Some(12.0),
        bottom: Some(6.0),
        cursor: Some(CursorCode::RowResize),
        text_align: Some(TextAlignCode::Right),
        box_shadows: Some(vec![crate::BoxShadow {
            offset_x: -2.0,
            offset_y: 3.0,
            blur_radius: 4.0,
            spread_radius: 1.0,
            color_rgba: 0x01020380,
            inset: true,
        }]),
        font_family: Some("Avenir Next".into()),
        transition: Some(Transition {
            duration_ms: 100,
            delay_ms: 0,
            easing: Easing::Linear,
            properties: TRANSITION_WIDTH | TRANSITION_HEIGHT,
        }),
        ..Style::default()
    });
    let snapshot = root_snapshot(1, vec![node]);
    let decoded = Snapshot::decode(&snapshot.encode().unwrap()).unwrap();
    assert_eq!(decoded, snapshot);
}

#[test]
fn patches_update_text_and_style_without_rebuilding_unrelated_nodes() {
    let mut raw = Node::new(3, 2, 0, KIND_RAW_TEXT);
    raw.text = Some("0".into());
    let mut store = NodeStore::default();
    store
        .apply_snapshot(root_snapshot(
            1,
            vec![
                Node::new(1, 0, 0, KIND_VIEW),
                Node::new(2, 1, 0, KIND_TEXT),
                raw,
                Node::new(4, 1, 1, KIND_PRESSABLE),
            ],
        ))
        .unwrap();

    store
        .apply_patch(Patch::new(
            7,
            3,
            1,
            2,
            vec![PatchOperation::Update {
                id: 3,
                mask: UPDATE_TEXT,
                style: None,
                text: Some("1".into()),
                listener_id: 0,
                host_properties: None,
                accessibility: None,
                focusable: false,
                selectable: false,
                tooltip: None,
                accepts_pointer_move: false,
            }],
        ))
        .unwrap();
    assert_eq!(store.get(2).unwrap().text_content.as_deref(), Some("1"));
    assert_eq!(
        store.last_patch_stats(),
        PatchStats {
            operation_count: 1,
            affected_nodes: 1,
            affected_parents: 1
        }
    );

    store
        .apply_patch(Patch::new(
            7,
            3,
            2,
            3,
            vec![PatchOperation::Update {
                id: 4,
                mask: UPDATE_STYLE | UPDATE_LISTENER,
                style: Some(Style {
                    width: Some(12.0),
                    ..Style::default()
                }),
                text: None,
                listener_id: 44,
                host_properties: None,
                accessibility: None,
                focusable: false,
                selectable: false,
                tooltip: None,
                accepts_pointer_move: false,
            }],
        ))
        .unwrap();
    assert_eq!(store.get(4).unwrap().listener_id, 44);
    assert_eq!(
        store.get(4).unwrap().style.as_ref().unwrap().width,
        Some(12.0)
    );
    store
        .apply_patch(Patch::new(
            7,
            3,
            3,
            4,
            vec![PatchOperation::Create(Node::new(5, 1, 2, KIND_VIEW))],
        ))
        .unwrap();
    assert_eq!(store.get(5).unwrap().parent_id, 1);
    store
        .apply_patch(Patch::new(
            7,
            3,
            4,
            5,
            vec![PatchOperation::Move {
                id: 5,
                parent_id: 1,
                index: 0,
            }],
        ))
        .unwrap();
    assert_eq!(store.get(5).unwrap().index, 0);
}

#[test]
fn malformed_patch_rolls_back_and_delete_removes_subtree() {
    let mut store = NodeStore::default();
    store
        .apply_snapshot(root_snapshot(
            1,
            vec![
                Node::new(1, 0, 0, KIND_VIEW),
                view_node(2, 1, 0),
                view_node(3, 2, 0),
            ],
        ))
        .unwrap();
    let before = store.clone();
    let invalid = Patch::new(
        7,
        3,
        1,
        2,
        vec![
            PatchOperation::Update {
                id: 2,
                mask: UPDATE_STYLE,
                style: Some(Style {
                    width: Some(4.0),
                    ..Style::default()
                }),
                text: None,
                listener_id: 0,
                host_properties: None,
                accessibility: None,
                focusable: false,
                selectable: false,
                tooltip: None,
                accepts_pointer_move: false,
            },
            PatchOperation::Delete { id: 999 },
        ],
    );
    assert!(store.apply_patch(invalid).is_err());
    assert_eq!(store, before);

    let cycle = Patch::new(
        7,
        3,
        1,
        2,
        vec![PatchOperation::Move {
            id: 2,
            parent_id: 3,
            index: 0,
        }],
    );
    assert!(matches!(
        store.apply_patch(cycle),
        Err(TreeError::PatchCycle { .. })
    ));
    assert_eq!(store, before);

    store
        .apply_patch(Patch::new(
            7,
            3,
            1,
            2,
            vec![PatchOperation::Delete { id: 2 }],
        ))
        .unwrap();
    assert!(store.get(2).is_none());
    assert!(store.get(3).is_none());
    assert_eq!(store.len(), 1);
}

#[test]
fn rejected_patch_restores_created_nodes_and_ancestor_text_caches() {
    let mut raw = Node::new(4, 3, 0, KIND_RAW_TEXT);
    raw.text = Some("before".into());
    let mut store = NodeStore::default();
    store
        .apply_snapshot(root_snapshot(
            1,
            vec![
                Node::new(1, 0, 0, KIND_VIEW),
                Node::new(2, 1, 0, KIND_TEXT),
                Node::new(3, 2, 0, KIND_TEXT),
                raw,
            ],
        ))
        .unwrap();
    let before = store.clone();
    let mut created = Node::new(5, 3, 1, KIND_RAW_TEXT);
    created.text = Some(" appended".into());
    let mut recreated = Node::new(5, 3, 0, KIND_RAW_TEXT);
    recreated.text = Some(" recreated".into());
    let valid_operations = vec![
        PatchOperation::Update {
            id: 4,
            mask: UPDATE_TEXT,
            style: None,
            text: Some("after".into()),
            listener_id: 0,
            host_properties: None,
            accessibility: None,
            focusable: false,
            selectable: false,
            tooltip: None,
            accepts_pointer_move: false,
        },
        PatchOperation::Create(created),
        PatchOperation::Move {
            id: 4,
            parent_id: 2,
            index: 1,
        },
        PatchOperation::Delete { id: 3 },
        PatchOperation::Create(Node::new(3, 2, 1, KIND_TEXT)),
        PatchOperation::Create(recreated),
    ];
    // Each prefix changes a different combination of identities, sibling indexes,
    // and derived text. Rejection must restore the entire published description.
    for length in 1..=valid_operations.len() {
        let mut operations = valid_operations[..length].to_vec();
        operations.push(PatchOperation::Delete { id: 999 });
        assert!(
            store
                .apply_patch(Patch::new(7, 3, 1, 2, operations))
                .is_err()
        );
        assert_eq!(store, before, "rollback after {length} accepted operations");
    }
    store
        .apply_patch(Patch::new(7, 3, 1, 2, valid_operations))
        .unwrap();
    assert_eq!(
        store.get(2).unwrap().text_content.as_deref(),
        Some("after recreated")
    );
    assert_eq!(
        store.get(3).unwrap().text_content.as_deref(),
        Some(" recreated")
    );
}

#[test]
fn patch_stats_scale_with_changed_nodes() {
    let mut nodes = vec![Node::new(1, 0, 0, KIND_VIEW)];
    for id in 2..2002 {
        nodes.push(view_node(id, 1, id - 2));
    }
    let mut store = NodeStore::default();
    store.apply_snapshot(root_snapshot(1, nodes)).unwrap();
    store
        .apply_patch(Patch::new(
            7,
            3,
            1,
            2,
            vec![PatchOperation::Update {
                id: 2001,
                mask: UPDATE_STYLE,
                style: Some(Style {
                    color_rgba: Some(0xff00ffff),
                    ..Style::default()
                }),
                text: None,
                listener_id: 0,
                host_properties: None,
                accessibility: None,
                focusable: false,
                selectable: false,
                tooltip: None,
                accepts_pointer_move: false,
            }],
        ))
        .unwrap();
    let stats = store.last_patch_stats();
    assert_eq!(stats.operation_count, 1);
    assert_eq!(stats.affected_nodes, 1);
    assert_eq!(stats.affected_parents, 0);
}

#[test]
fn image_host_properties_round_trip_and_reject_invalid_sources_or_children() {
    let mut image = Node::new(2, 1, 0, KIND_IMAGE);
    image.host_properties = Some(HostProperties::Image(ImageProperties {
        source: "assets/icon.png".into(),
        object_fit: 2,
        fallback_source: Some("assets/fallback.png".into()),
    }));
    let snapshot = Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), image]);
    assert_eq!(
        Snapshot::decode(&snapshot.encode().unwrap()).unwrap(),
        snapshot
    );
    NodeStore::default().apply_snapshot(snapshot).unwrap();

    let mut inline = Node::new(2, 1, 0, KIND_IMAGE);
    inline.host_properties = Some(HostProperties::Image(ImageProperties {
        source: format!("data:image/svg+xml,{}", "x".repeat(2048)),
        object_fit: 2,
        fallback_source: Some(format!("data:image/svg+xml,{}", "x".repeat(2048))),
    }));
    let inline = Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), inline]);
    let decoded = Snapshot::decode(&inline.encode().unwrap()).unwrap();
    assert_eq!(decoded, inline);
    NodeStore::default().apply_snapshot(decoded).unwrap();

    for properties in [
        ImageProperties {
            source: "x".repeat(crate::protocol::MAX_IMAGE_SOURCE_BYTES + 1),
            object_fit: 2,
            fallback_source: None,
        },
        ImageProperties {
            source: String::new(),
            object_fit: 2,
            fallback_source: None,
        },
        ImageProperties {
            source: "bad\npath".into(),
            object_fit: 2,
            fallback_source: None,
        },
        ImageProperties {
            source: "assets/icon.png".into(),
            object_fit: 6,
            fallback_source: None,
        },
        ImageProperties {
            source: "assets/icon.png".into(),
            object_fit: 2,
            fallback_source: Some("bad\npath".into()),
        },
    ] {
        let mut invalid = Node::new(2, 1, 0, KIND_IMAGE);
        invalid.host_properties = Some(HostProperties::Image(properties));
        assert!(matches!(
            NodeStore::default().apply_snapshot(Snapshot::new(
                7,
                3,
                0,
                1,
                vec![Node::new(1, 0, 0, KIND_VIEW), invalid],
            )),
            Err(TreeError::InvalidProperties { .. })
        ));
    }

    let child = Node::new(3, 2, 0, KIND_TEXT);
    let mut image = Node::new(2, 1, 0, KIND_IMAGE);
    image.host_properties = Some(HostProperties::Image(ImageProperties {
        source: "assets/icon.png".into(),
        object_fit: 2,
        fallback_source: None,
    }));
    assert!(matches!(
        NodeStore::default().apply_snapshot(Snapshot::new(
            7,
            3,
            0,
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), image, child],
        )),
        Err(TreeError::InvalidChild { .. })
    ));
}

#[test]
fn image_layout_listener_is_valid() {
    let mut image = Node::new(2, 1, 0, KIND_IMAGE);
    image.listener_id = 9;
    image.host_properties = Some(HostProperties::Image(ImageProperties {
        source: "assets/icon.png".into(),
        object_fit: 2,
        fallback_source: None,
    }));
    NodeStore::default()
        .apply_snapshot(Snapshot::new(
            7,
            3,
            0,
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), image],
        ))
        .expect("Image onLayout listener");
}

#[test]
fn icon_host_properties_round_trip_and_reject_invalid_names_or_children() {
    let mut icon = Node::new(2, 1, 0, KIND_ICON);
    icon.host_properties = Some(HostProperties::Icon(IconProperties {
        name: "lucide:home".into(),
        size: 16.0,
        color_rgba: Some(0x112233ff),
    }));
    let snapshot = Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), icon]);
    assert_eq!(
        Snapshot::decode(&snapshot.encode().unwrap()).unwrap(),
        snapshot
    );
    NodeStore::default().apply_snapshot(snapshot).unwrap();

    for properties in [
        IconProperties {
            name: String::new(),
            size: 16.0,
            color_rgba: None,
        },
        IconProperties {
            name: "lucide:not-real".into(),
            size: 16.0,
            color_rgba: None,
        },
        IconProperties {
            name: "lucide:home".into(),
            size: 0.0,
            color_rgba: None,
        },
        IconProperties {
            name: "lucide:home".into(),
            size: f32::NAN,
            color_rgba: None,
        },
    ] {
        let mut invalid = Node::new(2, 1, 0, KIND_ICON);
        invalid.host_properties = Some(HostProperties::Icon(properties));
        assert!(matches!(
            NodeStore::default().apply_snapshot(Snapshot::new(
                7,
                3,
                0,
                1,
                vec![Node::new(1, 0, 0, KIND_VIEW), invalid],
            )),
            Err(TreeError::InvalidProperties { .. })
        ));
    }

    let child = Node::new(3, 2, 0, KIND_TEXT);
    let mut icon = Node::new(2, 1, 0, KIND_ICON);
    icon.host_properties = Some(HostProperties::Icon(IconProperties {
        name: "lucide:home".into(),
        size: 16.0,
        color_rgba: None,
    }));
    let error = NodeStore::default()
        .apply_snapshot(Snapshot::new(
            7,
            3,
            0,
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), icon, child],
        ))
        .unwrap_err();
    assert!(matches!(
        error,
        TreeError::InvalidChild {
            reason: "Image and Icon cannot contain children",
            ..
        }
    ));
}

#[test]
fn extension_host_properties_are_neutral_and_require_nonzero_ids() {
    let extension_properties = || {
        HostProperties::Extension(ExtensionProperties {
            provider_id: [0; 16],
            catalog_digest: [0; 32],
            entry_id: 1,
            entry_version: 1,
            fields: vec![ExtensionField {
                id: 1,
                value: ExtensionValue::Bool(true),
            }],
            event_ids: Arc::from(vec![1_u32]),
        })
    };
    let mut extension = Node::new(2, 1, 0, KIND_EXTENSION);
    extension.host_properties = Some(extension_properties());
    NodeStore::default()
        .apply_snapshot(Snapshot::new(
            7,
            3,
            0,
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), extension],
        ))
        .expect("valid Extension properties");

    let mut mismatched = Node::new(2, 1, 0, KIND_EXTENSION);
    mismatched.host_properties = Some(HostProperties::Image(ImageProperties {
        source: "assets/icon.png".into(),
        object_fit: 2,
        fallback_source: None,
    }));
    assert!(matches!(
        NodeStore::default().apply_snapshot(Snapshot::new(
            7,
            3,
            0,
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), mismatched],
        )),
        Err(TreeError::InvalidProperties { .. })
    ));

    for invalid in [
        ExtensionProperties {
            entry_id: 0,
            ..match extension_properties() {
                HostProperties::Extension(properties) => properties,
                _ => unreachable!(),
            }
        },
        ExtensionProperties {
            fields: vec![ExtensionField {
                id: 0,
                value: ExtensionValue::Bool(true),
            }],
            ..match extension_properties() {
                HostProperties::Extension(properties) => properties,
                _ => unreachable!(),
            }
        },
        ExtensionProperties {
            event_ids: Arc::from(vec![0_u32]),
            ..match extension_properties() {
                HostProperties::Extension(properties) => properties,
                _ => unreachable!(),
            }
        },
    ] {
        let mut invalid_node = Node::new(2, 1, 0, KIND_EXTENSION);
        invalid_node.host_properties = Some(HostProperties::Extension(invalid));
        assert!(matches!(
            NodeStore::default().apply_snapshot(Snapshot::new(
                7,
                3,
                0,
                1,
                vec![Node::new(1, 0, 0, KIND_VIEW), invalid_node],
            )),
            Err(TreeError::InvalidProperties { .. })
        ));
    }
}

#[test]
fn style_code_decoders_reject_zero_values() {
    assert!(FlexDirectionCode::try_from(0).is_err());
    assert!(TextAlignCode::try_from(0).is_err());
}

#[test]
fn tree_rejects_invalid_virtual_list_property_patch() {
    let mut list = Node::new(2, 1, 0, KIND_VIRTUAL_LIST);
    list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
        item_count: 10,
        range_start: 0,
        range_end: 2,
        estimated_item_size: 20.0,
        overscan: 1,
    }));
    let mut store = NodeStore::default();
    store
        .apply_snapshot(Snapshot::new(
            7,
            3,
            0,
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), list],
        ))
        .unwrap();
    let invalid = Patch::new(
        7,
        3,
        1,
        2,
        vec![PatchOperation::Update {
            id: 2,
            mask: UPDATE_PROPERTIES,
            style: None,
            text: None,
            listener_id: 0,
            host_properties: Some(HostProperties::VirtualList(VirtualListProperties {
                item_count: 10,
                range_start: 3,
                range_end: 2,
                estimated_item_size: 20.0,
                overscan: 1,
            })),
            accessibility: None,
            focusable: false,
            selectable: false,
            tooltip: None,
            accepts_pointer_move: false,
        }],
    );
    assert!(matches!(
        store.apply_patch(invalid),
        Err(TreeError::InvalidPatchOperation { .. })
    ));
    assert_eq!(store.revision(), 1);
}

#[test]
fn focusable_view_and_pressable_listener_combinations_are_validated() {
    let mut valid = Node::new(2, 1, 0, KIND_VIEW);
    valid.focusable = true;
    valid.listener_id = 9;
    let mut store = NodeStore::default();
    store
        .apply_snapshot(root_snapshot(1, vec![Node::new(1, 0, 0, KIND_VIEW), valid]))
        .unwrap();
    assert!(store.get(2).unwrap().focusable);

    let mut valid_text_listener = Node::new(2, 1, 0, KIND_TEXT);
    valid_text_listener.focusable = true;
    valid_text_listener.listener_id = 9;
    NodeStore::default()
        .apply_snapshot(root_snapshot(
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), valid_text_listener],
        ))
        .expect("interactive Text focus is valid");
    let mut valid_pressable = Node::new(2, 1, 0, KIND_PRESSABLE);
    valid_pressable.focusable = true;
    valid_pressable.listener_id = 9;
    let mut store = NodeStore::default();
    store
        .apply_snapshot(root_snapshot(
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), valid_pressable],
        ))
        .unwrap();
    assert!(store.get(2).unwrap().focusable);
}

#[test]
fn focusable_text_without_press_listener_is_rejected() {
    let mut text = Node::new(2, 1, 0, KIND_TEXT);
    text.focusable = true;
    let error = NodeStore::default()
        .apply_snapshot(root_snapshot(1, vec![Node::new(1, 0, 0, KIND_VIEW), text]))
        .unwrap_err();
    assert!(matches!(
        error,
        TreeError::InvalidProperties {
            reason: "Text nodes may be focusable only when they have an onPress listener",
            ..
        }
    ));
}
#[test]
fn pressable_focusable_patch_is_accepted() {
    let mut pressable = Node::new(2, 1, 0, KIND_PRESSABLE);
    let mut store = NodeStore::default();
    store
        .apply_snapshot(root_snapshot(
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), pressable.clone()],
        ))
        .expect("initial Pressable tree");
    pressable.focusable = true;
    let patch = Patch::new(
        7,
        3,
        1,
        2,
        vec![PatchOperation::Update {
            id: 2,
            mask: UPDATE_FOCUSABLE,
            style: None,
            text: None,
            listener_id: 0,
            host_properties: None,
            accessibility: None,
            focusable: true,
            selectable: false,
            tooltip: None,
            accepts_pointer_move: false,
        }],
    );
    store.apply_patch(patch).expect("Pressable focusable patch");
    assert!(store.get(2).unwrap().focusable);
}

#[test]
fn interaction_patches_validate_retained_capabilities_before_publishing() {
    let mut text = Node::new(2, 1, 0, KIND_TEXT);
    text.listener_id = 2;
    text.focusable = true;
    let mut raw = Node::new(3, 2, 0, KIND_RAW_TEXT);
    raw.text = Some("label".into());
    let mut view = Node::new(4, 1, 1, KIND_VIEW);
    view.listener_id = 4;
    view.accepts_pointer_move = true;
    let mut store = NodeStore::default();
    store
        .apply_snapshot(root_snapshot(
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), text, raw, view],
        ))
        .unwrap();
    let before = store.clone();
    let update = |id, mask, focusable| PatchOperation::Update {
        id,
        mask,
        style: None,
        text: None,
        listener_id: 0,
        host_properties: None,
        accessibility: None,
        focusable,
        selectable: false,
        tooltip: None,
        accepts_pointer_move: false,
    };
    for operation in [
        update(2, UPDATE_LISTENER, false),
        update(4, UPDATE_LISTENER, false),
        update(3, UPDATE_FOCUSABLE, true),
    ] {
        assert!(
            store
                .apply_patch(Patch::new(7, 3, 1, 2, vec![operation]))
                .is_err()
        );
        assert_eq!(store, before);
    }
    store
        .apply_patch(Patch::new(
            7,
            3,
            1,
            2,
            vec![
                update(2, UPDATE_LISTENER | UPDATE_FOCUSABLE, false),
                update(4, UPDATE_LISTENER | UPDATE_POINTER_MOVE, false),
            ],
        ))
        .unwrap();
    assert!(!store.get(2).unwrap().focusable);
    assert!(!store.get(4).unwrap().accepts_pointer_move);
}

#[test]
fn tooltip_snapshot_and_patch_apply_for_view_and_pressable() {
    let mut view = Node::new(2, 1, 0, KIND_VIEW);
    view.tooltip = Some("View hint".to_owned());
    let mut pressable = Node::new(3, 1, 1, KIND_PRESSABLE);
    pressable.tooltip = Some("Press hint".to_owned());
    pressable.host_properties = Some(HostProperties::Drag(DragProperties {
        drag_type: Some("card".to_owned()),
        export_files: None,
        accepts_drag_over: true,
        accepts_drop: true,
    }));
    let mut store = NodeStore::default();
    store
        .apply_snapshot(root_snapshot(
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), view, pressable],
        ))
        .expect("tooltip snapshot applies");
    assert_eq!(store.get(2).unwrap().tooltip.as_deref(), Some("View hint"));
    assert_eq!(store.get(3).unwrap().tooltip.as_deref(), Some("Press hint"));
    assert!(matches!(
        store.get(3).unwrap().host_properties,
        Some(HostProperties::Drag(_))
    ));
    store
        .apply_patch(Patch::new(
            7,
            3,
            1,
            2,
            vec![PatchOperation::Update {
                id: 2,
                mask: UPDATE_TOOLTIP,
                style: None,
                text: None,
                listener_id: 0,
                host_properties: None,
                accessibility: None,
                focusable: false,
                selectable: false,
                tooltip: Some("Updated view hint".to_owned()),
                accepts_pointer_move: false,
            }],
        ))
        .expect("tooltip patch applies");
    assert_eq!(
        store.get(2).unwrap().tooltip.as_deref(),
        Some("Updated view hint")
    );
}
#[test]
fn pointer_move_capability_requires_interactive_listener_and_can_be_cleared() {
    let mut view = Node::new(2, 1, 0, KIND_VIEW);
    view.listener_id = 9;
    view.accepts_pointer_move = true;
    let mut store = NodeStore::default();
    store
        .apply_snapshot(root_snapshot(1, vec![Node::new(1, 0, 0, KIND_VIEW), view]))
        .expect("pointer move capability snapshot");
    assert!(store.get(2).unwrap().accepts_pointer_move);

    let patch = Patch::new(
        7,
        3,
        1,
        2,
        vec![PatchOperation::Update {
            id: 2,
            mask: UPDATE_POINTER_MOVE,
            style: None,
            text: None,
            listener_id: 0,
            host_properties: None,
            accessibility: None,
            focusable: false,
            selectable: false,
            tooltip: None,
            accepts_pointer_move: false,
        }],
    );
    store
        .apply_patch(patch)
        .expect("pointer move capability clear");
    assert!(!store.get(2).unwrap().accepts_pointer_move);

    let mut invalid = Node::new(3, 1, 1, KIND_TEXT);
    invalid.listener_id = 9;
    invalid.accepts_pointer_move = true;
    assert!(matches!(
        NodeStore::default().apply_snapshot(root_snapshot(
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), invalid]
        )),
        Err(TreeError::InvalidProperties { .. })
    ));
}

#[test]
fn extension_listener_survives_property_updates_and_can_be_replaced() {
    let properties = |disabled| {
        HostProperties::Extension(ExtensionProperties {
            provider_id: [0; 16],
            catalog_digest: [0; 32],
            entry_id: 1,
            entry_version: 1,
            fields: vec![ExtensionField {
                id: 1,
                value: ExtensionValue::Bool(disabled),
            }],
            event_ids: Arc::from([1_u32]),
        })
    };
    let mut button = Node::new(2, 1, 0, KIND_EXTENSION);
    button.listener_id = 10;
    button.host_properties = Some(properties(false));
    let mut store = NodeStore::default();
    store
        .apply_snapshot(root_snapshot(1, vec![view_node(1, 0, 0), button]))
        .unwrap();
    for (base, mask, listener, disabled) in [
        (1, UPDATE_PROPERTIES, 0, true),
        (2, UPDATE_PROPERTIES | UPDATE_LISTENER, 11, false),
        (3, UPDATE_LISTENER, 0, false),
    ] {
        store
            .apply_patch(Patch::new(
                7,
                3,
                base,
                base + 1,
                vec![PatchOperation::Update {
                    id: 2,
                    mask,
                    style: None,
                    text: None,
                    listener_id: listener,
                    host_properties: Some(properties(disabled)),
                    accessibility: None,
                    focusable: false,
                    selectable: false,
                    tooltip: None,
                    accepts_pointer_move: false,
                }],
            ))
            .expect(
                "an extension accepted by Snapshot must remain valid when its properties change",
            );
        assert_eq!(
            store.get(2).unwrap().listener_id,
            if base == 1 { 10 } else { listener }
        );
        assert_eq!(
            store.get(2).unwrap().host_properties,
            Some(properties(disabled))
        );
    }
}
