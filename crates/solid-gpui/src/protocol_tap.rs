use std::fmt::Write as _;
use std::fs::{File, OpenOptions};
use std::io::{self, Write as _};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use web_time::Instant;

pub const TAP_CAPACITY_BYTES: u64 = 64 * 1024 * 1024;
const TAP_STOP_RESERVE_BYTES: usize = 256;
static OPEN_FAILURE_DISABLED: AtomicBool = AtomicBool::new(false);
static OPEN_FAILURE_WARNED: AtomicBool = AtomicBool::new(false);

#[derive(Clone)]
pub struct ProtocolTap {
    state: Option<Arc<Mutex<TapState>>>,
}

struct TapState {
    file: File,
    started: Instant,
    bytes_written: u64,
    seq: u64,
    stopped: bool,
    capacity_bytes: u64,
}

#[derive(Clone, Copy)]
enum Direction {
    In,
    Out,
}

impl Direction {
    fn as_str(self) -> &'static str {
        match self {
            Self::In => "in",
            Self::Out => "out",
        }
    }
}

#[derive(Clone, Copy)]
enum Peer {
    Renderer,
}

impl Peer {
    fn as_str(self) -> &'static str {
        "renderer"
    }
}

#[derive(Clone, Copy)]
enum MessageKind {
    Snapshot,
    Patch,
    Event,
    Command,
    Unknown,
}

impl MessageKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Snapshot => "snapshot",
            Self::Patch => "patch",
            Self::Event => "event",
            Self::Command => "command",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy)]
struct Classification {
    kind: MessageKind,
    event_type: Option<u64>,
    command_kind: Option<u64>,
    request_id: Option<u64>,
    success: Option<bool>,
}

impl Classification {
    const UNKNOWN: Self = Self {
        kind: MessageKind::Unknown,
        event_type: None,
        command_kind: None,
        request_id: None,
        success: None,
    };
}

impl ProtocolTap {
    /// Construct a process-local tap from `SOLID_GPUI_TAP`.
    ///
    /// The environment is read exactly once for this transport/adapter. A
    /// missing path leaves the tap disabled, while an inaccessible path is a
    /// non-fatal warning and also disables it.
    pub fn from_env() -> Self {
        if OPEN_FAILURE_DISABLED.load(Ordering::Acquire) {
            return Self::disabled();
        }
        let Some(path) = std::env::var_os("SOLID_GPUI_TAP") else {
            return Self::disabled();
        };
        if path.is_empty() {
            return Self::disabled();
        }
        Self::open(Path::new(&path), TAP_CAPACITY_BYTES)
    }

    fn disabled() -> Self {
        Self { state: None }
    }

    fn open(path: &Path, capacity_bytes: u64) -> Self {
        match open_file(path) {
            Ok(file) => Self {
                state: Some(Arc::new(Mutex::new(TapState {
                    file,
                    started: Instant::now(),
                    bytes_written: 0,
                    seq: 1,
                    stopped: false,
                    capacity_bytes,
                }))),
            },
            Err(error) => Self::open_failed(path, error),
        }
    }

    fn open_failed(path: &Path, error: io::Error) -> Self {
        if !OPEN_FAILURE_WARNED.swap(true, Ordering::AcqRel) {
            eprintln!(
                "solid-gpui: protocol tap disabled; cannot open {}: {error}",
                path.display()
            );
        }
        OPEN_FAILURE_DISABLED.store(true, Ordering::Release);
        Self::disabled()
    }

    /// Record an outbound frame whose bytes include the four-byte length
    /// prefix. This is the narrow hook used by the embedded Bun adapter.
    pub fn record_outbound_frame(&self, frame: &[u8]) {
        self.record_full_frame(Direction::Out, Peer::Renderer, frame);
    }

    /// Record an inbound frame whose bytes include the four-byte length prefix.
    pub fn record_inbound_frame(&self, frame: &[u8]) {
        self.record_full_frame(Direction::In, Peer::Renderer, frame);
    }

    /// Record an outbound protocol payload after a host writer successfully
    /// wrote its four-byte frame header and payload.
    pub fn record_outbound_payload(&self, payload: &[u8]) {
        self.record_payload(Direction::Out, Peer::Renderer, payload);
    }

    /// Record an inbound protocol payload read after the host consumed its
    /// four-byte frame header.
    pub fn record_inbound_payload(&self, payload: &[u8]) {
        self.record_payload(Direction::In, Peer::Renderer, payload);
    }

    /// Return whether this tap opened a file successfully.
    pub fn is_enabled(&self) -> bool {
        self.state.is_some()
    }

    fn record_full_frame(&self, direction: Direction, peer: Peer, frame: &[u8]) {
        let payload = frame.get(4..).unwrap_or(&[]);
        self.record(direction, peer, payload, frame.len());
    }

    fn record_payload(&self, direction: Direction, peer: Peer, payload: &[u8]) {
        self.record(direction, peer, payload, payload.len().saturating_add(4));
    }

    fn record(&self, direction: Direction, peer: Peer, payload: &[u8], frame_bytes: usize) {
        let Some(state) = &self.state else {
            return;
        };
        let classification = classify(payload);
        let mut state = match state.lock() {
            Ok(state) => state,
            Err(_) => return,
        };
        if state.stopped {
            return;
        }
        let timestamp = state.started.elapsed().as_millis();
        let line = render_record(
            timestamp,
            direction,
            peer,
            classification,
            frame_bytes,
            state.seq,
        );
        if state.bytes_written.saturating_add(line.len() as u64)
            > state
                .capacity_bytes
                .saturating_sub(TAP_STOP_RESERVE_BYTES as u64)
        {
            let stop = render_stopped(
                timestamp,
                direction,
                peer,
                frame_bytes,
                state.seq,
                "capacity",
            );
            if state.bytes_written.saturating_add(stop.len() as u64) <= state.capacity_bytes
                && write_line(&mut state, &stop).is_ok()
            {
                state.seq = state.seq.saturating_add(1);
            }
            state.stopped = true;
            return;
        }
        if write_line(&mut state, &line).is_err() {
            state.stopped = true;
            return;
        }
        state.seq = state.seq.saturating_add(1);
    }
}

fn open_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

fn write_line(state: &mut TapState, line: &str) -> io::Result<()> {
    state.file.write_all(line.as_bytes())?;
    state.file.flush()?;
    state.bytes_written = state.bytes_written.saturating_add(line.len() as u64);
    Ok(())
}

fn render_record(
    timestamp: u128,
    direction: Direction,
    peer: Peer,
    classification: Classification,
    frame_bytes: usize,
    seq: u64,
) -> String {
    let mut line = String::with_capacity(160);
    write!(
        line,
        "{{\"t\":{timestamp},\"dir\":\"{}\",\"peer\":\"{}\",\"kind\":\"{}\",\"bytes\":{frame_bytes},\"seq\":{seq}",
        direction.as_str(),
        peer.as_str(),
        classification.kind.as_str(),
    )
    .expect("writing protocol tap metadata cannot fail");
    if let Some(event_type) = classification.event_type {
        write!(line, ",\"event_type\":{event_type}")
            .expect("writing protocol tap metadata cannot fail");
    }
    if let Some(command_kind) = classification.command_kind {
        write!(line, ",\"command_kind\":{command_kind}")
            .expect("writing protocol tap metadata cannot fail");
    }
    if let Some(request_id) = classification.request_id {
        write!(line, ",\"request_id\":{request_id}")
            .expect("writing protocol tap metadata cannot fail");
    }
    if let Some(success) = classification.success {
        write!(line, ",\"success\":{success}").expect("writing protocol tap metadata cannot fail");
    }
    line.push_str("}\n");
    line
}

fn render_stopped(
    timestamp: u128,
    direction: Direction,
    peer: Peer,
    frame_bytes: usize,
    seq: u64,
    reason: &str,
) -> String {
    format!(
        "{{\"t\":{timestamp},\"dir\":\"{}\",\"peer\":\"{}\",\"kind\":\"tap_stopped\",\"bytes\":{frame_bytes},\"seq\":{seq},\"reason\":\"{reason}\"}}\n",
        direction.as_str(),
        peer.as_str(),
    )
}

fn classify(payload: &[u8]) -> Classification {
    let Ok((message, event_type, command_kind, request_id, success)) =
        crate::protocol::classify_payload(payload)
    else {
        return Classification::UNKNOWN;
    };
    let kind = match message {
        crate::protocol::SNAPSHOT_MESSAGE => MessageKind::Snapshot,
        crate::protocol::PATCH_MESSAGE => MessageKind::Patch,
        crate::protocol::EVENT_MESSAGE => MessageKind::Event,
        crate::protocol::COMMAND_MESSAGE => MessageKind::Command,
        _ => MessageKind::Unknown,
    };
    Classification {
        kind,
        event_type: event_type.map(u64::from),
        command_kind: command_kind.map(u64::from),
        request_id: request_id.map(u64::from),
        success,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{
        COMMAND_GET_WINDOW_SIZE, Command, CommandMeta, CommandOperation, EVENT_WINDOW_RESIZE, Event,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("solid-gpui-tap-{name}-{stamp}.jsonl"))
    }

    #[test]
    fn classifies_generated_message_and_subtype_metadata() {
        let event = Event::window_resize(7, 3, 1, 1, 1, 0, 800.0, 600.0);
        let result = classify(&event.encode().unwrap());
        assert!(matches!(result.kind, MessageKind::Event));
        assert_eq!(result.event_type, Some(u64::from(EVENT_WINDOW_RESIZE)));

        let command = Command::new(
            CommandMeta {
                surface_id: 7,
                epoch: 3,
                after_revision: 1,
                request_id: 42,
                node_id: 1,
            },
            CommandOperation::GetWindowSize,
        );
        let result = classify(&command.encode().unwrap());
        assert!(matches!(result.kind, MessageKind::Command));
        assert_eq!(
            result.command_kind,
            Some(u64::from(COMMAND_GET_WINDOW_SIZE))
        );
        assert_eq!(result.request_id, Some(42));

        assert!(matches!(
            classify(&[0]),
            Classification {
                kind: MessageKind::Unknown,
                ..
            }
        ));
    }

    #[test]
    fn writes_metadata_with_monotonic_sequence_and_capacity_stop() {
        let path = temp_path("capacity");
        let tap = ProtocolTap::open(&path, 300);
        let payload = Event::press(7, 3, 1, 1, 1, 1).encode().unwrap();
        tap.record_inbound_payload(&payload);
        tap.record_outbound_payload(&payload);
        tap.record_inbound_payload(&payload);
        let text = fs::read_to_string(&path).expect("tap output");
        let lines: Vec<_> = text.lines().collect();
        assert!(
            lines
                .iter()
                .any(|line| line.contains("\"kind\":\"tap_stopped\""))
        );
        let seq: Vec<_> = lines
            .iter()
            .filter_map(|line| {
                line.split("\"seq\":")
                    .nth(1)
                    .and_then(|tail| tail.split([',', '}']).next())
                    .and_then(|value| value.parse::<u64>().ok())
            })
            .collect();
        assert!(seq.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(fs::metadata(&path).expect("tap metadata").len() <= 300);
        let _ = fs::remove_file(path);
    }
}
