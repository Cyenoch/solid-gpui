use std::fmt::Write as _;
use std::fs::{File, OpenOptions};
use std::io::{self, Write as _};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

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
    /// Construct a process-local tap from `REACT_GPUI_TAP`.
    ///
    /// The environment is read exactly once for this transport/adapter. A
    /// missing path leaves the tap disabled, while an inaccessible path is a
    /// non-fatal warning and also disables it.
    pub fn from_env() -> Self {
        if OPEN_FAILURE_DISABLED.load(Ordering::Acquire) {
            return Self::disabled();
        }
        let Some(path) = std::env::var_os("REACT_GPUI_TAP") else {
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
                "react-gpui: protocol tap disabled; cannot open {}: {error}",
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
    let mut cursor = Cursor::new(payload);
    let Ok(length) = cursor.read_array_len() else {
        return Classification::UNKNOWN;
    };
    if length < 2 {
        return Classification::UNKNOWN;
    }
    if cursor.read_u64().is_err() {
        return Classification::UNKNOWN;
    }
    let Ok(message) = cursor.read_u64() else {
        return Classification::UNKNOWN;
    };
    match message {
        1 => Classification {
            kind: MessageKind::Snapshot,
            ..Classification::UNKNOWN
        },
        3 => Classification {
            kind: MessageKind::Patch,
            ..Classification::UNKNOWN
        },
        2 => classify_event(&mut cursor, length),
        4 => classify_command(&mut cursor, length),
        _ => Classification::UNKNOWN,
    }
}

fn classify_event(cursor: &mut Cursor<'_>, length: usize) -> Classification {
    if length < 9 {
        return Classification::UNKNOWN;
    }
    for index in 2..=8 {
        if index == 8 {
            let Ok(event_type) = cursor.read_u64() else {
                return Classification::UNKNOWN;
            };
            let mut result = Classification {
                kind: MessageKind::Event,
                event_type: Some(event_type),
                ..Classification::UNKNOWN
            };
            if event_type == 6 && length > 9 {
                result = parse_command_result(cursor, result);
            }
            return result;
        }
        if cursor.skip_value().is_err() {
            return Classification::UNKNOWN;
        }
    }
    Classification::UNKNOWN
}

fn parse_command_result(cursor: &mut Cursor<'_>, mut result: Classification) -> Classification {
    let Ok(length) = cursor.read_array_len() else {
        return result;
    };
    if length < 5 {
        return result;
    }
    let Ok(tag) = cursor.read_u64() else {
        return result;
    };
    if tag != 2 {
        return result;
    }
    let Ok(request_id) = cursor.read_u64() else {
        return result;
    };
    result.request_id = Some(request_id);
    if cursor.skip_value().is_err() || cursor.skip_value().is_err() {
        return result;
    }
    result.success = cursor.read_bool().ok();
    result
}

fn classify_command(cursor: &mut Cursor<'_>, length: usize) -> Classification {
    if length < 8 {
        return Classification::UNKNOWN;
    }
    let mut request_id = None;
    let mut command_kind = None;
    for index in 2..=7 {
        if index == 5 {
            request_id = cursor.read_u64().ok();
            if request_id.is_none() {
                return Classification::UNKNOWN;
            }
        } else if index == 7 {
            command_kind = cursor.read_u64().ok();
            if command_kind.is_none() {
                return Classification::UNKNOWN;
            }
        } else if cursor.skip_value().is_err() {
            return Classification::UNKNOWN;
        }
    }
    Classification {
        kind: MessageKind::Command,
        command_kind,
        request_id,
        ..Classification::UNKNOWN
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn byte(&mut self) -> Result<u8, ()> {
        let byte = *self.bytes.get(self.offset).ok_or(())?;
        self.offset += 1;
        Ok(byte)
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], ()> {
        let end = self.offset.checked_add(count).ok_or(())?;
        let bytes = self.bytes.get(self.offset..end).ok_or(())?;
        self.offset = end;
        Ok(bytes)
    }

    fn read_array_len(&mut self) -> Result<usize, ()> {
        let tag = self.byte()?;
        match tag {
            0x90..=0x9f => Ok((tag & 0x0f) as usize),
            0xdc => Ok(u16::from_be_bytes(self.take(2)?.try_into().map_err(|_| ())?) as usize),
            0xdd => {
                let length = u32::from_be_bytes(self.take(4)?.try_into().map_err(|_| ())?);
                usize::try_from(length).map_err(|_| ())
            }
            _ => Err(()),
        }
    }

    fn read_u64(&mut self) -> Result<u64, ()> {
        let tag = self.byte()?;
        match tag {
            0x00..=0x7f => Ok(tag as u64),
            0xcc => Ok(self.byte()? as u64),
            0xcd => Ok(u16::from_be_bytes(self.take(2)?.try_into().map_err(|_| ())?) as u64),
            0xce => Ok(u32::from_be_bytes(self.take(4)?.try_into().map_err(|_| ())?) as u64),
            0xcf => Ok(u64::from_be_bytes(
                self.take(8)?.try_into().map_err(|_| ())?,
            )),
            0xd0 => Ok(i8::from_be_bytes([self.byte()?])
                .try_into()
                .map_err(|_| ())?),
            0xd1 => Ok(
                i16::from_be_bytes(self.take(2)?.try_into().map_err(|_| ())?)
                    .try_into()
                    .map_err(|_| ())?,
            ),
            0xd2 => Ok(
                i32::from_be_bytes(self.take(4)?.try_into().map_err(|_| ())?)
                    .try_into()
                    .map_err(|_| ())?,
            ),
            0xd3 => Ok(
                i64::from_be_bytes(self.take(8)?.try_into().map_err(|_| ())?)
                    .try_into()
                    .map_err(|_| ())?,
            ),
            _ => Err(()),
        }
    }

    fn read_bool(&mut self) -> Result<bool, ()> {
        match self.byte()? {
            0xc2 => Ok(false),
            0xc3 => Ok(true),
            _ => Err(()),
        }
    }

    fn skip_value(&mut self) -> Result<(), ()> {
        self.skip_value_depth(0)
    }

    fn skip_value_depth(&mut self, depth: usize) -> Result<(), ()> {
        if depth > 64 {
            return Err(());
        }
        let tag = self.byte()?;
        match tag {
            0x00..=0x7f | 0xe0..=0xff | 0xc0 | 0xc2..=0xc3 => Ok(()),
            0xa0..=0xbf => self.take((tag & 0x1f) as usize).map(|_| ()),
            0x90..=0x9f => self.skip_many((tag & 0x0f) as usize, depth),
            0x80..=0x8f => self.skip_many((tag & 0x0f) as usize * 2, depth),
            0xc4 => self.skip_sized(1),
            0xc5 => self.skip_sized(2),
            0xc6 => self.skip_sized(4),
            0xca => self.take(4).map(|_| ()),
            0xcb => self.take(8).map(|_| ()),
            0xcc | 0xd0 => self.take(1).map(|_| ()),
            0xcd | 0xd1 => self.take(2).map(|_| ()),
            0xce | 0xd2 => self.take(4).map(|_| ()),
            0xcf | 0xd3 => self.take(8).map(|_| ()),
            0xd9 => self.skip_sized(1),
            0xda => self.skip_sized(2),
            0xdb => self.skip_sized(4),
            0xdc => {
                let length = u16::from_be_bytes(self.take(2)?.try_into().map_err(|_| ())?) as usize;
                self.skip_many(length, depth)
            }
            0xdd => {
                let length = u32::from_be_bytes(self.take(4)?.try_into().map_err(|_| ())?);
                self.skip_many(usize::try_from(length).map_err(|_| ())?, depth)
            }
            0xde => {
                let length = u16::from_be_bytes(self.take(2)?.try_into().map_err(|_| ())?) as usize;
                self.skip_many(length.saturating_mul(2), depth)
            }
            0xdf => {
                let length = u32::from_be_bytes(self.take(4)?.try_into().map_err(|_| ())?);
                self.skip_many(
                    usize::try_from(length).map_err(|_| ())?.saturating_mul(2),
                    depth,
                )
            }
            0xc7 => self.skip_ext(1),
            0xc8 => self.skip_ext(2),
            0xc9 => self.skip_ext(4),
            0xd4 => self.take(2).map(|_| ()),
            0xd5 => self.take(3).map(|_| ()),
            0xd6 => self.take(5).map(|_| ()),
            0xd7 => self.take(9).map(|_| ()),
            0xd8 => self.take(17).map(|_| ()),
            _ => Err(()),
        }
    }

    fn skip_many(&mut self, count: usize, depth: usize) -> Result<(), ()> {
        for _ in 0..count {
            self.skip_value_depth(depth + 1)?;
        }
        Ok(())
    }

    fn skip_sized(&mut self, width: usize) -> Result<(), ()> {
        let length = match width {
            1 => self.byte()? as usize,
            2 => u16::from_be_bytes(self.take(2)?.try_into().map_err(|_| ())?) as usize,
            4 => usize::try_from(u32::from_be_bytes(
                self.take(4)?.try_into().map_err(|_| ())?,
            ))
            .map_err(|_| ())?,
            _ => return Err(()),
        };
        self.take(length).map(|_| ())
    }

    fn skip_ext(&mut self, width: usize) -> Result<(), ()> {
        let length = match width {
            1 => self.byte()? as usize,
            2 => u16::from_be_bytes(self.take(2)?.try_into().map_err(|_| ())?) as usize,
            4 => usize::try_from(u32::from_be_bytes(
                self.take(4)?.try_into().map_err(|_| ())?,
            ))
            .map_err(|_| ())?,
            _ => return Err(()),
        };
        self.take(length.saturating_add(1)).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("react-gpui-tap-{name}-{stamp}.jsonl"))
    }

    fn array(values: &[u8]) -> Vec<u8> {
        let mut payload = vec![0x90 + values.len() as u8];
        payload.extend_from_slice(values);
        payload
    }

    #[test]
    fn classifies_message_and_subtype_prefixes_without_decoding_payload() {
        let event = array(&[0x03, 0x02, 0, 0, 0, 0, 0, 0, 0x0e]);
        let result = classify(&event);
        assert!(matches!(result.kind, MessageKind::Event));
        assert_eq!(result.event_type, Some(14));

        let command = array(&[0x03, 0x04, 0, 0, 0, 0x2a, 0, 0x0e]);
        let result = classify(&command);
        assert!(matches!(result.kind, MessageKind::Command));
        assert_eq!(result.command_kind, Some(14));
        assert_eq!(result.request_id, Some(42));

        assert!(matches!(
            classify(&[0x91, 0x03]),
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
        let payload = array(&[0x03, 0x01]);
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
