use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use react_gpui::protocol::{
    self, Command, Event, HostProperties, Patch, PatchOperation, ProtocolError, Snapshot,
};
use react_gpui::{EventPayload, ScrollEvent};

struct Vector {
    id: String,
    kind: String,
    payload: Vec<u8>,
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/protocol")
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert!(
        value.len().is_multiple_of(2),
        "odd hex length in fixture: {value}"
    );
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).expect("valid fixture hex"))
        .collect()
}

fn valid_vectors(file: &str) -> Vec<Vector> {
    let path = fixture_dir().join(file);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            assert_eq!(fields.len(), 3, "invalid vector row: {line}");
            Vector {
                id: fields[0].to_owned(),
                kind: fields[1].to_owned(),
                payload: decode_hex(fields[2]),
            }
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
enum Decoded {
    Snapshot(Snapshot),
    Patch(Patch),
    Command(Command),
    Event(Event),
}

fn decode_vector(vector: &Vector) -> Decoded {
    match vector.kind.as_str() {
        "snapshot" => Decoded::Snapshot(
            Snapshot::decode(&vector.payload)
                .unwrap_or_else(|error| panic!("{}: snapshot decode failed: {error}", vector.id)),
        ),
        "patch" => Decoded::Patch(
            Patch::decode(&vector.payload)
                .unwrap_or_else(|error| panic!("{}: patch decode failed: {error}", vector.id)),
        ),
        "command" => Decoded::Command(
            Command::decode(&vector.payload)
                .unwrap_or_else(|error| panic!("{}: command decode failed: {error}", vector.id)),
        ),
        "event" => Decoded::Event(
            Event::decode(&vector.payload)
                .unwrap_or_else(|error| panic!("{}: event decode failed: {error}", vector.id)),
        ),
        other => panic!("unknown fixture message kind: {other}"),
    }
}

fn encode_decoded(decoded: &Decoded) -> Vec<u8> {
    match decoded {
        Decoded::Snapshot(value) => value.encode().expect("encode Snapshot"),
        Decoded::Patch(value) => value.encode().expect("encode Patch"),
        Decoded::Command(value) => value.encode().expect("encode Command"),
        Decoded::Event(value) => value.encode().expect("encode Event"),
    }
}

fn assert_representative_fields(vector: &Vector) {
    match vector.id.as_str() {
        "rust-snapshot-all-kinds" | "ts-snapshot-all-kinds" => {
            let snapshot = Snapshot::decode(&vector.payload).expect("snapshot fixture");
            assert_eq!(snapshot.nodes.len(), 8);
            assert!(matches!(
                snapshot.nodes[4].host_properties,
                Some(HostProperties::TextInput(_))
            ));
            assert!(matches!(
                snapshot.nodes[5].host_properties,
                Some(HostProperties::VirtualList(_))
            ));
            assert!(matches!(
                snapshot.nodes[6].host_properties,
                Some(HostProperties::Image(_))
            ));
            assert!(snapshot.nodes[0].style.is_some());
            assert!(matches!(
                snapshot.nodes[7].host_properties,
                Some(HostProperties::Drag(_))
            ));
        }
        "rust-patch-all-operations" | "ts-patch-all-operations" => {
            let patch = Patch::decode(&vector.payload).expect("patch fixture");
            assert!(matches!(patch.operations[0], PatchOperation::Create(_)));
            assert!(matches!(patch.operations[1], PatchOperation::Update { .. }));
            assert!(matches!(patch.operations[2], PatchOperation::Move { .. }));
            assert!(matches!(patch.operations[3], PatchOperation::Delete { .. }));
        }
        id if id.ends_with("event-scroll") => {
            let event = Event::decode(&vector.payload).expect("scroll fixture");
            assert!(matches!(
                event.payload,
                Some(EventPayload::Scroll(ScrollEvent { dx, dy, .. }))
                    if (dx - 3.5).abs() < f32::EPSILON && (dy + 2.25).abs() < f32::EPSILON
            ));
        }
        _ => {}
    }
}

#[test]
fn rust_checks_own_bytes_and_cross_direction_semantics() {
    let rust_vectors = valid_vectors("rust_to_ts.hex");
    let ts_vectors = valid_vectors("ts_to_rust.hex");
    let ts_by_key: HashMap<_, _> = ts_vectors
        .iter()
        .map(|vector| (vector.id.trim_start_matches("ts-"), vector))
        .collect();
    let mut count = 0;
    for vector in &rust_vectors {
        let key = vector.id.trim_start_matches("rust-");
        let rust_decoded = decode_vector(vector);
        assert_eq!(
            encode_decoded(&rust_decoded),
            vector.payload,
            "Rust own bytes: {}",
            vector.id
        );
        let ts_vector = ts_by_key.get(key).expect("producer pair");
        let ts_decoded = decode_vector(ts_vector);
        assert_eq!(
            rust_decoded, ts_decoded,
            "cross-direction semantic value: {key}"
        );
        assert_representative_fields(vector);
        count += 1;
    }
    assert!(count >= 20, "golden corpus unexpectedly small: {count}");
}

#[test]
fn rust_rejects_invalid_golden_payloads() {
    let path = fixture_dir().join("invalid.hex");
    let content = fs::read_to_string(&path).expect("invalid fixture file");
    let mut count = 0;
    for line in content
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 5, "invalid vector row: {line}");
        let payload = decode_hex(fields[2]);
        let result: Result<(), ProtocolError> = match fields[1] {
            "snapshot" => Snapshot::decode(&payload).map(|_| ()),
            "patch" => Patch::decode(&payload).map(|_| ()),
            "command" => Command::decode(&payload).map(|_| ()),
            "event" => Event::decode(&payload).map(|_| ()),
            other => panic!("unknown invalid message kind: {other}"),
        };
        match fields[3] {
            "error" => assert!(
                result.is_err(),
                "Rust accepted invalid vector {}",
                fields[0]
            ),
            "ok" => assert!(
                result.is_ok(),
                "Rust rejected forward-compatible vector {}",
                fields[0]
            ),
            other => panic!("unknown Rust invalid-vector expectation: {other}"),
        }
        count += 1;
    }
    assert!(count >= 4);
}

#[test]
fn rust_frame_vectors_lock_length_boundaries() {
    let path = fixture_dir().join("frames.hex");
    let content = fs::read_to_string(&path).expect("frame fixture file");
    let mut count = 0;
    for line in content
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 5, "frame vector row: {line}");
        let mut frame = decode_hex(fields[1]);
        frame.extend(decode_hex(fields[2]));
        let result = protocol::read_frame(&mut Cursor::new(frame));
        match fields[3] {
            "ok" => assert_eq!(result.expect("valid frame"), Some(Vec::new())),
            "truncated" => assert!(matches!(
                result,
                Err(ProtocolError::TruncatedHeader(_) | ProtocolError::TruncatedPayload { .. })
            )),
            "oversize" => assert!(matches!(result, Err(ProtocolError::FrameTooLarge(_)))),
            other => panic!("unknown Rust frame expectation: {other}"),
        }
        count += 1;
    }
    assert!(count >= 5);
}
