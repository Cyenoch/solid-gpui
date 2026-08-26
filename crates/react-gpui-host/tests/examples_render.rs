#![cfg(target_os = "macos")]

#[allow(dead_code)]
#[path = "../src/main.rs"]
mod host;

use gpui::TestAppContext;
use react_gpui::{
    EVENT_POINTER, EVENT_POINTER_DOWN, EVENT_POINTER_UP, Event, EventPayload, KIND_PRESSABLE,
    KIND_TEXT, KIND_VIEW, Node, Patch, PatchOperation, Snapshot, Style, read_frame, write_frame,
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
                        style.width == Some(200.0)
                            && style.background_rgba == Some(0xffff_ffff)
                    }) => {
                Some(node.id)
            }
            _ => None,
        })
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
        .cloned()
        .filter(|quad| {
            (quad.bounds.2 - 80.0 * scale).abs() < 0.01
                && (quad.bounds.3 - 40.0 * scale).abs() < 0.01
        })
        .collect::<Vec<_>>();
    let coordinate = |bounds: (f32, f32, f32, f32)| {
        if axis == 0 {
            (bounds.0, bounds.2)
        } else {
            (bounds.1, bounds.3)
        }
    };
    quads.sort_by(|left, right| coordinate(left.bounds).0.total_cmp(&coordinate(right.bounds).0));
    assert_eq!(quads.len(), 2, "{label} gap probe rendered unexpected quads: {quads:?}");
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
fn gallery_layout_events(surface: &host::test_support::HeadlessSurface) -> HashMap<u32, (f32, f32, f32, f32)> {
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

fn collect_gallery_layout_events(
    surface: &host::test_support::HeadlessSurface,
    cx: &mut TestAppContext,
    target_nodes: &[u32],
) -> HashMap<u32, (f32, f32, f32, f32)> {
    let mut layouts = HashMap::new();
    for _ in 0..8 {
        layouts.extend(gallery_layout_events(surface));
        if target_nodes.iter().all(|node_id| layouts.contains_key(node_id)) {
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
                style.width == Some(800.0) && style.height == Some(600.0) && style.overflow == Some(3)
            })
        })
        .expect("gallery root");
    let body = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.parent_id == root.id
                && node.style.as_ref().is_some_and(|style| {
                    style.flex_direction.is_some() && style.gap == Some(12.0) && style.padding.is_none()
                })
        })
        .expect("gallery body");
    let panels = snapshot
        .nodes
        .iter()
        .filter(|node| {
            node.parent_id == body.id
                && node
                    .style
                    .as_ref()
                    .is_some_and(|style| style.border_radius == Some(12.0))
        })
        .map(|node| node.id)
        .collect::<Vec<_>>();
    let mut target_nodes = vec![root.id, body.id];
    target_nodes.extend(panels.iter().copied());
    let layouts = collect_gallery_layout_events(surface, cx, &target_nodes);
    assert_eq!(
        panels.len(),
        2,
        "gallery first-level panel count changed: body={body:?} direct={:?}",
        snapshot.nodes.iter().filter(|node| node.parent_id == body.id).collect::<Vec<_>>()
    );
    let frame = |node_id: u32| layouts.get(&node_id).copied().unwrap_or_else(|| {
        panic!("gallery node {node_id} emitted no layout event; events={layouts:?}")
    });
    let root_frame = frame(root.id);
    assert!(
        root_frame.0 >= -0.1
            && root_frame.1 >= -0.1
            && root_frame.2 <= width + 0.1
            && root_frame.3 >= height - 0.1,
        "{width}x{height} root frame escaped viewport bounds: {root_frame:?}"
    );
    let body_frame = frame(body.id);
    assert!(
        (body_frame.0 - 20.0).abs() < 0.1
            && (body_frame.1 - 96.0).abs() < 0.1
            && (body_frame.2 - (width - 40.0)).abs() < 0.1,
        "{width}x{height} body frame escaped inset: {body_frame:?}"
    );
    let mut panel_frames = panels.iter().map(|id| frame(*id)).collect::<Vec<_>>();
    panel_frames.sort_by(|left, right| left.1.total_cmp(&right.1).then(left.0.total_cmp(&right.0)));
    for panel in &panel_frames {
        assert!(
            panel.0 >= 20.0 - 0.1 && panel.0 + panel.2 <= width - 20.0 + 0.1,
            "{width}x{height} panel escaped horizontal viewport: {panel:?}"
        );
    }
    if width < 1100.0 {
        assert!(
            panel_frames.iter().all(|panel| {
                (panel.0 - 20.0).abs() < 0.1 && (panel.2 - (width - 40.0)).abs() < 0.1
            }),
            "{width}x{height} compact panels are not full-width: {panel_frames:?}"
        );
        assert!(
            panel_frames[1].1 >= panel_frames[0].1 + panel_frames[0].3 + 12.0 - 0.1,
            "{width}x{height} compact panels overlap vertically: {panel_frames:?}"
        );
    } else {
        assert!(
            (panel_frames[0].1 - body_frame.1).abs() < 0.1
                && (panel_frames[1].1 - body_frame.1).abs() < 0.1
                && panel_frames[1].0 >= panel_frames[0].0 + panel_frames[0].2 + 12.0 - 0.1,
            "{width}x{height} wide panels lost their horizontal gap: {panel_frames:?}"
        );
    }
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
