use std::io::Cursor;

use crate::protocol::{
    AlignItemsCode, AlignSelfCode, Command, CommandKind, CommandMeta, CommandOperation,
    CommandResult, CommandValue, CursorCode, Event, FlexDirectionCode, FontStyleCode,
    FontWeightCode, HostProperties, JustifyContentCode, Node, OverflowCode, Patch, PatchOperation,
    PositionCode, ProtocolError, Snapshot, Style, TextAlignCode, TextDecorationCode,
    TextInputProperties, TextOverflowCode, WindowOpenOptions,
};

fn message(fields: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(4 + fields.len());
    result.extend_from_slice(&(fields.len() as u32).to_le_bytes());
    result.extend_from_slice(fields);
    result
}
fn union(tag: u8, value: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(5 + value.len());
    result.extend_from_slice(&(value.len() as u32).to_le_bytes());
    result.push(tag);
    result.extend_from_slice(value);
    result
}
fn envelope(body: &[u8]) -> Vec<u8> {
    let fields = [&[1, 5, 0, 0, 0, 2][..], body, &[0][..]].concat();
    message(&fields)
}
fn required_node(include_listener: bool, include_accessibility: bool) -> Vec<u8> {
    let mut fields = Vec::new();
    fields.push(1);
    fields.extend_from_slice(&1u32.to_le_bytes());
    fields.push(2);
    fields.extend_from_slice(&0u32.to_le_bytes());
    fields.push(3);
    fields.extend_from_slice(&0u32.to_le_bytes());
    fields.extend_from_slice(&[4, 1]);
    if include_listener {
        fields.push(7);
        fields.extend_from_slice(&1u32.to_le_bytes());
    }
    if include_accessibility {
        fields.push(9);
        fields.extend_from_slice(&message(&[4, 0]));
    }
    fields.extend_from_slice(&[10, 0, 11, 0, 13, 0, 0]);
    message(&fields)
}
fn snapshot_with_node(node: Vec<u8>) -> Vec<u8> {
    let mut nodes = Vec::new();
    nodes.extend_from_slice(&1u32.to_le_bytes());
    nodes.extend_from_slice(&node);
    let mut fields = Vec::new();
    for (id, value) in [(1, 7), (2, 3), (3, 0), (4, 1)] {
        fields.push(id);
        fields.extend_from_slice(&(value as u32).to_le_bytes());
    }
    fields.push(5);
    fields.extend_from_slice(&nodes);
    fields.push(0);
    envelope(&union(1, &message(&fields)))
}
fn snapshot_with_style_code(field: u8, value: u32) -> Vec<u8> {
    let mut style = vec![field];
    style.extend_from_slice(&value.to_le_bytes());
    style.push(0);
    let mut node = Vec::new();
    node.push(1);
    node.extend_from_slice(&1u32.to_le_bytes());
    node.push(2);
    node.extend_from_slice(&0u32.to_le_bytes());
    node.push(3);
    node.extend_from_slice(&0u32.to_le_bytes());
    node.extend_from_slice(&[4, 1, 5]);
    node.extend_from_slice(&message(&style));
    node.extend_from_slice(&[10, 0, 11, 0, 13, 0, 0]);
    snapshot_with_node(message(&node))
}
fn patch_with_update(
    mask: u32,
    include_text: bool,
    focusable: Option<bool>,
    selectable: Option<bool>,
) -> Vec<u8> {
    let mut update = Vec::new();
    update.push(1);
    update.extend_from_slice(&1u32.to_le_bytes());
    update.push(2);
    update.extend_from_slice(&mask.to_le_bytes());
    if include_text {
        update.push(5);
        update.extend_from_slice(&(1u32).to_le_bytes());
        update.push(b'x');
    }
    if let Some(focusable) = focusable {
        update.extend_from_slice(&[9, u8::from(focusable)]);
    }
    if let Some(selectable) = selectable {
        update.extend_from_slice(&[10, u8::from(selectable)]);
    }
    update.push(0);
    let operation_value = union(2, &message(&update));
    let mut operation = Vec::new();
    operation.push(1);
    operation.extend_from_slice(&operation_value);
    operation.push(0);
    let mut operations = Vec::new();
    operations.extend_from_slice(&1u32.to_le_bytes());
    operations.extend_from_slice(&message(&operation));
    let mut fields = Vec::new();
    for (id, value) in [(1, 7), (2, 3), (3, 1), (4, 2)] {
        fields.push(id);
        fields.extend_from_slice(&(value as u32).to_le_bytes());
    }
    fields.push(5);
    fields.extend_from_slice(&operations);
    fields.push(0);
    envelope(&union(3, &message(&fields)))
}
fn command_with_missing_menu_flags() -> Vec<u8> {
    let mut action = Vec::new();
    action.push(1);
    action.extend_from_slice(&1u32.to_le_bytes());
    action.extend_from_slice(b"x");
    action.push(0);
    let action = message(&action);
    let action_value = union(2, &action);
    let mut item = Vec::new();
    item.push(1);
    item.extend_from_slice(&action_value);
    item.push(0);
    let item = message(&item);
    let mut menu = Vec::new();
    menu.push(1);
    menu.extend_from_slice(&1u32.to_le_bytes());
    menu.extend_from_slice(b"m");
    menu.push(2);
    menu.extend_from_slice(&1u32.to_le_bytes());
    menu.extend_from_slice(&item);
    menu.push(0);
    let menu = message(&menu);
    let mut menus = Vec::new();
    menus.extend_from_slice(&1u32.to_le_bytes());
    menus.extend_from_slice(&menu);
    let mut payload = Vec::new();
    payload.push(1);
    payload.extend_from_slice(&menus);
    payload.push(0);
    let payload = union(8, &message(&payload));
    let mut command = Vec::new();
    for (id, value) in [(1, 7), (2, 3), (3, 1), (4, 1), (5, 1)] {
        command.push(id);
        command.extend_from_slice(&(value as u32).to_le_bytes());
    }
    command.extend_from_slice(&[6, 21, 7]);
    command.extend_from_slice(&payload);
    command.push(0);
    envelope(&union(4, &message(&command)))
}

fn root_node() -> Node {
    let mut node = Node::new(1, 0, 0, 1);
    node.style = Some(Style {
        width: Some(100.0),
        height: Some(40.0),
        flex_direction: Some(FlexDirectionCode::Row),
        flex_grow: Some(1.0),
        padding: Some(2.0),
        gap: Some(3.0),
        justify_content: Some(JustifyContentCode::Center),
        align_items: Some(AlignItemsCode::Stretch),
        border_radius: Some(2.0),
        border_width: Some(1.0),
        border_color_rgba: Some(0x112233ff),
        font_size: Some(14.0),
        font_weight: Some(FontWeightCode::Bold),
        overflow: Some(OverflowCode::Visible),
        line_clamp: Some(2),
        text_overflow: Some(TextOverflowCode::Ellipsis),
        margin_top: Some(1.0),
        margin_right: Some(2.0),
        margin_bottom: Some(3.0),
        margin_left: Some(4.0),
        font_style: Some(FontStyleCode::Italic),
        text_decoration: Some(TextDecorationCode::Underline),
        line_height: Some(18.0),
        min_width: Some(10.0),
        max_width: Some(200.0),
        min_height: Some(10.0),
        max_height: Some(80.0),
        flex_shrink: Some(1.0),
        align_self: Some(AlignSelfCode::Center),
        position: Some(PositionCode::Relative),
        left: Some(-1.0),
        top: Some(2.0),
        right: Some(3.0),
        bottom: Some(4.0),
        cursor: Some(CursorCode::Pointer),
        text_align: Some(TextAlignCode::Left),
        box_shadows: None,
        background_rgba: Some(0x223344ff),
        color_rgba: Some(0xff0000ff),
        opacity: Some(0.8),
        transition: None,
        font_family: Some("Inter".to_owned()),
        padding_left: Some(88.),
        padding_top: Some(0.),
        border_bottom_width: Some(1.),
        border_top_right_radius: Some(0.),
        border_bottom_color: Some(0xd4688cff),
        flex_wrap: Some(crate::protocol::FlexWrapCode::Wrap),
        linear_gradient: Some(crate::protocol::LinearGradient {
            angle: 180.,
            start_color: 0x13121700,
            start_position: 0.2,
            end_color: 0x131217ff,
            end_position: 1.,
        }),
        ..Style::default()
    });
    node
}

#[test]
fn snapshot_and_patch_round_trip_with_presence_and_full_style() {
    let mut input = Node::new(2, 1, 0, 5);
    input.listener_id = 7;
    input.host_properties = Some(HostProperties::TextInput(TextInputProperties {
        value: "value".to_owned(),
        placeholder: Some("hint".to_owned()),
        multiline: false,
        disabled: false,
        controlled: true,
        ack_edit_seq: 4,
        selection_start: 1,
        selection_end: 3,
        marked_start: Some(1),
        marked_end: Some(2),
        max_length: Some(20),
        selection_reversed: false,
    }));
    let snapshot = Snapshot::new(7, 3, 0, 1, vec![root_node(), input]);
    let encoded = snapshot.encode().expect("encode snapshot");
    assert_eq!(
        Snapshot::decode(&encoded).expect("decode snapshot"),
        snapshot
    );

    let patch = Patch::new(
        7,
        3,
        1,
        2,
        vec![
            PatchOperation::Update {
                id: 1,
                mask: crate::protocol::UPDATE_STYLE,
                style: Some(Style::default()),
                text: None,
                listener_id: 0,
                host_properties: None,
                accessibility: None,
                focusable: false,
                selectable: false,
                tooltip: None,
                accepts_pointer_move: false,
            },
            PatchOperation::Update {
                id: 1,
                mask: crate::protocol::UPDATE_STYLE | crate::protocol::UPDATE_TEXT,
                style: None,
                text: None,
                listener_id: 0,
                host_properties: None,
                accessibility: None,
                focusable: false,
                selectable: false,
                tooltip: None,
                accepts_pointer_move: false,
            },
            PatchOperation::Move {
                id: 2,
                parent_id: 1,
                index: 0,
            },
            PatchOperation::Delete { id: 2 },
        ],
    );
    assert_eq!(
        Patch::decode(&patch.encode().expect("encode patch")).expect("decode patch"),
        patch
    );
}

#[test]
fn style_wire_values_are_closed_and_invalid_codes_are_rejected() {
    for flex_direction in [
        FlexDirectionCode::Row,
        FlexDirectionCode::Column,
        FlexDirectionCode::RowReverse,
        FlexDirectionCode::ColumnReverse,
    ] {
        for text_align in [
            TextAlignCode::Left,
            TextAlignCode::Center,
            TextAlignCode::Right,
        ] {
            let mut node = Node::new(2, 1, 0, 2);
            node.style = Some(Style {
                flex_direction: Some(flex_direction),
                text_align: Some(text_align),
                ..Style::default()
            });
            let snapshot = Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, 1), node]);
            let encoded = snapshot.encode().expect("accepted style encodes");
            assert_eq!(
                Snapshot::decode(&encoded).expect("accepted style decodes"),
                snapshot
            );
        }
    }

    for (field, value) in [(3, 0), (40, 0)] {
        assert!(Snapshot::decode(&snapshot_with_style_code(field, value)).is_err());
    }
}

#[test]
fn malformed_style_codes_are_rejected_before_semantic_tree_validation() {
    let error = Snapshot::decode(&snapshot_with_style_code(3, 0))
        .expect_err("invalid style code must not decode")
        .to_string();
    assert!(error.contains("style"));
}

#[test]
fn semantic_command_errors_include_command_path() {
    let command = Command::new(
        CommandMeta {
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 1,
            node_id: 1,
        },
        CommandOperation::ResizeWindow {
            width: 0,
            height: 1,
        },
    );
    let error = command
        .encode()
        .expect_err("invalid window size must fail")
        .to_string();
    assert!(error.contains("body.command(ResizeWindow).windowSize"));
}

#[test]
fn command_variants_keep_u32_and_optional_values() {
    let open = Command::new(
        CommandMeta {
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 9,
            node_id: 1,
        },
        CommandOperation::OpenSurface {
            title: "Inspector".to_owned(),
            width: 640,
            height: 480,
            options: Some(WindowOpenOptions {
                kind: Some(1),
                resizable: Some(false),
                min_size: Some((320, 240)),
            }),
        },
    );
    assert_eq!(
        Command::decode(&open.encode().expect("encode open")).expect("decode open"),
        open
    );

    let write = Command::new(
        CommandMeta {
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 10,
            node_id: 1,
        },
        CommandOperation::WriteTextFile {
            path: "/tmp/file.txt".to_owned(),
            content: "text".to_owned(),
        },
    );
    assert_eq!(
        Command::decode(&write.encode().expect("encode write")).expect("decode write"),
        write
    );

    let offset = Command::new(
        CommandMeta {
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 11,
            node_id: 2,
        },
        CommandOperation::ScrollToOffset { offset: 37.5 },
    );
    assert_eq!(
        Command::decode(&offset.encode().expect("encode offset")).expect("decode offset"),
        offset
    );

    let get = Command::new(
        CommandMeta {
            request_id: 12,
            ..offset.meta
        },
        CommandOperation::GetScrollOffset,
    );
    assert_eq!(
        Command::decode(&get.encode().expect("encode get")).expect("decode get"),
        get
    );
}

#[test]
fn decoder_rejects_missing_semantic_node_and_accessibility_fields() {
    assert!(Snapshot::decode(&snapshot_with_node(required_node(false, false))).is_err());
    assert!(Snapshot::decode(&snapshot_with_node(required_node(true, true))).is_err());
    assert!(Command::decode(&command_with_missing_menu_flags()).is_err());
}

#[test]
fn decoder_rejects_unmasked_text_and_missing_masked_boolean() {
    assert!(
        Patch::decode(&patch_with_update(0, true, Some(false), Some(false))).is_err(),
        "text must not be present when UPDATE_TEXT is clear",
    );
    assert!(
        Patch::decode(&patch_with_update(
            crate::protocol::UPDATE_FOCUSABLE,
            false,
            None,
            Some(false),
        ))
        .is_err(),
        "focusable must be present when UPDATE_FOCUSABLE is set",
    );
}

#[test]
fn patch_update_boolean_presence_follows_mask() {
    let omitted = Patch::decode(&patch_with_update(0, false, None, None))
        .expect("unset booleans are omitted");
    let PatchOperation::Update {
        focusable,
        selectable,
        ..
    } = &omitted.operations[0]
    else {
        panic!("expected update operation");
    };
    assert!(!focusable);
    assert!(!selectable);

    let focusable_false = Patch::decode(&patch_with_update(
        crate::protocol::UPDATE_FOCUSABLE,
        false,
        Some(false),
        None,
    ))
    .expect("masked false focusable is present");
    let PatchOperation::Update {
        focusable,
        selectable,
        ..
    } = &focusable_false.operations[0]
    else {
        panic!("expected update operation");
    };
    assert!(!focusable);
    assert!(!selectable);

    assert!(
        Patch::decode(&patch_with_update(0, false, Some(false), None)).is_err(),
        "focusable presence is invalid when UPDATE_FOCUSABLE is clear",
    );
    assert!(
        Patch::decode(&patch_with_update(0, false, None, Some(false))).is_err(),
        "selectable presence is invalid when UPDATE_SELECTABLE is clear",
    );
}

#[test]
fn command_encoder_rejects_invalid_operation() {
    let command = Command::new(
        CommandMeta {
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 1,
            node_id: 1,
        },
        CommandOperation::FileDialogOpen {
            title: "x".repeat(257),
            directories: true,
            multiple: false,
        },
    );
    assert!(command.encode().is_err());
}

#[test]
fn command_value_number_preserves_u32_ids() {
    let event = Event::command_result(
        7,
        3,
        1,
        1,
        CommandResult {
            request_id: 1,
            command: CommandKind::OpenSurface,
            node_id: 1,
            success: true,
            error: None,
            value: Some(CommandValue::Number(u32::MAX)),
        },
    );
    let encoded = event.encode().expect("encode u32 command result");
    assert_eq!(
        Event::decode(&encoded).expect("decode u32 command result"),
        event
    );
}

#[test]
fn protocol_version_and_structural_limits_are_strict() {
    let event = Event::press(7, 3, 1, 1, 1, 1);
    let mut wrong_version = event.encode().expect("encode event");
    wrong_version[5..9].copy_from_slice(&3u32.to_le_bytes());
    assert!(matches!(
        Event::decode(&wrong_version),
        Err(ProtocolError::UnsupportedProtocol {
            received: 3,
            expected: 5
        })
    ));

    let snapshot_body = message(&[5, 0x20, 0xa1, 0x07, 0]);
    let body = union(1, &snapshot_body);
    let payload = envelope(&body);
    assert!(matches!(
        Snapshot::decode(&payload),
        Err(ProtocolError::BoundedDecode(_))
    ));

    let unknown_body = union(1, &message(&[99, 0]));
    let unknown_payload = envelope(&unknown_body);
    assert!(Snapshot::decode(&unknown_payload).is_err());
}

#[test]
fn encoder_rejects_invalid_file_dialog_payload() {
    let command = Command::new(
        CommandMeta {
            surface_id: 1,
            epoch: 1,
            after_revision: 1,
            request_id: 1,
            node_id: 1,
        },
        CommandOperation::FileDialogOpen {
            title: "x".repeat(257),
            directories: true,
            multiple: false,
        },
    );
    assert!(command.encode().is_err());
}

#[test]
fn frame_reader_and_writer_preserve_bebop_payloads() {
    let payload = Event::press(7, 3, 1, 1, 1, 1).encode().unwrap();
    let mut framed = Vec::new();
    crate::protocol::write_frame(&mut framed, &payload).unwrap();
    let mut reader = Cursor::new(framed);
    assert_eq!(
        crate::protocol::read_frame(&mut reader).unwrap(),
        Some(payload.clone())
    );
    assert_eq!(crate::protocol::read_frame(&mut reader).unwrap(), None);

    let mut truncated = Cursor::new(vec![3, 0, 0, 0, 1]);
    assert!(matches!(
        crate::protocol::read_frame(&mut truncated),
        Err(ProtocolError::TruncatedPayload { .. })
    ));
}

#[test]
fn large_text_input_snapshot_round_trips() {
    let value = "word ".repeat(20_000);
    let mut input = Node::new(2, 1, 0, 5);
    input.listener_id = 1;
    input.host_properties = Some(HostProperties::TextInput(TextInputProperties {
        value: value.clone(),
        placeholder: None,
        multiline: true,
        disabled: false,
        controlled: true,
        ack_edit_seq: 0,
        selection_start: value.encode_utf16().count() as u32,
        selection_end: value.encode_utf16().count() as u32,
        marked_start: None,
        marked_end: None,
        max_length: None,
        selection_reversed: false,
    }));
    let snapshot = Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, 1), input]);
    let payload = snapshot.encode().unwrap();
    assert_eq!(Snapshot::decode(&payload).unwrap(), snapshot);
}

fn native_command_wire(
    module_id: &[u8],
    digest: &[u8],
    function_id: u32,
    args: &[u8],
    node_id: u32,
) -> Vec<u8> {
    let mut payload = Vec::new();
    for (field, bytes) in [(1, module_id), (2, digest)] {
        payload.push(field);
        payload.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        payload.extend_from_slice(bytes);
    }
    payload.push(3);
    payload.extend_from_slice(&function_id.to_le_bytes());
    payload.push(4);
    payload.extend_from_slice(&(args.len() as u32).to_le_bytes());
    payload.extend_from_slice(args);
    payload.push(0);
    let mut command = Vec::new();
    for (field, value) in [(1, 7u32), (2, 3), (3, 1), (4, 1), (5, node_id)] {
        command.push(field);
        command.extend_from_slice(&value.to_le_bytes());
    }
    command.extend_from_slice(&[6, 36, 7]);
    command.extend_from_slice(&union(12, &message(&payload)));
    command.push(0);
    envelope(&union(4, &message(&command)))
}

#[test]
fn native_command_identity_function_node_and_payload_bounds() {
    let max = crate::protocol::MAX_NATIVE_CALL_BYTES;
    for (function_id, args, node_id) in [(1, Vec::new(), 1), (u32::MAX, vec![0xff; max], 23)] {
        let wire = native_command_wire(&[1; 16], &[2; 32], function_id, &args, node_id);
        let command = Command::decode(&wire).unwrap();
        assert_eq!(
            command.operation,
            CommandOperation::InvokeNative {
                module_id: [1; 16],
                module_digest: [2; 32],
                function_id,
                args,
            }
        );
        assert_eq!(command.encode().unwrap(), wire);
    }
    for (id_len, digest_len, function_id, args_len, node_id) in [
        (15, 32, 1, 0, 1),
        (17, 32, 1, 0, 1),
        (16, 31, 1, 0, 1),
        (16, 33, 1, 0, 1),
        (16, 32, 0, 0, 1),
        (16, 32, 1, max + 1, 1),
        (16, 32, 1, 0, 0),
    ] {
        assert!(
            Command::decode(&native_command_wire(
                &vec![1; id_len],
                &vec![2; digest_len],
                function_id,
                &vec![0; args_len],
                node_id,
            ))
            .is_err()
        );
    }
    for (function_id, args_len, node_id) in [(0, 0, 1), (1, max + 1, 1), (1, 0, 0)] {
        let command = Command::new(
            CommandMeta {
                surface_id: 7,
                epoch: 3,
                after_revision: 1,
                request_id: 1,
                node_id,
            },
            CommandOperation::InvokeNative {
                module_id: [1; 16],
                module_digest: [2; 32],
                function_id,
                args: vec![0; args_len],
            },
        );
        assert!(command.encode().is_err());
    }
}

fn native_result_wire(
    command: u32,
    node_id: u32,
    event_node_id: u32,
    success: bool,
    bytes: Option<&[u8]>,
) -> Vec<u8> {
    let mut result = Vec::new();
    for (field, value) in [(1, 1u32), (2, command), (3, node_id)] {
        result.push(field);
        result.extend_from_slice(&value.to_le_bytes());
    }
    result.extend_from_slice(&[4, u8::from(success)]);
    if let Some(bytes) = bytes {
        let mut value = vec![1];
        value.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        value.extend_from_slice(bytes);
        value.push(0);
        result.push(6);
        result.extend_from_slice(&union(11, &message(&value)));
    }
    result.push(0);
    let mut event = Vec::new();
    for (field, value) in [
        (1, 7u32),
        (2, 3),
        (3, 1),
        (4, 1),
        (5, event_node_id),
        (6, 0),
    ] {
        event.push(field);
        event.extend_from_slice(&value.to_le_bytes());
    }
    event.extend_from_slice(&[7, 6, 8]);
    event.extend_from_slice(&union(2, &message(&result)));
    event.push(0);
    envelope(&union(2, &message(&event)))
}

#[test]
fn native_result_bytes_require_matching_successful_invocation() {
    let max = crate::protocol::MAX_NATIVE_CALL_BYTES;
    for (node_id, bytes) in [(1, Vec::new()), (23, vec![0xff; max])] {
        let wire = native_result_wire(36, node_id, node_id, true, Some(&bytes));
        let event = Event::decode(&wire).unwrap();
        assert_eq!(event.encode().unwrap(), wire);
        let crate::protocol::EventPayload::CommandResult(result) = event.payload else {
            panic!("expected command result")
        };
        assert_eq!(result.value, Some(CommandValue::Bytes(bytes)));
    }
    let failure = native_result_wire(36, 1, 1, false, None);
    assert_eq!(Event::decode(&failure).unwrap().encode().unwrap(), failure);
    for wire in [
        native_result_wire(36, 1, 1, true, None),
        native_result_wire(36, 1, 1, false, Some(&[])),
        native_result_wire(1, 1, 1, true, Some(&[])),
        native_result_wire(36, 2, 1, true, Some(&[])),
        native_result_wire(36, 1, 2, true, Some(&[])),
        native_result_wire(36, 1, 1, true, Some(&vec![0; max + 1])),
    ] {
        assert!(Event::decode(&wire).is_err());
    }
    for (command, node_id, success, error, value) in [
        (CommandKind::InvokeNative, 1, true, None, None),
        (
            CommandKind::InvokeNative,
            1,
            true,
            None,
            Some(CommandValue::Text(String::new())),
        ),
        (
            CommandKind::InvokeNative,
            1,
            true,
            Some("error".to_owned()),
            Some(CommandValue::Bytes(vec![])),
        ),
        (
            CommandKind::InvokeNative,
            1,
            false,
            Some("error".to_owned()),
            Some(CommandValue::Bytes(vec![])),
        ),
        (
            CommandKind::InvokeNative,
            0,
            true,
            None,
            Some(CommandValue::Bytes(vec![])),
        ),
        (
            CommandKind::Focus,
            1,
            true,
            None,
            Some(CommandValue::Bytes(vec![])),
        ),
        (
            CommandKind::InvokeNative,
            1,
            true,
            None,
            Some(CommandValue::Bytes(vec![0; max + 1])),
        ),
    ] {
        assert!(
            Event::command_result(
                7,
                3,
                1,
                1,
                CommandResult {
                    request_id: 1,
                    command,
                    node_id,
                    success,
                    error,
                    value,
                }
            )
            .encode()
            .is_err()
        );
    }
}
