#![cfg(feature = "embedded-bun")]

use std::path::PathBuf;
use std::sync::Arc;

use solid_gpui::runtime::embedded::{CommitPoll, EmbeddedBunAdapter};
use solid_gpui::{Event, RuntimeAdapter, Snapshot};

fn counter_entry() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/press-roundtrip.ts")
}

fn counter_roundtrip() {
    eprintln!("embedded case: counter roundtrip");
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
        .send_event(Event::press(
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

struct Script(PathBuf);
impl Script {
    fn new(name: &str, source: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("solid-bun-tests-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join(format!("{name}.mjs"));
        std::fs::write(&path, source).unwrap();
        Self(path)
    }

    fn start(&self) -> Arc<EmbeddedBunAdapter> {
        eprintln!("embedded case: {}", self.0.display());
        EmbeddedBunAdapter::start(&self.0).unwrap()
    }
}
impl Drop for Script {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

const WRITE_FRAME: &str = r#"
function frame(n) {
  const bytes = Buffer.alloc(8);
  bytes.writeUInt32LE(4, 0);
  bytes.writeUInt32LE(n, 4);
  return process.stdout.write(bytes);
}
"#;

fn number(runtime: &EmbeddedBunAdapter) -> u32 {
    match runtime
        .recv_commit_timeout(std::time::Duration::from_secs(20))
        .unwrap()
    {
        CommitPoll::Commit(payload) => u32::from_le_bytes(payload.try_into().unwrap()),
        other => panic!("expected frame, received {other:?}"),
    }
}

fn stop(runtime: &Arc<EmbeddedBunAdapter>) {
    let runtime = Arc::clone(runtime);
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || sender.send(runtime.shutdown()).unwrap());
    receiver
        .recv_timeout(std::time::Duration::from_secs(20))
        .expect("VM teardown must interrupt busy JS and close live resources")
        .unwrap();
}

// A single test deliberately owns the process-wide JSC engine across all
// sessions. These are actual VM/ABI checks, including restart after termination.
#[test]
fn embedded_vm_roundtrip_pressure_termination_and_restart() {
    let minimal = Script::new("bootstrap", &format!("{WRITE_FRAME}\nframe(123);"));
    let runtime = minimal.start();
    assert_eq!(number(&runtime), 123);
    stop(&runtime);
    std::fs::write(&minimal.0, format!("{WRITE_FRAME}\nframe(124);")).unwrap();
    let runtime = minimal.start();
    assert_eq!(
        number(&runtime),
        124,
        "a new session sees changed entry contents"
    );
    stop(&runtime);
    counter_roundtrip();

    let eof = Script::new(
        "input-eof",
        &format!(
            r#"{WRITE_FRAME}
let received = 0;
process.stdin.on('data', () => received++);
process.stdin.on('end', () => frame(received));
process.stdin.on('close', () => frame(99));
process.once('beforeExit', () => setTimeout(() => frame(101), 0));
frame(0);
"#
        ),
    );
    let runtime = eof.start();
    assert_eq!(number(&runtime), 0);
    for sequence in 1..=3 {
        runtime
            .send_event(Event::press(1, 1, 1, sequence, 2, 2))
            .unwrap();
    }
    runtime.close_input();
    runtime.close_input();
    assert!(runtime.send_event(Event::press(1, 1, 1, 4, 2, 2)).is_err());
    assert_eq!(
        number(&runtime),
        3,
        "EOF follows every already queued input"
    );
    assert_eq!(
        number(&runtime),
        99,
        "input closes once, with output still writable"
    );
    assert_eq!(
        number(&runtime),
        101,
        "beforeExit can schedule its last async work"
    );
    assert_eq!(
        runtime
            .recv_commit_timeout(std::time::Duration::from_secs(20))
            .unwrap(),
        CommitPoll::Ended
    );
    assert_eq!(runtime.runtime_status(), Some(0));
    stop(&runtime);

    let early_input = Script::new(
        "early-input",
        &format!(
            r#"{WRITE_FRAME}
let count = 0;
await new Promise(resolve => setTimeout(resolve, 5));
process.stdin.on('data', () => {{ if (++count === 512) frame(count); }});
frame(0);
setInterval(() => {{}}, 1000);
"#
        ),
    );
    let runtime = early_input.start();
    assert!(matches!(
        EmbeddedBunAdapter::start(&early_input.0),
        Err(solid_gpui::runtime::embedded::EmbeddedBunError::SessionActive)
    ));
    for sequence in 1..=512 {
        runtime
            .send_event(Event::press(1, 1, 1, sequence, 2, 2))
            .unwrap();
    }
    assert_eq!(number(&runtime), 0);
    assert_eq!(
        number(&runtime),
        512,
        "input queued before VM readiness remains ordered and wakes JS"
    );
    stop(&runtime);

    let recovered = Script::new(
        "handled-input-error",
        &format!(
            r#"{WRITE_FRAME}
let calls = 0;
process.on('uncaughtException', () => {{}});
process.stdin.on('data', () => {{
  if (++calls === 1) throw new Error('handled listener failure');
  frame(calls);
}});
frame(0);
"#
        ),
    );
    let runtime = recovered.start();
    assert_eq!(number(&runtime), 0);
    for sequence in 1..=2 {
        runtime
            .send_event(Event::press(1, 1, 1, sequence, 2, 2))
            .unwrap();
    }
    assert_eq!(
        number(&runtime),
        2,
        "handled listener failure must not strand queued input"
    );
    stop(&runtime);

    let pressure = Script::new(
        "pressure",
        &format!(
            r#"{WRITE_FRAME}
import {{ once }} from 'node:events';
for (let n = 1; n <= 128; n++) {{
  if (!frame(n)) await once(process.stdout, 'drain');
}}
frame(999);
"#
        ),
    );
    let runtime = pressure.start();
    // Let the producer reach the watermark before the consumer starts reading.
    std::thread::sleep(std::time::Duration::from_millis(100));
    for expected in 1..=128 {
        assert_eq!(
            number(&runtime),
            expected,
            "write(false) must retain the frame"
        );
    }
    assert_eq!(number(&runtime), 999);
    stop(&runtime);

    let _worker = Script::new(
        "network-worker",
        "setInterval(() => {}, 1000); postMessage(23);",
    );
    let network = Script::new(
        "network",
        &format!(
            r#"{WRITE_FRAME}
const worker = new Worker(new URL('./network-worker.mjs', import.meta.url));
const workerReady = await new Promise((resolve, reject) => {{
  worker.onmessage = event => resolve(event.data);
  worker.onerror = event => reject(new Error(event.message));
}});
if (workerReady !== 23) throw new Error('worker response mismatch');
let pendingStarted;
const pendingObserved = new Promise(resolve => {{ pendingStarted = resolve; }});
const server = Bun.serve({{ port: 0, async fetch(request) {{
  if (new URL(request.url).pathname === '/pending') {{
    pendingStarted();
    await new Promise(() => {{}});
  }}
  return new Response('ready');
}} }});
const response = await fetch(`http://127.0.0.1:${{server.port}}`);
if (await response.text() !== 'ready') throw new Error('network response mismatch');
void fetch(`http://127.0.0.1:${{server.port}}/pending`).catch(() => {{}});
await pendingObserved;
await new Promise(resolve => setTimeout(resolve, 5));
frame(42);
setInterval(() => {{}}, 1000);
"#
        ),
    );
    for _ in 0..3 {
        let runtime = network.start();
        assert_eq!(
            number(&runtime),
            42,
            "network and timers work after prior VM teardown"
        );
        stop(&runtime);
        assert!(runtime.runtime_status().is_some());
    }

    let callback_loop = Script::new(
        "busy-input-callback",
        &format!(
            "{WRITE_FRAME}\nprocess.stdin.on('data', () => {{ frame(7); while (true) {{}} }}); frame(0);"
        ),
    );
    let runtime = callback_loop.start();
    assert_eq!(number(&runtime), 0);
    runtime.send_event(Event::press(1, 1, 1, 1, 2, 2)).unwrap();
    assert_eq!(number(&runtime), 7);
    stop(&runtime);

    for (name, busy) in [
        ("busy-loop", "while (true) {}"),
        (
            "busy-microtasks",
            "const spin = () => queueMicrotask(spin); spin();",
        ),
    ] {
        let script = Script::new(name, &format!("{WRITE_FRAME}\nframe(7);\n{busy}"));
        let runtime = script.start();
        assert_eq!(number(&runtime), 7);
        stop(&runtime);
    }

    let exit = Script::new(
        "exit",
        &format!("{WRITE_FRAME}\nprocess.on('exit', () => frame(12)); frame(11); process.exit(7);"),
    );
    let runtime = exit.start();
    assert_eq!(number(&runtime), 11);
    assert_eq!(
        number(&runtime),
        12,
        "process.exit dispatches its exit listeners once"
    );
    assert!(
        runtime
            .recv_commit_timeout(std::time::Duration::from_secs(20))
            .is_err()
    );
    assert_eq!(
        runtime.runtime_status(),
        Some(7),
        "process.exit must end the VM, keeping the host alive"
    );
    stop(&runtime);

    let runtime = network.start();
    stop(&runtime); // Termination may arrive before VM publication.
    let rejected = Script::new(
        "rejected-entry",
        "await new Promise(resolve => setTimeout(resolve, 5)); throw new Error('entry rejected');",
    );
    let runtime = rejected.start();
    assert!(
        runtime
            .recv_commit_timeout(std::time::Duration::from_secs(20))
            .is_err()
    );
    assert_eq!(runtime.runtime_status(), Some(1));
    stop(&runtime);
    counter_roundtrip(); // A terminated session cannot poison the next VM.
}
