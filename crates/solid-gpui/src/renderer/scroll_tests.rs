use super::*;
use crate::protocol::VirtualListProperties;
use crate::protocol::{Event, EventPayload, Node, OverflowCode, Snapshot, Style};
use crate::transport::InMemoryAdapter;
use crate::tree::KIND_VIEW;
use gpui::{AppContext as _, ScrollDelta, ScrollWheelEvent, TestAppContext, px};

fn draw_window(cx: &mut TestAppContext, window: gpui::AnyWindowHandle) {
    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .expect("draw scroll test window");
    cx.update_window(window, |_, window, cx| {
        window.simulate_next_frame(cx);
    })
    .expect("deliver scroll test frame callbacks");
    cx.run_until_parked();
}
fn typescript_scroll_snapshot() -> Snapshot {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let script = r#"
import { MemoryTransport, View, createRoot } from "./packages/solid-gpui/src/index.ts";
import { createComponent } from "./packages/solid-gpui/src/runtime.ts";
const transport = new MemoryTransport();
const root = createRoot(transport, { surfaceId: 7, epoch: 3 });
root.render(() => createComponent(View, {
  style: { width: 100, height: 100, overflow: "scroll" },
  children: createComponent(View, {
    style: { width: 100, height: 300 },
    onLayout: () => {},
  }),
}));
process.stdout.write(transport.submitted[0]!);
"#;
    let output = std::process::Command::new("bun")
        .current_dir(repo_root)
        .args([
            "--conditions=browser",
            "--preload",
            "./scripts/solid-jsx.ts",
            "-e",
            script,
        ])
        .output()
        .expect("run TypeScript scroll fixture");
    assert!(
        output.status.success(),
        "TypeScript scroll fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.len() > 4,
        "TypeScript fixture emitted no frame"
    );
    Snapshot::decode(&output.stdout[4..]).expect("decode TypeScript scroll snapshot")
}

#[gpui::test]
fn typescript_overflow_scroll_moves_children_after_wheel(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let window = cx.open_window(gpui::size(px(100.0), px(100.0)), {
        let runtime = runtime.clone();
        move |_, _| SolidRoot::new(runtime)
    });
    let root = window.root(cx).expect("TypeScript scroll test root");
    let snapshot = typescript_scroll_snapshot();
    let scroll_node = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.style.as_ref().and_then(|style| style.overflow) == Some(OverflowCode::Scroll)
        })
        .expect("TypeScript overflow node");
    assert_eq!(
        scroll_node.listener_id, 0,
        "overflow style does not require a callback"
    );
    let child_node = snapshot
        .nodes
        .iter()
        .find(|node| node.id != scroll_node.id && node.parent_id == scroll_node.id)
        .expect("TypeScript child node");
    assert_ne!(
        child_node.listener_id, 0,
        "layout callback allocates a listener"
    );
    let child_id = child_node.id;
    root.update(cx, |root, cx| {
        root.apply_payload(
            &snapshot
                .encode()
                .expect("encode TypeScript scroll snapshot"),
            cx,
        )
    })
    .expect("apply TypeScript scroll snapshot");
    draw_window(cx, window.into());
    while runtime
        .take_event()
        .expect("drain initial TypeScript scroll events")
        .is_some()
    {}
    let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
    visual.simulate_event(ScrollWheelEvent {
        position: gpui::point(px(50.0), px(50.0)),
        delta: ScrollDelta::Pixels(gpui::point(px(0.0), px(-40.0))),
        ..Default::default()
    });
    draw_window(cx, window.into());

    let mut layout_events = Vec::new();
    while let Some(event) = runtime
        .take_event()
        .expect("read TypeScript scroll layout event")
    {
        if event.payload.event_kind() == crate::protocol::EventKind::Layout {
            layout_events.push(event);
        }
    }
    let child = layout_events
        .iter()
        .find(|event| event.meta.node_id == child_id)
        .expect("TypeScript child layout after scroll");
    assert!(matches!(
        child.payload,
        EventPayload::Layout { y, .. } if y < 0.0
    ));
}

fn zero_listener_gallery_snapshot() -> Snapshot {
    let output = std::process::Command::new("bun")
        .current_dir(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("crate parent")
                .parent()
                .expect("workspace root"),
        )
        .args([
            "--conditions=browser",
            "--preload",
            "./scripts/solid-jsx.ts",
            "-e",
            r##"
import { MemoryTransport, View, createRoot } from "./packages/solid-gpui/src/index.ts";
import { createComponent } from "./packages/solid-gpui/src/runtime.ts";
const transport = new MemoryTransport();
const root = createRoot(transport, { surfaceId: 7, epoch: 3 });
root.render(() => createComponent(View, {
  style: { width: 100, height: 100, overflow: "scroll" },
  children: createComponent(View, {
    style: { width: 100, height: 300, backgroundColor: "#334455" },
  }),
}));
process.stdout.write(transport.submitted[0]!);
"##,
        ])
        .output()
        .expect("run zero-listener gallery fixture");
    assert!(
        output.status.success(),
        "zero-listener gallery fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.len() > 4,
        "zero-listener fixture emitted no frame"
    );
    Snapshot::decode(&output.stdout[4..]).expect("decode zero-listener gallery snapshot")
}

#[gpui::test]
fn zero_listener_gallery_overflow_moves_plain_children_after_wheel(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let window = cx.open_window(gpui::size(px(100.0), px(100.0)), {
        let runtime = runtime.clone();
        move |_, _| SolidRoot::new(runtime)
    });
    let root = window.root(cx).expect("zero-listener gallery root");
    let snapshot = zero_listener_gallery_snapshot();
    let scroll_node = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.style.as_ref().and_then(|style| style.overflow) == Some(OverflowCode::Scroll)
        })
        .expect("zero-listener overflow node");
    assert!(
        snapshot.nodes.iter().all(|node| node.listener_id == 0),
        "gallery-shaped tree must contain no listeners: {:?}",
        snapshot
            .nodes
            .iter()
            .map(|node| (node.id, node.listener_id))
            .collect::<Vec<_>>()
    );
    let child_id = snapshot
        .nodes
        .iter()
        .find(|node| node.parent_id == scroll_node.id)
        .expect("zero-listener overflow child")
        .id;
    assert!(child_id != 0, "zero-listener child id must be non-zero");
    root.update(cx, |root, cx| {
        root.apply_payload(
            &snapshot
                .encode()
                .expect("encode zero-listener gallery snapshot"),
            cx,
        )
    })
    .expect("apply zero-listener gallery snapshot");
    draw_window(cx, window.into());
    while runtime
        .take_event()
        .expect("drain zero-listener gallery initial events")
        .is_some()
    {}

    let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
    let child_y = |visual: &mut gpui::VisualTestContext| {
        visual.update(|window, _| {
            let quads = window.painted_quads();
            quads
                .into_iter()
                .find(|quad| quad.bounds.size.height.as_f32() >= 300.0)
                .map(|quad| quad.bounds.origin.y.as_f32())
                .expect("zero-listener child background quad")
        })
    };
    let before = child_y(&mut visual);
    visual.simulate_event(ScrollWheelEvent {
        position: gpui::point(px(50.0), px(50.0)),
        delta: ScrollDelta::Pixels(gpui::point(px(0.0), px(-40.0))),
        ..Default::default()
    });
    draw_window(cx, window.into());
    let after = child_y(&mut visual);
    assert!(
        after < before,
        "zero-listener child should move after wheel: before={before}, after={after}"
    );
}

fn scroll_snapshot() -> Snapshot {
    let mut scroll = Node::new(2, 1, 0, KIND_VIEW);
    scroll.listener_id = 2;
    scroll.style = Some(Style {
        width: Some(100.0),
        height: Some(100.0),
        overflow: Some(OverflowCode::Scroll),
        ..Style::default()
    });

    let mut child = Node::new(3, 2, 0, KIND_VIEW);
    child.listener_id = 3;
    child.style = Some(Style {
        width: Some(100.0),
        height: Some(300.0),
        ..Style::default()
    });

    Snapshot::new(
        7,
        3,
        0,
        1,
        vec![Node::new(1, 0, 0, KIND_VIEW), scroll, child],
    )
}

#[gpui::test]
fn overflow_scroll_moves_children_after_wheel(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let window = cx.open_window(gpui::size(px(100.0), px(100.0)), {
        let runtime = runtime.clone();
        move |_, _| SolidRoot::new(runtime)
    });
    let root = window.root(cx).expect("scroll test root");
    root.update(cx, |root, cx| {
        root.apply_payload(
            &scroll_snapshot()
                .encode()
                .expect("encode scroll test snapshot"),
            cx,
        )
    })
    .expect("apply scroll test snapshot");
    draw_window(cx, window.into());
    while runtime
        .take_event()
        .expect("drain initial scroll events")
        .is_some()
    {}

    let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
    visual.simulate_event(ScrollWheelEvent {
        position: gpui::point(px(50.0), px(50.0)),
        delta: ScrollDelta::Pixels(gpui::point(px(0.0), px(-40.0))),
        ..Default::default()
    });
    draw_window(cx, window.into());

    let mut layout_events = Vec::new();
    while let Some(event) = runtime.take_event().expect("read scroll layout event") {
        if event.payload.event_kind() == crate::protocol::EventKind::Layout {
            layout_events.push(event);
        }
    }
    let child = layout_events
        .iter()
        .find(|event| event.meta.node_id == 3)
        .expect("child layout after scroll");
    assert!(matches!(
        child.payload,
        EventPayload::Layout { y, .. } if y < 0.0
    ));
}
fn nested_horizontal_strip_snapshot() -> Snapshot {
    let mut main = styled_view(
        2,
        1,
        0,
        Style {
            width: Some(100.0),
            height: Some(100.0),
            overflow: Some(OverflowCode::Scroll),
            ..Style::default()
        },
    );
    main.listener_id = 2;
    let strip = styled_view(
        3,
        2,
        0,
        Style {
            width: Some(100.0),
            height: Some(20.0),
            overflow: Some(OverflowCode::Scroll),
            ..Style::default()
        },
    );
    let strip_content = styled_view(
        4,
        3,
        0,
        Style {
            width: Some(300.0),
            height: Some(20.0),
            ..Style::default()
        },
    );
    let page = styled_view(
        5,
        2,
        1,
        Style {
            width: Some(100.0),
            height: Some(300.0),
            ..Style::default()
        },
    );
    Snapshot::new(
        7,
        3,
        0,
        1,
        vec![
            Node::new(1, 0, 0, KIND_VIEW),
            main,
            strip,
            strip_content,
            page,
        ],
    )
}

#[gpui::test]
fn vertical_wheel_over_horizontal_strip_reaches_parent_scroll(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let window = cx.open_window(gpui::size(px(100.0), px(100.0)), {
        let runtime = runtime.clone();
        move |_, _| SolidRoot::new(runtime)
    });
    let root = window.root(cx).expect("nested strip scroll test root");
    root.update(cx, |root, cx| {
        root.apply_payload(
            &nested_horizontal_strip_snapshot()
                .encode()
                .expect("encode nested strip scroll test snapshot"),
            cx,
        )
    })
    .expect("apply nested strip scroll test snapshot");
    draw_window(cx, window.into());
    let mut initial_events = Vec::new();
    while let Some(event) = runtime
        .take_event()
        .expect("read nested strip initial event")
    {
        if event.payload.event_kind() == crate::protocol::EventKind::Layout {
            initial_events.push(event);
        }
    }
    let main_viewport = layout_frame(&initial_events, 2);
    let strip_viewport = layout_frame(&initial_events, 3);
    let strip_content = layout_frame(&initial_events, 4);
    let page = layout_frame(&initial_events, 5);
    assert!(
        strip_content.2 > strip_viewport.2,
        "strip fixture must overflow horizontally: content={strip_content:?}, viewport={strip_viewport:?}"
    );
    assert!(
        strip_content.3 <= strip_viewport.3,
        "strip fixture must not overflow vertically: content={strip_content:?}, viewport={strip_viewport:?}"
    );
    assert!(
        page.3 > main_viewport.3,
        "parent fixture must overflow vertically: page={page:?}, viewport={main_viewport:?}"
    );

    let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
    visual.simulate_event(ScrollWheelEvent {
        position: gpui::point(px(50.0), px(10.0)),
        delta: ScrollDelta::Pixels(gpui::point(px(0.0), px(-40.0))),
        ..Default::default()
    });
    draw_window(cx, window.into());

    let mut events = Vec::new();
    while let Some(event) = runtime
        .take_event()
        .expect("read nested strip layout event")
    {
        if event.payload.event_kind() == crate::protocol::EventKind::Layout {
            events.push(event);
        }
    }
    assert!(
        layout_y(&events, 5) < 20.0,
        "vertical wheel should move parent content over strip"
    );
}

fn styled_view(id: u32, parent_id: u32, index: u32, style: Style) -> Node {
    let mut node = Node::new(id, parent_id, index, KIND_VIEW);
    node.listener_id = id;
    node.style = Some(style);
    node
}

fn gallery_snapshot() -> Snapshot {
    let header = styled_view(
        2,
        1,
        0,
        Style {
            width: Some(100.0),
            height: Some(20.0),
            flex_shrink: Some(0.0),
            ..Style::default()
        },
    );
    let body = styled_view(
        3,
        1,
        1,
        Style {
            flex_direction: Some(crate::protocol::FlexDirectionCode::Row),
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
            width: Some(40.0),
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
    let footer = styled_view(
        9,
        4,
        1,
        Style {
            margin_top: Some(24.0),
            min_width: Some(0.0),
            padding: Some(4.0),
            gap: Some(8.0),
            ..Style::default()
        },
    );
    let page = styled_view(
        40,
        6,
        0,
        Style {
            min_width: Some(0.0),
            gap: Some(24.0),
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
        footer,
        page,
    ];
    for index in 0..6 {
        nodes.push(styled_view(
            10 + index,
            7,
            index + 1,
            Style {
                height: Some(30.0),
                padding: Some(8.0),
                flex_direction: Some(crate::protocol::FlexDirectionCode::Row),
                ..Style::default()
            },
        ));
        nodes.push(styled_view(
            20 + index,
            10 + index,
            0,
            Style {
                height: Some(14.0),
                ..Style::default()
            },
        ));
        nodes.push(styled_view(
            50 + index,
            40,
            index,
            Style {
                height: Some(40.0),
                ..Style::default()
            },
        ));
    }
    nodes.push(styled_view(
        30,
        9,
        0,
        Style {
            height: Some(15.0),
            ..Style::default()
        },
    ));
    Snapshot::new(7, 3, 0, 1, nodes)
}

fn layout_frame(events: &[Event], node_id: u32) -> (f32, f32, f32, f32) {
    events
        .iter()
        .find(|event| {
            event.payload.event_kind() == crate::protocol::EventKind::Layout
                && event.meta.node_id == node_id
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
        .expect("layout event")
}

fn layout_y(events: &[Event], node_id: u32) -> f32 {
    layout_frame(events, node_id).1
}

#[gpui::test]
fn gallery_nested_overflow_views_move_sidebar_and_content_after_wheel(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let window = cx.open_window(gpui::size(px(100.0), px(100.0)), {
        let runtime = runtime.clone();
        move |_, _| SolidRoot::new(runtime)
    });
    let root = window.root(cx).expect("gallery scroll test root");
    root.update(cx, |root, cx| {
        root.apply_payload(
            &gallery_snapshot()
                .encode()
                .expect("encode gallery scroll snapshot"),
            cx,
        )
    })
    .expect("apply gallery scroll snapshot");
    draw_window(cx, window.into());
    let mut initial_events = Vec::new();
    while let Some(event) = runtime.take_event().expect("read gallery initial events") {
        if event.payload.event_kind() == crate::protocol::EventKind::Layout {
            initial_events.push(event);
        }
    }
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

    let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
    visual.simulate_event(ScrollWheelEvent {
        position: gpui::point(px(20.0), px(70.0)),
        delta: ScrollDelta::Pixels(gpui::point(px(0.0), px(-40.0))),
        ..Default::default()
    });
    draw_window(cx, window.into());
    let mut sidebar_events = Vec::new();
    while let Some(event) = runtime.take_event().expect("read sidebar scroll events") {
        sidebar_events.push(event);
    }
    assert!(
        layout_y(&sidebar_events, 10) < 20.0,
        "sidebar child should move after wheel"
    );

    visual.simulate_event(ScrollWheelEvent {
        position: gpui::point(px(70.0), px(70.0)),
        delta: ScrollDelta::Pixels(gpui::point(px(0.0), px(-40.0))),
        ..Default::default()
    });
    draw_window(cx, window.into());
    let mut content_events = Vec::new();
    while let Some(event) = runtime.take_event().expect("read content scroll events") {
        content_events.push(event);
    }
    assert!(
        layout_y(&content_events, 50) < 20.0,
        "content child should move after wheel"
    );
}

#[gpui::test]
fn virtual_list_at_top_passes_wheel_to_outer_content(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let window = cx.open_window(gpui::size(px(100.0), px(150.0)), {
        let runtime = runtime.clone();
        move |_, _| SolidRoot::new(runtime)
    });
    let root = window.root(cx).unwrap();
    let main = styled_view(
        2,
        1,
        0,
        Style {
            width: Some(100.0),
            height: Some(150.0),
            overflow: Some(OverflowCode::Scroll),
            ..Style::default()
        },
    );
    let spacer = styled_view(
        3,
        2,
        0,
        Style {
            height: Some(50.0),
            ..Style::default()
        },
    );
    let mut list = Node::new(4, 2, 1, KIND_VIRTUAL_LIST);
    list.listener_id = 4;
    list.style = Some(Style {
        width: Some(100.0),
        height: Some(100.0),
        ..Style::default()
    });
    list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
        item_count: 100,
        range_start: 0,
        range_end: 0,
        estimated_item_size: 20.0,
        overscan: 2,
    }));
    let tail = styled_view(
        5,
        2,
        2,
        Style {
            height: Some(200.0),
            ..Style::default()
        },
    );
    let snapshot = Snapshot::new(
        7,
        3,
        0,
        1,
        vec![Node::new(1, 0, 0, KIND_VIEW), main, spacer, list, tail],
    );
    root.update(cx, |root, cx| {
        root.apply_payload(&snapshot.encode().unwrap(), cx)
    })
    .unwrap();
    draw_window(cx, window.into());
    let mut events = Vec::new();
    while let Some(event) = runtime.take_event().unwrap() {
        events.push(event);
    }
    assert_eq!(layout_y(&events, 3), 0.0);
    // Start outside the list, leaving the outer page scrolled down by 30 px.
    let wheel = |cx: &mut TestAppContext, y: f32, dy: f32| {
        let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
        visual.simulate_event(ScrollWheelEvent {
            position: gpui::point(px(50.0), px(y)),
            delta: ScrollDelta::Pixels(gpui::point(px(0.0), px(dy))),
            ..Default::default()
        });
        draw_window(cx, window.into());
    };
    wheel(cx, 10.0, -30.0);
    events.clear();
    while let Some(event) = runtime.take_event().unwrap() {
        events.push(event);
    }
    assert_eq!(layout_y(&events, 3), -30.0);
    // An upward wheel over an already-top list must move the outer page.
    wheel(cx, 50.0, 20.0);
    events.clear();
    while let Some(event) = runtime.take_event().unwrap() {
        events.push(event);
    }
    assert_eq!(
        layout_y(&events, 3),
        -10.0,
        "list at top must release upward scrolling"
    );
    // Downward scrolling still belongs to the list while it can move.
    wheel(cx, 60.0, -20.0);
    root.read_with(cx, |root, _| {
        assert!(root.virtual_lists[&4].logical_scroll_top().item_ix > 0);
    });
    events.clear();
    while let Some(event) = runtime.take_event().unwrap() {
        events.push(event);
    }
    assert!(
        !events.iter().any(|event| event.meta.node_id == 3
            && event.payload.event_kind() == crate::protocol::EventKind::Layout),
        "consumed wheel must not also scroll parent"
    );
}
