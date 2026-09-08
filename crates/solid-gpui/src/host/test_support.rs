use super::*;
use crate::*;
use gpui::{SharedString, TestAppContext, TextRun, VisualTestContext, font, px};
use std::sync::Arc;
fn styled_view(id: u32, parent_id: u32, index: u32, style: Style) -> Node {
    let mut node = Node::new(id, parent_id, index, KIND_VIEW);
    node.listener_id = id;
    node.style = Some(style);
    node
}

fn command(
    request_id: u32,
    kind: u32,
    node_id: u32,
    payload: Option<(u32, u32)>,
    title: Option<&str>,
    body: Option<&str>,
    menus: Option<Vec<MenuDefinition>>,
) -> Command {
    let kind = CommandKind::try_from(kind).expect("known command kind");
    let text = || title.unwrap_or_default().to_owned();
    let operation = match kind {
        CommandKind::Focus => CommandOperation::Focus,
        CommandKind::Blur => CommandOperation::Blur,
        CommandKind::SetSelection => {
            let (start, end) = payload.expect("selection payload");
            CommandOperation::SetSelection { start, end }
        }
        CommandKind::ScrollToIndex => {
            let (index, alignment) = payload.expect("scroll-to-index payload");
            CommandOperation::ScrollToIndex { index, alignment }
        }
        CommandKind::ScrollToEnd => CommandOperation::ScrollToEnd,
        CommandKind::SetTitle => CommandOperation::SetTitle { title: text() },
        CommandKind::ResizeWindow => {
            let (width, height) = payload.expect("resize payload");
            CommandOperation::ResizeWindow { width, height }
        }
        CommandKind::ZoomWindow => CommandOperation::ZoomWindow,
        CommandKind::ToggleFullscreen => CommandOperation::ToggleFullscreen,
        CommandKind::OpenUrl => CommandOperation::OpenUrl { url: text() },
        CommandKind::FocusNext => CommandOperation::FocusNext,
        CommandKind::FocusPrev => CommandOperation::FocusPrev,
        CommandKind::GetWindowSize => CommandOperation::GetWindowSize,
        CommandKind::GetFocus => CommandOperation::GetFocus,
        CommandKind::ClipboardWrite => CommandOperation::ClipboardWrite { text: text() },
        CommandKind::ClipboardRead => CommandOperation::ClipboardRead,
        CommandKind::OpenSurface => {
            let (width, height) = payload.expect("open-surface dimensions");
            CommandOperation::OpenSurface {
                title: text(),
                width,
                height,
                options: None,
            }
        }
        CommandKind::FileDialogOpen => {
            let (directories, multiple) = payload.expect("file-dialog-open flags");
            CommandOperation::FileDialogOpen {
                title: text(),
                directories: directories != 0,
                multiple: multiple != 0,
            }
        }
        CommandKind::FileDialogSave => CommandOperation::FileDialogSave {
            default_name: text(),
        },
        CommandKind::ShowNotification => CommandOperation::ShowNotification {
            title: text(),
            body: body.unwrap_or_default().to_owned(),
            actions: None,
        },
        CommandKind::SetMenus => CommandOperation::SetMenus {
            menus: menus.unwrap_or_default(),
        },
        CommandKind::SetKeybindings => CommandOperation::SetKeybindings {
            bindings: Vec::new(),
        },
        CommandKind::SetClosePolicy => CommandOperation::SetClosePolicy { policy: text() },
        CommandKind::ResolveCloseRequest => {
            let (request_id, allow) = payload.expect("close-resolution payload");
            CommandOperation::ResolveCloseRequest {
                request_id,
                allow: allow != 0,
            }
        }
        CommandKind::ReadTextFile => CommandOperation::ReadTextFile { path: text() },
        CommandKind::WriteTextFile => CommandOperation::WriteTextFile {
            path: text(),
            content: body.unwrap_or_default().to_owned(),
        },
        CommandKind::ClipboardWriteImage => CommandOperation::ClipboardWriteImage {
            image: ClipboardImage {
                format: 1,
                bytes: vec![0x89, 0x50, 0x4e, 0x47],
            },
        },
        CommandKind::ClipboardReadImage => CommandOperation::ClipboardReadImage,
        CommandKind::LoadFont => CommandOperation::LoadFont { path: text() },
        CommandKind::MinimizeWindow => CommandOperation::MinimizeWindow,
        CommandKind::GetWindowBounds => CommandOperation::GetWindowBounds,
        CommandKind::GetWindowState => CommandOperation::GetWindowState,
        CommandKind::ActivateWindow => CommandOperation::ActivateWindow,
        CommandKind::GetScrollOffset => CommandOperation::GetScrollOffset,
        CommandKind::ScrollToOffset => CommandOperation::ScrollToOffset {
            offset: payload
                .map(|(offset, _)| f32::from_bits(offset))
                .unwrap_or_default(),
        },
        CommandKind::CancelNative => CommandOperation::CancelNative {
            request_id: payload.unwrap().0,
        },
        CommandKind::OpenPopup | CommandKind::ClosePopup => {
            panic!("construct popup requests with explicit anchors and identity")
        }
        CommandKind::ConfigureApplication => {
            panic!("application controls use application-scoped metadata")
        }
        CommandKind::InvokeNative => {
            panic!("construct native invocation fixtures with their module's typed contract")
        }
    };
    Command::new(
        CommandMeta {
            surface_id: 1,
            epoch: 1,
            after_revision: 1,
            request_id,
            node_id,
        },
        operation,
    )
}

fn keybinding_command(
    surface_id: u32,
    request_id: u32,
    bindings: Vec<KeybindingDefinition>,
) -> Command {
    let mut command = command(
        request_id,
        crate::COMMAND_SET_KEYBINDINGS,
        1,
        None,
        None,
        None,
        None,
    );
    command.meta.surface_id = surface_id;
    command.operation = CommandOperation::SetKeybindings { bindings };
    command
}

fn notification_command(
    request_id: u32,
    title: &str,
    body: &str,
    actions: Vec<NotificationActionDefinition>,
) -> Command {
    let mut command = command(
        request_id,
        crate::COMMAND_SHOW_NOTIFICATION,
        1,
        None,
        Some(title),
        Some(body),
        None,
    );
    command.operation = CommandOperation::ShowNotification {
        title: title.to_owned(),
        body: body.to_owned(),
        actions: Some(actions),
    };
    command
}

fn snapshot() -> Snapshot {
    snapshot_for(1)
}

fn snapshot_for(surface_id: u32) -> Snapshot {
    let mut view = Node::new(2, 1, 0, KIND_VIEW);
    view.listener_id = 10;
    view.focusable = true;

    let mut input = Node::new(3, 1, 1, KIND_TEXT_INPUT);
    input.listener_id = 11;
    input.host_properties = Some(HostProperties::TextInput(TextInputProperties {
        value: "hello".to_owned(),
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

    let mut list = Node::new(4, 1, 2, KIND_VIRTUAL_LIST);
    list.listener_id = 12;
    list.style = Some(Style {
        height: Some(240.0),
        flex_shrink: Some(0.0),
        ..Style::default()
    });
    list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
        item_count: 100,
        range_start: 0,
        range_end: 10,
        estimated_item_size: 24.0,
        overscan: 2,
    }));

    Snapshot::new(
        surface_id,
        1,
        0,
        1,
        vec![Node::new(1, 0, 0, KIND_VIEW), view, input, list],
    )
}
fn font_snapshot() -> Snapshot {
    let mut text = Node::new(2, 1, 0, crate::KIND_TEXT);
    text.style = Some(crate::Style {
        font_family: Some("Tuffy".to_owned()),
        font_size: Some(24.0),
        color_rgba: Some(0x000000ff),
        ..crate::Style::default()
    });
    let mut raw = Node::new(3, 2, 0, crate::KIND_RAW_TEXT);
    raw.text = Some("Tuffy A".to_owned());
    Snapshot::new(1, 1, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), text, raw])
}

fn window_for(
    registry: &Entity<NativeStateRegistry>,
    cx: &mut TestAppContext,
    surface_id: u32,
) -> WindowHandle<SolidRoot> {
    registry.read_with(cx, |registry, _| {
        registry
            .surfaces
            .get(&surface_id)
            .expect("test surface")
            .window
            .downcast::<SolidRoot>()
            .expect("default host root type")
    })
}

fn draw_surface(
    registry: &Entity<NativeStateRegistry>,
    cx: &mut TestAppContext,
    surface_id: u32,
) -> WindowHandle<SolidRoot> {
    let window = window_for(registry, cx, surface_id);
    cx.update_window(window.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
    })
    .expect("draw test surface");
    cx.run_until_parked();
    window
}
fn advance_frame(window: WindowHandle<SolidRoot>, cx: &mut TestAppContext) {
    cx.update_window(window.into(), |_, window, cx| {
        window.simulate_next_frame(cx);
    })
    .expect("advance test frame");
    cx.run_until_parked();
}

fn route_command(
    registry: &Entity<NativeStateRegistry>,
    cx: &mut TestAppContext,
    command: Command,
) {
    let payload = command.encode().expect("encode test command");
    registry
        .update(cx, |registry, cx| registry.route_payload(&payload, cx))
        .expect("route test command");
    draw_surface(registry, cx, command.meta.surface_id);
}

fn install_notification_callback(registry: &Entity<NativeStateRegistry>, cx: &mut TestAppContext) {
    let registry = registry.downgrade();
    cx.update(|cx| {
        cx.on_system_notification_response(move |response, cx| {
            if let Some(registry) = registry.upgrade() {
                registry.update(cx, |registry, cx| {
                    registry.emit_notification_response(response, cx)
                });
            }
        });
    });
}

fn take_events(runtime: &InMemoryAdapter) -> Vec<crate::Event> {
    let mut events = Vec::new();
    while let Some(event) = runtime.take_event().expect("read test event") {
        events.push(event);
    }
    events
}

fn command_result(events: &[crate::Event], request_id: u32) -> crate::CommandResult {
    events
        .iter()
        .find_map(|event| match &event.payload {
            EventPayload::CommandResult(result) if result.request_id == request_id => {
                Some(result.clone())
            }
            _ => None,
        })
        .expect("command result event")
}
fn gallery_scroll_snapshot() -> Snapshot {
    let header = styled_view(
        2,
        1,
        0,
        Style {
            height: Some(60.0),
            flex_shrink: Some(0.0),
            ..Style::default()
        },
    );
    let body = styled_view(
        3,
        1,
        1,
        Style {
            flex_direction: Some(FlexDirectionCode::Row),
            flex_grow: Some(1.0),
            min_width: Some(0.0),
            min_height: Some(0.0),
            overflow: Some(OverflowCode::Hidden),
            ..Style::default()
        },
    );
    let sidebar = styled_view(
        4,
        3,
        0,
        Style {
            width: Some(260.0),
            flex_shrink: Some(0.0),
            min_width: Some(0.0),
            padding: Some(12.0),
            gap: Some(16.0),
            overflow: Some(OverflowCode::Scroll),
            ..Style::default()
        },
    );
    let main = styled_view(
        5,
        3,
        1,
        Style {
            flex_grow: Some(1.0),
            flex_shrink: Some(1.0),
            min_width: Some(0.0),
            min_height: Some(0.0),
            padding: Some(24.0),
            overflow: Some(OverflowCode::Scroll),
            ..Style::default()
        },
    );
    let inner = styled_view(
        6,
        5,
        0,
        Style {
            flex_grow: Some(1.0),
            flex_shrink: Some(1.0),
            min_width: Some(0.0),
            min_height: Some(0.0),
            ..Style::default()
        },
    );
    let category = styled_view(
        7,
        4,
        0,
        Style {
            min_width: Some(0.0),
            gap: Some(4.0),
            ..Style::default()
        },
    );
    let category_header = styled_view(
        8,
        7,
        0,
        Style {
            height: Some(20.0),
            padding: Some(4.0),
            ..Style::default()
        },
    );
    let mut nodes = vec![
        Node::new(1, 0, 0, KIND_VIEW),
        header,
        body,
        sidebar,
        main,
        inner,
        category,
        category_header,
    ];
    for index in 0..24 {
        nodes.push(styled_view(
            10 + index,
            7,
            index + 1,
            Style {
                height: Some(40.0),
                padding: Some(8.0),
                flex_direction: Some(FlexDirectionCode::Row),
                ..Style::default()
            },
        ));
        nodes.push(styled_view(
            100 + index,
            6,
            index,
            Style {
                height: Some(60.0),
                ..Style::default()
            },
        ));
    }
    Snapshot::new(1, 1, 0, 1, nodes)
}

fn layout_frame(events: &[crate::Event], node_id: u32) -> (f32, f32, f32, f32) {
    events
        .iter()
        .find(|event| {
            event.payload.event_kind() == crate::EventKind::Layout && event.meta.node_id == node_id
        })
        .and_then(|event| match &event.payload {
            EventPayload::Layout {
                x,
                y,
                width,
                height,
            } => Some((*x, *y, *width, *height)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("layout event for {node_id}; events: {events:?}"))
}

fn scroll_layout_events(
    registry: &Entity<NativeStateRegistry>,
    runtime: &InMemoryAdapter,
    window: WindowHandle<SolidRoot>,
    cx: &mut TestAppContext,
    position: gpui::Point<gpui::Pixels>,
) -> Vec<crate::Event> {
    let mut visual = VisualTestContext::from_window(window.into(), cx);
    visual.simulate_event(gpui::ScrollWheelEvent {
        position,
        delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.0), px(-120.0))),
        ..Default::default()
    });
    draw_surface(registry, cx, 1);
    advance_frame(window, cx);
    take_events(runtime)
}

pub fn nested_overflow_scroll_moves_through_host(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open nested overflow surface");
    let snapshot = gallery_scroll_snapshot()
        .encode()
        .expect("encode nested overflow snapshot");
    registry
        .update(cx, |registry, cx| registry.route_payload(&snapshot, cx))
        .expect("route nested overflow snapshot");
    let window = draw_surface(&registry, cx, 1);
    advance_frame(window, cx);
    let initial_events = take_events(&runtime);
    let sidebar_viewport = layout_frame(&initial_events, 4);
    let sidebar_content = layout_frame(&initial_events, 7);
    assert!(
        sidebar_content.3 > sidebar_viewport.3,
        "sidebar fixture must overflow vertically: content={sidebar_content:?}, viewport={sidebar_viewport:?}"
    );
    let main_viewport = layout_frame(&initial_events, 5);
    let main_content = layout_frame(&initial_events, 6);
    assert!(
        main_content.3 > main_viewport.3,
        "main fixture must overflow vertically: content={main_content:?}, viewport={main_viewport:?}"
    );
    let sidebar_first = layout_frame(&initial_events, 10).1;
    let sidebar_events = scroll_layout_events(
        &registry,
        &runtime,
        window,
        cx,
        gpui::point(px(100.0), px(300.0)),
    );
    assert!(
        layout_frame(&sidebar_events, 10).1 < sidebar_first,
        "host wheel should move sidebar content"
    );
    let main_first = layout_frame(&initial_events, 100).1;
    let main_events = scroll_layout_events(
        &registry,
        &runtime,
        window,
        cx,
        gpui::point(px(600.0), px(300.0)),
    );
    assert!(
        layout_frame(&main_events, 100).1 < main_first,
        "host wheel should move main content"
    );
}

pub fn startup_command_order_roundtrip(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open startup surface");
    let title = command(
        1,
        COMMAND_SET_TITLE,
        1,
        None,
        Some("Startup title"),
        None,
        None,
    );
    // Bootstrap messages can share one foreground batch before any frame draws.
    let payloads = [
        snapshot().encode().unwrap(),
        title.encode().unwrap(),
        command(3, COMMAND_FOCUS, 2, None, None, None, None)
            .encode()
            .unwrap(),
        command(4, COMMAND_GET_FOCUS, 2, None, None, None, None)
            .encode()
            .unwrap(),
        Patch::new(1, 1, 1, 2, Vec::new()).encode().unwrap(),
    ];
    registry.update(cx, |registry, cx| {
        for payload in payloads {
            registry.route_payload(&payload, cx).unwrap();
        }
    });
    draw_surface(&registry, cx, 1);
    let events = take_events(&runtime);
    assert!(
        command_result(&events, 1).success,
        "a later patch must not overtake an admitted title command: {events:?}"
    );
    assert!(command_result(&events, 3).success);
    assert_eq!(
        command_result(&events, 4).value,
        Some(CommandValue::Bool(true))
    );

    let mut stale_title = title;
    stale_title.meta.request_id = 2;
    route_command(&registry, cx, stale_title);
    let events = take_events(&runtime);
    assert!(!command_result(&events, 2).success);
}

pub fn command_roundtrip(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let mut mapped = WindowOptions::default();
    NativeStateRegistry::apply_window_open_options(
        &mut mapped,
        Some(&WindowOpenOptions {
            kind: Some(1),
            resizable: Some(false),
            min_size: Some((320, 240)),
        }),
    )
    .expect("map OpenSurface options");
    assert!(matches!(mapped.kind, WindowKind::Floating));
    assert!(!mapped.is_resizable);
    assert_eq!(mapped.window_min_size, Some(size(px(320.0), px(240.0))));
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open initial test surface");
    let window = draw_surface(&registry, cx, 1);

    install_notification_callback(&registry, cx);
    let action_registry = registry.downgrade();
    cx.update(|cx| {
        cx.on_action(move |action: &MenuAction, cx| {
            if let Some(registry) = action_registry.upgrade() {
                registry.update(cx, |registry, cx| {
                    registry.emit_action(action.name.clone(), cx)
                });
            }
        });
    });

    let snapshot_payload = snapshot().encode().expect("encode test snapshot");
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&snapshot_payload, cx)
        })
        .expect("route test snapshot");
    draw_surface(&registry, cx, 1);
    advance_frame(window, cx);
    assert!(registry.read_with(cx, |registry, _| {
        registry.surfaces.get(&1).is_some_and(|surface| {
            surface
                .root
                .read_with(cx, |root, _| root.store().get(4).is_some())
        })
    }));

    route_command(
        &registry,
        cx,
        command(1, COMMAND_FOCUS, 2, None, None, None, None),
    );
    let events = take_events(&runtime);
    assert!(command_result(&events, 1).success);
    for event_type in [
        crate::EVENT_WINDOW_RESIZE,
        crate::EVENT_WINDOW_ACTIVATION,
        crate::EVENT_WINDOW_APPEARANCE,
        crate::protocol::EVENT_LAYOUT,
    ] {
        assert!(
            events
                .iter()
                .any(|event| u32::from(event.event_kind()) == event_type),
            "initial observer event {event_type}"
        );
    }

    route_command(
        &registry,
        cx,
        command(2, COMMAND_GET_FOCUS, 2, None, None, None, None),
    );
    let events = take_events(&runtime);
    assert_eq!(
        command_result(&events, 2).value,
        Some(crate::CommandValue::Bool(true))
    );

    route_command(
        &registry,
        cx,
        command(3, COMMAND_FOCUS, 3, None, None, None, None),
    );
    let events = take_events(&runtime);
    assert!(
        command_result(&events, 3).success,
        "focus command result: {:?}",
        command_result(&events, 3)
    );
    assert!(
        events
            .iter()
            .any(|event| { u32::from(event.event_kind()) == crate::EVENT_FOCUS })
    );
    route_command(
        &registry,
        cx,
        command(4, COMMAND_SET_SELECTION, 3, Some((1, 4)), None, None, None),
    );
    let events = take_events(&runtime);
    assert!(command_result(&events, 4).success);
    assert!(events.iter().any(|event| {
        u32::from(event.event_kind()) == crate::EVENT_SELECTION
            && matches!(
                &event.payload,
                EventPayload::TextInputSelection(input)
                    if input.selection_start == 1 && input.selection_end == 4
            )
    }));

    route_command(
        &registry,
        cx,
        command(20, COMMAND_BLUR, 3, None, None, None, None),
    );
    let events = take_events(&runtime);
    assert!(command_result(&events, 20).success);
    assert!(
        events
            .iter()
            .any(|event| { u32::from(event.event_kind()) == crate::EVENT_BLUR })
    );
    route_command(
        &registry,
        cx,
        command(21, COMMAND_GET_FOCUS, 3, None, None, None, None),
    );
    assert_eq!(
        command_result(&take_events(&runtime), 21).value,
        Some(crate::CommandValue::Bool(false))
    );
    route_command(
        &registry,
        cx,
        command(
            5,
            COMMAND_CLIPBOARD_WRITE,
            1,
            None,
            Some("copied"),
            None,
            None,
        ),
    );
    assert!(command_result(&take_events(&runtime), 5).success);
    route_command(
        &registry,
        cx,
        command(6, COMMAND_CLIPBOARD_READ, 1, None, None, None, None),
    );
    assert_eq!(
        command_result(&take_events(&runtime), 6).value,
        Some(crate::CommandValue::Text("copied".to_owned()))
    );
    assert_eq!(
        cx.read_from_clipboard()
            .and_then(|item| item.text())
            .as_deref(),
        Some("copied")
    );

    route_command(
        &registry,
        cx,
        command(22, COMMAND_CLIPBOARD_WRITE_IMAGE, 1, None, None, None, None),
    );
    let image_events = take_events(&runtime);
    let image_result = command_result(&image_events, 22);
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        assert!(
            image_result.success,
            "native image clipboard write should succeed"
        );
    } else {
        assert_eq!(
            image_result.error.as_deref(),
            Some(
                "clipboard image command is unsupported on this platform; use text clipboard commands or run on macOS/Windows"
            )
        );
    }
    route_command(
        &registry,
        cx,
        command(23, COMMAND_CLIPBOARD_READ_IMAGE, 1, None, None, None, None),
    );
    let image_read_result = command_result(&take_events(&runtime), 23);
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        assert_eq!(
            image_read_result.value,
            Some(crate::CommandValue::Image(ClipboardImage {
                format: 1,
                bytes: vec![0x89, 0x50, 0x4e, 0x47],
            }))
        );
    } else {
        assert_eq!(
            image_read_result.error.as_deref(),
            Some(
                "clipboard image command is unsupported on this platform; use text clipboard commands or run on macOS/Windows"
            )
        );
    }

    route_command(
        &registry,
        cx,
        command(
            7,
            COMMAND_SCROLL_TO_INDEX,
            4,
            Some((42, 0)),
            None,
            None,
            None,
        ),
    );
    advance_frame(window, cx);
    let events = take_events(&runtime);
    assert!(command_result(&events, 7).success);
    assert!(
        events
            .iter()
            .any(|event| u32::from(event.event_kind()) == crate::EVENT_VISIBLE_RANGE)
    );
    route_command(
        &registry,
        cx,
        command(8, COMMAND_SCROLL_TO_END, 4, None, None, None, None),
    );
    assert!(command_result(&take_events(&runtime), 8).success);

    route_command(
        &registry,
        cx,
        command(9, COMMAND_SET_TITLE, 1, None, Some("Headless"), None, None),
    );
    assert!(command_result(&take_events(&runtime), 9).success);
    route_command(
        &registry,
        cx,
        command(10, COMMAND_GET_WINDOW_SIZE, 1, None, None, None, None),
    );
    assert!(matches!(
        command_result(&take_events(&runtime), 10).value,
        Some(crate::CommandValue::Pair((width, height)))
            if width > 0.0 && height > 0.0
    ));
    route_command(
        &registry,
        cx,
        command(24, COMMAND_GET_WINDOW_BOUNDS, 1, None, None, None, None),
    );
    let bounds = command_result(&take_events(&runtime), 24);
    assert!(bounds.success, "getWindowBounds result: {:?}", bounds.error);
    assert!(matches!(
        bounds.value,
        Some(crate::CommandValue::Bounds((x, y, width, height)))
            if x.is_finite() && y.is_finite() && width > 0.0 && height > 0.0
    ));
    route_command(
        &registry,
        cx,
        command(25, COMMAND_GET_WINDOW_STATE, 1, None, None, None, None),
    );
    let state = command_result(&take_events(&runtime), 25);
    assert!(state.success, "getWindowState result: {:?}", state.error);
    assert_eq!(
        state.value,
        Some(crate::CommandValue::WindowState((false, false)))
    );
    let invalid_minimize = command(26, COMMAND_MINIMIZE_WINDOW, 2, None, None, None, None);
    assert!(
        invalid_minimize.encode().is_err(),
        "minimizeWindow must reject non-root node IDs"
    );
    route_command(
        &registry,
        cx,
        command(27, COMMAND_ACTIVATE_WINDOW, 1, None, None, None, None),
    );
    assert!(command_result(&take_events(&runtime), 27).success);

    for (request_id, kind, payload) in [
        (11, COMMAND_RESIZE_WINDOW, Some((640, 480))),
        (13, COMMAND_TOGGLE_FULLSCREEN, None),
    ] {
        route_command(
            &registry,
            cx,
            command(request_id, kind, 1, payload, None, None, None),
        );
        assert!(command_result(&take_events(&runtime), request_id).success);
    }
    route_command(
        &registry,
        cx,
        command(
            14,
            COMMAND_OPEN_URL,
            1,
            None,
            Some("https://example.test"),
            None,
            None,
        ),
    );
    assert!(command_result(&take_events(&runtime), 14).success);
    assert_eq!(cx.opened_url().as_deref(), Some("https://example.test"));

    for (request_id, kind) in [(15, COMMAND_FOCUS_NEXT), (16, COMMAND_FOCUS_PREV)] {
        route_command(
            &registry,
            cx,
            command(request_id, kind, 1, None, None, None, None),
        );
        assert!(command_result(&take_events(&runtime), request_id).success);
    }

    route_command(
        &registry,
        cx,
        command(
            17,
            COMMAND_SET_MENUS,
            1,
            None,
            None,
            None,
            Some(vec![MenuDefinition {
                title: "Test".to_owned(),
                items: vec![MenuItemDefinition::Action {
                    name: "test-action".to_owned(),
                    disabled: false,
                    checked: false,
                }],
            }]),
        ),
    );
    assert!(command_result(&take_events(&runtime), 17).success);
    cx.update_window(window.into(), |_, window, _| window.activate_window())
        .expect("activate menu test window");
    cx.dispatch_action(
        window.into(),
        MenuAction {
            name: "test-action".to_owned(),
        },
    );
    cx.run_until_parked();
    assert!(take_events(&runtime).iter().any(|event| {
        u32::from(event.event_kind()) == crate::EVENT_ACTION
            && matches!(
                &event.payload,
                EventPayload::EventAction { action } if action == "test-action"
            )
    }));

    cx.update(|cx| {
        cx.set_app_identity("com.example.solid-gpui", "Solid GPUI");
    });
    route_command(
        &registry,
        cx,
        command(
            18,
            COMMAND_SHOW_NOTIFICATION,
            1,
            None,
            Some("Title"),
            Some("Body"),
            None,
        ),
    );
    assert!(command_result(&take_events(&runtime), 18).success);
    assert_eq!(cx.shown_system_notifications().len(), 1);

    let mut open_surface = command(
        19,
        COMMAND_OPEN_SURFACE,
        1,
        Some((320, 240)),
        Some("Aux"),
        None,
        None,
    );
    if let CommandOperation::OpenSurface { options, .. } = &mut open_surface.operation {
        *options = Some(WindowOpenOptions {
            kind: Some(1),
            resizable: Some(false),
            min_size: Some((160, 120)),
        });
    }
    route_command(&registry, cx, open_surface);
    let events = take_events(&runtime);
    assert!(command_result(&events, 19).success);
    assert_eq!(
        registry.read_with(cx, |registry, _| registry.surfaces.len()),
        2
    );
    let auxiliary_snapshot = snapshot_for(2).encode().expect("encode auxiliary snapshot");
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&auxiliary_snapshot, cx)
        })
        .expect("route auxiliary snapshot");
    let auxiliary_window = draw_surface(&registry, cx, 2);
    let closed_window_id = auxiliary_window.window_id();
    registry.update(cx, |registry, cx| {
        assert!(!registry.window_closed(closed_window_id, cx));
    });
    assert!(take_events(&runtime).iter().any(|event| {
        u32::from(event.event_kind()) == crate::EVENT_SURFACE_CLOSED && event.meta.surface_id == 2
    }));
}
pub fn virtual_list_scroll_offset_roundtrip(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open virtual list offset surface");
    let window = window_for(&registry, cx, 1);
    let mut list = Node::new(2, 1, 0, KIND_VIRTUAL_LIST);
    list.listener_id = 12;
    list.style = Some(crate::Style {
        height: Some(100.0),
        ..crate::Style::default()
    });
    list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
        item_count: 100,
        range_start: 0,
        range_end: 10,
        estimated_item_size: 24.0,
        overscan: 2,
    }));
    let snapshot = Snapshot::new(1, 1, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), list]);
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&snapshot.encode().expect("encode offset snapshot"), cx)
        })
        .expect("route offset snapshot");
    draw_surface(&registry, cx, 1);
    take_events(&runtime);

    let mut visual = VisualTestContext::from_window(window.into(), cx);
    let wheel = || gpui::ScrollWheelEvent {
        position: gpui::point(px(60.0), px(50.0)),
        delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.0), px(-48.0))),
        ..Default::default()
    };
    visual.simulate_event(wheel());
    route_command(
        &registry,
        cx,
        command(1, COMMAND_GET_SCROLL_OFFSET, 2, None, None, None, None),
    );
    let first_offset = match command_result(&take_events(&runtime), 1).value {
        Some(crate::CommandValue::ScrollOffset(offset)) => offset,
        value => panic!("unexpected first offset result: {value:?}"),
    };
    assert!(first_offset > 0.0, "wheel should move the list");

    visual.simulate_event(wheel());
    route_command(
        &registry,
        cx,
        command(2, COMMAND_GET_SCROLL_OFFSET, 2, None, None, None, None),
    );
    let second_offset = match command_result(&take_events(&runtime), 2).value {
        Some(crate::CommandValue::ScrollOffset(offset)) => offset,
        value => panic!("unexpected second offset result: {value:?}"),
    };
    assert!(
        second_offset > first_offset,
        "wheel offset should be monotonic"
    );

    let mut set_offset = command(3, COMMAND_SCROLL_TO_OFFSET, 2, None, None, None, None);
    set_offset.operation = CommandOperation::ScrollToOffset { offset: 42.5 };
    route_command(&registry, cx, set_offset);
    assert!(command_result(&take_events(&runtime), 3).success);
    route_command(
        &registry,
        cx,
        command(4, COMMAND_GET_SCROLL_OFFSET, 2, None, None, None, None),
    );
    assert!(
        matches!(command_result(&take_events(&runtime), 4).value, Some(crate::CommandValue::ScrollOffset(offset)) if (offset - 42.5).abs() < 0.01)
    );
}

pub fn cross_surface_focus_blur_roundtrip(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open initial focus test surface");
    let first_snapshot = snapshot_for(1)
        .encode()
        .expect("encode first focus snapshot");
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&first_snapshot, cx)
        })
        .expect("route first focus snapshot");
    let first_window = draw_surface(&registry, cx, 1);

    let mut open_surface = command(
        1,
        COMMAND_OPEN_SURFACE,
        1,
        Some((320, 240)),
        Some("Focus auxiliary"),
        None,
        None,
    );
    if let CommandOperation::OpenSurface { options, .. } = &mut open_surface.operation {
        *options = Some(WindowOpenOptions {
            kind: Some(1),
            resizable: Some(false),
            min_size: None,
        });
    }
    route_command(&registry, cx, open_surface);
    assert!(command_result(&take_events(&runtime), 1).success);

    let second_snapshot = snapshot_for(2)
        .encode()
        .expect("encode second focus snapshot");
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&second_snapshot, cx)
        })
        .expect("route second focus snapshot");
    let second_window = draw_surface(&registry, cx, 2);
    cx.update_window(first_window.into(), |_, window, _| window.activate_window())
        .expect("activate first focus surface");
    cx.run_until_parked();
    route_command(
        &registry,
        cx,
        command(2, COMMAND_FOCUS, 2, None, None, None, None),
    );
    let first_focus_events = take_events(&runtime);
    assert!(first_focus_events.iter().any(|event| {
        u32::from(event.event_kind()) == crate::EVENT_FOCUS && event.meta.surface_id == 1
    }));

    cx.update_window(second_window.into(), |_, window, _| {
        window.activate_window()
    })
    .expect("activate second focus surface");
    cx.run_until_parked();
    draw_surface(&registry, cx, 1);
    let blur_events = take_events(&runtime);
    assert!(blur_events.iter().any(|event| {
        u32::from(event.event_kind()) == crate::EVENT_BLUR && event.meta.surface_id == 1
    }));

    let mut second_focus = command(3, COMMAND_FOCUS, 2, None, None, None, None);
    second_focus.meta.surface_id = 2;
    route_command(&registry, cx, second_focus);
    let second_focus_events = take_events(&runtime);
    assert!(
        second_focus_events
            .iter()
            .any(|event| u32::from(event.event_kind()) == crate::EVENT_FOCUS
                && event.meta.surface_id == 2),
        "auxiliary focus events: {second_focus_events:?}"
    );
    let first_focus = first_focus_events
        .iter()
        .find(|event| u32::from(event.event_kind()) == crate::EVENT_FOCUS)
        .expect("first focus event");
    let second_focus = second_focus_events
        .iter()
        .find(|event| u32::from(event.event_kind()) == crate::EVENT_FOCUS)
        .expect("second focus event");
    assert_ne!(first_focus.meta.surface_id, second_focus.meta.surface_id);
}
fn focus_node(id: u32, parent_id: u32, index: u32, kind: u32, listener_id: u32) -> Node {
    let mut node = Node::new(id, parent_id, index, kind);
    node.listener_id = listener_id;
    node.focusable = true;
    node.style = Some(crate::Style {
        flex_direction: Some(crate::FlexDirectionCode::Row),
        width: Some(100.0),
        height: Some(30.0),
        ..Default::default()
    });
    node
}

fn focus_surface_snapshot(nodes: Vec<Node>) -> Snapshot {
    Snapshot::new(1, 1, 0, 1, nodes)
}

fn open_focus_surface(
    cx: &mut TestAppContext,
    nodes: Vec<Node>,
) -> (
    Entity<NativeStateRegistry>,
    WindowHandle<SolidRoot>,
    Arc<InMemoryAdapter>,
) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open focus test surface");
    let payload = focus_surface_snapshot(nodes)
        .encode()
        .expect("encode focus snapshot");
    registry
        .update(cx, |registry, cx| registry.route_payload(&payload, cx))
        .expect("route focus snapshot");
    let window = draw_surface(&registry, cx, 1);
    cx.update_window(window.into(), |_, window, _| window.activate_window())
        .expect("activate focus test surface");
    cx.run_until_parked();
    take_events(&runtime);
    (registry, window, runtime)
}

pub fn focus_traversal_roundtrip(cx: &mut TestAppContext) {
    let (registry, window, runtime) = open_focus_surface(
        cx,
        vec![
            Node::new(1, 0, 0, KIND_VIEW),
            focus_node(2, 1, 0, KIND_VIEW, 20),
            focus_node(3, 1, 1, KIND_VIEW, 30),
            focus_node(4, 1, 2, KIND_VIEW, 40),
        ],
    );
    route_command(
        &registry,
        cx,
        command(1, COMMAND_FOCUS, 2, None, None, None, None),
    );
    let _ = take_events(&runtime);
    draw_surface(&registry, cx, 1);
    for (request_id, kind, expected) in [
        (2, COMMAND_FOCUS_NEXT, 3),
        (3, COMMAND_FOCUS_NEXT, 4),
        (4, COMMAND_FOCUS_NEXT, 2),
        (5, COMMAND_FOCUS_PREV, 4),
    ] {
        route_command(
            &registry,
            cx,
            command(request_id, kind, 1, None, None, None, None),
        );
        let mut events = take_events(&runtime);
        for _ in 0..4 {
            if events
                .iter()
                .any(|event| u32::from(event.event_kind()) == crate::EVENT_FOCUS)
            {
                break;
            }
            advance_frame(window, cx);
            events.extend(take_events(&runtime));
        }
        assert!(command_result(&events, request_id).success);
        assert!(
            events
                .iter()
                .any(|event| u32::from(event.event_kind()) == crate::EVENT_FOCUS
                    && event.meta.node_id == expected),
            "focus traversal request {request_id} did not focus node {expected}: {events:?}"
        );
    }
}
pub fn disabled_pressable_is_skipped_roundtrip(cx: &mut TestAppContext) {
    let (registry, window, runtime) = open_focus_surface(
        cx,
        vec![
            Node::new(1, 0, 0, KIND_VIEW),
            focus_node(2, 1, 0, KIND_VIEW, 20),
            Node::new(3, 1, 1, KIND_PRESSABLE),
            focus_node(4, 1, 2, KIND_VIEW, 40),
        ],
    );
    route_command(
        &registry,
        cx,
        command(1, COMMAND_FOCUS, 2, None, None, None, None),
    );
    let _ = take_events(&runtime);
    draw_surface(&registry, cx, 1);
    route_command(
        &registry,
        cx,
        command(2, COMMAND_FOCUS_NEXT, 1, None, None, None, None),
    );
    let mut events = take_events(&runtime);
    for _ in 0..4 {
        if events
            .iter()
            .any(|event| u32::from(event.event_kind()) == crate::EVENT_FOCUS)
        {
            break;
        }
        advance_frame(window, cx);
        events.extend(take_events(&runtime));
    }
    assert!(command_result(&events, 2).success);
    assert!(events.iter().any(|event| {
        u32::from(event.event_kind()) == crate::EVENT_FOCUS && event.meta.node_id == 4
    }));
    assert!(!events.iter().any(|event| event.meta.node_id == 3));
}

pub fn conditional_focus_mount_keeps_tree_order(cx: &mut TestAppContext) {
    let (registry, _window, runtime) = open_focus_surface(
        cx,
        vec![
            Node::new(1, 0, 0, KIND_VIEW),
            focus_node(2, 1, 0, KIND_VIEW, 20),
            focus_node(4, 1, 1, KIND_VIEW, 40),
        ],
    );
    route_command(
        &registry,
        cx,
        command(1, COMMAND_FOCUS, 2, None, None, None, None),
    );
    take_events(&runtime);
    let patch = Patch::new(
        1,
        1,
        1,
        2,
        vec![PatchOperation::Create(focus_node(3, 1, 1, KIND_VIEW, 30))],
    )
    .encode()
    .expect("encode conditional focus patch");
    registry
        .update(cx, |registry, cx| registry.route_payload(&patch, cx))
        .expect("route conditional focus patch");
    draw_surface(&registry, cx, 1);
    take_events(&runtime);
    let mut next = command(2, COMMAND_FOCUS_NEXT, 1, None, None, None, None);
    next.meta.after_revision = 2;
    route_command(&registry, cx, next);
    let events = take_events(&runtime);
    assert!(command_result(&events, 2).success);
    assert!(events.iter().any(|event| {
        u32::from(event.event_kind()) == crate::EVENT_FOCUS && event.meta.node_id == 3
    }));
}

pub fn focused_unmount_blurs_and_restores_ancestor(cx: &mut TestAppContext) {
    let (registry, window, runtime) = open_focus_surface(
        cx,
        vec![
            Node::new(1, 0, 0, KIND_VIEW),
            focus_node(2, 1, 0, KIND_VIEW, 20),
            focus_node(3, 2, 0, KIND_PRESSABLE, 30),
        ],
    );
    route_command(
        &registry,
        cx,
        command(1, COMMAND_FOCUS, 3, None, None, None, None),
    );
    take_events(&runtime);
    let patch = Patch::new(1, 1, 1, 2, vec![PatchOperation::Delete { id: 3 }])
        .encode()
        .expect("encode focus unmount patch");
    registry
        .update(cx, |registry, cx| registry.route_payload(&patch, cx))
        .expect("route focus unmount patch");
    draw_surface(&registry, cx, 1);
    advance_frame(window, cx);
    let events = take_events(&runtime);
    assert!(
        events.iter().any(|event| {
            u32::from(event.event_kind()) == crate::EVENT_BLUR && event.meta.node_id == 3
        }),
        "unmounted focus blur missing: {events:?}"
    );
    assert!(
        events.iter().any(|event| {
            u32::from(event.event_kind()) == crate::EVENT_FOCUS && event.meta.node_id == 2
        }),
        "focus was not restored to ancestor: {events:?}"
    );
}
pub fn pressable_focusable_update_roundtrip(cx: &mut TestAppContext) {
    let (registry, window, runtime) = open_focus_surface(
        cx,
        vec![Node::new(1, 0, 0, KIND_VIEW), {
            let mut pressable = Node::new(2, 1, 0, KIND_PRESSABLE);
            pressable.listener_id = 21;
            pressable
        }],
    );
    let patch = Patch::new(
        1,
        1,
        1,
        2,
        vec![PatchOperation::Update {
            id: 2,
            mask: crate::protocol::UPDATE_FOCUSABLE,
            style: None,
            text: None,
            listener_id: 21,
            host_properties: None,
            accessibility: None,
            focusable: true,
            selectable: false,
            tooltip: None,
            accepts_pointer_move: false,
        }],
    )
    .encode()
    .expect("encode Pressable focusable update");
    registry
        .update(cx, |registry, cx| registry.route_payload(&patch, cx))
        .expect("route Pressable focusable update");
    assert!(registry.read_with(cx, |registry, _| {
        registry.surfaces.get(&1).is_some_and(|surface| {
            surface.root.read_with(cx, |root, _| {
                root.store().get(2).is_some_and(|node| node.focusable)
            })
        })
    }));
    draw_surface(&registry, cx, 1);
    let mut focus = command(1, COMMAND_FOCUS, 2, None, None, None, None);
    focus.meta.after_revision = 2;
    route_command(&registry, cx, focus);
    let mut events = take_events(&runtime);
    for _ in 0..4 {
        if events.iter().any(|event| {
            u32::from(event.event_kind()) == crate::EVENT_FOCUS && event.meta.node_id == 2
        }) {
            break;
        }
        advance_frame(window, cx);
        events.extend(take_events(&runtime));
    }
    assert!(command_result(&events, 1).success);
    assert!(events.iter().any(|event| {
        u32::from(event.event_kind()) == crate::EVENT_FOCUS && event.meta.node_id == 2
    }));
}
pub fn keybinding_roundtrip(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open initial keybinding test surface");
    let window = draw_surface(&registry, cx, 1);
    let action_registry = registry.downgrade();
    cx.update(|cx| {
        cx.on_action(move |action: &MenuAction, cx| {
            if let Some(registry) = action_registry.upgrade() {
                registry.update(cx, |registry, cx| {
                    registry.emit_action(action.name.clone(), cx)
                });
            }
        });
    });

    let snapshot = Snapshot::new(1, 1, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW)]);
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&snapshot.encode().expect("encode keybinding snapshot"), cx)
        })
        .expect("route keybinding snapshot");
    draw_surface(&registry, cx, 1);

    route_command(
        &registry,
        cx,
        keybinding_command(
            1,
            1,
            vec![
                KeybindingDefinition {
                    keystrokes: "secondary-shift-p".to_owned(),
                    action_name: "palette.open".to_owned(),
                },
                KeybindingDefinition {
                    keystrokes: "ctrl-k ctrl-1".to_owned(),
                    action_name: "menu.other".to_owned(),
                },
            ],
        ),
    );
    assert!(command_result(&take_events(&runtime), 1).success);
    cx.update_window(window.into(), |_, window, _| window.activate_window())
        .expect("activate chord test surface");
    cx.run_until_parked();
    cx.simulate_keystrokes(window.into(), "ctrl-k ctrl-1");
    assert!(take_events(&runtime).iter().any(|event| {
        matches!(&event.payload, EventPayload::EventAction { action } if action == "menu.other")
    }));
    route_command(
        &registry,
        cx,
        keybinding_command(
            1,
            5,
            vec![KeybindingDefinition {
                keystrokes: "secondary-shift-p".to_owned(),
                action_name: "palette.open".to_owned(),
            }],
        ),
    );
    assert!(command_result(&take_events(&runtime), 5).success);
    cx.update_window(window.into(), |_, window, _| window.activate_window())
        .expect("activate replacement keybinding surface");
    cx.run_until_parked();
    cx.simulate_keystrokes(window.into(), "ctrl-k ctrl-1");
    assert!(
        !take_events(&runtime).iter().any(|event| {
            u32::from(event.event_kind()) == crate::EVENT_ACTION
                && matches!(
                    &event.payload,
                    EventPayload::EventAction { action } if action == "menu.other"
                )
        }),
        "replaced keybinding should be removed"
    );
    cx.update_window(window.into(), |_, window, _| window.activate_window())
        .expect("activate keybinding test surface");
    cx.run_until_parked();
    cx.simulate_keystrokes(window.into(), "secondary-shift-p");
    let events = take_events(&runtime);
    assert!(events.iter().any(|event| {
        u32::from(event.event_kind()) == crate::EVENT_ACTION
            && matches!(
                &event.payload,
                EventPayload::EventAction { action } if action == "palette.open"
            )
    }));
    assert!(
        !events
            .iter()
            .any(|event| u32::from(event.event_kind()) == crate::EVENT_KEY),
        "consumed keybinding should not emit a raw key event"
    );

    route_command(
        &registry,
        cx,
        command(
            2,
            COMMAND_OPEN_SURFACE,
            1,
            Some((320, 240)),
            Some("Aux"),
            None,
            None,
        ),
    );
    assert!(command_result(&take_events(&runtime), 2).success);
    let auxiliary_window = window_for(&registry, cx, 2);
    let mut input = Node::new(2, 1, 0, KIND_TEXT_INPUT);
    input.listener_id = 2;
    input.host_properties = Some(HostProperties::TextInput(TextInputProperties {
        value: "search".to_owned(),
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
    let auxiliary_snapshot = Snapshot::new(2, 1, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), input]);
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(
                &auxiliary_snapshot
                    .encode()
                    .expect("encode auxiliary keybinding snapshot"),
                cx,
            )
        })
        .expect("route auxiliary keybinding snapshot");
    draw_surface(&registry, cx, 2);
    route_command(
        &registry,
        cx,
        keybinding_command(
            2,
            3,
            vec![
                KeybindingDefinition {
                    keystrokes: "ctrl-alt-p".to_owned(),
                    action_name: "auxiliary.open".to_owned(),
                },
                KeybindingDefinition {
                    keystrokes: "secondary-shift-p".to_owned(),
                    action_name: "auxiliary.palette".to_owned(),
                },
            ],
        ),
    );
    assert!(command_result(&take_events(&runtime), 3).success);
    let mut focus = command(7, COMMAND_FOCUS, 2, None, None, None, None);
    focus.meta.surface_id = 2;
    route_command(&registry, cx, focus);
    assert!(command_result(&take_events(&runtime), 7).success);
    cx.update_window(auxiliary_window.into(), |_, window, _| {
        window.activate_window()
    })
    .expect("activate auxiliary keybinding surface");
    cx.run_until_parked();
    cx.simulate_keystrokes(auxiliary_window.into(), "ctrl-alt-p");
    let events = take_events(&runtime);
    assert!(events.iter().any(|event| {
        u32::from(event.event_kind()) == crate::EVENT_ACTION
            && matches!(
                &event.payload,
                EventPayload::EventAction { action } if action == "auxiliary.open"
            )
    }));

    cx.simulate_keystrokes(auxiliary_window.into(), "secondary-shift-p");
    let events = take_events(&runtime);
    assert!(events.iter().any(|event| {
        event.meta.surface_id == 2
            && matches!(&event.payload, EventPayload::EventAction { action } if action == "auxiliary.palette")
    }));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event.payload, EventPayload::Key(_))),
        "a consumed shortcut must not leak into the focused text input"
    );
    cx.update_window(window.into(), |_, window, _| window.activate_window())
        .expect("activate original surface for shortcut isolation");
    cx.run_until_parked();
    cx.simulate_keystrokes(window.into(), "ctrl-alt-p");
    assert!(
        !take_events(&runtime)
            .iter()
            .any(|event| matches!(event.payload, EventPayload::EventAction { .. })),
        "another surface's shortcut must not consume a key or emit an action here"
    );

    let invalid = keybinding_command(
        1,
        4,
        vec![KeybindingDefinition {
            keystrokes: "not-a-valid-keystroke".to_owned(),
            action_name: "invalid".to_owned(),
        }],
    );
    route_command(&registry, cx, invalid);
    let events = take_events(&runtime);
    let result = command_result(&events, 4);
    assert!(!result.success);
    assert!(
        result
            .error
            .as_deref()
            .is_some_and(|error| error.contains("invalid keystroke"))
    );
    cx.update_window(window.into(), |_, window, _| window.activate_window())
        .expect("reactivate original keybinding surface");
    cx.run_until_parked();
    cx.simulate_keystrokes(window.into(), "secondary-shift-p");
    assert!(take_events(&runtime).iter().any(|event| {
        event.meta.surface_id == 1
            && u32::from(event.event_kind()) == crate::EVENT_ACTION
            && matches!(
                &event.payload,
                EventPayload::EventAction { action } if action == "palette.open"
            )
    }));
    route_command(&registry, cx, keybinding_command(1, 6, Vec::new()));
    assert!(command_result(&take_events(&runtime), 6).success);
    cx.simulate_keystrokes(window.into(), "secondary-shift-p");
    assert!(
        !take_events(&runtime)
            .iter()
            .any(|event| u32::from(event.event_kind()) == crate::EVENT_ACTION),
        "empty replacement should clear the surface binding"
    );

    let replacement = Snapshot::new(2, 2, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW)]);
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&replacement.encode().expect("encode new surface epoch"), cx)
        })
        .expect("replace auxiliary surface epoch");
    draw_surface(&registry, cx, 2);
    cx.update_window(auxiliary_window.into(), |_, window, _| {
        window.activate_window()
    })
    .expect("activate reloaded surface");
    cx.run_until_parked();
    take_events(&runtime);
    cx.simulate_keystrokes(auxiliary_window.into(), "ctrl-alt-p secondary-shift-p");
    assert!(
        !take_events(&runtime)
            .iter()
            .any(|event| matches!(event.payload, EventPayload::EventAction { .. })),
        "a new epoch must register its own shortcuts"
    );
}

pub fn dialog_command_roundtrip(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open initial dialog test surface");
    let window = draw_surface(&registry, cx, 1);
    install_notification_callback(&registry, cx);
    let snapshot_payload = snapshot().encode().expect("encode dialog snapshot");
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&snapshot_payload, cx)
        })
        .expect("route dialog snapshot");
    draw_surface(&registry, cx, 1);

    route_command(
        &registry,
        cx,
        command(
            101,
            COMMAND_FILE_DIALOG_OPEN,
            1,
            Some((0, 1)),
            Some("Choose files"),
            None,
            None,
        ),
    );
    assert!(cx.did_prompt_for_paths());
    cx.simulate_path_prompt_response(|options| {
        assert!(options.files);
        assert!(options.multiple);
        Some(vec![
            PathBuf::from("/tmp/one.txt"),
            PathBuf::from("/tmp/two.txt"),
        ])
    });
    cx.run_until_parked();
    advance_frame(window, cx);
    let events = take_events(&runtime);
    let result = command_result(&events, 101);
    assert!(result.success);
    assert_eq!(
        result.value,
        Some(crate::CommandValue::Paths(vec![
            "/tmp/one.txt".to_owned(),
            "/tmp/two.txt".to_owned(),
        ]))
    );

    route_command(
        &registry,
        cx,
        command(
            102,
            COMMAND_FILE_DIALOG_OPEN,
            1,
            Some((0, 0)),
            Some("Choose one"),
            None,
            None,
        ),
    );
    cx.simulate_path_prompt_response(|options| {
        assert!(options.files);
        assert!(!options.multiple);
        Some(vec![PathBuf::from("/tmp/one.txt")])
    });
    cx.run_until_parked();
    advance_frame(window, cx);
    let result = command_result(&take_events(&runtime), 102);
    assert!(result.success);
    assert_eq!(
        result.value,
        Some(crate::CommandValue::Paths(vec!["/tmp/one.txt".to_owned()]))
    );

    route_command(
        &registry,
        cx,
        command(
            103,
            COMMAND_FILE_DIALOG_OPEN,
            1,
            Some((1, 0)),
            Some("Choose directory"),
            None,
            None,
        ),
    );
    cx.simulate_path_prompt_response(|options| {
        assert!(!options.files);
        assert!(!options.multiple);
        Some(vec![PathBuf::from("/tmp/directory")])
    });
    cx.run_until_parked();
    advance_frame(window, cx);
    let result = command_result(&take_events(&runtime), 103);
    assert!(result.success);
    assert_eq!(
        result.value,
        Some(crate::CommandValue::Paths(vec![
            "/tmp/directory".to_owned()
        ]))
    );

    route_command(
        &registry,
        cx,
        command(
            104,
            COMMAND_FILE_DIALOG_OPEN,
            1,
            Some((0, 0)),
            Some("Cancel"),
            None,
            None,
        ),
    );
    cx.simulate_path_prompt_response(|_| None);
    cx.run_until_parked();
    advance_frame(window, cx);
    let result = command_result(&take_events(&runtime), 104);
    assert!(result.success);
    assert!(result.value.is_none());
    #[cfg(unix)]
    {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        route_command(
            &registry,
            cx,
            command(
                109,
                COMMAND_FILE_DIALOG_OPEN,
                1,
                Some((0, 0)),
                Some("Invalid UTF-8"),
                None,
                None,
            ),
        );
        cx.simulate_path_prompt_response(|_| {
            Some(vec![PathBuf::from(OsString::from_vec(vec![0xff]))])
        });
        cx.run_until_parked();
        advance_frame(window, cx);
        let result = command_result(&take_events(&runtime), 109);
        assert!(!result.success);
        assert!(result.error.is_some_and(|error| error.contains("UTF-8")));
    }

    route_command(
        &registry,
        cx,
        command(
            105,
            COMMAND_FILE_DIALOG_SAVE,
            1,
            None,
            Some("output.txt"),
            None,
            None,
        ),
    );
    cx.simulate_new_path_selection(|_| Some(PathBuf::from("/tmp/output.txt")));
    cx.run_until_parked();
    advance_frame(window, cx);
    let result = command_result(&take_events(&runtime), 105);
    assert!(result.success);
    assert_eq!(
        result.value,
        Some(crate::CommandValue::Text("/tmp/output.txt".to_owned()))
    );

    route_command(
        &registry,
        cx,
        command(
            106,
            COMMAND_FILE_DIALOG_SAVE,
            1,
            None,
            Some("Cancel"),
            None,
            None,
        ),
    );
    cx.simulate_new_path_selection(|_| None);
    cx.run_until_parked();
    advance_frame(window, cx);
    let result = command_result(&take_events(&runtime), 106);
    assert!(result.success);
    assert!(result.value.is_none());

    let invalid = command(
        107,
        COMMAND_FILE_DIALOG_OPEN,
        2,
        Some((0, 0)),
        Some("not root"),
        None,
        None,
    );
    assert!(invalid.encode().is_err());

    let close_runtime = InMemoryAdapter::new();
    let close_registry = cx.new(|_| NativeStateRegistry::new(close_runtime.clone()));
    close_registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open close-safety test surface");
    let close_window = draw_surface(&close_registry, cx, 1);
    let close_snapshot = snapshot().encode().expect("encode close snapshot");
    close_registry
        .update(cx, |registry, cx| {
            registry.route_payload(&close_snapshot, cx)
        })
        .expect("route close snapshot");
    draw_surface(&close_registry, cx, 1);
    route_command(
        &close_registry,
        cx,
        command(
            110,
            COMMAND_FILE_DIALOG_OPEN,
            1,
            Some((0, 0)),
            Some("close me"),
            None,
            None,
        ),
    );
    assert!(cx.did_prompt_for_paths());
    let close_window_id = close_window.window_id();
    close_window
        .update(cx, |_, window, _| window.remove_window())
        .expect("remove close-safety test window");
    close_registry.update(cx, |registry, cx| {
        assert!(registry.window_closed(close_window_id, cx));
    });
    cx.simulate_path_prompt_response(|_| Some(vec![PathBuf::from("/tmp/dropped")]));
    cx.run_until_parked();
    assert!(
        take_events(&close_runtime)
            .iter()
            .all(|event| u32::from(event.event_kind()) != crate::EVENT_COMMAND_RESULT)
    );
}
pub fn text_file_command_roundtrip(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open text file test surface");
    let window = draw_surface(&registry, cx, 1);
    let snapshot_payload = snapshot().encode().expect("encode text file snapshot");
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&snapshot_payload, cx)
        })
        .expect("route text file snapshot");
    draw_surface(&registry, cx, 1);
    let dir = std::env::temp_dir().join(format!("solid-gpui-file-{}", std::process::id()));
    let path = dir.join("notes.txt");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create text file test directory");
    let path = path.to_str().expect("temporary path is UTF-8").to_owned();
    route_command(
        &registry,
        cx,
        command(
            201,
            COMMAND_WRITE_TEXT_FILE,
            1,
            None,
            Some(&path),
            Some("hello π"),
            None,
        ),
    );
    cx.run_until_parked();
    advance_frame(window, cx);
    let result = command_result(&take_events(&runtime), 201);
    assert!(result.success, "write result: {:?}", result.error);
    assert_eq!(result.value, Some(crate::CommandValue::Number(8)));
    assert_eq!(
        std::fs::read_to_string(&path).expect("read written file"),
        "hello π"
    );
    route_command(
        &registry,
        cx,
        command(
            202,
            COMMAND_READ_TEXT_FILE,
            1,
            None,
            Some(&path),
            None,
            None,
        ),
    );
    cx.run_until_parked();
    advance_frame(window, cx);
    let result = command_result(&take_events(&runtime), 202);
    assert!(result.success, "read result: {:?}", result.error);
    assert_eq!(
        result.value,
        Some(crate::CommandValue::FileText("hello π".to_owned()))
    );
    route_command(
        &registry,
        cx,
        command(
            203,
            COMMAND_READ_TEXT_FILE,
            1,
            None,
            Some(&format!("{path}.missing")),
            None,
            None,
        ),
    );
    cx.run_until_parked();
    advance_frame(window, cx);
    let result = command_result(&take_events(&runtime), 203);
    assert!(!result.success);
    assert_eq!(result.error.as_deref(), Some("file not found"));
    route_command(
        &registry,
        cx,
        command(
            204,
            COMMAND_READ_TEXT_FILE,
            1,
            None,
            Some(&dir.to_string_lossy()),
            None,
            None,
        ),
    );
    cx.run_until_parked();
    advance_frame(window, cx);
    let result = command_result(&take_events(&runtime), 204);
    assert!(!result.success);
    assert_eq!(result.error.as_deref(), Some("path is a directory"));
    let _ = std::fs::remove_dir_all(dir);
}
pub fn font_command_roundtrip(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open font test surface");
    let window = window_for(&registry, cx, 1);
    let snapshot_payload = font_snapshot().encode().expect("encode font snapshot");
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&snapshot_payload, cx)
        })
        .expect("route font snapshot");

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/host-fixtures/tuffy.ttf");
    let path = fixture
        .to_str()
        .expect("font fixture path is UTF-8")
        .to_owned();
    let load_command = command(205, COMMAND_LOAD_FONT, 1, None, Some(&path), None, None);
    let load_payload = load_command.encode().expect("encode font command");
    registry
        .update(cx, |registry, cx| registry.route_payload(&load_payload, cx))
        .expect("route font command");
    cx.run_until_parked();
    let result = command_result(&take_events(&runtime), 205);
    assert!(result.success, "font result: {:?}", result.error);
    assert_eq!(
        result.value,
        Some(crate::CommandValue::Text("Tuffy".to_owned()))
    );

    // Use the public WindowTextSystem shaping path after a retained-tree draw.
    draw_surface(&registry, cx, 1);
    // The resulting run/glyph data is the input consumed by GPUI's text painter.
    let shaped = cx
        .update_window(window.into(), |_, window, _| {
            let text = SharedString::from("Tuffy A");
            window.text_system().shape_line(
                text.clone(),
                px(24.0),
                &[TextRun {
                    len: text.len(),
                    font: font("Tuffy"),
                    color: gpui::black(),
                    ..TextRun::default()
                }],
                None,
            )
        })
        .expect("shape loaded Tuffy text in the rendered window");
    assert!(shaped.width() > px(0.0), "loaded Tuffy line has no advance");
    assert!(
        !shaped.runs.is_empty(),
        "loaded Tuffy line has no shaped runs"
    );
    assert!(
        shaped.runs.iter().any(|run| !run.glyphs.is_empty()),
        "loaded Tuffy line has no shaped glyphs"
    );
}
pub fn notification_response_roundtrip(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open notification test surface");
    let window = draw_surface(&registry, cx, 1);
    install_notification_callback(&registry, cx);
    cx.update(|cx| {
        cx.set_app_identity("com.example.solid-gpui", "Solid GPUI");
    });
    let snapshot_payload = snapshot().encode().expect("encode notification snapshot");
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&snapshot_payload, cx)
        })
        .expect("route notification snapshot");
    draw_surface(&registry, cx, 1);

    route_command(
        &registry,
        cx,
        notification_command(
            201,
            "Build",
            "Finished",
            vec![
                NotificationActionDefinition {
                    id: "open".to_owned(),
                    label: "Open".to_owned(),
                },
                NotificationActionDefinition {
                    id: "dismiss".to_owned(),
                    label: "Dismiss".to_owned(),
                },
            ],
        ),
    );
    assert!(command_result(&take_events(&runtime), 201).success);
    let notification = cx
        .shown_system_notifications()
        .last()
        .cloned()
        .expect("shown notification");
    assert_eq!(notification.actions.len(), 2);
    assert_eq!(notification.actions[0].id.as_ref(), "open");
    assert_eq!(notification.actions[0].label.as_ref(), "Open");

    cx.simulate_system_notification_response(SystemNotificationResponse {
        tag: notification.tag.clone(),
        action_id: Some("open".into()),
    });
    cx.run_until_parked();
    let response = take_events(&runtime)
        .into_iter()
        .find(|event| u32::from(event.event_kind()) == crate::EVENT_NOTIFICATION_RESPONSE)
        .expect("notification action response event");
    assert_eq!(
        response.payload,
        EventPayload::NotificationResponse(crate::NotificationResponseEvent {
            tag: notification.tag.to_string(),
            action_id: Some("open".to_owned()),
        })
    );

    cx.simulate_system_notification_response(SystemNotificationResponse {
        tag: notification.tag.clone(),
        action_id: None,
    });
    cx.run_until_parked();
    let response = take_events(&runtime)
        .into_iter()
        .find(|event| u32::from(event.event_kind()) == crate::EVENT_NOTIFICATION_RESPONSE)
        .expect("notification body response event");
    assert_eq!(
        response.payload,
        EventPayload::NotificationResponse(crate::NotificationResponseEvent {
            tag: notification.tag.to_string(),
            action_id: None,
        })
    );

    let window_id = window.window_id();
    window
        .update(cx, |_, window, _| window.remove_window())
        .expect("remove notification test surface");
    registry.update(cx, |registry, cx| {
        assert!(registry.window_closed(window_id, cx));
    });
    cx.simulate_system_notification_response(SystemNotificationResponse {
        tag: notification.tag,
        action_id: Some("open".into()),
    });
    cx.run_until_parked();
    assert!(
        take_events(&runtime)
            .iter()
            .all(|event| u32::from(event.event_kind()) != crate::EVENT_NOTIFICATION_RESPONSE)
    );
}

pub fn renderer_termination_closes_surfaces_without_reentrant_update(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open renderer termination test surface");

    let callback_called = std::rc::Rc::new(std::cell::Cell::new(false));
    let callback_called_for_close = callback_called.clone();
    let registry_for_close = registry.downgrade();
    let close_subscription = cx.update(|cx| {
        cx.on_window_closed(move |cx, window_id| {
            callback_called_for_close.set(true);
            if let Some(registry) = registry_for_close.upgrade() {
                registry.update(cx, |registry, cx| {
                    registry.window_closed(window_id, cx);
                });
            }
        })
    });
    registry.update(cx, |registry, _| {
        registry.close_subscription = Some(close_subscription);
    });

    runtime
        .close()
        .expect("mark renderer runtime terminated for close test");
    registry.update(cx, |registry, cx| registry.close_all(cx));

    assert!(!callback_called.get());
    assert!(registry.read_with(cx, |registry, _| registry.surfaces.is_empty()));
}

fn install_close_callback(registry: &Entity<NativeStateRegistry>, cx: &mut TestAppContext) {
    let registry_for_close = registry.downgrade();
    let close_subscription = cx.update(|cx| {
        cx.on_window_closed(move |cx, window_id| {
            if let Some(registry) = registry_for_close.upgrade() {
                registry.update(cx, |registry, cx| {
                    registry.window_closed(window_id, cx);
                });
            }
        })
    });
    registry.update(cx, |registry, _| {
        registry.close_subscription = Some(close_subscription);
    });
}

fn prepare_close_surface(
    cx: &mut TestAppContext,
    runtime: &Arc<InMemoryAdapter>,
    registry: &Entity<NativeStateRegistry>,
) -> WindowHandle<SolidRoot> {
    let snapshot_payload = snapshot().encode().expect("encode close-policy snapshot");
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&snapshot_payload, cx)
        })
        .expect("route close-policy snapshot");
    let window = draw_surface(registry, cx, 1);
    let _ = take_events(runtime);
    install_close_callback(registry, cx);
    window
}

fn close_policy_command(
    registry: &Entity<NativeStateRegistry>,
    cx: &mut TestAppContext,
    runtime: &InMemoryAdapter,
    request_id: u32,
    kind: u32,
    payload: Option<(u32, u32)>,
    title: Option<&str>,
) -> Vec<crate::Event> {
    let command = command(request_id, kind, 1, payload, title, None, None);
    let surface_id = command.meta.surface_id;
    let payload = command.encode().expect("encode close-policy command");
    registry
        .update(cx, |registry, cx| registry.route_payload(&payload, cx))
        .expect("route close-policy command");
    if registry.read_with(cx, |registry, _| {
        registry.surfaces.contains_key(&surface_id)
    }) {
        draw_surface(registry, cx, surface_id);
    }
    take_events(runtime)
}

pub fn close_policy_simulate_close_roundtrip(cx: &mut TestAppContext) {
    let allow_runtime = InMemoryAdapter::new();
    let allow_registry = cx.new(|_| NativeStateRegistry::new(allow_runtime.clone()));
    allow_registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open default close-policy surface");
    let allow_window = prepare_close_surface(cx, &allow_runtime, &allow_registry);
    let mut allow_visual = VisualTestContext::from_window(allow_window.into(), cx);
    assert!(
        allow_visual.simulate_close(),
        "default policy must allow close"
    );
    allow_window
        .update(cx, |_, window, _| window.remove_window())
        .expect("remove default close-policy surface");
    assert!(allow_registry.read_with(cx, |registry, _| registry.surfaces.is_empty()));
    assert!(allow_window.update(cx, |_, _, _| ()).is_err());
    let allow_events = take_events(&allow_runtime);
    assert!(allow_events.iter().any(|event| {
        u32::from(event.event_kind()) == crate::EVENT_SURFACE_CLOSED && event.meta.surface_id == 1
    }));

    let deny_runtime = InMemoryAdapter::new();
    let deny_registry = cx.new(|_| NativeStateRegistry::new(deny_runtime.clone()));
    deny_registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open confirmation close-policy surface");
    let deny_window = prepare_close_surface(cx, &deny_runtime, &deny_registry);
    let policy_events = close_policy_command(
        &deny_registry,
        cx,
        &deny_runtime,
        1,
        COMMAND_SET_CLOSE_POLICY,
        None,
        Some("require-confirmation"),
    );
    assert!(command_result(&policy_events, 1).success);

    let mut deny_visual = VisualTestContext::from_window(deny_window.into(), cx);
    assert!(
        !deny_visual.simulate_close(),
        "confirmation policy must veto close"
    );
    let first_request = take_events(&deny_runtime);
    let request = first_request
        .iter()
        .find(|event| u32::from(event.event_kind()) == crate::EVENT_CLOSE_REQUESTED)
        .expect("close request event");
    assert_eq!(request.meta.surface_id, 1);
    assert_eq!(request.meta.node_id, 1);
    assert_eq!(request.meta.listener_id, 0);
    assert_eq!(
        request.payload,
        EventPayload::CloseRequested { request_id: 1 }
    );

    assert!(
        !deny_visual.simulate_close(),
        "duplicate pending close must remain vetoed"
    );
    assert!(
        take_events(&deny_runtime)
            .iter()
            .all(|event| u32::from(event.event_kind()) != crate::EVENT_CLOSE_REQUESTED)
    );
    assert!(deny_registry.read_with(cx, |registry, _| registry.surfaces.contains_key(&1)));

    let stale_events = close_policy_command(
        &deny_registry,
        cx,
        &deny_runtime,
        2,
        COMMAND_RESOLVE_CLOSE_REQUEST,
        Some((99, 1)),
        None,
    );
    assert!(command_result(&stale_events, 2).success);
    assert!(deny_registry.read_with(cx, |registry, _| registry.surfaces.contains_key(&1)));

    let deny_events = close_policy_command(
        &deny_registry,
        cx,
        &deny_runtime,
        3,
        COMMAND_RESOLVE_CLOSE_REQUEST,
        Some((1, 0)),
        None,
    );
    assert!(command_result(&deny_events, 3).success);
    assert!(deny_registry.read_with(cx, |registry, _| registry.surfaces.contains_key(&1)));
    let resolved_events = close_policy_command(
        &deny_registry,
        cx,
        &deny_runtime,
        5,
        COMMAND_RESOLVE_CLOSE_REQUEST,
        Some((1, 1)),
        None,
    );
    assert!(command_result(&resolved_events, 5).success);
    assert!(
        resolved_events
            .iter()
            .all(|event| u32::from(event.event_kind()) != crate::EVENT_CLOSE_REQUESTED)
    );
    assert!(deny_registry.read_with(cx, |registry, _| registry.surfaces.contains_key(&1)));

    assert!(
        !deny_visual.simulate_close(),
        "denied request must leave policy pending again"
    );
    let second_request = take_events(&deny_runtime)
        .into_iter()
        .find(|event| u32::from(event.event_kind()) == crate::EVENT_CLOSE_REQUESTED)
        .expect("second close request event");
    assert_eq!(
        second_request.payload,
        EventPayload::CloseRequested { request_id: 2 }
    );

    let allow_events = close_policy_command(
        &deny_registry,
        cx,
        &deny_runtime,
        4,
        COMMAND_RESOLVE_CLOSE_REQUEST,
        Some((2, 1)),
        None,
    );
    assert!(command_result(&allow_events, 4).success);
    assert!(deny_registry.read_with(cx, |registry, _| registry.surfaces.is_empty()));
    let sequences = allow_events
        .iter()
        .map(|event| event.meta.sequence)
        .chain(
            take_events(&deny_runtime)
                .iter()
                .map(|event| event.meta.sequence),
        )
        .collect::<Vec<_>>();
    assert!(sequences.windows(2).all(|window| window[0] < window[1]));
}

pub fn last_surface_close_requests_application_quit(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open initial lifecycle surface");
    let initial_snapshot = snapshot_for(1)
        .encode()
        .expect("encode lifecycle initial snapshot");
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&initial_snapshot, cx)
        })
        .expect("route lifecycle initial snapshot");
    let initial_window = draw_surface(&registry, cx, 1);

    let quit_requested = std::rc::Rc::new(std::cell::Cell::new(false));
    let quit_requested_for_close = quit_requested.clone();
    let registry_for_close = registry.downgrade();
    let close_subscription = cx.update(|cx| {
        cx.on_window_closed(move |cx, window_id| {
            if let Some(registry) = registry_for_close.upgrade() {
                let should_quit =
                    registry.update(cx, |registry, cx| registry.window_closed(window_id, cx));
                if should_quit {
                    quit_requested_for_close.set(true);
                    cx.quit();
                }
            }
        })
    });
    registry.update(cx, |registry, _| {
        registry.close_subscription = Some(close_subscription);
    });

    route_command(
        &registry,
        cx,
        command(
            1,
            COMMAND_OPEN_SURFACE,
            1,
            Some((320, 240)),
            Some("Lifecycle auxiliary"),
            None,
            None,
        ),
    );
    assert_eq!(
        registry.read_with(cx, |registry, _| registry.surfaces.len()),
        2
    );
    let auxiliary_snapshot = snapshot_for(2)
        .encode()
        .expect("encode lifecycle auxiliary snapshot");
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&auxiliary_snapshot, cx)
        })
        .expect("route lifecycle auxiliary snapshot");
    let auxiliary_window = draw_surface(&registry, cx, 2);

    initial_window
        .update(cx, |_, window, _| window.remove_window())
        .expect("remove first lifecycle surface");
    assert!(!quit_requested.get());
    assert_eq!(
        registry.read_with(cx, |registry, _| registry.surfaces.len()),
        1
    );

    auxiliary_window
        .update(cx, |_, window, _| window.remove_window())
        .expect("remove final lifecycle surface");
    assert!(quit_requested.get());
    assert!(registry.read_with(cx, |registry, _| registry.surfaces.is_empty()));
}

#[gpui::test]
fn application_connection_survives_last_window_and_reopen_allocates_a_fresh_surface(
    cx: &mut TestAppContext,
) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    registry.update(cx, |registry, cx| {
        registry.open_initial(cx).unwrap();
        let configure = Command::new(
            CommandMeta {
                surface_id: 0,
                epoch: 1,
                after_revision: 0,
                request_id: 1,
                node_id: 0,
            },
            CommandOperation::ConfigureApplication {
                keep_alive: true,
                quit: false,
                acknowledged_sequence: 0,
            },
        );
        registry
            .route_payload(&configure.encode().unwrap(), cx)
            .unwrap();
    });
    let launch = runtime.take_event().unwrap().unwrap();
    assert!(matches!(
        launch.payload,
        EventPayload::ApplicationActivation {
            target_surface_id: 1,
            ..
        }
    ));
    registry.update(cx, |registry, cx| {
        registry
            .application
            .configure(1, true, false, launch.meta.sequence)
            .unwrap();
        let window = registry.surfaces[&1].window;
        assert!(!registry.window_closed(window.window_id(), cx));
        window
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
        assert!(registry.surfaces.is_empty());
        registry
            .activate_application("open-urls", vec!["file:///tmp/report.txt".into()], cx)
            .unwrap();
        assert!(registry.surfaces.contains_key(&2));
        assert!(registry.retired_surface_ids.contains(&1));
    });
    let mut reopened = None;
    while let Some(event) = runtime.take_event().unwrap() {
        if let EventPayload::ApplicationActivation {
            target_surface_id,
            reason,
            urls,
        } = event.payload
        {
            reopened = Some((target_surface_id, reason, urls));
        }
    }
    assert_eq!(
        reopened,
        Some((2, "open-urls".into(), vec!["file:///tmp/report.txt".into()]))
    );
    registry.update(cx, |registry, cx| {
        let auxiliary_id = registry.allocate_surface_id().unwrap();
        assert_eq!(auxiliary_id, 3);
        let auxiliary = registry
            .open_window(Some("Auxiliary"), 320, 240, None, cx)
            .unwrap();
        registry.insert_surface(auxiliary_id, auxiliary);
        let application_window = registry.surfaces[&2].window;
        assert!(!registry.window_closed(application_window.window_id(), cx));
        application_window
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
        registry
            .activate_application("reopen", Vec::new(), cx)
            .unwrap();
        assert_eq!(registry.application_surface_id, Some(4));
        assert!(registry.surfaces.contains_key(&3));
        assert!(registry.surfaces.contains_key(&4));
        registry
            .activate_application("open-urls", vec!["demo://second-document".into()], cx)
            .unwrap();
        assert_eq!(registry.surfaces.len(), 2);
    });
    let mut targets = Vec::new();
    while let Some(event) = runtime.take_event().unwrap() {
        if let EventPayload::ApplicationActivation {
            target_surface_id, ..
        } = event.payload
        {
            targets.push(target_surface_id);
        }
    }
    assert_eq!(
        targets,
        vec![4, 4],
        "activation must recreate the app root without stealing an auxiliary Surface"
    );
}

#[cfg(feature = "quickjs")]
#[gpui::test]
fn generation_preflight_validates_all_windows_and_zero_window_activation(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    registry.update(cx, |registry, cx| {
        registry.open_initial(cx).unwrap();
        let auxiliary = registry
            .open_window(Some("Auxiliary"), 320, 240, None, cx)
            .unwrap();
        let id = registry.allocate_surface_id().unwrap();
        registry.insert_surface(id, auxiliary);
        let snapshot =
            |id, epoch| Snapshot::new(id, epoch, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW)]);
        for id in [1, 2] {
            registry
                .apply_to_surface(DecodedMessage::Snapshot(snapshot(id, 1)), cx)
                .unwrap();
        }
        let configuration = Command::new(
            CommandMeta {
                surface_id: 0,
                epoch: 2,
                after_revision: 0,
                request_id: 1,
                node_id: 0,
            },
            CommandOperation::ConfigureApplication {
                keep_alive: true,
                quit: false,
                acknowledged_sequence: 0,
            },
        );
        let valid = vec![snapshot(1, 2), snapshot(2, 2)];
        assert!(
            registry
                .prepare_generation(&valid, &configuration, 2, cx)
                .is_ok()
        );
        let mut invalid = valid.clone();
        invalid[1].nodes[0].kind = u32::MAX;
        assert!(
            registry
                .prepare_generation(&invalid, &configuration, 2, cx)
                .is_err()
        );
        assert!(
            registry
                .prepare_generation(&valid[..1], &configuration, 2, cx)
                .is_err()
        );
        for surface in registry.surfaces.values() {
            assert_eq!(surface.root.read(cx).store().epoch(), 1);
        }
        registry.surfaces.clear();
        let lifecycle = registry
            .prepare_generation(&[], &configuration, 2, cx)
            .unwrap();
        assert_eq!(lifecycle.epoch, 2);
        assert!(lifecycle.keep_alive);
    });
}

#[gpui::test]
fn system_popover_cancellation_and_epoch_replacement_retire_owned_surfaces(
    cx: &mut TestAppContext,
) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    let weak = registry.downgrade();
    let subscription = cx.update(|cx| {
        cx.on_window_closed(move |cx, id| {
            let _ = weak.update(cx, |registry, cx| registry.window_closed(id, cx));
        })
    });
    registry.update(cx, |registry, cx| {
        registry.close_subscription = Some(subscription);
        registry.open_initial(cx).unwrap();
    });
    let snapshot = |epoch| {
        Snapshot::new(
            1,
            epoch,
            0,
            1,
            vec![
                Node::new(1, 0, 0, KIND_VIEW),
                styled_view(
                    2,
                    1,
                    0,
                    Style {
                        width: Some(120.0),
                        height: Some(32.0),
                        ..Default::default()
                    },
                ),
            ],
        )
    };
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&snapshot(1).encode().unwrap(), cx)
        })
        .unwrap();
    draw_surface(&registry, cx, 1);
    take_events(&runtime);
    let request = |request_id, operation| {
        Command::new(
            CommandMeta {
                surface_id: 1,
                epoch: 1,
                after_revision: 1,
                request_id,
                node_id: 1,
            },
            operation,
        )
    };
    let open = || CommandOperation::OpenPopup {
        anchor_node_id: 2,
        width: 300,
        height: 180,
        placement: 0,
        gap: 8.0,
    };
    registry.update(cx, |registry, cx| {
        registry
            .route_payload(&request(1, open()).encode().unwrap(), cx)
            .unwrap();
        assert_eq!(registry.popups.len(), 1);
        registry
            .route_payload(
                &request(2, CommandOperation::ClosePopup { request_id: 1 })
                    .encode()
                    .unwrap(),
                cx,
            )
            .unwrap();
        assert!(registry.popups.is_empty());
        assert_eq!(registry.surfaces.len(), 1);
    });
    assert!(!command_result(&take_events(&runtime), 1).success);
    route_command(&registry, cx, request(3, open()));
    let result = command_result(&take_events(&runtime), 3);
    assert!(result.success, "{:?}", result.error);
    let Some(CommandValue::Number(child)) = result.value else {
        panic!("popup Surface ID")
    };
    assert_eq!(child, 3, "cancelled IDs must never be reused");
    draw_surface(&registry, cx, 1);
    assert!(
        take_events(&runtime)
            .iter()
            .all(|event| !matches!(event.payload, EventPayload::CommandResult(_)))
    );
    registry.update(cx, |registry, cx| {
        let content = Snapshot::new(child, 1, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW)]);
        registry
            .route_payload(&content.encode().unwrap(), cx)
            .unwrap();
        #[cfg(feature = "quickjs")]
        {
            let configuration = Command::new(
                CommandMeta {
                    surface_id: 0,
                    epoch: 2,
                    after_revision: 0,
                    request_id: 1,
                    node_id: 0,
                },
                CommandOperation::ConfigureApplication {
                    keep_alive: false,
                    quit: false,
                    acknowledged_sequence: 0,
                },
            );
            assert!(
                registry
                    .prepare_generation(&[snapshot(2)], &configuration, 2, cx)
                    .is_ok()
            );
        }
        registry
            .route_payload(&snapshot(2).encode().unwrap(), cx)
            .unwrap();
    });
    cx.run_until_parked();
    registry.read_with(cx, |registry, cx| {
        assert_eq!(registry.surfaces.len(), 1);
        assert!(registry.popups.is_empty());
        assert!(registry.retired_surface_ids.contains(&child));
        assert!(registry.surfaces[&1].root.read(cx).popup_anchors.is_empty());
    });
    take_events(&runtime);
    registry.update(cx, |registry, cx| {
        let late = Snapshot::new(child, 1, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW)]);
        assert!(
            registry.route_payload(&late.encode().unwrap(), cx).is_ok(),
            "a retired Surface must not terminate the application"
        );
        let unknown = Snapshot::new(999, 1, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW)]);
        assert!(
            registry
                .route_payload(&unknown.encode().unwrap(), cx)
                .is_err()
        );
    });
    assert!(
        take_events(&runtime)
            .iter()
            .any(|event| event.meta.surface_id == child
                && matches!(event.payload, EventPayload::SurfaceClosed))
    );
}

#[gpui::test]
fn system_popover_placement_flips_at_edges_and_recovers_requested_size(cx: &mut TestAppContext) {
    use gpui::{point, popup::*};
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime));
    registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .unwrap();
    let parent = window_for(&registry, cx, 1).into();
    let displays = [
        Bounds::new(point(px(-1280.0), px(0.0)), size(px(1280.0), px(800.0))),
        Bounds::new(point(px(0.0), px(-900.0)), size(px(1600.0), px(900.0))),
    ];
    assert_eq!(
        popup_display(
            Bounds::new(point(px(-10.0), px(-100.0)), size(px(100.0), px(30.0))),
            &displays
        ),
        Some(1)
    );
    assert_eq!(
        popup_display(
            Bounds::new(point(px(-900.0), px(200.0)), size(px(100.0), px(30.0))),
            &displays
        ),
        Some(0)
    );
    assert_eq!(
        popup_display(
            Bounds::new(point(px(50.0), px(40.0)), size(px(20.0), px(20.0))),
            &displays
        ),
        Some(1)
    );
    let requested = size(px(300.0), px(200.0));
    let mut options = PopupOptions {
        parent,
        anchor_rect: Bounds::new(point(px(-80.0), px(720.0)), size(px(60.0), px(32.0))),
        anchor: PopupAnchor::BottomLeft,
        gravity: PopupGravity::BottomRight,
        constraint_adjustment: PopupConstraintAdjustment::all(),
        offset: point(px(0.0), px(8.0)),
        grab: false,
    };
    let work = Bounds::new(point(px(-1280.0), px(0.0)), size(px(1280.0), px(800.0)));
    let bounds = popup_bounds(&options, requested, work);
    assert_eq!(bounds, Bounds::new(point(px(-320.0), px(512.0)), requested));
    let tiny = Bounds::new(point(px(-200.0), px(0.0)), size(px(200.0), px(120.0)));
    let constrained = popup_bounds(&options, requested, tiny);
    assert_eq!(constrained, tiny);
    options.anchor_rect.origin = point(px(-900.0), px(200.0));
    let restored = popup_bounds(&options, requested, work);
    assert_eq!(restored.size, requested);
    assert_eq!(restored.origin, point(px(-900.0), px(240.0)));
}

#[gpui::test]
fn system_popover_nested_activation_uses_native_focus_before_observers_catch_up(
    cx: &mut TestAppContext,
) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| NativeStateRegistry::new(runtime.clone()));
    let snapshot = |surface| {
        Snapshot::new(
            surface,
            1,
            0,
            1,
            vec![
                Node::new(1, 0, 0, KIND_VIEW),
                styled_view(
                    2,
                    1,
                    0,
                    Style {
                        width: Some(100.0),
                        height: Some(32.0),
                        ..Default::default()
                    },
                ),
            ],
        )
    };
    registry.update(cx, |registry, cx| {
        registry.open_initial(cx).unwrap();
        registry
            .route_payload(&snapshot(1).encode().unwrap(), cx)
            .unwrap();
    });
    let open = |owner| {
        Command::new(
            CommandMeta {
                surface_id: owner,
                epoch: 1,
                after_revision: 1,
                request_id: 1,
                node_id: 1,
            },
            CommandOperation::OpenPopup {
                anchor_node_id: 2,
                width: 240,
                height: 160,
                placement: 0,
                gap: 8.0,
            },
        )
    };
    route_command(&registry, cx, open(1));
    registry
        .update(cx, |registry, cx| {
            registry.route_payload(&snapshot(2).encode().unwrap(), cx)
        })
        .unwrap();
    draw_surface(&registry, cx, 2);
    route_command(&registry, cx, open(2));
    registry.update(cx, |registry, cx| {
        registry
            .route_payload(&snapshot(3).encode().unwrap(), cx)
            .unwrap();
        // The OS already names the new key window, before GPUI activation callbacks run.
        assert_eq!(
            cx.active_window().unwrap().window_id(),
            registry.surfaces[&3].window.window_id()
        );
        registry.dismiss_unrelated_popups(cx);
        assert!(
            registry.popups.contains_key(&2),
            "nested activation must retain its parent"
        );
        assert!(
            registry.popups.contains_key(&3),
            "nested activation must retain its child"
        );
    });
    cx.run_until_parked();
}
