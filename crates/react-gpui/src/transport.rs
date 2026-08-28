use std::collections::VecDeque;
use std::io;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
#[cfg(test)]
use std::time::{Duration, Instant};

use crate::protocol::{Event, MAX_FRAME_LENGTH, ProtocolError, read_frame, write_frame};
pub use crate::protocol_tap::ProtocolTap;

/// Build the process-local protocol tap from `REACT_GPUI_TAP`.
pub fn protocol_tap_from_env() -> ProtocolTap {
    ProtocolTap::from_env()
}

/// The terminal state observed at the runtime adapter boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeStatus {
    Running,
    Failed,
    Shutdown,
    Exited {
        code: Option<i32>,
        signal: Option<i32>,
    },
}

impl RuntimeStatus {
    pub const fn is_failure(self) -> bool {
        !matches!(self, Self::Shutdown)
    }
}

impl std::fmt::Display for RuntimeStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => formatter.write_str("still running after its commit stream closed"),
            Self::Failed => formatter.write_str("event writer failed"),
            Self::Shutdown => formatter.write_str("shut down by the host"),
            Self::Exited { code, signal } => match (code, signal) {
                (Some(code), _) => write!(formatter, "exited with status {code}"),
                (None, Some(signal)) => write!(formatter, "terminated by signal {signal}"),
                (None, None) => formatter.write_str("exited without a status"),
            },
        }
    }
}

/// The renderer transport seam. The renderer tree never depends on whether
/// bytes came from Bun, an embedded runtime, or an in-memory test peer.
/// A child process or embedded runtime can close its stream during explicit
/// host shutdown. Callers use this status to distinguish that path from an
/// unexpected runtime termination.
pub trait RuntimeAdapter: Send + Sync {
    fn recv_commit(&self) -> Result<Option<Vec<u8>>, ProtocolError>;
    fn send_event(&self, event: &Event) -> Result<(), ProtocolError>;
    fn shutdown(&self) -> Result<(), ProtocolError>;
    fn status(&self) -> RuntimeStatus;

    /// Observe an inbound payload at the commit-reader boundary. Adapters
    /// that do not have a tap (for example, the in-memory test adapter) keep
    /// the default no-op.
    fn tap_inbound_payload(&self, _payload: &[u8]) {}
}

/// Stop the runtime and terminate the host after a fatal runtime-boundary
/// failure. This is the single fatal path shared by inbound and outbound
/// failures; it never touches GPUI state.
pub fn fatal_runtime_failure(
    runtime: &dyn RuntimeAdapter,
    context: &str,
    detail: impl std::fmt::Display,
) -> ! {
    eprintln!("react-gpui-host: {context}: {detail}");
    let _ = runtime.shutdown();
    std::process::exit(1);
}

pub(crate) fn format_event_metadata(event: &Event) -> String {
    format!(
        "event_type={}, surface_id={}, epoch={}, revision={}, sequence={}, node_id={}, listener_id={}",
        event.event_type,
        event.surface_id,
        event.epoch,
        event.revision,
        event.sequence,
        event.node_id,
        event.listener_id,
    )
}

/// Send one native event and fail the host on an unexpected transport error.
///
/// The only non-fatal send failure is an explicit host shutdown: the runtime
/// has already been stopped, so the foreground caller must simply stop work.
/// All other failures carry the event context to stderr, stop the runtime, and
/// terminate the host with a nonzero status.
pub fn send_event_or_exit(runtime: &dyn RuntimeAdapter, context: &str, event: &Event) -> bool {
    match runtime.send_event(event) {
        Ok(()) => true,
        Err(_error) if !runtime.status().is_failure() => false,
        Err(error) => {
            let context = format!(
                "failed to send {context} ({})",
                format_event_metadata(event)
            );
            fatal_runtime_failure(runtime, &context, error)
        }
    }
}

pub(crate) const PROCESS_EVENT_QUEUE_CAPACITY: usize = 32;
pub(crate) const PROCESS_EVENT_QUEUE_MAX_BYTES: usize = MAX_FRAME_LENGTH;

#[derive(Debug, Default)]
struct EventQueueState {
    payloads: VecDeque<Vec<u8>>,
    bytes: usize,
    closed: bool,
    failure: Option<String>,
}

struct EventQueue {
    state: Mutex<EventQueueState>,
    changed: Condvar,
}

impl EventQueue {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(EventQueueState::default()),
            changed: Condvar::new(),
        })
    }

    fn try_push(&self, payload: Vec<u8>) -> Result<(), ProtocolError> {
        let payload_len = payload.len();
        let mut state = self.state.lock().map_err(|_| {
            ProtocolError::Io(io::Error::other("renderer event queue lock poisoned"))
        })?;
        if let Some(failure) = state.failure.as_ref() {
            return Err(ProtocolError::Io(io::Error::new(
                io::ErrorKind::BrokenPipe,
                format!("renderer event writer failed: {failure}"),
            )));
        }
        if state.closed {
            return Err(ProtocolError::Io(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "renderer event writer is closed",
            )));
        }
        if state.payloads.len() >= PROCESS_EVENT_QUEUE_CAPACITY {
            return Err(ProtocolError::Io(io::Error::new(
                io::ErrorKind::WouldBlock,
                format!(
                    "renderer event writer queue frame capacity reached: {} frames",
                    PROCESS_EVENT_QUEUE_CAPACITY
                ),
            )));
        }
        if payload_len > PROCESS_EVENT_QUEUE_MAX_BYTES.saturating_sub(state.bytes) {
            return Err(ProtocolError::Io(io::Error::new(
                io::ErrorKind::WouldBlock,
                format!(
                    "renderer event writer queue byte capacity reached: {} bytes",
                    PROCESS_EVENT_QUEUE_MAX_BYTES
                ),
            )));
        }
        state.bytes += payload_len;
        state.payloads.push_back(payload);
        self.changed.notify_one();
        Ok(())
    }

    fn pop(&self) -> Result<Option<Vec<u8>>, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "renderer event queue lock poisoned".to_owned())?;
        loop {
            if let Some(payload) = state.payloads.pop_front() {
                state.bytes -= payload.len();
                self.changed.notify_all();
                return Ok(Some(payload));
            }
            if state.closed {
                return Ok(None);
            }
            state = self
                .changed
                .wait(state)
                .map_err(|_| "renderer event queue lock poisoned".to_owned())?;
        }
    }

    fn fail(&self, failure: String) {
        if let Ok(mut state) = self.state.lock() {
            state.failure = Some(failure);
            state.closed = true;
            state.payloads.clear();
            state.bytes = 0;
            self.changed.notify_all();
        }
    }

    fn close(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.closed = true;
            state.payloads.clear();
            state.bytes = 0;
            self.changed.notify_all();
        }
    }

    fn failure(&self) -> Option<String> {
        self.state
            .lock()
            .ok()
            .and_then(|state| state.failure.clone())
    }
}

struct EventWriter {
    queue: Arc<EventQueue>,
    join: Mutex<Option<JoinHandle<()>>>,
    #[cfg(test)]
    completion: Arc<(Mutex<bool>, Condvar)>,
}

impl EventWriter {
    fn spawn(
        mut writer: ChildStdin,
        on_failure: Arc<dyn Fn() + Send + Sync + 'static>,
        tap: ProtocolTap,
    ) -> io::Result<Self> {
        let queue = EventQueue::new();
        let queue_for_thread = Arc::clone(&queue);
        let on_failure_for_thread = Arc::clone(&on_failure);
        #[cfg(test)]
        let completion = Arc::new((Mutex::new(false), Condvar::new()));
        #[cfg(test)]
        let completion_for_thread = Arc::clone(&completion);
        let join = thread::Builder::new()
            .name("react-gpui-event-writer".to_owned())
            .spawn(move || {
                loop {
                    let payload = match queue_for_thread.pop() {
                        Ok(Some(payload)) => payload,
                        Ok(None) => break,
                        Err(error) => {
                            queue_for_thread.fail(error);
                            on_failure_for_thread();
                            #[cfg(test)]
                            mark_writer_failure_complete(&completion_for_thread);
                            break;
                        }
                    };
                    if let Err(error) = write_frame(&mut writer, &payload) {
                        queue_for_thread.fail(error.to_string());
                        on_failure_for_thread();
                        #[cfg(test)]
                        mark_writer_failure_complete(&completion_for_thread);
                        break;
                    }
                    tap.record_outbound_payload(&payload);
                }
            })?;
        Ok(Self {
            queue,
            join: Mutex::new(Some(join)),
            #[cfg(test)]
            completion,
        })
    }

    fn send(&self, payload: Vec<u8>) -> Result<(), ProtocolError> {
        self.queue.try_push(payload)
    }

    fn failure(&self) -> Option<String> {
        self.queue.failure()
    }

    #[cfg(test)]
    fn wait_for_failure(&self, timeout: Duration) -> Option<String> {
        let (lock, changed) = &*self.completion;
        let mut complete = lock.lock().ok()?;
        let started = Instant::now();
        while !*complete {
            let remaining = timeout.checked_sub(started.elapsed())?;
            if remaining.is_zero() {
                return None;
            }
            let (next, result) = changed.wait_timeout(complete, remaining).ok()?;
            complete = next;
            if result.timed_out() && !*complete {
                return None;
            }
        }
        self.failure()
    }

    fn close(&self) {
        self.queue.close();
    }

    fn join(&self) {
        if let Ok(mut join) = self.join.lock()
            && let Some(join) = join.take()
        {
            let _ = join.join();
        }
    }
}

#[cfg(test)]
fn mark_writer_failure_complete(completion: &Arc<(Mutex<bool>, Condvar)>) {
    let (lock, changed) = &**completion;
    if let Ok(mut complete) = lock.lock() {
        *complete = true;
        changed.notify_all();
    }
}

fn stop_child(
    mut try_wait: impl FnMut() -> io::Result<Option<()>>,
    mut kill: impl FnMut() -> io::Result<()>,
    mut wait: impl FnMut() -> io::Result<()>,
) -> (Result<(), ProtocolError>, bool) {
    match try_wait() {
        Ok(Some(())) => (Ok(()), true),
        Ok(None) => match kill() {
            Ok(()) => match wait() {
                Ok(()) => (Ok(()), true),
                Err(error) => (Err(ProtocolError::Io(error)), false),
            },
            Err(kill_error) => {
                let kill_error = kill_error.to_string();
                match try_wait() {
                    Ok(Some(())) => (Ok(()), true),
                    Ok(None) => (
                        Err(ProtocolError::Io(io::Error::other(format!(
                            "renderer child kill failed: {kill_error}"
                        )))),
                        false,
                    ),
                    Err(status_error) => (
                        Err(ProtocolError::Io(io::Error::other(format!(
                            "renderer child kill failed: {kill_error}; status check failed: {status_error}"
                        )))),
                        false,
                    ),
                }
            }
        },
        Err(error) => (Err(ProtocolError::Io(error)), false),
    }
}

struct ProcessChildState {
    child: Mutex<Option<Child>>,
    shutdown_requested: AtomicBool,
}

impl ProcessChildState {
    fn new(child: Child) -> Arc<Self> {
        Arc::new(Self {
            child: Mutex::new(Some(child)),
            shutdown_requested: AtomicBool::new(false),
        })
    }

    fn try_wait(&self) -> io::Result<Option<std::process::ExitStatus>> {
        let mut child = self
            .child
            .lock()
            .map_err(|_| io::Error::other("renderer child lock poisoned"))?;
        let Some(child) = child.as_mut() else {
            return Ok(None);
        };
        child.try_wait()
    }

    fn stop(&self, mark_shutdown: bool) -> (Result<(), ProtocolError>, bool) {
        if mark_shutdown {
            self.shutdown_requested.store(true, Ordering::Release);
        }
        match self.child.lock() {
            Err(_) => (
                Err(ProtocolError::Io(io::Error::other(
                    "renderer child lock poisoned",
                ))),
                false,
            ),
            Ok(mut child) => match child.as_mut() {
                None => (Ok(()), true),
                Some(child_process) => {
                    let child_process = std::cell::RefCell::new(child_process);
                    let (result, confirmed) = stop_child(
                        || {
                            child_process
                                .borrow_mut()
                                .try_wait()
                                .map(|status| status.map(|_| ()))
                        },
                        || child_process.borrow_mut().kill(),
                        || child_process.borrow_mut().wait().map(|_| ()),
                    );
                    if confirmed {
                        child.take();
                    }
                    (result, confirmed)
                }
            },
        }
    }

    fn stop_after_writer_failure(&self) {
        if self.shutdown_requested.load(Ordering::Acquire) {
            return;
        }
        let _ = self.stop(false);
    }

    fn status(&self) -> RuntimeStatus {
        if self.shutdown_requested.load(Ordering::Acquire) {
            return RuntimeStatus::Shutdown;
        }
        let exit = self.child.lock().ok().and_then(|mut child| {
            let child = child.as_mut()?;
            for _ in 0..10 {
                if let Some(status) = child.try_wait().ok().flatten() {
                    return Some(status);
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            None
        });
        let Some(exit) = exit else {
            return RuntimeStatus::Running;
        };
        let signal = {
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                exit.signal()
            }
            #[cfg(not(unix))]
            {
                None
            }
        };
        RuntimeStatus::Exited {
            code: exit.code(),
            signal,
        }
    }
}

/// A child-process adapter using framed stdin/stdout. Child stderr is inherited
/// by the host so renderer logs remain visible without entering the protocol.
pub struct ProcessAdapter {
    child: Arc<ProcessChildState>,
    commits: Mutex<ChildStdout>,
    events: EventWriter,
    tap: ProtocolTap,
}

impl ProcessAdapter {
    pub fn spawn(mut command: Command) -> io::Result<Arc<Self>> {
        let tap = protocol_tap_from_env();
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let commits = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("renderer stdout was not piped"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("renderer stdin was not piped"))?;
        let child = ProcessChildState::new(child);
        let child_for_writer = Arc::clone(&child);
        let on_failure: Arc<dyn Fn() + Send + Sync> =
            Arc::new(move || child_for_writer.stop_after_writer_failure());
        let events = match EventWriter::spawn(stdin, on_failure, tap.clone()) {
            Ok(events) => events,
            Err(error) => {
                let _ = child.stop(true);
                return Err(error);
            }
        };
        Ok(Arc::new(Self {
            child,
            commits: Mutex::new(commits),
            events,
            tap,
        }))
    }
    /// Wait for the writer's retained failure. The writer records this only
    /// after its child-stop callback has completed, so this is a completion
    /// signal for the writer/child shutdown path rather than a timing guess.
    #[cfg(test)]
    pub(crate) fn wait_for_event_writer_failure(&self, timeout: Duration) -> bool {
        self.events.wait_for_failure(timeout).is_some()
    }

    pub fn try_wait(&self) -> io::Result<Option<std::process::ExitStatus>> {
        self.child.try_wait()
    }

    /// Returns the writer's retained I/O failure, if it has stopped.
    pub fn event_writer_error(&self) -> Option<String> {
        self.events.failure()
    }

    pub fn status(&self) -> RuntimeStatus {
        if self.child.shutdown_requested.load(Ordering::Acquire) {
            return RuntimeStatus::Shutdown;
        }
        if self.events.failure().is_some() {
            return RuntimeStatus::Failed;
        }
        self.child.status()
    }
}

impl RuntimeAdapter for ProcessAdapter {
    fn recv_commit(&self) -> Result<Option<Vec<u8>>, ProtocolError> {
        let mut reader = self
            .commits
            .lock()
            .map_err(|_| ProtocolError::Io(io::Error::other("renderer stdout lock poisoned")))?;
        read_frame(&mut *reader)
    }

    fn send_event(&self, event: &Event) -> Result<(), ProtocolError> {
        let payload = event.encode()?;
        if payload.len() > MAX_FRAME_LENGTH {
            return Err(ProtocolError::FrameTooLarge(payload.len()));
        }
        self.events.send(payload)
    }

    fn shutdown(&self) -> Result<(), ProtocolError> {
        let (result, child_exit_confirmed) = self.child.stop(true);
        self.events.close();
        if child_exit_confirmed {
            self.events.join();
        }
        result
    }

    fn status(&self) -> RuntimeStatus {
        ProcessAdapter::status(self)
    }
    fn tap_inbound_payload(&self, payload: &[u8]) {
        self.tap.record_inbound_payload(payload);
    }
}

impl Drop for ProcessAdapter {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[derive(Debug, Default)]
struct InMemoryState {
    commits: VecDeque<Vec<u8>>,
    events: VecDeque<Vec<u8>>,
    closed: bool,
}

/// Deterministic adapter used by protocol/tree tests and future embedded Bun
/// integration. Commits are supplied by `push_commit`, and encoded events are
/// available through `take_event`.
pub struct InMemoryAdapter {
    state: Mutex<InMemoryState>,
    changed: Condvar,
}

impl InMemoryAdapter {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(InMemoryState::default()),
            changed: Condvar::new(),
        })
    }

    pub fn push_commit(&self, payload: Vec<u8>) -> Result<(), ProtocolError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| ProtocolError::Io(io::Error::other("in-memory lock poisoned")))?;
        if state.closed {
            return Err(ProtocolError::Io(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "in-memory adapter is closed",
            )));
        }
        state.commits.push_back(payload);
        self.changed.notify_one();
        Ok(())
    }

    pub fn take_event(&self) -> Result<Option<Event>, ProtocolError> {
        let payload = self
            .state
            .lock()
            .map_err(|_| ProtocolError::Io(io::Error::other("in-memory lock poisoned")))?
            .events
            .pop_front();
        payload.map(|payload| Event::decode(&payload)).transpose()
    }

    pub fn close(&self) -> Result<(), ProtocolError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| ProtocolError::Io(io::Error::other("in-memory lock poisoned")))?;
        state.closed = true;
        self.changed.notify_all();
        Ok(())
    }
}

impl RuntimeAdapter for InMemoryAdapter {
    fn recv_commit(&self) -> Result<Option<Vec<u8>>, ProtocolError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| ProtocolError::Io(io::Error::other("in-memory lock poisoned")))?;
        loop {
            if let Some(payload) = state.commits.pop_front() {
                return Ok(Some(payload));
            }
            if state.closed {
                return Ok(None);
            }
            state = self
                .changed
                .wait(state)
                .map_err(|_| ProtocolError::Io(io::Error::other("in-memory lock poisoned")))?;
        }
    }

    fn send_event(&self, event: &Event) -> Result<(), ProtocolError> {
        let payload = event.encode()?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| ProtocolError::Io(io::Error::other("in-memory lock poisoned")))?;
        if state.closed {
            return Err(ProtocolError::Io(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "in-memory adapter is closed",
            )));
        }
        state.events.push_back(payload);
        Ok(())
    }

    fn shutdown(&self) -> Result<(), ProtocolError> {
        self.close()
    }

    fn status(&self) -> RuntimeStatus {
        if self.state.lock().ok().is_some_and(|state| state.closed) {
            RuntimeStatus::Shutdown
        } else {
            RuntimeStatus::Running
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_queue_preserves_order() {
        let queue = EventQueue::new();
        queue.try_push(vec![1]).unwrap();
        queue.try_push(vec![2]).unwrap();
        queue.try_push(vec![3]).unwrap();

        assert_eq!(queue.pop().unwrap(), Some(vec![1]));
        assert_eq!(queue.pop().unwrap(), Some(vec![2]));
        assert_eq!(queue.pop().unwrap(), Some(vec![3]));
        queue.close();
        assert_eq!(queue.pop().unwrap(), None);
    }

    #[test]
    fn event_queue_enforces_frame_and_byte_limits() {
        let queue = EventQueue::new();
        for index in 0..PROCESS_EVENT_QUEUE_CAPACITY {
            queue.try_push(vec![index as u8]).unwrap();
        }
        let error = queue.try_push(vec![0]).unwrap_err().to_string();
        assert!(error.contains("frame capacity"));

        let queue = EventQueue::new();
        queue
            .try_push(vec![0; PROCESS_EVENT_QUEUE_MAX_BYTES])
            .unwrap();
        let error = queue.try_push(vec![0]).unwrap_err().to_string();
        assert!(error.contains("byte capacity"));
    }
    #[test]
    fn stop_child_does_not_join_when_kill_cannot_confirm_exit() {
        let mut status_checks = 0;
        let mut wait_calls = 0;
        let (result, confirmed) = stop_child(
            || {
                status_checks += 1;
                Ok(None)
            },
            || Err(io::Error::other("kill denied")),
            || {
                wait_calls += 1;
                Err(io::Error::other("wait must not run"))
            },
        );

        assert!(result.is_err());
        assert!(!confirmed);
        assert_eq!(status_checks, 2);
        assert_eq!(wait_calls, 0);
    }

    #[test]
    fn stop_child_does_not_join_when_wait_fails() {
        let (result, confirmed) = stop_child(
            || Ok(None),
            || Ok(()),
            || Err(io::Error::other("wait failed")),
        );

        assert!(result.is_err());
        assert!(!confirmed);
    }

    #[test]
    fn stop_child_accepts_kill_race_when_status_confirms_exit() {
        let mut status_checks = 0;
        let (result, confirmed) = stop_child(
            || {
                status_checks += 1;
                Ok((status_checks > 1).then_some(()))
            },
            || Err(io::Error::other("already exited")),
            || Err(io::Error::other("wait must not run")),
        );

        assert!(result.is_ok());
        assert!(confirmed);
        assert_eq!(status_checks, 2);
    }
}
