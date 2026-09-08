//! Development-only generation supervision. The host and its Rust services stay alive.
use super::quickjs::{CommitPoll, QuickJsAdapter};
use crate::protocol::{Command, CommandOperation, DecodedMessage, decode_message};
use crate::{Event, ProtocolError, RuntimeAdapter, RuntimeStatus, Snapshot};
use std::{
    io::{self, Read, Write},
    net::TcpStream,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

const MAX_SOURCE: usize = 32 * 1024 * 1024;
const PREPARE_TIMEOUT: Duration = Duration::from_secs(3);

pub struct Replacement {
    pub snapshots: Vec<Snapshot>,
    pub configuration: Command,
    pub epoch: u32,
    candidate: Arc<QuickJsAdapter>,
    active: Arc<Mutex<Arc<QuickJsAdapter>>>,
    previous: Arc<QuickJsAdapter>,
    latest: Arc<AtomicU64>,
    build: u64,
    response: Option<std::sync::mpsc::SyncSender<Result<(), String>>>,
}
impl Replacement {
    /// Called only after the foreground has validated every affected Surface.
    pub fn begin(&self) -> Result<(), String> {
        if self.latest.load(Ordering::Acquire) != self.build {
            return Err("reload superseded".into());
        }
        *self.active.lock().unwrap() = Arc::clone(&self.candidate);
        Ok(())
    }
    pub fn finish(mut self) {
        self.candidate.resume(true);
        let _ = self.previous.request_shutdown();
        let previous = Arc::clone(&self.previous);
        std::thread::spawn(move || {
            let _ = previous.shutdown();
        });
        let _ = self.response.take().unwrap().send(Ok(()));
    }
    pub fn reject(mut self, error: String) {
        self.previous.resume(false);
        let _ = self.candidate.request_shutdown();
        let _ = self.response.take().unwrap().send(Err(error));
    }
}
impl Drop for Replacement {
    fn drop(&mut self) {
        if self.response.is_some() {
            self.previous.resume(false);
            let _ = self.candidate.request_shutdown();
        }
    }
}

struct Build {
    id: u64,
    source: Vec<u8>,
}
/// Stable adapter identity retained by native roots, commands, and the commit pump.
pub struct ReloadableQuickJs {
    active: Arc<Mutex<Arc<QuickJsAdapter>>>,
    latest: Arc<AtomicU64>,
    pending: Arc<(Mutex<Option<Build>>, Condvar)>,
    acknowledgement: Mutex<Option<std::sync::mpsc::Receiver<Result<(), String>>>>,
    control: Mutex<TcpStream>,
    epoch: Mutex<u32>,
    name: String,
    stopped: AtomicBool,
    recovery_state: Mutex<String>,
}
impl ReloadableQuickJs {
    pub fn start(entry: &str, address: &str) -> Result<Arc<Self>, String> {
        let address: std::net::SocketAddr = address
            .parse()
            .map_err(|_| "invalid development endpoint")?;
        if !address.ip().is_loopback() {
            return Err("development endpoint must be loopback".into());
        }
        let control = TcpStream::connect(address).map_err(|e| e.to_string())?;
        let mut reader = control.try_clone().map_err(|e| e.to_string())?;
        let initial = QuickJsAdapter::launch(
            entry.into(),
            std::fs::read(entry).map_err(|e| e.to_string())?,
            Some((
                1,
                r#"{"state":[],"surfaceId":1,"open":true,"activationSequence":0}"#.into(),
            )),
        )
        .map_err(|e| e.to_string())?;
        // Initial launch has no old generation to preserve. Publish its normal
        // frames and run onMount, using the same generation lifecycle as reload.
        if let Err(error) = initial.wait_ready(PREPARE_TIMEOUT) {
            let _ = initial.shutdown();
            return Err(error);
        }
        initial.resume(true);
        let latest = Arc::new(AtomicU64::new(0));
        let pending = Arc::new((Mutex::new(None), Condvar::new()));
        let result = Arc::new(Self {
            active: Arc::new(Mutex::new(initial)),
            latest: Arc::clone(&latest),
            pending: Arc::clone(&pending),
            acknowledgement: Mutex::new(None),
            control: Mutex::new(control),
            epoch: Mutex::new(1),
            name: entry.into(),
            stopped: AtomicBool::new(false),
            recovery_state: Mutex::new(
                r#"{"state":[],"surfaceId":1,"open":true,"activationSequence":0}"#.into(),
            ),
        });
        let owner = Arc::downgrade(&result);
        std::thread::Builder::new()
            .name("quickjs-dev-control".into())
            .spawn(move || {
                loop {
                    let mut length = [0; 4];
                    if reader.read_exact(&mut length).is_err() {
                        break;
                    }
                    let length = u32::from_le_bytes(length) as usize;
                    if length == 0 || length > MAX_SOURCE {
                        break;
                    }
                    let mut source = vec![0; length];
                    if reader.read_exact(&mut source).is_err() {
                        break;
                    }
                    let id = latest.fetch_add(1, Ordering::AcqRel) + 1;
                    *pending.0.lock().unwrap() = Some(Build { id, source });
                    pending.1.notify_all();
                    if let Some(owner) = owner.upgrade() {
                        owner.active().wake_reader();
                    }
                }
                if let Some(owner) = owner.upgrade() {
                    let _ = owner.request_shutdown();
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(result)
    }
    fn active(&self) -> Arc<QuickJsAdapter> {
        Arc::clone(&self.active.lock().unwrap())
    }
    fn update_recovery(&self, update: impl FnOnce(&mut serde_json::Value)) {
        let mut state = self.recovery_state.lock().unwrap();
        let mut value: serde_json::Value =
            serde_json::from_str(&state).expect("validated generation state");
        update(&mut value);
        *state = value.to_string();
    }

    fn report(&self, result: Result<(), String>) {
        let status = match result {
            Ok(()) => "applied".into(),
            Err(error) => format!("rejected: {error}"),
        };
        eprintln!("quickjs reload: {status}");
        let mut control = self.control.lock().unwrap();
        let _ = writeln!(control, "{}", status.replace(['\n', '\r'], " "));
    }
    fn prepare(&self, build: Build) -> Result<Replacement, String> {
        let previous = self.active();
        let state = if previous.status() == RuntimeStatus::Failed {
            self.recovery_state.lock().unwrap().clone()
        } else {
            match previous.capture() {
                Ok(state) => state,
                Err(error) => {
                    previous.resume(false);
                    return Err(error);
                }
            }
        };
        *self.recovery_state.lock().unwrap() = state.clone();
        if state.len() > 1024 * 1024 {
            previous.resume(false);
            return Err("reload state exceeds 1 MiB".into());
        }
        let Some(epoch) = self.epoch.lock().unwrap().checked_add(1) else {
            previous.resume(false);
            return Err("reload epoch exhausted".into());
        };
        let candidate = match QuickJsAdapter::launch(
            format!("{}?generation={}", self.name, build.id),
            build.source,
            Some((epoch, state)),
        ) {
            Ok(candidate) => candidate,
            Err(error) => {
                previous.resume(false);
                return Err(error.to_string());
            }
        };
        let prepare = || -> Result<(Vec<Snapshot>, Command), String> {
            candidate.wait_ready(PREPARE_TIMEOUT)?;
            {
                if self.stopped.load(Ordering::Acquire) {
                    return Err("development session closed".into());
                }
                if self.latest.load(Ordering::Acquire) != build.id {
                    return Err("reload superseded".into());
                }
                if let Some(frames) = candidate.candidate_frames()? {
                    let mut snapshots = Vec::new();
                    let mut configuration = None;
                    for frame in frames {
                        match decode_message(&frame).map_err(|e| e.to_string())? {
                            DecodedMessage::Snapshot(snapshot) if snapshot.epoch == epoch => snapshots.push(snapshot),
                            DecodedMessage::Command(command) if matches!(command.operation, CommandOperation::ConfigureApplication { quit: false, .. }) && command.meta.epoch == epoch && configuration.is_none() => configuration = Some(command),
                            _ => return Err("candidate may only stage initial snapshots and application configuration; move native effects to onMount".into()),
                        }
                    }
                    return Ok((
                        snapshots,
                        configuration.ok_or("candidate did not configure application")?,
                    ));
                }
                Err("candidate did not become ready".into())
            }
        };
        let (snapshots, configuration) = match prepare() {
            Ok(value) => value,
            Err(error) => {
                previous.resume(false);
                let _ = candidate.shutdown();
                return Err(error);
            }
        };
        let (response, acknowledgement) = std::sync::mpsc::sync_channel(1);
        *self.acknowledgement.lock().unwrap() = Some(acknowledgement);
        *self.epoch.lock().unwrap() = epoch;
        Ok(Replacement {
            snapshots,
            configuration,
            epoch,
            candidate,
            previous,
            active: Arc::clone(&self.active),
            latest: Arc::clone(&self.latest),
            build: build.id,
            response: Some(response),
        })
    }
}
impl RuntimeAdapter for ReloadableQuickJs {
    fn recv_host_commit(&self) -> Result<Option<crate::transport::HostCommit>, ProtocolError> {
        if let Some(receiver) = self.acknowledgement.lock().unwrap().take() {
            self.report(
                receiver
                    .recv()
                    .unwrap_or_else(|_| Err("host discarded replacement".into())),
            );
        }
        let mut reported_failure = false;
        loop {
            if self.stopped.load(Ordering::Acquire) {
                return Ok(None);
            }
            let build = self.pending.0.lock().unwrap().take();
            if let Some(build) = build {
                match self.prepare(build) {
                    Ok(candidate) => {
                        return Ok(Some(crate::transport::HostCommit::Replacement(Box::new(
                            candidate,
                        ))));
                    }
                    Err(error) => self.report(Err(error)),
                }
            }
            match self.active().recv_interruptible() {
                Ok(CommitPoll::Commit(frame)) => {
                    if let Ok(DecodedMessage::Command(Command {
                        operation:
                            CommandOperation::ConfigureApplication {
                                acknowledged_sequence,
                                ..
                            },
                        ..
                    })) = decode_message(&frame)
                    {
                        self.update_recovery(|state| {
                            state["activationSequence"] = acknowledged_sequence.into()
                        });
                    }
                    return Ok(Some(crate::transport::HostCommit::Frame(frame)));
                }
                Ok(CommitPoll::Ended) if self.stopped.load(Ordering::Acquire) => return Ok(None),
                Ok(CommitPoll::Timeout) => {}
                result => {
                    if !reported_failure {
                        self.report(Err(format!("active runtime stopped; edit to recover from the last captured state: {result:?}")));
                        reported_failure = true;
                    }
                    let pending = self.pending.0.lock().unwrap();
                    drop(
                        self.pending
                            .1
                            .wait_while(pending, |p| {
                                p.is_none() && !self.stopped.load(Ordering::Acquire)
                            })
                            .unwrap(),
                    );
                }
            }
        }
    }
    fn recv_commit(&self) -> Result<Option<Vec<u8>>, ProtocolError> {
        Err(ProtocolError::Io(io::Error::other(
            "reloadable runtime requires host generation coordination",
        )))
    }
    fn send_event(&self, event: Event) -> Result<(), ProtocolError> {
        match &event.payload {
            crate::EventPayload::ApplicationActivation {
                target_surface_id, ..
            } => self.update_recovery(|state| {
                state["surfaceId"] = (*target_surface_id).into();
                state["open"] = true.into();
            }),
            crate::EventPayload::SurfaceClosed => self.update_recovery(|state| {
                if state["surfaceId"].as_u64() == Some(event.meta.surface_id.into()) {
                    state["open"] = false.into();
                }
            }),
            _ => {}
        }
        let active = self.active();
        // A failed development generation has no event consumer. Keep its last
        // tree visible until the next edit; the commit reader reports the failure.
        if active.status() != RuntimeStatus::Running {
            return Ok(());
        }
        match active.send_event(event) {
            Err(_) if active.status() == RuntimeStatus::Failed => Ok(()),
            result => result,
        }
    }
    fn request_shutdown(&self) -> Result<(), ProtocolError> {
        self.stopped.store(true, Ordering::Release);
        self.pending.1.notify_all();
        let _ = self
            .control
            .lock()
            .unwrap()
            .shutdown(std::net::Shutdown::Both);
        self.active().request_shutdown()
    }
    fn shutdown(&self) -> Result<(), ProtocolError> {
        self.request_shutdown()?;
        self.active().shutdown()
    }
    fn status(&self) -> RuntimeStatus {
        if self.stopped.load(Ordering::Acquire) {
            RuntimeStatus::Shutdown
        } else {
            RuntimeStatus::Running
        }
    }
}

impl Drop for ReloadableQuickJs {
    fn drop(&mut self) {
        let _ = self.request_shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::HostCommit;
    use std::time::Instant;
    use std::{net::TcpListener, process::Command as ProcessCommand};

    #[test]
    fn real_vm_reload_preserves_state_rejects_candidates_and_retires_old_epoch() {
        let directory = std::env::temp_dir().join(format!("quickjs-reload-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let entry = directory.join("app.js");
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert!(
            ProcessCommand::new("bun")
                .current_dir(root)
                .args([
                    "packages/solid-gpui/src/vite/build.ts",
                    "--runtime",
                    "quickjs",
                    "fixtures/quickjs-counter.tsx"
                ])
                .arg(&entry)
                .status()
                .unwrap()
                .success()
        );
        let source = std::fs::read_to_string(&entry).unwrap();
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let runtime = ReloadableQuickJs::start(
            entry.to_str().unwrap(),
            &server.local_addr().unwrap().to_string(),
        )
        .unwrap();
        let (mut control, _) = server.accept().unwrap();
        control
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let (send, receive) = std::sync::mpsc::sync_channel(8);
        let reader = Arc::clone(&runtime);
        let pump = std::thread::spawn(move || {
            while let Some(frame) = reader.recv_host_commit().unwrap() {
                if send.send(frame).is_err() {
                    break;
                }
            }
        });
        let receive_frame = || match receive.recv_timeout(Duration::from_secs(5)).unwrap() {
            HostCommit::Frame(frame) => frame,
            _ => panic!("expected ordinary frame"),
        };
        let first = Snapshot::decode(&receive_frame()).unwrap();
        receive_frame(); // application configuration
        let button = first
            .nodes
            .iter()
            .find(|n| n.kind == crate::KIND_PRESSABLE)
            .unwrap();
        runtime
            .send_event(Event::press(1, 1, 1, 1, button.id, button.listener_id))
            .unwrap();
        let patch = crate::Patch::decode(&receive_frame()).unwrap();
        assert_eq!(patch.revision, 2);
        let write = |control: &mut TcpStream, source: &str| {
            control
                .write_all(&(source.len() as u32).to_le_bytes())
                .unwrap();
            control.write_all(source.as_bytes()).unwrap();
        };
        write(&mut control, &source.replace("Count:", "Updated:"));
        let HostCommit::Replacement(replacement) =
            receive.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("expected replacement");
        };
        assert_eq!(replacement.epoch, 2);
        assert!(
            replacement.snapshots[0]
                .nodes
                .iter()
                .any(|n| n.text.as_deref() == Some("Updated: 1 — 🌍")),
            "{:?}",
            replacement.snapshots[0]
                .nodes
                .iter()
                .map(|n| &n.text)
                .collect::<Vec<_>>()
        );
        let next_button = replacement.snapshots[0]
            .nodes
            .iter()
            .find(|n| n.kind == crate::KIND_PRESSABLE)
            .unwrap()
            .clone();
        replacement.begin().unwrap();
        replacement.finish();
        let mut diagnostic = std::io::BufReader::new(control.try_clone().unwrap());
        let mut line = String::new();
        use std::io::BufRead;
        diagnostic.read_line(&mut line).unwrap();
        assert!(line.contains("applied"));
        for (index, broken) in [
            "export const broken = ;".to_owned(),
            "throw new Error('broken render');".to_owned(),
            format!("{source}\nthrow new Error('broken module tail');"),
        ]
        .into_iter()
        .enumerate()
        {
            write(&mut control, &broken);
            line.clear();
            diagnostic.read_line(&mut line).unwrap();
            assert!(line.contains("rejected"), "{line}");
            runtime
                .send_event(Event::press(
                    1,
                    2,
                    index as u32 + 1,
                    index as u32 + 1,
                    next_button.id,
                    next_button.listener_id,
                ))
                .unwrap();
            // The unchanged generation still receives events after failed candidates.
            let frame = receive_frame();
            assert_eq!(crate::Patch::decode(&frame).unwrap().epoch, 2);
        }
        write(&mut control, &source);
        let HostCommit::Replacement(replacement) =
            receive.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("expected recovery");
        };
        replacement.reject("native snapshot validation rejected".into());
        line.clear();
        diagnostic.read_line(&mut line).unwrap();
        assert!(line.contains("native snapshot"));
        // Supersession is checked at the native activation boundary, not only
        // when the tooling finishes building.
        write(&mut control, &source);
        let HostCommit::Replacement(stale) = receive.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("expected staged candidate");
        };
        let requested = runtime.latest.load(Ordering::Acquire) + 1;
        write(
            &mut control,
            &format!(
                "{source}\nsetTimeout(() => {{ throw new Error('post-activation failure'); }}, 0);"
            ),
        );
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime.latest.load(Ordering::Acquire) < requested {
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert!(stale.begin().is_err());
        stale.reject("superseded at activation".into());
        line.clear();
        diagnostic.read_line(&mut line).unwrap();
        let HostCommit::Replacement(candidate) =
            receive.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("expected latest candidate");
        };
        candidate.begin().unwrap();
        candidate.finish();
        line.clear();
        diagnostic.read_line(&mut line).unwrap();
        assert!(line.contains("applied"));
        line.clear();
        diagnostic.read_line(&mut line).unwrap();
        assert!(line.contains("active runtime stopped"));
        write(&mut control, &source);
        let HostCommit::Replacement(candidate) =
            receive.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("expected recovery after active failure");
        };
        candidate.begin().unwrap();
        candidate.finish();
        runtime.shutdown().unwrap();
        pump.join().unwrap();
        std::fs::remove_dir_all(directory).unwrap();
    }
}
