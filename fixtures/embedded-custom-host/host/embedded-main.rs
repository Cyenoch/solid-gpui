// Included by the generated binary crate after the module-graph include, so
// `BUN_EMBEDDED_ENTRY` and `BUN_EMBEDDED_WORKERS` are already in scope: the
// serializer's identities, not file names reconstructed at run time.
//
// It lives outside `src/` on purpose. It is not compiled on its own — the
// constants it uses belong to the generated crate — so it must not be
// auto-detected as the host package's binary target.

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--check-result") {
        if let Err(error) = check_result() {
            eprintln!("Embedded result check failed: {error}");
            std::process::exit(1);
        }
        return;
    }
    let session =
        embedded_custom_host::Session::start(BUN_EMBEDDED_ENTRY).unwrap_or_else(|error| {
            eprintln!("Embedded application startup failed: {error}");
            std::process::exit(1);
        });
    if BUN_EMBEDDED_WORKERS.len() > 1 {
        eprintln!(
            "Embedded application declared {} worker entries; only one is expected",
            BUN_EMBEDDED_WORKERS.len()
        );
    }
    solid_gpui::run_application_with_profile(
        solid_gpui::host::DefaultHostProfile::default,
        session.runtime(),
    );
}

/// Runs the packaged session headlessly and prints the completion result the
/// application declared.
///
/// This is the packaged-path check for the typed result: the JavaScript entry
/// calls the published completion API, and this host reads what the runtime
/// recorded. It opens no window and sends no input, so it proves the ABI and
/// packaging path only — not UI behaviour.
fn check_result() -> Result<(), Box<dyn std::error::Error>> {
    use solid_gpui::RuntimeAdapter as _;
    use solid_gpui::runtime::embedded::{CommitPoll, EmbeddedResult};

    let session = embedded_custom_host::Session::start(BUN_EMBEDDED_ENTRY)?;
    let runtime = session.runtime();
    loop {
        match runtime.recv_commit_timeout(std::time::Duration::from_secs(30))? {
            CommitPoll::Commit(_) => continue,
            CommitPoll::Timeout => {
                return Err("the packaged session produced no outcome within 30 seconds".into());
            }
            CommitPoll::Ended => break,
        }
    }
    runtime.shutdown()?;
    match session.declared_result() {
        Some(EmbeddedResult { code }) => {
            println!("declared result: {code}");
            Ok(())
        }
        None => Err("the packaged application declared no completion result".into()),
    }
}
