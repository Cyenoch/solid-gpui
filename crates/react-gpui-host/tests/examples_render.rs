#![cfg(target_os = "macos")]

#[allow(dead_code)]
#[path = "../src/main.rs"]
mod host;

use gpui::TestAppContext;
use react_gpui::{
    EVENT_POINTER, EVENT_POINTER_DOWN, EVENT_POINTER_UP, Event, EventPayload, KIND_PRESSABLE,
    KIND_TEXT, Patch, PatchOperation, Snapshot, read_frame, write_frame,
};
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
                        style.position == Some(2)
                            && style.width == Some(180.0)
                            && style.top == Some(42.0)
                            && style.background_rgba == Some(0xffff_ffff)
                    }) =>
            {
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
    surface.resize(cx, 800.0, 800.0);
    surface.apply(cx, &patch_payload);
    surface.draw(cx);

    let quads = surface.painted_quads(cx);
    let Some(menu_quad) = quads
        .iter()
        .filter(|quad| quad.bounds.2 >= 340.0 && quad.bounds.2 <= 380.0)
        .filter(|quad| quad.bounds.3 >= 120.0 && quad.bounds.3 <= 220.0)
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
