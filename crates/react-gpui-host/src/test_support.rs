use super::*;
use gpui::{TestAppContext, VisualTestContext};
use react_gpui::{
    COMMAND_BLUR, COMMAND_CLIPBOARD_READ, COMMAND_CLIPBOARD_READ_IMAGE, COMMAND_CLIPBOARD_WRITE,
    COMMAND_CLIPBOARD_WRITE_IMAGE, COMMAND_FILE_DIALOG_OPEN, COMMAND_FILE_DIALOG_SAVE,
    COMMAND_FOCUS, COMMAND_FOCUS_NEXT, COMMAND_FOCUS_PREV, COMMAND_GET_FOCUS,
    COMMAND_GET_WINDOW_SIZE, COMMAND_OPEN_SURFACE, COMMAND_OPEN_URL, COMMAND_READ_TEXT_FILE,
    COMMAND_RESIZE_WINDOW, COMMAND_RESOLVE_CLOSE_REQUEST, COMMAND_SCROLL_TO_END,
    COMMAND_SCROLL_TO_INDEX, COMMAND_SET_CLOSE_POLICY, COMMAND_SET_KEYBINDINGS, COMMAND_SET_MENUS,
    COMMAND_SET_SELECTION, COMMAND_SET_TITLE, COMMAND_SHOW_NOTIFICATION, COMMAND_TOGGLE_FULLSCREEN,
    COMMAND_WRITE_TEXT_FILE, ClipboardImage, EventPayload, HostProperties, InMemoryAdapter,
    KIND_PRESSABLE, KIND_TEXT_INPUT, KIND_VIEW, KIND_VIRTUAL_LIST, KeybindingDefinition,
    MenuAction, MenuDefinition, MenuItemDefinition, Node, NotificationActionDefinition,
    PROTOCOL_VERSION, PatchOperation, TextInputProperties, VirtualListProperties,
    WindowOpenOptions,
};
fn command(
    request_id: u32,
    kind: u32,
    node_id: u32,
    payload: Option<(u32, u32)>,
    title: Option<&str>,
    body: Option<&str>,
    menus: Option<Vec<MenuDefinition>>,
) -> Command {
    Command {
        protocol: PROTOCOL_VERSION,
        message: react_gpui::COMMAND_MESSAGE,
        surface_id: 1,
        epoch: 1,
        after_revision: 1,
        request_id,
        node_id,
        kind,
        payload,
        title: title.map(str::to_owned),
        body: body.map(str::to_owned),
        actions: None,
        menus,
        keybindings: None,
        window_options: None,
        image: None,
    }
}
fn keybinding_command(
    surface_id: u32,
    request_id: u32,
    bindings: Vec<KeybindingDefinition>,
) -> Command {
    let mut command = command(
        request_id,
        COMMAND_SET_KEYBINDINGS,
        1,
        None,
        None,
        None,
        None,
    );
    command.surface_id = surface_id;
    command.keybindings = Some(bindings);
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
        COMMAND_SHOW_NOTIFICATION,
        1,
        None,
        Some(title),
        Some(body),
        None,
    );
    command.actions = Some(actions);
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

fn window_for(
    registry: &Entity<SurfaceRegistry>,
    cx: &mut TestAppContext,
    surface_id: u32,
) -> WindowHandle<ReactRoot> {
    registry.read_with(cx, |registry, _| {
        registry
            .surfaces
            .get(&surface_id)
            .expect("test surface")
            .window
    })
}

fn draw_surface(
    registry: &Entity<SurfaceRegistry>,
    cx: &mut TestAppContext,
    surface_id: u32,
) -> WindowHandle<ReactRoot> {
    let window = window_for(registry, cx, surface_id);
    cx.update_window(window.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
    })
    .expect("draw test surface");
    cx.run_until_parked();
    window
}
fn advance_frame(window: WindowHandle<ReactRoot>, cx: &mut TestAppContext) {
    cx.update_window(window.into(), |_, window, cx| {
        window.simulate_next_frame(cx);
    })
    .expect("advance test frame");
    cx.run_until_parked();
}
pub struct HeadlessSurface {
    registry: Entity<SurfaceRegistry>,
    window: WindowHandle<ReactRoot>,
    runtime: Arc<InMemoryAdapter>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaintedQuad {
    pub order: u32,
    pub bounds: (f32, f32, f32, f32),
    pub content_mask: (f32, f32, f32, f32),
    pub background: String,
}

impl HeadlessSurface {
    pub fn new(cx: &mut TestAppContext) -> Self {
        let runtime = InMemoryAdapter::new();
        let registry = cx.new(|_| SurfaceRegistry::new(runtime.clone()));
        registry
            .update(cx, |registry, cx| registry.open_initial(cx))
            .expect("open headless test surface");
        let window = window_for(&registry, cx, 1);
        Self {
            registry,
            window,
            runtime,
        }
    }

    pub fn apply(&self, cx: &mut TestAppContext, payload: &[u8]) {
        self.registry
            .update(cx, |registry, cx| registry.route_payload(payload, cx))
            .expect("apply headless test payload");
    }

    pub fn draw(&self, cx: &mut TestAppContext) {
        cx.update_window(self.window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw headless test surface");
        cx.run_until_parked();
    }
    pub fn advance_frame(&self, cx: &mut TestAppContext) {
        advance_frame(self.window, cx);
    }
    pub fn resize(&self, cx: &mut TestAppContext, width: f32, height: f32) {
        cx.update_window(self.window.into(), |_, window, _| {
            window.resize(size(px(width), px(height)));
        })
        .expect("resize headless test surface");
        cx.run_until_parked();
    }
    pub fn activate(&self, cx: &mut TestAppContext) {
        cx.update_window(self.window.into(), |_, window, _| window.activate_window())
            .expect("activate headless test surface");
        cx.run_until_parked();
    }
    pub fn click(&self, cx: &mut TestAppContext, x: f32, y: f32) {
        let mut visual = gpui::VisualTestContext::from_window(self.window.into(), cx);
        let point = gpui::point(px(x), px(y));
        visual.simulate_mouse_down(point, gpui::MouseButton::Left, gpui::Modifiers::none());
        visual.simulate_mouse_up(point, gpui::MouseButton::Left, gpui::Modifiers::none());
    }
    pub fn scroll(&self, cx: &mut TestAppContext, x: f32, y: f32, dx: f32, dy: f32) {
        let mut visual = gpui::VisualTestContext::from_window(self.window.into(), cx);
        visual.simulate_event(gpui::ScrollWheelEvent {
            position: gpui::point(gpui::px(x), gpui::px(y)),
            delta: gpui::ScrollDelta::Pixels(gpui::point(gpui::px(dx), gpui::px(dy))),
            modifiers: gpui::Modifiers::none(),
            touch_phase: gpui::TouchPhase::Moved,
        });
    }
    pub fn begin_drag(&self, cx: &mut TestAppContext, from: (f32, f32), to: (f32, f32)) {
        let mut visual = gpui::VisualTestContext::from_window(self.window.into(), cx);
        visual.simulate_mouse_down(
            gpui::point(gpui::px(from.0), gpui::px(from.1)),
            gpui::MouseButton::Left,
            gpui::Modifiers::none(),
        );
        visual.simulate_mouse_move(
            gpui::point(gpui::px(to.0), gpui::px(to.1)),
            Some(gpui::MouseButton::Left),
            gpui::Modifiers::none(),
        );
    }

    pub fn move_drag(&self, cx: &mut TestAppContext, to: (f32, f32)) {
        let mut visual = gpui::VisualTestContext::from_window(self.window.into(), cx);
        visual.simulate_mouse_move(
            gpui::point(gpui::px(to.0), gpui::px(to.1)),
            Some(gpui::MouseButton::Left),
            gpui::Modifiers::none(),
        );
    }

    pub fn end_drag(&self, cx: &mut TestAppContext, at: (f32, f32)) {
        let mut visual = gpui::VisualTestContext::from_window(self.window.into(), cx);
        visual.simulate_mouse_up(
            gpui::point(gpui::px(at.0), gpui::px(at.1)),
            gpui::MouseButton::Left,
            gpui::Modifiers::none(),
        );
    }
    pub fn scale_factor(&self, cx: &mut TestAppContext) -> f32 {
        cx.update_window(self.window.into(), |_, window, _| window.scale_factor())
            .expect("read headless test scale factor")
    }

    pub fn painted_quads(&self, cx: &mut TestAppContext) -> Vec<PaintedQuad> {
        cx.update_window(self.window.into(), |_, window, _| {
            window
                .painted_quads()
                .into_iter()
                .map(|quad| PaintedQuad {
                    order: quad.order,
                    bounds: (
                        quad.bounds.origin.x.as_f32(),
                        quad.bounds.origin.y.as_f32(),
                        quad.bounds.size.width.as_f32(),
                        quad.bounds.size.height.as_f32(),
                    ),
                    content_mask: (
                        quad.content_mask.bounds.origin.x.as_f32(),
                        quad.content_mask.bounds.origin.y.as_f32(),
                        quad.content_mask.bounds.size.width.as_f32(),
                        quad.content_mask.bounds.size.height.as_f32(),
                    ),
                    background: format!("{:?}", quad.background),
                })
                .collect()
        })
        .expect("read painted headless quads")
    }

    pub fn events(&self) -> Vec<react_gpui::Event> {
        take_events(&self.runtime)
    }
}

fn route_command(registry: &Entity<SurfaceRegistry>, cx: &mut TestAppContext, command: Command) {
    let payload = command.encode().expect("encode test command");
    registry
        .update(cx, |registry, cx| registry.route_payload(&payload, cx))
        .expect("route test command");
    draw_surface(registry, cx, command.surface_id);
}
fn install_notification_callback(registry: &Entity<SurfaceRegistry>, cx: &mut TestAppContext) {
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

fn take_events(runtime: &InMemoryAdapter) -> Vec<react_gpui::Event> {
    let mut events = Vec::new();
    while let Some(event) = runtime.take_event().expect("read test event") {
        events.push(event);
    }
    events
}

fn command_result(events: &[react_gpui::Event], request_id: u32) -> react_gpui::CommandResult {
    events
        .iter()
        .find_map(|event| match &event.payload {
            Some(EventPayload::CommandResult(result)) if result.request_id == request_id => {
                Some(result.clone())
            }
            _ => None,
        })
        .expect("command result event")
}

pub fn command_roundtrip(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let mut mapped = WindowOptions::default();
    SurfaceRegistry::apply_window_open_options(
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
    let registry = cx.new(|_| SurfaceRegistry::new(runtime.clone()));
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
        react_gpui::EVENT_WINDOW_RESIZE,
        react_gpui::EVENT_WINDOW_ACTIVATION,
        react_gpui::EVENT_WINDOW_APPEARANCE,
        react_gpui::protocol::EVENT_LAYOUT,
    ] {
        assert!(
            events.iter().any(|event| event.event_type == event_type),
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
        Some(react_gpui::CommandValue::Bool(true))
    );

    route_command(
        &registry,
        cx,
        command(3, COMMAND_FOCUS, 3, None, None, None, None),
    );
    let events = take_events(&runtime);
    assert!(command_result(&events, 3).success);
    assert!(
        events
            .iter()
            .any(|event| { event.event_type == react_gpui::EVENT_FOCUS })
    );
    route_command(
        &registry,
        cx,
        command(4, COMMAND_SET_SELECTION, 3, Some((1, 4)), None, None, None),
    );
    let events = take_events(&runtime);
    assert!(command_result(&events, 4).success);
    assert!(events.iter().any(|event| {
        event.event_type == react_gpui::EVENT_SELECTION
            && matches!(
                &event.payload,
                Some(EventPayload::TextInput(input))
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
            .any(|event| { event.event_type == react_gpui::EVENT_BLUR })
    );
    route_command(
        &registry,
        cx,
        command(21, COMMAND_GET_FOCUS, 3, None, None, None, None),
    );
    assert_eq!(
        command_result(&take_events(&runtime), 21).value,
        Some(react_gpui::CommandValue::Bool(false))
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
        Some(react_gpui::CommandValue::Text("copied".to_owned()))
    );
    assert_eq!(
        cx.read_from_clipboard()
            .and_then(|item| item.text())
            .as_deref(),
        Some("copied")
    );

    let mut image_command = command(22, COMMAND_CLIPBOARD_WRITE_IMAGE, 1, None, None, None, None);
    image_command.image = Some(ClipboardImage {
        format: 1,
        bytes: vec![0x89, 0x50, 0x4e, 0x47],
    });
    route_command(&registry, cx, image_command);
    let image_events = take_events(&runtime);
    let image_result = command_result(&image_events, 22);
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        assert!(
            image_result.success,
            "native image clipboard write should succeed"
        );
    } else {
        assert_eq!(image_result.error.as_deref(), Some("platform-unsupported"));
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
            Some(react_gpui::CommandValue::Image(ClipboardImage {
                format: 1,
                bytes: vec![0x89, 0x50, 0x4e, 0x47],
            }))
        );
    } else {
        assert_eq!(
            image_read_result.error.as_deref(),
            Some("platform-unsupported")
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
            .any(|event| event.event_type == react_gpui::EVENT_VISIBLE_RANGE)
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
        Some(react_gpui::CommandValue::Pair((width, height)))
            if width > 0.0 && height > 0.0
    ));

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
        event.event_type == react_gpui::EVENT_ACTION
            && matches!(
                &event.payload,
                Some(EventPayload::EventAction { action }) if action == "test-action"
            )
    }));

    cx.update(|cx| {
        cx.set_app_identity("com.example.react-gpui", "React GPUI");
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
    open_surface.window_options = Some(WindowOpenOptions {
        kind: Some(1),
        resizable: Some(false),
        min_size: Some((160, 120)),
    });
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
        event.event_type == react_gpui::EVENT_SURFACE_CLOSED && event.surface_id == 2
    }));
}
pub fn cross_surface_focus_blur_roundtrip(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| SurfaceRegistry::new(runtime.clone()));
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
    open_surface.window_options = Some(WindowOpenOptions {
        kind: Some(1),
        resizable: Some(false),
        min_size: None,
    });
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
    assert!(
        first_focus_events
            .iter()
            .any(|event| { event.event_type == react_gpui::EVENT_FOCUS && event.surface_id == 1 })
    );

    cx.update_window(second_window.into(), |_, window, _| {
        window.activate_window()
    })
    .expect("activate second focus surface");
    cx.run_until_parked();
    draw_surface(&registry, cx, 1);
    let blur_events = take_events(&runtime);
    assert!(
        blur_events
            .iter()
            .any(|event| { event.event_type == react_gpui::EVENT_BLUR && event.surface_id == 1 })
    );

    let mut second_focus = command(3, COMMAND_FOCUS, 2, None, None, None, None);
    second_focus.surface_id = 2;
    route_command(&registry, cx, second_focus);
    let second_focus_events = take_events(&runtime);
    assert!(
        second_focus_events
            .iter()
            .any(|event| event.event_type == react_gpui::EVENT_FOCUS && event.surface_id == 2),
        "auxiliary focus events: {second_focus_events:?}"
    );
    let first_focus = first_focus_events
        .iter()
        .find(|event| event.event_type == react_gpui::EVENT_FOCUS)
        .expect("first focus event");
    let second_focus = second_focus_events
        .iter()
        .find(|event| event.event_type == react_gpui::EVENT_FOCUS)
        .expect("second focus event");
    assert_ne!(first_focus.surface_id, second_focus.surface_id);
}
fn focus_node(id: u32, parent_id: u32, index: u32, kind: u32, listener_id: u32) -> Node {
    let mut node = Node::new(id, parent_id, index, kind);
    node.listener_id = listener_id;
    node.focusable = true;
    node.style = Some(react_gpui::Style {
        flex_direction: Some(1),
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
    Entity<SurfaceRegistry>,
    WindowHandle<ReactRoot>,
    Arc<InMemoryAdapter>,
) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| SurfaceRegistry::new(runtime.clone()));
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
                .any(|event| event.event_type == react_gpui::EVENT_FOCUS)
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
                .any(|event| event.event_type == react_gpui::EVENT_FOCUS
                    && event.node_id == expected),
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
            .any(|event| event.event_type == react_gpui::EVENT_FOCUS)
        {
            break;
        }
        advance_frame(window, cx);
        events.extend(take_events(&runtime));
    }
    assert!(command_result(&events, 2).success);
    assert!(
        events
            .iter()
            .any(|event| { event.event_type == react_gpui::EVENT_FOCUS && event.node_id == 4 })
    );
    assert!(!events.iter().any(|event| event.node_id == 3));
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
    next.after_revision = 2;
    route_command(&registry, cx, next);
    let events = take_events(&runtime);
    assert!(command_result(&events, 2).success);
    assert!(
        events
            .iter()
            .any(|event| { event.event_type == react_gpui::EVENT_FOCUS && event.node_id == 3 })
    );
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
        events
            .iter()
            .any(|event| { event.event_type == react_gpui::EVENT_BLUR && event.node_id == 3 }),
        "unmounted focus blur missing: {events:?}"
    );
    assert!(
        events
            .iter()
            .any(|event| { event.event_type == react_gpui::EVENT_FOCUS && event.node_id == 2 }),
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
            mask: react_gpui::protocol::UPDATE_FOCUSABLE,
            style: None,
            text: None,
            listener_id: 21,
            host_properties: None,
            accessibility: None,
            focusable: true,
            selectable: false,
            tooltip: None,
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
    focus.after_revision = 2;
    route_command(&registry, cx, focus);
    let mut events = take_events(&runtime);
    for _ in 0..4 {
        if events
            .iter()
            .any(|event| event.event_type == react_gpui::EVENT_FOCUS && event.node_id == 2)
        {
            break;
        }
        advance_frame(window, cx);
        events.extend(take_events(&runtime));
    }
    assert!(command_result(&events, 1).success);
    assert!(
        events
            .iter()
            .any(|event| { event.event_type == react_gpui::EVENT_FOCUS && event.node_id == 2 })
    );
}
pub fn keybinding_roundtrip(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| SurfaceRegistry::new(runtime.clone()));
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
                    keystrokes: "cmd-shift-p".to_owned(),
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
    route_command(
        &registry,
        cx,
        keybinding_command(
            1,
            5,
            vec![KeybindingDefinition {
                keystrokes: "cmd-shift-p".to_owned(),
                action_name: "palette.open".to_owned(),
            }],
        ),
    );
    assert!(command_result(&take_events(&runtime), 5).success);
    cx.update_window(window.into(), |_, window, _| window.activate_window())
        .expect("activate replacement keybinding surface");
    cx.simulate_keystrokes(window.into(), "ctrl-k ctrl-1");
    assert!(
        !take_events(&runtime).iter().any(|event| {
            event.event_type == react_gpui::EVENT_ACTION
                && matches!(
                    &event.payload,
                    Some(EventPayload::EventAction { action }) if action == "menu.other"
                )
        }),
        "replaced keybinding should be removed"
    );
    cx.update_window(window.into(), |_, window, _| window.activate_window())
        .expect("activate keybinding test surface");
    cx.simulate_keystrokes(window.into(), "cmd-shift-p");
    let events = take_events(&runtime);
    assert!(events.iter().any(|event| {
        event.event_type == react_gpui::EVENT_ACTION
            && matches!(
                &event.payload,
                Some(EventPayload::EventAction { action }) if action == "palette.open"
            )
    }));
    assert!(
        !events
            .iter()
            .any(|event| event.event_type == react_gpui::EVENT_KEY),
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
    let auxiliary_snapshot = Snapshot::new(2, 1, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW)]);
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
            vec![KeybindingDefinition {
                keystrokes: "ctrl-alt-p".to_owned(),
                action_name: "auxiliary.open".to_owned(),
            }],
        ),
    );
    assert!(command_result(&take_events(&runtime), 3).success);
    cx.update_window(auxiliary_window.into(), |_, window, _| {
        window.activate_window()
    })
    .expect("activate auxiliary keybinding surface");
    cx.simulate_keystrokes(auxiliary_window.into(), "ctrl-alt-p");
    let events = take_events(&runtime);
    assert!(events.iter().any(|event| {
        event.event_type == react_gpui::EVENT_ACTION
            && matches!(
                &event.payload,
                Some(EventPayload::EventAction { action }) if action == "auxiliary.open"
            )
    }));

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
    cx.simulate_keystrokes(window.into(), "cmd-shift-p");
    assert!(take_events(&runtime).iter().any(|event| {
        event.event_type == react_gpui::EVENT_ACTION
            && matches!(
                &event.payload,
                Some(EventPayload::EventAction { action }) if action == "palette.open"
            )
    }));
    route_command(&registry, cx, keybinding_command(1, 6, Vec::new()));
    assert!(command_result(&take_events(&runtime), 6).success);
    cx.simulate_keystrokes(window.into(), "cmd-shift-p");
    assert!(
        !take_events(&runtime)
            .iter()
            .any(|event| event.event_type == react_gpui::EVENT_ACTION),
        "empty replacement should clear the surface binding"
    );
}

pub fn dialog_command_roundtrip(cx: &mut TestAppContext) {
    use std::path::PathBuf;

    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| SurfaceRegistry::new(runtime.clone()));
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
        Some(react_gpui::CommandValue::Paths(vec![
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
        Some(react_gpui::CommandValue::Paths(vec![
            "/tmp/one.txt".to_owned()
        ]))
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
        Some(react_gpui::CommandValue::Paths(vec![
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
        Some(react_gpui::CommandValue::Text("/tmp/output.txt".to_owned()))
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

    for invalid in [
        command(
            107,
            COMMAND_FILE_DIALOG_OPEN,
            2,
            Some((0, 0)),
            Some("not root"),
            None,
            None,
        ),
        command(
            108,
            COMMAND_FILE_DIALOG_OPEN,
            1,
            Some((2, 0)),
            Some("bad flags"),
            None,
            None,
        ),
    ] {
        let payload = invalid.encode().expect("encode invalid dialog frame");
        assert!(
            registry
                .update(cx, |registry, cx| registry.route_payload(&payload, cx))
                .is_err()
        );
    }

    let close_runtime = InMemoryAdapter::new();
    let close_registry = cx.new(|_| SurfaceRegistry::new(close_runtime.clone()));
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
            .all(|event| event.event_type != react_gpui::EVENT_COMMAND_RESULT)
    );
}
pub fn text_file_command_roundtrip(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| SurfaceRegistry::new(runtime.clone()));
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
    let dir = std::env::temp_dir().join(format!("react-gpui-file-{}", std::process::id()));
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
    assert_eq!(result.value, Some(react_gpui::CommandValue::Number(8.0)));
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
        Some(react_gpui::CommandValue::FileText("hello π".to_owned()))
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
pub fn notification_response_roundtrip(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| SurfaceRegistry::new(runtime.clone()));
    registry
        .update(cx, |registry, cx| registry.open_initial(cx))
        .expect("open notification test surface");
    let window = draw_surface(&registry, cx, 1);
    install_notification_callback(&registry, cx);
    cx.update(|cx| {
        cx.set_app_identity("com.example.react-gpui", "React GPUI");
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
        .find(|event| event.event_type == react_gpui::EVENT_NOTIFICATION_RESPONSE)
        .expect("notification action response event");
    assert_eq!(
        response.payload,
        Some(EventPayload::NotificationResponse(
            react_gpui::NotificationResponseEvent {
                tag: notification.tag.to_string(),
                action_id: Some("open".to_owned()),
            }
        ))
    );

    cx.simulate_system_notification_response(SystemNotificationResponse {
        tag: notification.tag.clone(),
        action_id: None,
    });
    cx.run_until_parked();
    let response = take_events(&runtime)
        .into_iter()
        .find(|event| event.event_type == react_gpui::EVENT_NOTIFICATION_RESPONSE)
        .expect("notification body response event");
    assert_eq!(
        response.payload,
        Some(EventPayload::NotificationResponse(
            react_gpui::NotificationResponseEvent {
                tag: notification.tag.to_string(),
                action_id: None,
            }
        ))
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
            .all(|event| event.event_type != react_gpui::EVENT_NOTIFICATION_RESPONSE)
    );
}

pub fn renderer_termination_closes_surfaces_without_reentrant_update(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let registry = cx.new(|_| SurfaceRegistry::new(runtime.clone()));
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

fn install_close_callback(registry: &Entity<SurfaceRegistry>, cx: &mut TestAppContext) {
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
    registry: &Entity<SurfaceRegistry>,
) -> WindowHandle<ReactRoot> {
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
    registry: &Entity<SurfaceRegistry>,
    cx: &mut TestAppContext,
    runtime: &InMemoryAdapter,
    request_id: u32,
    kind: u32,
    payload: Option<(u32, u32)>,
    title: Option<&str>,
) -> Vec<react_gpui::Event> {
    let command = command(request_id, kind, 1, payload, title, None, None);
    let surface_id = command.surface_id;
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
    let allow_registry = cx.new(|_| SurfaceRegistry::new(allow_runtime.clone()));
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
        event.event_type == react_gpui::EVENT_SURFACE_CLOSED && event.surface_id == 1
    }));

    let deny_runtime = InMemoryAdapter::new();
    let deny_registry = cx.new(|_| SurfaceRegistry::new(deny_runtime.clone()));
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
        .find(|event| event.event_type == react_gpui::EVENT_CLOSE_REQUESTED)
        .expect("close request event");
    assert_eq!(request.surface_id, 1);
    assert_eq!(request.node_id, 1);
    assert_eq!(request.listener_id, 0);
    assert_eq!(
        request.payload,
        Some(EventPayload::CloseRequested { request_id: 1 })
    );

    assert!(
        !deny_visual.simulate_close(),
        "duplicate pending close must remain vetoed"
    );
    assert!(
        take_events(&deny_runtime)
            .iter()
            .all(|event| event.event_type != react_gpui::EVENT_CLOSE_REQUESTED)
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
            .all(|event| event.event_type != react_gpui::EVENT_CLOSE_REQUESTED)
    );
    assert!(deny_registry.read_with(cx, |registry, _| registry.surfaces.contains_key(&1)));

    assert!(
        !deny_visual.simulate_close(),
        "denied request must leave policy pending again"
    );
    let second_request = take_events(&deny_runtime)
        .into_iter()
        .find(|event| event.event_type == react_gpui::EVENT_CLOSE_REQUESTED)
        .expect("second close request event");
    assert_eq!(
        second_request.payload,
        Some(EventPayload::CloseRequested { request_id: 2 })
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
        .map(|event| event.sequence)
        .chain(
            take_events(&deny_runtime)
                .iter()
                .map(|event| event.sequence),
        )
        .collect::<Vec<_>>();
    assert!(sequences.windows(2).all(|window| window[0] < window[1]));
}
