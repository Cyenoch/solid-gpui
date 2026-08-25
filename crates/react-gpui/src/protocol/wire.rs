use std::fmt;
use std::io::Cursor;

use super::*;
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};

pub(super) fn encode_snapshot(snapshot: &Snapshot) -> Result<Vec<u8>, ProtocolError> {
    let wire = SnapshotWire(
        snapshot.protocol,
        snapshot.message,
        snapshot.surface_id,
        snapshot.epoch,
        snapshot.base_revision,
        snapshot.revision,
        snapshot.nodes.iter().map(NodeWire::from).collect(),
    );
    rmp_serde::to_vec(&wire).map_err(ProtocolError::Encode)
}

pub(super) fn decode_snapshot(payload: &[u8]) -> Result<Snapshot, ProtocolError> {
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
    Ok(Snapshot {
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

pub(super) fn encode_patch(patch: &Patch) -> Result<Vec<u8>, ProtocolError> {
    let wire = PatchWire(
        patch.protocol,
        patch.message,
        patch.surface_id,
        patch.epoch,
        patch.base_revision,
        patch.revision,
        patch.operations.iter().map(OperationWire::from).collect(),
    );
    rmp_serde::to_vec(&wire).map_err(ProtocolError::Encode)
}

pub(super) fn decode_patch(payload: &[u8]) -> Result<Patch, ProtocolError> {
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
    Ok(Patch {
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

pub(super) fn encode_command(command: &Command) -> Result<Vec<u8>, ProtocolError> {
    let payload = match command.kind {
        COMMAND_OPEN_SURFACE | COMMAND_FILE_DIALOG_OPEN => {
            if command.body.is_some() || command.menus.is_some() {
                return Err(ProtocolError::InvalidCommandPayload);
            }
            match (&command.payload, &command.title) {
                (Some(payload), Some(title)) => Some(CommandPayloadWire::StringWithPair((
                    title.clone(),
                    *payload,
                ))),
                _ => return Err(ProtocolError::InvalidCommandPayload),
            }
        }
        COMMAND_FILE_DIALOG_SAVE => {
            if command.body.is_some() || command.menus.is_some() {
                return Err(ProtocolError::InvalidCommandPayload);
            }
            match (&command.payload, &command.title) {
                (None, Some(default_name)) => Some(CommandPayloadWire::Title(default_name.clone())),
                _ => return Err(ProtocolError::InvalidCommandPayload),
            }
        }
        COMMAND_SHOW_NOTIFICATION => match (
            &command.payload,
            &command.title,
            &command.body,
            &command.menus,
        ) {
            (None, Some(title), Some(body), None) => Some(CommandPayloadWire::StringPair((
                title.clone(),
                body.clone(),
            ))),
            _ => return Err(ProtocolError::InvalidCommandPayload),
        },
        COMMAND_SET_MENUS => match (
            &command.payload,
            &command.title,
            &command.body,
            &command.menus,
        ) {
            (None, None, None, Some(menus)) => Some(CommandPayloadWire::Menus(
                menus.iter().map(MenuWire::from).collect(),
            )),
            _ => return Err(ProtocolError::InvalidCommandPayload),
        },
        _ => {
            if command.body.is_some() || command.menus.is_some() {
                return Err(ProtocolError::InvalidCommandPayload);
            }
            match (&command.payload, &command.title) {
                (Some(payload), None) => Some(CommandPayloadWire::Pair(*payload)),
                (None, Some(title)) => Some(CommandPayloadWire::Title(title.clone())),
                (None, None) => None,
                (Some(_), Some(_)) => return Err(ProtocolError::InvalidCommandPayload),
            }
        }
    };
    rmp_serde::to_vec(&CommandWire(
        command.protocol,
        command.message,
        command.surface_id,
        command.epoch,
        command.after_revision,
        command.request_id,
        command.node_id,
        command.kind,
        payload,
    ))
    .map_err(ProtocolError::Encode)
}

fn valid_http_url(url: &str) -> bool {
    if url.is_empty() || url.len() > 2048 || url.chars().any(char::is_whitespace) {
        return false;
    }
    url.strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .is_some_and(|host| !host.is_empty())
}

pub(super) fn decode_command(payload: &[u8]) -> Result<Command, ProtocolError> {
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
    ) {
        return Err(ProtocolError::UnknownCommand(wire.7));
    }
    // The untagged wire enum only describes shapes. The command kind selects
    // the meaning, so the same [string,[u32,u32]] shape is validated
    // independently for OpenSurface versus FileDialogOpen.
    let command_payload = wire.8.clone();
    let (payload, title, body): (Option<(u32, u32)>, Option<String>, Option<String>) =
        match (wire.7, wire.8) {
            (COMMAND_SET_TITLE, Some(CommandPayloadWire::Title(title)))
                if wire.6 == 1 && !title.is_empty() && title.chars().count() <= 256 =>
            {
                (None, Some(title), None)
            }
            (COMMAND_SET_TITLE, _) => return Err(ProtocolError::InvalidCommandPayload),
            (COMMAND_OPEN_SURFACE, Some(CommandPayloadWire::StringWithPair((title, size))))
                if wire.6 == 1
                    && title.chars().count() <= 256
                    && ((size.0 == 0 && size.1 == 0)
                        || (size.0 > 0
                            && size.0 <= MAX_WINDOW_DIMENSION
                            && size.1 > 0
                            && size.1 <= MAX_WINDOW_DIMENSION)) =>
            {
                (Some(size), Some(title), None)
            }
            (COMMAND_OPEN_SURFACE, _) => return Err(ProtocolError::InvalidCommandPayload),
            (
                COMMAND_FILE_DIALOG_OPEN,
                Some(CommandPayloadWire::StringWithPair((title, flags))),
            ) if wire.6 == 1 && title.chars().count() <= 256 && flags.0 <= 1 && flags.1 <= 1 => {
                (Some(flags), Some(title), None)
            }
            (COMMAND_FILE_DIALOG_OPEN, _) => return Err(ProtocolError::InvalidCommandPayload),
            (COMMAND_FILE_DIALOG_SAVE, Some(CommandPayloadWire::Title(default_name)))
                if wire.6 == 1 && default_name.chars().count() <= 256 =>
            {
                (None, Some(default_name), None)
            }
            (COMMAND_FILE_DIALOG_SAVE, _) => return Err(ProtocolError::InvalidCommandPayload),
            (COMMAND_SHOW_NOTIFICATION, Some(CommandPayloadWire::StringPair((title, body))))
                if wire.6 == 1 && title.len() <= 256 && body.len() <= 1024 =>
            {
                (None, Some(title), Some(body))
            }
            (COMMAND_SHOW_NOTIFICATION, _) => return Err(ProtocolError::InvalidCommandPayload),
            (COMMAND_SET_MENUS, Some(CommandPayloadWire::Menus(_))) if wire.6 == 1 => {
                (None, None, None)
            }
            (COMMAND_SET_MENUS, _) => return Err(ProtocolError::InvalidCommandPayload),
            (COMMAND_OPEN_URL, Some(CommandPayloadWire::Title(url)))
                if wire.6 == 1 && valid_http_url(&url) =>
            {
                (None, Some(url), None)
            }
            (COMMAND_OPEN_URL, _) => return Err(ProtocolError::InvalidCommandPayload),
            (COMMAND_CLIPBOARD_WRITE, Some(CommandPayloadWire::Title(text)))
                if wire.6 == 1 && text.len() <= MAX_CLIPBOARD_TEXT_BYTES =>
            {
                (None, Some(text), None)
            }
            (COMMAND_CLIPBOARD_WRITE, _) => return Err(ProtocolError::InvalidCommandPayload),
            (COMMAND_CLIPBOARD_READ, None) if wire.6 == 1 => (None, None, None),
            (COMMAND_CLIPBOARD_READ, _) => return Err(ProtocolError::InvalidCommandPayload),
            (COMMAND_RESIZE_WINDOW, Some(CommandPayloadWire::Pair(payload)))
                if wire.6 == 1
                    && payload.0 > 0
                    && payload.0 <= MAX_WINDOW_DIMENSION
                    && payload.1 > 0
                    && payload.1 <= MAX_WINDOW_DIMENSION =>
            {
                (Some(payload), None, None)
            }
            (COMMAND_RESIZE_WINDOW, _) => return Err(ProtocolError::InvalidCommandPayload),
            (COMMAND_ZOOM_WINDOW | COMMAND_TOGGLE_FULLSCREEN, None) if wire.6 == 1 => {
                (None, None, None)
            }
            (COMMAND_ZOOM_WINDOW | COMMAND_TOGGLE_FULLSCREEN, _) => {
                return Err(ProtocolError::InvalidCommandPayload);
            }
            (COMMAND_FOCUS_NEXT | COMMAND_FOCUS_PREV, None) if wire.6 == 1 => (None, None, None),
            (COMMAND_FOCUS_NEXT | COMMAND_FOCUS_PREV, _) => {
                return Err(ProtocolError::InvalidCommandPayload);
            }
            (COMMAND_GET_WINDOW_SIZE, None) if wire.6 == 1 => (None, None, None),
            (COMMAND_GET_WINDOW_SIZE, _) => return Err(ProtocolError::InvalidCommandPayload),
            (COMMAND_GET_FOCUS, None) => (None, None, None),
            (COMMAND_GET_FOCUS, _) => return Err(ProtocolError::InvalidCommandPayload),
            (COMMAND_FOCUS | COMMAND_BLUR | COMMAND_SCROLL_TO_END, None) => (None, None, None),
            (
                COMMAND_SET_SELECTION | COMMAND_SCROLL_TO_INDEX,
                Some(CommandPayloadWire::Pair(payload)),
            ) => {
                if wire.7 == COMMAND_SET_SELECTION && payload.0 > payload.1 {
                    return Err(ProtocolError::InvalidCommandPayload);
                }
                (Some(payload), None, None)
            }
            _ => return Err(ProtocolError::InvalidCommandPayload),
        };
    let menus = if wire.7 == COMMAND_SET_MENUS {
        match command_payload {
            Some(CommandPayloadWire::Menus(menus)) => Some(
                menus
                    .into_iter()
                    .map(MenuDefinition::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            _ => return Err(ProtocolError::InvalidCommandPayload),
        }
    } else {
        None
    };
    Ok(Command {
        protocol: wire.0,
        message: wire.1,
        surface_id: wire.2,
        epoch: wire.3,
        after_revision: wire.4,
        request_id: wire.5,
        node_id: wire.6,
        kind: wire.7,
        payload,
        title,
        body,
        menus,
    })
}

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
                        | COMMAND_CLIPBOARD_READ
                        | COMMAND_OPEN_SURFACE
                        | COMMAND_FILE_DIALOG_OPEN
                        | COMMAND_FILE_DIALOG_SAVE
                        | COMMAND_SHOW_NOTIFICATION
                        | COMMAND_SET_MENUS
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
            let (width, height) = value.dimensions();
            if !width.is_finite() || width < 0.0 || !height.is_finite() || height < 0.0 {
                return Err(ProtocolError::InvalidEventPayload);
            }
            Some(EventPayload::WindowResize { width, height })
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
        (EVENT_HOVER | EVENT_SUBMIT, None) => None,
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
#[derive(Debug, Serialize, Deserialize)]
struct SnapshotWire(u32, u32, u32, u32, u32, u32, Vec<NodeWire>);
#[derive(Debug, Serialize, Deserialize)]
struct PatchWire(u32, u32, u32, u32, u32, u32, Vec<OperationWire>);
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum CommandPayloadWire {
    Pair((u32, u32)),
    Title(String),
    StringPair((String, String)),
    StringWithPair((String, (u32, u32))),
    Menus(Vec<MenuWire>),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MenuWire(String, Vec<MenuItemWire>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum MenuItemWire {
    Separator((u32,)),
    Action((u32, String)),
    Submenu((u32, MenuWire)),
}
fn valid_menu_text(value: &str) -> bool {
    !value.is_empty() && value.chars().count() <= 256
}

impl From<&MenuDefinition> for MenuWire {
    fn from(menu: &MenuDefinition) -> Self {
        Self(
            menu.title.clone(),
            menu.items.iter().map(MenuItemWire::from).collect(),
        )
    }
}

impl From<&MenuItemDefinition> for MenuItemWire {
    fn from(item: &MenuItemDefinition) -> Self {
        match item {
            MenuItemDefinition::Separator => Self::Separator((0,)),
            MenuItemDefinition::Action(action) => Self::Action((1, action.clone())),
            MenuItemDefinition::Submenu(menu) => Self::Submenu((2, MenuWire::from(menu))),
        }
    }
}

impl TryFrom<MenuWire> for MenuDefinition {
    type Error = ProtocolError;

    fn try_from(menu: MenuWire) -> Result<Self, Self::Error> {
        if !valid_menu_text(&menu.0) {
            return Err(ProtocolError::InvalidCommandPayload);
        }
        Ok(Self {
            title: menu.0,
            items: menu
                .1
                .into_iter()
                .map(MenuItemDefinition::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<MenuItemWire> for MenuItemDefinition {
    type Error = ProtocolError;

    fn try_from(item: MenuItemWire) -> Result<Self, Self::Error> {
        match item {
            MenuItemWire::Separator((0,)) => Ok(Self::Separator),
            MenuItemWire::Action((1, action)) if valid_menu_text(&action) => {
                Ok(Self::Action(action))
            }
            MenuItemWire::Submenu((2, menu)) => Ok(Self::Submenu(MenuDefinition::try_from(menu)?)),
            _ => Err(ProtocolError::InvalidCommandPayload),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
struct CommandWire(
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    Option<CommandPayloadWire>,
);

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
    bool,
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
    bool,
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
    bool,
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
    Image(ImageWire),
    Drag(DragWire),
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
    Option<u32>,
);

#[derive(Debug, Serialize, Deserialize)]
struct VirtualListWire(u32, u32, u32, u32, f32, u32);
#[derive(Debug, Serialize, Deserialize)]
struct ImageWire(u32, String, u32);
#[derive(Debug, Serialize, Deserialize)]
struct DragWire(u32, Option<String>);
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
    Option<u32>,
    Option<u32>,
    Option<f32>,
    Option<f32>,
    Option<u32>,
    Option<f32>,
    Option<u32>,
    Option<u32>,
    Option<u32>,
    Option<u32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<u32>,
    Option<u32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<u32>,
    Option<u32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<f32>,
    Option<u32>,
);

#[derive(Debug, Serialize, Deserialize)]
struct TransitionWire(u32, u32, u32, u32);
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
    Layout((f32, f32, f32, f32)),
    Drag(DragPayloadWire),
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
    FloatFloat((f32, f32)),
    IntInt((u32, u32)),
    FloatInt((f32, u32)),
    IntFloat((u32, f32)),
}

impl WindowResizeWire {
    fn dimensions(self) -> (f32, f32) {
        match self {
            Self::FloatFloat((width, height)) => (width, height),
            Self::IntInt((width, height)) => (width as f32, height as f32),
            Self::FloatInt((width, height)) => (width, height as f32),
            Self::IntFloat((width, height)) => (width as f32, height),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TextInputEventWire(u32, String, u32, u32, Option<u32>, Option<u32>, u32);
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum CommandResultWire {
    New(
        (
            u32,
            u32,
            u32,
            u32,
            bool,
            Option<String>,
            Option<CommandValueWire>,
        ),
    ),
    Old((u32, u32, u32, u32, bool, Option<String>)),
}

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
            node.focusable,
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
            focusable: node.9,
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
            style.justify_content,
            style.align_items,
            style.border_radius,
            style.border_width,
            style.border_color_rgba,
            style.font_size,
            style.font_weight,
            style.overflow,
            style.line_clamp,
            style.text_overflow,
            style.margin_top,
            style.margin_right,
            style.margin_bottom,
            style.margin_left,
            style.font_style,
            style.text_decoration,
            style.line_height,
            style.min_width,
            style.max_width,
            style.min_height,
            style.max_height,
            style.flex_shrink,
            style.align_self,
            style.position,
            style.left,
            style.top,
            style.right,
            style.bottom,
            style.cursor,
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
            justify_content: style.10,
            align_items: style.11,
            border_radius: style.12,
            border_width: style.13,
            border_color_rgba: style.14,
            font_size: style.15,
            font_weight: style.16,
            overflow: style.17,
            line_clamp: style.18,
            text_overflow: style.19,
            margin_top: style.20,
            margin_right: style.21,
            margin_bottom: style.22,
            margin_left: style.23,
            font_style: style.24,
            text_decoration: style.25,
            line_height: style.26,
            min_width: style.27,
            max_width: style.28,
            min_height: style.29,
            max_height: style.30,
            flex_shrink: style.31,
            align_self: style.32,
            position: style.33,
            left: style.34,
            top: style.35,
            right: style.36,
            bottom: style.37,
            cursor: style.38,
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
            HostProperties::Image(value) => Self::Image(ImageWire::from(value)),
            HostProperties::Drag(value) => Self::Drag(DragWire(4, value.drag_type.clone())),
        }
    }
}

fn valid_image_source(source: &str) -> bool {
    !source.is_empty() && source.len() <= 1024 && !source.chars().any(char::is_control)
}

fn valid_drag_type(drag_type: Option<&str>) -> bool {
    drag_type.is_none_or(|value| {
        !value.is_empty() && value.chars().count() <= 128 && !value.chars().any(char::is_control)
    })
}
fn valid_external_path(path: &str) -> bool {
    !path.is_empty() && path.len() <= 4096 && !path.chars().any(char::is_control)
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
            HostPropertiesWire::Image(value)
                if value.0 == 3 && valid_image_source(&value.1) && (1..=5).contains(&value.2) =>
            {
                Ok(Self::Image(ImageProperties {
                    source: value.1,
                    object_fit: value.2,
                }))
            }
            HostPropertiesWire::Drag(value)
                if value.0 == 4 && valid_drag_type(value.1.as_deref()) =>
            {
                Ok(Self::Drag(DragProperties { drag_type: value.1 }))
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
        (1 | 3, Some(HostProperties::Drag(_)))
        | (5, Some(HostProperties::TextInput(_)))
        | (6, Some(HostProperties::VirtualList(_)))
        | (7, Some(HostProperties::Image(_))) => Ok(()),
        (5..=7, None) => Err(ProtocolError::InvalidHostProperties),
        (_, Some(_)) => Err(ProtocolError::InvalidHostProperties),
        _ => Ok(()),
    }
}

fn validate_style_wire(style: &StyleWire) -> Result<(), ProtocolError> {
    if style.2.is_some_and(|direction| direction > 2)
        || style.33.is_some_and(|position| position > 1)
        || style.38.is_some_and(|cursor| cursor > 18)
        || [style.34, style.35, style.36, style.37]
            .into_iter()
            .flatten()
            .any(|value| !value.is_finite())
        || style.10.is_some_and(|justify| !(1..=6).contains(&justify))
        || style.11.is_some_and(|align| !(1..=5).contains(&align))
        || style
            .17
            .is_some_and(|overflow| !(1..=3).contains(&overflow))
        || style
            .18
            .is_some_and(|line_clamp| !(1..=100).contains(&line_clamp))
        || style
            .19
            .is_some_and(|text_overflow| !matches!(text_overflow, 1 | 2))
        || style
            .24
            .is_some_and(|font_style| !matches!(font_style, 0 | 1))
        || style
            .25
            .is_some_and(|text_decoration| !matches!(text_decoration, 0..=2))
        || style
            .32
            .is_some_and(|align_self| !(1..=7).contains(&align_self))
        || [
            style.0, style.1, style.3, style.4, style.5, style.8, style.12, style.13, style.20,
            style.21, style.22, style.23, style.26, style.27, style.28, style.29, style.30,
            style.31,
        ]
        .into_iter()
        .flatten()
        .any(|value| !value.is_finite() || value < 0.0)
        || style.8.is_some_and(|opacity| opacity > 1.0)
        || style
            .15
            .is_some_and(|size| !size.is_finite() || size <= 0.0)
        || style
            .16
            .is_some_and(|weight| !matches!(weight, 400 | 500 | 600 | 700 | 900))
    {
        return Err(ProtocolError::InvalidStyle);
    }
    if let Some(transition) = style.9.as_ref()
        && (transition.2 > 3
            || transition.3 == 0
            || transition.3
                & !(TRANSITION_OPACITY
                    | TRANSITION_BACKGROUND_COLOR
                    | TRANSITION_WIDTH
                    | TRANSITION_HEIGHT)
                != 0)
    {
        return Err(ProtocolError::InvalidStyle);
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
            value.max_length,
        )
    }
}

impl From<&ImageProperties> for ImageWire {
    fn from(value: &ImageProperties) -> Self {
        Self(3, value.source.clone(), value.object_fit)
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
            max_length: value.11,
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
            event.click_count,
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
        match self {
            Self::New(value) => value.0,
            Self::Old(value) => value.0,
        }
    }

    fn command(&self) -> u32 {
        match self {
            Self::New(value) => value.2,
            Self::Old(value) => value.2,
        }
    }
}

impl From<&CommandResult> for CommandResultWire {
    fn from(result: &CommandResult) -> Self {
        Self::New((
            2,
            result.request_id,
            result.command,
            result.node_id,
            result.success,
            result.error.clone(),
            result.value.as_ref().map(CommandValueWire::from),
        ))
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
        let (request_id, command, node_id, success, error, value) = match result {
            CommandResultWire::New((_, request_id, command, node_id, success, error, value)) => {
                (request_id, command, node_id, success, error, value)
            }
            CommandResultWire::Old((_, request_id, command, node_id, success, error)) => {
                (request_id, command, node_id, success, error, None)
            }
        };
        Ok(Self {
            request_id,
            command,
            node_id,
            success,
            error,
            value: value.map(CommandValue::try_from).transpose()?,
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
            EventPayload::WindowResize { width, height } => {
                Self::WindowResize(WindowResizeWire::FloatFloat((*width, *height)))
            }
            EventPayload::WindowActivation { active } => Self::WindowActivation(*active),
            EventPayload::EventAction { action } => Self::Action(action.clone()),
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
                node.focusable,
            )),
            PatchOperation::Update {
                id,
                mask,
                style,
                text,
                listener_id,
                host_properties,
                accessibility,
                focusable,
            } => Self::Update(UpdateWire(
                2,
                *id,
                *mask,
                style.as_ref().map(StyleWire::from),
                text.clone(),
                *listener_id,
                host_properties.as_ref().map(HostPropertiesWire::from),
                accessibility.as_ref().map(AccessibilityWire::from),
                *focusable,
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
                    focusable: wire.10,
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
                    focusable: wire.8,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_surface_round_trips_title_and_size() {
        let command = Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 1,
            epoch: 2,
            after_revision: 3,
            request_id: 4,
            node_id: 1,
            kind: COMMAND_OPEN_SURFACE,
            payload: Some((640, 480)),
            title: Some("child".to_owned()),
            body: None,
            menus: None,
        };
        let decoded = Command::decode(&command.encode().expect("encode open surface"))
            .expect("decode open surface");
        assert_eq!(decoded, command);
    }

    #[test]
    fn open_surface_requires_root_and_valid_size() {
        let command = Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 1,
            epoch: 2,
            after_revision: 3,
            request_id: 4,
            node_id: 2,
            kind: COMMAND_OPEN_SURFACE,
            payload: Some((640, 480)),
            title: Some(String::new()),
            body: None,
            menus: None,
        };
        let encoded = rmp_serde::to_vec(&CommandWire(
            command.protocol,
            command.message,
            command.surface_id,
            command.epoch,
            command.after_revision,
            command.request_id,
            command.node_id,
            command.kind,
            Some(CommandPayloadWire::StringWithPair((
                command.title.clone().expect("title"),
                command.payload.expect("size"),
            ))),
        ))
        .expect("encode invalid root command");
        assert!(Command::decode(&encoded).is_err());

        let command = Command {
            node_id: 1,
            payload: Some((0, 640)),
            ..command
        };
        let encoded = rmp_serde::to_vec(&CommandWire(
            command.protocol,
            command.message,
            command.surface_id,
            command.epoch,
            command.after_revision,
            command.request_id,
            command.node_id,
            command.kind,
            Some(CommandPayloadWire::StringWithPair((
                command.title.clone().expect("title"),
                command.payload.expect("size"),
            ))),
        ))
        .expect("encode invalid size command");
        assert!(Command::decode(&encoded).is_err());
    }

    #[test]
    fn file_dialog_commands_use_kind_specific_payloads() {
        let open = Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 1,
            epoch: 2,
            after_revision: 3,
            request_id: 5,
            node_id: 1,
            kind: COMMAND_FILE_DIALOG_OPEN,
            payload: Some((1, 1)),
            title: Some("Choose".to_owned()),
            body: None,
            menus: None,
        };
        assert_eq!(
            Command::decode(&open.encode().expect("encode open dialog"))
                .expect("decode open dialog"),
            open
        );

        let save = Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 1,
            epoch: 2,
            after_revision: 3,
            request_id: 6,
            node_id: 1,
            kind: COMMAND_FILE_DIALOG_SAVE,
            payload: None,
            title: Some("report.txt".to_owned()),
            body: None,
            menus: None,
        };
        assert_eq!(
            Command::decode(&save.encode().expect("encode save dialog"))
                .expect("decode save dialog"),
            save
        );

        let invalid_open = Command {
            payload: Some((2, 0)),
            ..open
        };
        assert!(
            Command::decode(&invalid_open.encode().expect("encode invalid open dialog")).is_err()
        );
    }

    #[test]
    fn notification_and_menu_commands_and_action_events_round_trip() {
        let notification = Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 1,
            epoch: 2,
            after_revision: 3,
            request_id: 7,
            node_id: 1,
            kind: COMMAND_SHOW_NOTIFICATION,
            payload: None,
            title: Some("Done".to_owned()),
            body: Some("Finished".to_owned()),
            menus: None,
        };
        assert_eq!(
            Command::decode(&notification.encode().expect("encode notification"))
                .expect("decode notification"),
            notification
        );

        let menus = Command {
            protocol: PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 1,
            epoch: 2,
            after_revision: 3,
            request_id: 8,
            node_id: 1,
            kind: COMMAND_SET_MENUS,
            payload: None,
            title: None,
            body: None,
            menus: Some(vec![MenuDefinition {
                title: "File".to_owned(),
                items: vec![
                    MenuItemDefinition::Action("open".to_owned()),
                    MenuItemDefinition::Separator,
                    MenuItemDefinition::Submenu(MenuDefinition {
                        title: "More".to_owned(),
                        items: vec![MenuItemDefinition::Action("other".to_owned())],
                    }),
                ],
            }]),
        };
        assert_eq!(
            Command::decode(&menus.encode().expect("encode menus")).expect("decode menus"),
            menus
        );

        let action = Event::action(1, 2, 3, 4, "open".to_owned());
        assert_eq!(
            Event::decode(&action.encode().expect("encode action")).unwrap(),
            action
        );
    }

    #[test]
    fn surface_closed_event_round_trips_without_payload() {
        let event = Event::surface_closed(7, 8, 9, 10);
        let decoded = Event::decode(&event.encode().expect("encode close event"))
            .expect("decode close event");
        assert_eq!(decoded, event);
        assert_eq!(decoded.node_id, 0);
        assert_eq!(decoded.listener_id, 0);
        assert_eq!(decoded.payload, None);
        let mut invalid = event;
        invalid.node_id = 1;
        assert!(Event::decode(&invalid.encode().expect("encode invalid close event")).is_err());
    }
}
