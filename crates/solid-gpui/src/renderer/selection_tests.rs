use super::*;
use crate::native::text::*;
use crate::protocol::{Node, Snapshot};
use crate::transport::InMemoryAdapter;
use crate::tree::KIND_RAW_TEXT;
use gpui::{AppContext, VisualTestContext, point, size};

fn paragraph(id: u32, index: u32, content: &str) -> Vec<Node> {
    let mut text = Node::new(id, 1, index, KIND_TEXT);
    text.selectable = true;
    let mut raw = Node::new(id + 1, id, 0, KIND_RAW_TEXT);
    raw.text = Some(content.into());
    vec![text, raw]
}

#[gpui::test]
fn surface_selection_observer_routes_revisions_and_unmount_retires_subscription(
    cx: &mut gpui::TestAppContext,
) {
    use crate::protocol::{
        EventPayload, ExtensionField, ExtensionProperties, ExtensionValue, Patch,
    };
    let runtime = InMemoryAdapter::new();
    let module = crate::native::text::native_module();
    let mut observer = Node::new(6, 1, 2, KIND_EXTENSION);
    observer.listener_id = 10;
    observer.host_properties = Some(HostProperties::Extension(ExtensionProperties {
        provider_id: module.id(),
        catalog_digest: module.digest(),
        entry_id: module.component_id("TextSelectionObserver").unwrap(),
        entry_version: 1,
        event_ids: vec![1].into(),
        fields: vec![ExtensionField {
            id: 1,
            value: ExtensionValue::Bytes(crate::native::encode_native_request(module.build_digest(), &serde_json::json!({})).unwrap()),
        }],
    }));
    let root = cx.new(|_| SolidRoot::with_extensions(runtime.clone(), Rc::new(module)));
    let mut nodes = vec![Node::new(1, 0, 0, KIND_VIEW)];
    nodes.extend(paragraph(2, 0, "first"));
    nodes.extend(paragraph(4, 1, "second"));
    nodes.push(observer);
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Snapshot(Snapshot::new(20, 1, 0, 1, nodes)),
            cx,
        )
    })
    .unwrap();
    while runtime.take_event().unwrap().is_some() {}
    root.update(cx, |root, _| {
        root.set_text_selection(TextSelectionRequest {
            text_revision: root.text_selection_snapshot().unwrap().text_revision,
            anchor: TextPosition {
                node_id: 2,
                offset: 0,
            },
            head: TextPosition {
                node_id: 4,
                offset: 6,
            },
        })
        .unwrap();
    });
    let event = runtime.take_event().unwrap().unwrap();
    assert_eq!(event.meta.listener_id, 10);
    let EventPayload::Extension { fields, .. } = event.payload else {
        panic!("selection observer event")
    };
    let ExtensionValue::Bytes(bytes) = &fields[0].value else {
        panic!("revision JSON")
    };
    let event: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    assert_eq!(event["selectedParagraphs"], 2);
    assert!(
        event.get("text").is_none(),
        "drag notifications carry metadata only"
    );
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Patch(Patch::new(
                20,
                1,
                1,
                2,
                vec![PatchOperation::Delete { id: 6 }],
            )),
            cx,
        )
    })
    .unwrap();
    root.update(cx, |root, _| root.clear_text_selection());
    assert!(runtime.take_event().unwrap().is_none());
}

#[gpui::test]
fn surface_text_service_rejects_stale_unicode_ranges_and_search_and_releases_epoch(
    cx: &mut gpui::TestAppContext,
) {
    use crate::protocol::{CommandOperation, EventPayload};
    let runtime = InMemoryAdapter::new();
    let module = crate::native::text::native_module();
    let (id, digest) = (module.id(), module.digest());
    let window = cx.open_window(size(px(300.), px(180.)), {
        let runtime = runtime.clone();
        move |_, _| SolidRoot::with_extensions(runtime, Rc::new(module))
    });
    let root = window.root(cx).unwrap();
    let mut nodes = vec![Node::new(1, 0, 0, KIND_VIEW)];
    nodes.extend(paragraph(2, 0, "A🙂 新值"));
    nodes.extend(paragraph(4, 1, "e\u{301} tail"));
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Snapshot(Snapshot::new(9, 1, 0, 1, nodes)),
            cx,
        )
    })
    .unwrap();
    let contract = crate::native::text::native_module();
    let call = |cx: &mut gpui::TestAppContext, name: &str, args: serde_json::Value, request_id| {
        cx.update_window(window.into(), |_, window, cx| {
            root.update(cx, |root, cx| {
                root.apply_decoded_message(
                    DecodedMessage::Command(Command::new(
                        CommandMeta {
                            surface_id: 9,
                            epoch: 1,
                            after_revision: root.store.revision(),
                            request_id,
                            node_id: 1,
                        },
                        CommandOperation::InvokeNative {
                            module_id: id,
                            module_digest: digest,
                            function_id: contract.command_id(name).unwrap(),
                            args: crate::native::encode_native_request(contract.build_digest(), &args).unwrap(),
                        },
                    )),
                    cx,
                )
                .unwrap();
                root.process_commands(window, cx);
            })
        })
        .unwrap();
        loop {
            if let EventPayload::CommandResult(result) =
                runtime.take_event().unwrap().expect("native reply").payload
            {
                break result;
            }
        }
    };
    let result = call(cx, "getTextSelection", serde_json::Value::Null, 1);
    assert!(result.success);
    let revision = root.read_with(cx, |root, _| {
        root.text_selection_snapshot().unwrap().text_revision
    });
    let result = call(
        cx,
        "setTextSelection",
        serde_json::json!({"textRevision":revision,"anchor":{"nodeId":2,"offset":1},"head":{"nodeId":4,"offset":2}}),
        2,
    );
    assert!(result.success);
    let result = call(
        cx,
        "setTextSelection",
        serde_json::json!({"textRevision":revision,"anchor":{"nodeId":2,"offset":2},"head":{"nodeId":4,"offset":2}}),
        3,
    );
    assert!(!result.success, "surrogate split must reject atomically");
    assert_eq!(
        root.read_with(cx, |root, _| root.text_selection_snapshot().unwrap().text),
        "🙂 新值\ne\u{301}"
    );
    let search = root
        .update(cx, |root, _| {
            root.search_text(TextSearchRequest {
                query: "新值\ne\u{301}".into(),
            })
        })
        .unwrap();
    assert_eq!(search.matches.len(), 1);
    assert_eq!(search.matches[0].len(), 2);
    cx.update_window(window.into(), |_, window, cx| {
        root.update(cx, |root, cx| {
            root.select_text_search_match(
                TextSearchSelection {
                    text_revision: search.text_revision,
                    search_revision: search.search_revision,
                    match_index: 0,
                },
                window,
                cx,
            )
            .unwrap();
        })
    })
    .unwrap();
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    root.read_with(cx, |root, _| {
        for id in [2, 4] {
            let layout = &root.selectable_text_layouts[&id];
            let ranges = root.search_ranges(id);
            assert_eq!(ranges.len(), 1);
            assert!(
                layout
                    .text
                    .selection_bounds_per_line(ranges[0].clone(), layout.bounds)
                    .iter()
                    .any(|bounds| bounds.size.width > px(0.))
            );
        }
    });
    // Restore the original directed range before the reorder contract check.
    root.update(cx, |root, _| {
        root.set_text_selection(TextSelectionRequest {
            text_revision: revision,
            anchor: TextPosition {
                node_id: 2,
                offset: 1,
            },
            head: TextPosition {
                node_id: 4,
                offset: 2,
            },
        })
    })
    .unwrap();
    let other = cx.new(|_| SolidRoot::new(InMemoryAdapter::new()));
    other.update(cx, |other, cx| {
        let mut nodes = vec![Node::new(1, 0, 0, KIND_VIEW)];
        nodes.extend(paragraph(2, 0, "independent"));
        other
            .apply_decoded_message(
                DecodedMessage::Snapshot(Snapshot::new(11, 1, 0, 1, nodes)),
                cx,
            )
            .unwrap();
        other
            .set_text_selection(TextSelectionRequest {
                text_revision: other.text_selection_snapshot().unwrap().text_revision,
                anchor: TextPosition {
                    node_id: 2,
                    offset: 0,
                },
                head: TextPosition {
                    node_id: 2,
                    offset: 11,
                },
            })
            .unwrap();
    });
    assert_eq!(
        other.read_with(cx, |other, _| other.text_selection_snapshot().unwrap().text),
        "independent"
    );
    let mut nodes = vec![Node::new(1, 0, 0, KIND_VIEW)];
    nodes.extend(paragraph(4, 0, "e\u{301} tail"));
    nodes.extend(paragraph(2, 1, "A🙂 新值"));
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Snapshot(Snapshot::new(9, 1, 1, 2, nodes)),
            cx,
        )
    })
    .unwrap();
    assert_eq!(
        root.read_with(cx, |root, _| root.text_selection_snapshot().unwrap().text),
        " tail\nA"
    );
    let result = call(
        cx,
        "selectTextSearchMatch",
        serde_json::json!({"textRevision":search.text_revision,"searchRevision":search.search_revision,"matchIndex":0}),
        4,
    );
    assert!(!result.success);
    let stale = call(
        cx,
        "setTextSelection",
        serde_json::json!({"textRevision":revision,"anchor":{"nodeId":2,"offset":1},"head":{"nodeId":4,"offset":2}}),
        5,
    );
    assert!(!stale.success);
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Snapshot(Snapshot::new(
                9,
                2,
                0,
                1,
                vec![Node::new(1, 0, 0, KIND_VIEW)],
            )),
            cx,
        )
    })
    .unwrap();
    assert!(root.read_with(cx, |root, _| {
        root.text_selection_snapshot().unwrap().text.is_empty()
    }));
    assert!(root.read_with(cx, |root, _| {
        root.text_search_snapshot().unwrap().matches.is_empty()
    }));
    assert_eq!(
        other.read_with(cx, |other, _| other.text_selection_snapshot().unwrap().text),
        "independent"
    );
    other.update(cx, |root, _| root.emit_surface_closed());
    assert!(other.read_with(cx, |root, _| {
        root.text_selection_snapshot().unwrap().text.is_empty()
    }));
}

#[gpui::test]
fn surface_selection_virtual_eviction_retains_copy_and_data_replacement_clears(
    cx: &mut gpui::TestAppContext,
) {
    use crate::protocol::{HostProperties, VirtualListProperties};
    let window = cx.open_window(size(px(300.), px(150.)), |_, _| {
        SolidRoot::new(InMemoryAdapter::new())
    });
    let root = window.root(cx).unwrap();
    let snapshot = |base, revision, range_start, data_revision, rows: &[(u32, &str)]| {
        let mut list = Node::new(2, 1, 0, crate::tree::KIND_VIRTUAL_LIST);
        list.style = Some(Style {
            height: Some(120.),
            ..Style::default()
        });
        list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
            item_count: 100,
            range_start,
            range_end: range_start + rows.len() as u32,
            data_revision,
            estimated_item_size: 24.,
            overscan: 1,
            data_edit: None,
        }));
        let mut nodes = vec![Node::new(1, 0, 0, KIND_VIEW), list];
        for (index, (id, content)) in rows.iter().enumerate() {
            let mut row = Node::new(*id, 2, index as u32, KIND_VIEW);
            row.style = Some(Style {
                height: Some(24.),
                ..Style::default()
            });
            let mut text = Node::new(*id + 1, *id, 0, KIND_TEXT);
            text.selectable = true;
            let mut raw = Node::new(*id + 2, *id + 1, 0, KIND_RAW_TEXT);
            raw.text = Some((*content).into());
            nodes.extend([row, text, raw]);
        }
        Snapshot::new(10, 1, base, revision, nodes)
    };
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Snapshot(snapshot(0, 1, 0, 1, &[(3, "first🙂"), (6, "second")])),
            cx,
        )
    })
    .unwrap();
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    let (start, end) = root.read_with(cx, |root, _| {
        let a = &root.selectable_text_layouts[&4];
        let b = &root.selectable_text_layouts[&7];
        (
            a.bounds.origin + point(px(0.1), px(8.)),
            b.bounds.origin + b.text.position_for_utf8(6).point + point(px(0.1), px(8.)),
        )
    });
    let mut visual = VisualTestContext::from_window(window.into(), cx);
    visual.simulate_mouse_down(start, gpui::MouseButton::Left, gpui::Modifiers::none());
    visual.simulate_mouse_move(end, Some(gpui::MouseButton::Left), gpui::Modifiers::none());
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Snapshot(snapshot(1, 2, 1, 1, &[(6, "second"), (9, "third")])),
            cx,
        )
    })
    .unwrap();
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    let detached = root.read_with(cx, |root, _| root.text_selection_snapshot().unwrap());
    assert!(detached.detached);
    assert_eq!(detached.text, "first🙂\nsecond");
    let end = root.read_with(cx, |root, _| {
        let layout = &root.selectable_text_layouts[&10];
        layout.bounds.origin + layout.text.position_for_utf8(5).point + point(px(0.1), px(8.))
    });
    visual.simulate_mouse_move(end, Some(gpui::MouseButton::Left), gpui::Modifiers::none());
    visual.simulate_mouse_up(end, gpui::MouseButton::Left, gpui::Modifiers::none());
    visual.simulate_keystrokes("cmd-c");
    assert_eq!(
        cx.read_from_clipboard()
            .and_then(|item| item.text())
            .as_deref(),
        Some("first🙂\nsecond\nthird")
    );
    // A virtualized focus owner may be gone. The service still copies retained
    // bytes without depending on any surviving per-node key handler.
    cx.update_window(window.into(), |_, window, cx| {
        root.update(cx, |root, cx| {
            let module = crate::native::text::native_module();
            let module = module.native_module(module.id(), module.digest()).unwrap();
            module
                .invoke_renderer(
                    crate::native::text::native_module()
                        .command_id("copyTextSelection")
                        .unwrap(),
                    &crate::native::encode_native_request(crate::native::text::native_module().build_digest(), &()).unwrap(),
                    root,
                    window,
                    cx,
                )
                .unwrap()
                .unwrap();
        })
    })
    .unwrap();
    assert_eq!(
        cx.read_from_clipboard()
            .and_then(|item| item.text())
            .as_deref(),
        Some("first🙂\nsecond\nthird")
    );
    let (start, old_offset, edge) = root.read_with(cx, |root, _| {
        let state = &root.virtual_lists[&2];
        let layout = &root.selectable_text_layouts[&7];
        (
            layout.bounds.origin + point(px(0.1), px(8.)),
            state.scroll_px_offset_for_scrollbar(),
            state.viewport_bounds().bottom_right() - point(px(2.), px(2.)),
        )
    });
    visual.simulate_mouse_down(start, gpui::MouseButton::Left, gpui::Modifiers::none());
    visual.simulate_mouse_move(edge, Some(gpui::MouseButton::Left), gpui::Modifiers::none());
    let new_offset = root.read_with(cx, |root, _| {
        root.virtual_lists[&2].scroll_px_offset_for_scrollbar()
    });
    assert!(
        new_offset.y < old_offset.y,
        "edge drag moves the native viewport"
    );
    visual.simulate_mouse_up(edge, gpui::MouseButton::Left, gpui::Modifiers::none());
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Snapshot(snapshot(2, 3, 1, 2, &[(6, "changed"), (9, "third")])),
            cx,
        )
    })
    .unwrap();
    assert!(root.read_with(cx, |root, _| {
        root.text_selection_snapshot().unwrap().text.is_empty()
    }));
    root.update(cx, |root, cx| {
        root.apply_decoded_message(
            DecodedMessage::Snapshot(Snapshot::new(
                10,
                1,
                3,
                4,
                vec![Node::new(1, 0, 0, KIND_VIEW)],
            )),
            cx,
        )
    })
    .unwrap();
    root.read_with(cx, |root, _| {
        assert!(root.selectable_text_layouts.is_empty());
        assert!(root.document_text_clips.is_empty());
    });
}

#[gpui::test]
fn surface_selection_drag_uses_styled_unicode_geometry_and_native_copy(
    cx: &mut gpui::TestAppContext,
) {
    let window = cx.open_window(size(px(320.), px(180.)), |_, _| {
        SolidRoot::new(InMemoryAdapter::new())
    });
    let root = window.root(cx).unwrap();
    let mut a = Node::new(2, 1, 0, KIND_TEXT);
    a.selectable = true;
    let mut raw = Node::new(3, 2, 0, KIND_RAW_TEXT);
    raw.text = Some("A🙂 ".into());
    let mut styled = Node::new(4, 2, 1, KIND_TEXT);
    styled.style = Some(Style {
        color_rgba: Some(0xff0000ff),
        ..Style::default()
    });
    let mut inline = Node::new(5, 4, 0, KIND_RAW_TEXT);
    inline.text = Some("新值".into());
    let mut b = Node::new(6, 1, 1, KIND_TEXT);
    b.selectable = true;
    let mut end = Node::new(7, 6, 0, KIND_RAW_TEXT);
    end.text = Some("e\u{301} tail".into());
    let snapshot = Snapshot::new(
        7,
        3,
        0,
        1,
        vec![
            Node::new(1, 0, 0, KIND_VIEW),
            a,
            raw,
            styled,
            inline,
            b,
            end,
        ],
    );
    root.update(cx, |root, cx| {
        root.apply_payload(&snapshot.encode().unwrap(), cx)
    })
    .unwrap();
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    let (start, end) = root.read_with(cx, |root, _| {
        let a = &root.selectable_text_layouts[&2];
        let b = &root.selectable_text_layouts[&6];
        (
            a.bounds.origin + a.text.position_for_utf8(1).point + point(px(0.1), px(8.)),
            b.bounds.origin + b.text.position_for_utf8(3).point + point(px(0.1), px(8.)),
        )
    });
    let mut visual = VisualTestContext::from_window(window.into(), cx);
    visual.simulate_mouse_down(start, gpui::MouseButton::Left, gpui::Modifiers::none());
    visual.simulate_mouse_move(end, Some(gpui::MouseButton::Left), gpui::Modifiers::none());
    visual.simulate_mouse_up(end, gpui::MouseButton::Left, gpui::Modifiers::none());
    visual.simulate_keystrokes("cmd-c");
    assert_eq!(
        cx.read_from_clipboard()
            .and_then(|item| item.text())
            .as_deref(),
        Some("🙂 新值\ne\u{301}")
    );
    let selected = root.read_with(cx, |root, _| root.text_selection_snapshot());
    assert_eq!(selected.unwrap().spans.len(), 2);
    visual.simulate_keystrokes("shift-right cmd-c");
    assert_eq!(
        cx.read_from_clipboard()
            .and_then(|item| item.text())
            .as_deref(),
        Some("🙂 新值\ne\u{301} ")
    );
}
