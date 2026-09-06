use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use solid_gpui::protocol::{
    Command, Event, Patch, ProtocolError, Snapshot, read_frame, write_frame,
};

struct Vector {
    id: String,
    kind: String,
    payload: Vec<u8>,
}

struct InvalidVector {
    id: String,
    kind: String,
    payload: Vec<u8>,
    rust_expected: String,
}

struct FrameVector {
    id: String,
    header: Vec<u8>,
    payload: Vec<u8>,
    rust_expected: String,
    ts_expected: String,
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/protocol")
}
fn decode_hex(value: &str) -> Vec<u8> {
    assert!(value.len().is_multiple_of(2));
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).unwrap())
        .collect()
}
fn vectors(file: &str) -> Vec<Vector> {
    fs::read_to_string(fixture_dir().join(file))
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut fields = line.split('\t');
            Vector {
                id: fields.next().unwrap().to_owned(),
                kind: fields.next().unwrap().to_owned(),
                payload: decode_hex(fields.next().unwrap()),
            }
        })
        .collect()
}
fn invalid_vectors() -> Vec<InvalidVector> {
    fs::read_to_string(fixture_dir().join("invalid.hex"))
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut fields = line.split('\t');
            let id = fields.next().unwrap().to_owned();
            let kind = fields.next().unwrap().to_owned();
            let payload = decode_hex(fields.next().unwrap());
            let rust_expected = fields.next().unwrap().to_owned();
            let _ = fields.next().unwrap();
            InvalidVector {
                id,
                kind,
                payload,
                rust_expected,
            }
        })
        .collect()
}
fn frame_vectors() -> Vec<FrameVector> {
    fs::read_to_string(fixture_dir().join("frames.hex"))
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut fields = line.split('\t');
            FrameVector {
                id: fields.next().unwrap().to_owned(),
                header: decode_hex(fields.next().unwrap()),
                payload: decode_hex(fields.next().unwrap()),
                rust_expected: fields.next().unwrap().to_owned(),
                ts_expected: fields.next().unwrap().to_owned(),
            }
        })
        .collect()
}
fn decode_encode(vector: &Vector) -> Result<Vec<u8>, ProtocolError> {
    match vector.kind.as_str() {
        "snapshot" => Snapshot::decode(&vector.payload)?.encode(),
        "patch" => Patch::decode(&vector.payload)?.encode(),
        "command" => Command::decode(&vector.payload)?.encode(),
        "event" => Event::decode(&vector.payload)?.encode(),
        kind => panic!("unknown vector kind {kind}"),
    }
}

#[test]
fn rust_decodes_each_typescript_vector_and_round_trips_semantics() {
    let ts = vectors("ts_to_rust.hex");
    let rust = vectors("rust_to_ts.hex");
    assert!(ts.iter().any(|vector| vector.id == "ts-command-36"));
    assert!(ts.iter().any(|vector| vector.id == "ts-event-38"));
    assert_eq!(rust.len(), ts.len());
    for vector in &ts {
        let encoded = decode_encode(vector)
            .unwrap_or_else(|error| panic!("{} ({}) failed: {error}", vector.id, vector.kind));
        let second = decode_encode(&Vector {
            id: vector.id.clone(),
            kind: vector.kind.clone(),
            payload: encoded.clone(),
        })
        .expect("decoded semantic vector");
        assert_eq!(second, encoded, "semantic round trip for {}", vector.id);
    }
    for vector in &rust {
        let encoded = decode_encode(vector)
            .unwrap_or_else(|error| panic!("{} ({}) failed: {error}", vector.id, vector.kind));
        assert_eq!(
            encoded, vector.payload,
            "Rust golden bytes for {}",
            vector.id
        );
    }
    for (left, right) in ts.iter().zip(rust.iter()) {
        assert_eq!(left.id, right.id);
        assert_eq!(left.kind, right.kind);
        assert_eq!(
            left.payload, right.payload,
            "cross-language bytes for {}",
            left.id
        );
    }
}

#[test]
fn rust_rejects_invalid_bebop_vectors_with_declared_expectations() {
    for vector in invalid_vectors() {
        let result = match vector.kind.as_str() {
            "event" => Event::decode(&vector.payload).map(|_| ()),
            "snapshot" => Snapshot::decode(&vector.payload).map(|_| ()),
            "patch" => Patch::decode(&vector.payload).map(|_| ()),
            "command" => Command::decode(&vector.payload).map(|_| ()),
            kind => panic!("unknown invalid vector kind {kind}"),
        };
        let actual = if result.is_err() { "error" } else { "ok" };
        assert_eq!(actual, vector.rust_expected, "invalid vector {}", vector.id);
    }
}

#[test]
fn rust_frame_decoder_matches_declared_expectations() {
    for vector in frame_vectors() {
        let mut bytes = vector.header;
        bytes.extend_from_slice(&vector.payload);
        let actual = match read_frame(&mut Cursor::new(bytes)) {
            Ok(Some(_)) => "ok",
            Ok(None) => "pending",
            Err(ProtocolError::FrameTooLarge(_)) => "oversize",
            Err(ProtocolError::TruncatedHeader(_) | ProtocolError::TruncatedPayload { .. }) => {
                "truncated"
            }
            Err(error) => panic!("frame {} failed unexpectedly: {error}", vector.id),
        };
        assert_eq!(actual, vector.rust_expected, "frame vector {}", vector.id);
        assert!(
            !vector.ts_expected.is_empty(),
            "frame vector {} lacks TS expectation",
            vector.id
        );
    }
}

#[test]
fn frame_vectors_keep_the_little_endian_transport_contract() {
    let payload = Event::press(7, 3, 1, 1, 1, 1).encode().unwrap();
    let mut framed = Vec::new();
    write_frame(&mut framed, &payload).unwrap();
    let mut reader = Cursor::new(framed);
    assert_eq!(read_frame(&mut reader).unwrap(), Some(payload));
    assert_eq!(read_frame(&mut reader).unwrap(), None);
}
