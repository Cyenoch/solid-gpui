use std::fmt;
use std::io::Cursor;

use super::node::valid_drag_type;
use super::*;
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};

pub(super) fn encode_event(event: &Event) -> Result<Vec<u8>, ProtocolError> {
    rmp_serde::to_vec(&EventWire::from(event)).map_err(ProtocolError::Encode)
}

pub(super) fn decode_event(payload: &[u8]) -> Result<Event, ProtocolError> {
    let mut deserializer = rmp_serde::Deserializer::new(Cursor::new(payload));
    let wire = EventWire::deserialize(&mut deserializer).map_err(ProtocolError::Decode)?;
    let consumed = deserializer.into_inner().position() as usize;
    if consumed != payload.len() {
        return Err(ProtocolError::TrailingBytes(payload.len() - consumed));
    }
    if wire.0 != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedProtocol {
            received: wire.0,
            expected: PROTOCOL_VERSION,
        });
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
            | EVENT_KEY
            | EVENT_POINTER
            | EVENT_HOVER
            | EVENT_SCROLL
            | EVENT_SUBMIT
            | EVENT_WINDOW_RESIZE
            | EVENT_WINDOW_ACTIVATION
            | EVENT_SURFACE_CLOSED
            | EVENT_ACTION
            | EVENT_WINDOW_APPEARANCE
            | EVENT_LAYOUT
            | EVENT_DRAG
            | EVENT_NOTIFICATION_RESPONSE
            | EVENT_POINTER_DOWN_OUTSIDE
            | EVENT_CLOSE_REQUESTED
    ) {
        return Err(ProtocolError::UnknownEvent(wire.8));
    }
    let payload = match (wire.8, wire.9) {
        (EVENT_PRESS, None) => None,
        (
            EVENT_CHANGE | EVENT_SELECTION | EVENT_FOCUS | EVENT_BLUR,
            Some(EventPayloadWire::Text(value)),
        ) => Some(EventPayload::TextInput(TextInputEvent::try_from(value)?)),
        (EVENT_FOCUS | EVENT_BLUR, None) if wire.6 != 0 && wire.7 != 0 => None,
        (EVENT_COMMAND_RESULT, Some(EventPayloadWire::Command(value))) => {
            if value.tag() != 2
                || !matches!(
                    value.command(),
                    COMMAND_FOCUS
                        | COMMAND_BLUR
                        | COMMAND_SET_SELECTION
                        | COMMAND_SCROLL_TO_INDEX
                        | COMMAND_SCROLL_TO_END
                        | COMMAND_SET_TITLE
                        | COMMAND_RESIZE_WINDOW
                        | COMMAND_ZOOM_WINDOW
                        | COMMAND_TOGGLE_FULLSCREEN
                        | COMMAND_OPEN_URL
                        | COMMAND_FOCUS_NEXT
                        | COMMAND_FOCUS_PREV
                        | COMMAND_GET_WINDOW_SIZE
                        | COMMAND_GET_FOCUS
                        | COMMAND_CLIPBOARD_WRITE
                        | COMMAND_CLIPBOARD_READ
                        | COMMAND_OPEN_SURFACE
                        | COMMAND_FILE_DIALOG_OPEN
                        | COMMAND_FILE_DIALOG_SAVE
                        | COMMAND_SHOW_NOTIFICATION
                        | COMMAND_SET_MENUS
                        | COMMAND_SET_KEYBINDINGS
                        | COMMAND_SET_CLOSE_POLICY
                        | COMMAND_RESOLVE_CLOSE_REQUEST
                        | COMMAND_READ_TEXT_FILE
                        | COMMAND_WRITE_TEXT_FILE
                )
            {
                return Err(ProtocolError::InvalidEventPayload);
            }
            Some(EventPayload::CommandResult(CommandResult::try_from(value)?))
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
        (EVENT_KEY, Some(EventPayloadWire::Key(value))) => {
            Some(EventPayload::Key(KeyEvent::try_from(value)?))
        }
        (EVENT_POINTER, Some(EventPayloadWire::Pointer(value))) => {
            Some(EventPayload::Pointer(PointerEvent::try_from(value)?))
        }
        (EVENT_SCROLL, Some(EventPayloadWire::Scroll(value))) => {
            Some(EventPayload::Scroll(ScrollEvent::try_from(value)?))
        }
        (EVENT_WINDOW_RESIZE, Some(EventPayloadWire::WindowResize(value))) => {
            let (width, height, scale_factor) = value.dimensions();
            if !width.is_finite()
                || width < 0.0
                || !height.is_finite()
                || height < 0.0
                || !scale_factor.is_finite()
                || scale_factor <= 0.0
            {
                return Err(ProtocolError::InvalidEventPayload);
            }
            Some(EventPayload::WindowResize {
                width,
                height,
                scale_factor,
            })
        }
        (EVENT_WINDOW_ACTIVATION, Some(EventPayloadWire::WindowActivation(active))) => {
            Some(EventPayload::WindowActivation { active })
        }
        (EVENT_ACTION, Some(EventPayloadWire::Action(action)))
            if wire.6 == 1
                && wire.7 == 0
                && !action.is_empty()
                && action.chars().count() <= 256 =>
        {
            Some(EventPayload::EventAction { action })
        }
        (EVENT_NOTIFICATION_RESPONSE, Some(EventPayloadWire::Notification((tag, action_id))))
            if wire.6 == 1
                && wire.7 == 0
                && !tag.is_empty()
                && tag.chars().count() <= 256
                && action_id
                    .as_ref()
                    .is_none_or(|action| !action.is_empty() && action.len() <= 64) =>
        {
            Some(EventPayload::NotificationResponse(
                NotificationResponseEvent { tag, action_id },
            ))
        }
        (EVENT_WINDOW_APPEARANCE, Some(EventPayloadWire::WindowAppearance(appearance)))
            if wire.6 == 1 && wire.7 == 0 && matches!(appearance.as_str(), "light" | "dark") =>
        {
            let appearance = if appearance == "dark" {
                WindowAppearance::Dark
            } else {
                WindowAppearance::Light
            };
            Some(EventPayload::WindowAppearance { appearance })
        }
        (EVENT_POINTER_DOWN_OUTSIDE, Some(EventPayloadWire::PointerDownOutside(value)))
            if wire.6 != 0
                && wire.7 != 0
                && value.0 == 8
                && value.1.is_finite()
                && value.2.is_finite() =>
        {
            Some(EventPayload::PointerDownOutside {
                x: value.1,
                y: value.2,
            })
        }
        (EVENT_CLOSE_REQUESTED, Some(EventPayloadWire::CloseRequested((tag, request_id))))
            if wire.6 == 1 && wire.7 == 0 && tag == 9 =>
        {
            Some(EventPayload::CloseRequested { request_id })
        }
        (EVENT_LAYOUT, Some(EventPayloadWire::Layout((x, y, width, height))))
            if wire.6 != 0
                && wire.7 != 0
                && [x, y, width, height].into_iter().all(f32::is_finite) =>
        {
            Some(EventPayload::Layout {
                x,
                y,
                width,
                height,
            })
        }
        (EVENT_DRAG, Some(EventPayloadWire::Drag(payload))) if wire.6 != 0 && wire.7 != 0 => {
            match payload {
                DragPayloadWire::Text((1, drag_type)) if valid_drag_type(Some(&drag_type)) => {
                    Some(EventPayload::DragOver { drag_type })
                }
                DragPayloadWire::Text((2, drag_type)) if valid_drag_type(Some(&drag_type)) => {
                    Some(EventPayload::DragDrop { drag_type })
                }
                DragPayloadWire::Paths((3, paths))
                    if !paths.is_empty()
                        && paths.len() <= 256
                        && paths.iter().all(|path| valid_external_path(path)) =>
                {
                    Some(EventPayload::ExternalFileDrop { paths })
                }
                _ => return Err(ProtocolError::InvalidEventPayload),
            }
        }
        (EVENT_SUBMIT, Some(EventPayloadWire::Submit(text))) => Some(EventPayload::Submit { text }),
        (EVENT_SURFACE_CLOSED, None) if wire.6 == 0 && wire.7 == 0 => None,
        (EVENT_HOVER, None) => None,
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
#[derive(Debug, Serialize)]
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
struct EventWireVisitor;

impl<'de> Visitor<'de> for EventWireVisitor {
    type Value = EventWire;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a ten-field event array")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let protocol = sequence
            .next_element()?
            .ok_or_else(|| de::Error::custom("missing event protocol"))?;
        let message = sequence
            .next_element()?
            .ok_or_else(|| de::Error::custom("missing event message"))?;
        let surface_id = sequence
            .next_element()?
            .ok_or_else(|| de::Error::custom("missing event surface"))?;
        let epoch = sequence
            .next_element()?
            .ok_or_else(|| de::Error::custom("missing event epoch"))?;
        let revision = sequence
            .next_element()?
            .ok_or_else(|| de::Error::custom("missing event revision"))?;
        let sequence_number = sequence
            .next_element()?
            .ok_or_else(|| de::Error::custom("missing event sequence"))?;
        let node_id = sequence
            .next_element()?
            .ok_or_else(|| de::Error::custom("missing event node"))?;
        let listener_id = sequence
            .next_element()?
            .ok_or_else(|| de::Error::custom("missing event listener"))?;
        let event_type: u32 = sequence
            .next_element()?
            .ok_or_else(|| de::Error::custom("missing event type"))?;
        macro_rules! typed_payload {
            ($type:ty, $variant:path) => {
                sequence
                    .next_element::<Option<$type>>()?
                    .flatten()
                    .map($variant)
            };
        }
        let payload = match event_type {
            EVENT_CHANGE | EVENT_SELECTION | EVENT_FOCUS | EVENT_BLUR => {
                typed_payload!(TextInputEventWire, EventPayloadWire::Text)
            }
            EVENT_COMMAND_RESULT => typed_payload!(CommandResultWire, EventPayloadWire::Command),
            EVENT_VISIBLE_RANGE => typed_payload!(VisibleRangeWire, EventPayloadWire::Visible),
            EVENT_ANIMATION_COMPLETE => {
                typed_payload!(AnimationCompleteWire, EventPayloadWire::Animation)
            }
            EVENT_KEY => typed_payload!(KeyEventWire, EventPayloadWire::Key),
            EVENT_POINTER => typed_payload!(PointerEventWire, EventPayloadWire::Pointer),
            EVENT_SCROLL => typed_payload!(ScrollEventWire, EventPayloadWire::Scroll),
            EVENT_WINDOW_RESIZE => {
                typed_payload!(WindowResizeWire, EventPayloadWire::WindowResize)
            }
            EVENT_WINDOW_ACTIVATION => sequence
                .next_element::<Option<bool>>()?
                .flatten()
                .map(EventPayloadWire::WindowActivation),
            EVENT_LAYOUT => sequence
                .next_element::<Option<(f32, f32, f32, f32)>>()?
                .flatten()
                .map(EventPayloadWire::Layout),
            EVENT_DRAG => sequence
                .next_element::<Option<DragPayloadWire>>()?
                .flatten()
                .map(EventPayloadWire::Drag),
            EVENT_SUBMIT => sequence
                .next_element::<Option<String>>()?
                .flatten()
                .map(EventPayloadWire::Submit),
            EVENT_ACTION => sequence
                .next_element::<Option<String>>()?
                .flatten()
                .map(EventPayloadWire::Action),
            EVENT_WINDOW_APPEARANCE => sequence
                .next_element::<Option<String>>()?
                .flatten()
                .map(EventPayloadWire::WindowAppearance),
            EVENT_NOTIFICATION_RESPONSE => sequence
                .next_element::<Option<(String, Option<String>)>>()?
                .flatten()
                .map(EventPayloadWire::Notification),
            EVENT_POINTER_DOWN_OUTSIDE => {
                typed_payload!(PointerDownOutsideWire, EventPayloadWire::PointerDownOutside)
            }
            EVENT_CLOSE_REQUESTED => sequence
                .next_element::<Option<(u32, u32)>>()?
                .flatten()
                .map(EventPayloadWire::CloseRequested),
            EVENT_PRESS | EVENT_HOVER => {
                let payload: Option<Option<de::IgnoredAny>> = sequence.next_element()?;
                if payload.flatten().is_some() {
                    return Err(de::Error::custom("event payload must be null"));
                }
                None
            }
            _ => {
                let _: Option<de::IgnoredAny> = sequence.next_element()?;
                None
            }
        };
        Ok(EventWire(
            protocol,
            message,
            surface_id,
            epoch,
            revision,
            sequence_number,
            node_id,
            listener_id,
            event_type,
            payload,
        ))
    }
}

impl<'de> Deserialize<'de> for EventWire {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(10, EventWireVisitor)
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum EventPayloadWire {
    WindowResize(WindowResizeWire),
    Text(TextInputEventWire),
    Command(CommandResultWire),
    Visible(VisibleRangeWire),
    Animation(AnimationCompleteWire),
    Key(KeyEventWire),
    Pointer(PointerEventWire),
    Scroll(ScrollEventWire),
    Submit(String),
    WindowActivation(bool),
    Action(String),
    WindowAppearance(String),
    Notification((String, Option<String>)),
    Layout((f32, f32, f32, f32)),
    Drag(DragPayloadWire),
    PointerDownOutside(PointerDownOutsideWire),
    CloseRequested((u32, u32)),
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum DragPayloadWire {
    Text((u32, String)),
    Paths((u32, Vec<String>)),
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum WindowResizeWire {
    AllFloat((f32, f32, f32)),
    IntegerPair((u32, u32, f32)),
    WidthFloat((f32, u32, f32)),
    HeightFloat((u32, f32, f32)),
}

impl WindowResizeWire {
    fn dimensions(self) -> (f32, f32, f32) {
        match self {
            Self::AllFloat((width, height, scale_factor)) => (width, height, scale_factor),
            Self::IntegerPair((width, height, scale_factor)) => {
                (width as f32, height as f32, scale_factor)
            }
            Self::WidthFloat((width, height, scale_factor)) => (width, height as f32, scale_factor),
            Self::HeightFloat((width, height, scale_factor)) => {
                (width as f32, height, scale_factor)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TextInputEventWire(u32, String, u32, u32, Option<u32>, Option<u32>, u32, bool);
#[derive(Debug, Serialize, Deserialize)]
struct CommandResultWire(
    u32,
    u32,
    u32,
    u32,
    bool,
    Option<String>,
    Option<CommandValueWire>,
);

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum CommandValueWire {
    Number((u32, f32)),
    Pair((u32, (f32, f32))),
    Bool((u32, bool)),
    Text((u32, String)),
    Paths((u32, Vec<String>)),
}
#[derive(Debug, Serialize, Deserialize)]
struct VisibleRangeWire(u32, u32, u32);
#[derive(Debug, Serialize, Deserialize)]
struct KeyEventWire(u32, String, Vec<String>, u32);
#[derive(Debug, Serialize, Deserialize)]
struct AnimationCompleteWire(u32, u32);
#[derive(Debug, Serialize, Deserialize)]
struct PointerEventWire(u32, u32, Vec<String>, u32, u32);
#[derive(Debug, Serialize, Deserialize)]
struct ScrollEventWire(u32, u32, f32, f32, f32, f32, Vec<String>);
#[derive(Debug, Serialize, Deserialize)]
struct PointerDownOutsideWire(u32, f32, f32);

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
            event.reversed,
        )
    }
}

impl TryFrom<TextInputEventWire> for TextInputEvent {
    type Error = ProtocolError;

    fn try_from(event: TextInputEventWire) -> Result<Self, Self::Error> {
        if event.0 != 1
            || event.2 > event.3
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
            reversed: event.7,
        })
    }
}
impl From<&KeyEvent> for KeyEventWire {
    fn from(event: &KeyEvent) -> Self {
        let action = match event.action {
            KeyAction::Down => EVENT_KEY_DOWN,
            KeyAction::Repeat => EVENT_KEY_REPEAT,
            KeyAction::Up => EVENT_KEY_UP,
        };
        Self(5, event.key.clone(), event.modifiers.clone(), action)
    }
}

impl TryFrom<KeyEventWire> for KeyEvent {
    type Error = ProtocolError;

    fn try_from(event: KeyEventWire) -> Result<Self, Self::Error> {
        if event.0 != 5
            || event.1.is_empty()
            || event.2.iter().enumerate().any(|(index, modifier)| {
                !matches!(
                    modifier.as_str(),
                    "cmd" | "ctrl" | "alt" | "shift" | "function"
                ) || event.2[..index].contains(modifier)
            })
        {
            return Err(ProtocolError::InvalidEventPayload);
        }
        let action = match event.3 {
            EVENT_KEY_DOWN => KeyAction::Down,
            EVENT_KEY_REPEAT => KeyAction::Repeat,
            EVENT_KEY_UP => KeyAction::Up,
            _ => return Err(ProtocolError::InvalidEventPayload),
        };
        Ok(Self {
            key: event.1,
            modifiers: event.2,
            action,
        })
    }
}
impl TryFrom<PointerEventWire> for PointerEvent {
    type Error = ProtocolError;

    fn try_from(event: PointerEventWire) -> Result<Self, Self::Error> {
        if event.0 != 6
            || !matches!(
                event.1,
                POINTER_BUTTON_LEFT
                    | POINTER_BUTTON_RIGHT
                    | POINTER_BUTTON_MIDDLE
                    | POINTER_BUTTON_BACK
                    | POINTER_BUTTON_FORWARD
            )
            || event.2.iter().enumerate().any(|(index, modifier)| {
                !matches!(
                    modifier.as_str(),
                    "cmd" | "ctrl" | "alt" | "shift" | "function"
                ) || event.2[..index].contains(modifier)
            })
            || !matches!(event.3, EVENT_POINTER_DOWN | EVENT_POINTER_UP)
            || event.4 == 0
        {
            return Err(ProtocolError::InvalidEventPayload);
        }
        Ok(Self {
            button: event.1,
            modifiers: event.2,
            action: event.3,
            click_count: event.4,
        })
    }
}

impl From<&PointerEvent> for PointerEventWire {
    fn from(event: &PointerEvent) -> Self {
        Self(
            6,
            event.button,
            event.modifiers.clone(),
            event.action,
            event.click_count.max(1),
        )
    }
}
impl TryFrom<ScrollEventWire> for ScrollEvent {
    type Error = ProtocolError;

    fn try_from(event: ScrollEventWire) -> Result<Self, Self::Error> {
        if event.0 != 7
            || !matches!(event.1, SCROLL_DELTA_PIXELS | SCROLL_DELTA_LINES)
            || [event.2, event.3, event.4, event.5]
                .into_iter()
                .any(|value| !value.is_finite())
            || event.6.iter().enumerate().any(|(index, modifier)| {
                !matches!(
                    modifier.as_str(),
                    "cmd" | "ctrl" | "alt" | "shift" | "function"
                ) || event.6[..index].contains(modifier)
            })
        {
            return Err(ProtocolError::InvalidEventPayload);
        }
        Ok(Self {
            delta_kind: event.1,
            dx: event.2,
            dy: event.3,
            x: event.4,
            y: event.5,
            modifiers: event.6,
        })
    }
}
impl From<&ScrollEvent> for ScrollEventWire {
    fn from(event: &ScrollEvent) -> Self {
        Self(
            7,
            event.delta_kind,
            event.dx,
            event.dy,
            event.x,
            event.y,
            event.modifiers.clone(),
        )
    }
}

impl CommandResultWire {
    fn tag(&self) -> u32 {
        self.0
    }

    fn command(&self) -> u32 {
        self.2
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
            result.value.as_ref().map(CommandValueWire::from),
        )
    }
}

impl From<&CommandValue> for CommandValueWire {
    fn from(value: &CommandValue) -> Self {
        match value {
            CommandValue::Number(number) => Self::Number((1, *number)),
            CommandValue::Pair((width, height)) => Self::Pair((2, (*width, *height))),
            CommandValue::Bool(active) => Self::Bool((3, *active)),
            CommandValue::Text(text) => Self::Text((4, text.clone())),
            CommandValue::Paths(paths) => Self::Paths((5, paths.clone())),
            CommandValue::FileText(text) => Self::Text((6, text.clone())),
        }
    }
}

impl TryFrom<CommandValueWire> for CommandValue {
    type Error = ProtocolError;

    fn try_from(value: CommandValueWire) -> Result<Self, Self::Error> {
        match value {
            CommandValueWire::Number((1, number)) if number.is_finite() => Ok(Self::Number(number)),
            CommandValueWire::Pair((2, (width, height)))
                if width.is_finite() && width >= 0.0 && height.is_finite() && height >= 0.0 =>
            {
                Ok(Self::Pair((width, height)))
            }
            CommandValueWire::Bool((3, active)) => Ok(Self::Bool(active)),
            CommandValueWire::Text((4, text)) if text.len() <= MAX_CLIPBOARD_TEXT_BYTES => {
                Ok(Self::Text(text))
            }
            CommandValueWire::Text((6, text)) if text.len() <= MAX_FILE_READ_BYTES => {
                Ok(Self::FileText(text))
            }
            CommandValueWire::Paths((5, paths))
                if !paths.is_empty() && paths.iter().all(|path| !path.is_empty()) =>
            {
                Ok(Self::Paths(paths))
            }
            _ => Err(ProtocolError::InvalidEventPayload),
        }
    }
}

impl TryFrom<CommandResultWire> for CommandResult {
    type Error = ProtocolError;

    fn try_from(result: CommandResultWire) -> Result<Self, Self::Error> {
        Ok(Self {
            request_id: result.1,
            command: result.2,
            node_id: result.3,
            success: result.4,
            error: result.5,
            value: result.6.map(CommandValue::try_from).transpose()?,
        })
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
            EventPayload::Key(event) => Self::Key(KeyEventWire::from(event)),
            EventPayload::Pointer(event) => Self::Pointer(PointerEventWire::from(event)),
            EventPayload::Scroll(event) => Self::Scroll(ScrollEventWire::from(event)),
            EventPayload::Submit { text } => Self::Submit(text.clone()),
            EventPayload::WindowResize {
                width,
                height,
                scale_factor,
            } => Self::WindowResize(WindowResizeWire::AllFloat((*width, *height, *scale_factor))),
            EventPayload::WindowActivation { active } => Self::WindowActivation(*active),
            EventPayload::EventAction { action } => Self::Action(action.clone()),
            EventPayload::NotificationResponse(response) => {
                Self::Notification((response.tag.clone(), response.action_id.clone()))
            }
            EventPayload::WindowAppearance { appearance } => {
                Self::WindowAppearance(appearance.as_str().to_owned())
            }
            EventPayload::Layout {
                x,
                y,
                width,
                height,
            } => Self::Layout((*x, *y, *width, *height)),
            EventPayload::DragOver { drag_type } => {
                Self::Drag(DragPayloadWire::Text((1, drag_type.clone())))
            }
            EventPayload::DragDrop { drag_type } => {
                Self::Drag(DragPayloadWire::Text((2, drag_type.clone())))
            }
            EventPayload::ExternalFileDrop { paths } => {
                Self::Drag(DragPayloadWire::Paths((3, paths.clone())))
            }
            EventPayload::PointerDownOutside { x, y } => {
                Self::PointerDownOutside(PointerDownOutsideWire(8, *x, *y))
            }
            EventPayload::CloseRequested { request_id } => Self::CloseRequested((9, *request_id)),
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
fn valid_external_path(path: &str) -> bool {
    !path.is_empty() && path.len() <= 4096 && !path.chars().any(char::is_control)
}
