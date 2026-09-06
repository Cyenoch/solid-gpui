use crate::protocol::{
    CommandKind, CommandResult, CommandValue, EVENT_BLUR, EVENT_CHANGE, EVENT_FOCUS,
    EVENT_SELECTION, Event, EventMeta, EventPayload, KeyAction, NotificationResponseEvent,
    PointerEvent, ScrollEvent, TextInputEvent, WindowAppearance,
};

#[test]
fn every_event_semantic_round_trips_through_bebop() {
    let text = TextInputEvent {
        text: "hé😀".to_owned(),
        selection_start: 1,
        selection_end: 3,
        marked_start: Some(1),
        marked_end: Some(2),
        edit_seq: 8,
        reversed: true,
    };
    let events = vec![
        Event::press(7, 3, 1, 1, 2, 4),
        Event::text_input(EVENT_CHANGE, 7, 3, 1, 2, 2, 4, text.clone()),
        Event::text_input(EVENT_SELECTION, 7, 3, 1, 3, 2, 4, text.clone()),
        Event::text_input(EVENT_FOCUS, 7, 3, 1, 4, 2, 4, text.clone()),
        Event::text_input(EVENT_BLUR, 7, 3, 1, 5, 2, 4, text),
        Event::command_result(
            7,
            3,
            1,
            6,
            CommandResult {
                request_id: 1,
                command: CommandKind::SetTitle,
                node_id: 1,
                success: true,
                error: None,
                value: Some(CommandValue::Text("ok".to_owned())),
            },
        ),
        Event::visible_range(7, 3, 1, 7, 2, 4, 1, 9),
        Event::animation_complete(7, 3, 1, 8, 2, 4, 4),
        Event::key(
            7,
            3,
            1,
            9,
            2,
            4,
            "Enter".to_owned(),
            vec!["ctrl".to_owned()],
            KeyAction::Repeat,
        ),
        Event::pointer(
            10,
            7,
            3,
            1,
            10,
            2,
            4,
            1,
            vec!["cmd".to_owned()],
            1,
            1,
            1.25,
            2.5,
        ),
        Event::pointer_move(7, 3, 1, 11, 2, 4, 1.0, 2.0, vec![]),
        Event::hover(7, 3, 1, 12, 2, 4),
        Event::scroll(
            7,
            3,
            1,
            13,
            2,
            4,
            1,
            1.0,
            -2.0,
            3.0,
            4.0,
            vec!["alt".to_owned()],
        ),
        Event::submit(7, 3, 1, 14, 2, 4, "submitted".to_owned()),
        Event::window_resize_with_scale(7, 3, 1, 15, 1, 0, 800.0, 600.0, 2.0),
        Event::window_activation(7, 3, 1, 16, 1, 0, true),
        Event::surface_closed(7, 3, 1, 17),
        Event::action(7, 3, 1, 18, "open".to_owned()),
        Event::window_appearance(7, 3, 1, 19, WindowAppearance::Dark),
        Event::layout(7, 3, 1, 20, 2, 4, 1.0, 2.0, 100.0, 50.0),
        Event::drag_over(7, 3, 1, 21, 2, 4, "card".to_owned()),
        Event::drag_drop(7, 3, 1, 22, 2, 4, "card".to_owned()),
        Event::external_file_drop(7, 3, 1, 23, 2, 4, vec!["/tmp/a.txt".to_owned()]),
        Event::notification_response(7, 3, 1, 24, "tag".to_owned(), Some("open".to_owned())),
        Event::pointer_down_outside(7, 3, 1, 25, 2, 4, 2.0, 3.0),
        Event::close_requested(7, 3, 1, 26, 4),
    ];
    for (index, event) in events.into_iter().enumerate() {
        let encoded = event.encode().unwrap_or_else(|error| {
            panic!(
                "event {index} kind {:?}: {error:?}",
                event.payload.event_kind()
            )
        });
        assert_eq!(Event::decode(&encoded).expect("decode event"), event);
    }
}

#[test]
fn event_domain_validation_rejects_invalid_values() {
    let invalid_pointer = Event::new(
        EventMeta {
            surface_id: 7,
            epoch: 3,
            revision: 1,
            sequence: 1,
            node_id: 2,
            listener_id: 4,
        },
        EventPayload::Pointer(PointerEvent {
            button: 1,
            modifiers: vec!["cmd".to_owned(), "cmd".to_owned()],
            action: 1,
            click_count: 1,
            x: f32::NAN,
            y: 2.0,
        }),
    );
    let error = invalid_pointer
        .encode()
        .expect_err("malformed event must not encode");
    assert!(error.to_string().contains("body.event(type=Pointer)"));

    let invalid_scroll = Event::new(
        EventMeta {
            surface_id: 7,
            epoch: 3,
            revision: 1,
            sequence: 2,
            node_id: 2,
            listener_id: 4,
        },
        EventPayload::Scroll(ScrollEvent {
            delta_kind: 99,
            dx: 0.0,
            dy: 0.0,
            x: 0.0,
            y: 0.0,
            modifiers: Vec::new(),
        }),
    );
    assert!(invalid_scroll.encode().is_err());

    let invalid_text_ranges = [
        Event::text_input(
            EVENT_CHANGE,
            7,
            3,
            1,
            3,
            2,
            4,
            TextInputEvent {
                text: "value".to_owned(),
                selection_start: 4,
                selection_end: 3,
                marked_start: None,
                marked_end: None,
                edit_seq: 1,
                reversed: false,
            },
        ),
        Event::text_input(
            EVENT_SELECTION,
            7,
            3,
            1,
            4,
            2,
            4,
            TextInputEvent {
                text: "value".to_owned(),
                selection_start: 0,
                selection_end: 5,
                marked_start: Some(4),
                marked_end: Some(3),
                edit_seq: 1,
                reversed: false,
            },
        ),
    ];
    assert!(
        invalid_text_ranges
            .into_iter()
            .all(|event| event.encode().is_err())
    );
}

#[test]
fn focus_and_blur_keep_null_and_text_input_forms_distinct() {
    let null_focus = Event::focus(7, 3, 1, 1, 2, 4, true);
    assert_eq!(
        Event::decode(&null_focus.encode().unwrap()).unwrap(),
        null_focus
    );
    let text_focus = Event::text_input(
        EVENT_FOCUS,
        7,
        3,
        1,
        2,
        2,
        4,
        TextInputEvent {
            text: "value".to_owned(),
            selection_start: 0,
            selection_end: 5,
            marked_start: None,
            marked_end: None,
            edit_seq: 1,
            reversed: false,
        },
    );
    assert_eq!(
        Event::decode(&text_focus.encode().unwrap()).unwrap(),
        text_focus
    );
    let response = Event::notification_response(7, 3, 1, 3, "tag".to_owned(), None);
    assert_eq!(
        Event::decode(&response.encode().unwrap()).unwrap(),
        response
    );
    assert!(matches!(
        response.payload,
        EventPayload::NotificationResponse(NotificationResponseEvent {
            action_id: None,
            ..
        })
    ));
}
