use std::collections::VecDeque;
use std::io;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Condvar, Mutex};

use crate::protocol::{Event, ProtocolError, read_frame, write_frame};

/// The renderer transport seam. The renderer tree never depends on whether
/// bytes came from Bun, an embedded runtime, or an in-memory test peer.
pub trait RuntimeAdapter: Send + Sync {
    fn recv_commit(&self) -> Result<Option<Vec<u8>>, ProtocolError>;
    fn send_event(&self, event: &Event) -> Result<(), ProtocolError>;
    fn shutdown(&self) -> Result<(), ProtocolError>;
}

/// A child-process adapter using framed stdin/stdout. Child stderr is inherited
/// by the host so renderer logs remain visible without entering the protocol.
pub struct ProcessAdapter {
    child: Mutex<Option<Child>>,
    commits: Mutex<ChildStdout>,
    events: Mutex<ChildStdin>,
}

impl ProcessAdapter {
    pub fn spawn(mut command: Command) -> io::Result<Arc<Self>> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let commits = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("renderer stdout was not piped"))?;
        let events = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("renderer stdin was not piped"))?;
        Ok(Arc::new(Self {
            child: Mutex::new(Some(child)),
            commits: Mutex::new(commits),
            events: Mutex::new(events),
        }))
    }

    pub fn try_wait(&self) -> io::Result<Option<std::process::ExitStatus>> {
        let mut child = self
            .child
            .lock()
            .map_err(|_| io::Error::other("renderer child lock poisoned"))?;
        let Some(child) = child.as_mut() else {
            return Ok(None);
        };
        child.try_wait()
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
        let mut writer = self
            .events
            .lock()
            .map_err(|_| ProtocolError::Io(io::Error::other("renderer stdin lock poisoned")))?;
        write_frame(&mut *writer, &payload)
    }

    fn shutdown(&self) -> Result<(), ProtocolError> {
        let mut child = self
            .child
            .lock()
            .map_err(|_| ProtocolError::Io(io::Error::other("renderer child lock poisoned")))?;
        let Some(mut child_process) = child.take() else {
            return Ok(());
        };
        match child_process.try_wait() {
            Ok(Some(_)) => Ok(()),
            Ok(None) => {
                child_process.kill().map_err(ProtocolError::Io)?;
                child_process.wait().map(|_| ()).map_err(ProtocolError::Io)
            }
            Err(error) => Err(ProtocolError::Io(error)),
        }
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
}
