//! Accepted commits and native draws must preserve visible content while bounding
//! primitive construction independently of an unchanged sibling's node count.
use super::*;
use crate::protocol::{FlexDirectionCode, Node, Patch, Snapshot, UPDATE_TEXT};
use crate::transport::InMemoryAdapter;
use crate::tree::KIND_RAW_TEXT;
use gpui::{AppContext as _, TestAppContext};

pub(super) fn fixture(rows: u32) -> Snapshot {
    let mut shell = Node::new(1, 0, 0, KIND_VIEW);
    shell.style = Some(Style {
        flex_direction: Some(FlexDirectionCode::Row),
        ..Style::default()
    });
    let mut pane = Node::new(2, 1, 0, KIND_VIEW);
    pane.style = Some(Style {
        width: Some(240.0),
        height: Some(400.0),
        flex_shrink: Some(0.0),
        overflow: Some(crate::protocol::OverflowCode::Hidden),
        ..Style::default()
    });
    let mut counter = Node::new(3, 1, 1, KIND_TEXT);
    counter.selectable = true;
    let mut value = Node::new(4, 3, 0, KIND_RAW_TEXT);
    value.text = Some("Count: 0".into());
    let mut nodes = vec![shell, pane, counter, value];
    for row in 0..rows {
        let mut text = Node::new(10 + row * 2, 2, row, KIND_TEXT);
        text.style = Some(Style {
            height: Some(16.0),
            ..Style::default()
        });
        let mut raw = Node::new(11 + row * 2, text.id, 0, KIND_RAW_TEXT);
        raw.text = Some(format!("Static row {row}"));
        nodes.extend([text, raw]);
    }
    Snapshot::new(7, 3, 0, 1, nodes)
}

pub(super) fn update(id: u32, text: &str) -> PatchOperation {
    PatchOperation::Update {
        id,
        mask: UPDATE_TEXT,
        text: Some(text.into()),
        style: None,
        listener_id: 0,
        host_properties: None,
        accessibility: None,
        focusable: false,
        selectable: false,
        tooltip: None,
        accepts_pointer_move: false,
        observes_layout: false,
        observes_hover: false,
    }
}

pub(super) fn draw(cx: &mut TestAppContext, handle: gpui::AnyWindowHandle) {
    cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    cx.run_until_parked();
}

fn style_update(id: u32, style: Style) -> PatchOperation {
    let mut operation = update(id, "unused");
    if let PatchOperation::Update {
        mask,
        text,
        style: target,
        ..
    } = &mut operation
    {
        *mask = crate::protocol::UPDATE_STYLE;
        *text = None;
        *target = Some(style);
    }
    operation
}

#[gpui::test]
fn retained_content_and_inherited_layout_update_then_release_on_removal(cx: &mut TestAppContext) {
    let window = cx.open_window(gpui::size(px(800.0), px(600.0)), |_, _| {
        SolidRoot::new(InMemoryAdapter::new())
    });
    let root = window.root(cx).unwrap();
    root.update(cx, |root, cx| {
        root.apply_decoded_message(DecodedMessage::Snapshot(fixture(10)), cx)
    })
    .unwrap();
    draw(cx, window.into());
    let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
    let before = visual.debug_bounds("solid-gpui-primitive-10").unwrap();
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Patch(Patch::new(
                7,
                3,
                1,
                2,
                vec![update(11, "Updated retained row")],
            )),
            cx,
        )
    })
    .unwrap();
    draw(cx, window.into());
    root.read_with(cx, |root, _| {
        assert_eq!(root.regions.len(), 1);
        assert_eq!(root.painted_text.borrow()[&10], "Updated retained row");
    });
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Patch(Patch::new(
                7,
                3,
                2,
                3,
                vec![style_update(
                    1,
                    Style {
                        flex_direction: Some(FlexDirectionCode::Row),
                        font_size: Some(24.0),
                        padding: Some(30.0),
                        color_rgba: Some(0x00ff00ff),
                        ..Style::default()
                    },
                )],
            )),
            cx,
        )
    })
    .unwrap();
    draw(cx, window.into());
    let after = visual.debug_bounds("solid-gpui-primitive-10").unwrap();
    assert_eq!(after.origin.x - before.origin.x, px(30.0));
    for width in [560.0, 1280.0, 560.0, 800.0] {
        cx.simulate_window_resize(window.into(), gpui::size(px(width), px(600.0)));
        draw(cx, window.into());
        assert_eq!(
            visual
                .debug_bounds("solid-gpui-primitive-2")
                .unwrap()
                .size
                .width,
            px(240.0)
        );
    }
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Patch(Patch::new(
                7,
                3,
                3,
                4,
                vec![PatchOperation::Delete { id: 2 }],
            )),
            cx,
        )
    })
    .unwrap();
    draw(cx, window.into());
    root.read_with(cx, |root, _| assert_eq!(root.regions.len(), 0));
    assert!(visual.debug_bounds("solid-gpui-primitive-10").is_none());
}

#[gpui::test]
fn retained_flex_basis_matches_the_live_parent_allocation(cx: &mut TestAppContext) {
    let window = cx.open_window(gpui::size(px(800.0), px(600.0)), |_, _| {
        SolidRoot::new(InMemoryAdapter::new())
    });
    let root = window.root(cx).unwrap();
    let mut snapshot = fixture(10);
    snapshot.nodes[1].style.as_mut().unwrap().flex_basis = Some(crate::protocol::StyleLength {
        unit: crate::protocol::LengthUnit::Pixels,
        value: 320.0,
    });
    root.update(cx, |root, cx| {
        root.apply_decoded_message(DecodedMessage::Snapshot(snapshot), cx)
    })
    .unwrap();
    draw(cx, window.into());
    let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
    let cached = visual.debug_bounds("solid-gpui-primitive-2").unwrap();
    assert_eq!(cached.size.width, px(320.0));
    let mut operation = update(2, "unused");
    if let PatchOperation::Update {
        mask,
        text,
        listener_id,
        observes_layout,
        ..
    } = &mut operation
    {
        *mask = crate::protocol::UPDATE_LAYOUT | crate::protocol::UPDATE_LISTENER;
        *text = None;
        *listener_id = 9;
        *observes_layout = true;
    }
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Patch(Patch::new(7, 3, 1, 2, vec![operation])),
            cx,
        )
    })
    .unwrap();
    draw(cx, window.into());
    assert_eq!(
        visual.debug_bounds("solid-gpui-primitive-2").unwrap(),
        cached
    );
    root.read_with(cx, |root, _| assert!(!root.regions.contains(2)));
}

#[gpui::test]
fn live_native_input_and_listener_revision_survive_unrelated_retained_content(
    cx: &mut TestAppContext,
) {
    let runtime = InMemoryAdapter::new();
    let window = cx.open_window(gpui::size(px(800.0), px(600.0)), |_, _| {
        SolidRoot::new(runtime.clone())
    });
    let root = window.root(cx).unwrap();
    let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
    let mut snapshot = fixture(100);
    let mut input = Node::new(5, 1, 2, KIND_TEXT_INPUT);
    input.style = Some(Style {
        width: Some(180.0),
        height: Some(32.0),
        ..Style::default()
    });
    input.listener_id = 55;
    input.host_properties = Some(HostProperties::TextInput(TextInputProperties {
        value: "native".into(),
        placeholder: None,
        multiline: false,
        disabled: false,
        controlled: false,
        ack_edit_seq: 0,
        selection_start: 6,
        selection_end: 6,
        marked_start: None,
        marked_end: None,
        max_length: None,
        selection_reversed: false,
    }));
    let mut button = Node::new(6, 1, 3, KIND_PRESSABLE);
    button.listener_id = 66;
    button.style = Some(Style {
        width: Some(60.0),
        height: Some(32.0),
        ..Style::default()
    });
    snapshot.nodes.extend([input, button]);
    root.update(cx, |root, cx| {
        root.apply_decoded_message(DecodedMessage::Snapshot(snapshot), cx)
    })
    .unwrap();
    visual.update(|window, cx| window.draw(cx).clear(cx));
    visual.run_until_parked();
    let point = root.read_with(cx, |root, _| root.text_input_layouts[&5].bounds.center());
    visual.simulate_mouse_down(point, gpui::MouseButton::Left, gpui::Modifiers::none());
    visual.simulate_mouse_up(point, gpui::MouseButton::Left, gpui::Modifiers::none());
    // The first keystroke changes native input modality and deliberately forces
    // a full GPUI refresh. Bound sustained typing after that semantic transition.
    visual.simulate_input("!");
    visual.update(|window, cx| window.draw(cx).clear(cx));
    visual.run_until_parked();
    root.update(cx, |root, _| root.primitive_constructions.set(0));
    visual.simulate_input("!");
    visual.update(|window, cx| window.draw(cx).clear(cx));
    root.read_with(cx, |root, _| {
        assert!(root.text_input_layouts[&5].content.contains('!'));
        assert!(
            root.primitive_constructions.get() < 20,
            "native typing reconstructed {} primitives",
            root.primitive_constructions.get()
        );
    });
    for revision in 2..=5 {
        root.update(cx, |root, cx| {
            root.apply_decoded_message(
                DecodedMessage::Patch(Patch::new(
                    7,
                    3,
                    revision - 1,
                    revision,
                    vec![update(4, &format!("Count: {revision}"))],
                )),
                cx,
            )
        })
        .unwrap();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.run_until_parked();
    }
    let button_point = visual
        .debug_bounds("solid-gpui-primitive-6")
        .unwrap()
        .center();
    visual.simulate_mouse_down(
        button_point,
        gpui::MouseButton::Left,
        gpui::Modifiers::none(),
    );
    visual.simulate_mouse_up(
        button_point,
        gpui::MouseButton::Left,
        gpui::Modifiers::none(),
    );
    let press = loop {
        let event = runtime.take_event().unwrap().expect("hit-tested press");
        if event.event_kind() == crate::protocol::EventKind::Press {
            break event;
        }
    };
    assert_eq!(press.meta.revision, 5);
    assert_eq!(press.meta.listener_id, 66);
    root.read_with(cx, |root, _| {
        assert_eq!(root.selectable_text_layouts[&3].content, "Count: 5");
        assert!(root.text_input_layouts[&5].content.contains('!'));
    });
}

#[gpui::test]
fn intrinsic_width_remains_live_and_moves_its_neighbor(cx: &mut TestAppContext) {
    let window = cx.open_window(gpui::size(px(800.0), px(600.0)), |_, _| {
        SolidRoot::new(InMemoryAdapter::new())
    });
    let root = window.root(cx).unwrap();
    let mut snapshot = fixture(1);
    // Intrinsic geometry is intentionally outside the retained region contract.
    snapshot.nodes[1].style = Some(Style {
        height: Some(100.0),
        ..Style::default()
    });
    root.update(cx, |root, cx| {
        root.apply_decoded_message(DecodedMessage::Snapshot(snapshot), cx)
    })
    .unwrap();
    draw(cx, window.into());
    let before = root.read_with(cx, |root, _| {
        root.selectable_text_layouts[&3].bounds.origin.x
    });
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Patch(Patch::new(
                7,
                3,
                1,
                2,
                vec![update(
                    11,
                    "This text now requires a much wider intrinsic native layout",
                )],
            )),
            cx,
        )
    })
    .unwrap();
    draw(cx, window.into());
    root.read_with(cx, |root, _| {
        assert_eq!(root.regions.len(), 0);
        assert!(root.selectable_text_layouts[&3].bounds.origin.x > before);
        assert_eq!(
            root.painted_text.borrow()[&10],
            "This text now requires a much wider intrinsic native layout"
        );
    });
}

#[gpui::test]
fn adding_native_capabilities_releases_region_and_restoring_static_content_reclaims_it(
    cx: &mut TestAppContext,
) {
    let window = cx.open_window(gpui::size(px(800.0), px(600.0)), |_, _| {
        SolidRoot::new(InMemoryAdapter::new())
    });
    let root = window.root(cx).unwrap();
    root.update(cx, |root, cx| {
        root.apply_decoded_message(DecodedMessage::Snapshot(fixture(10)), cx)
    })
    .unwrap();
    draw(cx, window.into());
    let mut input = Node::new(100, 2, 10, KIND_TEXT_INPUT);
    input.style = Some(Style {
        height: Some(32.0),
        ..Style::default()
    });
    input.host_properties = Some(HostProperties::TextInput(TextInputProperties {
        value: "live inside former region".into(),
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
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Patch(Patch::new(7, 3, 1, 2, vec![PatchOperation::Create(input)])),
            cx,
        )
    })
    .unwrap();
    draw(cx, window.into());
    root.read_with(cx, |root, _| {
        assert!(!root.regions.contains(2));
        assert_eq!(
            root.text_input_layouts[&100].content,
            "live inside former region"
        );
    });
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Patch(Patch::new(
                7,
                3,
                2,
                3,
                vec![PatchOperation::Delete { id: 100 }],
            )),
            cx,
        )
    })
    .unwrap();
    draw(cx, window.into());
    root.read_with(cx, |root, _| assert!(root.regions.contains(2)));
    // Animated ancestor opacity is absent from GPUI's scene-cache key. Adding
    // it releases retained descendants before the animation's first frame.
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Patch(Patch::new(
                7,
                3,
                3,
                4,
                vec![style_update(
                    1,
                    Style {
                        opacity: Some(0.5),
                        transition: Some(crate::protocol::Transition {
                            duration_ms: 100,
                            delay_ms: 0,
                            easing: Easing::Linear,
                            properties: crate::protocol::TRANSITION_OPACITY,
                        }),
                        ..Style::default()
                    },
                )],
            )),
            cx,
        )
    })
    .unwrap();
    draw(cx, window.into());
    root.read_with(cx, |root, _| assert!(!root.regions.contains(2)));
    cx.executor().advance_clock(Duration::from_millis(120));
    draw(cx, window.into());
    root.read_with(cx, |root, _| {
        assert_eq!(root.animation.states[&1].opacity_target, 0.5)
    });
}

#[gpui::test]
fn retained_region_follows_native_scroll_and_keeps_last_content_reachable(cx: &mut TestAppContext) {
    let window = cx.open_window(gpui::size(px(300.0), px(180.0)), |_, _| {
        SolidRoot::new(InMemoryAdapter::new())
    });
    let root = window.root(cx).unwrap();
    let mut snapshot = fixture(10);
    snapshot.nodes[1].parent_id = 5;
    snapshot.nodes[1].index = 0;
    let mut scroll = Node::new(5, 1, 0, KIND_VIEW);
    scroll.style = Some(Style {
        width: Some(240.0),
        height: Some(100.0),
        overflow: Some(crate::protocol::OverflowCode::Scroll),
        ..Style::default()
    });
    snapshot.nodes.insert(1, scroll);
    root.update(cx, |root, cx| {
        root.apply_decoded_message(DecodedMessage::Snapshot(snapshot), cx)
    })
    .unwrap();
    draw(cx, window.into());
    let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
    let before = visual
        .debug_bounds("solid-gpui-primitive-10")
        .unwrap()
        .origin
        .y;
    visual.simulate_event(gpui::ScrollWheelEvent {
        position: gpui::point(px(50.0), px(50.0)),
        delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.0), px(-80.0))),
        ..Default::default()
    });
    draw(cx, window.into());
    let after = visual
        .debug_bounds("solid-gpui-primitive-10")
        .unwrap()
        .origin
        .y;
    assert!(
        after < before,
        "native wheel must displace retained visible content"
    );
    root.read_with(cx, |root, _| {
        assert!(root.regions.contains(2));
        assert_eq!(root.painted_text.borrow()[&28], "Static row 9");
    });
}

#[gpui::test]
fn local_commit_preserves_painted_text_without_constructing_static_sibling(
    cx: &mut TestAppContext,
) {
    for rows in [10, 1_000] {
        let window = cx.open_window(gpui::size(px(800.0), px(600.0)), |_, _| {
            SolidRoot::new(InMemoryAdapter::new())
        });
        let root = window.root(cx).unwrap();
        root.update(cx, |root, cx| {
            root.apply_decoded_message(DecodedMessage::Snapshot(fixture(rows)), cx)
        })
        .unwrap();
        draw(cx, window.into());
        root.update(cx, |root, cx| {
            root.primitive_constructions.set(0);
            root.apply_decoded_message(
                DecodedMessage::Patch(Patch::new(7, 3, 1, 2, vec![update(4, "Count: 12")])),
                cx,
            )
        })
        .unwrap();
        draw(cx, window.into());
        root.read_with(cx, |root, _| {
            assert_eq!(root.text_input_layouts.len(), 0);
            assert_eq!(root.selectable_text_layouts[&3].content, "Count: 12");
            assert!(
                root.primitive_constructions.get() <= 4,
                "unchanged pane constructed {} primitives for {rows} rows",
                root.primitive_constructions.get()
            );
        });
        let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
        assert!(
            visual
                .debug_bounds("solid-gpui-primitive-10")
                .unwrap()
                .size
                .width
                > px(0.0)
        );
        assert!(
            visual
                .debug_bounds("solid-gpui-primitive-2")
                .unwrap()
                .size
                .height
                == px(400.0)
        );
        cx.update_window(window.into(), |_, window, _| window.remove_window())
            .unwrap();
    }
}
