//! CPU-side native layout/paint probe. TestPlatform records a scene but does not
//! submit GPU work, present a drawable, or reproduce macOS live-resize scheduling.
use super::*;
use crate::protocol::{FlexDirectionCode, IconProperties, Node, Patch, Snapshot, UPDATE_TEXT};
use crate::transport::InMemoryAdapter;
use crate::tree::{KIND_ICON, KIND_RAW_TEXT};
use gpui::{AnyWindowHandle, AppContext as _, TestAppContext};

const SIZES: [(f32, f32); 6] = [
    (960.0, 720.0),
    (1120.0, 800.0),
    (1440.0, 960.0),
    (1280.0, 840.0),
    (1024.0, 768.0),
    (880.0, 680.0),
];
const SAMPLES: usize = 48;

fn node(nodes: &mut Vec<Node>, parent: u32, kind: u32, style: Style) -> u32 {
    let id = nodes.len() as u32 + 1;
    let order = nodes.iter().filter(|node| node.parent_id == parent).count() as u32;
    let mut node = Node::new(id, parent, order, kind);
    node.style = Some(style);
    nodes.push(node);
    id
}

fn text(nodes: &mut Vec<Node>, parent: u32, value: &str) -> u32 {
    let paragraph = node(
        nodes,
        parent,
        KIND_TEXT,
        Style {
            font_size: Some(13.0),
            color_rgba: Some(0xd7dce5ff),
            ..Style::default()
        },
    );
    let raw = node(nodes, paragraph, KIND_RAW_TEXT, Style::default());
    nodes.last_mut().unwrap().text = Some(value.into());
    raw
}

fn gallery_snapshot() -> (Snapshot, u32) {
    let mut nodes = Vec::new();
    let root = node(
        &mut nodes,
        0,
        KIND_VIEW,
        Style {
            flex_direction: Some(FlexDirectionCode::Row),
            flex_grow: Some(1.0),
            background_rgba: Some(0x11151cff),
            gap: Some(16.0),
            padding: Some(16.0),
            ..Style::default()
        },
    );
    let sidebar = node(
        &mut nodes,
        root,
        KIND_VIEW,
        Style {
            width: Some(176.0),
            flex_shrink: Some(0.0),
            flex_direction: Some(FlexDirectionCode::Column),
            gap: Some(10.0),
            ..Style::default()
        },
    );
    for title in [
        "Overview",
        "Buttons",
        "Typography",
        "Icons",
        "Inputs",
        "Checkboxes",
        "Switches",
        "Selects",
        "Cards",
        "Layout",
        "Lists",
        "Tabs",
        "Dialogs",
        "Menus",
        "Tooltips",
        "Progress",
        "Images",
        "Animation",
    ] {
        text(&mut nodes, sidebar, title);
    }
    let body = node(
        &mut nodes,
        root,
        KIND_VIEW,
        Style {
            flex_grow: Some(1.0),
            min_width: Some(0.0),
            flex_direction: Some(FlexDirectionCode::Column),
            gap: Some(12.0),
            ..Style::default()
        },
    );
    let mutation_target = text(
        &mut nodes,
        body,
        "Component workspace — native resize study",
    );
    let input = node(
        &mut nodes,
        body,
        KIND_TEXT_INPUT,
        Style {
            height: Some(32.0),
            font_size: Some(13.0),
            ..Style::default()
        },
    );
    nodes[(input - 1) as usize].host_properties =
        Some(HostProperties::TextInput(TextInputProperties {
            value: "Search components and examples".into(),
            placeholder: None,
            multiline: false,
            disabled: false,
            controlled: true,
            ack_edit_seq: 0,
            selection_start: 0,
            selection_end: 0,
            marked_start: None,
            marked_end: None,
            max_length: None,
            selection_reversed: false,
        }));
    for row in 0..6 {
        let row_id = node(
            &mut nodes,
            body,
            KIND_VIEW,
            Style {
                flex_direction: Some(FlexDirectionCode::Row),
                gap: Some(12.0),
                ..Style::default()
            },
        );
        for column in 0..3 {
            let card = node(
                &mut nodes,
                row_id,
                KIND_VIEW,
                Style {
                    flex_grow: Some(1.0),
                    min_width: Some(0.0),
                    flex_direction: Some(FlexDirectionCode::Column),
                    padding: Some(10.0),
                    gap: Some(6.0),
                    border_width: Some(1.0),
                    border_radius: Some(8.0),
                    border_color_rgba: Some(0x323a47ff),
                    background_rgba: Some(0x1b222dff),
                    ..Style::default()
                },
            );
            let icon = node(&mut nodes, card, KIND_ICON, Style::default());
            nodes[(icon - 1) as usize].host_properties =
                Some(HostProperties::Icon(IconProperties {
                    name: "lucide:home".into(),
                    size: 16.0,
                    color_rgba: Some(0x83b8ffff),
                }));
            text(
                &mut nodes,
                card,
                &format!("Example {}", row * 3 + column + 1),
            );
            text(
                &mut nodes,
                card,
                "Native text wraps as the viewport narrows.",
            );
            let paragraph = node(
                &mut nodes,
                card,
                KIND_TEXT,
                Style {
                    font_size: Some(11.0),
                    ..Style::default()
                },
            );
            for (index, value) in ["Ready ", "· ", "updated today"].into_iter().enumerate() {
                let run = node(
                    &mut nodes,
                    paragraph,
                    KIND_TEXT,
                    Style {
                        color_rgba: Some(if index == 0 { 0x79d9a0ff } else { 0x9daabfff }),
                        ..Style::default()
                    },
                );
                let raw = node(&mut nodes, run, KIND_RAW_TEXT, Style::default());
                nodes[(raw - 1) as usize].text = Some(value.into());
            }
        }
    }
    (Snapshot::new(7, 3, 0, 1, nodes), mutation_target)
}

#[derive(Clone, Copy, Debug)]
struct Counters {
    rich: usize,
    input_content: usize,
    input_runs: usize,
    input_shape: usize,
    cached_paragraphs: usize,
}

fn counters(root: &SolidRoot) -> Counters {
    Counters {
        rich: root.rich_text_assembly_count.get(),
        input_content: root.input_content_assembly_count.get(),
        input_runs: root.input_run_assembly_count.get(),
        input_shape: root.input_shape_count.get(),
        cached_paragraphs: root.rich_text_parts_cache.borrow().len(),
    }
}

fn draw(cx: &mut TestAppContext, handle: AnyWindowHandle) -> Duration {
    cx.update_window(handle, |_, window, cx| {
        // Force native element construction, Taffy layout, text shaping and scene
        // construction even when no protocol commit made the entity dirty.
        window.refresh();
        let started = Instant::now();
        window.draw(cx).clear(cx);
        started.elapsed()
    })
    .expect("draw resize probe")
}

fn percentile(samples: &[Duration], percentile: usize) -> f64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)].as_secs_f64() * 1_000.0
}

/// Run with --nocapture. Optional SOLID_GPUI_RESIZE_P95_MS turns a measured,
/// machine/build-specific baseline into a red-capable timing gate. No universal
/// frame budget is asserted against an uncalibrated test-platform CPU measurement.
#[gpui::test]
fn native_resize_feedback_loop(cx: &mut TestAppContext) {
    let runtime = InMemoryAdapter::new();
    let window = cx.open_window(gpui::size(px(960.0), px(720.0)), {
        let runtime = runtime.clone();
        move |_, _| SolidRoot::new(runtime)
    });
    let handle = window.into();
    let root = window.root(cx).expect("resize probe root");
    let (snapshot, mutation_target) = gallery_snapshot();
    let payload = snapshot.encode().expect("encode gallery fixture");
    root.update(cx, |root, cx| root.apply_payload(&payload, cx))
        .expect("apply gallery fixture");
    for _ in 0..2 {
        for (width, height) in SIZES {
            cx.simulate_window_resize(handle, gpui::size(px(width), px(height)));
            draw(cx, handle);
            cx.run_until_parked();
        }
    }
    eprintln!(
        "resize_probe: CPU native layout/paint only; no GPU submission/presentation; warmup=12 samples={SAMPLES} viewports={SIZES:?}"
    );
    let mut revision = 1;
    for phase in ["unchanged", "resize", "content_mutation"] {
        let before = root.read_with(cx, |root, _| counters(root));
        let mut samples = Vec::with_capacity(SAMPLES);
        let mut cycles = Vec::with_capacity(SAMPLES);
        for index in 0..SAMPLES {
            // Include automatic native draws triggered by resize or a commit,
            // not only the explicit draw measured below.
            let cycle_started = Instant::now();
            if phase != "unchanged" {
                let (width, height) = SIZES[index % SIZES.len()];
                cx.simulate_window_resize(handle, gpui::size(px(width), px(height)));
            }
            if phase == "content_mutation" {
                let patch = Patch::new(
                    7,
                    3,
                    revision,
                    revision + 1,
                    vec![PatchOperation::Update {
                        id: mutation_target,
                        mask: UPDATE_TEXT,
                        style: None,
                        text: Some(format!("Component workspace — revision {revision:03}")),
                        listener_id: 0,
                        host_properties: None,
                        accessibility: None,
                        focusable: false,
                        selectable: false,
                        tooltip: None,
                        accepts_pointer_move: false,
                    }],
                );
                let payload = patch.encode().expect("encode probe mutation");
                root.update(cx, |root, cx| root.apply_payload(&payload, cx))
                    .expect("apply probe mutation");
                revision += 1;
            }
            samples.push(draw(cx, handle));
            cx.update_window(handle, |_, window, cx| {
                window.simulate_next_frame(cx);
            })
            .expect("flush native frame callbacks");
            cx.run_until_parked();
            while runtime.take_event().expect("drain native events").is_some() {}
            cycles.push(cycle_started.elapsed());
        }
        let after = root.read_with(cx, |root, _| counters(root));
        let p95 = percentile(&samples, 95);
        eprintln!(
            "resize_probe: phase={phase} draw_p50_ms={:.3} draw_p95_ms={p95:.3} draw_max_ms={:.3} rich_assembly={} input_content={} input_runs={} input_shape={} cached_paragraphs={}->{}",
            percentile(&samples, 50),
            percentile(&samples, 100),
            after.rich - before.rich,
            after.input_content - before.input_content,
            after.input_runs - before.input_runs,
            after.input_shape - before.input_shape,
            before.cached_paragraphs,
            after.cached_paragraphs
        );
        eprintln!(
            "resize_probe: phase={phase} cycle_p50_ms={:.3} cycle_p95_ms={:.3} cycle_max_ms={:.3}",
            percentile(&cycles, 50),
            percentile(&cycles, 95),
            percentile(&cycles, 100)
        );
        assert_eq!(
            after.rich - before.rich,
            if phase == "content_mutation" {
                SAMPLES
            } else {
                0
            },
            "resize must reuse immutable rich text; each mutation must rebuild exactly its paragraph"
        );
        if phase == "resize" {
            if let Ok(budget) = std::env::var("SOLID_GPUI_RESIZE_P95_MS") {
                let budget: f64 = budget
                    .parse()
                    .expect("resize p95 budget must be milliseconds");
                assert!(
                    budget.is_finite() && budget > 0.0,
                    "resize budget must be positive and finite"
                );
                assert!(
                    p95 <= budget,
                    "resize draw p95 {p95:.3}ms exceeds calibrated budget {budget:.3}ms"
                );
            }
            if let Ok(budget) = std::env::var("SOLID_GPUI_RESIZE_CYCLE_P95_MS") {
                let budget: f64 = budget
                    .parse()
                    .expect("resize cycle p95 budget must be milliseconds");
                assert!(
                    budget.is_finite() && budget > 0.0,
                    "resize cycle budget must be positive and finite"
                );
                let cycle_p95 = percentile(&cycles, 95);
                assert!(
                    cycle_p95 <= budget,
                    "resize cycle p95 {cycle_p95:.3}ms exceeds calibrated budget {budget:.3}ms"
                );
            }
        }
    }
}

#[gpui::test]
fn native_grid_tracks_and_spans_relayout_after_patch(cx: &mut TestAppContext) {
    let window = cx.open_window(gpui::size(px(400.0), px(200.0)), |_, _| {
        SolidRoot::new(InMemoryAdapter::new())
    });
    let root = window.root(cx).unwrap();
    let mut nodes = Vec::new();
    let parent = node(
        &mut nodes,
        0,
        KIND_VIEW,
        Style {
            width: Some(210.0),
            height: Some(50.0),
            grid_columns: Some(2),
            gap: Some(10.0),
            ..Style::default()
        },
    );
    for index in 0..3 {
        node(
            &mut nodes,
            parent,
            KIND_VIEW,
            Style {
                height: Some(20.0),
                grid_column_span: (index == 2).then_some(2),
                background_rgba: Some(0x3366ccff),
                ..Style::default()
            },
        );
    }
    let payload = Snapshot::new(1, 1, 0, 1, nodes).encode().unwrap();
    root.update(cx, |root, cx| root.apply_payload(&payload, cx))
        .unwrap();
    draw(cx, window.into());
    let bounds = cx
        .update_window(window.into(), |_, window, _| {
            window
                .painted_quads()
                .into_iter()
                .map(|quad| {
                    quad.bounds
                        .map(|value| px(value.as_f32() / window.scale_factor()))
                })
                .filter(|bounds| bounds.size.height.as_f32() == 20.0)
                .collect::<Vec<_>>()
        })
        .unwrap();
    assert_eq!(bounds.len(), 3);
    assert_eq!(bounds[0].size.width.as_f32(), 100.0);
    assert_eq!((bounds[1].origin.x - bounds[0].origin.x).as_f32(), 110.0);
    assert_eq!(bounds[2].size.width.as_f32(), 210.0);
    assert_eq!((bounds[2].origin.y - bounds[0].origin.y).as_f32(), 30.0);
    let payload = Patch::new(
        1,
        1,
        1,
        2,
        vec![crate::protocol::PatchOperation::Update {
            id: parent,
            mask: crate::protocol::UPDATE_STYLE,
            style: Some(Style {
                width: Some(330.0),
                height: Some(50.0),
                grid_columns: Some(3),
                gap: Some(10.0),
                ..Style::default()
            }),
            text: None,
            listener_id: 0,
            host_properties: None,
            accessibility: None,
            focusable: false,
            selectable: false,
            tooltip: None,
            accepts_pointer_move: false,
        }],
    )
    .encode()
    .unwrap();
    root.update(cx, |root, cx| root.apply_payload(&payload, cx))
        .unwrap();
    draw(cx, window.into());
    let bounds = cx
        .update_window(window.into(), |_, window, _| {
            window
                .painted_quads()
                .into_iter()
                .map(|quad| {
                    quad.bounds
                        .map(|value| px(value.as_f32() / window.scale_factor()))
                })
                .filter(|bounds| bounds.size.height.as_f32() == 20.0)
                .collect::<Vec<_>>()
        })
        .unwrap();
    assert_eq!(bounds.len(), 3);
    assert!((bounds[0].size.width.as_f32() - 310.0 / 3.0).abs() < 1.0);
    assert!((bounds[2].size.width.as_f32() - (620.0 / 3.0 + 10.0)).abs() < 1.0);
}
