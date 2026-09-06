use super::support::*;
use crate::renderer::SolidRoot;
use gpui::AppContext as _;
use std::collections::{HashMap, HashSet};

const SOAK_ITERATIONS: usize = 2_000;
const SAMPLE_INTERVAL: usize = 200;
const TREE_NODE_COUNT: usize = 500;
const MAX_SIDE_MAP_LEN: usize = TREE_NODE_COUNT + 16;
const MAX_UNDO_HISTORY: usize = 100;
const MAX_SEQUENCE: u32 = 1 + (SOAK_ITERATIONS as u32 * 256);

const INPUT_ID: u32 = 6;

struct SoakSample {
    iteration: usize,
    elapsed_ms: f64,
    rss_kib: Option<u64>,
    sequence: u32,
    undo: usize,
    redo: usize,
    side_maps: Vec<(&'static str, usize)>,
}

fn soak_snapshot() -> Snapshot {
    let mut nodes = Vec::with_capacity(TREE_NODE_COUNT);
    nodes.push(Node::new(1, 0, 0, KIND_VIEW));

    let mut rich = Node::new(2, 1, 0, KIND_TEXT);
    rich.listener_id = 2;
    rich.style = Some(Style {
        font_size: Some(14.0),
        ..Style::default()
    });
    nodes.push(rich);
    nodes.push({
        let mut raw = Node::new(3, 2, 0, KIND_RAW_TEXT);
        raw.text = Some("soak rich text ".to_owned());
        raw
    });
    let mut run = Node::new(4, 2, 1, KIND_TEXT);
    run.listener_id = 4;
    run.focusable = true;
    run.style = Some(Style {
        color_rgba: Some(0x3366ccff),
        ..Style::default()
    });
    nodes.push(run);
    nodes.push({
        let mut raw = Node::new(5, 4, 0, KIND_RAW_TEXT);
        raw.text = Some("link".to_owned());
        raw
    });

    let mut input = Node::new(INPUT_ID, 1, 1, KIND_TEXT_INPUT);
    input.listener_id = INPUT_ID;
    input.style = Some(Style {
        width: Some(240.0),
        height: Some(24.0),
        ..Style::default()
    });
    input.host_properties = Some(HostProperties::TextInput(TextInputProperties {
        value: "input".to_owned(),
        placeholder: Some("soak".to_owned()),
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
    nodes.push(input);

    let mut list = Node::new(7, 1, 2, KIND_VIRTUAL_LIST);
    list.listener_id = 7;
    list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
        item_count: 100,
        range_start: 0,
        range_end: 4,
        estimated_item_size: 24.0,
        overscan: 2,
    }));
    nodes.push(list);

    // Thirty-eight fixed branches exercise recursive rendering and sibling
    // bookkeeping while keeping the tree at exactly 500 nodes.
    for branch in 0..38u32 {
        let parent_id = 1_000 + branch * 20;
        nodes.push(Node::new(parent_id, 1, branch + 3, KIND_VIEW));
        let child_count = if branch == 37 { 11 } else { 12 };
        for index in 0..child_count {
            nodes.push(Node::new(
                parent_id + index + 1,
                parent_id,
                index,
                KIND_VIEW,
            ));
        }
    }

    assert_eq!(nodes.len(), TREE_NODE_COUNT);
    Snapshot::new(7, 3, 0, 1, nodes)
}

fn patch_for(iteration: usize) -> Patch {
    let parent_id = 1_000 + (iteration % 38) as u32 * 20;
    let revision = iteration as u32 + 1;
    Patch::new(
        7,
        3,
        revision,
        revision + 1,
        vec![PatchOperation::Update {
            id: parent_id,
            mask: UPDATE_STYLE,
            style: Some(Style {
                width: Some(440.0 + (iteration % 20) as f32),
                height: Some(10.0 + (iteration % 7) as f32),
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
}

fn rss_kib() -> Option<u64> {
    let pid = std::process::id().to_string();
    let output = ProcessCommand::new("ps")
        .args(["-o", "rss=", "-p", &pid])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

fn drain_events(runtime: &InMemoryAdapter) {
    while let Ok(Some(_)) = runtime.take_event() {}
}
fn sample(
    root: &gpui::Entity<SolidRoot>,
    cx: &mut gpui::TestAppContext,
    iteration: usize,
    started: Instant,
) -> SoakSample {
    let (sequence, undo, redo, side_maps) = root.read_with(cx, |root, _| {
        let live: HashSet<u32> = root.store().iter().map(|node| node.id).collect();
        let side_maps = root
            .test_side_map_ids()
            .into_iter()
            .map(|(name, ids)| {
                assert!(
                    ids.iter().all(|id| live.contains(id)),
                    "side map {name} retained an ID outside the live tree"
                );
                assert!(
                    ids.len() <= MAX_SIDE_MAP_LEN,
                    "side map {name} exceeded bound: {} > {MAX_SIDE_MAP_LEN}",
                    ids.len()
                );
                (name, ids.len())
            })
            .collect();
        let (_, undo, redo) = root.test_input_history_lengths();
        (root.test_sequence(), undo, redo, side_maps)
    });
    assert!(
        sequence <= MAX_SEQUENCE,
        "event sequence exceeded soak bound at iteration {iteration}: {sequence}"
    );
    assert!(
        undo <= MAX_UNDO_HISTORY,
        "undo history exceeded bound at iteration {iteration}: {undo}"
    );
    assert!(
        redo <= MAX_UNDO_HISTORY,
        "redo history exceeded bound at iteration {iteration}: {redo}"
    );
    SoakSample {
        iteration,
        elapsed_ms: started.elapsed().as_secs_f64() * 1_000.0,
        rss_kib: rss_kib(),
        sequence,
        undo,
        redo,
        side_maps,
    }
}

#[gpui::test]
fn sustained_renderer_load_keeps_state_bounded_and_reports_memory_trend(
    cx: &mut gpui::TestAppContext,
) {
    let runtime = InMemoryAdapter::new();
    let window = cx.open_window(gpui::size(gpui::px(480.0), gpui::px(320.0)), {
        let runtime = Arc::clone(&runtime);
        move |_, _| SolidRoot::new(runtime)
    });
    let root = window.root(cx).expect("soak renderer root");
    let snapshot = soak_snapshot().encode().expect("encode soak snapshot");
    root.update(cx, |root, cx| root.apply_payload(&snapshot, cx))
        .expect("apply soak snapshot");
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .expect("warm soak draw");
    cx.run_until_parked();
    drain_events(&runtime);

    let input_point = root.read_with(cx, |root, _| {
        let (x, y, width, height) = root
            .test_input_bounds(INPUT_ID)
            .expect("input layout after warm soak draw");
        gpui::point(gpui::px(x + width * 0.5), gpui::px(y + height * 0.5))
    });
    let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
    visual.simulate_mouse_down(
        input_point,
        gpui::MouseButton::Left,
        gpui::Modifiers::none(),
    );
    visual.simulate_mouse_up(
        input_point,
        gpui::MouseButton::Left,
        gpui::Modifiers::none(),
    );
    drain_events(&runtime);

    let started = Instant::now();
    let mut samples = Vec::with_capacity(SOAK_ITERATIONS / SAMPLE_INTERVAL + 1);
    samples.push(sample(&root, cx, 0, started));
    let mut previous_lengths: HashMap<&'static str, usize> =
        samples[0].side_maps.iter().copied().collect();
    let mut increasing_runs: HashMap<&'static str, usize> = HashMap::new();

    for iteration in 0..SOAK_ITERATIONS {
        let payload = patch_for(iteration).encode().expect("encode soak patch");
        root.update(cx, |root, cx| root.apply_payload(&payload, cx))
            .expect("apply soak patch");

        // Exercise input dispatch periodically without changing the tree.
        if iteration % 20 == 0 {
            visual.simulate_input("x");
        }
        cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
            .expect("draw soak frame");
        cx.run_until_parked();
        drain_events(&runtime);

        if (iteration + 1) % SAMPLE_INTERVAL == 0 {
            let current = sample(&root, cx, iteration + 1, started);
            eprintln!(
                "soak_sample: iteration={} elapsed_ms={:.1} rss_kib={:?} sequence={} undo={} redo={} side_maps={:?}",
                current.iteration,
                current.elapsed_ms,
                current.rss_kib,
                current.sequence,
                current.undo,
                current.redo,
                current.side_maps,
            );
            for (name, length) in &current.side_maps {
                let run = if previous_lengths
                    .get(name)
                    .is_some_and(|previous| length > previous)
                {
                    increasing_runs.get(name).copied().unwrap_or_default() + 1
                } else {
                    0
                };
                assert!(
                    run < 4,
                    "side map {name} grew monotonically for {run} samples while tree stayed at {TREE_NODE_COUNT} nodes"
                );
                increasing_runs.insert(*name, run);
                previous_lengths.insert(*name, *length);
            }
            samples.push(current);
        }
    }

    let elapsed = started.elapsed();
    let rss_values: Vec<u64> = samples.iter().filter_map(|sample| sample.rss_kib).collect();
    let first_rss = rss_values.first().copied();
    let last_rss = rss_values.last().copied();
    let peak_rss = rss_values.iter().copied().max();
    let rss_delta = last_rss
        .zip(first_rss)
        .map(|(last, first)| last as i64 - first as i64);
    let delta_per_iteration = rss_delta.map(|delta| delta as f64 / SOAK_ITERATIONS as f64);
    eprintln!(
        "soak_summary: iterations={SOAK_ITERATIONS} nodes={TREE_NODE_COUNT} elapsed_ms={:.1} first_rss_kib={first_rss:?} last_rss_kib={last_rss:?} peak_rss_kib={peak_rss:?} rss_delta_kib={rss_delta:?} rss_delta_per_iteration_kib={delta_per_iteration:?}",
        elapsed.as_secs_f64() * 1_000.0,
    );
    for pair in samples.windows(2) {
        if let (Some(previous), Some(current)) = (pair[0].rss_kib, pair[1].rss_kib) {
            eprintln!(
                "soak_rss_delta: from_iteration={} to_iteration={} delta_kib={}",
                pair[0].iteration,
                pair[1].iteration,
                current as i64 - previous as i64,
            );
        }
    }

    assert_eq!(samples.len(), SOAK_ITERATIONS / SAMPLE_INTERVAL + 1);
    assert_eq!(
        root.read_with(cx, |root, _| root.store().len()),
        TREE_NODE_COUNT
    );
    let final_sample = sample(&root, cx, SOAK_ITERATIONS, started);
    assert!(final_sample.sequence <= MAX_SEQUENCE);
    assert!(final_sample.undo <= MAX_UNDO_HISTORY);
    assert!(final_sample.redo <= MAX_UNDO_HISTORY);
}
