use std::fs;

use solid_gpui::protocol::{
    EVENT_LAYOUT, KeyAction, KeyEvent, UPDATE_FOCUSABLE, UPDATE_SELECTABLE,
};
use solid_gpui::*;

fn full_style() -> Style {
    Style {
        padding_top: Some(0.0),
        padding_right: Some(4.0),
        padding_bottom: Some(8.0),
        padding_left: Some(88.0),
        border_top_width: Some(0.0),
        border_right_width: Some(1.0),
        border_bottom_width: Some(2.0),
        border_left_width: Some(3.0),
        border_top_left_radius: Some(8.0),
        border_top_right_radius: Some(0.0),
        border_bottom_right_radius: Some(0.0),
        border_bottom_left_radius: Some(8.0),
        border_top_color: Some(0xd4688cff),
        border_right_color: Some(0x2c2b33ff),
        border_bottom_color: Some(0x131217ff),
        border_left_color: Some(0xe07b9eff),
        width_percent: None,
        height_percent: None,
        flex_wrap: Some(solid_gpui::protocol::FlexWrapCode::Wrap),
        linear_gradient: Some(solid_gpui::protocol::LinearGradient {
            angle: 180.,
            start_color: 0x13121700,
            start_position: 0.2,
            end_color: 0x131217ff,
            end_position: 1.,
        }),
        width: Some(120.0),
        height: Some(48.0),
        flex_direction: None,
        grid_columns: Some(2),
        grid_rows: Some(3),
        grid_column_span: Some(2),
        grid_row_span: Some(1),
        flex_grow: Some(1.0),
        padding: Some(4.0),
        gap: Some(2.0),
        justify_content: Some(JustifyContentCode::SpaceBetween),
        align_items: Some(AlignItemsCode::Center),
        border_radius: Some(3.0),
        border_width: Some(1.0),
        border_color_rgba: Some(0x1122_3344),
        font_size: Some(14.0),
        font_weight: Some(FontWeightCode::Bold),
        background_rgba: Some(0x2233_44ff),
        color_rgba: Some(0xaabb_ccdd),
        opacity: Some(0.8),
        transition: Some(Transition {
            duration_ms: 100,
            delay_ms: 20,
            easing: Easing::EaseInOut,
            properties: TRANSITION_OPACITY | TRANSITION_WIDTH,
        }),
        overflow: Some(OverflowCode::Hidden),
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
        text_align: Some(TextAlignCode::Center),
        box_shadows: Some(vec![BoxShadow {
            offset_x: 1.0,
            offset_y: -2.0,
            blur_radius: 3.0,
            spread_radius: 0.0,
            color_rgba: 0x0000_0080,
            inset: true,
        }]),
        font_family: Some("Inter".to_owned()),
    }
}

fn accessibility() -> AccessibilityProperties {
    AccessibilityProperties {
        role: 5,
        label: Some("label".to_owned()),
        description: Some("description".to_owned()),
        disabled: false,
        checked: Some(true),
        selected: Some(false),
        value: Some("42".to_owned()),
        expanded: Some(true),
        level: Some(2),
        live: None,
    }
}

fn input_properties() -> TextInputProperties {
    TextInputProperties {
        value: "text".to_owned(),
        placeholder: Some("placeholder".to_owned()),
        multiline: false,
        disabled: false,
        controlled: true,
        ack_edit_seq: 4,
        selection_start: 1,
        selection_end: 3,
        marked_start: Some(1),
        marked_end: Some(2),
        max_length: Some(8),
        selection_reversed: false,
    }
}

fn virtual_list() -> VirtualListProperties {
    VirtualListProperties {
        item_count: 20,
        range_start: 2,
        range_end: 9,
        estimated_item_size: 24.5,
        overscan: 3,
    }
}

fn node(id: u32, parent_id: u32, index: u32, kind: u32) -> Node {
    Node::new(id, parent_id, index, kind)
}

fn snapshot() -> Snapshot {
    let mut root = node(1, 0, 0, KIND_VIEW);
    root.style = Some(full_style());
    root.accessibility = Some(accessibility());

    let mut text = node(2, 1, 0, KIND_TEXT);
    text.style = Some(full_style());
    text.selectable = true;

    let mut raw_text = node(3, 2, 0, KIND_RAW_TEXT);
    raw_text.text = Some("Hello 😀".to_owned());

    let mut pressable = node(4, 1, 1, KIND_PRESSABLE);
    pressable.listener_id = 7;
    pressable.tooltip = Some("Press to open".to_owned());
    pressable.accepts_pointer_move = true;
    pressable.accessibility = Some(accessibility());

    let mut input = node(5, 1, 2, KIND_TEXT_INPUT);
    input.listener_id = 8;
    input.host_properties = Some(HostProperties::TextInput(input_properties()));

    let mut list = node(6, 1, 3, KIND_VIRTUAL_LIST);
    list.host_properties = Some(HostProperties::VirtualList(virtual_list()));

    let mut image = node(7, 1, 4, KIND_IMAGE);
    image.host_properties = Some(HostProperties::Image(ImageProperties {
        source: "assets/😀.png".to_owned(),
        object_fit: 3,
        fallback_source: Some("assets/fallback.png".to_owned()),
    }));

    let mut drag = node(8, 1, 5, KIND_VIEW);
    drag.style = Some(Style {
        width_percent: Some(50.),
        height_percent: Some(100.),
        ..Style::default()
    });
    drag.listener_id = 11;
    drag.host_properties = Some(HostProperties::Drag(DragProperties {
        drag_type: Some("card".to_owned()),
        export_files: Some(vec!["assets/logo.png".to_owned()]),
        accepts_drag_over: true,
        accepts_drop: true,
    }));

    Snapshot::new(
        7,
        3,
        0,
        42,
        vec![root, text, raw_text, pressable, input, list, image, drag],
    )
}

fn patch() -> Patch {
    let created = node(9, 1, 6, KIND_TEXT);

    let updated = PatchOperation::Update {
        id: 4,
        mask: UPDATE_STYLE
            | UPDATE_TEXT
            | UPDATE_LISTENER
            | UPDATE_PROPERTIES
            | UPDATE_ACCESSIBILITY
            | UPDATE_FOCUSABLE
            | UPDATE_SELECTABLE
            | UPDATE_TOOLTIP
            | UPDATE_POINTER_MOVE,
        style: Some(full_style()),
        text: Some("new".to_owned()),
        listener_id: 12,
        host_properties: Some(HostProperties::Drag(DragProperties {
            drag_type: Some("card".to_owned()),
            export_files: Some(vec!["assets/logo.png".to_owned()]),
            accepts_drag_over: true,
            accepts_drop: true,
        })),
        accessibility: Some(accessibility()),
        focusable: true,
        selectable: true,
        tooltip: Some("updated".to_owned()),
        accepts_pointer_move: true,
    };
    let clear_style = PatchOperation::Update {
        id: 4,
        mask: UPDATE_STYLE,
        style: None,
        text: None,
        listener_id: 0,
        host_properties: None,
        accessibility: None,
        focusable: false,
        selectable: false,
        tooltip: None,
        accepts_pointer_move: false,
    };

    Patch::new(
        7,
        3,
        42,
        43,
        vec![
            PatchOperation::Create(created),
            updated,
            clear_style,
            PatchOperation::Move {
                id: 3,
                parent_id: 1,
                index: 0,
            },
            PatchOperation::Delete { id: 8 },
        ],
    )
}

fn command(request_id: u32, node_id: u32, operation: CommandOperation) -> Command {
    Command::new(
        CommandMeta {
            surface_id: 7,
            epoch: 3,
            after_revision: 43,
            request_id,
            node_id,
        },
        operation,
    )
}

fn commands() -> Vec<Command> {
    vec![
        command(101, 4, CommandOperation::Focus),
        command(102, 4, CommandOperation::Blur),
        command(103, 5, CommandOperation::SetSelection { start: 2, end: 4 }),
        command(
            104,
            6,
            CommandOperation::ScrollToIndex {
                index: 9,
                alignment: 0,
            },
        ),
        command(105, 6, CommandOperation::ScrollToEnd),
        command(
            106,
            1,
            CommandOperation::SetTitle {
                title: "title".to_owned(),
            },
        ),
        command(
            107,
            1,
            CommandOperation::ResizeWindow {
                width: 800,
                height: 600,
            },
        ),
        command(108, 1, CommandOperation::ZoomWindow),
        command(109, 1, CommandOperation::ToggleFullscreen),
        command(
            110,
            1,
            CommandOperation::OpenUrl {
                url: "https://example.com/😀".to_owned(),
            },
        ),
        command(111, 1, CommandOperation::FocusNext),
        command(112, 1, CommandOperation::FocusPrev),
        command(113, 1, CommandOperation::GetWindowSize),
        command(114, 4, CommandOperation::GetFocus),
        command(
            115,
            1,
            CommandOperation::ClipboardWrite {
                text: "clipboard".to_owned(),
            },
        ),
        command(116, 1, CommandOperation::ClipboardRead),
        command(
            117,
            1,
            CommandOperation::OpenSurface {
                title: "Child".to_owned(),
                width: 640,
                height: 480,
                options: Some(WindowOpenOptions {
                    kind: Some(1),
                    resizable: Some(false),
                    min_size: Some((320, 240)),
                }),
            },
        ),
        command(
            118,
            1,
            CommandOperation::FileDialogOpen {
                title: "Choose".to_owned(),
                directories: true,
                multiple: true,
            },
        ),
        command(
            119,
            1,
            CommandOperation::FileDialogSave {
                default_name: "report.json".to_owned(),
            },
        ),
        command(
            120,
            1,
            CommandOperation::ShowNotification {
                title: "Done".to_owned(),
                body: "Finished".to_owned(),
                actions: Some(vec![NotificationActionDefinition {
                    id: "open".to_owned(),
                    label: "Open".to_owned(),
                }]),
            },
        ),
        command(
            121,
            1,
            CommandOperation::SetMenus {
                menus: vec![MenuDefinition {
                    title: "File".to_owned(),
                    items: vec![
                        MenuItemDefinition::Action {
                            name: "open".to_owned(),
                            disabled: true,
                            checked: true,
                        },
                        MenuItemDefinition::Separator,
                        MenuItemDefinition::Submenu(MenuDefinition {
                            title: "More".to_owned(),
                            items: vec![MenuItemDefinition::Action {
                                name: "other".to_owned(),
                                disabled: false,
                                checked: false,
                            }],
                        }),
                    ],
                }],
            },
        ),
        command(
            122,
            1,
            CommandOperation::SetKeybindings {
                bindings: vec![KeybindingDefinition {
                    keystrokes: "cmd-shift-p".to_owned(),
                    action_name: "palette.open".to_owned(),
                }],
            },
        ),
        command(
            123,
            1,
            CommandOperation::SetClosePolicy {
                policy: "require-confirmation".to_owned(),
            },
        ),
        command(
            124,
            1,
            CommandOperation::ResolveCloseRequest {
                request_id: 123,
                allow: true,
            },
        ),
        command(
            125,
            1,
            CommandOperation::ReadTextFile {
                path: "/tmp/notes.txt".to_owned(),
            },
        ),
        command(
            126,
            1,
            CommandOperation::WriteTextFile {
                path: "/tmp/notes.txt".to_owned(),
                content: "hello π".to_owned(),
            },
        ),
        command(
            127,
            1,
            CommandOperation::ClipboardWriteImage {
                image: ClipboardImage {
                    format: 1,
                    bytes: vec![0x89, 0x50, 0x4e, 0x47],
                },
            },
        ),
        command(128, 1, CommandOperation::ClipboardReadImage),
        command(
            129,
            1,
            CommandOperation::LoadFont {
                path: "/tmp/Tuffy.ttf".to_owned(),
            },
        ),
        command(130, 1, CommandOperation::MinimizeWindow),
        command(131, 1, CommandOperation::GetWindowBounds),
        command(132, 1, CommandOperation::GetWindowState),
        command(133, 1, CommandOperation::ActivateWindow),
        command(134, 6, CommandOperation::GetScrollOffset),
        command(135, 6, CommandOperation::ScrollToOffset { offset: 37.5 }),
        command(
            136,
            1,
            CommandOperation::InvokeNative {
                module_id: [1; 16],
                module_digest: [2; 32],
                function_id: 1,
                args: vec![3, 4],
            },
        ),
    ]
}

fn event(
    event_type: u32,
    sequence: u32,
    node_id: u32,
    listener_id: u32,
    payload: Option<EventPayload>,
) -> Event {
    let payload = payload.unwrap_or_else(|| {
        match EventKind::try_from(event_type).expect("known event kind") {
            EventKind::Press => EventPayload::Press,
            EventKind::Focus => EventPayload::Focus,
            EventKind::Blur => EventPayload::Blur,
            EventKind::Hover => EventPayload::Hover,
            EventKind::SurfaceClosed => EventPayload::SurfaceClosed,
            kind => panic!("event kind {kind:?} requires a payload"),
        }
    });
    Event::new(
        EventMeta {
            surface_id: 7,
            epoch: 3,
            revision: 42,
            sequence,
            node_id,
            listener_id,
        },
        payload,
    )
}

fn text_data() -> TextInputEvent {
    TextInputEvent {
        text: "hé😀".to_owned(),
        selection_start: 2,
        selection_end: 4,
        marked_start: Some(2),
        marked_end: Some(3),
        edit_seq: 8,
        reversed: true,
    }
}

fn command_value_events() -> Vec<Event> {
    vec![
        event(
            EVENT_COMMAND_RESULT,
            29,
            1,
            0,
            Some(EventPayload::CommandResult(CommandResult {
                request_id: 201,
                command: CommandKind::GetWindowSize,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::Pair((800.0, 600.0))),
            })),
        ),
        event(
            EVENT_COMMAND_RESULT,
            30,
            1,
            0,
            Some(EventPayload::CommandResult(CommandResult {
                request_id: 202,
                command: CommandKind::GetFocus,
                node_id: 2,
                success: true,
                error: None,
                value: Some(CommandValue::Bool(false)),
            })),
        ),
        event(
            EVENT_COMMAND_RESULT,
            31,
            1,
            0,
            Some(EventPayload::CommandResult(CommandResult {
                request_id: 203,
                command: CommandKind::ClipboardRead,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::Text("clipboard".to_owned())),
            })),
        ),
        event(
            EVENT_COMMAND_RESULT,
            32,
            1,
            0,
            Some(EventPayload::CommandResult(CommandResult {
                request_id: 204,
                command: CommandKind::FileDialogOpen,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::Paths(vec![
                    "/tmp/a".to_owned(),
                    "/tmp/b".to_owned(),
                ])),
            })),
        ),
        event(
            EVENT_COMMAND_RESULT,
            33,
            1,
            0,
            Some(EventPayload::CommandResult(CommandResult {
                request_id: 205,
                command: CommandKind::ReadTextFile,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::FileText("contents".to_owned())),
            })),
        ),
        event(
            EVENT_COMMAND_RESULT,
            34,
            1,
            0,
            Some(EventPayload::CommandResult(CommandResult {
                request_id: 206,
                command: CommandKind::ClipboardReadImage,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::Image(ClipboardImage {
                    format: 1,
                    bytes: vec![1, 2, 3],
                })),
            })),
        ),
        event(
            EVENT_COMMAND_RESULT,
            35,
            1,
            0,
            Some(EventPayload::CommandResult(CommandResult {
                request_id: 207,
                command: CommandKind::GetWindowBounds,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::Bounds((1.0, 2.0, 3.0, 4.0))),
            })),
        ),
        event(
            EVENT_COMMAND_RESULT,
            36,
            1,
            0,
            Some(EventPayload::CommandResult(CommandResult {
                request_id: 208,
                command: CommandKind::GetWindowState,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::WindowState((true, false))),
            })),
        ),
        event(
            EVENT_COMMAND_RESULT,
            37,
            6,
            0,
            Some(EventPayload::CommandResult(CommandResult {
                request_id: 209,
                command: CommandKind::GetScrollOffset,
                node_id: 6,
                success: true,
                error: None,
                value: Some(CommandValue::ScrollOffset(37.5)),
            })),
        ),
        event(
            EVENT_COMMAND_RESULT,
            38,
            1,
            0,
            Some(EventPayload::CommandResult(CommandResult {
                request_id: 210,
                command: CommandKind::InvokeNative,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::Bytes(vec![5, 6])),
            })),
        ),
    ]
}

fn events() -> Vec<Event> {
    let text = text_data();
    let mut events = vec![
        event(EVENT_PRESS, 1, 4, 7, None),
        event(
            EVENT_CHANGE,
            2,
            5,
            8,
            Some(EventPayload::TextInputChange(text.clone())),
        ),
        event(
            EVENT_SELECTION,
            3,
            5,
            8,
            Some(EventPayload::TextInputSelection(text.clone())),
        ),
        event(EVENT_FOCUS, 4, 5, 8, None),
        event(
            EVENT_FOCUS,
            5,
            5,
            8,
            Some(EventPayload::FocusTextInput(text.clone())),
        ),
        event(EVENT_BLUR, 6, 5, 8, None),
        event(EVENT_BLUR, 7, 5, 8, Some(EventPayload::BlurTextInput(text))),
        event(
            EVENT_COMMAND_RESULT,
            8,
            1,
            0,
            Some(EventPayload::CommandResult(CommandResult {
                request_id: 117,
                command: CommandKind::OpenSurface,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::Number(9)),
            })),
        ),
        event(
            EVENT_VISIBLE_RANGE,
            9,
            6,
            10,
            Some(EventPayload::VisibleRange { start: 2, end: 9 }),
        ),
        event(
            EVENT_ANIMATION_COMPLETE,
            10,
            4,
            7,
            Some(EventPayload::AnimationComplete { generation: 4 }),
        ),
        event(
            EVENT_KEY,
            11,
            4,
            7,
            Some(EventPayload::Key(KeyEvent {
                key: "Enter".to_owned(),
                modifiers: vec!["ctrl".to_owned(), "shift".to_owned()],
                action: KeyAction::Repeat,
            })),
        ),
        event(
            EVENT_POINTER,
            12,
            4,
            7,
            Some(EventPayload::Pointer(PointerEvent {
                button: POINTER_BUTTON_LEFT,
                modifiers: vec!["cmd".to_owned()],
                action: EVENT_POINTER_DOWN,
                click_count: 2,
                x: 310.5,
                y: 220.25,
            })),
        ),
        event(
            EVENT_POINTER,
            13,
            4,
            7,
            Some(EventPayload::PointerMove(PointerMoveEvent {
                x: 310.5,
                y: 220.25,
                modifiers: vec!["cmd".to_owned(), "shift".to_owned()],
            })),
        ),
        event(EVENT_HOVER, 14, 4, 7, None),
        event(
            EVENT_SCROLL,
            15,
            4,
            7,
            Some(EventPayload::Scroll(ScrollEvent {
                delta_kind: SCROLL_DELTA_PIXELS,
                dx: 3.5,
                dy: -2.25,
                x: 10.0,
                y: 20.5,
                modifiers: vec!["alt".to_owned()],
            })),
        ),
        event(
            EVENT_SUBMIT,
            16,
            4,
            7,
            Some(EventPayload::Submit {
                text: "submitted text".to_owned(),
            }),
        ),
        event(
            EVENT_WINDOW_RESIZE,
            17,
            1,
            0,
            Some(EventPayload::WindowResize {
                width: 800.5,
                height: 600.5,
                scale_factor: 2.0,
            }),
        ),
        event(
            EVENT_WINDOW_ACTIVATION,
            18,
            1,
            0,
            Some(EventPayload::WindowActivation { active: true }),
        ),
        event(EVENT_SURFACE_CLOSED, 19, 0, 0, None),
        event(
            EVENT_ACTION,
            20,
            1,
            0,
            Some(EventPayload::EventAction {
                action: "open".to_owned(),
            }),
        ),
        event(
            EVENT_WINDOW_APPEARANCE,
            21,
            1,
            0,
            Some(EventPayload::WindowAppearance {
                appearance: WindowAppearance::Dark,
            }),
        ),
        event(
            EVENT_LAYOUT,
            22,
            4,
            7,
            Some(EventPayload::Layout {
                x: 12.5,
                y: -3.25,
                width: 100.0,
                height: 48.75,
            }),
        ),
        event(
            EVENT_DRAG,
            23,
            4,
            7,
            Some(EventPayload::DragOver {
                drag_type: "card".to_owned(),
            }),
        ),
        event(
            EVENT_DRAG,
            24,
            4,
            7,
            Some(EventPayload::DragDrop {
                drag_type: "card".to_owned(),
            }),
        ),
        event(
            EVENT_DRAG,
            25,
            4,
            7,
            Some(EventPayload::ExternalFileDrop {
                paths: vec!["/tmp/a.txt".to_owned()],
            }),
        ),
        event(
            EVENT_NOTIFICATION_RESPONSE,
            26,
            1,
            0,
            Some(EventPayload::NotificationResponse(
                NotificationResponseEvent {
                    tag: "solid-gpui:7:120".to_owned(),
                    action_id: Some("open".to_owned()),
                },
            )),
        ),
        event(
            EVENT_POINTER_DOWN_OUTSIDE,
            27,
            4,
            7,
            Some(EventPayload::PointerDownOutside { x: 12.5, y: -3.25 }),
        ),
        event(
            EVENT_CLOSE_REQUESTED,
            28,
            1,
            0,
            Some(EventPayload::CloseRequested { request_id: 123 }),
        ),
    ];
    events.extend(command_value_events());
    events
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "fixtures/protocol/rust_to_ts.hex".to_owned());
    let snapshot = snapshot();
    let patch = patch();
    let commands = commands();
    let events = events();
    assert_eq!(
        commands.len(),
        36,
        "Rust golden producer must cover every command kind"
    );
    assert_eq!(
        events.len(),
        38,
        "Rust golden producer must cover every event form"
    );
    let mut rows = Vec::with_capacity(2 + commands.len() + events.len());
    rows.push(format!(
        "ts-snapshot-full\tsnapshot\t{}",
        hex(&snapshot.encode()?)
    ));
    rows.push(format!(
        "ts-patch-all-operations\tpatch\t{}",
        hex(&patch.encode()?)
    ));
    for command in commands {
        rows.push(format!(
            "ts-command-{}\tcommand\t{}",
            u32::from(command.operation.kind()),
            hex(&command.encode()?)
        ));
    }
    for event in events {
        rows.push(format!(
            "ts-event-{}\tevent\t{}",
            event.meta.sequence,
            hex(&event.encode()?)
        ));
    }
    rows.push(format!(
        "ts-application-ready\tcommand\t{}",
        hex(&Command::new(
            CommandMeta {
                surface_id: 0,
                epoch: 3,
                after_revision: 0,
                request_id: 1,
                node_id: 0
            },
            CommandOperation::ConfigureApplication {
                keep_alive: true,
                quit: false,
                acknowledged_sequence: 0
            },
        )
        .encode()?)
    ));
    rows.push(format!(
        "ts-application-activation\tevent\t{}",
        hex(&Event::new(
            EventMeta {
                surface_id: 0,
                epoch: 3,
                revision: 0,
                sequence: 1,
                node_id: 0,
                listener_id: 0
            },
            EventPayload::ApplicationActivation {
                target_surface_id: 8,
                reason: "open-urls".into(),
                urls: vec!["demo://document/中文".into()]
            },
        )
        .encode()?)
    ));
    rows.sort();
    let output_text = format!(
        "# protocol-golden-v5\n# id\tmessage\tpayload_hex\n{}\n",
        rows.join("\n")
    );
    fs::write(&output, output_text)?;
    Ok(())
}
