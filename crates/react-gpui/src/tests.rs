use std::io::Cursor;
use std::process::Command as ProcessCommand;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::*;
use crate::protocol::{EVENT_KEY, EVENT_KEY_DOWN, KeyAction};

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
            focusable: false,
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
        title: None,
    };
    assert_eq!(
        Command::decode(&command.encode().unwrap()).unwrap(),
        command
    );
    let title = Command {
        kind: COMMAND_SET_TITLE,
        node_id: 1,
        payload: None,
        title: Some("React GPUI".into()),
        ..command.clone()
    };
    assert_eq!(Command::decode(&title.encode().unwrap()).unwrap(), title);
}
#[test]
fn surface_commands_round_trip_and_reject_invalid_arguments() {
    let commands = [
        Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 1,
            node_id: 1,
            kind: COMMAND_RESIZE_WINDOW,
            payload: Some((800, 600)),
            title: None,
        },
        Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 2,
            node_id: 1,
            kind: COMMAND_ZOOM_WINDOW,
            payload: None,
            title: None,
        },
        Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 3,
            node_id: 1,
            kind: COMMAND_TOGGLE_FULLSCREEN,
            payload: None,
            title: None,
        },
        Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 4,
            node_id: 1,
            kind: COMMAND_OPEN_URL,
            payload: None,
            title: Some("https://example.com/docs".into()),
        },
        Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 5,
            node_id: 1,
            kind: COMMAND_CLIPBOARD_WRITE,
            payload: None,
            title: Some("clipboard text".into()),
        },
        Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 6,
            node_id: 1,
            kind: COMMAND_CLIPBOARD_READ,
            payload: None,
            title: None,
        },
    ];
    for command in commands {
        assert_eq!(
            Command::decode(&command.encode().unwrap()).unwrap(),
            command
        );
    }

    for (request_id, kind) in [(5, COMMAND_FOCUS_NEXT), (6, COMMAND_FOCUS_PREV)] {
        let command = Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id,
            node_id: 1,
            kind,
            payload: None,
            title: None,
        };
        assert_eq!(
            Command::decode(&command.encode().unwrap()).unwrap(),
            command
        );
    }
    for command in [
        Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 5,
            node_id: 2,
            kind: COMMAND_RESIZE_WINDOW,
            payload: Some((0, 600)),
            title: None,
        },
        Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 6,
            node_id: 1,
            kind: COMMAND_OPEN_URL,
            payload: None,
            title: Some("file:///tmp/example".into()),
        },
        Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 7,
            node_id: 2,
            kind: COMMAND_ZOOM_WINDOW,
            payload: None,
            title: None,
        },
        Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 8,
            node_id: 1,
            kind: COMMAND_CLIPBOARD_WRITE,
            payload: None,
            title: Some("x".repeat((1 << 20) + 1)),
        },
    ] {
        assert!(matches!(
            Command::decode(&command.encode().unwrap()),
            Err(ProtocolError::InvalidCommandPayload)
        ));
    }
    let unknown = Command {
        protocol: PROTOCOL_VERSION,
        message: COMMAND_MESSAGE,
        surface_id: 7,
        epoch: 3,
        after_revision: 1,
        request_id: 8,
        node_id: 1,
        kind: 99,
        payload: None,
        title: None,
    };
    assert!(matches!(
        Command::decode(&unknown.encode().unwrap()),
        Err(ProtocolError::UnknownCommand(99))
    ));
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

    let mut pointer_listener_on_view = view_node(2, 1, 0);
    pointer_listener_on_view.listener_id = 9;
    let pointer_listener_snapshot = root_snapshot(
        1,
        vec![Node::new(1, 0, 0, KIND_VIEW), pointer_listener_on_view],
    );
    NodeStore::default()
        .apply_snapshot(pointer_listener_snapshot)
        .unwrap();
    let mut invalid_listener = Node::new(2, 1, 0, KIND_TEXT);
    invalid_listener.listener_id = 9;
    let invalid_listener = root_snapshot(1, vec![Node::new(1, 0, 0, KIND_VIEW), invalid_listener]);
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
        Style {
            justify_content: Some(7),
            ..Style::default()
        },
        Style {
            align_items: Some(6),
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
            font_weight: Some(800),
            ..Style::default()
        },
        Style {
            overflow: Some(4),
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
            text_overflow: Some(3),
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
            font_style: Some(2),
            ..Style::default()
        },
        Style {
            text_decoration: Some(3),
            ..Style::default()
        },
        Style {
            align_self: Some(8),
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
        justify_content: Some(4),
        align_items: Some(5),
        border_radius: Some(3.0),
        border_width: Some(2.0),
        border_color_rgba: Some(0x11223344),
        font_size: Some(14.0),
        font_weight: Some(600),
        color_rgba: Some(0xffffffff),
        opacity: None,
        transition: None,
        overflow: Some(3),
        line_clamp: Some(3),
        text_overflow: Some(2),
        margin_top: Some(1.0),
        margin_right: Some(2.0),
        margin_bottom: Some(3.0),
        margin_left: Some(4.0),
        font_style: Some(1),
        text_decoration: Some(2),
        line_height: Some(18.0),
        min_width: Some(4.0),
        max_width: Some(400.0),
        min_height: Some(4.0),
        max_height: Some(200.0),
        flex_shrink: Some(1.0),
        align_self: Some(5),
    });
    NodeStore::default()
        .apply_snapshot(root_snapshot(1, vec![valid]))
        .unwrap();
}

#[test]
fn style_wire_round_trips_layout_border_and_text_fields() {
    let mut node = Node::new(1, 0, 0, KIND_VIEW);
    node.style = Some(Style {
        justify_content: Some(6),
        align_items: Some(4),
        border_radius: Some(8.0),
        border_width: Some(2.0),
        border_color_rgba: Some(0x12345678),
        font_size: Some(16.0),
        font_weight: Some(900),
        overflow: Some(3),
        line_clamp: Some(3),
        text_overflow: Some(2),
        margin_top: Some(1.0),
        margin_right: Some(2.0),
        margin_bottom: Some(3.0),
        margin_left: Some(4.0),
        font_style: Some(1),
        text_decoration: Some(2),
        line_height: Some(18.0),
        min_width: Some(4.0),
        max_width: Some(400.0),
        min_height: Some(4.0),
        max_height: Some(200.0),
        flex_shrink: Some(0.5),
        align_self: Some(5),
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
    for style in [
        Style {
            overflow: Some(4),
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
            text_overflow: Some(3),
            ..Style::default()
        },
        Style {
            margin_top: Some(-1.0),
            ..Style::default()
        },
        Style {
            font_style: Some(2),
            ..Style::default()
        },
        Style {
            text_decoration: Some(3),
            ..Style::default()
        },
        Style {
            transition: Some(Transition {
                duration_ms: 100,
                delay_ms: 0,
                easing: Easing::Linear,
                properties: 16,
            }),
            ..Style::default()
        },
        Style {
            align_self: Some(8),
            ..Style::default()
        },
    ] {
        let mut invalid = Node::new(1, 0, 0, KIND_VIEW);
        invalid.style = Some(style);
        assert!(matches!(
            Snapshot::decode(&root_snapshot(1, vec![invalid]).encode().unwrap()),
            Err(ProtocolError::InvalidStyle)
        ));
    }
}

#[test]
fn pointer_and_hover_events_round_trip_and_reject_invalid_buttons() {
    let pointer = Event::pointer(
        EVENT_POINTER,
        7,
        3,
        1,
        2,
        9,
        11,
        POINTER_BUTTON_BACK,
        vec!["cmd".to_owned(), "shift".to_owned()],
        EVENT_POINTER_UP,
        2,
    );
    assert_eq!(Event::decode(&pointer.encode().unwrap()).unwrap(), pointer);

    let hover = Event::hover(7, 3, 1, 3, 9, 11);
    assert_eq!(Event::decode(&hover.encode().unwrap()).unwrap(), hover);
    let submit = Event::submit(7, 3, 1, 4, 2, 11);
    assert_eq!(Event::decode(&submit.encode().unwrap()).unwrap(), submit);

    let malformed = rmp_serde::to_vec(&(
        3u32,
        2u32,
        7u32,
        3u32,
        1u32,
        4u32,
        9u32,
        11u32,
        EVENT_POINTER,
        Some((6u32, 9u32, Vec::<String>::new(), EVENT_POINTER_DOWN, 1u32)),
    ))
    .unwrap();
    assert!(matches!(
        Event::decode(&malformed),
        Err(ProtocolError::InvalidEventPayload)
    ));
}

#[test]
fn scroll_events_round_trip_pixels_and_lines_and_reject_invalid_payloads() {
    for (delta_kind, dx, dy) in [
        (SCROLL_DELTA_PIXELS, 12.5, -8.0),
        (SCROLL_DELTA_LINES, 2.0, -1.5),
    ] {
        let scroll = Event::scroll(
            7,
            3,
            1,
            5,
            9,
            11,
            delta_kind,
            dx,
            dy,
            42.0,
            24.0,
            vec!["shift".into(), "cmd".into()],
        );
        assert_eq!(Event::decode(&scroll.encode().unwrap()).unwrap(), scroll);
    }

    for (delta_kind, dx, dy) in [
        (99u32, 1.0, 2.0),
        (SCROLL_DELTA_PIXELS, f32::NAN, 2.0),
        (SCROLL_DELTA_LINES, 1.0, f32::INFINITY),
    ] {
        let malformed = rmp_serde::to_vec(&(
            3u32,
            2u32,
            7u32,
            3u32,
            1u32,
            6u32,
            9u32,
            11u32,
            EVENT_SCROLL,
            Some((7u32, delta_kind, dx, dy, 42.0f32, 24.0f32, vec!["shift"])),
        ))
        .unwrap();
        assert!(matches!(
            Event::decode(&malformed),
            Err(ProtocolError::InvalidEventPayload)
        ));
    }
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
fn process_runtime_reports_unexpected_clean_eof_and_exit_status() {
    let mut command = ProcessCommand::new("sh");
    command.args(["-c", "exit 37"]);
    let runtime = ProcessAdapter::spawn(command).unwrap();

    assert_eq!(runtime.recv_commit().unwrap(), None);
    let status = runtime.status();
    assert_eq!(
        status,
        RuntimeStatus::Exited {
            code: Some(37),
            signal: None,
        }
    );
    assert!(status.is_failure());
}

#[test]
fn process_runtime_marks_explicit_shutdown_without_failure() {
    let mut command = ProcessCommand::new("sh");
    command.args(["-c", "sleep 10"]);
    let runtime = ProcessAdapter::spawn(command).unwrap();

    runtime.shutdown().unwrap();

    let status = runtime.status();
    assert_eq!(status, RuntimeStatus::Shutdown);
    assert!(!status.is_failure());
}

#[test]
fn process_event_writer_backpressures_bytes_without_blocking_and_shutdown_joins() {
    let mut command = ProcessCommand::new("sh");
    command.args(["-c", "sleep 5"]);
    let runtime = ProcessAdapter::spawn(command).unwrap();
    let event = Event::text_input(
        EVENT_CHANGE,
        7,
        3,
        1,
        1,
        2,
        3,
        TextInputEvent {
            text: "x".repeat(1024 * 1024),
            selection_start: 0,
            selection_end: 0,
            marked_start: None,
            marked_end: None,
            edit_seq: 0,
        },
    );

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut accepted = 0;
    let error = loop {
        assert!(
            Instant::now() < deadline,
            "event queue did not backpressure"
        );
        match runtime.send_event(&event) {
            Ok(()) => accepted += 1,
            Err(error) => break error,
        }
    };
    assert!(accepted > 0);
    assert!(error.to_string().contains("byte capacity"));

    let shutdown_started = Instant::now();
    runtime.shutdown().unwrap();
    assert!(shutdown_started.elapsed() < Duration::from_secs(1));
}

#[test]
fn concurrent_process_shutdowns_do_not_deadlock_writer_join() {
    let mut command = ProcessCommand::new("sh");
    command.args(["-c", "sleep 5"]);
    let runtime = ProcessAdapter::spawn(command).unwrap();
    let event = Event::text_input(
        EVENT_CHANGE,
        7,
        3,
        1,
        1,
        2,
        3,
        TextInputEvent {
            text: "x".repeat(1024 * 1024),
            selection_start: 0,
            selection_end: 0,
            marked_start: None,
            marked_end: None,
            edit_seq: 0,
        },
    );
    let _ = runtime.send_event(&event);

    let (sender, receiver) = std::sync::mpsc::channel();
    for _ in 0..2 {
        let runtime = Arc::clone(&runtime);
        let sender = sender.clone();
        std::thread::spawn(move || sender.send(runtime.shutdown()).unwrap());
    }
    drop(sender);
    for _ in 0..2 {
        let result = receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("concurrent shutdown timed out");
        assert!(result.is_ok());
    }
}

#[test]
fn process_event_writer_kills_closed_stdin_child_for_reader_eof() {
    let mut command = ProcessCommand::new("sh");
    command.args(["-c", "exec 0<&-; printf '\\000\\000\\000\\000'; sleep 5"]);
    let runtime = ProcessAdapter::spawn(command).unwrap();
    assert_eq!(runtime.recv_commit().unwrap(), Some(Vec::new()));
    let event = Event::press(7, 3, 1, 1, 2, 3);
    let _ = runtime.send_event(&event);

    let (sender, receiver) = std::sync::mpsc::channel();
    let reader_runtime = Arc::clone(&runtime);
    let reader = std::thread::spawn(move || {
        sender
            .send(reader_runtime.recv_commit())
            .expect("send reader result");
    });
    let result = match receiver.recv_timeout(Duration::from_secs(1)) {
        Ok(result) => result,
        Err(error) => {
            let _ = runtime.shutdown();
            let _ = reader.join();
            panic!("writer failure did not close commit reader: {error}");
        }
    };
    reader.join().unwrap();

    assert_eq!(result.unwrap(), None);
    assert!(runtime.event_writer_error().is_some());
    assert_eq!(runtime.status(), RuntimeStatus::Failed);
    let later_error = runtime.send_event(&event).unwrap_err().to_string();
    assert!(later_error.contains("event writer failed"));
    runtime.shutdown().unwrap();
}

#[test]
fn outbound_event_failure_during_shutdown_is_not_fatal() {
    let runtime = InMemoryAdapter::new();
    runtime.close().unwrap();
    let event = Event::press(7, 3, 1, 1, 1, 10);

    assert!(!send_event_or_exit(runtime.as_ref(), "press event", &event));
}

#[test]
fn outbound_event_metadata_includes_wire_identity_without_payload() {
    let event = Event::text_input(
        EVENT_CHANGE,
        7,
        3,
        11,
        19,
        23,
        29,
        TextInputEvent {
            text: "secret input".to_owned(),
            selection_start: 2,
            selection_end: 4,
            marked_start: Some(1),
            marked_end: Some(3),
            edit_seq: 5,
        },
    );

    assert_eq!(
        crate::transport::format_event_metadata(&event),
        "event_type=2, surface_id=7, epoch=3, revision=11, sequence=19, node_id=23, listener_id=29"
    );
    assert!(!crate::transport::format_event_metadata(&event).contains("secret input"));
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
                focusable: false,
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
        max_length: Some(5),
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
        Event::window_resize(7, 3, 1, 5, 1, 0, 640.0, 480.0),
        Event::window_activation(7, 3, 1, 6, 1, 0, true),
        Event::submit(7, 3, 1, 7, 5, 12),
        Event::submit_with_text(7, 3, 1, 8, 5, 12, "submitted text".into()),
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
                value: None,
            },
        ),
    ] {
        assert_eq!(Event::decode(&event.encode().unwrap()).unwrap(), event);
    }
}
#[test]
fn window_resize_wire_accepts_integer_dimensions() {
    let payload = rmp_serde::to_vec(&(
        3u32,
        2u32,
        7u32,
        3u32,
        1u32,
        1u32,
        1u32,
        0u32,
        EVENT_WINDOW_RESIZE,
        Some((800u32, 600u32)),
    ))
    .unwrap();
    let event = Event::decode(&payload).unwrap();
    assert_eq!(
        event.payload,
        Some(EventPayload::WindowResize {
            width: 800.0,
            height: 600.0,
        })
    );
}
#[test]
fn window_activation_wire_rejects_non_boolean_payloads() {
    let payload = rmp_serde::to_vec(&(
        3u32,
        2u32,
        7u32,
        3u32,
        1u32,
        1u32,
        1u32,
        0u32,
        EVENT_WINDOW_ACTIVATION,
        Some((0u32, 1u32)),
    ))
    .unwrap();
    assert!(matches!(
        Event::decode(&payload),
        Err(ProtocolError::Decode(_))
    ));
}

#[test]
fn command_result_accepts_surface_command_kinds() {
    for command in [
        COMMAND_SET_TITLE,
        COMMAND_RESIZE_WINDOW,
        COMMAND_ZOOM_WINDOW,
        COMMAND_TOGGLE_FULLSCREEN,
        COMMAND_OPEN_URL,
        COMMAND_FOCUS_NEXT,
        COMMAND_FOCUS_PREV,
        COMMAND_GET_WINDOW_SIZE,
        COMMAND_GET_FOCUS,
    ] {
        let event = Event::command_result(
            7,
            3,
            1,
            command,
            CommandResult {
                request_id: 5,
                command,
                node_id: 1,
                success: true,
                error: None,
                value: None,
            },
        );
        assert_eq!(Event::decode(&event.encode().unwrap()).unwrap(), event);
    }
    for (command, value) in [
        (COMMAND_GET_WINDOW_SIZE, CommandValue::Pair((640.0, 480.0))),
        (COMMAND_GET_FOCUS, CommandValue::Bool(true)),
        (
            COMMAND_CLIPBOARD_READ,
            CommandValue::Text("clipboard text".to_owned()),
        ),
    ] {
        let event = Event::command_result(
            7,
            3,
            1,
            command,
            CommandResult {
                request_id: 6,
                command,
                node_id: 1,
                success: true,
                error: None,
                value: Some(value),
            },
        );
        assert_eq!(Event::decode(&event.encode().unwrap()).unwrap(), event);
    }
    let old_wire = rmp_serde::to_vec(&(
        3u32,
        2u32,
        7u32,
        3u32,
        1u32,
        7u32,
        1u32,
        0u32,
        6u32,
        Some((
            2u32,
            77u32,
            COMMAND_FOCUS,
            1u32,
            true,
            Option::<String>::None,
        )),
    ))
    .unwrap();
    let decoded = Event::decode(&old_wire).unwrap();
    match decoded.payload {
        Some(EventPayload::CommandResult(result)) => assert_eq!(result.value, None),
        _ => panic!("old CommandResult wire did not decode"),
    }
}
#[test]
fn image_host_properties_round_trip_and_reject_invalid_sources_or_children() {
    let mut image = Node::new(2, 1, 0, KIND_IMAGE);
    image.host_properties = Some(HostProperties::Image(ImageProperties {
        source: "assets/icon.png".into(),
        object_fit: 2,
    }));
    let snapshot = Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), image]);
    assert_eq!(
        Snapshot::decode(&snapshot.encode().unwrap()).unwrap(),
        snapshot
    );
    NodeStore::default().apply_snapshot(snapshot).unwrap();

    for properties in [
        ImageProperties {
            source: String::new(),
            object_fit: 2,
        },
        ImageProperties {
            source: "bad\npath".into(),
            object_fit: 2,
        },
        ImageProperties {
            source: "assets/icon.png".into(),
            object_fit: 6,
        },
    ] {
        let mut invalid = Node::new(2, 1, 0, KIND_IMAGE);
        invalid.host_properties = Some(HostProperties::Image(properties));
        assert!(matches!(
            Snapshot::decode(
                &Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), invalid])
                    .encode()
                    .unwrap()
            ),
            Err(ProtocolError::InvalidHostProperties)
        ));
    }

    let mut child = Node::new(3, 2, 0, KIND_TEXT);
    child.text = None;
    let mut image = Node::new(2, 1, 0, KIND_IMAGE);
    image.host_properties = Some(HostProperties::Image(ImageProperties {
        source: "assets/icon.png".into(),
        object_fit: 2,
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
        max_length: None,
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
        Err(ProtocolError::Decode(_))
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
    for event_type in [EVENT_PRESS, EVENT_HOVER] {
        let malformed_null_payload = rmp_serde::to_vec(&(
            3u32,
            2u32,
            7u32,
            3u32,
            1u32,
            3u32,
            1u32,
            0u32,
            event_type,
            Some("unexpected"),
        ))
        .unwrap();
        assert!(matches!(
            Event::decode(&malformed_null_payload),
            Err(ProtocolError::Decode(_))
        ));
    }
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
        }],
    );
    assert!(matches!(
        store.apply_patch(invalid),
        Err(TreeError::InvalidPatchOperation { .. })
    ));
    assert_eq!(store.revision(), 1);
}

#[test]
fn key_event_payload_tag_round_trips_with_compact_action_and_modifiers() {
    let event = Event::key(
        7,
        3,
        1,
        8,
        2,
        44,
        "ArrowLeft".into(),
        vec!["shift".into(), "cmd".into()],
        KeyAction::Repeat,
    );
    let payload = event.encode().unwrap();
    assert_eq!(Event::decode(&payload).unwrap(), event);

    let malformed = rmp_serde::to_vec(&(
        3u32,
        2u32,
        7u32,
        3u32,
        1u32,
        9u32,
        2u32,
        44u32,
        EVENT_KEY,
        Some((5u32, "A", vec!["shift", "shift"], EVENT_KEY_DOWN)),
    ))
    .unwrap();
    assert!(matches!(
        Event::decode(&malformed),
        Err(ProtocolError::InvalidEventPayload)
    ));
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

    let mut invalid_listener = Node::new(2, 1, 0, KIND_TEXT);
    invalid_listener.listener_id = 9;
    assert!(matches!(
        NodeStore::default().apply_snapshot(root_snapshot(
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), invalid_listener]
        )),
        Err(TreeError::InvalidListener { .. })
    ));

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
