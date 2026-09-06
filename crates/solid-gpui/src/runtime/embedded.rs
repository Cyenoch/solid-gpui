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
}

#[derive(Debug, Eq, PartialEq)]
pub enum CommitPoll {
    Commit(Vec<u8>),
    Timeout,
    Ended,
}

pub struct EmbeddedBunAdapter {
    state: Arc<State>,
    tap: ProtocolTap,
}

impl EmbeddedBunAdapter {
    /// Start a session on the process-scoped engine thread. All application
    /// surfaces share this session. `shutdown` waits for complete VM teardown
    /// and releases admission for the next session.
    pub fn start(entry: impl AsRef<Path>) -> Result<Arc<Self>, EmbeddedBunError> {
        let entry = entry.as_ref();
        let entry = std::fs::canonicalize(entry)
            .map_err(|_| EmbeddedBunError::MissingEntry(entry.to_path_buf()))?;
        #[cfg(not(feature = "embedded-bun"))]
        {
            let _ = entry;
            Err(EmbeddedBunError::FeatureDisabled)
        }
        #[cfg(feature = "embedded-bun")]
        {
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
    }

    pub fn commit_count(&self) -> usize {
        self.state.commits_seen.load(Ordering::Acquire)
    }

    /// A status is published only after the session VM has been destroyed.
    pub fn runtime_status(&self) -> Option<i32> {
        *self.state.runtime_status.lock().unwrap()
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
            return Err(ProtocolError::Io(std::io::Error::other(format!(
                "embedded Bun runtime exited with status {status}"
            ))));
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
            RuntimeStatus::Exited {
                code: Some(status),
                signal: None,
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

    type Session = (PathBuf, Arc<State>);
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
                            let entry = entry.as_os_str().as_encoded_bytes();
                            // State, entry and callbacks outlive the synchronous call,
                            // which tears down the VM before releasing borrowed IO.
                            let status = unsafe {
                                bun_embedded_run(
                                    state.control.0,
                                    entry.as_ptr(),
                                    entry.len(),
                                    &callbacks,
                                )
                            };
                            state.events.lock().unwrap().closed = true;
                            let mut result = state.runtime_status.lock().unwrap();
                            *result = Some(status);
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
