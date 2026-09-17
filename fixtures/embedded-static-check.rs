// Compile with `solid-gpui embedded package --main <this file>` (or the
// checkout-local `bun run embedded:package --main <this file>`) to check a
// packaged counter without a test runner.
use solid_gpui::runtime::embedded::{CommitPoll, EmbeddedBunAdapter};
use solid_gpui::{DecodedMessage, Event, NodeStore, RuntimeAdapter, Snapshot};
use std::error::Error;
use std::time::Duration;

fn commit(runtime: &EmbeddedBunAdapter) -> Result<Vec<u8>, Box<dyn Error>> {
    match runtime.recv_commit_timeout(Duration::from_secs(30))? {
        CommitPoll::Commit(bytes) => Ok(bytes),
        other => Err(format!("Expected commit, received {other:?}").into()),
    }
}

fn counter_value(store: &NodeStore, value: u32) -> Result<(), Box<dyn Error>> {
    let expected = format!("Count: {value}");
    if !store.iter().any(|node| {
        node.text.as_deref() == Some(expected.as_str())
            || node.text_content.as_deref() == Some(expected.as_str())
    }) {
        return Err(format!("Missing {expected}").into());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let mode = args.next();
    if args.next().is_some() || !matches!(mode.as_deref(), None | Some("--gui" | "--check-graph")) {
        eprintln!("Usage: application [--gui | --check-graph]");
        std::process::exit(2);
    }
    if mode.as_deref() == Some("--gui") {
        let runtime = EmbeddedBunAdapter::start_packaged(BUN_EMBEDDED_ENTRY)?;
        solid_gpui::run_application_with_profile(
            solid_gpui::host::DefaultHostProfile::default,
            runtime,
        );
        return Ok(());
    }
    for session in 1..=2 {
        let runtime = EmbeddedBunAdapter::start_packaged(BUN_EMBEDDED_ENTRY)?;
        let snapshot = Snapshot::decode(&commit(&runtime)?)?;
        let mut store = NodeStore::empty();
        store.apply_snapshot(snapshot)?;
        if mode.as_deref() == Some("--check-graph") {
            match solid_gpui::decode_message(&commit(&runtime)?)? {
                DecodedMessage::Command(command) => {
                    if !matches!(
                        command.operation,
                        solid_gpui::protocol::CommandOperation::ConfigureApplication {
                            quit: false,
                            ..
                        }
                    ) {
                        return Err(format!("Unexpected startup command: {command:?}").into());
                    }
                }
                _ => return Err("Expected application startup control".into()),
            }
            for expected in [
                "Dynamic: dynamic-module-ok",
                "Worker: 42",
                "Resource: embedded-resource-ok",
                "Native extraction: blocked",
                "Child interpreter: guarded",
            ] {
                if !store.iter().any(|node| {
                    node.text.as_deref() == Some(expected)
                        || node.text_content.as_deref() == Some(expected)
                }) {
                    return Err(format!("Missing {expected}").into());
                }
            }
        }
        counter_value(&store, 0)?;
        let (node_id, listener_id) = store
            .iter()
            .find(|node| node.listener_id != 0)
            .map(|node| (node.id, node.listener_id))
            .ok_or("Missing counter listener")?;
        for count in 1..=3 {
            runtime.send_event(Event::press(
                store.surface_id(),
                store.epoch(),
                store.revision(),
                count,
                node_id,
                listener_id,
            ))?;
            match solid_gpui::decode_message(&commit(&runtime)?)? {
                DecodedMessage::Patch(patch) => store.apply_patch(patch)?,
                DecodedMessage::Snapshot(_) => {
                    return Err("Expected counter patch, got Snapshot".into());
                }
                DecodedMessage::Command(command) => {
                    return Err(format!("Expected counter patch, got {command:?}").into());
                }
            }
            counter_value(&store, count)?;
        }
        runtime.shutdown()?;
        match runtime.recv_commit_timeout(Duration::from_millis(10))? {
            CommitPoll::Ended => {}
            other => return Err(format!("Expected ended transport, got {other:?}").into()),
        }
        println!("Session {session}: initial count 0, three presses, count 3, clean shutdown");
    }
    if mode.as_deref() == Some("--check-graph") {
        println!(
            "PASS: Vite JSX, dynamic import, Worker, resource and native/child policies across two sessions"
        );
    }
    println!("PASS: packaged counter input and same-process restart");
    Ok(())
}
