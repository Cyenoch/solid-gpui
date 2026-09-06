use std::panic::{AssertUnwindSafe, catch_unwind};

use solid_gpui::protocol::{
    COMMAND_MESSAGE, Command, EVENT_MESSAGE, Event, PROTOCOL_VERSION, ProtocolError, Snapshot,
};

fn message(fields: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(fields.len() + 4);
    result.extend_from_slice(&(fields.len() as u32).to_le_bytes());
    result.extend_from_slice(fields);
    result
}
fn union(tag: u8, payload: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(payload.len() + 5);
    result.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    result.push(tag);
    result.extend_from_slice(payload);
    result
}
fn envelope(body: &[u8]) -> Vec<u8> {
    message(&[&[1, 5, 0, 0, 0, 2][..], body, &[0][..]].concat())
}
fn malformed_cases() -> Vec<Vec<u8>> {
    let event = Event::press(7, 3, 1, 1, 1, 1).encode().unwrap();
    let mut wrong_version = event.clone();
    wrong_version[5..9].copy_from_slice(&3u32.to_le_bytes());
    let repeated_nodes = envelope(&union(1, &message(&[5, 0x20, 0xa1, 0x07, 0])));
    let unknown_field = envelope(&[99, 0]);
    let unknown_union = envelope(&[2, 5, 0, 0, 0, 99, 0]);
    let mut trailing = event;
    trailing.push(0);
    vec![
        wrong_version,
        repeated_nodes,
        unknown_field,
        unknown_union,
        trailing,
    ]
}

#[test]
fn malformed_bebop_inputs_never_panic_and_are_rejected() {
    for payload in malformed_cases() {
        let result = catch_unwind(AssertUnwindSafe(|| {
            (
                Event::decode(&payload),
                Snapshot::decode(&payload),
                Command::decode(&payload),
            )
        }));
        let (event, snapshot, command) = result.expect("decoder panicked");
        assert!(
            event.is_err(),
            "Event decoder accepted {} bytes",
            payload.len()
        );
        assert!(
            snapshot.is_err(),
            "Snapshot decoder accepted {} bytes",
            payload.len()
        );
        assert!(
            command.is_err(),
            "Command decoder accepted {} bytes",
            payload.len()
        );
    }
}

#[test]
fn protocol_error_contract_uses_exact_v5() {
    assert_eq!(PROTOCOL_VERSION, 5);
    assert_eq!(EVENT_MESSAGE, 2);
    assert_eq!(COMMAND_MESSAGE, 4);
    let error = Event::decode(&malformed_cases()[0]).expect_err("protocol v3 must be rejected");
    assert!(matches!(
        error,
        ProtocolError::UnsupportedProtocol {
            received: 3,
            expected: 5
        }
    ));
}

fn event_payload(event_type: u32, payload: &[u8], node_id: u32, listener_id: u32) -> Vec<u8> {
    let mut fields = Vec::new();
    fields.extend_from_slice(&[1]);
    fields.extend_from_slice(&7u32.to_le_bytes());
    fields.extend_from_slice(&[2]);
    fields.extend_from_slice(&3u32.to_le_bytes());
    fields.extend_from_slice(&[3]);
    fields.extend_from_slice(&1u32.to_le_bytes());
    fields.extend_from_slice(&[4]);
    fields.extend_from_slice(&1u32.to_le_bytes());
    fields.extend_from_slice(&[5]);
    fields.extend_from_slice(&node_id.to_le_bytes());
    fields.extend_from_slice(&[6]);
    fields.extend_from_slice(&listener_id.to_le_bytes());
    fields.extend_from_slice(&[7]);
    fields.push(u8::try_from(event_type).expect("test event enum fits in u8"));
    if !payload.is_empty() {
        fields.push(8);
        fields.extend_from_slice(payload);
    }
    fields.push(0);
    envelope(&union(2, &message(&fields)))
}

#[test]
fn guard_rejects_strict_bool_enum_utf8_and_field_order_violations() {
    let bool_payload = union(11, &message(&[1, 2, 0]));
    assert!(Event::decode(&event_payload(15, &bool_payload, 1, 0)).is_err());

    let enum_payload = union(13, &message(&[1, 99, 0]));
    assert!(Event::decode(&event_payload(18, &enum_payload, 1, 0)).is_err());

    let key_fields = [
        &[1, 1, 0, 0, 0, 0xff][..],
        &[2, 0, 0, 0, 0],
        &[3, 1, 0, 0, 0],
        &[0][..],
    ]
    .concat();
    let key_payload = union(5, &message(&key_fields));
    assert!(Event::decode(&event_payload(9, &key_payload, 1, 1)).is_err());

    let mut duplicate_fields = event_payload(1, &[], 1, 1);
    let last = duplicate_fields.len() - 1;
    duplicate_fields[last] = 1;
    assert!(Event::decode(&duplicate_fields).is_err());
}
