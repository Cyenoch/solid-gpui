use solid_gpui::{
    DecodedMessage, RuntimeAdapter, RuntimeStatus, decode_message, runtime::vite::Vite,
};

#[test]
fn rust_starts_a_vite_project_and_owns_its_shutdown() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/solid-gpui");
    let mut command = Vite::new(root)
        .config_file("../../fixtures/vite.config.ts")
        .command()
        .unwrap();
    command
        .env("SOLID_GPUI_FIXTURE_RUNTIME", "bun")
        .env("SOLID_GPUI_FIXTURE", "fixtures/vite-counter.tsx");
    let runtime = solid_gpui::ProcessAdapter::spawn(command).unwrap();
    let reader = runtime.clone();
    let (sender, receiver) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        for _ in 0..3 {
            let frame = reader
                .recv_commit()
                .unwrap()
                .expect("Vite closed before the first Snapshot");
            if let DecodedMessage::Snapshot(snapshot) = decode_message(&frame).unwrap() {
                sender.send(snapshot).unwrap();
                return;
            }
        }
        panic!("Vite emitted no initial Snapshot");
    });
    let snapshot = receiver.recv_timeout(std::time::Duration::from_secs(15));
    runtime.shutdown().unwrap();
    worker.join().unwrap();
    assert_eq!(runtime.status(), RuntimeStatus::Shutdown);
    assert!(
        snapshot
            .unwrap()
            .nodes
            .iter()
            .any(|node| node.text.as_deref() == Some("Count: "))
    );
}
