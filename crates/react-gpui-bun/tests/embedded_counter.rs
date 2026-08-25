#![cfg(feature = "embedded-bun")]

use std::path::PathBuf;
use std::sync::Arc;

use react_gpui::{Event, RuntimeAdapter, Snapshot};
use react_gpui_bun::EmbeddedBunAdapter;

fn counter_entry() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/react-gpui/examples/counter.tsx")
}

#[test]
fn embedded_counter_commits_press_and_shutdown() {
    let runtime = EmbeddedBunAdapter::start(counter_entry()).expect("embedded Bun starts");
    let runtime_for_reader: Arc<dyn RuntimeAdapter> =
        Arc::clone(&runtime) as Arc<dyn RuntimeAdapter>;
    let initial = runtime
        .recv_commit_timeout(std::time::Duration::from_secs(10))
        .expect("initial commit transport")
        .unwrap_or_else(|| {
            panic!(
                "initial commit timeout: count={}, status={:?}",
                runtime.commit_count(),
                runtime.runtime_status()
            )
        });
    let snapshot = Snapshot::decode(&initial).expect("initial snapshot decodes");
    let pressable = snapshot
        .nodes
        .into_iter()
        .find(|node| node.listener_id != 0)
        .expect("counter press listener");
    runtime_for_reader
        .send_event(&Event::press(
            snapshot.surface_id,
            snapshot.epoch,
            snapshot.revision,
            1,
            pressable.id,
            pressable.listener_id,
        ))
        .expect("press reaches embedded Bun");
    let update = runtime
        .recv_commit_timeout(std::time::Duration::from_secs(10))
        .unwrap_or_else(|error| panic!("update commit transport: {error}"))
        .unwrap_or_else(|| {
            panic!(
                "update commit timeout: count={}, status={:?}",
                runtime.commit_count(),
                runtime.runtime_status()
            )
        });
    assert!(!update.is_empty(), "press produces an update commit");

    runtime.shutdown().expect("embedded Bun joins cleanly");
    assert!(runtime.commit_count() >= 2);
}
