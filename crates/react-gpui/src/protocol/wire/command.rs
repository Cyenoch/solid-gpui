use std::io::Cursor;

use super::*;
use serde::{Deserialize, Serialize};

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
