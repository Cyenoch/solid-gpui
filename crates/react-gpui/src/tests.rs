use std::io::Cursor;

use super::*;

fn root_snapshot(revision: u32, nodes: Vec<Node>) -> Snapshot {
    Snapshot::new(7, 3, revision.saturating_sub(1), revision, nodes)
}

fn synthetic_root(revision: u32) -> Snapshot {
    root_snapshot(revision, vec![Node::new(1, 0, 0, KIND_VIEW)])
}

fn view_node(id: u32, parent_id: u32, index: u32) -> Node {
    Node::new(id, parent_id, index, KIND_VIEW)
}

#[test]
fn snapshot_and_event_use_positional_msgpack_and_frame_round_trip() {
    let snapshot = root_snapshot(1, vec![Node::new(1, 0, 0, KIND_VIEW), view_node(2, 1, 0)]);
    let payload = snapshot.encode().unwrap();
    assert_eq!(Snapshot::decode(&payload).unwrap(), snapshot);
    assert_eq!(payload[0] & 0xf0, 0x90); // fixed-array MessagePack prefix

    let event = Event::press(7, 3, 1, 19, 2, 44);
    let event_payload = event.encode().unwrap();
    assert_eq!(Event::decode(&event_payload).unwrap(), event);

    let mut framed = Vec::new();
    write_frame(&mut framed, &payload).unwrap();
    write_frame(&mut framed, &event_payload).unwrap();
    let mut reader = Cursor::new(framed);
    assert_eq!(read_frame(&mut reader).unwrap(), Some(payload.clone()));
    assert_eq!(read_frame(&mut reader).unwrap(), Some(event_payload));
    let mut trailing = payload.clone();
    trailing.push(0xc0);
    assert!(matches!(
        Snapshot::decode(&trailing),
        Err(ProtocolError::TrailingBytes(1))
    ));
    assert_eq!(read_frame(&mut reader).unwrap(), None);
    let patch = Patch::new(
        7,
        3,
        1,
        2,
        vec![PatchOperation::Update {
            id: 2,
            mask: UPDATE_LISTENER,
            style: None,
            text: None,
            listener_id: 44,
            host_properties: None,
            accessibility: None,
        }],
    );
    let text_event = Event::text_input(
        EVENT_CHANGE,
        7,
        3,
        1,
        20,
        2,
        44,
        TextInputEvent {
            text: "你".into(),
            selection_start: 2,
            selection_end: 2,
            marked_start: Some(0),
            marked_end: Some(1),
            edit_seq: 3,
        },
    );
    let text_payload = text_event.encode().unwrap();
    assert_eq!(Event::decode(&text_payload).unwrap(), text_event);
    let patch_payload = patch.encode().unwrap();
    assert_eq!(Patch::decode(&patch_payload).unwrap(), patch);
    let command = Command {
        protocol: PROTOCOL_VERSION,
        message: COMMAND_MESSAGE,
        surface_id: 7,
        epoch: 3,
        after_revision: 2,
        request_id: 9,
        node_id: 4,
        kind: COMMAND_SET_SELECTION,
        payload: Some((2, 3)),
    };
    assert_eq!(
        Command::decode(&command.encode().unwrap()).unwrap(),
        command
    );
}

#[test]
fn frame_reader_rejects_truncation_and_oversize() {
    let mut truncated_header = Cursor::new(vec![1, 2]);
    assert!(matches!(
        read_frame(&mut truncated_header),
        Err(ProtocolError::TruncatedHeader(2))
    ));

    let mut truncated_payload = Cursor::new(vec![3, 0, 0, 0, 1, 2]);
    assert!(matches!(
        read_frame(&mut truncated_payload),
        Err(ProtocolError::TruncatedPayload {
            expected: 3,
            received: 2
        })
    ));

    let too_large = (MAX_FRAME_LENGTH as u32 + 1).to_le_bytes().to_vec();
    let mut oversized = Cursor::new(too_large);
    assert!(matches!(
        read_frame(&mut oversized),
        Err(ProtocolError::FrameTooLarge(_))
    ));
}

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
    });
    let store = {
        let mut store = NodeStore::default();
        store.apply_snapshot(root_snapshot(1, vec![label])).unwrap();
        store
    };
    assert_eq!(
        store.get(1).unwrap().accessibility_id.as_ref(),
        "react-gpui-node-1"
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
    });
    assert!(matches!(
        NodeStore::default().apply_snapshot(root_snapshot(1, vec![invalid])),
        Err(TreeError::InvalidProperties { .. })
    ));
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

    let mut listener_on_view = view_node(2, 1, 0);
    listener_on_view.listener_id = 9;
    let invalid_listener = root_snapshot(1, vec![Node::new(1, 0, 0, KIND_VIEW), listener_on_view]);
    assert!(matches!(
        NodeStore::default().apply_snapshot(invalid_listener),
        Err(TreeError::InvalidListener { .. })
    ));

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
    for style in [
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
            flex_direction: Some(3),
            ..Style::default()
        },
    ] {
        let mut root = Node::new(1, 0, 0, KIND_VIEW);
        root.style = Some(style);
        assert!(matches!(
            NodeStore::default().apply_snapshot(root_snapshot(1, vec![root])),
            Err(TreeError::InvalidStyle { .. })
        ));
    }

    let mut valid = Node::new(1, 0, 0, KIND_VIEW);
    valid.style = Some(Style {
        width: Some(10.0),
        height: Some(20.0),
        flex_direction: Some(2),
        flex_grow: Some(0.5),
        padding: Some(4.0),
        gap: Some(2.0),
        background_rgba: Some(0xff00ffff),
        color_rgba: Some(0xffffffff),
        opacity: None,
        transition: None,
    });
    NodeStore::default()
        .apply_snapshot(root_snapshot(1, vec![valid]))
        .unwrap();
}

#[test]
fn in_memory_runtime_round_trips_framed_commit_and_event() {
    let runtime = InMemoryAdapter::new();
    let snapshot = synthetic_root(1);
    runtime.push_commit(snapshot.encode().unwrap()).unwrap();
    let payload = runtime.recv_commit().unwrap().unwrap();
    assert_eq!(Snapshot::decode(&payload).unwrap(), snapshot);

    let event = Event::press(7, 3, 1, 1, 1, 10);
    runtime.send_event(&event).unwrap();
    assert_eq!(runtime.take_event().unwrap(), Some(event));
    runtime.close().unwrap();
    assert_eq!(runtime.recv_commit().unwrap(), None);
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
            }],
        ))
        .unwrap();
    let stats = store.last_patch_stats();
    assert_eq!(stats.operation_count, 1);
    assert_eq!(stats.affected_nodes, 1);
    assert_eq!(stats.affected_parents, 0);
}

#[test]
fn protocol_v3_host_properties_and_event_payload_tags_round_trip() {
    let mut input = Node::new(2, 1, 0, KIND_TEXT_INPUT);
    input.listener_id = 11;
    input.host_properties = Some(HostProperties::TextInput(TextInputProperties {
        value: "hello".into(),
        placeholder: Some("name".into()),
        multiline: false,
        disabled: false,
        controlled: true,
        ack_edit_seq: 3,
        selection_start: 2,
        selection_end: 2,
        marked_start: None,
        marked_end: None,
    }));
    let mut list = Node::new(3, 1, 1, KIND_VIRTUAL_LIST);
    list.listener_id = 12;
    list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
        item_count: 100_000,
        range_start: 10,
        range_end: 20,
        estimated_item_size: 24.5,
        overscan: 2,
    }));
    let snapshot = root_snapshot(1, vec![Node::new(1, 0, 0, KIND_VIEW), input, list]);
    assert_eq!(
        Snapshot::decode(&snapshot.encode().unwrap()).unwrap(),
        snapshot
    );
    let mut store = NodeStore::default();
    store
        .apply_snapshot(snapshot.clone())
        .expect("VirtualList listener is valid");
    assert_eq!(store.get(3).expect("VirtualList").listener_id, 12);

    for event in [
        Event::visible_range(7, 3, 1, 2, 3, 12, 10, 20),
        Event::animation_complete(7, 3, 1, 3, 3, 12, 9),
        Event::command_result(
            7,
            3,
            1,
            4,
            CommandResult {
                request_id: 5,
                command: COMMAND_SCROLL_TO_INDEX,
                node_id: 3,
                success: true,
                error: None,
            },
        ),
    ] {
        assert_eq!(Event::decode(&event.encode().unwrap()).unwrap(), event);
    }
}
#[test]
fn protocol_v3_rejects_mismatched_host_property_kind_on_decode() {
    let mut node = Node::new(2, 1, 0, KIND_VIEW);
    node.host_properties = Some(HostProperties::TextInput(TextInputProperties {
        value: String::new(),
        placeholder: None,
        multiline: false,
        disabled: false,
        controlled: false,
        ack_edit_seq: 0,
        selection_start: 0,
        selection_end: 0,
        marked_start: None,
        marked_end: None,
    }));
    let snapshot = Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), node]);
    assert!(matches!(
        Snapshot::decode(&snapshot.encode().unwrap()),
        Err(ProtocolError::InvalidHostProperties)
    ));
}

#[test]
fn protocol_v3_rejects_event_payload_tag_mismatches() {
    let malformed = rmp_serde::to_vec(&(
        3u32,
        2u32,
        7u32,
        3u32,
        1u32,
        1u32,
        2u32,
        11u32,
        7u32,
        Some((4u32, 1u32)),
    ))
    .unwrap();
    assert!(matches!(
        Event::decode(&malformed),
        Err(ProtocolError::InvalidEventPayload)
    ));
    let malformed_command = rmp_serde::to_vec(&(
        3u32,
        2u32,
        7u32,
        3u32,
        1u32,
        2u32,
        2u32,
        11u32,
        6u32,
        Some((99u32, 4u32, 2u32, 2u32, true, Option::<String>::None)),
    ))
    .unwrap();
    let decoded = Event::decode(&malformed_command);
    assert!(matches!(decoded, Err(ProtocolError::InvalidEventPayload)));
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
        }],
    );
    assert!(matches!(
        store.apply_patch(invalid),
        Err(TreeError::InvalidPatchOperation { .. })
    ));
    assert_eq!(store.revision(), 1);
}
