#![cfg(feature = "embedded-bun")]

use std::path::PathBuf;
use std::sync::Arc;

use react_gpui::{Event, RuntimeAdapter, Snapshot};
use react_gpui_bun::{CommitPoll, EmbeddedBunAdapter};

fn counter_entry() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/react-gpui/examples/counter.tsx")
}

#[test]
fn embedded_counter_commits_press_and_shutdown() {
    let runtime = EmbeddedBunAdapter::start(counter_entry()).expect("embedded Bun starts");
    let runtime_for_reader: Arc<dyn RuntimeAdapter> =
        Arc::clone(&runtime) as Arc<dyn RuntimeAdapter>;
    let initial = match runtime
        .recv_commit_timeout(std::time::Duration::from_secs(10))
        .expect("initial commit transport")
    {
        CommitPoll::Commit(commit) => commit,
        CommitPoll::Timeout => panic!("initial commit timed out"),
        CommitPoll::Ended => panic!("embedded runtime ended before initial commit"),
    };
    let snapshot = Snapshot::decode(&initial).expect("initial snapshot decodes");
    let pressable = snapshot
        .nodes
        .into_iter()
        .find(|node| node.listener_id != 0)
        .expect("counter press listener");
    assert_eq!(
        runtime
            .recv_commit_timeout(std::time::Duration::from_millis(5))
            .expect("idle timeout transport"),
        CommitPoll::Timeout
    );
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
    let update = match runtime
        .recv_commit_timeout(std::time::Duration::from_secs(10))
        .unwrap_or_else(|error| panic!("update commit transport: {error}"))
    {
        CommitPoll::Commit(commit) => commit,
        CommitPoll::Timeout => panic!("update commit timed out"),
        CommitPoll::Ended => panic!("embedded runtime ended before update"),
    };
    assert!(!update.is_empty(), "press produces an update commit");

    runtime.shutdown().expect("embedded Bun joins cleanly");
    assert_eq!(
        runtime
            .recv_commit_timeout(std::time::Duration::from_millis(10))
            .expect("ended commit transport"),
        CommitPoll::Ended
    );
    assert!(runtime.commit_count() >= 2);
}
