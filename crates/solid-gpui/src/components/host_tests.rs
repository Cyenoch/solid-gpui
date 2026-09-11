use super::*;
use gpui::{Bounds, KeyBinding, TestAppContext, WindowBounds, px, size};
use solid_gpui::protocol::{ExtensionField, ExtensionProperties, ExtensionValue};
use solid_gpui::{HostProperties, InMemoryAdapter, Node, Snapshot};
use std::sync::Arc;

fn provider_button_snapshot() -> Snapshot {
    Snapshot::new(
        1,
        1,
        0,
        1,
        vec![
            Node::new(1, 0, 0, solid_gpui::KIND_VIEW),
            {
                let mut button = Node::new(2, 1, 0, solid_gpui::KIND_EXTENSION);
                button.host_properties = Some(HostProperties::Extension(ExtensionProperties {
                    provider_id: super::super::native_module().id(),
                    catalog_digest: super::super::native_module().digest(),
                    entry_id: super::super::native_module()
                        .component_id("Button")
                        .unwrap(),
                    entry_version: 1,
                    fields: vec![ExtensionField {
                        id: 1,
                        value: ExtensionValue::Bytes(br#"{"label":"Provider button"}"#.to_vec()),
                    }],
                    event_ids: Arc::from([]),
                }));
                button
            },
            Node::new(3, 2, 0, solid_gpui::KIND_TEXT),
            {
                let mut text = Node::new(4, 3, 0, solid_gpui::KIND_RAW_TEXT);
                text.text = Some("Child".to_owned());
                text
            },
        ],
    )
}

#[gpui::test]
fn profile_wraps_solid_root_with_provider_root_and_renders_extension(cx: &mut TestAppContext) {
    let mut profile = ComponentHost::default();
    let runtime = InMemoryAdapter::new();
    let extensions = profile.extension_registry();
    let (window, solid_root) = cx.update(|app| {
        profile.initialize(app);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(480.0), px(320.0)),
                app,
            ))),
            ..Default::default()
        };
        profile
            .open_window(options, runtime.clone(), extensions, app)
            .expect("provider window opens")
    });

    assert!(window.downcast::<Root>().is_some());
    window
        .update(cx, |_, window, cx| {
            solid_root.update(cx, |root, cx| {
                root.apply_decoded_message_in_window(
                    crate::protocol::decode_message(&provider_button_snapshot().encode().unwrap())
                        .unwrap(),
                    window,
                    cx,
                )
                .unwrap();
            })
        })
        .unwrap();
    window
        .update(cx, |_, window, cx| window.draw(cx).clear(cx))
        .expect("provider Root draws SolidRoot content");
    cx.refresh().expect("provider refresh succeeds");
    window
        .update(cx, |_, window, cx| window.draw(cx).clear(cx))
        .expect("provider Root redraws after refresh");
    solid_root.read_with(cx, |root, _| {
        assert_eq!(root.store().surface_id(), 1);
        assert_eq!(root.store().revision(), 1);
        assert!(root.store().get(2).is_some());
    });
}

#[test]
fn profile_rejects_system_notifications_but_supports_keybindings() {
    let profile = ComponentHost::default();
    assert!(!profile.capabilities().notification_responses);
    assert!(profile.capabilities().set_keybindings);
    assert_eq!(
        profile.rejected_command_reason(solid_gpui::CommandKind::ShowNotification),
        Some(
            "ShowNotification is unavailable in the gpui-component host because gpui-component owns the global system notification callback"
        )
    );
    assert_eq!(
        profile.rejected_command_reason(solid_gpui::CommandKind::SetKeybindings),
        None
    );
}

#[gpui::test]
fn provider_keybindings_restore_baseline_for_replacement_and_close(cx: &mut TestAppContext) {
    let profile = ComponentHost::default();
    let baseline = [KeyBinding::new(
        "cmd-b",
        solid_gpui::MenuAction {
            name: "base".into(),
        },
        None,
    )];
    let first = vec![KeyBinding::new(
        "cmd-1",
        solid_gpui::MenuAction {
            name: "first".into(),
        },
        None,
    )];
    let replacement = vec![KeyBinding::new(
        "cmd-2",
        solid_gpui::MenuAction {
            name: "replacement".into(),
        },
        None,
    )];
    cx.update(|app| profile.restore_keybindings(&baseline, first, app));
    cx.update(|app| {
        let key_bindings = app.key_bindings();
        let bindings = key_bindings.borrow();
        assert_eq!(bindings.bindings().count(), 2);
    });
    cx.update(|app| profile.restore_keybindings(&baseline, replacement, app));
    cx.update(|app| {
        let key_bindings = app.key_bindings();
        let bindings = key_bindings.borrow();
        assert_eq!(bindings.bindings().count(), 2);
    });
    cx.update(|app| profile.restore_keybindings(&baseline, Vec::new(), app));
    cx.update(|app| {
        let key_bindings = app.key_bindings();
        let bindings = key_bindings.borrow();
        assert_eq!(bindings.bindings().count(), 1);
    });
}

#[gpui::test]
fn fps_monitor_is_explicit_per_window_and_released_with_its_window(cx: &mut TestAppContext) {
    let mut profile = ComponentHost::default();
    cx.update(|app| profile.initialize(app));
    let open = |profile: &ComponentHost, cx: &mut TestAppContext| {
        cx.update(|app| {
            profile
                .open_window(
                    WindowOptions::default(),
                    InMemoryAdapter::new(),
                    profile.extension_registry(),
                    app,
                )
                .unwrap()
                .0
        })
    };
    let disabled = open(&profile, cx);
    let read_monitor = |window: AnyWindowHandle, cx: &mut TestAppContext| {
        window
            .update(cx, |root, _, cx| {
                let root = root.downcast::<Root>().unwrap();
                let content = root
                    .read(cx)
                    .view()
                    .clone()
                    .downcast::<ProviderContent>()
                    .unwrap();
                content
                    .read(cx)
                    .frame_monitor
                    .as_ref()
                    .map(Entity::downgrade)
            })
            .unwrap()
    };
    assert!(read_monitor(disabled, cx).is_none());
    profile = profile.with_performance_monitor(true);
    let first = open(&profile, cx);
    let second = open(&profile, cx);
    let monitor = read_monitor(first, cx).unwrap();
    let other = read_monitor(second, cx).unwrap();
    assert_ne!(monitor.entity_id(), other.entity_id());
    first
        .update(cx, |_, window, _| window.remove_window())
        .unwrap();
    cx.run_until_parked();
    assert!(monitor.upgrade().is_none());
    assert!(other.upgrade().is_some());
    second
        .update(cx, |_, window, _| window.remove_window())
        .unwrap();
    disabled
        .update(cx, |_, window, _| window.remove_window())
        .unwrap();
    cx.run_until_parked();
    assert!(other.upgrade().is_none());
}

#[gpui::test]
fn scroll_shadow_composes_with_windowed_virtual_list(cx: &mut TestAppContext) {
    use crate::protocol::{
        ExtensionField, ExtensionProperties, ExtensionValue, VirtualListProperties,
    };
    use crate::{KIND_EXTENSION, KIND_VIEW, KIND_VIRTUAL_LIST, Style};
    let mut profile = ComponentHost::default();
    let runtime = InMemoryAdapter::new();
    let (window, root) = cx.update(|app| {
        profile.initialize(app);
        profile
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                        gpui::point(px(0.), px(0.)),
                        size(px(320.), px(240.)),
                    ))),
                    ..Default::default()
                },
                runtime.clone(),
                profile.extension_registry(),
                app,
            )
            .unwrap()
    });
    let module = super::super::native_module();
    let mut shadow = Node::new(2, 1, 0, KIND_EXTENSION);
    shadow.style = Some(Style {
        width: Some(300.),
        height: Some(180.),
        ..Default::default()
    });
    shadow.host_properties = Some(HostProperties::Extension(ExtensionProperties {
        provider_id: module.id(),
        catalog_digest: module.digest(),
        entry_id: module.component_id("ScrollShadow").unwrap(),
        entry_version: 1,
        fields: vec![ExtensionField {
            id: 1,
            value: ExtensionValue::Bytes(b"{}".to_vec()),
        }],
        event_ids: Arc::from([]),
    }));
    let mut list = Node::new(3, 2, 0, KIND_VIRTUAL_LIST);
    list.listener_id = 3;
    list.style = Some(Style {
        width_percent: Some(100.),
        height_percent: Some(100.),
        ..Default::default()
    });
    list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
        item_count: 100_000,
        range_start: 0,
        range_end: 10,
        estimated_item_size: 30.,
        overscan: 2,
    }));
    let mut nodes = vec![Node::new(1, 0, 0, KIND_VIEW), shadow, list];
    for index in 0..10 {
        let mut row = Node::new(10 + index, 3, index, KIND_VIEW);
        row.style = Some(Style {
            height: Some(30.),
            background_rgba: Some(0x334455ff),
            ..Default::default()
        });
        nodes.push(row);
    }
    let original_nodes = nodes.clone();
    window
        .update(cx, |_, window, cx| {
            root.update(cx, |root, cx| {
                root.apply_decoded_message_in_window(
                    crate::DecodedMessage::Snapshot(Snapshot::new(1, 1, 0, 1, nodes)),
                    window,
                    cx,
                )
                .unwrap()
            });
            window.draw(cx).clear(cx);
        })
        .unwrap();
    window
        .update(cx, |_, window, _| {
            let fades: Vec<_> = window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.background.as_solid().is_none())
                .collect();
            assert_eq!(
                fades.len(),
                1,
                "virtual content below the viewport must produce a trailing fade"
            );
        })
        .unwrap();
    let draw = |cx: &mut TestAppContext| {
        window
            .update(cx, |_, window, cx| {
                window.draw(cx).clear(cx);
                window.simulate_next_frame(cx);
            })
            .unwrap();
        cx.run_until_parked();
    };
    let faded_edges = |cx: &mut TestAppContext| {
        window
            .update(cx, |_, window, _| {
                window
                    .painted_quads()
                    .into_iter()
                    .filter(|quad| quad.background.as_solid().is_none())
                    .count()
            })
            .unwrap()
    };
    draw(cx);
    let ranges = |cx: &mut TestAppContext| {
        draw(cx);
        let mut ranges = Vec::new();
        while let Some(event) = runtime.take_event().unwrap() {
            if let crate::EventPayload::VisibleRange { start, end } = event.payload {
                ranges.push((start, end));
            }
        }
        ranges
    };
    let initial = ranges(cx);
    assert!(
        initial
            .iter()
            .any(|&(start, end)| start == 0 && (6..=10).contains(&end)),
        "first paint must request only visible rows plus overscan: {initial:?}"
    );
    let mut visual = gpui::VisualTestContext::from_window(window, cx);
    visual.simulate_event(gpui::ScrollWheelEvent {
        position: gpui::point(px(50.), px(50.)),
        delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.), px(-90.))),
        ..Default::default()
    });
    draw(cx);
    assert_eq!(
        faded_edges(cx),
        2,
        "wheel must move the list and reveal both faded edges"
    );
    let scrolled = ranges(cx);
    assert!(
        scrolled
            .iter()
            .any(|&(start, end)| start > 0 && end - start <= 10),
        "scroll must advance a bounded virtual range: {scrolled:?}"
    );
    let command = |node_id, request_id, operation, cx: &mut TestAppContext| {
        window
            .update(cx, |_, window, cx| {
                root.update(cx, |root, cx| {
                    root.apply_decoded_message_in_window(
                        crate::DecodedMessage::Command(crate::Command::new(
                            crate::CommandMeta {
                                surface_id: 1,
                                epoch: 1,
                                after_revision: root.store().revision(),
                                request_id,
                                node_id,
                            },
                            operation,
                        )),
                        window,
                        cx,
                    )
                    .unwrap()
                });
                window.draw(cx).clear(cx);
            })
            .unwrap();
        let mut result = None;
        while let Some(event) = runtime.take_event().unwrap() {
            if let crate::EventPayload::CommandResult(reply) = event.payload
                && reply.request_id == request_id
            {
                assert!(reply.success, "{reply:?}");
                result = reply.value;
            }
        }
        result
    };
    let shadow_command = |function_id, args: &[u8]| crate::CommandOperation::InvokeNative {
        module_id: module.id(),
        module_digest: module.digest(),
        function_id,
        args: args.to_vec(),
    };
    command(2, 1, shadow_command(2, br#"{"x":0,"y":150}"#), cx);
    assert_eq!(
        command(3, 2, crate::CommandOperation::GetScrollOffset, cx),
        Some(crate::CommandValue::ScrollOffset(150.)),
        "ScrollShadow.scrollTo must change the actual list rather than an outer scroller"
    );
    command(3, 3, crate::CommandOperation::ScrollToEnd, cx);
    draw(cx);
    assert_eq!(
        faded_edges(cx),
        1,
        "the final row must clear the trailing fade"
    );
    assert_eq!(
        command(3, 6, crate::CommandOperation::GetScrollOffset, cx),
        Some(crate::CommandValue::ScrollOffset(2_999_820.)),
        "unmeasured rows must remain in the scrollbar extent without eager rendering"
    );
    command(2, 4, shadow_command(2, br#"{"x":0,"y":0}"#), cx);
    draw(cx);
    assert_eq!(
        faded_edges(cx),
        1,
        "returning to the start must clear the leading fade"
    );
    assert_eq!(
        command(3, 5, crate::CommandOperation::GetScrollOffset, cx),
        Some(crate::CommandValue::ScrollOffset(0.))
    );
    let mut visual = gpui::VisualTestContext::from_window(window, cx);
    visual.simulate_mouse_move(
        gpui::point(px(296.), px(120.)),
        None,
        gpui::Modifiers::none(),
    );
    visual.simulate_mouse_down(
        gpui::point(px(296.), px(120.)),
        gpui::MouseButton::Left,
        gpui::Modifiers::none(),
    );
    visual.simulate_mouse_up(
        gpui::point(px(296.), px(120.)),
        gpui::MouseButton::Left,
        gpui::Modifiers::none(),
    );
    draw(cx);
    assert!(
        matches!(command(3, 7, crate::CommandOperation::GetScrollOffset, cx), Some(crate::CommandValue::ScrollOffset(offset)) if offset > 1_000_000.),
        "scrollbar input must navigate the full virtual dataset"
    );
    command(2, 8, shadow_command(2, br#"{"x":0,"y":0}"#), cx);
    for (step, width) in [180., 500., 180., 300.].into_iter().enumerate() {
        let mut resized = original_nodes.clone();
        resized[1].style.as_mut().unwrap().width = Some(width);
        window
            .update(cx, |_, window, cx| {
                root.update(cx, |root, cx| {
                    root.apply_decoded_message_in_window(
                        crate::DecodedMessage::Snapshot(Snapshot::new(
                            1,
                            1,
                            1 + step as u32,
                            2 + step as u32,
                            resized,
                        )),
                        window,
                        cx,
                    )
                    .unwrap()
                });
                window.draw(cx).clear(cx);
            })
            .unwrap();
        assert_eq!(
            faded_edges(cx),
            1,
            "resize must preserve the bounded list viewport and its overflow"
        );
    }
    command(3, 9, crate::CommandOperation::ScrollToEnd, cx);
    let mut filtered = original_nodes.clone();
    filtered.truncate(6);
    if let Some(HostProperties::VirtualList(list)) = &mut filtered[2].host_properties {
        list.item_count = 3;
        list.range_end = 3;
    }
    window
        .update(cx, |_, window, cx| {
            root.update(cx, |root, cx| {
                root.apply_decoded_message_in_window(
                    crate::DecodedMessage::Snapshot(Snapshot::new(1, 1, 5, 6, filtered)),
                    window,
                    cx,
                )
                .unwrap()
            });
            window.draw(cx).clear(cx);
        })
        .unwrap();
    assert_eq!(
        faded_edges(cx),
        0,
        "content that fits after filtering must clear both fades"
    );
    assert_eq!(
        command(3, 10, crate::CommandOperation::GetScrollOffset, cx),
        Some(crate::CommandValue::ScrollOffset(0.))
    );
}

#[gpui::test]
fn scroll_shadow_native_list_decoration_follows_reparenting(cx: &mut TestAppContext) {
    use crate::{KIND_EXTENSION, KIND_VIEW, Style};
    let mut profile = ComponentHost::default();
    let runtime = InMemoryAdapter::new();
    let (window, root) = cx.update(|app| {
        profile.initialize(app);
        profile
            .open_window(
                WindowOptions::default(),
                runtime.clone(),
                profile.extension_registry(),
                app,
            )
            .unwrap()
    });
    let module = super::super::native_module();
    let component = |id, parent, name, props: &[u8]| {
        let mut node = Node::new(id, parent, 0, KIND_EXTENSION);
        node.host_properties = Some(HostProperties::Extension(ExtensionProperties {
            provider_id: module.id(),
            catalog_digest: module.digest(),
            entry_id: module.component_id(name).unwrap(),
            entry_version: 1,
            fields: vec![ExtensionField {
                id: 1,
                value: ExtensionValue::Bytes(props.to_vec()),
            }],
            event_ids: Arc::from([]),
        }));
        node
    };
    let mut shadow = component(2, 1, "ScrollShadow", br#"{"axis":"horizontal"}"#);
    shadow.style = Some(Style {
        width: Some(300.),
        height: Some(100.),
        ..Default::default()
    });
    let mut list = component(
        3,
        2,
        "VirtualList",
        br#"{"orientation":"horizontal","itemSize":100}"#,
    );
    list.style = Some(Style {
        width_percent: Some(100.),
        height_percent: Some(100.),
        ..Default::default()
    });
    let mut nodes = vec![Node::new(1, 0, 0, KIND_VIEW), shadow, list];
    for index in 0..1000 {
        let mut row = Node::new(10 + index, 3, index, KIND_VIEW);
        row.style = Some(Style {
            width: Some(100.),
            height: Some(100.),
            background_rgba: Some(0x334455ff),
            ..Default::default()
        });
        nodes.push(row);
    }
    let apply = |revision, nodes, cx: &mut TestAppContext| {
        window
            .update(cx, |_, window, cx| {
                root.update(cx, |root, cx| {
                    root.apply_decoded_message_in_window(
                        crate::DecodedMessage::Snapshot(Snapshot::new(
                            1,
                            1,
                            revision - 1,
                            revision,
                            nodes,
                        )),
                        window,
                        cx,
                    )
                    .unwrap()
                });
                window.draw(cx).clear(cx);
            })
            .unwrap();
    };
    apply(1, nodes.clone(), cx);
    let geometry = |cx: &mut TestAppContext| {
        window
            .update(cx, |_, window, _| {
                let quads = window.painted_quads();
                let fades = quads
                    .iter()
                    .filter(|quad| quad.background.as_solid().is_none())
                    .count();
                let mut thumb_bounds = Vec::new();
                for quad in quads
                    .iter()
                    .filter(|quad| quad.corner_radii.top_left > Default::default())
                {
                    if !thumb_bounds.contains(&quad.bounds) {
                        thumb_bounds.push(quad.bounds);
                    }
                }
                let thumbs = thumb_bounds.len();
                let rows = quads
                    .iter()
                    .filter(|quad| {
                        quad.background
                            .as_solid()
                            .is_some_and(|color| color == gpui::rgba(0x334455ff).into())
                    })
                    .count();
                (fades, thumbs, rows)
            })
            .unwrap()
    };
    let (fades, thumbs, rows) = geometry(cx);
    assert_eq!(
        (fades, thumbs),
        (1, 1),
        "one borrowed viewport must have one fade and one scrollbar"
    );
    assert!(
        rows > 0 && rows < 10,
        "only visible native rows should be painted"
    );
    let mut visual = gpui::VisualTestContext::from_window(window, cx);
    visual.simulate_event(gpui::ScrollWheelEvent {
        position: gpui::point(px(50.), px(50.)),
        delta: gpui::ScrollDelta::Pixels(gpui::point(px(-150.), px(0.))),
        ..Default::default()
    });
    window
        .update(cx, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    assert_eq!(
        geometry(cx).0,
        2,
        "native horizontal scrolling must update borrowed fades"
    );
    let mut standalone = nodes.clone();
    standalone[2].parent_id = 1;
    standalone[2].index = 1;
    standalone[2].style = Some(Style {
        width: Some(300.),
        height: Some(100.),
        ..Default::default()
    });
    apply(2, standalone, cx);
    let (fades, thumbs, _) = geometry(cx);
    assert_eq!(
        (fades, thumbs),
        (0, 1),
        "reparented list must regain its own scrollbar immediately"
    );
    apply(3, nodes, cx);
    assert_eq!(
        geometry(cx).1,
        1,
        "re-decoration must suppress the list's scrollbar again"
    );
}
