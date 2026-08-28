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

fn valid_event() -> Event {
    Event::key(
        7,
        3,
        1,
        1,
        1,
        9,
        "ArrowLeft".into(),
        vec!["shift".into(), "cmd".into()],
        KeyAction::Down,
    )
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
        image: None,
    }
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
    let seeds = [
        ("snapshot", valid_snapshot().encode().unwrap()),
        ("patch", valid_patch().encode().unwrap()),
        ("event", valid_event().encode().unwrap()),
        ("command", valid_command().encode().unwrap()),
    ];

    for (label, payload) in &seeds {
        let mut frame = Vec::new();
        write_frame(&mut frame, payload).unwrap();
        assert_eq!(
            read_frame(&mut Cursor::new(frame)).unwrap(),
            Some(payload.clone())
        );
        match *label {
            "snapshot" => assert_eq!(Snapshot::decode(payload).unwrap(), valid_snapshot()),
            "patch" => assert_eq!(Patch::decode(payload).unwrap(), valid_patch()),
            "event" => assert_eq!(Event::decode(payload).unwrap(), valid_event()),
            "command" => assert_eq!(Command::decode(payload).unwrap(), valid_command()),
            _ => unreachable!(),
        }
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

    assert_eq!(cases, seeds.len() * RANDOM_CASES_PER_SEED + 15);
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "protocol fuzz exceeded 10 seconds: {:?}",
        started.elapsed()
    );
}
