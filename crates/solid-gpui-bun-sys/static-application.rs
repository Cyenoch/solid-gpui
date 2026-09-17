use solid_gpui::runtime::embedded::{CommitPoll, EmbeddedBunAdapter};
use solid_gpui::{RuntimeAdapter, Snapshot};
use std::sync::Arc;
use std::time::Duration;

fn start() -> Arc<EmbeddedBunAdapter> {
    EmbeddedBunAdapter::start_packaged(BUN_EMBEDDED_ENTRY).unwrap_or_else(|error| {
        eprintln!("Embedded application startup failed: {error}");
        std::process::exit(1);
    })
}

fn check_bundle() -> Result<(), Box<dyn std::error::Error>> {
    for session in 1..=2 {
        let runtime = start();
        let result = match runtime.recv_commit_timeout(Duration::from_secs(15)) {
            Ok(CommitPoll::Commit(bytes)) => Snapshot::decode(&bytes)
                .map(|snapshot| {
                    println!(
                        "Embedded session {session}: surface={} epoch={} nodes={}",
                        snapshot.surface_id,
                        snapshot.epoch,
                        snapshot.nodes.len()
                    );
                })
                .map_err(|error| error.to_string()),
            other => Err(format!("Expected initial Snapshot, received {other:?}")),
        };
        let shutdown = runtime.shutdown();
        result?;
        shutdown?;
    }
    println!("Embedded bundle startup, shutdown and same-process restart verified");
    Ok(())
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args == ["--version"] {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return;
    }
    if args == ["--check-bundle"] {
        if let Err(error) = check_bundle() {
            eprintln!("Embedded bundle check failed: {error}");
            std::process::exit(1);
        }
        return;
    }
    if !args.is_empty() {
        eprintln!("Usage: {} [--version | --check-bundle]", env!("CARGO_PKG_NAME"));
        std::process::exit(2);
    }
    solid_gpui::run_application_with_profile(solid_gpui::host::DefaultHostProfile::default, start());
}
