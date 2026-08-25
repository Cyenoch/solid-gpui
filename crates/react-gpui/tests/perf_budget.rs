use std::time::{Duration, Instant};

use react_gpui::{
    KIND_VIEW, Node, NodeStore, Patch, PatchOperation, Snapshot, Style, UPDATE_STYLE,
};

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
