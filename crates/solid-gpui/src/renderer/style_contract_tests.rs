//! Producer-to-native layout and paint checks. TestPlatform records CPU scenes;
//! these checks do not claim GPU presentation or physical input latency.
use super::*;
use crate::protocol::{EventPayload, Snapshot};
use crate::transport::InMemoryAdapter;
use gpui::{AppContext as _, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext};

fn producer_snapshot(children: &str) -> Snapshot {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let script = format!(
        r#"
import {{ createRoot, MemoryTransport, View, Pressable }} from "./packages/solid-gpui/src/index.ts";
import {{ createComponent }} from "./packages/solid-gpui/src/runtime.ts";
const transport = new MemoryTransport();
const root = createRoot(transport, {{surfaceId: 7, epoch: 3}});
root.render(() => {children});
process.stdout.write(transport.submitted[0]);
"#
    );
    let output = std::process::Command::new("bun")
        .current_dir(directory)
        .args(["--conditions=browser", "-e", &script])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Snapshot::decode(&output.stdout[4..]).unwrap()
}
fn draw(cx: &mut TestAppContext, window: gpui::AnyWindowHandle) {
    cx.update_window(window, |_, window, app| window.draw(app).clear(app))
        .unwrap();
    cx.update_window(window, |_, window, app| window.simulate_next_frame(app))
        .unwrap();
    cx.run_until_parked();
}

#[gpui::test]
fn style_contract_producer_units_spacing_and_axis_overflow_have_native_geometry(
    cx: &mut TestAppContext,
) {
    let snapshot = producer_snapshot(
        r##"createComponent(View, {
      style: { width: 200, height: 100, padding: 12, paddingX: 8, paddingY: 4, paddingLeft: 0, flexDirection: "row", overflowX: "hidden", overflowY: "scroll" },
      children: createComponent(View, { style: { width: { unit: "percent", value: 50 }, height: 300, flexBasis: { unit: "percent", value: 50 }, flexShrink: 0, minWidth: { unit: "rem", value: 2 }, backgroundColor: "#123456" }, onLayout: () => {} }),
    })"##,
    );
    let runtime = InMemoryAdapter::new();
    let window = cx.open_window(gpui::size(px(240.), px(120.)), {
        let runtime = runtime.clone();
        move |_, _| SolidRoot::new(runtime)
    });
    let root = window.root(cx).unwrap();
    root.update(cx, |root, app| {
        root.apply_payload(&snapshot.encode().unwrap(), app)
    })
    .unwrap();
    draw(cx, window.into());
    let mut frame = None;
    while let Some(event) = runtime.take_event().unwrap() {
        if let EventPayload::Layout {
            x,
            y,
            width,
            height,
        } = event.payload
        {
            frame = Some((x, y, width, height));
        }
    }
    assert_eq!(frame, Some((0., 4., 96., 300.)));
    let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
    visual.simulate_event(gpui::ScrollWheelEvent {
        position: gpui::point(px(40.), px(40.)),
        delta: gpui::ScrollDelta::Pixels(gpui::point(px(-20.), px(-40.))),
        ..Default::default()
    });
    draw(cx, window.into());
    let mut after = None;
    while let Some(event) = runtime.take_event().unwrap() {
        if let EventPayload::Layout { x, y, .. } = event.payload {
            after = Some((x, y));
        }
    }
    assert_eq!(after, Some((0., -36.)));
    // Rems must resolve against the native window font size, rather than being
    // producer-converted pixels. Auto height uses the declared aspect ratio.
    let snapshot = producer_snapshot(
        r##"createComponent(View, { style: { width: { unit: "rem", value: 5 }, height: "auto", aspectRatio: 2, flexShrink: 0, backgroundColor: "#123456" }, onLayout: () => {} })"##,
    );
    let snapshot = Snapshot::new(7, 4, 0, 1, snapshot.nodes);
    root.update(cx, |root, app| {
        root.apply_payload(&snapshot.encode().unwrap(), app)
    })
    .unwrap();
    cx.update_window(window.into(), |_, window, _| window.set_rem_size(px(20.)))
        .unwrap();
    draw(cx, window.into());
    let geometry = cx
        .update_window(window.into(), |_, window, _| {
            window
                .painted_quads()
                .into_iter()
                .find(|quad| quad.background == gpui::rgba(0x123456ff).into())
                .unwrap()
                .bounds
                .size
                .map(|v| v.as_f32() / window.scale_factor())
        })
        .unwrap();
    assert_eq!((geometry.width, geometry.height), (100., 50.));
}

#[gpui::test]
fn style_contract_native_refinements_paint_without_js_listeners(cx: &mut TestAppContext) {
    let snapshot = producer_snapshot(
        r##"createComponent(Pressable, {
      focusable: true, style: { width: 80, height: 40, backgroundColor: "#112233", borderWidth: 2,
        hover: { backgroundColor: "#445566" }, active: { backgroundColor: "#778899" },
        focusVisible: { borderColor: "#00ff00" } },
    })"##,
    );
    let node = snapshot
        .nodes
        .iter()
        .find(|v| v.style.as_ref().is_some_and(|s| s.hover.is_some()))
        .unwrap();
    assert_eq!(node.listener_id, 0);
    let id = node.id;
    let runtime = InMemoryAdapter::new();
    let window = cx.open_window(gpui::size(px(160.), px(100.)), {
        let runtime = runtime.clone();
        move |_, _| SolidRoot::new(runtime)
    });
    let root = window.root(cx).unwrap();
    root.update(cx, |root, app| {
        root.apply_payload(&snapshot.encode().unwrap(), app)
    })
    .unwrap();
    draw(cx, window.into());
    let color = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, _| {
            window
                .painted_quads()
                .into_iter()
                .find(|quad| quad.bounds.size.width.as_f32() == 80. * window.scale_factor())
                .unwrap()
                .background
        })
        .unwrap()
    };
    cx.update_window(window.into(), |_, window, app| {
        window.simulate_mouse_move(gpui::point(px(120.), px(80.)), app)
    })
    .unwrap();
    draw(cx, window.into());
    let initial = color(cx);
    assert_eq!(initial, gpui::rgba(0x112233ff).into());
    cx.update_window(window.into(), |_, window, app| {
        window.simulate_mouse_move(gpui::point(px(20.), px(20.)), app)
    })
    .unwrap();
    draw(cx, window.into());
    let hover = color(cx);
    assert_eq!(hover, gpui::rgba(0x445566ff).into());
    let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
    visual.simulate_event(MouseDownEvent {
        position: gpui::point(px(20.), px(20.)),
        button: MouseButton::Left,
        ..Default::default()
    });
    draw(cx, window.into());
    let active = color(cx);
    assert_eq!(active, gpui::rgba(0x778899ff).into());
    visual.simulate_event(MouseUpEvent {
        position: gpui::point(px(20.), px(20.)),
        button: MouseButton::Left,
        ..Default::default()
    });
    draw(cx, window.into());
    assert_eq!(color(cx), hover);
    cx.update_window(window.into(), |_, window, app| {
        window.simulate_mouse_move(gpui::point(px(120.), px(80.)), app)
    })
    .unwrap();
    draw(cx, window.into());
    assert_eq!(color(cx), initial);
    cx.simulate_keystrokes(window.into(), "tab");
    cx.update_window(window.into(), |_, window, app| {
        root.read(app)
            .focus_handles
            .get(&id)
            .unwrap()
            .clone()
            .focus(window, app);
    })
    .unwrap();
    draw(cx, window.into());
    let focused = cx
        .update_window(window.into(), |_, window, _| {
            window
                .painted_quads()
                .into_iter()
                .any(|quad| quad.border_color == gpui::rgba(0x00ff00ff).into())
        })
        .unwrap();
    assert!(focused, "keyboard focus must paint focusVisible border");
    while let Some(event) = runtime.take_event().unwrap() {
        assert!(
            !matches!(
                event.payload,
                EventPayload::Hover | EventPayload::Focus | EventPayload::Pointer(_)
            ),
            "native styling emitted an interaction event: {event:?}"
        );
    }
    let mut disabled = snapshot.nodes;
    let node = disabled.iter_mut().find(|v| v.id == id).unwrap();
    node.focusable = false;
    node.accessibility.as_mut().unwrap().disabled = true;
    root.update(cx, |root, app| {
        root.apply_payload(&Snapshot::new(7, 4, 0, 1, disabled).encode().unwrap(), app)
    })
    .unwrap();
    cx.update_window(window.into(), |_, window, app| {
        window.simulate_mouse_move(gpui::point(px(20.), px(20.)), app)
    })
    .unwrap();
    draw(cx, window.into());
    assert_eq!(
        color(cx),
        initial,
        "disabled controls suppress native refinements"
    );
}
