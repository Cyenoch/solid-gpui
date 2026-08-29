use std::io::Cursor;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::{Duration, Instant};

use react_gpui::protocol::{
    self, COMMAND_FOCUS, Command, EVENT_KEY, EVENT_KEY_DOWN, Event, KeyAction, MAX_FRAME_LENGTH,
    Node, PROTOCOL_VERSION, Patch, Snapshot, Style, VirtualListProperties,
};
use react_gpui::{KIND_VIEW, KIND_VIRTUAL_LIST, PatchOperation, read_frame, write_frame};

const RANDOM_CASES_PER_SEED: usize = 256;

#[derive(Debug, Clone, Copy)]
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        assert_ne!(seed, 0);
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        value
    }

    fn index(&mut self, upper: usize) -> usize {
        (self.next() as usize) % upper
    }
}

fn valid_snapshot() -> Snapshot {
    Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW)])
}

fn valid_patch() -> Patch {
    Patch::new(7, 3, 1, 2, Vec::<PatchOperation>::new())
}

fn valid_command() -> Command {
    Command {
        protocol: PROTOCOL_VERSION,
        message: protocol::COMMAND_MESSAGE,
        surface_id: 7,
        epoch: 3,
        after_revision: 1,
        request_id: 1,
        node_id: 1,
        kind: COMMAND_FOCUS,
        payload: None,
        title: None,
        body: None,
        actions: None,
        menus: None,
        keybindings: None,
        window_options: None,
        scroll_offset: None,
        image: None,
    }
}
fn command_with(
    kind: u32,
    node_id: u32,
    payload: Option<(u32, u32)>,
    title: Option<&str>,
) -> Command {
    Command {
        kind,
        node_id,
        payload,
        title: title.map(str::to_owned),
        ..valid_command()
    }
}

fn command_seeds() -> Vec<(&'static str, Vec<u8>)> {
    let mut seeds = Vec::new();
    let mut add = |label, command: Command| seeds.push((label, command.encode().unwrap()));
    add(
        "command-focus",
        command_with(protocol::COMMAND_FOCUS, 4, None, None),
    );
    add(
        "command-blur",
        command_with(protocol::COMMAND_BLUR, 4, None, None),
    );
    add(
        "command-selection",
        command_with(protocol::COMMAND_SET_SELECTION, 5, Some((2, 4)), None),
    );
    add(
        "command-scroll-index",
        command_with(protocol::COMMAND_SCROLL_TO_INDEX, 6, Some((9, 0)), None),
    );
    add(
        "command-scroll-end",
        command_with(protocol::COMMAND_SCROLL_TO_END, 6, None, None),
    );
    add(
        "command-title",
        command_with(protocol::COMMAND_SET_TITLE, 1, None, Some("Golden title")),
    );
    add(
        "command-resize",
        command_with(protocol::COMMAND_RESIZE_WINDOW, 1, Some((800, 600)), None),
    );
    add(
        "command-zoom",
        command_with(protocol::COMMAND_ZOOM_WINDOW, 1, None, None),
    );
    add(
        "command-fullscreen",
        command_with(protocol::COMMAND_TOGGLE_FULLSCREEN, 1, None, None),
    );
    add(
        "command-url",
        command_with(
            protocol::COMMAND_OPEN_URL,
            1,
            None,
            Some("https://example.com"),
        ),
    );
    add(
        "command-focus-next",
        command_with(protocol::COMMAND_FOCUS_NEXT, 1, None, None),
    );
    add(
        "command-focus-prev",
        command_with(protocol::COMMAND_FOCUS_PREV, 1, None, None),
    );
    add(
        "command-window-size",
        command_with(protocol::COMMAND_GET_WINDOW_SIZE, 1, None, None),
    );
    add(
        "command-get-focus",
        command_with(protocol::COMMAND_GET_FOCUS, 4, None, None),
    );
    add(
        "command-clipboard-write",
        command_with(
            protocol::COMMAND_CLIPBOARD_WRITE,
            1,
            None,
            Some("clipboard"),
        ),
    );
    add(
        "command-clipboard-read",
        command_with(protocol::COMMAND_CLIPBOARD_READ, 1, None, None),
    );
    add(
        "command-open-surface",
        Command {
            kind: protocol::COMMAND_OPEN_SURFACE,
            node_id: 1,
            payload: Some((640, 480)),
            title: Some("Child".into()),
            ..valid_command()
        },
    );
    add(
        "command-file-dialog-open",
        Command {
            kind: protocol::COMMAND_FILE_DIALOG_OPEN,
            node_id: 1,
            payload: Some((1, 1)),
            title: Some("Choose".into()),
            ..valid_command()
        },
    );
    add(
        "command-file-dialog-save",
        command_with(
            protocol::COMMAND_FILE_DIALOG_SAVE,
            1,
            None,
            Some("report.json"),
        ),
    );
    add(
        "command-notification",
        Command {
            kind: protocol::COMMAND_SHOW_NOTIFICATION,
            node_id: 1,
            title: Some("Done".into()),
            body: Some("Finished".into()),
            actions: Some(vec![protocol::NotificationActionDefinition {
                id: "open".into(),
                label: "Open".into(),
            }]),
            ..valid_command()
        },
    );
    add(
        "command-menus",
        Command {
            kind: protocol::COMMAND_SET_MENUS,
            node_id: 1,
            menus: Some(vec![protocol::MenuDefinition {
                title: "File".into(),
                items: vec![protocol::MenuItemDefinition::Action {
                    name: "open".into(),
                    disabled: false,
                    checked: false,
                }],
            }]),
            ..valid_command()
        },
    );
    add(
        "command-keybindings",
        Command {
            kind: protocol::COMMAND_SET_KEYBINDINGS,
            node_id: 1,
            keybindings: Some(vec![protocol::KeybindingDefinition {
                keystrokes: "cmd-k".into(),
                action_name: "menu.open".into(),
            }]),
            ..valid_command()
        },
    );
    add(
        "command-close-policy",
        command_with(
            protocol::COMMAND_SET_CLOSE_POLICY,
            1,
            None,
            Some("require-confirmation"),
        ),
    );
    add(
        "command-resolve-close",
        command_with(
            protocol::COMMAND_RESOLVE_CLOSE_REQUEST,
            1,
            Some((123, 1)),
            None,
        ),
    );
    add(
        "command-read-file",
        Command {
            kind: protocol::COMMAND_READ_TEXT_FILE,
            node_id: 1,
            title: Some("/tmp/notes.txt".into()),
            ..valid_command()
        },
    );
    add(
        "command-write-file",
        Command {
            kind: protocol::COMMAND_WRITE_TEXT_FILE,
            node_id: 1,
            title: Some("/tmp/notes.txt".into()),
            body: Some("hello".into()),
            ..valid_command()
        },
    );
    add(
        "command-clipboard-write-image",
        Command {
            kind: protocol::COMMAND_CLIPBOARD_WRITE_IMAGE,
            node_id: 1,
            image: Some(protocol::ClipboardImage {
                format: 1,
                bytes: vec![0x89, 0x50, 0x4e, 0x47],
            }),
            ..valid_command()
        },
    );
    add(
        "command-clipboard-read-image",
        command_with(protocol::COMMAND_CLIPBOARD_READ_IMAGE, 1, None, None),
    );
    add(
        "command-load-font",
        Command {
            kind: protocol::COMMAND_LOAD_FONT,
            node_id: 1,
            title: Some("/tmp/Tuffy.ttf".into()),
            ..valid_command()
        },
    );
    add(
        "command-minimize",
        command_with(protocol::COMMAND_MINIMIZE_WINDOW, 1, None, None),
    );
    add(
        "command-window-bounds",
        command_with(protocol::COMMAND_GET_WINDOW_BOUNDS, 1, None, None),
    );
    add(
        "command-window-state",
        command_with(protocol::COMMAND_GET_WINDOW_STATE, 1, None, None),
    );
    add(
        "command-activate",
        command_with(protocol::COMMAND_ACTIVATE_WINDOW, 1, None, None),
    );
    seeds
}

fn event_seeds() -> Vec<(&'static str, Vec<u8>)> {
    let mut seeds = Vec::new();
    let mut add = |label, event: Event| seeds.push((label, event.encode().unwrap()));
    add("event-press", Event::press(7, 3, 42, 1, 4, 7));
    let text_input = protocol::TextInputEvent {
        text: "text".into(),
        selection_start: 1,
        selection_end: 2,
        marked_start: None,
        marked_end: None,
        edit_seq: 3,
        reversed: false,
    };
    add(
        "event-change",
        Event::text_input(
            protocol::EVENT_CHANGE,
            7,
            3,
            42,
            2,
            5,
            9,
            text_input.clone(),
        ),
    );
    add(
        "event-selection",
        Event::text_input(protocol::EVENT_SELECTION, 7, 3, 42, 3, 5, 9, text_input),
    );
    add("event-focus", Event::focus(7, 3, 42, 4, 4, 7, true));
    add("event-blur", Event::focus(7, 3, 42, 5, 4, 7, false));
    add(
        "event-command-result",
        Event::command_result(
            7,
            3,
            42,
            6,
            protocol::CommandResult {
                request_id: 1,
                command: protocol::COMMAND_FOCUS,
                node_id: 4,
                success: true,
                error: None,
                value: None,
            },
        ),
    );
    add(
        "event-visible-range",
        Event::visible_range(7, 3, 42, 7, 6, 13, 2, 9),
    );
    add(
        "event-animation",
        Event::animation_complete(7, 3, 42, 8, 1, 0, 4),
    );
    add(
        "event-key",
        Event::key(
            7,
            3,
            42,
            9,
            4,
            7,
            "Enter".into(),
            vec!["shift".into()],
            KeyAction::Repeat,
        ),
    );
    add(
        "event-pointer-down",
        Event::pointer(
            protocol::EVENT_POINTER,
            7,
            3,
            42,
            10,
            4,
            7,
            protocol::POINTER_BUTTON_LEFT,
            vec![],
            protocol::EVENT_POINTER_DOWN,
            1,
            10.0,
            20.0,
        ),
    );
    add(
        "event-pointer-up",
        Event::pointer(
            protocol::EVENT_POINTER,
            7,
            3,
            42,
            11,
            4,
            7,
            protocol::POINTER_BUTTON_LEFT,
            vec![],
            protocol::EVENT_POINTER_UP,
            1,
            10.0,
            20.0,
        ),
    );
    add(
        "event-pointer-move",
        Event::pointer_move(7, 3, 42, 12, 4, 7, 10.0, 20.0, vec!["shift".into()]),
    );
    add("event-hover", Event::hover(7, 3, 42, 12, 4, 7));
    add(
        "event-scroll",
        Event::scroll(
            7,
            3,
            42,
            13,
            1,
            0,
            protocol::SCROLL_DELTA_PIXELS,
            1.0,
            -2.0,
            3.0,
            4.0,
            vec![],
        ),
    );
    add(
        "event-submit",
        Event::submit(7, 3, 42, 14, 5, 9, "submitted".into()),
    );
    add(
        "event-window-resize",
        Event::window_resize_with_scale(7, 3, 42, 15, 1, 0, 800.0, 600.0, 2.0),
    );
    add(
        "event-window-activation",
        Event::window_activation(7, 3, 42, 16, 1, 0, true),
    );
    add("event-surface-closed", Event::surface_closed(7, 3, 42, 17));
    add("event-action", Event::action(7, 3, 42, 18, "open".into()));
    add(
        "event-appearance",
        Event::window_appearance(7, 3, 42, 19, protocol::WindowAppearance::Dark),
    );
    add(
        "event-layout",
        Event::layout(7, 3, 42, 20, 4, 7, 1.0, 2.0, 100.0, 48.0),
    );
    add(
        "event-drag-over",
        Event::drag_over(7, 3, 42, 21, 8, 11, "card".into()),
    );
    add(
        "event-drag-drop",
        Event::drag_drop(7, 3, 42, 22, 8, 11, "card".into()),
    );
    add(
        "event-drag-external",
        Event::external_file_drop(7, 3, 42, 23, 8, 11, vec!["/tmp/a.txt".into()]),
    );
    add(
        "event-notification-response",
        Event::notification_response(7, 3, 42, 24, "tag".into(), Some("open".into())),
    );
    add(
        "event-pointer-down-outside",
        Event::pointer_down_outside(7, 3, 42, 25, 4, 7, 12.0, 13.0),
    );
    add(
        "event-close-requested",
        Event::close_requested(7, 3, 42, 26, 123),
    );
    seeds
}

fn framed(payload: &[u8], declared_length: usize) -> Vec<u8> {
    let mut frame = (declared_length as u32).to_le_bytes().to_vec();
    frame.extend_from_slice(payload);
    frame
}

fn assert_no_panic<T>(label: &str, callback: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(callback)) {
        Ok(value) => value,
        Err(_) => panic!("protocol decoder panicked for {label}"),
    }
}

fn exercise_payload(label: &str, payload: &[u8]) {
    let _ = assert_no_panic(label, || Snapshot::decode(payload));
    let _ = assert_no_panic(label, || Patch::decode(payload));
    let _ = assert_no_panic(label, || Command::decode(payload));
    let _ = assert_no_panic(label, || Event::decode(payload));
}

fn exercise_frame(label: &str, frame: &[u8]) {
    let _ = assert_no_panic(label, || {
        let mut reader = Cursor::new(frame);
        read_frame(&mut reader)
    });
}

fn random_payload(seed: &[u8], rng: &mut XorShift64) -> Vec<u8> {
    let mutation = rng.next() % 5;
    let mut payload = seed.to_vec();
    match mutation {
        0 => {
            let length = rng.index(payload.len() + 1);
            payload.truncate(length);
        }
        1 => {
            if !payload.is_empty() {
                let index = rng.index(payload.len());
                payload[index] ^= 1 << rng.index(8);
            }
        }
        2 => {
            if !payload.is_empty() {
                payload[0] = 0x90;
            }
        }
        3 => payload.extend_from_slice(&rng.next().to_le_bytes()),
        _ => {
            if !payload.is_empty() {
                let index = rng.index(payload.len());
                payload[index] = rng.next() as u8;
            }
        }
    }
    payload
}

fn structured_payloads() -> Vec<(&'static str, Vec<u8>)> {
    let invalid_event_tag = rmp_serde::to_vec(&(
        PROTOCOL_VERSION,
        protocol::EVENT_MESSAGE,
        7u32,
        3u32,
        1u32,
        1u32,
        1u32,
        9u32,
        EVENT_KEY,
        Some((99u32, "A", Vec::<String>::new(), EVENT_KEY_DOWN)),
    ))
    .unwrap();
    let invalid_modifier = rmp_serde::to_vec(&(
        PROTOCOL_VERSION,
        protocol::EVENT_MESSAGE,
        7u32,
        3u32,
        1u32,
        2u32,
        1u32,
        9u32,
        EVENT_KEY,
        Some((5u32, "A", vec!["bogus"], EVENT_KEY_DOWN)),
    ))
    .unwrap();
    let invalid_action = rmp_serde::to_vec(&(
        PROTOCOL_VERSION,
        protocol::EVENT_MESSAGE,
        7u32,
        3u32,
        1u32,
        3u32,
        1u32,
        9u32,
        EVENT_KEY,
        Some((5u32, "A", Vec::<String>::new(), 99u32)),
    ))
    .unwrap();
    let invalid_command = Command {
        kind: 99,
        ..valid_command()
    }
    .encode()
    .unwrap();
    let invalid_kind = Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, 99)])
        .encode()
        .unwrap();
    let invalid_utf8 = vec![0xd9, 1, 0xff];
    let overflow_field = rmp_serde::to_vec(&(
        PROTOCOL_VERSION,
        protocol::SNAPSHOT_MESSAGE,
        u64::from(u32::MAX) + 1,
        3u32,
        0u32,
        1u32,
        Vec::<Vec<u8>>::new(),
    ))
    .unwrap();

    let mut nan_node = Node::new(1, 0, 0, KIND_VIEW);
    nan_node.style = Some(Style {
        width: Some(f32::NAN),
        ..Style::default()
    });
    let nan_style = Snapshot::new(7, 3, 0, 1, vec![nan_node]).encode().unwrap();

    let mut inf_node = Node::new(1, 0, 0, KIND_VIEW);
    inf_node.style = Some(Style {
        height: Some(f32::INFINITY),
        ..Style::default()
    });
    let inf_style = Snapshot::new(7, 3, 0, 1, vec![inf_node]).encode().unwrap();

    let mut negative_node = Node::new(1, 0, 0, KIND_VIEW);
    negative_node.style = Some(Style {
        padding: Some(-1.0),
        ..Style::default()
    });
    let negative_style = Snapshot::new(7, 3, 0, 1, vec![negative_node])
        .encode()
        .unwrap();

    let mut huge_list = Node::new(2, 1, 0, KIND_VIRTUAL_LIST);
    huge_list.host_properties = Some(protocol::HostProperties::VirtualList(
        VirtualListProperties {
            item_count: u32::MAX,
            range_start: 0,
            range_end: 0,
            estimated_item_size: 1.0,
            overscan: 0,
        },
    ));
    let huge_count = Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), huge_list])
        .encode()
        .unwrap();

    vec![
        ("invalid event tag", invalid_event_tag),
        ("invalid modifier", invalid_modifier),
        ("invalid action", invalid_action),
        ("invalid command enum", invalid_command),
        ("invalid node kind", invalid_kind),
        ("invalid utf8", invalid_utf8),
        ("u32 overflow", overflow_field),
        ("nan style", nan_style),
        ("infinite style", inf_style),
        ("negative style", negative_style),
        ("huge count", huge_count),
    ]
}

#[test]
fn deterministic_protocol_decoders_never_panic_on_structured_mutations() {
    let started = Instant::now();
    let mut seeds = vec![
        ("snapshot", valid_snapshot().encode().unwrap()),
        ("patch", valid_patch().encode().unwrap()),
    ];
    seeds.extend(event_seeds());
    seeds.extend(command_seeds());

    for (label, payload) in &seeds {
        let mut frame = Vec::new();
        write_frame(&mut frame, payload).unwrap();
        assert_eq!(
            read_frame(&mut Cursor::new(frame)).unwrap(),
            Some(payload.clone())
        );
        assert!(
            Snapshot::decode(payload).is_ok()
                || Patch::decode(payload).is_ok()
                || Event::decode(payload).is_ok()
                || Command::decode(payload).is_ok(),
            "seed {label} must be a legal protocol payload"
        );
    }
    let wrong_arity = [0x90u8];
    assert!(assert_no_panic("fix-array arity", || Snapshot::decode(&wrong_arity)).is_err());
    assert!(assert_no_panic("fix-array arity", || Patch::decode(&wrong_arity)).is_err());
    assert!(assert_no_panic("fix-array arity", || Command::decode(&wrong_arity)).is_err());
    assert!(assert_no_panic("fix-array arity", || Event::decode(&wrong_arity)).is_err());

    let mut rng = XorShift64::new(0x4d595df4d0f33173);
    let mut cases = 0usize;
    for (label, seed) in &seeds {
        for iteration in 0..RANDOM_CASES_PER_SEED {
            let payload = random_payload(seed, &mut rng);
            exercise_payload(label, &payload);
            let frame_mutation = match iteration % 5 {
                0 => framed(&payload, payload.len()),
                1 => framed(&payload, payload.len().saturating_sub(1)),
                2 => framed(&payload, payload.len().saturating_add(1)),
                3 => framed(&payload, usize::from(u16::MAX)),
                _ => framed(&payload, MAX_FRAME_LENGTH + 1),
            };
            exercise_frame(label, &frame_mutation);
            if iteration % 3 == 0 {
                let mut truncated = frame_mutation.clone();
                truncated.truncate(rng.index(truncated.len() + 1));
                exercise_frame(label, &truncated);
            }
            cases += 1;
        }
    }
    for (label, seed) in &seeds {
        let mut wrong_arity = seed.clone();
        wrong_arity[0] = 0x90;
        exercise_payload(label, &wrong_arity);
        exercise_frame(label, &framed(&wrong_arity, wrong_arity.len()));
        cases += 1;
    }

    for (label, payload) in structured_payloads() {
        exercise_payload(label, &payload);
        exercise_frame(label, &framed(&payload, payload.len()));
        cases += 1;
    }

    assert_eq!(
        cases,
        seeds.len() * RANDOM_CASES_PER_SEED + seeds.len() + 11
    );
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "protocol fuzz exceeded 10 seconds: {:?}",
        started.elapsed()
    );
}
