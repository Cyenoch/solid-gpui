use gpui::{AnyWindowHandle, Entity, WindowOptions};
use gpui::{Bounds, ScrollDelta, ScrollWheelEvent, TestAppContext, WindowBounds, point, px, size};
use gpui_component::Root;
use solid_gpui::SolidRoot;
use solid_gpui::components::host::ComponentHost;
use solid_gpui::host::HostProfile;
use solid_gpui::protocol::EventPayload;
use solid_gpui::{InMemoryAdapter, Snapshot};
use std::collections::HashMap;

fn gallery_host_profile() -> ComponentHost {
    ComponentHost::new(vec![
        solid_gpui::components::native_module(),
        gallery_host::native_module(),
    ])
}

fn gallery_snapshot(
    width: f32,
    height: f32,
    theme: &str,
    route: &str,
) -> (Snapshot, Vec<Vec<Vec<u8>>>) {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .unwrap();
    let script = r#"
import { writeSync } from 'node:fs';
import { MemoryTransport, createRoot } from './packages/solid-gpui/dist/index.js';
import { createComponent } from './packages/solid-gpui/dist/runtime.js';
import { RouterProvider } from './packages/solid-gpui-router/dist/index.js';
import { createGalleryState } from './examples/gallery/src/gallery/context.ts';
import { createGalleryRouter } from './examples/gallery/src/gallery/routes.ts';
setTimeout(() => { console.error('Gallery snapshot timed out'); process.exit(1); }, 10000);
const transport = new MemoryTransport();
const gallery = createGalleryState(process.env.GALLERY_TEST_THEME);
gallery.windowSizeStore.set(Number(process.env.GALLERY_TEST_WIDTH), Number(process.env.GALLERY_TEST_HEIGHT), 1);
const root = createRoot(transport, { surfaceId: 1, epoch: 1 });
gallery.setRoot(root);
const router = createGalleryRouter([process.env.GALLERY_TEST_ROUTE]);
await router.load();
root.render(() => createComponent(RouterProvider, { router }));
if (!transport.submitted[0]) throw new Error('Gallery emitted no snapshot');
const stages = [transport.submitted.splice(0)];
await router.load();
await new Promise(resolve => setTimeout(resolve, 0));
stages.push(transport.submitted.splice(0));
for (const w of [1280, 800, 1280, 1680]) {
  gallery.windowSizeStore.set(w, Number(process.env.GALLERY_TEST_HEIGHT), 1);
  await new Promise(resolve => setTimeout(resolve, 0));
  stages.push(transport.submitted.splice(0));
}
await router.navigate({ to: '/drag-drop' });
await new Promise(resolve => setTimeout(resolve, 0));
stages.push(transport.submitted.splice(0));
for (const frames of stages) {
  const length = Buffer.alloc(4); length.writeUInt32LE(frames.length);
  writeSync(1, length);
  for (const frame of frames) writeSync(1, frame);
}
process.exit(0);
"#;
    let output = std::process::Command::new("bun")
        .current_dir(repo)
        .env("GALLERY_TEST_WIDTH", width.to_string())
        .env("GALLERY_TEST_HEIGHT", height.to_string())
        .env("GALLERY_TEST_THEME", theme)
        .env("GALLERY_TEST_ROUTE", route)
        .args([
            "--conditions=browser",
            "--preload",
            "./scripts/solid-jsx.ts",
            "-e",
            script,
        ])
        .output()
        .expect("capture actual routed gallery");
    assert!(
        output.status.success(),
        "Gallery capture failed: {}",
        String::from_utf8_lossy(&output.stderr[..output.stderr.len().min(4096)])
    );
    assert!(
        output.stdout.len() > 4,
        "Gallery emitted no framed snapshot"
    );
    let mut bytes = output.stdout.as_slice();
    let mut stages = Vec::new();
    while !bytes.is_empty() {
        let count = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        bytes = &bytes[4..];
        let mut frames = Vec::new();
        for _ in 0..count {
            let len = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
            frames.push(bytes[4..4 + len].to_vec());
            bytes = &bytes[4 + len..];
        }
        stages.push(frames);
    }
    let initial = stages.remove(0);
    assert_eq!(initial.len(), 1, "initial render emits one snapshot");
    (
        Snapshot::decode(&initial[0]).expect("decode actual gallery snapshot"),
        stages,
    )
}

fn apply_gallery_payload(
    cx: &mut TestAppContext,
    window: AnyWindowHandle,
    root: &Entity<SolidRoot>,
    payload: &[u8],
) {
    let message = solid_gpui::decode_message(payload).expect("decode live gallery update");
    window
        .update(cx, |_, window, cx| {
            root.update(cx, |root, cx| {
                root.apply_decoded_message_in_window(message, window, cx)
            })
        })
        .expect("gallery window remains alive")
        .expect("apply routed gallery through foreground host");
}

#[derive(Clone, Copy, Debug)]
struct Frame {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}
impl Frame {
    fn bottom(self) -> f32 {
        self.y + self.height
    }
}

fn draw(
    cx: &mut TestAppContext,
    window: AnyWindowHandle,
    runtime: &InMemoryAdapter,
    frames: &mut HashMap<u32, Frame>,
) {
    window
        .update(cx, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    finish_frame(cx, window, runtime, frames);
}

fn finish_frame(
    cx: &mut TestAppContext,
    window: AnyWindowHandle,
    runtime: &InMemoryAdapter,
    frames: &mut HashMap<u32, Frame>,
) {
    window
        .update(cx, |_, window, cx| window.simulate_next_frame(cx))
        .unwrap();
    cx.run_until_parked();
    while let Some(event) = runtime.take_event().expect("read gallery layout events") {
        if let EventPayload::Layout {
            x,
            y,
            width,
            height,
        } = event.payload
        {
            frames.insert(
                event.meta.node_id,
                Frame {
                    x,
                    y,
                    width,
                    height,
                },
            );
        }
    }
}

fn wheel(cx: &mut TestAppContext, window: AnyWindowHandle, pane: Frame, delta: f32) {
    let mut visual = gpui::VisualTestContext::from_window(window, cx);
    visual.simulate_event(ScrollWheelEvent {
        position: point(
            px(pane.x + pane.width / 2.0),
            px(pane.y + pane.height / 2.0),
        ),
        delta: ScrollDelta::Pixels(point(px(0.0), px(delta))),
        ..Default::default()
    });
}

fn unchanged(before: Frame, after: Frame) {
    assert!(
        (before.x - after.x).abs() < 1.0
            && (before.y - after.y).abs() < 1.0
            && (before.width - after.width).abs() < 1.0
            && (before.height - after.height).abs() < 1.0,
        "unrelated pane/chrome moved: before={before:?}, after={after:?}"
    );
}

#[gpui::test]
fn gallery_panes_scroll_independently(cx: &mut TestAppContext) {
    // Match production: initialize provider globals once per application,
    // not once per route/window in a multi-page audit.
    cx.update(|app| gallery_host_profile().initialize(app));
    // Optional exhaustive audit uses the same correctness and timing loop.
    if let Ok(routes) = std::env::var("SOLID_GPUI_GALLERY_ROUTES") {
        let width = std::env::var("SOLID_GPUI_GALLERY_WIDTH")
            .map(|v| v.parse::<f32>().expect("Gallery audit width"))
            .unwrap_or(800.0);
        assert!(width.is_finite() && width > 0.0);
        for route in routes.split(',') {
            assert!(route.starts_with('/'), "absolute Gallery route");
            exercise_gallery_panes(cx, width, 600.0, "light", route);
        }
        return;
    }
    for (width, height, theme, route) in [
        (800.0, 600.0, "dark", "/"),
        (560.0, 600.0, "light", "/"),
        (1280.0, 600.0, "dark", "/"),
        (1280.0, 600.0, "dark", "/drag-drop"),
        (800.0, 600.0, "light", "/drag-drop"),
        (1680.0, 900.0, "light", "/"),
    ] {
        exercise_gallery_panes(cx, width, height, theme, route);
    }
}

fn exercise_gallery_panes(
    cx: &mut TestAppContext,
    width: f32,
    height: f32,
    theme: &str,
    route: &str,
) {
    let (mut snapshot, updates) = gallery_snapshot(width, height, theme, route);
    eprintln!(
        "gallery {route} {width}x{height} {theme} fixture: nodes={}, text_nodes={}, raw_text_nodes={}",
        snapshot.nodes.len(),
        snapshot
            .nodes
            .iter()
            .filter(|node| node.kind == solid_gpui::KIND_TEXT)
            .count(),
        snapshot
            .nodes
            .iter()
            .filter(|node| node.kind == solid_gpui::KIND_RAW_TEXT)
            .count()
    );
    let labeled = |label: &str| {
        snapshot
            .nodes
            .iter()
            .find(|node| {
                node.accessibility.as_ref().and_then(|a| a.label.as_deref()) == Some(label)
            })
            .unwrap_or_else(|| panic!("missing gallery accessibility label {label}"))
            .id
    };
    let navigation = labeled("Component navigation");
    let content = labeled("Showcase content");
    let theme_button = labeled("Toggle color theme");
    let parents: HashMap<_, _> = snapshot.nodes.iter().map(|n| (n.id, n.parent_id)).collect();
    let descendant = |mut id: u32, ancestor: u32| {
        while let Some(&parent) = parents.get(&id) {
            if parent == ancestor {
                return true;
            }
            if parent == 0 {
                break;
            }
            id = parent;
        }
        false
    };
    let nav_end = snapshot
        .nodes
        .iter()
        .rfind(|n| {
            descendant(n.id, navigation)
                && n.accessibility
                    .as_ref()
                    .and_then(|a| a.label.as_ref())
                    .is_some()
        })
        .expect("last accessible gallery route")
        .id;
    let content_end = snapshot
        .nodes
        .iter()
        .rfind(|n| descendant(n.id, content) && n.kind == solid_gpui::KIND_TEXT)
        .expect("last real content text")
        .id;
    let footer = snapshot
        .nodes
        .iter()
        .find(|n| n.text.as_deref() == Some("Native surface"))
        .map(|node| {
            if node.kind == solid_gpui::KIND_TEXT {
                node.id
            } else {
                node.parent_id
            }
        })
        .expect("gallery footer");
    assert!(
        descendant(content_end, content),
        "overview must be routed inside content pane"
    );
    let nav_child = snapshot
        .nodes
        .iter()
        .find(|n| n.parent_id == navigation && n.kind == solid_gpui::KIND_VIEW)
        .unwrap()
        .id;
    let content_child = snapshot
        .nodes
        .iter()
        .find(|n| n.parent_id == content && n.kind == solid_gpui::KIND_VIEW)
        .unwrap()
        .id;
    let tracked = [
        navigation,
        content,
        nav_child,
        content_child,
        nav_end,
        content_end,
        theme_button,
        footer,
    ];
    // Only observe existing nodes; preserve the gallery's full topology, styles and providers.
    for node in &mut snapshot.nodes {
        if tracked.contains(&node.id) && node.listener_id == 0 {
            node.listener_id = node.id;
        }
    }
    let runtime = InMemoryAdapter::new();
    let profile = gallery_host_profile();
    let extensions = profile.extension_registry();
    let (window, root) = cx.update(|app| {
        profile
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(width), px(height)),
                        app,
                    ))),
                    ..Default::default()
                },
                runtime.clone(),
                extensions,
                app,
            )
            .expect("open real provider host")
    });
    assert!(
        window.downcast::<Root>().is_some(),
        "exercise gpui-component Root wrapper"
    );
    apply_gallery_payload(cx, window, &root, &snapshot.encode().unwrap());
    let mut frames = HashMap::new();
    draw(cx, window, &runtime, &mut frames);
    draw(cx, window, &runtime, &mut frames);
    for id in tracked {
        assert!(
            frames.contains_key(&id),
            "missing layout for tracked gallery node {id}"
        );
    }
    if route == "/drag-drop" {
        for (index, delta) in [-1.0, -2.0, -4.0, -8.0, -16.0, -32.0, -40.0, -40.0]
            .into_iter()
            .enumerate()
        {
            let before_y = frames[&content_child].y;
            let pane = frames[&content];
            let start = std::time::Instant::now();
            let mut visual = gpui::VisualTestContext::from_window(window, cx);
            visual.simulate_event(ScrollWheelEvent {
                position: point(
                    px(pane.x + pane.width / 2.0),
                    px(pane.y + pane.height / 2.0),
                ),
                delta: ScrollDelta::Pixels(point(px(0.0), px(delta))),
                touch_phase: if index == 0 {
                    gpui::TouchPhase::Started
                } else {
                    gpui::TouchPhase::Moved
                },
                ..Default::default()
            });
            finish_frame(cx, window, &runtime, &mut frames);
            eprintln!(
                "gallery {route} {width} cold wheel {index}: delta={delta}, moved={}, cpu_ms={:.3}",
                before_y - frames[&content_child].y,
                start.elapsed().as_secs_f64() * 1000.0
            );
            assert!(
                (frames[&content_child].y - before_y - delta).abs() < 0.5,
                "initial gesture must consume every small delta immediately"
            );
        }
        wheel(cx, window, frames[&content], 100_000.0);
        draw(cx, window, &runtime, &mut frames);
    }
    let initial = frames.clone();
    for (pane, child, marker) in [
        (navigation, nav_child, nav_end),
        (content, content_child, content_end),
    ] {
        let viewport = frames[&pane];
        assert!(
            viewport.width > 0.0
                && viewport.height > 0.0
                && viewport.x >= 0.0
                && viewport.x + viewport.width <= width + 1.0
                && viewport.y >= frames[&theme_button].bottom()
                && viewport.bottom() <= height + 1.0
                && viewport.bottom() <= frames[&footer].y + 1.0,
            "gallery pane escapes viewport/header/footer: {viewport:?}"
        );
        assert!(frames[&child].height > 0.0, "pane content must have height");
        if frames[&child].height > viewport.height + 1.0 {
            assert!(
                frames[&marker].bottom() > viewport.bottom(),
                "overflow end marker must require scrolling"
            );
        }
    }
    for (pane, child, marker, other_child) in [
        (navigation, nav_child, nav_end, content_child),
        (content, content_child, content_end, nav_child),
    ] {
        let before = frames.clone();
        wheel(cx, window, frames[&pane], -120.0);
        draw(cx, window, &runtime, &mut frames);
        let overflow = before[&child].height - before[&pane].height;
        if overflow <= 1.0 {
            unchanged(before[&child], frames[&child]);
        } else {
            assert!(
                frames[&marker].y < before[&marker].y - overflow.min(20.0) * 0.9,
                "wheel must move overflowing pane descendants"
            );
        }
        unchanged(before[&other_child], frames[&other_child]);
        for fixed in [navigation, content, theme_button, footer] {
            unchanged(initial[&fixed], frames[&fixed]);
        }
        wheel(cx, window, frames[&pane], -100_000.0);
        draw(cx, window, &runtime, &mut frames);
        // Bottom padding can make an otherwise fitting final text begin above
        // the viewport at the absolute end. Verify it is reachable in full,
        // rather than requiring that one particular offset fit it.
        if frames[&marker].height <= frames[&pane].height && frames[&marker].y < frames[&pane].y {
            wheel(
                cx,
                window,
                frames[&pane],
                frames[&pane].y - frames[&marker].y,
            );
            draw(cx, window, &runtime, &mut frames);
        }
        assert!(
            frames[&marker].bottom() > frames[&pane].y
                && frames[&marker].bottom() <= frames[&pane].bottom() + 1.0
                && (frames[&marker].height > frames[&pane].height
                    || frames[&marker].y >= frames[&pane].y - 1.0),
            "final gallery marker must become fully visible: route={route}, pane={:?}, marker={:?}, content={:?}",
            frames[&pane],
            frames[&marker],
            frames[&child]
        );
        unchanged(before[&other_child], frames[&other_child]);
        for fixed in [navigation, content, theme_button, footer] {
            unchanged(initial[&fixed], frames[&fixed]);
        }
    }
    measure_panes(
        cx,
        window,
        &runtime,
        &mut frames,
        [
            ("navigation", navigation, nav_child),
            ("content", content, content_child),
        ],
        &format!("{route} {width}x{height} {theme}"),
    );
    if width == 800.0 {
        for (index, patches) in updates.iter().enumerate() {
            if index == 5 && route != "/" && route != "/drag-drop" {
                break;
            }
            let next_width = [800.0, 1280.0, 800.0, 1280.0, 1680.0, 1680.0][index];
            let nav_before = frames[&nav_child];
            cx.simulate_window_resize(window, size(px(next_width), px(height)));
            for payload in patches {
                apply_gallery_payload(cx, window, &root, payload);
            }
            draw(cx, window, &runtime, &mut frames);
            draw(cx, window, &runtime, &mut frames);
            unchanged(nav_before, frames[&nav_child]);
            measure_panes(
                cx,
                window,
                &runtime,
                &mut frames,
                [
                    ("navigation", navigation, nav_child),
                    ("content", content, content_child),
                ],
                &format!("{route} live stage {index} {next_width}x{height}"),
            );
        }
    }
    window
        .update(cx, |_, window, _| window.remove_window())
        .unwrap();
}

fn measure_panes(
    cx: &mut TestAppContext,
    window: AnyWindowHandle,
    runtime: &InMemoryAdapter,
    frames: &mut HashMap<u32, Frame>,
    panes: [(&str, u32, u32); 2],
    label: &str,
) {
    // Measure the same routed provider scene, not a reduced scrolling surrogate.
    // Alternate directions away from the limits so every timed wheel actually scrolls.
    for (name, pane, child) in panes {
        wheel(cx, window, frames[&pane], 100_000.0);
        draw(cx, window, runtime, frames);
        let overflow = frames[&child].height - frames[&pane].height;
        if overflow <= 1.0 {
            let before = frames[&child];
            wheel(cx, window, frames[&pane], -40.0);
            draw(cx, window, runtime, frames);
            unchanged(before, frames[&child]);
            eprintln!(
                "gallery {label} {name}: content fits; verified stationary, no scroll timing"
            );
            continue;
        }
        let delta = (overflow / 3.0).min(40.0);
        wheel(cx, window, frames[&pane], -delta);
        draw(cx, window, runtime, frames);
        for iteration in 0..8 {
            wheel(
                cx,
                window,
                frames[&pane],
                if iteration % 2 == 0 { -delta } else { delta },
            );
            draw(cx, window, runtime, frames);
        }
        let mut elapsed = Vec::with_capacity(48);
        for iteration in 0..48 {
            let before_y = frames[&child].y;
            let start = std::time::Instant::now();
            wheel(
                cx,
                window,
                frames[&pane],
                if iteration % 2 == 0 { -delta } else { delta },
            );
            // simulate_event already draws through the test platform's input callback.
            // Flush its layout notifications without forcing a second native redraw.
            finish_frame(cx, window, runtime, frames);
            elapsed.push(start.elapsed());
            assert!(
                (frames[&child].y - before_y).abs() > delta * 0.9,
                "timed {name} wheel {iteration} must scroll, not measure an idle boundary"
            );
        }
        elapsed.sort_unstable();
        let milliseconds = |index: usize| elapsed[index].as_secs_f64() * 1000.0;
        eprintln!(
            "gallery {label} wheel->frame {name}: samples={}, warmup=8, p50_ms={:.3}, p95_ms={:.3}, max_ms={:.3}",
            elapsed.len(),
            milliseconds(23),
            milliseconds(45),
            milliseconds(47)
        );
        if let Ok(budget) = std::env::var("SOLID_GPUI_SCROLL_P95_MS") {
            let budget: f64 = budget
                .parse()
                .expect("positive scroll p95 budget in milliseconds");
            assert!(budget.is_finite() && budget > 0.0);
            assert!(
                milliseconds(45) <= budget,
                "{name} scroll p95 {:.3} ms exceeds {budget} ms",
                milliseconds(45)
            );
        }
    }
}

#[gpui::test]
fn gallery_virtual_list_renders_initial_rows(cx: &mut TestAppContext) {
    let (mut snapshot, _) = gallery_snapshot(1280.0, 1000.0, "dark", "/virtual-list");
    let list = snapshot
        .nodes
        .iter()
        .find(|n| n.kind == solid_gpui::KIND_VIRTUAL_LIST)
        .expect("actual list");
    let viewport = list.parent_id;
    let row_text = snapshot
        .nodes
        .iter()
        .find(|n| {
            n.text
                .as_deref()
                .is_some_and(|t| t.starts_with("Virtual Dataset Item #1 ("))
        })
        .expect("first committed row");
    let row = if row_text.kind == solid_gpui::KIND_TEXT {
        row_text.id
    } else {
        row_text.parent_id
    };
    let third_text = snapshot
        .nodes
        .iter()
        .find(|n| {
            n.text
                .as_deref()
                .is_some_and(|t| t.starts_with("Virtual Dataset Item #3 ("))
        })
        .expect("third committed row");
    let third_row = if third_text.kind == solid_gpui::KIND_TEXT {
        third_text.id
    } else {
        third_text.parent_id
    };
    for node in &mut snapshot.nodes {
        if [viewport, row, third_row].contains(&node.id) {
            node.listener_id = node.id;
        }
    }
    let runtime = InMemoryAdapter::new();
    let mut profile = gallery_host_profile();
    let (window, root) = cx.update(|app| {
        profile.initialize(app);
        profile
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(1280.0), px(1000.0)),
                        app,
                    ))),
                    ..Default::default()
                },
                runtime.clone(),
                profile.extension_registry(),
                app,
            )
            .unwrap()
    });
    apply_gallery_payload(cx, window, &root, &snapshot.encode().unwrap());
    let mut frames = HashMap::new();
    draw(cx, window, &runtime, &mut frames);
    draw(cx, window, &runtime, &mut frames);
    let viewport_id = viewport;
    let viewport = frames[&viewport];
    assert!(viewport.height >= 318.0, "bounded Gallery list viewport");
    let row = frames
        .get(&row)
        .expect("nonempty data must paint the first row");
    assert!(
        row.width > 0.0
            && row.height > 0.0
            && row.y >= viewport.y
            && row.bottom() <= viewport.bottom(),
        "first row must be visible inside the list: row={row:?}, viewport={viewport:?}"
    );
    if std::env::var_os("SOLID_GPUI_LIST_TIMING").is_some() {
        let mut timings = Vec::new();
        for index in 0..120 {
            wheel(
                cx,
                window,
                viewport,
                if index % 2 == 0 { -20.0 } else { 20.0 },
            );
            let started = std::time::Instant::now();
            draw(cx, window, &runtime, &mut frames);
            timings.push(started.elapsed().as_secs_f64() * 1000.0);
        }
        timings.sort_by(f64::total_cmp);
        eprintln!(
            "list scroll CPU p50={:.3}ms p95={:.3}ms",
            timings[60], timings[114]
        );
    }
    let before = frames[&third_row];
    wheel(cx, window, viewport, -40.0);
    draw(cx, window, &runtime, &mut frames);
    unchanged(viewport, frames[&viewport_id]);
    assert!(
        (before.y - frames[&third_row].y - 40.0).abs() < 0.5,
        "nested list scrolls without moving its outer viewport"
    );
}

#[gpui::test]
fn gallery_button_labels_fit_control_height(cx: &mut TestAppContext) {
    let (mut snapshot, _) = gallery_snapshot(800.0, 600.0, "light", "/pressable");
    let labels: Vec<_> = snapshot
        .nodes
        .iter()
        .filter(|node| {
            node.kind == solid_gpui::KIND_PRESSABLE
                && node.style.as_ref().is_some_and(|s| s.height.is_some())
        })
        .flat_map(|button| {
            snapshot
                .nodes
                .iter()
                .filter(move |node| {
                    node.parent_id == button.id && node.kind == solid_gpui::KIND_TEXT
                })
                .map(move |label| (button.id, label.id))
        })
        .collect();
    assert!(
        labels.len() >= 6,
        "actual Gallery variants and sizes must be covered"
    );
    for node in &mut snapshot.nodes {
        if labels
            .iter()
            .any(|&(button, label)| node.id == button || node.id == label)
        {
            node.listener_id = node.id;
        }
    }
    let runtime = InMemoryAdapter::new();
    let mut profile = gallery_host_profile();
    let (window, root) = cx.update(|app| {
        profile.initialize(app);
        profile
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(800.0), px(600.0)),
                        app,
                    ))),
                    ..Default::default()
                },
                runtime.clone(),
                profile.extension_registry(),
                app,
            )
            .unwrap()
    });
    apply_gallery_payload(cx, window, &root, &snapshot.encode().unwrap());
    let mut frames = HashMap::new();
    draw(cx, window, &runtime, &mut frames);
    for (button, label) in labels {
        let button = frames[&button];
        let label = frames[&label];
        assert!(
            label.height > 0.0 && label.y >= button.y && label.bottom() <= button.bottom(),
            "fixed-height control must contain its label: {button:?}, {label:?}"
        );
    }
    window
        .update(cx, |_, window, _| window.remove_window())
        .unwrap();
}
