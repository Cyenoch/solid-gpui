#![cfg(all(target_os = "macos", feature = "embedded-bun"))]

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use react_gpui::{KIND_RAW_TEXT, RuntimeAdapter, Snapshot};
use react_gpui_bun::{CommitPoll, EmbeddedBunAdapter};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);

const EXAMPLES: &[(&str, &str, usize)] = &[
    ("gallery", "packages/react-gpui/examples/gallery.tsx", 30),
    (
        "text-input",
        "packages/react-gpui/examples/text-input.tsx",
        10,
    ),
    (
        "virtual-list",
        "packages/react-gpui/examples/virtual-list.tsx",
        20,
    ),
    ("notes", "packages/react-gpui/examples/notes.tsx", 10),
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repository root")
        .to_path_buf()
}

fn startup_snapshot(runtime: &EmbeddedBunAdapter, name: &str) -> Snapshot {
    for _ in 0..8 {
        let payload = match runtime
            .recv_commit_timeout(STARTUP_TIMEOUT)
            .unwrap_or_else(|error| panic!("{name}: embedded startup transport failed: {error}"))
        {
            CommitPoll::Commit(payload) => payload,
            CommitPoll::Timeout => panic!("{name}: embedded runtime emitted no startup Snapshot"),
            CommitPoll::Ended => panic!(
                "{name}: embedded runtime ended before startup Snapshot (status={:?})",
                runtime.runtime_status()
            ),
        };
        if let Ok(snapshot) = Snapshot::decode(&payload)
            && snapshot.nodes.len() > 1
        {
            return snapshot;
        }
    }
    panic!("{name}: embedded runtime emitted no nontrivial Snapshot within eight commits");
}

fn contains_startup_signal(snapshot: &Snapshot, expected: &str) -> bool {
    snapshot.nodes.iter().any(|node| {
        node.text
            .as_deref()
            .is_some_and(|text| text.contains(expected))
            || node.accessibility.as_ref().is_some_and(|accessibility| {
                accessibility
                    .label
                    .as_deref()
                    .is_some_and(|label| label.contains(expected))
                    || accessibility
                        .description
                        .as_deref()
                        .is_some_and(|description| description.contains(expected))
            })
    })
}

fn assert_clean_shutdown(runtime: &EmbeddedBunAdapter, name: &str) {
    runtime
        .shutdown()
        .unwrap_or_else(|error| panic!("{name}: embedded shutdown failed: {error}"));
    assert_eq!(
        runtime.runtime_status(),
        Some(0),
        "{name}: embedded Bun returned a nonzero startup status"
    );
    assert!(
        runtime.commit_count() >= 1,
        "{name}: embedded runtime accepted no commit frames"
    );
}

fn expected_signal(name: &str) -> &'static str {
    match name {
        "gallery" => "React GPUI Gallery",
        "text-input" => "Text input demo",
        "virtual-list" => "Row 0",
        "notes" => "Choose a note to begin",
        _ => unreachable!("matrix entry without startup signal"),
    }
}

fn run_example(name: &str, entry: &str, minimum_nodes: usize) {
    let root = repo_root();
    let runtime = EmbeddedBunAdapter::start(root.join(entry))
        .unwrap_or_else(|error| panic!("{name}: embedded Bun failed to start: {error}"));
    let snapshot = startup_snapshot(&runtime, name);
    assert_eq!(snapshot.surface_id, 1, "{name}: startup surface must be 1");
    assert_eq!(snapshot.epoch, 1, "{name}: startup epoch must be 1");
    assert!(
        snapshot.nodes.len() >= minimum_nodes,
        "{name}: startup tree is unexpectedly small ({} nodes, expected at least {minimum_nodes}): {:?}",
        snapshot.nodes.len(),
        snapshot.nodes
    );
    assert!(
        snapshot.nodes.len() < 10_000,
        "{name}: startup tree exceeded the bounded integration-test size"
    );
    assert!(
        snapshot.nodes.iter().any(|node| node.kind == KIND_RAW_TEXT),
        "{name}: startup Snapshot contains no raw text nodes"
    );
    let expected = expected_signal(name);
    assert!(
        contains_startup_signal(&snapshot, expected),
        "{name}: startup Snapshot did not contain known signal {expected:?}"
    );
    assert!(
        runtime.runtime_status().is_none(),
        "{name}: embedded runtime terminated during startup (status={:?})",
        runtime.runtime_status()
    );
    assert_clean_shutdown(&runtime, name);
}

#[test]
fn embedded_representative_examples_emit_startup_snapshots() {
    if let Ok(name) = env::var("REACT_GPUI_EMBEDDED_EXAMPLE_WORKER") {
        let &(name, entry, minimum_nodes) = EXAMPLES
            .iter()
            .find(|(candidate, _, _)| *candidate == name)
            .unwrap_or_else(|| panic!("unknown embedded example worker {name:?}"));
        run_example(name, entry, minimum_nodes);
        return;
    }

    let executable = env::current_exe().expect("embedded example test executable");
    for &(name, _, _) in EXAMPLES {
        let output = Command::new(&executable)
            .env("REACT_GPUI_EMBEDDED_EXAMPLE_WORKER", name)
            .args([
                "embedded_representative_examples_emit_startup_snapshots",
                "--exact",
            ])
            .output()
            .unwrap_or_else(|error| panic!("{name}: spawn embedded example worker: {error}"));
        assert!(
            output.status.success(),
            "{name}: embedded example worker failed; stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !String::from_utf8_lossy(&output.stderr).contains("bun embedded eval failed"),
            "{name}: embedded Bun reported an evaluation failure on stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn embedded_refresh_queue_keeps_runtime_alive_until_shutdown() {
    let root = repo_root();
    let entry = root.join("packages/react-gpui/examples/counter.tsx");
    let runtime = EmbeddedBunAdapter::start(&entry).expect("embedded counter starts for refresh");
    let initial = startup_snapshot(&runtime, "counter refresh");
    assert!(contains_startup_signal(&initial, "Increment"));

    runtime
        .refresh_entry(&entry)
        .expect("embedded refresh entry queues");
    let refreshed = match runtime
        .recv_commit_timeout(STARTUP_TIMEOUT)
        .expect("embedded refresh transport")
    {
        CommitPoll::Commit(payload) => Snapshot::decode(&payload).expect("refresh is a Snapshot"),
        CommitPoll::Timeout => panic!("embedded refresh emitted no commit"),
        CommitPoll::Ended => panic!(
            "embedded refresh ended runtime before commit (status={:?})",
            runtime.runtime_status()
        ),
    };
    assert!(
        refreshed
            .nodes
            .iter()
            .any(|node| node.kind == KIND_RAW_TEXT),
        "embedded refresh commit contains no text nodes"
    );
    assert_eq!(runtime.refresh_count(), 1);
    assert!(
        runtime.runtime_status().is_none(),
        "embedded refresh terminated runtime before shutdown (status={:?})",
        runtime.runtime_status()
    );
    assert_clean_shutdown(&runtime, "counter refresh");
}
