use solid_gpui::{
    EVENT_PRESS, Event, KIND_PRESSABLE, KIND_VIEW, Patch, ProcessAdapter, RuntimeAdapter, Snapshot,
};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn peer_path() -> String {
    std::env::var("CARGO_BIN_EXE_solid-gpui-frame-peer")
        .or_else(|_| std::env::var("CARGO_BIN_EXE_solid_gpui_frame_peer"))
        .expect("Cargo should provide frame peer binary path")
}

#[test]
fn process_adapter_round_trips_snapshot_and_event_frames() {
    let runtime = ProcessAdapter::spawn(Command::new(peer_path())).expect("spawn frame peer");

    let payload = runtime
        .recv_commit()
        .expect("read snapshot frame")
        .expect("frame peer should emit a snapshot");
    let snapshot = Snapshot::decode(&payload).expect("decode snapshot frame");
    assert_eq!(snapshot.nodes.len(), 1);
    assert_eq!(snapshot.nodes[0].kind, KIND_VIEW);

    let event = Event::press(1, 1, 1, 1, 1, 7);
    assert_eq!(u32::from(event.event_kind()), EVENT_PRESS);
    runtime.send_event(event).expect("write event frame");
    assert_eq!(runtime.recv_commit().expect("read peer EOF"), None);
    runtime.shutdown().expect("shutdown frame peer");
}
#[test]
fn process_adapter_round_trips_actual_bun_counter() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .parent()
        .expect("repository root")
        .to_path_buf();
    let mut command = Command::new("bun");
    command.current_dir(&repo_root).args([
        "run",
        "--conditions=browser",
        "fixtures/press-roundtrip.ts",
    ]);
    let runtime = ProcessAdapter::spawn(command).expect("spawn Bun counter");
    let payload = runtime
        .recv_commit()
        .expect("read Bun counter snapshot")
        .expect("Bun counter should emit a snapshot");
    let snapshot = Snapshot::decode(&payload).expect("decode Bun counter snapshot");
    let pressable = snapshot
        .nodes
        .iter()
        .find(|node| node.kind == KIND_PRESSABLE && node.listener_id != 0)
        .expect("counter pressable");
    let event = Event::press(
        snapshot.surface_id,
        snapshot.epoch,
        snapshot.revision,
        1,
        pressable.id,
        pressable.listener_id,
    );
    assert_eq!(u32::from(event.event_kind()), EVENT_PRESS);
    runtime.send_event(event).expect("send Bun press");
    let patch_payload = runtime
        .recv_commit()
        .expect("read Bun counter patch")
        .expect("Bun counter should emit a patch");
    let patch = Patch::decode(&patch_payload).expect("decode Bun counter patch");
    assert_eq!(patch.base_revision, snapshot.revision);
    assert!(patch.operations.iter().any(|operation| {
        matches!(
            operation,
            solid_gpui::PatchOperation::Update {
                mask,
                text: Some(text),
                ..
            } if mask & solid_gpui::UPDATE_TEXT != 0 && text == "Count: 1"
        )
    }));
    runtime.shutdown().expect("shutdown Bun counter");
}

#[test]
fn process_shutdown_unblocks_a_reader_waiting_for_the_next_frame() {
    let runtime = ProcessAdapter::spawn(Command::new(peer_path())).expect("spawn frame peer");
    runtime
        .recv_commit()
        .expect("read initial snapshot")
        .expect("frame peer should emit a snapshot");

    let reader_runtime = Arc::clone(&runtime);
    let reader = thread::spawn(move || reader_runtime.recv_commit());
    thread::sleep(Duration::from_millis(25));

    runtime.shutdown().expect("shutdown blocked frame peer");
    assert_eq!(
        reader
            .join()
            .expect("join blocked reader")
            .expect("read EOF"),
        None
    );
}
