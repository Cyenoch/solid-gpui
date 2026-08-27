#![cfg(target_os = "macos")]

#[allow(dead_code)]
#[path = "../src/main.rs"]
mod host;

use gpui::TestAppContext;
use react_gpui::Command as ReactCommand;
use react_gpui::{
    COMMAND_BLUR, COMMAND_FOCUS, EVENT_BLUR, EVENT_FOCUS, EVENT_POINTER, EVENT_POINTER_DOWN,
    EVENT_POINTER_DOWN_OUTSIDE, EVENT_POINTER_UP, Event, EventPayload, KIND_PRESSABLE, KIND_TEXT,
    KIND_VIEW, Node, PROTOCOL_VERSION, Patch, PatchOperation, Snapshot, Style, read_frame,
    write_frame,
};
use std::collections::HashMap;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

const EXAMPLES: &[(&str, &str)] = &[
    ("counter", "packages/react-gpui/examples/counter.tsx"),
    ("todo", "packages/react-gpui/examples/todo.tsx"),
    ("text-input", "packages/react-gpui/examples/text-input.tsx"),
    (
        "virtual-list",
        "packages/react-gpui/examples/virtual-list.tsx",
    ),
    ("stress", "packages/react-gpui/examples/stress.tsx"),
    ("gallery", "packages/react-gpui/examples/gallery.tsx"),
    ("keyboard", "packages/react-gpui/examples/keyboard.tsx"),
    (
        "selectable-text",
        "packages/react-gpui/examples/selectable-text.tsx",
    ),
];

struct RendererProcess {
    entry: String,
    child: Child,
    stdin: ChildStdin,
    stdout: Option<BufReader<ChildStdout>>,
}

impl RendererProcess {
    fn spawn(repo_root: &PathBuf, entry: &str) -> Self {
        let mut child = Command::new("bun")
            .args(["run", entry])
            .current_dir(repo_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|error| panic!("spawn {entry}: {error}"));
        let stdin = child.stdin.take().expect("renderer stdin");
        let stdout = BufReader::new(child.stdout.take().expect("renderer stdout"));
        Self {
            entry: entry.to_owned(),
            child,
            stdin,
            stdout: Some(stdout),
        }
    }

    fn try_read_frame(&mut self) -> Result<Option<Vec<u8>>, String> {
        read_frame(self.stdout.as_mut().expect("renderer stdout"))
            .map_err(|error| format!("read {} renderer frame: {error}", self.entry))
    }

    fn read_frame_with_timeout(&mut self) -> Vec<u8> {
        let stdout = self.stdout.take().expect("renderer stdout");
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut stdout = stdout;
            let result = read_frame(&mut stdout)
                .map_err(|error| error.to_string())
                .and_then(|frame| frame.ok_or_else(|| "renderer ended before patch".to_owned()));
            let _ = tx.send((stdout, result));
        });
        let (stdout, result) = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("renderer did not emit gallery patch within 2s");
        self.stdout = Some(stdout);
        result.unwrap_or_else(|error| panic!("read gallery patch: {error}"))
    }

    fn send_event(&mut self, event: &Event) {
        write_frame(&mut self.stdin, &event.encode().expect("encode event")).expect("send event");
        self.stdin.flush().expect("flush event");
    }
}

impl Drop for RendererProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[derive(Debug)]
struct TextSignal {
    text_nodes: usize,
    opaque_backgrounds: usize,
    unreadable: Vec<String>,
}

struct ExampleAudit {
    name: &'static str,
    startup_error: Option<String>,
    text: Option<TextSignal>,
    dropdown_signal: Option<String>,
    dropdown_error: Option<String>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("repository root")
        .to_path_buf()
}

fn rgba(value: u32) -> (f32, f32, f32, f32) {
    (
        ((value >> 24) & 0xff) as f32 / 255.0,
        ((value >> 16) & 0xff) as f32 / 255.0,
        ((value >> 8) & 0xff) as f32 / 255.0,
        (value & 0xff) as f32 / 255.0,
    )
}

fn luminance(value: (f32, f32, f32, f32)) -> f32 {
    fn channel(value: f32) -> f32 {
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(value.0) + 0.7152 * channel(value.1) + 0.0722 * channel(value.2)
}

fn effective_color(snapshot: &Snapshot, node_id: u32) -> Option<u32> {
    let mut current = snapshot.nodes.iter().find(|node| node.id == node_id)?;
    loop {
        if let Some(color) = current.style.as_ref().and_then(|style| style.color_rgba) {
            return Some(color);
        }
        let Some(parent) = snapshot
            .nodes
            .iter()
            .find(|node| node.id == current.parent_id)
        else {
            return Some(0x0000_00ff);
        };
        current = parent;
    }
}

fn effective_background(snapshot: &Snapshot, node_id: u32) -> Option<u32> {
    let mut current = snapshot.nodes.iter().find(|node| node.id == node_id)?;
    loop {
        if let Some(background) = current
            .style
            .as_ref()
            .and_then(|style| style.background_rgba)
        {
            return Some(background);
        }
        current = snapshot
            .nodes
            .iter()
            .find(|node| node.id == current.parent_id)?;
    }
}
fn opaque_quad(quad: &host::test_support::PaintedQuad) -> bool {
    quad.background.contains("a: 1.")
}

fn white_quad(quad: &host::test_support::PaintedQuad) -> bool {
    opaque_quad(quad) && quad.background.contains("s: 0.0") && quad.background.contains("l: 1.0")
}

fn panel_surface_bounds(
    quads: &[host::test_support::PaintedQuad],
    viewport_width: f32,
    scale: f32,
) -> Vec<(f32, f32, f32, f32)> {
    let min_width = (viewport_width - 40.0) * scale - 1.0;
    quads
        .iter()
        .filter(|quad| {
            white_quad(quad) && quad.bounds.2 >= min_width && quad.bounds.3 >= 300.0 * scale
        })
        .map(|quad| quad.bounds)
        .collect()
}

fn activity_row_bounds(
    quads: &[host::test_support::PaintedQuad],
    viewport_width: f32,
    scale: f32,
) -> Vec<(f32, f32, f32, f32)> {
    let min_width = (viewport_width - 70.0) * scale;
    quads
        .iter()
        .filter(|quad| {
            opaque_quad(quad)
                && quad.bounds.2 >= min_width
                && (40.0 * scale..=80.0 * scale).contains(&quad.bounds.3)
        })
        .map(|quad| quad.bounds)
        .collect()
}

fn gap_only_nested_container_snapshot() -> Snapshot {
    let root = Node::new(1, 0, 0, KIND_VIEW);
    let mut container = Node::new(2, 1, 0, KIND_VIEW);
    container.style = Some(Style {
        width: Some(240.0),
        height: Some(120.0),
        gap: Some(16.0),
        ..Style::default()
    });
    let mut first = Node::new(3, 2, 0, KIND_VIEW);
    first.style = Some(Style {
        width: Some(80.0),
        height: Some(40.0),
        background_rgba: Some(0xff0000ff),
        ..Style::default()
    });
    let mut second = Node::new(4, 2, 1, KIND_VIEW);
    second.style = Some(Style {
        width: Some(80.0),
        height: Some(40.0),
        background_rgba: Some(0x0000ffff),
        ..Style::default()
    });
    Snapshot::new(1, 1, 0, 1, vec![root, container, first, second])
}

fn record_button_bounds(
    quads: &[host::test_support::PaintedQuad],
    viewport_height: f32,
    scale: f32,
) -> Option<(f32, f32, f32, f32)> {
    quads
        .iter()
        .filter(|quad| {
            white_quad(quad)
                && (180.0..=240.0).contains(&quad.bounds.2)
                && (50.0..=80.0).contains(&quad.bounds.3)
                && quad.bounds.1 >= 0.0
                && quad.bounds.1 + quad.bounds.3 <= viewport_height * scale
        })
        .map(|quad| quad.bounds)
        .max_by_key(|bounds| (bounds.1 * 100.0) as i32)
}

fn text_signal(snapshot: &Snapshot, quads: &[host::test_support::PaintedQuad]) -> TextSignal {
    let mut text_nodes = 0;
    let mut opaque_backgrounds = 0;
    let mut unreadable = Vec::new();
    for node in snapshot.nodes.iter().filter(|node| node.kind == KIND_TEXT) {
        text_nodes += 1;
        let Some(background) = effective_background(snapshot, node.id) else {
            unreadable.push(format!("node {} has no rendered background", node.id));
            continue;
        };
        let color = effective_color(snapshot, node.id).unwrap_or(0x0000_00ff);
        let fg = rgba(color);
        let bg = rgba(background);
        if bg.3 >= 0.99 {
            opaque_backgrounds += 1;
        }
        let fg_luma = luminance(fg);
        let bg_luma = luminance(bg);
        let contrast = (fg_luma.max(bg_luma) + 0.05) / (fg_luma.min(bg_luma) + 0.05);
        if fg.3 < 0.99 || bg.3 < 0.99 || contrast < 3.0 {
            unreadable.push(format!(
                "node {} fg=#{color:08x} bg=#{background:08x} alpha={:.2}/{:.2} contrast={contrast:.2}",
                node.id, fg.3, bg.3
            ));
        }
    }
    let rendered_backgrounds = quads.iter().filter(|quad| opaque_quad(quad)).count();
    if text_nodes > 0 && rendered_backgrounds == 0 {
        unreadable.push("GPUI rendered no opaque background quad".to_owned());
    }
    TextSignal {
        text_nodes,
        opaque_backgrounds,
        unreadable,
    }
}

fn menu_node(patch: &Patch) -> Option<u32> {
    patch
        .operations
        .iter()
        .find_map(|operation| match operation {
            PatchOperation::Create(node)
                if node.kind == react_gpui::KIND_VIEW
                    && node.style.as_ref().is_some_and(|style| {
                        style.width == Some(200.0) && style.background_rgba == Some(0xffff_ffff)
                    }) =>
            {
                Some(node.id)
            }
            _ => None,
        })
}

fn gallery_button_center(quads: &[host::test_support::PaintedQuad], scale: f32) -> (f32, f32) {
    let bounds = quads
        .iter()
        .filter(|quad| {
            opaque_quad(quad)
                && quad.bounds.1 >= 90.0 * scale
                && (80.0 * scale..=220.0 * scale).contains(&quad.bounds.2)
                && (24.0 * scale..=48.0 * scale).contains(&quad.bounds.3)
        })
        .min_by(|left, right| {
            left.bounds
                .1
                .total_cmp(&right.bounds.1)
                .then(left.bounds.0.total_cmp(&right.bounds.0))
        })
        .map(|quad| quad.bounds)
        .unwrap_or_else(|| panic!("gallery menu button quad is absent: {quads:?}"));
    (
        (bounds.0 + bounds.2 / 2.0) / scale,
        (bounds.1 + bounds.3 / 2.0) / scale,
    )
}

fn updated_style(patch: &Patch, node_id: u32) -> Option<&Style> {
    patch
        .operations
        .iter()
        .find_map(|operation| match operation {
            PatchOperation::Update {
                id,
                mask,
                style: Some(style),
                ..
            } if *id == node_id && mask & react_gpui::UPDATE_STYLE != 0 => Some(style),
            _ => None,
        })
}

fn forward_event_patch(
    process: &mut RendererProcess,
    surface: &host::test_support::HeadlessSurface,
    cx: &mut TestAppContext,
    event: &Event,
) -> Patch {
    process.send_event(event);
    let patch_payload = process.read_frame_with_timeout();
    let patch = Patch::decode(&patch_payload).expect("decode gallery interaction patch");
    surface.apply(cx, &patch_payload);
    surface.draw(cx);
    patch
}

fn focus_command(
    snapshot: &Snapshot,
    after_revision: u32,
    request_id: u32,
    node_id: u32,
    kind: u32,
) -> ReactCommand {
    ReactCommand {
        protocol: PROTOCOL_VERSION,
        message: react_gpui::COMMAND_MESSAGE,
        surface_id: snapshot.surface_id,
        epoch: snapshot.epoch,
        after_revision,
        request_id,
        node_id,
        kind,
        payload: None,
        title: None,
        body: None,
        actions: None,
        menus: None,
        keybindings: None,
        window_options: None,
    }
}

fn rect_overlap(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> f32 {
    let left = a.0.max(b.0);
    let top = a.1.max(b.1);
    let right = (a.0 + a.2).min(b.0 + b.2);
    let bottom = (a.1 + a.3).min(b.1 + b.3);
    (right - left).max(0.0) * (bottom - top).max(0.0)
}

fn gap_probe_snapshot(nested_direction: Option<u32>) -> Snapshot {
    let mut root = Node::new(1, 0, 0, KIND_VIEW);
    let parent_id = nested_direction.map(|_| 2).unwrap_or(1);
    let mut nodes = vec![root.clone()];
    if let Some(flex_direction) = nested_direction {
        let height = if flex_direction == 2 { 120.0 } else { 80.0 };
        let mut container = Node::new(2, 1, 0, KIND_VIEW);
        container.style = Some(Style {
            width: Some(240.0),
            height: Some(height),
            flex_direction: Some(flex_direction),
            gap: Some(16.0),
            ..Style::default()
        });
        nodes.push(container);
    } else {
        root.style = Some(Style {
            width: Some(240.0),
            height: Some(80.0),
            flex_direction: Some(1),
            gap: Some(16.0),
            ..Style::default()
        });
        nodes[0] = root;
    }
    let mut first = Node::new(3, parent_id, 0, KIND_VIEW);
    first.style = Some(Style {
        width: Some(80.0),
        height: Some(40.0),
        background_rgba: Some(0xff0000ff),
        ..Style::default()
    });
    let mut second = Node::new(4, parent_id, 1, KIND_VIEW);
    second.style = Some(Style {
        width: Some(80.0),
        height: Some(40.0),
        background_rgba: Some(0x0000ffff),
        ..Style::default()
    });
    nodes.extend([first, second]);
    Snapshot::new(1, 1, 0, 1, nodes)
}

fn assert_gap(snapshot: Snapshot, axis: usize, label: &str) {
    let mut cx = TestAppContext::single();
    let surface = host::test_support::HeadlessSurface::new(&mut cx);
    surface.resize(&mut cx, 320.0, 120.0);
    surface.apply(&mut cx, &snapshot.encode().unwrap());
    surface.draw(&mut cx);
    let all_quads = surface.painted_quads(&mut cx);
    let scale = surface.scale_factor(&mut cx);
    let mut quads = all_quads
        .iter()
        .filter(|quad| {
            (quad.bounds.2 - 80.0 * scale).abs() < 0.01
                && (quad.bounds.3 - 40.0 * scale).abs() < 0.01
        })
        .cloned()
        .collect::<Vec<_>>();
    let coordinate = |bounds: (f32, f32, f32, f32)| {
        if axis == 0 {
            (bounds.0, bounds.2)
        } else {
            (bounds.1, bounds.3)
        }
    };
    quads.sort_by(|left, right| {
        coordinate(left.bounds)
            .0
            .total_cmp(&coordinate(right.bounds).0)
    });
    assert_eq!(
        quads.len(),
        2,
        "{label} gap probe rendered unexpected quads: {quads:?}"
    );
    let (first_start, first_extent) = coordinate(quads[0].bounds);
    let second_start = coordinate(quads[1].bounds).0;
    let actual_gap = (second_start - (first_start + first_extent)) / scale;
    assert!(
        (actual_gap - 16.0).abs() < 0.01,
        "{label} fixed sibling gap changed: expected 16px, got {actual_gap:.2}; quads={quads:?}"
    );
}

#[test]
fn renderer_applies_gap_between_fixed_siblings_in_nested_row() {
    assert_gap(gap_probe_snapshot(Some(1)), 0, "nested row");
}

#[test]
fn renderer_applies_gap_between_fixed_siblings_in_nested_column() {
    assert_gap(gap_probe_snapshot(Some(2)), 1, "nested column");
}

#[test]
fn renderer_defaults_gap_only_nested_containers_to_column() {
    assert_gap(
        gap_only_nested_container_snapshot(),
        1,
        "gap-only nested container",
    );
}

#[test]
fn renderer_applies_gap_between_fixed_siblings_on_root() {
    assert_gap(gap_probe_snapshot(None), 0, "root row");
}
fn read_snapshot(process: &mut RendererProcess) -> Result<Option<(Vec<u8>, Snapshot)>, String> {
    for _ in 0..8 {
        let Some(payload) = process.try_read_frame()? else {
            return Ok(None);
        };
        if let Ok(snapshot) = Snapshot::decode(&payload) {
            return Ok(Some((payload, snapshot)));
        }
    }
    Err("renderer emitted eight non-snapshot frames before bootstrap".to_owned())
}

fn gallery_dropdown_signal(
    process: &mut RendererProcess,
    snapshot: &Snapshot,
    surface: &host::test_support::HeadlessSurface,
    cx: &mut TestAppContext,
) -> Result<String, String> {
    let button = snapshot
        .nodes
        .iter()
        .find(|node| node.kind == KIND_PRESSABLE && node.listener_id != 0)
        .ok_or_else(|| "gallery menu button is absent".to_owned())?;
    process.send_event(&Event::press(
        snapshot.surface_id,
        snapshot.epoch,
        snapshot.revision,
        1,
        button.id,
        button.listener_id,
    ));
    let patch_payload = process.read_frame_with_timeout();
    let patch = Patch::decode(&patch_payload).map_err(|error| error.to_string())?;
    let menu_id =
        menu_node(&patch).ok_or_else(|| "gallery expanded menu node is absent".to_owned())?;
    surface.resize(cx, 800.0, 600.0);
    surface.apply(cx, &patch_payload);
    surface.draw(cx);

    let quads = surface.painted_quads(cx);
    let Some(menu_quad) = quads
        .iter()
        .filter(|quad| quad.bounds.2 >= 300.0 && quad.bounds.2 <= 500.0)
        .filter(|quad| quad.bounds.3 >= 40.0)
        .filter(|quad| opaque_quad(quad))
        .max_by_key(|quad| quad.order)
    else {
        return Err(format!(
            "menu-node={menu_id} has no 360px background quad; quads={quads:?}"
        ));
    };
    let menu_area = menu_quad.bounds.2 * menu_quad.bounds.3;
    let visible_area = rect_overlap(menu_quad.bounds, menu_quad.content_mask);
    let covered_later = quads.iter().any(|quad| {
        quad.order > menu_quad.order
            && rect_overlap(menu_quad.bounds, quad.bounds) > menu_area * 0.15
            && opaque_quad(quad)
    });
    let scale_factor = surface.scale_factor(cx);
    surface.click(
        cx,
        (menu_quad.bounds.0 + menu_quad.bounds.2 / 2.0) / scale_factor,
        (menu_quad.bounds.1 + menu_quad.bounds.3 / 2.0) / scale_factor,
    );
    let click_events = surface.events();
    let clickable_down = click_events.iter().any(|event| {
        event.node_id == menu_id
            && event.event_type == EVENT_POINTER
            && matches!(
                &event.payload,
                Some(EventPayload::Pointer(pointer)) if pointer.action == EVENT_POINTER_DOWN
            )
    });
    let clickable_up = click_events.iter().any(|event| {
        event.node_id == menu_id
            && event.event_type == EVENT_POINTER
            && matches!(
                &event.payload,
                Some(EventPayload::Pointer(pointer)) if pointer.action == EVENT_POINTER_UP
            )
    });
    let clickable = clickable_down && clickable_up;
    let viewport = menu_quad.content_mask;
    let in_viewport = menu_quad.bounds.0 >= viewport.0
        && menu_quad.bounds.1 >= viewport.1
        && menu_quad.bounds.0 + menu_quad.bounds.2 <= viewport.0 + viewport.2
        && menu_quad.bounds.1 + menu_quad.bounds.3 <= viewport.1 + viewport.3;
    let above_sibling = quads.iter().any(|quad| {
        quad.order < menu_quad.order
            && rect_overlap(menu_quad.bounds, quad.bounds) > menu_area * 0.15
            && opaque_quad(quad)
    });
    let signal = format!(
        "menu-node={menu_id} bounds={:?} mask={:?} visible_ratio={:.2} in_viewport={in_viewport} above_sibling={above_sibling} clickable_down={clickable_down} clickable_up={clickable_up} covered_later={covered_later}",
        menu_quad.bounds,
        menu_quad.content_mask,
        if menu_area > 0.0 {
            visible_area / menu_area
        } else {
            0.0
        },
    );
    if menu_area <= 0.0
        || visible_area / menu_area < 0.98
        || !in_viewport
        || !above_sibling
        || !clickable
        || covered_later
    {
        Err(signal)
    } else {
        Ok(signal)
    }
}
#[test]
fn gallery_real_click_inside_menu_stays_inside_and_outside_closes_menu() {
    let root = repo_root();
    let mut process = RendererProcess::spawn(&root, "packages/react-gpui/examples/gallery.tsx");
    let (first_payload, snapshot) = read_snapshot(&mut process)
        .expect("read gallery pointer-outside snapshot")
        .expect("gallery pointer-outside probe emitted no snapshot");
    let menu_button = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.kind == KIND_PRESSABLE
                && node
                    .accessibility
                    .as_ref()
                    .and_then(|accessibility| accessibility.label.as_deref())
                    == Some("Show activity menu")
        })
        .expect("gallery menu button");

    let mut cx = TestAppContext::single();
    let surface = host::test_support::HeadlessSurface::new(&mut cx);
    surface.resize(&mut cx, 800.0, 600.0);
    surface.activate(&mut cx);
    surface.apply(&mut cx, &first_payload);
    surface.draw(&mut cx);
    let _ = surface.events();

    let scale = surface.scale_factor(&mut cx);
    let (button_x, button_y) = gallery_button_center(&surface.painted_quads(&mut cx), scale);
    surface.click(&mut cx, button_x, button_y);
    let trigger_events = surface.events();
    assert!(
        trigger_events.iter().any(|event| {
            event.node_id == menu_button.id && event.event_type == react_gpui::EVENT_PRESS
        }),
        "real menu-button click did not produce a press event: {trigger_events:?}"
    );
    assert!(
        trigger_events
            .iter()
            .all(|event| event.event_type != EVENT_POINTER_DOWN_OUTSIDE),
        "menu-button click unexpectedly produced an outside event: {trigger_events:?}"
    );
    let mut open_patches = Vec::new();
    for event in trigger_events
        .iter()
        .filter(|event| event.node_id == menu_button.id)
    {
        open_patches.push(forward_event_patch(&mut process, &surface, &mut cx, event));
    }
    let menu_id = open_patches
        .iter()
        .find_map(menu_node)
        .expect("real menu-button click did not open the menu");

    let _ = surface.events();
    let menu_quad = surface
        .painted_quads(&mut cx)
        .into_iter()
        .filter(|quad| quad.bounds.2 >= 300.0 && quad.bounds.2 <= 500.0)
        .filter(|quad| quad.bounds.3 >= 40.0)
        .filter(|quad| opaque_quad(quad))
        .max_by_key(|quad| quad.order)
        .expect("expanded menu background quad");
    surface.click(
        &mut cx,
        (menu_quad.bounds.0 + 10.0 * scale) / scale,
        (menu_quad.bounds.1 + 10.0 * scale) / scale,
    );
    let inside_events = surface.events();
    assert!(
        inside_events
            .iter()
            .all(|event| event.event_type != EVENT_POINTER_DOWN_OUTSIDE),
        "menu-internal click produced an outside event: {inside_events:?}"
    );
    for event in inside_events.iter().filter(|event| {
        event.node_id == menu_id
            && event.event_type == EVENT_POINTER
            && matches!(
                &event.payload,
                Some(EventPayload::Pointer(pointer)) if pointer.action == EVENT_POINTER_DOWN
            )
    }) {
        forward_event_patch(&mut process, &surface, &mut cx, event);
    }
    assert!(
        surface
            .painted_quads(&mut cx)
            .iter()
            .any(|quad| quad.bounds.2 >= 300.0 && quad.bounds.2 <= 500.0 && opaque_quad(quad)),
        "menu-internal click unexpectedly removed the menu"
    );
    let _ = surface.events();

    surface.click(&mut cx, 790.0, 590.0);
    let outside_events = surface.events();
    let outside_event = outside_events
        .iter()
        .find(|event| event.event_type == EVENT_POINTER_DOWN_OUTSIDE)
        .cloned()
        .expect("real outside click did not produce pointer-down-outside event");
    assert_eq!(outside_event.node_id, menu_id);
    assert!(matches!(
        outside_event.payload,
        Some(EventPayload::PointerDownOutside { x, y }) if x.is_finite() && y.is_finite()
    ));
    let close_patch = forward_event_patch(&mut process, &surface, &mut cx, &outside_event);
    assert!(
        close_patch
            .operations
            .iter()
            .any(|operation| matches!(operation, PatchOperation::Delete { id } if *id == menu_id)),
        "outside event reached renderer without deleting menu {close_patch:?}"
    );
}

#[test]
fn gallery_focus_command_emits_focus_and_blur_style_patches() {
    let root = repo_root();
    let mut process = RendererProcess::spawn(&root, "packages/react-gpui/examples/gallery.tsx");
    let (first_payload, snapshot) = read_snapshot(&mut process)
        .expect("read gallery focus snapshot")
        .expect("gallery focus probe emitted no snapshot");
    let menu_button = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.kind == KIND_PRESSABLE
                && node
                    .accessibility
                    .as_ref()
                    .and_then(|accessibility| accessibility.label.as_deref())
                    == Some("Show activity menu")
        })
        .expect("gallery menu button");

    let mut cx = TestAppContext::single();
    let surface = host::test_support::HeadlessSurface::new(&mut cx);
    surface.resize(&mut cx, 800.0, 600.0);
    surface.activate(&mut cx);
    surface.apply(&mut cx, &first_payload);
    surface.draw(&mut cx);
    let _ = surface.events();

    let focus = focus_command(
        &snapshot,
        snapshot.revision,
        900,
        menu_button.id,
        COMMAND_FOCUS,
    );
    surface.apply(&mut cx, &focus.encode().expect("encode focus command"));
    surface.draw(&mut cx);
    let mut focus_events = Vec::new();
    for _ in 0..4 {
        focus_events.extend(surface.events());
        if focus_events
            .iter()
            .any(|event| event.event_type == EVENT_FOCUS)
        {
            break;
        }
        surface.advance_frame(&mut cx);
    }
    let focus_event = focus_events
        .iter()
        .find(|event| {
            event.event_type == EVENT_FOCUS
                && event.node_id == menu_button.id
                && event.listener_id == menu_button.listener_id
        })
        .cloned()
        .expect("focus command did not emit a menu-button focus event");
    let focus_patch = forward_event_patch(&mut process, &surface, &mut cx, &focus_event);
    assert_eq!(
        updated_style(&focus_patch, menu_button.id)
            .expect("focus patch omitted menu-button style")
            .background_rgba,
        Some(0x2458b8ff),
        "focus event did not apply the focused menu-button style"
    );
    let _ = surface.events();

    let blur = focus_command(
        &snapshot,
        focus_patch.revision,
        901,
        menu_button.id,
        COMMAND_BLUR,
    );
    surface.apply(&mut cx, &blur.encode().expect("encode blur command"));
    surface.draw(&mut cx);
    let mut blur_events = Vec::new();
    for _ in 0..4 {
        blur_events.extend(surface.events());
        if blur_events
            .iter()
            .any(|event| event.event_type == EVENT_BLUR)
        {
            break;
        }
        surface.advance_frame(&mut cx);
    }
    let blur_event = blur_events
        .iter()
        .find(|event| {
            event.event_type == EVENT_BLUR
                && event.node_id == menu_button.id
                && event.listener_id == menu_button.listener_id
        })
        .cloned()
        .expect("blur command did not emit a menu-button blur event");
    let blur_patch = forward_event_patch(&mut process, &surface, &mut cx, &blur_event);
    assert_eq!(
        updated_style(&blur_patch, menu_button.id)
            .expect("blur patch omitted menu-button style")
            .background_rgba,
        Some(0x2d6cdfff),
        "blur event did not restore the menu-button style"
    );
}

#[test]
fn gallery_root_scroll_reaches_content() {
    let root = repo_root();
    let mut process = RendererProcess::spawn(&root, "packages/react-gpui/examples/gallery.tsx");
    let (first_payload, _) = read_snapshot(&mut process)
        .expect("read gallery scroll probe snapshot")
        .expect("gallery scroll probe emitted no snapshot");
    let mut cx = TestAppContext::single();
    let surface = host::test_support::HeadlessSurface::new(&mut cx);
    surface.resize(&mut cx, 800.0, 600.0);
    surface.apply(&mut cx, &first_payload);
    surface.draw(&mut cx);
    let before = surface.painted_quads(&mut cx);
    surface.scroll(&mut cx, 400.0, 50.0, 0.0, -500.0);
    surface.draw(&mut cx);
    let after = surface.painted_quads(&mut cx);
    let moved = before.iter().any(|quad| !after.contains(quad));
    assert!(
        moved,
        "gallery viewport did not move after a real wheel event: before={before:?} after={after:?}"
    );
}
fn gallery_layout_events(
    surface: &host::test_support::HeadlessSurface,
) -> HashMap<u32, (f32, f32, f32, f32)> {
    surface
        .events()
        .into_iter()
        .filter(|event| event.event_type == react_gpui::protocol::EVENT_LAYOUT)
        .filter_map(|event| match event.payload {
            Some(EventPayload::Layout {
                x,
                y,
                width,
                height,
            }) => Some((event.node_id, (x, y, width, height))),
            _ => None,
        })
        .collect()
}

fn send_renderer_drag_events(
    process: &mut RendererProcess,
    surface: &host::test_support::HeadlessSurface,
    cx: &mut TestAppContext,
) -> Vec<Patch> {
    let mut patches = Vec::new();
    for event in surface.events() {
        if !matches!(
            &event.payload,
            Some(EventPayload::DragOver { .. } | EventPayload::DragDrop { .. })
        ) {
            continue;
        }
        process.send_event(&event);
        let patch_payload = process.read_frame_with_timeout();
        let patch = Patch::decode(&patch_payload).expect("decode Gallery drag patch");
        surface.apply(cx, &patch_payload);
        surface.draw(cx);
        patches.push(patch);
    }
    patches
}

fn collect_gallery_layout_events(
    surface: &host::test_support::HeadlessSurface,
    cx: &mut TestAppContext,
    target_nodes: &[u32],
) -> HashMap<u32, (f32, f32, f32, f32)> {
    let mut layouts = HashMap::new();
    for _ in 0..8 {
        layouts.extend(gallery_layout_events(surface));
        if target_nodes
            .iter()
            .all(|node_id| layouts.contains_key(node_id))
        {
            return layouts;
        }
        surface.advance_frame(cx);
    }
    panic!("gallery layout events did not arrive for {target_nodes:?}: {layouts:?}");
}

fn assert_gallery_layout(
    snapshot: &Snapshot,
    surface: &host::test_support::HeadlessSurface,
    cx: &mut TestAppContext,
    width: f32,
    height: f32,
) {
    let root = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.style.as_ref().is_some_and(|style| {
                style.width == Some(800.0)
                    && style.height == Some(600.0)
                    && style.overflow == Some(3)
            })
        })
        .expect("gallery root");
    let body = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.parent_id == root.id
                && node.style.as_ref().is_some_and(|style| {
                    style.flex_direction.is_some()
                        && style.gap == Some(12.0)
                        && style.padding.is_none()
                })
        })
        .expect("gallery body");
    let panel_count = snapshot
        .nodes
        .iter()
        .filter(|node| {
            node.parent_id == body.id
                && node
                    .style
                    .as_ref()
                    .is_some_and(|style| style.border_radius == Some(12.0))
        })
        .count();
    assert_eq!(panel_count, 2, "gallery first-level panel count changed");

    let layouts = collect_gallery_layout_events(surface, cx, &[root.id]);
    let root_frame = layouts[&root.id];
    assert!(
        root_frame.0 >= -0.1
            && root_frame.1 >= -0.1
            && root_frame.2 <= width + 0.1
            && root_frame.3 >= height - 0.1,
        "{width}x{height} root frame escaped viewport bounds: {root_frame:?}"
    );

    let scale = surface.scale_factor(cx);
    let mut panel_bounds = panel_surface_bounds(&surface.painted_quads(cx), width, scale);
    panel_bounds.sort_by(|left, right| left.1.total_cmp(&right.1).then(left.0.total_cmp(&right.0)));
    assert_eq!(
        panel_bounds.len(),
        2,
        "{width}x{height} expected two un-clipped panel surfaces: {panel_bounds:?}"
    );
    let viewport_left = 20.0 * scale;
    let viewport_right = (width - 20.0) * scale;
    for bounds in &panel_bounds {
        assert!(
            bounds.0 >= viewport_left - 0.1 && bounds.0 + bounds.2 <= viewport_right + 0.1,
            "{width}x{height} panel surface escaped horizontal viewport before clipping: {bounds:?}"
        );
        assert!(
            (bounds.0 - viewport_left).abs() < 0.1
                && (bounds.2 - (width - 40.0) * scale).abs() < 0.1,
            "{width}x{height} compact panel surface is not full-width: {panel_bounds:?}"
        );
    }
    assert!(
        panel_bounds[1].1 >= panel_bounds[0].1 + panel_bounds[0].3 + 12.0 * scale - 0.1,
        "{width}x{height} compact panel surfaces overlap: {panel_bounds:?}"
    );
}

#[test]
fn gallery_layout_uses_vertical_scroll_without_horizontal_overflow_at_compact_widths() {
    let root = repo_root();
    for (width, height) in [(800.0, 600.0), (916.0, 588.0)] {
        let mut process = RendererProcess::spawn(&root, "packages/react-gpui/examples/gallery.tsx");
        let (first_payload, snapshot) = read_snapshot(&mut process)
            .expect("read gallery responsive snapshot")
            .expect("gallery responsive probe emitted no snapshot");
        let mut cx = TestAppContext::single();
        let surface = host::test_support::HeadlessSurface::new(&mut cx);
        surface.resize(&mut cx, width, height);
        surface.apply(&mut cx, &first_payload);
        surface.draw(&mut cx);
        if width != 800.0 {
            process.send_event(&Event::window_resize(
                snapshot.surface_id,
                snapshot.epoch,
                snapshot.revision,
                1,
                1,
                0,
                width,
                height,
            ));
            let patch_payload = process.read_frame_with_timeout();
            Patch::decode(&patch_payload).expect("decode gallery responsive resize patch");
            surface.apply(&mut cx, &patch_payload);
            surface.draw(&mut cx);
            surface.advance_frame(&mut cx);
        }
        assert_gallery_layout(&snapshot, &surface, &mut cx, width, height);
        let scale = surface.scale_factor(&mut cx);
        let max_right = surface
            .painted_quads(&mut cx)
            .into_iter()
            .map(|quad| quad.bounds.0 + quad.bounds.2)
            .fold(0.0_f32, f32::max);
        assert!(
            max_right <= width * scale + 0.1,
            "{width}x{height} painted content overflowed horizontally: max_right={max_right:.2}, viewport={:.2}",
            width * scale
        );
    }
}

#[test]
fn gallery_hover_updates_only_the_hovered_button_style() {
    let root = repo_root();
    let mut process = RendererProcess::spawn(&root, "packages/react-gpui/examples/gallery.tsx");
    let (_first_payload, snapshot) = read_snapshot(&mut process)
        .expect("read gallery hover snapshot")
        .expect("gallery hover probe emitted no snapshot");
    let menu_button = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.kind == KIND_PRESSABLE
                && node
                    .accessibility
                    .as_ref()
                    .and_then(|accessibility| accessibility.label.as_deref())
                    == Some("Show activity menu")
        })
        .expect("gallery menu button");
    let activate_button = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.kind == KIND_PRESSABLE
                && node
                    .accessibility
                    .as_ref()
                    .and_then(|accessibility| accessibility.label.as_deref())
                    == Some("Activate live panel")
        })
        .expect("gallery activate button");
    process.send_event(&Event::hover(
        snapshot.surface_id,
        snapshot.epoch,
        snapshot.revision,
        1,
        menu_button.id,
        menu_button.listener_id,
    ));
    let patch_payload = process.read_frame_with_timeout();
    let patch = Patch::decode(&patch_payload).expect("decode gallery hover patch");
    let updated_style = |node_id| {
        patch
            .operations
            .iter()
            .find_map(|operation| match operation {
                PatchOperation::Update {
                    id,
                    mask,
                    style: Some(style),
                    ..
                } if *id == node_id && mask & react_gpui::UPDATE_STYLE != 0 => Some(style),
                _ => None,
            })
    };
    let menu_style = updated_style(menu_button.id).expect("hovered menu button style patch");
    assert_eq!(menu_style.background_rgba, Some(0x2458b8ff));
    let activate_style = updated_style(activate_button.id).expect("activate button style patch");
    assert_eq!(
        activate_style.background_rgba,
        Some(0x2d6cdfff),
        "activate button changed while hovering menu: {patch:?}"
    );
}
#[test]
fn gallery_page_scroll_reveals_activity_controls_and_nested_wheel_scrolls_rows() {
    let root = repo_root();
    let mut process = RendererProcess::spawn(&root, "packages/react-gpui/examples/gallery.tsx");
    let (first_payload, snapshot) = read_snapshot(&mut process)
        .expect("read gallery nested scroll snapshot")
        .expect("gallery nested scroll probe emitted no snapshot");
    let gallery_root = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.style.as_ref().is_some_and(|style| {
                style.width == Some(800.0)
                    && style.height == Some(600.0)
                    && style.overflow == Some(3)
            })
        })
        .expect("gallery root");
    let body = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.parent_id == gallery_root.id
                && node
                    .style
                    .as_ref()
                    .is_some_and(|style| style.flex_direction == Some(2) && style.gap == Some(12.0))
        })
        .expect("compact gallery body");
    let mut panels = snapshot
        .nodes
        .iter()
        .filter(|node| node.parent_id == body.id)
        .collect::<Vec<_>>();
    panels.sort_by_key(|node| node.index);
    assert_eq!(panels.len(), 2, "gallery activity panel is absent");
    let panel_children = snapshot
        .nodes
        .iter()
        .filter(|node| node.parent_id == panels[1].id)
        .collect::<Vec<_>>();
    panel_children
        .iter()
        .find(|node| node.index == 0)
        .expect("activity heading container");
    panel_children
        .iter()
        .find(|node| node.index == 1 && node.kind == react_gpui::KIND_VIRTUAL_LIST)
        .expect("activity virtual list");
    let record_button = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.kind == KIND_PRESSABLE
                && node.listener_id != 0
                && node
                    .accessibility
                    .as_ref()
                    .and_then(|accessibility| accessibility.label.as_deref())
                    == Some("Record press")
        })
        .expect("record button");

    let mut cx = TestAppContext::single();
    let surface = host::test_support::HeadlessSurface::new(&mut cx);
    surface.resize(&mut cx, 800.0, 600.0);
    surface.apply(&mut cx, &first_payload);
    surface.draw(&mut cx);
    let scale = surface.scale_factor(&mut cx);
    let initial_quads = surface.painted_quads(&mut cx);
    let initial_panels = panel_surface_bounds(&initial_quads, 800.0, scale);

    surface.scroll(&mut cx, 400.0, 590.0, 0.0, -500.0);
    surface.draw(&mut cx);
    surface.advance_frame(&mut cx);
    let page_quads = surface.painted_quads(&mut cx);
    let mut page_panels = panel_surface_bounds(&page_quads, 800.0, scale);
    page_panels.sort_by(|left, right| left.1.total_cmp(&right.1));
    assert_eq!(
        page_panels.len(),
        2,
        "page scroll lost a panel surface: {page_panels:?}"
    );
    assert!(
        initial_panels != page_panels,
        "page wheel did not move the actual panel surfaces: initial={initial_panels:?} page={page_panels:?}"
    );
    assert!(
        page_panels
            .iter()
            .any(|bounds| { bounds.1 < 600.0 * scale && bounds.1 + bounds.3 > 0.0 }),
        "page wheel did not reveal an actual activity panel surface: {page_panels:?}"
    );

    let mut page_rows = activity_row_bounds(&page_quads, 800.0, scale);
    page_rows.sort_by(|left, right| left.1.total_cmp(&right.1));
    assert!(
        page_rows.len() >= 2,
        "page wheel did not reveal enough painted activity rows for gap probe: {page_quads:?}"
    );
    for pair in page_rows.windows(2) {
        let gap = (pair[1].1 - (pair[0].1 + pair[0].3)) / scale;
        assert!(
            (pair[0].3 / scale - 44.0).abs() < 0.1 && gap >= 8.0 - 0.1,
            "activity cards must be 44px tall with an 8px visible gap: rows={page_rows:?}"
        );
    }
    let record_quad = record_button_bounds(&page_quads, 600.0, scale)
        .expect("page wheel did not reveal the painted Record press hitbox");
    surface.click(
        &mut cx,
        (record_quad.0 + record_quad.2 / 2.0) / scale,
        (record_quad.1 + record_quad.3 / 2.0) / scale,
    );
    let click_events = surface.events();
    let pointer_event = |action| {
        click_events.iter().any(|event| {
            event.node_id == record_button.id
                && event.event_type == EVENT_POINTER
                && matches!(
                    &event.payload,
                    Some(EventPayload::Pointer(pointer)) if pointer.action == action
                )
        })
    };
    assert!(
        pointer_event(EVENT_POINTER_DOWN) && pointer_event(EVENT_POINTER_UP),
        "painted Record press hitbox did not target its actual node: quad={record_quad:?} events={click_events:?}"
    );

    process.send_event(&Event::press(
        snapshot.surface_id,
        snapshot.epoch,
        snapshot.revision,
        1,
        record_button.id,
        record_button.listener_id,
    ));
    let press_patch_payload = process.read_frame_with_timeout();
    let press_patch = Patch::decode(&press_patch_payload).expect("decode record press patch");
    assert!(
        press_patch.operations.iter().any(|operation| {
            matches!(
                operation,
                PatchOperation::Update {
                    text: Some(text),
                    ..
                } if text == "1"
            )
        }),
        "Record press did not produce the presses state patch: {press_patch:?}"
    );

    let row_probe = page_rows[0];
    surface.scroll(
        &mut cx,
        (row_probe.0 + row_probe.2 / 2.0) / scale,
        (row_probe.1 + row_probe.3 / 2.0) / scale,
        0.0,
        -220.0,
    );
    surface.draw(&mut cx);
    surface.advance_frame(&mut cx);
    let nested_quads = surface.painted_quads(&mut cx);
    let mut nested_rows = activity_row_bounds(&nested_quads, 800.0, scale);
    nested_rows.sort_by(|left, right| left.1.total_cmp(&right.1));
    assert!(
        !nested_rows.is_empty() && page_rows != nested_rows,
        "nested wheel did not change painted VirtualList rows: page_rows={page_rows:?} nested_rows={nested_rows:?}"
    );
    let mut nested_panels = panel_surface_bounds(&nested_quads, 800.0, scale);
    nested_panels.sort_by(|left, right| left.1.total_cmp(&right.1));
    assert_eq!(
        page_panels, nested_panels,
        "nested VirtualList wheel changed outer page/panel scene bounds: page={page_panels:?} nested={nested_panels:?}"
    );
}
#[test]
fn gallery_drag_preview_is_neutral_and_compact() {
    let root = repo_root();
    let mut process = RendererProcess::spawn(&root, "packages/react-gpui/examples/gallery.tsx");
    let (first_payload, _) = read_snapshot(&mut process)
        .expect("read gallery drag preview snapshot")
        .expect("gallery drag preview probe emitted no snapshot");
    let mut cx = TestAppContext::single();
    let surface = host::test_support::HeadlessSurface::new(&mut cx);
    surface.resize(&mut cx, 800.0, 600.0);
    surface.apply(&mut cx, &first_payload);
    surface.draw(&mut cx);
    let scale = surface.scale_factor(&mut cx);
    surface.scroll(&mut cx, 400.0, 590.0, 0.0, -500.0);
    surface.draw(&mut cx);
    surface.advance_frame(&mut cx);
    let before_drag = surface.painted_quads(&mut cx);
    let rows = activity_row_bounds(&before_drag, 800.0, scale);
    assert!(
        !rows.is_empty(),
        "gallery drag preview needs a visible activity row"
    );
    let row = rows[0];
    let from = ((row.0 + row.2 / 2.0) / scale, (row.1 + row.3 / 2.0) / scale);
    let to = (from.0 + 20.0, from.1);
    surface.begin_drag(&mut cx, from, to);
    surface.draw(&mut cx);
    let after_drag = surface.painted_quads(&mut cx);
    let added = after_drag
        .iter()
        .filter(|quad| !before_drag.contains(quad))
        .collect::<Vec<_>>();
    assert!(
        !added.iter().any(|quad| {
            (quad.bounds.2 - 24.0 * scale).abs() < 0.1 && (quad.bounds.3 - 24.0 * scale).abs() < 0.1
        }),
        "drag preview must not be the legacy 24px square: added={added:?}"
    );
    let preview = added
        .iter()
        .find(|quad| {
            opaque_quad(quad)
                && quad.bounds.2 > quad.bounds.3
                && quad.bounds.2 < 240.0 * scale
                && quad.bounds.3 < 80.0 * scale
        })
        .expect("drag preview should add a compact neutral labeled surface");
    assert!(
        !preview.background.contains("s: 1.0"),
        "drag preview surface should use a neutral fill: {preview:?}"
    );
    assert!(
        (preview.bounds.2 / scale - 96.0).abs() < 0.1
            && (preview.bounds.3 / scale - 32.0).abs() < 0.1,
        "drag preview geometry changed: {preview:?}"
    );
    let pointer = (to.0 * scale, to.1 * scale);
    assert!(
        ((preview.bounds.0 + preview.bounds.2 / 2.0) - pointer.0).abs() < 0.1
            && ((preview.bounds.1 + preview.bounds.3 / 2.0) - pointer.1).abs() < 0.1,
        "drag preview should stay centered under the pointer: preview={preview:?} pointer={pointer:?}"
    );
    surface.end_drag(&mut cx, from);
}
#[test]
fn gallery_drag_drop_reorders_rows_and_preserves_drag_over_feedback() {
    let root = repo_root();
    let mut process = RendererProcess::spawn(&root, "packages/react-gpui/examples/gallery.tsx");
    let (first_payload, _snapshot) = read_snapshot(&mut process)
        .expect("read gallery drag-drop snapshot")
        .expect("gallery drag-drop probe emitted no snapshot");
    let mut cx = TestAppContext::single();
    let surface = host::test_support::HeadlessSurface::new(&mut cx);
    surface.resize(&mut cx, 800.0, 600.0);
    surface.apply(&mut cx, &first_payload);
    surface.draw(&mut cx);
    let scale = surface.scale_factor(&mut cx);
    surface.scroll(&mut cx, 400.0, 590.0, 0.0, -500.0);
    surface.draw(&mut cx);
    surface.advance_frame(&mut cx);
    let page_quads = surface.painted_quads(&mut cx);
    let mut rows = activity_row_bounds(&page_quads, 800.0, scale);
    rows.sort_by(|left, right| left.1.total_cmp(&right.1));
    assert!(
        rows.len() >= 2,
        "gallery drag-drop needs two visible rows: {page_quads:?}"
    );
    let source = rows[0];
    let target = rows[1];
    let source_center = (
        (source.0 + source.2 / 2.0) / scale,
        (source.1 + source.3 / 2.0) / scale,
    );
    let target_center = (
        (target.0 + target.2 / 2.0) / scale,
        (target.1 + target.3 / 2.0) / scale,
    );

    surface.begin_drag(
        &mut cx,
        source_center,
        (source_center.0 + 20.0, source_center.1),
    );
    surface.move_drag(&mut cx, target_center);
    let over_patches = send_renderer_drag_events(&mut process, &surface, &mut cx);
    assert!(
        over_patches.iter().any(|patch| {
            patch.operations.iter().any(|operation| {
                matches!(
                    operation,
                    PatchOperation::Update {
                        style: Some(style),
                        ..
                    } if style.background_rgba == Some(0xeaf1ffff)
                )
            })
        }),
        "dragging over a row must apply its target feedback style: {over_patches:?}"
    );
    let after_over_quads = surface.painted_quads(&mut cx);
    let before_target = page_quads
        .iter()
        .find(|quad| quad.bounds == target && opaque_quad(quad))
        .expect("target row surface before drag-over");
    let after_target = after_over_quads
        .iter()
        .find(|quad| quad.bounds == target && opaque_quad(quad))
        .expect("target row surface after drag-over");
    assert_ne!(
        before_target.background, after_target.background,
        "drag-over should visibly style the target row"
    );

    surface.end_drag(&mut cx, target_center);
    let drop_patches = send_renderer_drag_events(&mut process, &surface, &mut cx);
    assert!(
        drop_patches.iter().any(|patch| {
            patch
                .operations
                .iter()
                .any(|operation| matches!(operation, PatchOperation::Move { .. }))
        }),
        "dropping on a different row must reorder keyed Gallery children: {drop_patches:?}"
    );
}

#[test]
fn all_examples_render_readable_text_and_gallery_dropdown_above_siblings() {
    let root = repo_root();
    let mut audits = Vec::new();
    for &(name, entry) in EXAMPLES {
        let mut process = RendererProcess::spawn(&root, entry);
        let (first_payload, snapshot) = match read_snapshot(&mut process) {
            Ok(Some(first)) => first,
            Ok(None) => {
                audits.push(ExampleAudit {
                    name,
                    startup_error: Some("renderer emitted no startup snapshot".to_owned()),
                    text: None,
                    dropdown_signal: None,
                    dropdown_error: None,
                });
                continue;
            }
            Err(error) => {
                audits.push(ExampleAudit {
                    name,
                    startup_error: Some(error),
                    text: None,
                    dropdown_signal: None,
                    dropdown_error: None,
                });
                continue;
            }
        };
        let mut cx = TestAppContext::single();
        let surface = host::test_support::HeadlessSurface::new(&mut cx);
        surface.apply(&mut cx, &first_payload);
        surface.draw(&mut cx);
        let quads = surface.painted_quads(&mut cx);
        let text = text_signal(&snapshot, &quads);
        let (dropdown_signal, dropdown_error) = if name == "gallery" {
            match gallery_dropdown_signal(&mut process, &snapshot, &surface, &mut cx) {
                Ok(signal) => (Some(signal), None),
                Err(error) => (None, Some(error)),
            }
        } else {
            (Some("not-applicable".to_owned()), None)
        };
        audits.push(ExampleAudit {
            name,
            startup_error: None,
            text: Some(text),
            dropdown_signal,
            dropdown_error,
        });
    }
    let failed = audits
        .iter()
        .filter(|audit| {
            audit.startup_error.is_some()
                || audit.text.as_ref().is_some_and(|text| {
                    text.text_nodes == 0
                        || text.opaque_backgrounds < text.text_nodes
                        || !text.unreadable.is_empty()
                })
                || audit.dropdown_error.is_some()
        })
        .map(|audit| {
            format!(
                "{}: startup={:?} text={:?} dropdown={:?} dropdown_error={:?}",
                audit.name,
                audit.startup_error,
                audit.text,
                audit.dropdown_signal,
                audit.dropdown_error
            )
        })
        .collect::<Vec<_>>();
    assert!(
        failed.is_empty(),
        "example rendered audit failed: {failed:?}"
    );
}
