use react_gpui::{
    Node, NodeStore, Patch, PatchOperation, Snapshot, Style, KIND_VIEW, UPDATE_STYLE,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const NODE_COUNT: usize = 10_000;
const PATCH_OPERATION_COUNT: usize = 1_000;
const CHAIN_LENGTH: u32 = 2_000;
const FAN_PARENT_COUNT: u32 = 100;

// These are smoke regression guards, not benchmarks. They are deliberately
// loose: each local debug measurement must retain at least 10x headroom.
const SNAPSHOT_ENCODE_BUDGET: Duration = Duration::from_secs(2);
const SNAPSHOT_DECODE_BUDGET: Duration = Duration::from_secs(2);
const SNAPSHOT_APPLY_BUDGET: Duration = Duration::from_secs(2);
const PATCH_ENCODE_BUDGET: Duration = Duration::from_secs(2);
const PATCH_DECODE_BUDGET: Duration = Duration::from_secs(2);
const PATCH_APPLY_BUDGET: Duration = Duration::from_secs(2);
// 2026-08-26 current baseline: snapshot encode 8.190 ms, decode 15.620 ms,
// apply 22.710 ms; patch encode 0.795 ms, decode 2.861 ms, apply 79.480 ms.
// New budgets are measured * 10 on 2026-08-26 after the final local run:
// style full 85.895 ms, style null 13.809 ms, layout report 47.661 ms.
const STYLE_FULL_BUDGET: Duration = Duration::from_millis(860);
const STYLE_NULL_BUDGET: Duration = Duration::from_millis(140);
const LAYOUT_BATCH_BUDGET: Duration = Duration::from_millis(480);

fn build_snapshot() -> Snapshot {
    let mut nodes = Vec::with_capacity(NODE_COUNT);
    nodes.push(Node::new(1, 0, 0, KIND_VIEW));
    let chain_end = CHAIN_LENGTH + 1;
    for id in 2..=chain_end {
        nodes.push(Node::new(id, id - 1, 0, KIND_VIEW));
    }
    let fan_start = chain_end + 1;
    let fan_end = fan_start + FAN_PARENT_COUNT - 1;
    for id in fan_start..=fan_end {
        nodes.push(Node::new(id, 1, id - chain_end, KIND_VIEW));
    }
    let leaf_start = fan_end + 1;
    for id in leaf_start..=NODE_COUNT as u32 {
        let offset = id - leaf_start;
        let parent_id = fan_start + offset % FAN_PARENT_COUNT;
        let index = offset / FAN_PARENT_COUNT;
        nodes.push(Node::new(id, parent_id, index, KIND_VIEW));
    }
    Snapshot::new(7, 3, 0, 1, nodes)
}
fn full_style() -> Style {
    Style {
        width: Some(120.0),
        height: Some(48.0),
        flex_direction: Some(2),
        flex_grow: Some(1.0),
        padding: Some(8.0),
        gap: Some(4.0),
        justify_content: Some(4),
        align_items: Some(4),
        border_radius: Some(6.0),
        border_width: Some(1.0),
        border_color_rgba: Some(0x1122_3344),
        font_size: Some(14.0),
        font_weight: Some(600),
        background_rgba: Some(0x2233_4455),
        color_rgba: Some(0xff00_00ff),
        opacity: Some(0.9),
        overflow: Some(2),
        line_clamp: Some(3),
        text_overflow: Some(2),
        margin_top: Some(1.0),
        margin_right: Some(2.0),
        margin_bottom: Some(3.0),
        margin_left: Some(4.0),
        font_style: Some(1),
        text_decoration: Some(2),
        line_height: Some(18.0),
        min_width: Some(10.0),
        max_width: Some(400.0),
        min_height: Some(20.0),
        max_height: Some(200.0),
        flex_shrink: Some(0.5),
        align_self: Some(5),
        position: Some(1),
        left: Some(-4.0),
        top: Some(2.0),
        right: Some(8.0),
        bottom: Some(6.0),
        cursor: Some(18),
        ..Style::default()
    }
}

fn build_styled_snapshot() -> Snapshot {
    let mut snapshot = build_snapshot();
    let style = full_style();
    for node in &mut snapshot.nodes {
        node.style = Some(style.clone());
    }
    snapshot
}

fn build_patch() -> Patch {
    let chain_end = CHAIN_LENGTH + 1;
    let fan_start = chain_end + 1;
    let fan_end = fan_start + FAN_PARENT_COUNT - 1;
    let leaf_start = fan_end + 1;
    let update_start = leaf_start;
    let move_start = leaf_start + 3_000;
    let delete_start = leaf_start + 6_500;
    let mut operations = Vec::with_capacity(PATCH_OPERATION_COUNT);

    for offset in 0..400u32 {
        operations.push(PatchOperation::Update {
            id: update_start + offset,
            mask: UPDATE_STYLE,
            style: Some(Style {
                width: Some(32.0 + (offset % 8) as f32),
                ..Style::default()
            }),
            text: None,
            listener_id: 0,
            host_properties: None,
            accessibility: None,
            focusable: false,
        });
    }
    for offset in 0..300u32 {
        operations.push(PatchOperation::Move {
            id: move_start + offset,
            parent_id: 2,
            index: 0,
        });
    }
    for offset in 0..300u32 {
        operations.push(PatchOperation::Delete {
            id: delete_start + offset,
        });
    }
    debug_assert_eq!(operations.len(), PATCH_OPERATION_COUNT);
    Patch::new(7, 3, 1, 2, operations)
}

fn assert_budget(stage: &str, elapsed: Duration, budget: Duration) {
    eprintln!(
        "perf_budget: {stage}: {:.3} ms (budget {:.0} ms)",
        elapsed.as_secs_f64() * 1_000.0,
        budget.as_secs_f64() * 1_000.0,
    );
    assert!(
        elapsed < budget,
        "{stage} exceeded smoke budget: {:?} >= {:?}",
        elapsed,
        budget
    );
}

#[test]
fn protocol_and_tree_smoke_budget() {
    let snapshot = build_snapshot();
    assert_eq!(snapshot.nodes.len(), NODE_COUNT);
    let patch = build_patch();
    assert_eq!(patch.operations.len(), PATCH_OPERATION_COUNT);

    let started = Instant::now();
    let snapshot_payload = snapshot.encode().expect("encode snapshot");
    assert_budget("snapshot encode", started.elapsed(), SNAPSHOT_ENCODE_BUDGET);

    let started = Instant::now();
    let decoded_snapshot = Snapshot::decode(&snapshot_payload).expect("decode snapshot");
    assert_budget("snapshot decode", started.elapsed(), SNAPSHOT_DECODE_BUDGET);
    assert_eq!(decoded_snapshot, snapshot);

    let started = Instant::now();
    let mut store = NodeStore::default();
    store
        .apply_snapshot(decoded_snapshot)
        .expect("apply snapshot");
    assert_budget("snapshot apply", started.elapsed(), SNAPSHOT_APPLY_BUDGET);

    let started = Instant::now();
    let patch_payload = patch.encode().expect("encode patch");
    assert_budget("patch encode", started.elapsed(), PATCH_ENCODE_BUDGET);

    let started = Instant::now();
    let decoded_patch = Patch::decode(&patch_payload).expect("decode patch");
    assert_budget("patch decode", started.elapsed(), PATCH_DECODE_BUDGET);
    assert_eq!(decoded_patch, patch);

    let started = Instant::now();
    store.apply_patch(decoded_patch).expect("apply patch");
    assert_budget("patch apply", started.elapsed(), PATCH_APPLY_BUDGET);
    assert_eq!(store.revision(), 2);
}

#[test]
fn style_full_and_null_roundtrip_budget() {
    let full = build_styled_snapshot();
    let null = build_snapshot();
    for (name, snapshot, budget) in [
        ("style full", full, STYLE_FULL_BUDGET),
        ("style null", null, STYLE_NULL_BUDGET),
    ] {
        let started = Instant::now();
        let payload = snapshot.encode().expect("encode styled snapshot");
        let encoded = started.elapsed();
        let started = Instant::now();
        let decoded = Snapshot::decode(&payload).expect("decode styled snapshot");
        let decoded_elapsed = started.elapsed();
        assert_eq!(decoded, snapshot);
        assert_budget(
            &format!("{name} encode+decode"),
            encoded + decoded_elapsed,
            budget,
        );
        eprintln!(
            "perf_budget: {name}: encode {:.3} ms, decode {:.3} ms",
            encoded.as_secs_f64() * 1_000.0,
            decoded_elapsed.as_secs_f64() * 1_000.0,
        );
    }
}

#[test]
fn layout_report_enqueue_and_dedupe_budget() {
    const REPORT_COUNT: u32 = 100_000;
    let started = Instant::now();
    let mut last = HashMap::<u32, (u32, u32)>::with_capacity(10_000);
    let mut emitted = 0u32;
    for index in 0..REPORT_COUNT {
        let node_id = index % 10_000;
        let frame = (index % 64, index / 64);
        if last.get(&node_id) != Some(&frame) {
            last.insert(node_id, frame);
            emitted += 1;
        }
        if last.get(&node_id) != Some(&frame) {
            unreachable!("same frame must dedupe");
        }
    }
    let elapsed = started.elapsed();
    assert!(emitted > 0);
    assert_budget("layout report enqueue+dedupe", elapsed, LAYOUT_BATCH_BUDGET);
}
