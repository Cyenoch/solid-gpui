//! A process-scoped Bun owner thread, with one VM session at a time.
//!
//! JSC binds its main RunLoop to the first initialization thread. Keeping that
//! thread alive across sessions preserves its identity; each session still owns
//! and tears down its VM. Only owned protocol bytes cross this boundary.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::transport::ProtocolTap;
use crate::{Event, MAX_FRAME_LENGTH, ProtocolError, RuntimeAdapter, RuntimeStatus};
use thiserror::Error;

const EVENT_CAPACITY: usize = 4096;
const COMMIT_HIGH_WATER: usize = 32;
const INPUT_READY: u32 = 1;
const OUTPUT_DRAIN: u32 = 2;

/// The ABI selects the entry kind with a leading NUL byte: a tagged entry is a
/// bundled module-graph identity, an untagged one is a disk path. Mirrors
/// `solid_gpui_bun_sys::PACKAGED_ENTRY_TAG`.
const PACKAGED_ENTRY_TAG: u8 = 0x00;

#[derive(Default)]
struct Frames {
    frames: VecDeque<Vec<u8>>,
    bytes: usize,
    closed: bool,
    pressured: bool,
}

impl Frames {
    fn pop(&mut self) -> Option<Vec<u8>> {
        let frame = self.frames.pop_front()?;
        self.bytes -= frame.len();
        Some(frame)
    }

    fn writable(&self) -> bool {
        self.frames.len() < COMMIT_HIGH_WATER && self.bytes < MAX_FRAME_LENGTH
    }

    #[cfg(any(feature = "embedded-bun", test))]
    fn accept_commit(&mut self, payload: &[u8]) -> i32 {
        // write(false) accepts the current frame. The second watermark catches
        // a producer that continues writing without waiting for drain.
        if self.closed
            || self.frames.len() >= 2 * COMMIT_HIGH_WATER
            || payload.len() > (2 * MAX_FRAME_LENGTH).saturating_sub(self.bytes)
        {
            return -1;
        }
        self.bytes += payload.len();
        self.frames.push_back(payload.to_vec());
        self.pressured |= !self.writable();
        i32::from(!self.pressured)
    }

    fn take_drain(&mut self) -> bool {
        let drain = self.pressured && self.writable();
        if drain {
            self.pressured = false;
        }
        drain
    }
}

struct State {
    commits: Mutex<Frames>,
    commit_ready: Condvar,
    events: Mutex<Frames>,
    shutdown_requested: AtomicBool,
    runtime_status: Mutex<Option<i32>>,
    /// The completion the application declared, published with the status once
    /// the session has settled.
    result: Mutex<Option<EmbeddedResult>>,
    #[cfg(feature = "embedded-bun")]
    terminated: Condvar,
    commits_seen: AtomicUsize,
    #[cfg(feature = "embedded-bun")]
    control: engine::Control,
}

impl State {
    fn wake(&self, flags: u32) {
        #[cfg(feature = "embedded-bun")]
        self.control.wake(flags);
        #[cfg(not(feature = "embedded-bun"))]
        let _ = flags;
    }

    fn request_shutdown(&self) {
        if self.shutdown_requested.swap(true, Ordering::AcqRel) {
            return;
        }
        self.events.lock().unwrap().closed = true;
        self.commits.lock().unwrap().closed = true;
        self.commit_ready.notify_all();
        #[cfg(feature = "embedded-bun")]
        self.control.terminate();
    }
}

/// Converts the ABI's completion record into the host's typed view.
///
/// `present` is the ABI's explicit presence flag, not a sentinel code: a
/// session that declared the result `0` still has a result, and one that never
/// declared a completion has none regardless of what its exit status was.
#[cfg(feature = "embedded-bun")]
fn declared_result(raw: solid_gpui_bun_sys::BunEmbeddedResult) -> Option<EmbeddedResult> {
    (raw.present == 1).then_some(EmbeddedResult { code: raw.code })
}

#[derive(Debug, Error)]
pub enum EmbeddedBunError {
    #[error("embedded Bun support is disabled; enable the `embedded-bun` feature")]
    FeatureDisabled,
    #[error("embedded Bun entry does not exist: {0}")]
    MissingEntry(PathBuf),
    #[error("embedded Bun owner thread failed to start")]
    ThreadStart,
    #[error("another embedded Bun session is active; shut it down before starting a new session")]
    SessionActive,
    #[error("embedded Bun packaged entry is not a graph identity: {0}")]
    InvalidGraphEntry(String),
}

/// Diagnose a negative session status. Packaged graph failures are the only
/// negative statuses: VM exit codes occupy `0..=255`, so the two never
/// collide, and a packaged session that fails here never evaluated anything and
/// never consulted the filesystem.
#[cfg(feature = "embedded-bun")]
fn graph_failure(status: i32) -> Option<&'static str> {
    use solid_gpui_bun_sys::packaged_graph_status as graph;
    match status {
        graph::UNAVAILABLE => Some("this executable exposes no usable bundled module graph"),
        graph::MALFORMED => Some("the bundled module graph is corrupt"),
        graph::NOT_VIRTUAL => Some("the packaged entry is not a virtual module-graph path"),
        graph::MISSING => Some("the bundled module graph has no such entry"),
        graph::BYTECODE => Some("the bundled module graph carries precompiled bytecode"),
        graph::NATIVE_LIBRARY => Some("the bundled module graph embeds a native library"),
        _ => None,
    }
}

#[cfg(not(feature = "embedded-bun"))]
fn graph_failure(_status: i32) -> Option<&'static str> {
    None
}

#[derive(Debug, Eq, PartialEq)]
pub enum CommitPoll {
    Commit(Vec<u8>),
    Timeout,
    Ended,
}

/// The application's declared completion of an embedded session.
///
/// This is not the VM's exit status. `RuntimeStatus::Exited` reports what the
/// VM exited with, which the operating system limits to one byte; an application
/// that reports a real process result — a Windows UAC cancellation is 1223 —
/// declares it through `@solid-gpui/core/embedded`'s completion API, and the
/// host reads the full 32-bit value here. A session that never declared one has
/// no result, and its outcome is only the VM exit status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmbeddedResult {
    pub code: u32,
}

pub struct EmbeddedBunAdapter {
    state: Arc<State>,
    tap: ProtocolTap,
}

impl EmbeddedBunAdapter {
    /// Start a session on the process-scoped engine thread. All application
    /// surfaces share this session. `shutdown` waits for complete VM teardown
    /// and releases admission for the next session.
    ///
    /// `entry` is the application path the host will `import()`, canonicalized
    /// so the module loader sees the same identity the host resolved.
    pub fn start(entry: impl AsRef<Path>) -> Result<Arc<Self>, EmbeddedBunError> {
        let entry = entry.as_ref();
        let entry = std::fs::canonicalize(entry)
            .map_err(|_| EmbeddedBunError::MissingEntry(entry.to_path_buf()))?;
        Self::admit(entry.as_os_str().as_encoded_bytes().to_vec())
    }

    /// Start a session whose application is served from the executable's
    /// embedded module graph.
    ///
    /// `entry` is the graph key the packager emitted, including its virtual
    /// root (`/$bunfs/root/index.js`, or `B:/~BUN/root/index.js` on Windows).
    /// The runtime resolves nothing for it on disk: if the executable has no
    /// graph, or no such entry, the session fails closed and reports a
    /// `packaged_graph_status` through the commit/status path instead of
    /// evaluating some other file.
    pub fn start_packaged(entry: &str) -> Result<Arc<Self>, EmbeddedBunError> {
        if entry.is_empty() || entry.contains('\0') {
            return Err(EmbeddedBunError::InvalidGraphEntry(entry.to_owned()));
        }
        let mut bytes = Vec::with_capacity(entry.len() + 1);
        bytes.push(PACKAGED_ENTRY_TAG);
        bytes.extend_from_slice(entry.as_bytes());
        Self::admit(bytes)
    }

    /// Take the process-wide session slot and hand the entry to the owner
    /// thread. `entry` is already in the ABI's byte form, because only the
    /// entry's kind decides whether a filesystem path or a graph identity will
    /// be evaluated.
    #[cfg(feature = "embedded-bun")]
    fn admit(entry: Vec<u8>) -> Result<Arc<Self>, EmbeddedBunError> {
        let engine = engine::owner()?;
        if engine.active.swap(true, Ordering::AcqRel) {
            return Err(EmbeddedBunError::SessionActive);
        }
        let state = Arc::new(State {
            commits: Mutex::new(Frames::default()),
            commit_ready: Condvar::new(),
            events: Mutex::new(Frames::default()),
            shutdown_requested: AtomicBool::new(false),
            runtime_status: Mutex::new(None),
            result: Mutex::new(None),
            terminated: Condvar::new(),
            commits_seen: AtomicUsize::new(0),
            control: engine::Control::new(),
        });
        if engine.sender.send((entry, Arc::clone(&state))).is_err() {
            engine.active.store(false, Ordering::Release);
            return Err(EmbeddedBunError::ThreadStart);
        }
        Ok(Arc::new(Self {
            state,
            tap: ProtocolTap::from_env(),
        }))
    }

    #[cfg(not(feature = "embedded-bun"))]
    fn admit(entry: Vec<u8>) -> Result<Arc<Self>, EmbeddedBunError> {
        let _ = entry;
        Err(EmbeddedBunError::FeatureDisabled)
    }

    pub fn commit_count(&self) -> usize {
        self.state.commits_seen.load(Ordering::Acquire)
    }

    /// A status is published only after the session VM has been destroyed.
    /// A negative status describes a packaged session that never started (the
    /// negative graph statuses of the embedding ABI); VM exit codes are
    /// `0..=255`.
    pub fn runtime_status(&self) -> Option<i32> {
        *self.state.runtime_status.lock().unwrap()
    }

    /// The completion the application declared, if it declared one.
    ///
    /// Independent of [`Self::runtime_status`]: an application can report a
    /// result and still exit with whatever status its VM ended on, and a session
    /// that never declared a result reports `None` even though it exited.
    pub fn result(&self) -> Option<EmbeddedResult> {
        *self.state.result.lock().unwrap()
    }

    /// Finish the host-to-JS stream after delivering its queued events. JS
    /// receives stdin end/close and can still submit its final commits. Timers
    /// and other live work retain their normal lifetime; `shutdown` interrupts
    /// the VM when the host needs unconditional termination.
    pub fn close_input(&self) {
        let mut events = self.state.events.lock().unwrap();
        if !events.closed {
            events.closed = true;
            drop(events);
            self.state.wake(INPUT_READY);
        }
    }

    pub fn recv_commit_timeout(&self, timeout: Duration) -> Result<CommitPoll, ProtocolError> {
        self.receive(Some(timeout))
    }

    fn receive(&self, timeout: Option<Duration>) -> Result<CommitPoll, ProtocolError> {
        let commits = self.state.commits.lock().unwrap();
        let empty = |queue: &mut Frames| queue.frames.is_empty() && !queue.closed;
        let mut commits = match timeout {
            Some(timeout) => {
                self.state
                    .commit_ready
                    .wait_timeout_while(commits, timeout, empty)
                    .unwrap()
                    .0
            }
            None => self.state.commit_ready.wait_while(commits, empty).unwrap(),
        };
        if let Some(commit) = commits.pop() {
            let drain = commits.take_drain();
            drop(commits);
            if drain {
                self.state.wake(OUTPUT_DRAIN);
            }
            return Ok(CommitPoll::Commit(commit));
        }
        if !commits.closed {
            return Ok(CommitPoll::Timeout);
        }
        drop(commits);
        if !self.state.shutdown_requested.load(Ordering::Acquire)
            && let Some(status) = self.runtime_status().filter(|status| *status != 0)
        {
            return Err(ProtocolError::Io(std::io::Error::other(
                match graph_failure(status) {
                    Some(reason) => {
                        format!("embedded Bun packaged session could not start: {reason}")
                    }
                    None => format!("embedded Bun runtime exited with status {status}"),
                },
            )));
        }
        Ok(CommitPoll::Ended)
    }
}

impl RuntimeAdapter for EmbeddedBunAdapter {
    fn recv_commit(&self) -> Result<Option<Vec<u8>>, ProtocolError> {
        match self.receive(None)? {
            CommitPoll::Commit(commit) => Ok(Some(commit)),
            CommitPoll::Ended => Ok(None),
            CommitPoll::Timeout => unreachable!("unbounded receive cannot time out"),
        }
    }

    fn send_event(&self, event: Event) -> Result<(), ProtocolError> {
        let payload = event.encode()?;
        if payload.len() > MAX_FRAME_LENGTH {
            return Err(ProtocolError::FrameTooLarge(payload.len()));
        }
        let mut frame = Vec::with_capacity(4 + payload.len());
        frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        frame.extend_from_slice(&payload);
        let mut events = self.state.events.lock().unwrap();
        if events.closed {
            return Err(ProtocolError::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "embedded event queue closed",
            )));
        }
        // The UI thread never waits for the JS consumer. Exhaustion is an
        // explicit transport error, with no silently dropped or reordered input.
        if events.frames.len() >= EVENT_CAPACITY
            || frame.len() > (MAX_FRAME_LENGTH + 4).saturating_sub(events.bytes)
        {
            return Err(ProtocolError::Io(std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                "embedded event queue capacity exceeded",
            )));
        }
        self.tap.record_outbound_frame(&frame);
        events.bytes += frame.len();
        events.frames.push_back(frame);
        drop(events);
        self.state.wake(INPUT_READY);
        Ok(())
    }

    fn request_shutdown(&self) -> Result<(), ProtocolError> {
        self.state.request_shutdown();
        Ok(())
    }

    /// Waits for VM teardown. Hosts must call this on a background executor.
    fn shutdown(&self) -> Result<(), ProtocolError> {
        self.request_shutdown()?;
        #[cfg(feature = "embedded-bun")]
        {
            drop(
                self.state
                    .terminated
                    .wait_while(self.state.runtime_status.lock().unwrap(), |status| {
                        status.is_none()
                    })
                    .unwrap(),
            );
        }
        Ok(())
    }

    fn status(&self) -> RuntimeStatus {
        if self.state.shutdown_requested.load(Ordering::Acquire) {
            RuntimeStatus::Shutdown
        } else if let Some(status) = self.runtime_status() {
            // A negative status is a packaged session that never started, not a
            // VM exit code: report it as a failure rather than an exit.
            if status < 0 {
                RuntimeStatus::Failed
            } else {
                RuntimeStatus::Exited {
                    code: Some(status),
                    signal: None,
                }
            }
        } else {
            RuntimeStatus::Running
        }
    }

    fn tap_inbound_payload(&self, payload: &[u8]) {
        self.tap.record_inbound_payload(payload);
    }
}

impl Drop for EmbeddedBunAdapter {
    fn drop(&mut self) {
        // The engine retains State through teardown. Dropping a UI owner never
        // joins a native runtime thread or invalidates live callback pointers.
        self.state.request_shutdown();
    }
}

#[cfg(feature = "embedded-bun")]
mod engine {
    use super::*;
    use solid_gpui_bun_sys::*;
    use std::ffi::c_void;
    use std::sync::{OnceLock, mpsc};

    pub(super) struct Control(*mut c_void);
    // This opaque handle contains only Bun's thread-safe VM gate and wake state.
    // It never exposes a VM or JS value to the caller. State retains it until
    // both the synchronous run call and all host operations have returned.
    unsafe impl Send for Control {}
    unsafe impl Sync for Control {}
    impl Control {
        pub(super) fn new() -> Self {
            Self(unsafe { bun_embedded_create() })
        }
        pub(super) fn wake(&self, flags: u32) {
            unsafe { bun_embedded_wake(self.0, flags) }
        }
        pub(super) fn terminate(&self) {
            unsafe { bun_embedded_terminate(self.0) }
        }
    }
    impl Drop for Control {
        fn drop(&mut self) {
            unsafe { bun_embedded_destroy(self.0) }
        }
    }

    /// A session's entry in the ABI's byte form, plus the state the callbacks
    /// borrow until the VM is torn down.
    type Session = (Vec<u8>, Arc<State>);
    pub(super) struct Engine {
        pub(super) sender: mpsc::Sender<Session>,
        pub(super) active: Arc<AtomicBool>,
    }

    pub(super) fn owner() -> Result<&'static Engine, EmbeddedBunError> {
        static ENGINE: OnceLock<Result<Engine, std::io::Error>> = OnceLock::new();
        ENGINE
            .get_or_init(|| {
                let (sender, receiver) = mpsc::channel::<Session>();
                let active = Arc::new(AtomicBool::new(false));
                let worker_active = Arc::clone(&active);
                std::thread::Builder::new()
                    .name("solid-gpui-bun".into())
                    .stack_size(18 * 1024 * 1024)
                    .spawn(move || {
                        for (entry, state) in receiver {
                            let callbacks = BunIoCallbacks {
                                context: Arc::as_ptr(&state) as *mut c_void,
                                write_commit,
                                read_event,
                            };
                            // State, entry and callbacks outlive the synchronous call,
                            // which tears down the VM before releasing borrowed IO.
                            // A negative status is a packaged graph failure; the
                            // runtime reported it before creating a VM.
                            let exit_status = unsafe {
                                bun_embedded_run(
                                    state.control.0,
                                    entry.as_ptr(),
                                    entry.len(),
                                    &callbacks,
                                )
                            };
                            state.events.lock().unwrap().closed = true;
                            // Read the application's declared completion while the
                            // control allocation is still owned by this session, and
                            // publish it alongside the status so a host that observed
                            // either sees both.
                            let mut declared = BunEmbeddedResult::default();
                            let declared_ok =
                                unsafe { bun_embedded_result(state.control.0, &mut declared) } == 0;
                            let mut result = state.result.lock().unwrap();
                            *result = if declared_ok {
                                declared_result(declared)
                            } else {
                                None
                            };
                            drop(result);
                            let mut status = state.runtime_status.lock().unwrap();
                            *status = Some(exit_status);
                            drop(status);
                            worker_active.store(false, Ordering::Release);
                            state.commits.lock().unwrap().closed = true;
                            state.commit_ready.notify_all();
                            state.terminated.notify_all();
                        }
                    })?;
                Ok(Engine { sender, active })
            })
            .as_ref()
            .map_err(|_| EmbeddedBunError::ThreadStart)
    }

    unsafe extern "C" fn write_commit(context: *mut c_void, ptr: *const u8, len: usize) -> i32 {
        if context.is_null() || ptr.is_null() || len < 4 || len - 4 > MAX_FRAME_LENGTH {
            return -1;
        }
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
        if u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize != len - 4 {
            return -1;
        }
        let state = unsafe { &*context.cast::<State>() };
        let Ok(mut commits) = state.commits.lock() else {
            return -1;
        };
        let result = commits.accept_commit(&bytes[4..]);
        if result >= 0 {
            state.commits_seen.fetch_add(1, Ordering::Relaxed);
            state.commit_ready.notify_one();
        }
        result
    }

    unsafe extern "C" fn read_event(context: *mut c_void, ptr: *mut u8, cap: usize) -> usize {
        if context.is_null() || ptr.is_null() {
            return usize::MAX;
        }
        let state = unsafe { &*context.cast::<State>() };
        let Ok(mut events) = state.events.lock() else {
            return usize::MAX;
        };
        let Some(frame) = events.frames.front() else {
            return if events.closed { usize::MAX } else { 0 };
        };
        if frame.len() > cap {
            return usize::MAX;
        }
        let frame = events.pop().unwrap();
        unsafe { std::ptr::copy_nonoverlapping(frame.as_ptr(), ptr, frame.len()) };
        frame.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The declared completion is a full 32-bit value and is present only when
    /// the application declared it. Both halves matter: a truncating host view
    /// would report 199 for a Windows UAC cancellation (1223), and a
    /// presence-by-nonzero rule would lose an application that completes with 0.
    #[cfg(feature = "embedded-bun")]
    #[test]
    fn declared_completion_preserves_full_u32_and_presence() {
        assert_eq!(
            declared_result(solid_gpui_bun_sys::BunEmbeddedResult {
                present: 1,
                code: 1223,
            }),
            Some(EmbeddedResult { code: 1223 })
        );
        assert_eq!(
            declared_result(solid_gpui_bun_sys::BunEmbeddedResult {
                present: 1,
                code: 0,
            }),
            Some(EmbeddedResult { code: 0 })
        );
        assert_eq!(
            declared_result(solid_gpui_bun_sys::BunEmbeddedResult {
                present: 1,
                code: u32::MAX,
            }),
            Some(EmbeddedResult { code: u32::MAX })
        );
        assert_eq!(
            declared_result(solid_gpui_bun_sys::BunEmbeddedResult {
                present: 0,
                code: 1223,
            }),
            None
        );
    }

    #[test]
    fn commit_backpressure_accepts_last_frame_and_drains_once() {
        let mut commits = Frames::default();
        for n in 0..COMMIT_HIGH_WATER {
            assert_eq!(
                commits.accept_commit(&[n as u8]),
                i32::from(n + 1 < COMMIT_HIGH_WATER)
            );
        }
        assert_eq!(commits.pop().unwrap(), vec![0]);
        assert!(commits.take_drain());
        assert!(!commits.take_drain());
        for n in 1..COMMIT_HIGH_WATER {
            assert_eq!(commits.pop().unwrap(), vec![n as u8]);
        }
        assert_eq!(commits.bytes, 0);
        commits.closed = true;
        assert_eq!(commits.accept_commit(&[1]), -1);
    }
}
