use super::support::*;

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
            selectable: false,
            tooltip: None,
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
            reversed: true,
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
        body: None,
        actions: None,
        menus: None,
        keybindings: None,
        window_options: None,
        image: None,
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
        body: None,
        actions: None,
        menus: None,
        keybindings: None,
        window_options: None,
        image: None,
        ..command.clone()
    };
    assert_eq!(Command::decode(&title.encode().unwrap()).unwrap(), title);
}

#[test]
fn clipboard_image_commands_and_values_round_trip_with_binary_bytes() {
    let image = ClipboardImage {
        format: 2,
        bytes: vec![0, 1, 2, 0xff],
    };
    let command = Command {
        protocol: PROTOCOL_VERSION,
        message: COMMAND_MESSAGE,
        surface_id: 7,
        epoch: 3,
        after_revision: 2,
        request_id: 27,
        node_id: 1,
        kind: COMMAND_CLIPBOARD_WRITE_IMAGE,
        payload: None,
        title: None,
        body: None,
        actions: None,
        menus: None,
        keybindings: None,
        window_options: None,
        image: Some(image.clone()),
    };
    assert_eq!(
        Command::decode(&command.encode().unwrap()).unwrap(),
        command
    );
    let read = Command {
        kind: COMMAND_CLIPBOARD_READ_IMAGE,
        request_id: 28,
        image: None,
        ..command
    };
    assert_eq!(Command::decode(&read.encode().unwrap()).unwrap(), read);
    let event = Event::command_result(
        7,
        3,
        2,
        1,
        CommandResult {
            request_id: 28,
            command: COMMAND_CLIPBOARD_READ_IMAGE,
            node_id: 1,
            success: true,
            error: None,
            value: Some(CommandValue::Image(image)),
        },
    );
    assert_eq!(Event::decode(&event.encode().unwrap()).unwrap(), event);
}

#[test]
fn clipboard_image_wire_rejects_invalid_formats_and_sizes() {
    let base = Command {
        protocol: PROTOCOL_VERSION,
        message: COMMAND_MESSAGE,
        surface_id: 7,
        epoch: 3,
        after_revision: 2,
        request_id: 1,
        node_id: 1,
        kind: COMMAND_CLIPBOARD_WRITE_IMAGE,
        payload: None,
        title: None,
        body: None,
        actions: None,
        menus: None,
        keybindings: None,
        window_options: None,
        image: Some(ClipboardImage {
            format: 9,
            bytes: vec![1],
        }),
    };
    assert!(matches!(
        base.encode(),
        Err(ProtocolError::InvalidCommandPayload)
    ));
    let empty = Command {
        image: Some(ClipboardImage {
            format: 1,
            bytes: vec![],
        }),
        ..base
    };
    assert!(matches!(
        empty.encode(),
        Err(ProtocolError::InvalidCommandPayload)
    ));
}

#[test]
fn protocol_v3_rejects_adjacent_versions_with_actionable_diagnostics() {
    assert_eq!(PROTOCOL_VERSION, 3);
    let snapshot = Snapshot::new(7, 3, 0, 1, vec![]);
    let patch = Patch::new(7, 3, 0, 1, vec![]);
    let command = Command {
        protocol: PROTOCOL_VERSION,
        message: COMMAND_MESSAGE,
        surface_id: 7,
        epoch: 3,
        after_revision: 1,
        request_id: 1,
        node_id: 0,
        kind: COMMAND_BLUR,
        payload: None,
        title: None,
        body: None,
        actions: None,
        menus: None,
        keybindings: None,
        window_options: None,
        image: None,
    };
    let event = Event::press(7, 3, 1, 1, 0, 0);
    let assert_mismatch = |result: Result<(), ProtocolError>, received: u32| {
        let error = result.expect_err("adjacent protocol version must be rejected");
        assert!(matches!(
            &error,
            ProtocolError::UnsupportedProtocol {
                received: actual,
                expected,
            } if *actual == received && *expected == PROTOCOL_VERSION
        ));
        let message = error.to_string();
        assert!(message.contains(&format!("protocol v{received}")));
        assert!(message.contains(&format!("protocol v{PROTOCOL_VERSION}")));
        assert!(message.contains("update the host binary"));
        assert!(message.contains("pin @react-gpui/core to a v3 release"));
    };

    for version in [PROTOCOL_VERSION - 1, PROTOCOL_VERSION + 1] {
        let mut snapshot_payload = snapshot.encode().unwrap();
        snapshot_payload[1] = u8::try_from(version).unwrap();
        assert_mismatch(Snapshot::decode(&snapshot_payload).map(|_| ()), version);

        let mut patch_payload = patch.encode().unwrap();
        patch_payload[1] = u8::try_from(version).unwrap();
        assert_mismatch(Patch::decode(&patch_payload).map(|_| ()), version);

        let mut command_payload = command.encode().unwrap();
        command_payload[1] = u8::try_from(version).unwrap();
        assert_mismatch(Command::decode(&command_payload).map(|_| ()), version);

        let mut event_payload = event.encode().unwrap();
        event_payload[1] = u8::try_from(version).unwrap();
        assert_mismatch(Event::decode(&event_payload).map(|_| ()), version);
    }
}
#[test]
fn generic_focus_and_pointer_down_outside_events_round_trip() {
    let focus = Event::focus(7, 3, 1, 20, 2, 44, true);
    assert_eq!(Event::decode(&focus.encode().unwrap()).unwrap(), focus);
    assert!(focus.payload.is_none());

    let blur = Event::focus(7, 3, 1, 21, 2, 44, false);
    assert_eq!(Event::decode(&blur.encode().unwrap()).unwrap(), blur);
    assert!(blur.payload.is_none());

    let outside = Event::pointer_down_outside(7, 3, 1, 22, 2, 44, 12.5, -3.25);
    assert_eq!(Event::decode(&outside.encode().unwrap()).unwrap(), outside);
    assert!(matches!(
        outside.payload,
        Some(EventPayload::PointerDownOutside { x, y })
            if x == 12.5 && y == -3.25
    ));
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
            body: None,
            actions: None,
            menus: None,
            keybindings: None,
            window_options: None,
            image: None,
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
            body: None,
            actions: None,
            menus: None,
            keybindings: None,
            window_options: None,
            image: None,
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
            body: None,
            actions: None,
            menus: None,
            keybindings: None,
            window_options: None,
            image: None,
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
            body: None,
            actions: None,
            menus: None,
            keybindings: None,
            window_options: None,
            image: None,
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
            body: None,
            actions: None,
            menus: None,
            keybindings: None,
            window_options: None,
            image: None,
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
            body: None,
            actions: None,
            menus: None,
            keybindings: None,
            window_options: None,
            image: None,
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
            body: None,
            actions: None,
            menus: None,
            keybindings: None,
            window_options: None,
            image: None,
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
            body: None,
            actions: None,
            menus: None,
            keybindings: None,
            window_options: None,
            image: None,
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
            body: None,
            actions: None,
            menus: None,
            keybindings: None,
            window_options: None,
            image: None,
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
            body: None,
            actions: None,
            menus: None,
            keybindings: None,
            window_options: None,
            image: None,
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
            body: None,
            actions: None,
            menus: None,
            keybindings: None,
            window_options: None,
            image: None,
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
        body: None,
        actions: None,
        menus: None,
        keybindings: None,
        window_options: None,
        image: None,
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
        selection_reversed: true,
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
        Event::window_appearance(7, 3, 1, 7, WindowAppearance::Dark),
        Event::submit(7, 3, 1, 7, 5, 12, String::new()),
        Event::submit(7, 3, 1, 8, 5, 12, "submitted text".into()),
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
fn command_result_accepts_surface_command_kinds() {
    for command in [
        COMMAND_SET_TITLE,
        COMMAND_RESIZE_WINDOW,
        COMMAND_ZOOM_WINDOW,
        COMMAND_TOGGLE_FULLSCREEN,
        COMMAND_OPEN_URL,
        COMMAND_OPEN_SURFACE,
        COMMAND_FILE_DIALOG_OPEN,
        COMMAND_FILE_DIALOG_SAVE,
        COMMAND_SHOW_NOTIFICATION,
        COMMAND_SET_MENUS,
        COMMAND_FOCUS_NEXT,
        COMMAND_FOCUS_PREV,
        COMMAND_GET_WINDOW_SIZE,
        COMMAND_GET_FOCUS,
        COMMAND_CLIPBOARD_WRITE,
        COMMAND_CLIPBOARD_READ,
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
        (
            COMMAND_FILE_DIALOG_OPEN,
            CommandValue::Paths(vec!["/tmp/a.txt".to_owned(), "/tmp/b.txt".to_owned()]),
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
    let empty_paths = rmp_serde::to_vec(&(
        3u32,
        2u32,
        7u32,
        3u32,
        1u32,
        8u32,
        1u32,
        0u32,
        6u32,
        Some((
            2u32,
            8u32,
            COMMAND_FILE_DIALOG_OPEN,
            1u32,
            true,
            Option::<String>::None,
            Some((5u32, Vec::<String>::new())),
        )),
    ))
    .unwrap();
    assert!(matches!(
        Event::decode(&empty_paths),
        Err(ProtocolError::InvalidEventPayload)
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
        selection_reversed: false,
    }));
    let snapshot = Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), node]);
    assert!(matches!(
        Snapshot::decode(&snapshot.encode().unwrap()),
        Err(ProtocolError::InvalidHostProperties)
    ));
}
