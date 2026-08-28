use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use react_gpui::protocol::{KeyAction, UPDATE_FOCUSABLE, UPDATE_TOOLTIP};
use react_gpui::{
    AccessibilityProperties, BoxShadow, COMMAND_BLUR, COMMAND_CLIPBOARD_READ,
    COMMAND_CLIPBOARD_READ_IMAGE, COMMAND_CLIPBOARD_WRITE, COMMAND_CLIPBOARD_WRITE_IMAGE,
    COMMAND_FILE_DIALOG_OPEN, COMMAND_FILE_DIALOG_SAVE, COMMAND_FOCUS, COMMAND_FOCUS_NEXT,
    COMMAND_FOCUS_PREV, COMMAND_GET_FOCUS, COMMAND_GET_WINDOW_SIZE, COMMAND_OPEN_SURFACE,
    COMMAND_OPEN_URL, COMMAND_READ_TEXT_FILE, COMMAND_RESIZE_WINDOW, COMMAND_SCROLL_TO_END,
    COMMAND_SCROLL_TO_INDEX, COMMAND_SET_KEYBINDINGS, COMMAND_SET_MENUS, COMMAND_SET_SELECTION,
    COMMAND_SET_TITLE, COMMAND_SHOW_NOTIFICATION, COMMAND_TOGGLE_FULLSCREEN,
    COMMAND_WRITE_TEXT_FILE, COMMAND_ZOOM_WINDOW, ClipboardImage, Command, CommandResult,
    CommandValue, DragProperties, EVENT_CHANGE, EVENT_POINTER, EVENT_POINTER_UP, Easing, Event,
    HostProperties, ImageProperties, KIND_PRESSABLE, KIND_RAW_TEXT, KIND_TEXT, KIND_TEXT_INPUT,
    KIND_VIEW, KIND_VIRTUAL_LIST, KeybindingDefinition, MenuDefinition, MenuItemDefinition, Node,
    NotificationActionDefinition, PROTOCOL_VERSION, Patch, PatchOperation, SCROLL_DELTA_PIXELS,
    Snapshot, Style, TRANSITION_BACKGROUND_COLOR, TRANSITION_HEIGHT, TRANSITION_OPACITY,
    TRANSITION_WIDTH, TextInputEvent, TextInputProperties, Transition, UPDATE_ACCESSIBILITY,
    UPDATE_LISTENER, UPDATE_PROPERTIES, UPDATE_STYLE, UPDATE_TEXT, VirtualListProperties,
    WindowAppearance, WindowOpenOptions,
};

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("write to String");
    }
    output
}

fn style() -> Style {
    Style {
        width: Some(1.5),
        height: Some(2.25),
        flex_direction: Some(2),
        flex_grow: Some(3.5),
        padding: Some(4.5),
        gap: Some(5.25),
        justify_content: Some(6),
        align_items: Some(5),
        border_radius: Some(7.5),
        border_width: Some(1.25),
        border_color_rgba: Some(0x1234_5678),
        font_size: Some(14.5),
        font_weight: Some(700),
        background_rgba: Some(0x1122_3344),
        color_rgba: Some(0xaabb_ccdd),
        opacity: Some(0.75),
        transition: Some(Transition {
            duration_ms: 250,
            delay_ms: 15,
            easing: Easing::EaseInOut,
            properties: TRANSITION_OPACITY
                | TRANSITION_BACKGROUND_COLOR
                | TRANSITION_WIDTH
                | TRANSITION_HEIGHT,
        }),
        overflow: Some(3),
        line_clamp: Some(4),
        text_overflow: Some(2),
        margin_top: Some(1.5),
        margin_right: Some(2.5),
        margin_bottom: Some(3.5),
        margin_left: Some(4.5),
        font_style: Some(1),
        text_decoration: Some(2),
        line_height: Some(6.5),
        min_width: Some(10.5),
        max_width: Some(20.5),
        min_height: Some(30.5),
        max_height: Some(40.5),
        flex_shrink: Some(0.75),
        align_self: Some(6),
        position: Some(1),
        left: Some(-8.0),
        top: Some(4.0),
        right: Some(12.0),
        bottom: Some(6.0),
        cursor: Some(2),
        text_align: Some(3),
        box_shadows: Some(vec![BoxShadow {
            offset_x: -2.0,
            offset_y: 3.0,
            blur_radius: 4.0,
            spread_radius: 1.0,
            color_rgba: 0x0102_0380,
            inset: true,
        }]),
        font_family: Some("Avenir Next".into()),
    }
}
fn shadow_snapshot(shadows: Vec<BoxShadow>) -> Snapshot {
    let mut root = Node::new(1, 0, 0, KIND_VIEW);
    let mut root_style = style();
    root_style.box_shadows = Some(shadows);
    root.style = Some(root_style);
    Snapshot::new(7, 3, 0, 44, vec![root])
}

fn accessibility() -> AccessibilityProperties {
    AccessibilityProperties {
        role: 5,
        label: Some("golden label".into()),
        description: Some("golden description".into()),
        disabled: false,
        checked: Some(true),
        selected: Some(false),
        value: Some("42".into()),
        expanded: Some(true),
        level: Some(2),
    }
}

fn snapshot() -> Snapshot {
    let mut root = Node::new(1, 0, 0, KIND_VIEW);
    root.style = Some(style());
    root.accessibility = Some(accessibility());
    let mut text = Node::new(2, 1, 0, KIND_TEXT);
    text.text = Some("hello".into());
    text.selectable = true;
    let mut raw = Node::new(3, 2, 0, KIND_RAW_TEXT);
    raw.text = Some("raw 😀".into());
    let mut pressable = Node::new(4, 1, 1, KIND_PRESSABLE);
    pressable.listener_id = 7;
    pressable.accessibility = Some(accessibility());
    pressable.tooltip = Some("Press to open".into());
    let mut drag = Node::new(8, 1, 5, KIND_PRESSABLE);
    drag.listener_id = 11;
    drag.host_properties = Some(HostProperties::Drag(DragProperties {
        drag_type: Some("card".into()),
        export_files: Some(vec!["assets/logo.png".into()]),
        accepts_drag_over: true,
        accepts_drop: true,
    }));
    let mut input = Node::new(5, 1, 2, KIND_TEXT_INPUT);
    input.host_properties = Some(HostProperties::TextInput(TextInputProperties {
        value: "text".into(),
        placeholder: Some("placeholder".into()),
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
    }));
    let mut list = Node::new(6, 1, 3, KIND_VIRTUAL_LIST);
    list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
        item_count: 20,
        range_start: 2,
        range_end: 9,
        estimated_item_size: 24.5,
        overscan: 3,
    }));
    let mut image = Node::new(7, 1, 4, 7);
    image.host_properties = Some(HostProperties::Image(ImageProperties {
        source: "assets/😀.png".into(),
        object_fit: 3,
        fallback_source: Some("assets/avatar-fallback.png".into()),
    }));
    Snapshot::new(
        7,
        3,
        0,
        42,
        vec![root, text, raw, pressable, input, list, image, drag],
    )
}

fn patch() -> Patch {
    let mut created = Node::new(8, 1, 5, KIND_PRESSABLE);
    created.listener_id = 11;
    created.host_properties = Some(HostProperties::Drag(DragProperties {
        drag_type: Some("card".into()),
        export_files: Some(vec!["assets/logo.png".into()]),
        accepts_drag_over: true,
        accepts_drop: true,
    }));
    Patch::new(
        7,
        3,
        42,
        43,
        vec![
            PatchOperation::Create(created),
            PatchOperation::Update {
                id: 4,
                mask: UPDATE_STYLE
                    | UPDATE_TEXT
                    | UPDATE_LISTENER
                    | UPDATE_PROPERTIES
                    | UPDATE_ACCESSIBILITY
                    | UPDATE_FOCUSABLE
                    | UPDATE_TOOLTIP,
                style: Some(style()),
                text: Some("updated".into()),
                listener_id: 12,
                host_properties: Some(HostProperties::Image(ImageProperties {
                    source: "assets/logo.png".into(),
                    object_fit: 2,
                    fallback_source: None,
                })),
                accessibility: Some(accessibility()),
                focusable: true,
                selectable: false,
                tooltip: Some("Updated tooltip".into()),
            },
            PatchOperation::Move {
                id: 4,
                parent_id: 1,
                index: 0,
            },
            PatchOperation::Delete { id: 6 },
        ],
    )
}

fn clipboard_image_command(kind: u32, request_id: u32, image: Option<ClipboardImage>) -> Command {
    Command {
        protocol: PROTOCOL_VERSION,
        message: 4,
        surface_id: 7,
        epoch: 3,
        after_revision: 42,
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
        image,
    }
}

fn command(kind: u32, node_id: u32, payload: Option<(u32, u32)>, title: Option<&str>) -> Command {
    Command {
        protocol: PROTOCOL_VERSION,
        message: 4,
        surface_id: 7,
        epoch: 3,
        after_revision: 42,
        request_id: kind + 100,
        node_id,
        kind,
        payload,
        title: title.map(str::to_owned),
        body: None,
        actions: None,
        menus: None,
        keybindings: None,
        window_options: None,
        image: None,
    }
}
fn text_file_command(kind: u32, request_id: u32, path: &str, content: Option<&str>) -> Command {
    Command {
        protocol: PROTOCOL_VERSION,
        message: 4,
        surface_id: 7,
        epoch: 3,
        after_revision: 42,
        request_id,
        node_id: 1,
        kind,
        payload: None,
        title: Some(path.to_owned()),
        body: content.map(str::to_owned),
        actions: None,
        menus: None,
        keybindings: None,
        window_options: None,
        image: None,
    }
}
fn surface_command(title: &str, width: u32, height: u32) -> Command {
    Command {
        protocol: PROTOCOL_VERSION,
        message: 4,
        surface_id: 7,
        epoch: 3,
        after_revision: 42,
        request_id: 117,
        node_id: 1,
        kind: COMMAND_OPEN_SURFACE,
        payload: Some((width, height)),
        title: Some(title.to_owned()),
        body: None,
        actions: None,
        menus: None,
        keybindings: None,
        window_options: None,
        image: None,
    }
}
fn surface_options_command() -> Command {
    let mut command = surface_command("Inspector", 640, 480);
    command.request_id = 123;
    command.window_options = Some(WindowOpenOptions {
        kind: Some(1),
        resizable: Some(false),
        min_size: Some((320, 240)),
    });
    command
}
fn notification_command(title: &str, body: &str) -> Command {
    Command {
        protocol: PROTOCOL_VERSION,
        message: 4,
        surface_id: 7,
        epoch: 3,
        after_revision: 42,
        request_id: 120,
        node_id: 1,
        kind: COMMAND_SHOW_NOTIFICATION,
        payload: None,
        title: Some(title.to_owned()),
        body: Some(body.to_owned()),
        actions: Some(vec![NotificationActionDefinition {
            id: "open".into(),
            label: "Open".into(),
        }]),
        menus: None,
        keybindings: None,
        window_options: None,
        image: None,
    }
}

fn menus_command() -> Command {
    Command {
        protocol: PROTOCOL_VERSION,
        message: 4,
        surface_id: 7,
        epoch: 3,
        after_revision: 42,
        request_id: 121,
        node_id: 1,
        kind: COMMAND_SET_MENUS,
        payload: None,
        title: None,
        body: None,
        actions: None,
        menus: Some(vec![MenuDefinition {
            title: "File".into(),
            items: vec![
                MenuItemDefinition::Action {
                    name: "open".into(),
                    disabled: true,
                    checked: true,
                },
                MenuItemDefinition::Separator,
                MenuItemDefinition::Submenu(MenuDefinition {
                    title: "More".into(),
                    items: vec![MenuItemDefinition::Action {
                        name: "other".into(),
                        disabled: false,
                        checked: false,
                    }],
                }),
            ],
        }]),
        keybindings: None,
        window_options: None,
        image: None,
    }
}
fn keybindings_command() -> Command {
    Command {
        protocol: PROTOCOL_VERSION,
        message: 4,
        surface_id: 7,
        epoch: 3,
        after_revision: 42,
        request_id: 122,
        node_id: 1,
        kind: COMMAND_SET_KEYBINDINGS,
        payload: None,
        title: None,
        body: None,
        actions: None,
        menus: None,
        keybindings: Some(vec![
            KeybindingDefinition {
                keystrokes: "cmd-shift-p".into(),
                action_name: "palette.open".into(),
            },
            KeybindingDefinition {
                keystrokes: "ctrl-k ctrl-1".into(),
                action_name: "menu.other".into(),
            },
        ]),
        window_options: None,
        image: None,
    }
}

fn emit(rows: &mut Vec<String>, id: &str, kind: &str, bytes: Vec<u8>) {
    rows.push(format!("{id}\t{kind}\t{}", hex(&bytes)));
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("fixtures/protocol/rust_to_ts.hex"));
    let mut rows = Vec::new();
    emit(
        &mut rows,
        "rust-snapshot-all-kinds",
        "snapshot",
        snapshot().encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-snapshot-box-shadow-double",
        "snapshot",
        shadow_snapshot(vec![
            BoxShadow {
                offset_x: -2.0,
                offset_y: 3.0,
                blur_radius: 4.0,
                spread_radius: 0.0,
                color_rgba: 0x1122_3344,
                inset: false,
            },
            BoxShadow {
                offset_x: 0.0,
                offset_y: -1.0,
                blur_radius: 8.0,
                spread_radius: 2.0,
                color_rgba: 0xaabb_ccdd,
                inset: true,
            },
        ])
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-patch-all-operations",
        "patch",
        patch().encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-press",
        "event",
        Event::press(7, 3, 42, 1, 4, 7).encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-hover",
        "event",
        Event::hover(7, 3, 42, 10, 4, 7).encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-focus",
        "event",
        Event::focus(7, 3, 42, 23, 4, 7, true).encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-blur",
        "event",
        Event::focus(7, 3, 42, 24, 4, 7, false).encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-pointer-down-outside",
        "event",
        Event::pointer_down_outside(7, 3, 42, 25, 4, 7, 12.5, -3.25)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-text-unicode",
        "event",
        Event::text_input(
            EVENT_CHANGE,
            7,
            3,
            42,
            2,
            5,
            9,
            TextInputEvent {
                text: "hé😀".into(),
                selection_start: 2,
                selection_end: 4,
                marked_start: Some(2),
                marked_end: Some(3),
                edit_seq: 8,
                reversed: true,
            },
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-key",
        "event",
        Event::key(
            7,
            3,
            42,
            3,
            4,
            7,
            "Enter".into(),
            vec!["ctrl".into(), "shift".into()],
            KeyAction::Repeat,
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-pointer",
        "event",
        Event::pointer(
            EVENT_POINTER,
            7,
            3,
            42,
            4,
            4,
            7,
            5,
            vec!["cmd".into()],
            EVENT_POINTER_UP,
            2,
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-pointer-up-min-click-count",
        "event",
        Event::pointer(
            EVENT_POINTER,
            7,
            3,
            42,
            26,
            4,
            7,
            5,
            vec!["cmd".into()],
            EVENT_POINTER_UP,
            0,
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-scroll",
        "event",
        Event::scroll(
            7,
            3,
            42,
            5,
            1,
            0,
            SCROLL_DELTA_PIXELS,
            3.5,
            -2.25,
            10.0,
            20.5,
            vec!["alt".into()],
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-visible",
        "event",
        Event::visible_range(7, 3, 42, 6, 6, 13, 2, 9)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-animation",
        "event",
        Event::animation_complete(7, 3, 42, 7, 1, 0, 4)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-command-result",
        "event",
        Event::command_result(
            7,
            3,
            42,
            8,
            CommandResult {
                request_id: 109,
                command: COMMAND_OPEN_URL,
                node_id: 1,
                success: false,
                error: Some("rejected".into()),
                value: None,
            },
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-window-resize",
        "event",
        Event::window_resize_with_scale(7, 3, 42, 11, 1, 0, 800.5, 600.5, 2.0)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-window-activation",
        "event",
        Event::window_activation(7, 3, 42, 12, 1, 0, true)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-window-appearance",
        "event",
        Event::window_appearance(7, 3, 42, 16, WindowAppearance::Dark)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-drag-over",
        "event",
        Event::drag_over(7, 3, 42, 18, 8, 11, "card".into())
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-drag-drop",
        "event",
        Event::drag_drop(7, 3, 42, 19, 8, 11, "card".into())
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-drag-external",
        "event",
        Event::external_file_drop(
            7,
            3,
            42,
            20,
            8,
            11,
            vec!["/tmp/a.txt".into(), "/tmp/b".into()],
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-layout",
        "event",
        Event::layout(7, 3, 42, 17, 4, 7, 12.5, -3.25, 100.0, 48.75)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-command-result-size",
        "event",
        Event::command_result(
            7,
            3,
            42,
            13,
            CommandResult {
                request_id: 110,
                command: COMMAND_GET_WINDOW_SIZE,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::Pair((800.5, 600.5))),
            },
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-command-result-focus",
        "event",
        Event::command_result(
            7,
            3,
            42,
            14,
            CommandResult {
                request_id: 111,
                command: COMMAND_GET_FOCUS,
                node_id: 4,
                success: true,
                error: None,
                value: Some(CommandValue::Bool(true)),
            },
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-command-result-clipboard",
        "event",
        Event::command_result(
            7,
            3,
            42,
            15,
            CommandResult {
                request_id: 112,
                command: COMMAND_CLIPBOARD_READ,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::Text("pasted text".into())),
            },
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-submit-text",
        "event",
        Event::submit(7, 3, 42, 16, 5, 9, "submitted text".into())
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-surface-closed",
        "event",
        Event::surface_closed(7, 3, 42, 17).encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-action",
        "event",
        Event::action(7, 3, 42, 21, "open".into()).encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-notification-response",
        "event",
        Event::notification_response(7, 3, 42, 22, "react-gpui:7:120".into(), Some("open".into()))
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-command-result-open-surface",
        "event",
        Event::command_result(
            7,
            3,
            42,
            18,
            CommandResult {
                request_id: 117,
                command: COMMAND_OPEN_SURFACE,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::Number(41.0)),
            },
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-command-result-file-open",
        "event",
        Event::command_result(
            7,
            3,
            42,
            19,
            CommandResult {
                request_id: 118,
                command: COMMAND_FILE_DIALOG_OPEN,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::Paths(vec![
                    "/tmp/a.txt".into(),
                    "/tmp/b.txt".into(),
                ])),
            },
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-command-result-file-save",
        "event",
        Event::command_result(
            7,
            3,
            42,
            20,
            CommandResult {
                request_id: 119,
                command: COMMAND_FILE_DIALOG_SAVE,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::Text("/tmp/report.json".into())),
            },
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-command-result-notification",
        "event",
        Event::command_result(
            7,
            3,
            42,
            22,
            CommandResult {
                request_id: 120,
                command: COMMAND_SHOW_NOTIFICATION,
                node_id: 1,
                success: true,
                error: None,
                value: None,
            },
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-command-result-menus",
        "event",
        Event::command_result(
            7,
            3,
            42,
            23,
            CommandResult {
                request_id: 121,
                command: COMMAND_SET_MENUS,
                node_id: 1,
                success: true,
                error: None,
                value: None,
            },
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-focus",
        "command",
        command(COMMAND_FOCUS, 4, None, None).encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-selection",
        "command",
        command(COMMAND_SET_SELECTION, 5, Some((2, 4)), None)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-scroll-index",
        "command",
        command(COMMAND_SCROLL_TO_INDEX, 6, Some((9, 0)), None)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-scroll-end",
        "command",
        command(COMMAND_SCROLL_TO_END, 6, None, None)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-title",
        "command",
        command(COMMAND_SET_TITLE, 1, None, Some("Golden 😀 title"))
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-resize",
        "command",
        command(COMMAND_RESIZE_WINDOW, 1, Some((800, 600)), None)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-zoom",
        "command",
        command(COMMAND_ZOOM_WINDOW, 1, None, None)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-fullscreen",
        "command",
        command(COMMAND_TOGGLE_FULLSCREEN, 1, None, None)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-url",
        "command",
        command(COMMAND_OPEN_URL, 1, None, Some("https://example.com/😀"))
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-get-window-size",
        "command",
        command(COMMAND_GET_WINDOW_SIZE, 1, None, None)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-clipboard-write-image",
        "command",
        clipboard_image_command(
            COMMAND_CLIPBOARD_WRITE_IMAGE,
            127,
            Some(ClipboardImage {
                format: 1,
                bytes: vec![0x89, 0x50, 0x4e, 0x47],
            }),
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-clipboard-read-image",
        "command",
        clipboard_image_command(COMMAND_CLIPBOARD_READ_IMAGE, 128, None)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-command-result-clipboard-image",
        "event",
        Event::command_result(
            7,
            3,
            42,
            26,
            CommandResult {
                request_id: 128,
                command: COMMAND_CLIPBOARD_READ_IMAGE,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::Image(ClipboardImage {
                    format: 1,
                    bytes: vec![0x89, 0x50, 0x4e, 0x47],
                })),
            },
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-get-focus",
        "command",
        command(COMMAND_GET_FOCUS, 4, None, None).encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-clipboard-write",
        "command",
        command(COMMAND_CLIPBOARD_WRITE, 1, None, Some("clipboard text"))
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-clipboard-read",
        "command",
        command(COMMAND_CLIPBOARD_READ, 1, None, None)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-open-surface",
        "command",
        surface_command("Child", 640, 480).encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-open-surface-options",
        "command",
        surface_options_command().encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-file-dialog-open",
        "command",
        command(COMMAND_FILE_DIALOG_OPEN, 1, Some((1, 1)), Some("Choose"))
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-file-dialog-save",
        "command",
        command(COMMAND_FILE_DIALOG_SAVE, 1, None, Some("report.json"))
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-read-text-file",
        "command",
        text_file_command(COMMAND_READ_TEXT_FILE, 125, "/tmp/notes.txt", None)
            .encode()
            .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-write-text-file",
        "command",
        text_file_command(
            COMMAND_WRITE_TEXT_FILE,
            126,
            "/tmp/notes.txt",
            Some("hello π"),
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-command-result-read-text-file",
        "event",
        Event::command_result(
            7,
            3,
            42,
            24,
            CommandResult {
                request_id: 125,
                command: COMMAND_READ_TEXT_FILE,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::FileText("hello π".into())),
            },
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-event-command-result-write-text-file",
        "event",
        Event::command_result(
            7,
            3,
            42,
            25,
            CommandResult {
                request_id: 126,
                command: COMMAND_WRITE_TEXT_FILE,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::Number(8.0)),
            },
        )
        .encode()
        .unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-notification",
        "command",
        notification_command("Done", "Finished").encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-set-menus",
        "command",
        menus_command().encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-set-keybindings",
        "command",
        keybindings_command().encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-focus-next",
        "command",
        command(COMMAND_FOCUS_NEXT, 1, None, None).encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-focus-prev",
        "command",
        command(COMMAND_FOCUS_PREV, 1, None, None).encode().unwrap(),
    );
    emit(
        &mut rows,
        "rust-command-blur",
        "command",
        command(COMMAND_BLUR, 4, None, None).encode().unwrap(),
    );
    rows.sort();
    let text = format!(
        "# protocol-golden-v1\n# id\tmessage\tpayload_hex\n{}\n",
        rows.join("\n")
    );
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create fixture directory");
    }
    fs::write(output, text).expect("write Rust golden vectors");
}
