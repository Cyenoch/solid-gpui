use super::support::*;

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
fn layout_events_round_trip_and_reject_non_finite_bounds() {
    let event = Event::layout(7, 3, 1, 6, 9, 11, 12.5, -3.25, 100.0, 48.75);
    assert_eq!(Event::decode(&event.encode().unwrap()).unwrap(), event);

    let malformed = rmp_serde::to_vec(&(
        3u32,
        2u32,
        7u32,
        3u32,
        1u32,
        7u32,
        9u32,
        11u32,
        EVENT_LAYOUT,
        Some((12.5f32, -3.25f32, f32::NAN, 48.75f32)),
    ))
    .unwrap();
    assert!(matches!(
        Event::decode(&malformed),
        Err(ProtocolError::InvalidEventPayload)
    ));
}

#[test]
fn drag_events_round_trip_all_payload_kinds_and_reject_invalid_paths() {
    let events = [
        Event::drag_over(7, 3, 1, 7, 9, 11, "card".into()),
        Event::drag_drop(7, 3, 1, 8, 9, 11, "card".into()),
        Event::external_file_drop(
            7,
            3,
            1,
            9,
            9,
            11,
            vec!["/tmp/a.txt".into(), "/tmp/b".into()],
        ),
    ];
    for event in events {
        assert_eq!(Event::decode(&event.encode().unwrap()).unwrap(), event);
    }

    let malformed = rmp_serde::to_vec(&(
        3u32,
        2u32,
        7u32,
        3u32,
        1u32,
        10u32,
        9u32,
        11u32,
        EVENT_DRAG,
        Some((3u32, vec!["".to_owned()])),
    ))
    .unwrap();
    assert!(matches!(
        Event::decode(&malformed),
        Err(ProtocolError::InvalidEventPayload)
    ));

    let mut node = Node::new(2, 1, 0, KIND_VIEW);
    node.host_properties = Some(HostProperties::Drag(DragProperties {
        drag_type: Some("card".into()),
    }));
    let snapshot = root_snapshot(1, vec![Node::new(1, 0, 0, KIND_VIEW), node]);
    assert_eq!(
        Snapshot::decode(&snapshot.encode().unwrap()).unwrap(),
        snapshot
    );
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
            scale_factor: 1.0,
        })
    );
}
#[test]
fn window_resize_wire_accepts_scale_factor() {
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
        Some((800u32, 600u32, 1.5f32)),
    ))
    .unwrap();
    let event = Event::decode(&payload).unwrap();
    assert_eq!(
        event.payload,
        Some(EventPayload::WindowResize {
            width: 800.0,
            height: 600.0,
            scale_factor: 1.5,
        })
    );
}

#[test]
fn window_resize_wire_rejects_non_positive_scale_factor() {
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
        Some((800u32, 600u32, 0.0f32)),
    ))
    .unwrap();
    assert!(matches!(
        Event::decode(&payload),
        Err(ProtocolError::InvalidEventPayload)
    ));
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
fn window_appearance_wire_accepts_light_dark_and_rejects_other_values() {
    for appearance in ["light", "dark"] {
        let payload = rmp_serde::to_vec(&(
            3u32,
            2u32,
            7u32,
            3u32,
            1u32,
            1u32,
            1u32,
            0u32,
            EVENT_WINDOW_APPEARANCE,
            appearance,
        ))
        .unwrap();
        assert!(matches!(
            Event::decode(&payload).unwrap().payload,
            Some(EventPayload::WindowAppearance { .. })
        ));
    }
    for appearance in [true, false] {
        let payload = rmp_serde::to_vec(&(
            3u32,
            2u32,
            7u32,
            3u32,
            1u32,
            1u32,
            1u32,
            0u32,
            EVENT_WINDOW_APPEARANCE,
            appearance,
        ))
        .unwrap();
        assert!(matches!(
            Event::decode(&payload),
            Err(ProtocolError::Decode(_))
        ));
    }
    let payload = rmp_serde::to_vec(&(
        3u32,
        2u32,
        7u32,
        3u32,
        1u32,
        1u32,
        1u32,
        0u32,
        EVENT_WINDOW_APPEARANCE,
        "system",
    ))
    .unwrap();
    assert!(matches!(
        Event::decode(&payload),
        Err(ProtocolError::InvalidEventPayload)
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
