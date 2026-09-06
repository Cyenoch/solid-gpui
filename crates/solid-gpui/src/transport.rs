use std::collections::VecDeque;
use std::io::{self, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
#[cfg(test)]
use std::time::{Duration, Instant};

use crate::protocol::{Event, MAX_FRAME_LENGTH, ProtocolError, read_frame};
pub use crate::protocol_tap::ProtocolTap;

/// Build the process-local protocol tap from `SOLID_GPUI_TAP`.
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
    fn send_event(&self, event: Event) -> Result<(), ProtocolError>;
    /// Revoke transport input and request termination without waiting for the
    /// runtime or its worker threads. Safe to call from the UI/fatal path.
    fn request_shutdown(&self) -> Result<(), ProtocolError>;
    /// Request termination and wait for resource cleanup. Call off the UI thread.
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
    eprintln!("solid-gpui-host: {context}: {detail}");
    let _ = runtime.request_shutdown();
    std::process::exit(1);
}
#[cfg(test)]
pub(crate) fn format_event_metadata(event: &Event) -> String {
    format!(
        "event_type={}, surface_id={}, epoch={}, revision={}, sequence={}, node_id={}, listener_id={}",
        u32::from(event.payload.event_kind()),
        event.meta.surface_id,
        event.meta.epoch,
        event.meta.revision,
        event.meta.sequence,
        event.meta.node_id,
        event.meta.listener_id,
    )
}

/// Send one native event and fail the host on an unexpected transport error.
///
/// The only non-fatal send failure is an explicit host shutdown: the runtime
/// has already been stopped, so the foreground caller must simply stop work.
/// All other failures carry the event context to stderr, stop the runtime, and
/// terminate the host with a nonzero status.
pub fn send_event_or_exit(runtime: &dyn RuntimeAdapter, context: &str, event: Event) -> bool {
    let metadata = (
        u32::from(event.payload.event_kind()),
        event.meta.surface_id,
        event.meta.epoch,
        event.meta.revision,
        event.meta.sequence,
        event.meta.node_id,
        event.meta.listener_id,
    );
    match runtime.send_event(event) {
        Ok(()) => true,
        Err(_error) if !runtime.status().is_failure() => false,
        Err(error) => {
            let context = format!(
                "failed to send {context} (event_type={}, surface_id={}, epoch={}, revision={}, sequence={}, node_id={}, listener_id={})",
                metadata.0, metadata.1, metadata.2, metadata.3, metadata.4, metadata.5, metadata.6
            );
            fatal_runtime_failure(runtime, &context, error)
        }
    }
}

pub(crate) const PROCESS_EVENT_QUEUE_CAPACITY: usize = 4096;
pub(crate) const PROCESS_EVENT_QUEUE_MAX_BYTES: usize = MAX_FRAME_LENGTH;

#[derive(Debug, Default)]
struct EventQueueState {
    events: VecDeque<(Event, usize)>,
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

    fn push(&self, event: Event, frame_size: usize) -> Result<(), ProtocolError> {
        if frame_size > PROCESS_EVENT_QUEUE_MAX_BYTES {
            return Err(ProtocolError::Io(io::Error::new(
                io::ErrorKind::WouldBlock,
                format!(
                    "renderer event writer queue byte capacity reached: {} bytes",
                    PROCESS_EVENT_QUEUE_MAX_BYTES
                ),
            )));
        }
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
        if state.events.len() >= PROCESS_EVENT_QUEUE_CAPACITY
            || frame_size > PROCESS_EVENT_QUEUE_MAX_BYTES.saturating_sub(state.bytes)
        {
            return Err(ProtocolError::Io(io::Error::new(
                io::ErrorKind::WouldBlock,
                "renderer event queue capacity exceeded",
            )));
        }
        state.bytes += frame_size;
        state.events.push_back((event, frame_size));
        self.changed.notify_one();
        Ok(())
    }

    fn pop(&self) -> Result<Option<(Event, usize)>, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "renderer event queue lock poisoned".to_owned())?;
        loop {
            if let Some(event) = state.events.pop_front() {
                state.bytes -= event.1;
                self.changed.notify_all();
                return Ok(Some(event));
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
            state.events.clear();
            state.bytes = 0;
            self.changed.notify_all();
        }
    }

    fn close(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.closed = true;
            state.events.clear();
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
            .name("solid-gpui-event-writer".to_owned())
            .spawn(move || {
                let mut frame = Vec::new();
                loop {
                    let (event, _frame_size) = match queue_for_thread.pop() {
                        Ok(Some(event)) => event,
                        Ok(None) => break,
                        Err(error) => {
                            queue_for_thread.fail(error);
                            on_failure_for_thread();
                            #[cfg(test)]
                            mark_writer_failure_complete(&completion_for_thread);
                            break;
                        }
                    };
                    if let Err(error) = event.encode_frame_into(&mut frame).and_then(|()| {
                        writer
                            .write_all(&frame)
                            .and_then(|()| writer.flush())
                            .map_err(ProtocolError::Io)
                    }) {
                        queue_for_thread.fail(error.to_string());
                        on_failure_for_thread();
                        #[cfg(test)]
                        mark_writer_failure_complete(&completion_for_thread);
                        break;
                    }
                    tap.record_outbound_payload(&frame[4..]);
                }
            })?;
        Ok(Self {
            queue,
            join: Mutex::new(Some(join)),
            #[cfg(test)]
            completion,
        })
    }

    fn send(&self, event: Event) -> Result<(), ProtocolError> {
        let frame_size = event.frame_size()?;
        self.queue.push(event, frame_size)
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
    child: Mutex<Child>,
    shutdown_requested: AtomicBool,
}

impl ProcessChildState {
    fn new(child: Child) -> Arc<Self> {
        Arc::new(Self {
            child: Mutex::new(child),
            shutdown_requested: AtomicBool::new(false),
        })
    }

    fn try_wait(&self) -> io::Result<Option<std::process::ExitStatus>> {
        let mut child = self
            .child
            .lock()
            .map_err(|_| io::Error::other("renderer child lock poisoned"))?;
        child.try_wait()
    }

    fn stop(&self, mark_shutdown: bool) -> (Result<(), ProtocolError>, bool) {
        if mark_shutdown {
            self.shutdown_requested.store(true, Ordering::Release);
        }
        // Child::wait must never hold the mutex needed by a foreground stop
        // request. Poll only on this cleanup thread, releasing the lock between
        // checks; another request can always reach Child::kill promptly.
        stop_child(
            || self.try_wait().map(|status| status.map(|_| ())),
            || self.kill(),
            || loop {
                if self.try_wait()?.is_some() {
                    return Ok(());
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            },
        )
    }

    fn kill(&self) -> io::Result<()> {
        let mut child = self
            .child
            .lock()
            .map_err(|_| io::Error::other("renderer child lock poisoned"))?;
        // try_wait also handles an already completed child's cached status.
        if child.try_wait()?.is_none()
            && let Err(error) = child.kill()
            && child.try_wait()?.is_none()
        {
            return Err(error);
        }
        Ok(())
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
        let exit = (|| {
            for _ in 0..10 {
                if let Some(status) = self.try_wait().ok().flatten() {
                    return Some(status);
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            None
        })();
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

    fn send_event(&self, event: Event) -> Result<(), ProtocolError> {
        self.events.send(event)
    }

    fn request_shutdown(&self) -> Result<(), ProtocolError> {
        self.child.shutdown_requested.store(true, Ordering::Release);
        self.events.close();
        self.child.kill().map_err(ProtocolError::Io)
    }

    fn shutdown(&self) -> Result<(), ProtocolError> {
        self.request_shutdown()?;
        let (result, child_exit_confirmed) = self.child.stop(true);
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

/// Deterministic adapter used by protocol/tree tests. Commits are supplied
/// by `push_commit`, and encoded events are available through `take_event`.
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

    fn send_event(&self, event: Event) -> Result<(), ProtocolError> {
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

    fn request_shutdown(&self) -> Result<(), ProtocolError> {
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

    fn event(sequence: u32) -> Event {
        Event::press(7, 3, 1, sequence, 1, 1)
    }

    #[test]
    fn event_queue_preserves_order() {
        let queue = EventQueue::new();
        queue.push(event(1), 1).unwrap();
        queue.push(event(2), 1).unwrap();
        queue.push(event(3), 1).unwrap();

        assert_eq!(
            queue.pop().unwrap().map(|(event, _)| event.meta.sequence),
            Some(1)
        );
        assert_eq!(
            queue.pop().unwrap().map(|(event, _)| event.meta.sequence),
            Some(2)
        );
        assert_eq!(
            queue.pop().unwrap().map(|(event, _)| event.meta.sequence),
            Some(3)
        );
        queue.close();
        assert_eq!(queue.pop().unwrap(), None);
    }
    #[test]
    fn event_frame_size_matches_encoded_frame() {
        let events = [
            Event::press(7, 3, 1, 1, 1, 1),
            Event::key(
                7,
                3,
                1,
                2,
                2,
                2,
                "Enter".to_owned(),
                vec!["ctrl".to_owned(), "shift".to_owned()],
                crate::protocol::KeyAction::Down,
            ),
            Event::text_input(
                crate::protocol::EVENT_CHANGE,
                7,
                3,
                1,
                3,
                2,
                2,
                crate::protocol::TextInputEvent {
                    text: "hello".to_owned(),
                    selection_start: 0,
                    selection_end: 5,
                    marked_start: None,
                    marked_end: None,
                    edit_seq: 1,
                    reversed: false,
                },
            ),
            Event::external_file_drop(
                7,
                3,
                1,
                4,
                2,
                2,
                vec!["/tmp/a".to_owned(), "/tmp/b".to_owned()],
            ),
            Event::command_result(
                7,
                3,
                1,
                5,
                crate::protocol::CommandResult {
                    request_id: 1,
                    command: crate::protocol::CommandKind::ClipboardReadImage,
                    node_id: 1,
                    success: true,
                    error: Some("ok".to_owned()),
                    value: Some(crate::protocol::CommandValue::Image(
                        crate::protocol::ClipboardImage {
                            format: 1,
                            bytes: vec![1, 2, 3],
                        },
                    )),
                },
            ),
        ];
        for event in events {
            assert_eq!(
                event.frame_size().unwrap(),
                event.encode().unwrap().len() + 4
            );
        }
    }

    #[test]
    fn event_queue_exhaustion_never_waits_for_the_renderer() {
        let queue = EventQueue::new();
        for sequence in 0..PROCESS_EVENT_QUEUE_CAPACITY as u32 {
            queue.push(event(sequence), 1).unwrap();
        }
        let error = queue.push(event(u32::MAX), 1).unwrap_err();
        assert!(
            matches!(error, ProtocolError::Io(error) if error.kind() == io::ErrorKind::WouldBlock)
        );
        for sequence in 0..PROCESS_EVENT_QUEUE_CAPACITY as u32 {
            assert_eq!(queue.pop().unwrap().unwrap().0.meta.sequence, sequence);
        }
        queue.push(event(7), PROCESS_EVENT_QUEUE_MAX_BYTES).unwrap();
        assert!(queue.push(event(8), 1).is_err());
        queue.fail("writer stopped".to_owned());
        assert!(
            queue
                .push(event(9), 1)
                .unwrap_err()
                .to_string()
                .contains("writer stopped")
        );
    }

    #[test]
    fn event_queue_rejects_a_frame_larger_than_the_byte_bound() {
        let queue = EventQueue::new();
        let error = queue
            .push(event(1), PROCESS_EVENT_QUEUE_MAX_BYTES + 1)
            .unwrap_err()
            .to_string();
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
