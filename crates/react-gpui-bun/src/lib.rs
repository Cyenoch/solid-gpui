//! Embedded Bun runtime adapter for React GPUI.
//!
//! The Bun VM is owned by one dedicated thread. GPUI never crosses the
//! boundary: only framed, immutable byte batches move through bounded
//! channels and plain C callbacks. The optional `embedded-bun` feature links
//! the pinned Bun source build prepared by `build.rs`.

#[cfg(feature = "embedded-bun")]
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "embedded-bun")]
use std::sync::mpsc::{self, TryRecvError, TrySendError};
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Arc, Mutex};
#[cfg(feature = "embedded-bun")]
use std::thread::{self, JoinHandle};
use std::time::Duration;

use react_gpui::{Event, MAX_FRAME_LENGTH, ProtocolError, RuntimeAdapter};
use thiserror::Error;

#[cfg(feature = "embedded-bun")]
const CHANNEL_CAPACITY: usize = 32;

#[cfg(feature = "embedded-bun")]
unsafe extern "C" {
    fn bun_embedded_eval_with_io(entry: *const u8, len: usize, io: *const BunIoCallbacks) -> i32;
}

#[cfg(feature = "embedded-bun")]
#[repr(C)]
struct BunIoCallbacks {
    context: *mut c_void,
    write_commit: unsafe extern "C" fn(*mut c_void, *const u8, usize) -> bool,
    read_event: unsafe extern "C" fn(*mut c_void, *mut u8, usize) -> usize,
    read_refresh: unsafe extern "C" fn(*mut c_void, *mut u8, usize) -> usize,
}

struct State {
    commits: Mutex<Receiver<Vec<u8>>>,
    #[cfg(feature = "embedded-bun")]
    commits_out: Mutex<Option<SyncSender<Vec<u8>>>>,
    events: Mutex<Option<SyncSender<Vec<u8>>>>,
    #[cfg(feature = "embedded-bun")]
    event_rx: Mutex<Receiver<Vec<u8>>>,
    #[cfg(feature = "embedded-bun")]
    refresh: Mutex<Option<SyncSender<Vec<u8>>>>,
    #[cfg(feature = "embedded-bun")]
    refresh_rx: Mutex<Receiver<Vec<u8>>>,
    closed: AtomicBool,
    runtime_status: Mutex<Option<i32>>,
    #[cfg(feature = "embedded-bun")]
    commits_seen: std::sync::atomic::AtomicUsize,
    #[cfg(feature = "embedded-bun")]
    refreshes_queued: std::sync::atomic::AtomicUsize,
    #[cfg(feature = "embedded-bun")]
    runtime: Mutex<Option<JoinHandle<()>>>,
    #[cfg(feature = "embedded-bun")]
    watcher: Mutex<Option<JoinHandle<()>>>,
}

#[cfg(feature = "embedded-bun")]
struct CallbackContext {
    state: Arc<State>,
}

/// Errors returned while creating or driving the embedded runtime.
#[derive(Debug, Error)]
pub enum EmbeddedBunError {
    #[error("embedded Bun support is disabled; enable the `embedded-bun` feature")]
    FeatureDisabled,
    #[error("embedded Bun entry does not exist: {0}")]
    MissingEntry(PathBuf),
    #[error("embedded Bun runtime thread failed to start")]
    ThreadStart,
    #[error("embedded Bun adapter is closed")]
    Closed,
    #[error("embedded Bun event frame is too large: {0} bytes")]
    EventTooLarge(usize),
    #[error("embedded Bun event encoding failed: {0}")]
    Event(#[from] ProtocolError),
}

/// A concrete in-process RuntimeAdapter backed by Bun/JSC on a dedicated
/// runtime thread.
pub struct EmbeddedBunAdapter {
    state: Arc<State>,
}

impl EmbeddedBunAdapter {
    /// Start evaluating `entry` in an embedded Bun VM.
    ///
    /// The entry path is copied before the runtime thread starts. No GPUI
    /// object, Rust closure, or JavaScript value is transferred to Bun.
    pub fn start(entry: impl AsRef<Path>) -> Result<Arc<Self>, EmbeddedBunError> {
        let entry = entry.as_ref();
        let entry = std::fs::canonicalize(entry)
            .map_err(|_| EmbeddedBunError::MissingEntry(entry.to_path_buf()))?;

        #[cfg(not(feature = "embedded-bun"))]
        {
            let _ = entry;
            return Err(EmbeddedBunError::FeatureDisabled);
        }

        #[cfg(feature = "embedded-bun")]
        {
            let (commit_tx, commit_rx) = mpsc::sync_channel(CHANNEL_CAPACITY);
            let (event_tx, event_rx) = mpsc::sync_channel(CHANNEL_CAPACITY);
            let (refresh_tx, refresh_rx) = mpsc::sync_channel(CHANNEL_CAPACITY);
            let state = Arc::new(State {
                commits: Mutex::new(commit_rx),
                commits_out: Mutex::new(Some(commit_tx)),
                events: Mutex::new(Some(event_tx)),
                event_rx: Mutex::new(event_rx),
                refresh: Mutex::new(Some(refresh_tx)),
                refresh_rx: Mutex::new(refresh_rx),
                closed: AtomicBool::new(false),
                runtime_status: Mutex::new(None),
                commits_seen: std::sync::atomic::AtomicUsize::new(0),
                refreshes_queued: std::sync::atomic::AtomicUsize::new(0),
                runtime: Mutex::new(None),
                watcher: Mutex::new(None),
            });
            let state_for_thread = Arc::clone(&state);

            let handle = thread::Builder::new()
                .name("react-gpui-bun".to_owned())
                .spawn(move || {
                    let context = Box::new(CallbackContext {
                        state: Arc::clone(&state_for_thread),
                    });
                    let context_ptr = (&*context) as *const CallbackContext as *mut c_void;
                    let callbacks = Box::new(BunIoCallbacks {
                        context: context_ptr,
                        write_commit,
                        read_event,
                        read_refresh,
                    });
                    let callbacks_ptr = (&*callbacks) as *const BunIoCallbacks;
                    // Both boxes stay on this thread until Bun has torn down
                    // its VM and released all callback references.
                    let _context = context;
                    let _callbacks = callbacks;
                    let bytes = entry.to_string_lossy().as_bytes().to_vec();
                    // SAFETY: the callback table and entry bytes stay alive for
                    // the complete synchronous FFI call; Bun calls back only
                    // on this thread.
                    let status = unsafe {
                        bun_embedded_eval_with_io(bytes.as_ptr(), bytes.len(), callbacks_ptr)
                    };
                    if let Ok(mut runtime_status) = state_for_thread.runtime_status.lock() {
                        *runtime_status = Some(status);
                    }
                    state_for_thread.closed.store(true, Ordering::Release);
                    state_for_thread
                        .events
                        .lock()
                        .ok()
                        .and_then(|mut guard| guard.take());
                    state_for_thread
                        .commits_out
                        .lock()
                        .ok()
                        .and_then(|mut guard| guard.take());
                    state_for_thread
                        .refresh
                        .lock()
                        .ok()
                        .and_then(|mut guard| guard.take());
                })
                .map_err(|_| EmbeddedBunError::ThreadStart)?;

            *state
                .runtime
                .lock()
                .map_err(|_| EmbeddedBunError::ThreadStart)? = Some(handle);

            Ok(Arc::new(Self { state }))
        }
    }

    /// Number of complete protocol frames accepted from Bun. Intended for
    /// host smoke diagnostics; it does not affect RuntimeAdapter semantics.
    #[cfg(feature = "embedded-bun")]
    pub fn commit_count(&self) -> usize {
        self.state.commits_seen.load(Ordering::Acquire)
    }

    /// Exit status returned by Bun when its runtime thread has completed.
    #[cfg(feature = "embedded-bun")]
    pub fn runtime_status(&self) -> Option<i32> {
        self.state
            .runtime_status
            .lock()
            .ok()
            .and_then(|status| *status)
    }

    #[cfg(feature = "embedded-bun")]
    pub fn refresh_count(&self) -> usize {
        self.state.refreshes_queued.load(Ordering::Acquire)
    }

    /// Bounded wait variant used by startup/smoke callers so a broken Bun
    /// entrypoint cannot leave the host waiting forever.
    pub fn recv_commit_timeout(&self, timeout: Duration) -> Result<Option<Vec<u8>>, ProtocolError> {
        let receiver = self.state.commits.lock().map_err(|_| {
            ProtocolError::Io(std::io::Error::other("embedded commit lock poisoned"))
        })?;
        match receiver.recv_timeout(timeout) {
            Ok(commit) => Ok(Some(commit)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Ok(None),
        }
    }

    /// Queue a source module for Fast Refresh in the existing Bun VM.
    #[cfg(feature = "embedded-bun")]
    pub fn refresh_entry(&self, entry: impl AsRef<Path>) -> Result<(), EmbeddedBunError> {
        if self.state.closed.load(Ordering::Acquire) {
            return Err(EmbeddedBunError::Closed);
        }
        let bytes = entry.as_ref().to_string_lossy().as_bytes().to_vec();
        let sender = self
            .state
            .refresh
            .lock()
            .map_err(|_| EmbeddedBunError::Closed)?
            .as_ref()
            .cloned()
            .ok_or(EmbeddedBunError::Closed)?;
        sender
            .try_send(bytes)
            .map_err(|_| EmbeddedBunError::Closed)?;
        self.state.refreshes_queued.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }

    /// Watch a source file and queue cache-busting imports when it changes.
    /// The watcher is a Rust thread; the JS module/evaluation remains on Bun's
    /// dedicated runtime thread.
    #[cfg(feature = "embedded-bun")]
    pub fn watch(self: &Arc<Self>, entry: impl AsRef<Path>) -> Result<(), EmbeddedBunError> {
        let entry = std::fs::canonicalize(entry.as_ref())
            .map_err(|_| EmbeddedBunError::MissingEntry(entry.as_ref().to_path_buf()))?;
        let adapter = Arc::clone(self);
        let handle = thread::Builder::new()
            .name("react-gpui-bun-refresh".to_owned())
            .spawn(move || {
                let mut last = std::fs::read(&entry).ok();
                while !adapter.state.closed.load(Ordering::Acquire) {
                    thread::sleep(Duration::from_millis(100));
                    let current = std::fs::read(&entry).ok();
                    if current.is_some() && current != last {
                        last = current;
                        if let Err(error) = adapter.refresh_entry(&entry) {
                            eprintln!("react-gpui-bun: refresh queue failed: {error}");
                            break;
                        }
                    }
                }
            })
            .map_err(|_| EmbeddedBunError::ThreadStart)?;
        *self
            .state
            .watcher
            .lock()
            .map_err(|_| EmbeddedBunError::ThreadStart)? = Some(handle);
        Ok(())
    }

    fn join_runtime(&self) {
        self.state.closed.store(true, Ordering::Release);
        #[cfg(feature = "embedded-bun")]
        {
            if let Ok(mut watcher) = self.state.watcher.lock() {
                if let Some(handle) = watcher.take() {
                    let _ = handle.join();
                }
            }
            if let Ok(mut events) = self.state.events.lock() {
                events.take();
            }
            if let Ok(mut refresh) = self.state.refresh.lock() {
                refresh.take();
            }
            if let Ok(mut runtime) = self.state.runtime.lock() {
                if let Some(handle) = runtime.take() {
                    let _ = handle.join();
                }
            }
        }
    }
}

impl RuntimeAdapter for EmbeddedBunAdapter {
    fn recv_commit(&self) -> Result<Option<Vec<u8>>, ProtocolError> {
        let receiver = self.state.commits.lock().map_err(|_| {
            ProtocolError::Io(std::io::Error::other("embedded commit lock poisoned"))
        })?;
        match receiver.recv() {
            Ok(commit) => Ok(Some(commit)),
            Err(_) => {
                let status = self
                    .state
                    .runtime_status
                    .lock()
                    .ok()
                    .and_then(|status| *status);
                if let Some(status) = status.filter(|status| *status != 0) {
                    return Err(ProtocolError::Io(std::io::Error::other(format!(
                        "embedded Bun runtime exited with status {status}"
                    ))));
                }
                Ok(None)
            }
        }
    }

    fn send_event(&self, event: &Event) -> Result<(), ProtocolError> {
        if self.state.closed.load(Ordering::Acquire) {
            return Err(ProtocolError::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "embedded Bun adapter is closed",
            )));
        }
        let payload = event.encode()?;
        if payload.len() > MAX_FRAME_LENGTH {
            return Err(ProtocolError::FrameTooLarge(payload.len()));
        }
        let mut frame = Vec::with_capacity(4 + payload.len());
        frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        frame.extend_from_slice(&payload);
        let sender = self
            .state
            .events
            .lock()
            .map_err(|_| ProtocolError::Io(std::io::Error::other("embedded event lock poisoned")))?
            .as_ref()
            .cloned()
            .ok_or_else(|| {
                ProtocolError::Io(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "embedded event queue closed",
                ))
            })?;
        sender.send(frame).map_err(|_| {
            ProtocolError::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "embedded event queue closed",
            ))
        })
    }

    fn shutdown(&self) -> Result<(), ProtocolError> {
        self.join_runtime();
        Ok(())
    }
}

impl Drop for EmbeddedBunAdapter {
    fn drop(&mut self) {
        self.join_runtime();
    }
}

#[cfg(feature = "embedded-bun")]
unsafe extern "C" fn write_commit(context: *mut c_void, ptr: *const u8, len: usize) -> bool {
    if context.is_null() || ptr.is_null() || len < 4 || len - 4 > MAX_FRAME_LENGTH {
        return false;
    }
    // SAFETY: Bun owns the immutable frame for this synchronous callback.
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let payload_len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    if payload_len != len - 4 {
        return false;
    }
    // SAFETY: context points to the callback context held by the runtime
    // thread for the complete Bun call.
    let context = unsafe { &*(context as *const CallbackContext) };
    let sender = match context.state.state_commit_sender() {
        Some(sender) => sender,
        None => return false,
    };
    match sender.try_send(bytes[4..].to_vec()) {
        Ok(()) => {
            context.state.commits_seen.fetch_add(1, Ordering::AcqRel);
            true
        }
        Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => false,
    }
}

#[cfg(feature = "embedded-bun")]
unsafe extern "C" fn read_event(context: *mut c_void, ptr: *mut u8, cap: usize) -> usize {
    if context.is_null() || ptr.is_null() {
        return usize::MAX;
    }
    // SAFETY: context points to the callback context held by the runtime
    // thread for the complete Bun call.
    let context = unsafe { &*(context as *const CallbackContext) };
    let receiver = match context.state.event_rx.lock() {
        Ok(receiver) => receiver,
        Err(_) => return usize::MAX,
    };
    match receiver.try_recv() {
        Ok(frame) if frame.len() <= cap => {
            // SAFETY: Bun provided `cap` writable bytes.
            unsafe { std::ptr::copy_nonoverlapping(frame.as_ptr(), ptr, frame.len()) };
            frame.len()
        }
        Ok(_) => usize::MAX,
        Err(TryRecvError::Empty) => {
            if context.state.closed.load(Ordering::Acquire) {
                usize::MAX
            } else {
                0
            }
        }
        Err(TryRecvError::Disconnected) => usize::MAX,
    }
}

#[cfg(feature = "embedded-bun")]
unsafe extern "C" fn read_refresh(context: *mut c_void, ptr: *mut u8, cap: usize) -> usize {
    if context.is_null() || ptr.is_null() {
        return usize::MAX;
    }
    // SAFETY: context points to the callback context held by the runtime
    // thread for the complete Bun call.
    let context = unsafe { &*(context as *const CallbackContext) };
    let receiver = match context.state.refresh_rx.lock() {
        Ok(receiver) => receiver,
        Err(_) => return usize::MAX,
    };
    match receiver.try_recv() {
        Ok(path) if path.len() <= cap => {
            // SAFETY: Bun provided `cap` writable bytes.
            unsafe { std::ptr::copy_nonoverlapping(path.as_ptr(), ptr, path.len()) };
            path.len()
        }
        Ok(_) => usize::MAX,
        Err(TryRecvError::Empty) => {
            if context.state.closed.load(Ordering::Acquire) {
                usize::MAX
            } else {
                0
            }
        }
        Err(TryRecvError::Disconnected) => usize::MAX,
    }
}

#[cfg(feature = "embedded-bun")]
impl State {
    fn state_commit_sender(&self) -> Option<SyncSender<Vec<u8>>> {
        self.commits_out.lock().ok()?.as_ref().cloned()
    }
}
