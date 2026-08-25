use std::io::{self, Cursor, Read, Write};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PROTOCOL_VERSION: u32 = 3;
pub const SNAPSHOT_MESSAGE: u32 = 1;
pub const EVENT_MESSAGE: u32 = 2;
pub const PATCH_MESSAGE: u32 = 3;
pub const COMMAND_MESSAGE: u32 = 4;

pub const EVENT_PRESS: u32 = 1;
pub const EVENT_CHANGE: u32 = 2;
pub const EVENT_SELECTION: u32 = 3;
pub const EVENT_FOCUS: u32 = 4;
pub const EVENT_BLUR: u32 = 5;
pub const EVENT_COMMAND_RESULT: u32 = 6;
pub const EVENT_VISIBLE_RANGE: u32 = 7;
pub const EVENT_ANIMATION_COMPLETE: u32 = 8;

pub const COMMAND_FOCUS: u32 = 1;
pub const COMMAND_BLUR: u32 = 2;
pub const COMMAND_SET_SELECTION: u32 = 3;
pub const COMMAND_SCROLL_TO_INDEX: u32 = 4;
pub const COMMAND_SCROLL_TO_END: u32 = 5;

pub const UPDATE_STYLE: u32 = 1;
pub const UPDATE_TEXT: u32 = 2;
pub const UPDATE_LISTENER: u32 = 4;
pub const UPDATE_PROPERTIES: u32 = 8;
pub const UPDATE_ACCESSIBILITY: u32 = 16;
pub const MAX_FRAME_LENGTH: usize = 16 * 1024 * 1024;

pub const TRANSITION_OPACITY: u32 = 1;
pub const TRANSITION_BACKGROUND_COLOR: u32 = 2;

/// A complete immutable React commit. It is encoded as
/// `[3,1,surfaceId,epoch,baseRevision,revision,nodes]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub protocol: u32,
    pub message: u32,
    pub surface_id: u32,
    pub epoch: u32,
    pub base_revision: u32,
    pub revision: u32,
    pub nodes: Vec<Node>,
}

impl Snapshot {
    pub fn new(
        surface_id: u32,
        epoch: u32,
        base_revision: u32,
        revision: u32,
        nodes: Vec<Node>,
    ) -> Self {
        Self {
            protocol: PROTOCOL_VERSION,
            message: SNAPSHOT_MESSAGE,
            surface_id,
            epoch,
            base_revision,
            revision,
            nodes,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let wire = SnapshotWire(
            self.protocol,
            self.message,
            self.surface_id,
            self.epoch,
            self.base_revision,
            self.revision,
            self.nodes.iter().map(NodeWire::from).collect(),
        );
        rmp_serde::to_vec(&wire).map_err(ProtocolError::Encode)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, ProtocolError> {
        let mut deserializer = rmp_serde::Deserializer::new(Cursor::new(payload));
        let wire = SnapshotWire::deserialize(&mut deserializer).map_err(ProtocolError::Decode)?;
        let consumed = deserializer.into_inner().position() as usize;
        if consumed != payload.len() {
            return Err(ProtocolError::TrailingBytes(payload.len() - consumed));
        }
        if wire.0 != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedProtocol(wire.0));
        }
        if wire.1 != SNAPSHOT_MESSAGE {
            return Err(ProtocolError::WrongMessageType(wire.1));
        }
        Ok(Self {
            protocol: wire.0,
            message: wire.1,
            surface_id: wire.2,
            epoch: wire.3,
            base_revision: wire.4,
            revision: wire.5,
            nodes: wire
                .6
                .into_iter()
                .map(Node::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// An atomic incremental commit. It is encoded as
/// `[3,3,surfaceId,epoch,baseRevision,revision,operations]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    pub protocol: u32,
    pub message: u32,
    pub surface_id: u32,
    pub epoch: u32,
    pub base_revision: u32,
    pub revision: u32,
    pub operations: Vec<PatchOperation>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatchOperation {
    Create(Node),
    Update {
        id: u32,
        mask: u32,
        style: Option<Style>,
        text: Option<String>,
        listener_id: u32,
        host_properties: Option<HostProperties>,
        accessibility: Option<AccessibilityProperties>,
    },
    Move {
        id: u32,
        parent_id: u32,
        index: u32,
    },
    Delete {
        id: u32,
    },
}

impl Patch {
    pub fn new(
        surface_id: u32,
        epoch: u32,
        base_revision: u32,
        revision: u32,
        operations: Vec<PatchOperation>,
    ) -> Self {
        Self {
            protocol: PROTOCOL_VERSION,
            message: PATCH_MESSAGE,
            surface_id,
            epoch,
            base_revision,
            revision,
            operations,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let wire = PatchWire(
            self.protocol,
            self.message,
            self.surface_id,
            self.epoch,
            self.base_revision,
            self.revision,
            self.operations.iter().map(OperationWire::from).collect(),
        );
        rmp_serde::to_vec(&wire).map_err(ProtocolError::Encode)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, ProtocolError> {
        let mut deserializer = rmp_serde::Deserializer::new(Cursor::new(payload));
        let wire = PatchWire::deserialize(&mut deserializer).map_err(ProtocolError::Decode)?;
        let consumed = deserializer.into_inner().position() as usize;
        if consumed != payload.len() {
            return Err(ProtocolError::TrailingBytes(payload.len() - consumed));
        }
        if wire.0 != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedProtocol(wire.0));
        }
        if wire.1 != PATCH_MESSAGE {
            return Err(ProtocolError::WrongMessageType(wire.1));
        }
        Ok(Self {
            protocol: wire.0,
            message: wire.1,
            surface_id: wire.2,
            epoch: wire.3,
            base_revision: wire.4,
            revision: wire.5,
            operations: wire
                .6
                .into_iter()
                .map(PatchOperation::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub protocol: u32,
    pub message: u32,
    pub surface_id: u32,
    pub epoch: u32,
    pub after_revision: u32,
    pub request_id: u32,
    pub node_id: u32,
    pub kind: u32,
    pub payload: Option<(u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResult {
    pub request_id: u32,
    pub command: u32,
    pub node_id: u32,
    pub success: bool,
    pub error: Option<String>,
}

impl Command {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        rmp_serde::to_vec(&CommandWire(
            self.protocol,
            self.message,
            self.surface_id,
            self.epoch,
            self.after_revision,
            self.request_id,
            self.node_id,
            self.kind,
            self.payload,
        ))
        .map_err(ProtocolError::Encode)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, ProtocolError> {
        let mut deserializer = rmp_serde::Deserializer::new(Cursor::new(payload));
        let wire = CommandWire::deserialize(&mut deserializer).map_err(ProtocolError::Decode)?;
        let consumed = deserializer.into_inner().position() as usize;
        if consumed != payload.len() {
            return Err(ProtocolError::TrailingBytes(payload.len() - consumed));
        }
        if wire.0 != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedProtocol(wire.0));
        }
        if wire.1 != COMMAND_MESSAGE {
            return Err(ProtocolError::WrongMessageType(wire.1));
        }
        if !matches!(
            wire.7,
            COMMAND_FOCUS
                | COMMAND_BLUR
                | COMMAND_SET_SELECTION
                | COMMAND_SCROLL_TO_INDEX
                | COMMAND_SCROLL_TO_END
        ) {
            return Err(ProtocolError::UnknownCommand(wire.7));
        }
        match wire.7 {
            COMMAND_FOCUS | COMMAND_BLUR | COMMAND_SCROLL_TO_END if wire.8.is_some() => {
                return Err(ProtocolError::InvalidCommandPayload);
            }
            COMMAND_SET_SELECTION | COMMAND_SCROLL_TO_INDEX if wire.8.is_none() => {
                return Err(ProtocolError::InvalidCommandPayload);
            }
            COMMAND_SET_SELECTION => {
                let (start, end) = wire.8.expect("checked above");
                if start > end {
                    return Err(ProtocolError::InvalidCommandPayload);
                }
            }
            _ => {}
        }
        Ok(Self {
            protocol: wire.0,
            message: wire.1,
            surface_id: wire.2,
            epoch: wire.3,
            after_revision: wire.4,
            request_id: wire.5,
            node_id: wire.6,
            kind: wire.7,
            payload: wire.8,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: u32,
    pub parent_id: u32,
    pub index: u32,
    pub kind: u32,
    pub style: Option<Style>,
    pub text: Option<String>,
    pub listener_id: u32,
    pub host_properties: Option<HostProperties>,
    pub accessibility: Option<AccessibilityProperties>,
}

impl Node {
    pub fn new(id: u32, parent_id: u32, index: u32, kind: u32) -> Self {
        Self {
            id,
            parent_id,
            index,
            kind,
            style: None,
            text: None,
            listener_id: 0,
            host_properties: None,
            accessibility: None,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct AccessibilityProperties {
    pub role: u32,
    pub label: Option<String>,
    pub description: Option<String>,
    pub disabled: bool,
    pub checked: Option<bool>,
    pub selected: Option<bool>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HostProperties {
    TextInput(TextInputProperties),
    VirtualList(VirtualListProperties),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextInputProperties {
    pub value: String,
    pub placeholder: Option<String>,
    pub multiline: bool,
    pub disabled: bool,
    pub controlled: bool,
    pub ack_edit_seq: u32,
    pub selection_start: u32,
    pub selection_end: u32,
    pub marked_start: Option<u32>,
    pub marked_end: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VirtualListProperties {
    pub item_count: u32,
    pub range_start: u32,
    pub range_end: u32,
    pub estimated_item_size: f32,
    pub overscan: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    pub duration_ms: u32,
    pub delay_ms: u32,
    pub easing: Easing,
    pub properties: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub flex_direction: Option<u32>,
    pub flex_grow: Option<f32>,
    pub padding: Option<f32>,
    pub gap: Option<f32>,
    pub background_rgba: Option<u32>,
    pub color_rgba: Option<u32>,
    pub opacity: Option<f32>,
    pub transition: Option<Transition>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            flex_direction: None,
            flex_grow: None,
            padding: None,
            gap: None,
            background_rgba: None,
            color_rgba: None,
            opacity: None,
            transition: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputEvent {
    pub text: String,
    pub selection_start: u32,
    pub selection_end: u32,
    pub marked_start: Option<u32>,
    pub marked_end: Option<u32>,
    pub edit_seq: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventPayload {
    TextInput(TextInputEvent),
    CommandResult(CommandResult),
    VisibleRange { start: u32, end: u32 },
    AnimationComplete { generation: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub protocol: u32,
    pub message: u32,
    pub surface_id: u32,
    pub epoch: u32,
    pub revision: u32,
    pub sequence: u32,
    pub node_id: u32,
    pub listener_id: u32,
    pub event_type: u32,
    pub payload: Option<EventPayload>,
}

impl Event {
    pub fn press(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
    ) -> Self {
        Self {
            protocol: PROTOCOL_VERSION,
            message: EVENT_MESSAGE,
            surface_id,
            epoch,
            revision,
            sequence,
            node_id,
            listener_id,
            event_type: EVENT_PRESS,
            payload: None,
        }
    }

    pub fn text_input(
        event_type: u32,
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        payload: TextInputEvent,
    ) -> Self {
        Self {
            protocol: PROTOCOL_VERSION,
            message: EVENT_MESSAGE,
            surface_id,
            epoch,
            revision,
            sequence,
            node_id,
            listener_id,
            event_type,
            payload: Some(EventPayload::TextInput(payload)),
        }
    }

    pub fn command_result(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        result: CommandResult,
    ) -> Self {
        Self {
            protocol: PROTOCOL_VERSION,
            message: EVENT_MESSAGE,
            surface_id,
            epoch,
            revision,
            sequence,
            node_id: result.node_id,
            listener_id: 0,
            event_type: EVENT_COMMAND_RESULT,
            payload: Some(EventPayload::CommandResult(result)),
        }
    }

    pub fn visible_range(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        start: u32,
        end: u32,
    ) -> Self {
        Self {
            protocol: PROTOCOL_VERSION,
            message: EVENT_MESSAGE,
            surface_id,
            epoch,
            revision,
            sequence,
            node_id,
            listener_id,
            event_type: EVENT_VISIBLE_RANGE,
            payload: Some(EventPayload::VisibleRange { start, end }),
        }
    }

    pub fn animation_complete(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        generation: u32,
    ) -> Self {
        Self {
            protocol: PROTOCOL_VERSION,
            message: EVENT_MESSAGE,
            surface_id,
            epoch,
            revision,
            sequence,
            node_id,
            listener_id,
            event_type: EVENT_ANIMATION_COMPLETE,
            payload: Some(EventPayload::AnimationComplete { generation }),
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        rmp_serde::to_vec(&EventWire::from(self)).map_err(ProtocolError::Encode)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, ProtocolError> {
        let mut deserializer = rmp_serde::Deserializer::new(Cursor::new(payload));
        let wire = EventWire::deserialize(&mut deserializer).map_err(ProtocolError::Decode)?;
        let consumed = deserializer.into_inner().position() as usize;
        if consumed != payload.len() {
            return Err(ProtocolError::TrailingBytes(payload.len() - consumed));
        }
        if wire.0 != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedProtocol(wire.0));
        }
        if wire.1 != EVENT_MESSAGE {
            return Err(ProtocolError::WrongMessageType(wire.1));
        }
        if !matches!(
            wire.8,
            EVENT_PRESS
                | EVENT_CHANGE
                | EVENT_SELECTION
                | EVENT_FOCUS
                | EVENT_BLUR
                | EVENT_COMMAND_RESULT
                | EVENT_VISIBLE_RANGE
                | EVENT_ANIMATION_COMPLETE
        ) {
            return Err(ProtocolError::UnknownEvent(wire.8));
        }
        let payload = match (wire.8, wire.9) {
            (EVENT_PRESS, None) => None,
            (
                EVENT_CHANGE | EVENT_SELECTION | EVENT_FOCUS | EVENT_BLUR,
                Some(EventPayloadWire::Text(value)),
            ) => Some(EventPayload::TextInput(TextInputEvent::try_from(value)?)),
            (EVENT_COMMAND_RESULT, Some(EventPayloadWire::Command(value))) => {
                if value.0 != 2
                    || !matches!(
                        value.2,
                        COMMAND_FOCUS
                            | COMMAND_BLUR
                            | COMMAND_SET_SELECTION
                            | COMMAND_SCROLL_TO_INDEX
                            | COMMAND_SCROLL_TO_END
                    )
                {
                    return Err(ProtocolError::InvalidEventPayload);
                }
                Some(EventPayload::CommandResult(CommandResult::from(value)))
            }
            (EVENT_VISIBLE_RANGE, Some(EventPayloadWire::Visible(value))) => {
                if value.0 != 3 || value.1 > value.2 {
                    return Err(ProtocolError::InvalidEventPayload);
                }
                Some(EventPayload::VisibleRange {
                    start: value.1,
                    end: value.2,
                })
            }
            (EVENT_ANIMATION_COMPLETE, Some(EventPayloadWire::Animation(value))) => {
                if value.0 != 4 {
                    return Err(ProtocolError::InvalidEventPayload);
                }
                Some(EventPayload::AnimationComplete {
                    generation: value.1,
                })
            }
            _ => return Err(ProtocolError::InvalidEventPayload),
        };
        Ok(Event {
            protocol: wire.0,
            message: wire.1,
            surface_id: wire.2,
            epoch: wire.3,
            revision: wire.4,
            sequence: wire.5,
            node_id: wire.6,
            listener_id: wire.7,
            event_type: wire.8,
            payload,
        })
    }
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("invalid MessagePack payload: {0}")]
    Decode(#[source] rmp_serde::decode::Error),
    #[error("could not encode MessagePack payload: {0}")]
    Encode(#[source] rmp_serde::encode::Error),
    #[error("transport I/O error: {0}")]
    Io(#[source] io::Error),
    #[error("MessagePack payload has {0} trailing bytes")]
    TrailingBytes(usize),
    #[error("unsupported protocol version {0}")]
    UnsupportedProtocol(u32),
    #[error("unexpected message type {0}")]
    WrongMessageType(u32),
    #[error("unknown event type {0}")]
    UnknownEvent(u32),
    #[error("event payload does not match event type")]
    InvalidEventPayload,
    #[error("unknown patch operation {0}")]
    UnknownPatchOperation(u32),
    #[error("frame payload is {0} bytes, maximum is {MAX_FRAME_LENGTH}")]
    FrameTooLarge(usize),
    #[error("truncated frame header after {0} bytes")]
    TruncatedHeader(usize),
    #[error("unknown command {0}")]
    UnknownCommand(u32),
    #[error("invalid command payload")]
    InvalidCommandPayload,
    #[error("invalid host properties")]
    InvalidHostProperties,
    #[error("invalid style payload")]
    InvalidStyle,
    #[error("invalid text input event payload")]
    InvalidTextInputEvent,
    #[error("truncated frame payload: expected {expected} bytes, received {received}")]
    TruncatedPayload { expected: usize, received: usize },
    #[error("frame length does not fit in this platform's usize: {0}")]
    LengthOverflow(u32),
}

/// Read one little-endian length-prefixed frame. EOF before a new header is a
/// clean shutdown; EOF in a header or payload is a protocol error.
pub fn read_frame<R: Read>(reader: &mut R) -> Result<Option<Vec<u8>>, ProtocolError> {
    let mut header = [0; 4];
    let mut read = 0;
    while read < header.len() {
        match reader.read(&mut header[read..]) {
            Ok(0) if read == 0 => return Ok(None),
            Ok(0) => return Err(ProtocolError::TruncatedHeader(read)),
            Ok(count) => read += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(ProtocolError::Io(error)),
        }
    }

    let raw_length = u32::from_le_bytes(header);
    let length =
        usize::try_from(raw_length).map_err(|_| ProtocolError::LengthOverflow(raw_length))?;
    if length > MAX_FRAME_LENGTH {
        return Err(ProtocolError::FrameTooLarge(length));
    }

    let mut payload = vec![0; length];
    let mut received = 0;
    while received < length {
        match reader.read(&mut payload[received..]) {
            Ok(0) => {
                return Err(ProtocolError::TruncatedPayload {
                    expected: length,
                    received,
                });
            }
            Ok(count) => received += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(ProtocolError::Io(error)),
        }
    }
    Ok(Some(payload))
}

/// Write one little-endian length-prefixed frame and flush it. The flush is
/// deliberate: events must not wait behind a buffered child-process write.
pub fn write_frame<W: Write>(writer: &mut W, payload: &[u8]) -> Result<(), ProtocolError> {
    if payload.len() > MAX_FRAME_LENGTH {
        return Err(ProtocolError::FrameTooLarge(payload.len()));
    }
    let length =
        u32::try_from(payload.len()).map_err(|_| ProtocolError::FrameTooLarge(payload.len()))?;
    writer
        .write_all(&length.to_le_bytes())
        .and_then(|_| writer.write_all(payload))
        .and_then(|_| writer.flush())
        .map_err(ProtocolError::Io)
}

#[derive(Debug, Serialize, Deserialize)]
struct SnapshotWire(u32, u32, u32, u32, u32, u32, Vec<NodeWire>);
#[derive(Debug, Serialize, Deserialize)]
struct PatchWire(u32, u32, u32, u32, u32, u32, Vec<OperationWire>);
#[derive(Debug, Serialize, Deserialize)]
struct CommandWire(u32, u32, u32, u32, u32, u32, u32, u32, Option<(u32, u32)>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum OperationWire {
    Create(CreateWire),
    Update(UpdateWire),
    Move(MoveWire),
    Delete(DeleteWire),
}

#[derive(Debug, Serialize, Deserialize)]
struct NodeWire(
    u32,
    u32,
    u32,
    u32,
    Option<StyleWire>,
    Option<String>,
    u32,
    Option<HostPropertiesWire>,
    Option<AccessibilityWire>,
);

#[derive(Debug, Serialize, Deserialize)]
struct CreateWire(
    u32,
    u32,
    u32,
    u32,
    u32,
    Option<StyleWire>,
    Option<String>,
    u32,
    Option<HostPropertiesWire>,
    Option<AccessibilityWire>,
);

#[derive(Debug, Serialize, Deserialize)]
struct UpdateWire(
    u32,
    u32,
    u32,
    Option<StyleWire>,
    Option<String>,
    u32,
    Option<HostPropertiesWire>,
    Option<AccessibilityWire>,
);

#[derive(Debug, Serialize, Deserialize)]
struct MoveWire(u32, u32, u32, u32);

#[derive(Debug, Serialize, Deserialize)]
struct DeleteWire(u32, u32);

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum HostPropertiesWire {
    TextInput(TextInputWire),
    VirtualList(VirtualListWire),
}

#[derive(Debug, Serialize, Deserialize)]
struct TextInputWire(
    u32,
    String,
    Option<String>,
    bool,
    bool,
    bool,
    u32,
    u32,
    u32,
    Option<u32>,
    Option<u32>,
);

#[derive(Debug, Serialize, Deserialize)]
struct VirtualListWire(u32, u32, u32, u32, f32, u32);

#[derive(Debug, Serialize, Deserialize)]
struct AccessibilityWire(
    u32,
    Option<String>,
    Option<String>,
    bool,
    Option<bool>,
    Option<bool>,
    Option<String>,
);

#[derive(Debug, Serialize, Deserialize)]
struct StyleWire(
    Option<f32>,
    Option<f32>,
    Option<u32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<u32>,
    Option<u32>,
    Option<f32>,
    Option<TransitionWire>,
);

#[derive(Debug, Serialize, Deserialize)]
struct TransitionWire(u32, u32, u32, u32);

#[derive(Debug, Serialize, Deserialize)]
struct EventWire(
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    Option<EventPayloadWire>,
);

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum EventPayloadWire {
    Text(TextInputEventWire),
    Command(CommandResultWire),
    Visible(VisibleRangeWire),
    Animation(AnimationCompleteWire),
}

#[derive(Debug, Serialize, Deserialize)]
struct TextInputEventWire(u32, String, u32, u32, Option<u32>, Option<u32>, u32);
#[derive(Debug, Serialize, Deserialize)]
struct CommandResultWire(u32, u32, u32, u32, bool, Option<String>);
#[derive(Debug, Serialize, Deserialize)]
struct VisibleRangeWire(u32, u32, u32);
#[derive(Debug, Serialize, Deserialize)]
struct AnimationCompleteWire(u32, u32);

impl From<&Node> for NodeWire {
    fn from(node: &Node) -> Self {
        Self(
            node.id,
            node.parent_id,
            node.index,
            node.kind,
            node.style.as_ref().map(StyleWire::from),
            node.text.clone(),
            node.listener_id,
            node.host_properties.as_ref().map(HostPropertiesWire::from),
            node.accessibility.as_ref().map(AccessibilityWire::from),
        )
    }
}

impl TryFrom<NodeWire> for Node {
    type Error = ProtocolError;

    fn try_from(node: NodeWire) -> Result<Self, Self::Error> {
        if let Some(style) = node.4.as_ref() {
            validate_style_wire(style)?;
        }
        let host_properties = node.7.map(HostProperties::try_from).transpose()?;
        validate_host_kind(node.3, host_properties.as_ref())?;
        Ok(Self {
            id: node.0,
            parent_id: node.1,
            index: node.2,
            kind: node.3,
            style: node.4.map(Style::from),
            text: node.5,
            listener_id: node.6,
            host_properties,
            accessibility: node.8.map(AccessibilityProperties::from),
        })
    }
}

impl From<&Style> for StyleWire {
    fn from(style: &Style) -> Self {
        Self(
            style.width,
            style.height,
            style.flex_direction,
            style.flex_grow,
            style.padding,
            style.gap,
            style.background_rgba,
            style.color_rgba,
            style.opacity,
            style.transition.as_ref().map(TransitionWire::from),
        )
    }
}

impl From<StyleWire> for Style {
    fn from(style: StyleWire) -> Self {
        Self {
            width: style.0,
            height: style.1,
            flex_direction: style.2,
            flex_grow: style.3,
            padding: style.4,
            gap: style.5,
            background_rgba: style.6,
            color_rgba: style.7,
            opacity: style.8,
            transition: style.9.map(Transition::from),
        }
    }
}

impl From<&Transition> for TransitionWire {
    fn from(transition: &Transition) -> Self {
        Self(
            transition.duration_ms,
            transition.delay_ms,
            transition.easing as u32,
            transition.properties,
        )
    }
}

impl From<TransitionWire> for Transition {
    fn from(transition: TransitionWire) -> Self {
        Self {
            duration_ms: transition.0,
            delay_ms: transition.1,
            easing: match transition.2 {
                0 => Easing::Linear,
                1 => Easing::EaseIn,
                2 => Easing::EaseOut,
                _ => Easing::EaseInOut,
            },
            properties: transition.3,
        }
    }
}

impl From<&HostProperties> for HostPropertiesWire {
    fn from(value: &HostProperties) -> Self {
        match value {
            HostProperties::TextInput(value) => Self::TextInput(TextInputWire::from(value)),
            HostProperties::VirtualList(value) => Self::VirtualList(VirtualListWire::from(value)),
        }
    }
}

impl TryFrom<HostPropertiesWire> for HostProperties {
    type Error = ProtocolError;

    fn try_from(value: HostPropertiesWire) -> Result<Self, Self::Error> {
        match value {
            HostPropertiesWire::TextInput(value) if value.0 == 1 => {
                if value.7 > value.8
                    || value.9.is_some() != value.10.is_some()
                    || value
                        .9
                        .zip(value.10)
                        .is_some_and(|(start, end)| start > end)
                {
                    return Err(ProtocolError::InvalidHostProperties);
                }
                Ok(Self::TextInput(TextInputProperties::from(value)))
            }
            HostPropertiesWire::VirtualList(value) if value.0 == 2 => {
                if value.2 > value.3 || value.3 > value.1 || !value.4.is_finite() || value.4 <= 0.0
                {
                    return Err(ProtocolError::InvalidHostProperties);
                }
                Ok(Self::VirtualList(VirtualListProperties::from(value)))
            }
            _ => Err(ProtocolError::InvalidHostProperties),
        }
    }
}

fn validate_host_kind(
    kind: u32,
    host_properties: Option<&HostProperties>,
) -> Result<(), ProtocolError> {
    match (kind, host_properties) {
        (5, Some(HostProperties::TextInput(_))) | (6, Some(HostProperties::VirtualList(_))) => {
            Ok(())
        }
        (5 | 6, None) => Err(ProtocolError::InvalidHostProperties),
        (_, Some(_)) => Err(ProtocolError::InvalidHostProperties),
        _ => Ok(()),
    }
}

fn validate_style_wire(style: &StyleWire) -> Result<(), ProtocolError> {
    if style.2.is_some_and(|direction| direction > 2)
        || [style.0, style.1, style.3, style.4, style.5, style.8]
            .into_iter()
            .flatten()
            .any(|value| !value.is_finite() || value < 0.0)
        || style.8.is_some_and(|opacity| opacity > 1.0)
    {
        return Err(ProtocolError::InvalidStyle);
    }
    if let Some(transition) = style.9.as_ref() {
        if transition.2 > 3
            || transition.3 == 0
            || transition.3 & !(TRANSITION_OPACITY | TRANSITION_BACKGROUND_COLOR) != 0
        {
            return Err(ProtocolError::InvalidStyle);
        }
    }
    Ok(())
}

impl From<&TextInputProperties> for TextInputWire {
    fn from(value: &TextInputProperties) -> Self {
        Self(
            1,
            value.value.clone(),
            value.placeholder.clone(),
            value.multiline,
            value.disabled,
            value.controlled,
            value.ack_edit_seq,
            value.selection_start,
            value.selection_end,
            value.marked_start,
            value.marked_end,
        )
    }
}

impl From<TextInputWire> for TextInputProperties {
    fn from(value: TextInputWire) -> Self {
        Self {
            value: value.1,
            placeholder: value.2,
            multiline: value.3,
            disabled: value.4,
            controlled: value.5,
            ack_edit_seq: value.6,
            selection_start: value.7,
            selection_end: value.8,
            marked_start: value.9,
            marked_end: value.10,
        }
    }
}

impl From<&VirtualListProperties> for VirtualListWire {
    fn from(value: &VirtualListProperties) -> Self {
        Self(
            2,
            value.item_count,
            value.range_start,
            value.range_end,
            value.estimated_item_size,
            value.overscan,
        )
    }
}

impl From<VirtualListWire> for VirtualListProperties {
    fn from(value: VirtualListWire) -> Self {
        Self {
            item_count: value.1,
            range_start: value.2,
            range_end: value.3,
            estimated_item_size: value.4,
            overscan: value.5,
        }
    }
}

impl From<&AccessibilityProperties> for AccessibilityWire {
    fn from(value: &AccessibilityProperties) -> Self {
        Self(
            value.role,
            value.label.clone(),
            value.description.clone(),
            value.disabled,
            value.checked,
            value.selected,
            value.value.clone(),
        )
    }
}

impl From<AccessibilityWire> for AccessibilityProperties {
    fn from(value: AccessibilityWire) -> Self {
        Self {
            role: value.0,
            label: value.1,
            description: value.2,
            disabled: value.3,
            checked: value.4,
            selected: value.5,
            value: value.6,
        }
    }
}

impl From<&TextInputEvent> for TextInputEventWire {
    fn from(event: &TextInputEvent) -> Self {
        Self(
            1,
            event.text.clone(),
            event.selection_start,
            event.selection_end,
            event.marked_start,
            event.marked_end,
            event.edit_seq,
        )
    }
}

impl TryFrom<TextInputEventWire> for TextInputEvent {
    type Error = ProtocolError;

    fn try_from(event: TextInputEventWire) -> Result<Self, Self::Error> {
        if event.0 != 1
            || event.selection_start() > event.selection_end()
            || (event.4.is_some() != event.5.is_some())
            || event.4.zip(event.5).is_some_and(|(start, end)| start > end)
        {
            return Err(ProtocolError::InvalidTextInputEvent);
        }
        Ok(Self {
            text: event.1,
            selection_start: event.2,
            selection_end: event.3,
            marked_start: event.4,
            marked_end: event.5,
            edit_seq: event.6,
        })
    }
}

impl TextInputEventWire {
    fn selection_start(&self) -> u32 {
        self.2
    }

    fn selection_end(&self) -> u32 {
        self.3
    }
}

impl From<&CommandResult> for CommandResultWire {
    fn from(result: &CommandResult) -> Self {
        Self(
            2,
            result.request_id,
            result.command,
            result.node_id,
            result.success,
            result.error.clone(),
        )
    }
}

impl From<CommandResultWire> for CommandResult {
    fn from(result: CommandResultWire) -> Self {
        Self {
            request_id: result.1,
            command: result.2,
            node_id: result.3,
            success: result.4,
            error: result.5,
        }
    }
}

impl From<&Event> for EventWire {
    fn from(event: &Event) -> Self {
        Self(
            event.protocol,
            event.message,
            event.surface_id,
            event.epoch,
            event.revision,
            event.sequence,
            event.node_id,
            event.listener_id,
            event.event_type,
            event.payload.as_ref().map(EventPayloadWire::from),
        )
    }
}

impl From<&EventPayload> for EventPayloadWire {
    fn from(payload: &EventPayload) -> Self {
        match payload {
            EventPayload::TextInput(event) => Self::Text(TextInputEventWire::from(event)),
            EventPayload::CommandResult(result) => Self::Command(CommandResultWire::from(result)),
            EventPayload::VisibleRange { start, end } => {
                Self::Visible(VisibleRangeWire(3, *start, *end))
            }
            EventPayload::AnimationComplete { generation } => {
                Self::Animation(AnimationCompleteWire(4, *generation))
            }
        }
    }
}

impl From<&PatchOperation> for OperationWire {
    fn from(operation: &PatchOperation) -> Self {
        match operation {
            PatchOperation::Create(node) => Self::Create(CreateWire(
                1,
                node.id,
                node.parent_id,
                node.index,
                node.kind,
                node.style.as_ref().map(StyleWire::from),
                node.text.clone(),
                node.listener_id,
                node.host_properties.as_ref().map(HostPropertiesWire::from),
                node.accessibility.as_ref().map(AccessibilityWire::from),
            )),
            PatchOperation::Update {
                id,
                mask,
                style,
                text,
                listener_id,
                host_properties,
                accessibility,
            } => Self::Update(UpdateWire(
                2,
                *id,
                *mask,
                style.as_ref().map(StyleWire::from),
                text.clone(),
                *listener_id,
                host_properties.as_ref().map(HostPropertiesWire::from),
                accessibility.as_ref().map(AccessibilityWire::from),
            )),
            PatchOperation::Move {
                id,
                parent_id,
                index,
            } => Self::Move(MoveWire(3, *id, *parent_id, *index)),
            PatchOperation::Delete { id } => Self::Delete(DeleteWire(4, *id)),
        }
    }
}

impl TryFrom<OperationWire> for PatchOperation {
    type Error = ProtocolError;

    fn try_from(operation: OperationWire) -> Result<Self, Self::Error> {
        match operation {
            OperationWire::Create(wire) => {
                if wire.0 != 1 {
                    return Err(ProtocolError::UnknownPatchOperation(wire.0));
                }
                if let Some(style) = wire.5.as_ref() {
                    validate_style_wire(style)?;
                }
                let host_properties = wire.8.map(HostProperties::try_from).transpose()?;
                Ok(Self::Create(Node {
                    id: wire.1,
                    parent_id: wire.2,
                    index: wire.3,
                    kind: wire.4,
                    style: wire.5.map(Style::from),
                    text: wire.6,
                    listener_id: wire.7,
                    host_properties,
                    accessibility: wire.9.map(AccessibilityProperties::from),
                }))
            }
            OperationWire::Update(wire) => {
                if wire.0 != 2 {
                    return Err(ProtocolError::UnknownPatchOperation(wire.0));
                }
                if let Some(style) = wire.3.as_ref() {
                    validate_style_wire(style)?;
                }
                let host_properties = wire.6.map(HostProperties::try_from).transpose()?;
                Ok(Self::Update {
                    id: wire.1,
                    mask: wire.2,
                    style: wire.3.map(Style::from),
                    text: wire.4,
                    listener_id: wire.5,
                    host_properties,
                    accessibility: wire.7.map(AccessibilityProperties::from),
                })
            }
            OperationWire::Move(wire) => {
                if wire.0 != 3 {
                    return Err(ProtocolError::UnknownPatchOperation(wire.0));
                }
                Ok(Self::Move {
                    id: wire.1,
                    parent_id: wire.2,
                    index: wire.3,
                })
            }
            OperationWire::Delete(wire) => {
                if wire.0 != 4 {
                    return Err(ProtocolError::UnknownPatchOperation(wire.0));
                }
                Ok(Self::Delete { id: wire.1 })
            }
        }
    }
}
