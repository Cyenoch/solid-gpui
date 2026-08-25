use std::io::{self, Read, Write};

use thiserror::Error;

mod wire;

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
pub const EVENT_KEY: u32 = 9;
pub const EVENT_POINTER: u32 = 10;
pub const EVENT_HOVER: u32 = 11;
pub const EVENT_SCROLL: u32 = 12;
pub const EVENT_SUBMIT: u32 = 13;
pub const EVENT_WINDOW_RESIZE: u32 = 14;
pub const EVENT_WINDOW_ACTIVATION: u32 = 15;
pub const EVENT_SURFACE_CLOSED: u32 = 16;
pub const EVENT_ACTION: u32 = 17;
pub const EVENT_WINDOW_APPEARANCE: u32 = 18;
pub const EVENT_LAYOUT: u32 = 19;
pub const EVENT_DRAG: u32 = 20;
pub const EVENT_POINTER_DOWN: u32 = 1;
pub const EVENT_POINTER_UP: u32 = 2;
pub const POINTER_BUTTON_LEFT: u32 = 1;
pub const POINTER_BUTTON_RIGHT: u32 = 2;
pub const POINTER_BUTTON_MIDDLE: u32 = 3;
pub const POINTER_BUTTON_BACK: u32 = 4;
pub const POINTER_BUTTON_FORWARD: u32 = 5;
pub const EVENT_KEY_DOWN: u32 = 1;
pub const EVENT_KEY_REPEAT: u32 = 2;
pub const EVENT_KEY_UP: u32 = 3;
pub const SCROLL_DELTA_PIXELS: u32 = 1;
pub const SCROLL_DELTA_LINES: u32 = 2;

pub const COMMAND_FOCUS: u32 = 1;
pub const COMMAND_BLUR: u32 = 2;
pub const COMMAND_SET_SELECTION: u32 = 3;
pub const COMMAND_SCROLL_TO_INDEX: u32 = 4;
pub const COMMAND_SCROLL_TO_END: u32 = 5;
pub const COMMAND_SET_TITLE: u32 = 6;
pub const COMMAND_RESIZE_WINDOW: u32 = 7;
pub const COMMAND_ZOOM_WINDOW: u32 = 8;
pub const COMMAND_TOGGLE_FULLSCREEN: u32 = 9;
pub const COMMAND_OPEN_URL: u32 = 10;
pub const COMMAND_FOCUS_NEXT: u32 = 11;
pub const COMMAND_FOCUS_PREV: u32 = 12;
pub const COMMAND_GET_WINDOW_SIZE: u32 = 13;
pub const COMMAND_GET_FOCUS: u32 = 14;
pub const COMMAND_CLIPBOARD_WRITE: u32 = 15;
pub const COMMAND_CLIPBOARD_READ: u32 = 16;
pub const COMMAND_OPEN_SURFACE: u32 = 17;
pub const COMMAND_FILE_DIALOG_OPEN: u32 = 18;
pub const COMMAND_FILE_DIALOG_SAVE: u32 = 19;
pub const COMMAND_SHOW_NOTIFICATION: u32 = 20;
pub const COMMAND_SET_MENUS: u32 = 21;
pub const MAX_WINDOW_DIMENSION: u32 = 16_384;
pub const MAX_CLIPBOARD_TEXT_BYTES: usize = 1 << 20;

pub const UPDATE_STYLE: u32 = 1;
pub const UPDATE_TEXT: u32 = 2;
pub const UPDATE_LISTENER: u32 = 4;
pub const UPDATE_PROPERTIES: u32 = 8;
pub const UPDATE_ACCESSIBILITY: u32 = 16;
pub const UPDATE_FOCUSABLE: u32 = 32;
pub const MAX_FRAME_LENGTH: usize = 16 * 1024 * 1024;

pub const TRANSITION_OPACITY: u32 = 1;
pub const TRANSITION_BACKGROUND_COLOR: u32 = 2;
pub const TRANSITION_WIDTH: u32 = 4;
pub const TRANSITION_HEIGHT: u32 = 8;

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
        wire::encode_snapshot(self)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, ProtocolError> {
        wire::decode_snapshot(payload)
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
        focusable: bool,
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
        wire::encode_patch(self)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, ProtocolError> {
        wire::decode_patch(payload)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuDefinition {
    pub title: String,
    pub items: Vec<MenuItemDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuItemDefinition {
    Separator,
    Action(String),
    Submenu(MenuDefinition),
}
#[derive(Clone, Debug, PartialEq, gpui::Action)]
#[action(namespace = react_gpui, no_json)]
pub struct MenuAction {
    pub name: String,
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
    pub title: Option<String>,
    pub body: Option<String>,
    pub menus: Option<Vec<MenuDefinition>>,
}

/// Optional typed data returned by a command. The tag is part of the wire
/// contract: `1=number`, `2=window-size pair`, `3=boolean`, `4=text`, and
/// `5=selected paths`.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandValue {
    Number(f32),
    Pair((f32, f32)),
    Bool(bool),
    Text(String),
    Paths(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandResult {
    pub request_id: u32,
    pub command: u32,
    pub node_id: u32,
    pub success: bool,
    pub error: Option<String>,
    /// `None` is also used by older peers that do not return a value.
    pub value: Option<CommandValue>,
}

impl Command {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        wire::encode_command(self)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, ProtocolError> {
        wire::decode_command(payload)
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
    pub focusable: bool,
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
            focusable: false,
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
    Image(ImageProperties),
    Drag(DragProperties),
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
    pub max_length: Option<u32>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ImageProperties {
    pub source: String,
    pub object_fit: u32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct DragProperties {
    pub drag_type: Option<String>,
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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Style {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub flex_direction: Option<u32>,
    pub flex_grow: Option<f32>,
    pub padding: Option<f32>,
    pub gap: Option<f32>,
    pub justify_content: Option<u32>,
    pub align_items: Option<u32>,
    pub border_radius: Option<f32>,
    pub border_width: Option<f32>,
    pub border_color_rgba: Option<u32>,
    pub font_size: Option<f32>,
    pub font_weight: Option<u32>,
    pub background_rgba: Option<u32>,
    pub color_rgba: Option<u32>,
    pub opacity: Option<f32>,
    pub transition: Option<Transition>,
    pub overflow: Option<u32>,
    pub line_clamp: Option<u32>,
    pub text_overflow: Option<u32>,
    pub margin_top: Option<f32>,
    pub margin_right: Option<f32>,
    pub margin_bottom: Option<f32>,
    pub margin_left: Option<f32>,
    pub font_style: Option<u32>,
    pub text_decoration: Option<u32>,
    pub line_height: Option<f32>,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub align_self: Option<u32>,
    pub position: Option<u32>,
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub cursor: Option<u32>,
    pub text_align: Option<u32>,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Down,
    Repeat,
    Up,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: String,
    pub modifiers: Vec<String>,
    pub action: KeyAction,
}

impl KeyEvent {
    pub fn new(key: String, modifiers: Vec<String>, action: KeyAction) -> Self {
        Self {
            key,
            modifiers,
            action,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointerEvent {
    pub button: u32,
    pub modifiers: Vec<String>,
    pub action: u32,
    pub click_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScrollEvent {
    pub delta_kind: u32,
    pub dx: f32,
    pub dy: f32,
    pub x: f32,
    pub y: f32,
    pub modifiers: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowAppearance {
    Light,
    Dark,
}

impl WindowAppearance {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventPayload {
    TextInput(TextInputEvent),
    CommandResult(CommandResult),
    VisibleRange {
        start: u32,
        end: u32,
    },
    AnimationComplete {
        generation: u32,
    },
    Key(KeyEvent),
    Pointer(PointerEvent),
    Scroll(ScrollEvent),
    Submit {
        text: String,
    },
    WindowResize {
        width: f32,
        height: f32,
    },
    WindowActivation {
        active: bool,
    },
    EventAction {
        action: String,
    },
    WindowAppearance {
        appearance: WindowAppearance,
    },
    Layout {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    DragOver {
        drag_type: String,
    },
    DragDrop {
        drag_type: String,
    },
    ExternalFileDrop {
        paths: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
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

    // Keep the positional wire fields explicit at this stable protocol seam.
    #[allow(clippy::too_many_arguments)]
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
    // Keep the positional wire fields explicit at this stable protocol seam.
    #[allow(clippy::too_many_arguments)]
    pub fn key(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        key: String,
        modifiers: Vec<String>,
        action: KeyAction,
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
            event_type: EVENT_KEY,
            payload: Some(EventPayload::Key(KeyEvent::new(key, modifiers, action))),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn pointer(
        event_type: u32,
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        button: u32,
        modifiers: Vec<String>,
        action: u32,
        click_count: u32,
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
            payload: Some(EventPayload::Pointer(PointerEvent {
                button,
                modifiers,
                action,
                click_count,
            })),
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn scroll(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        delta_kind: u32,
        dx: f32,
        dy: f32,
        x: f32,
        y: f32,
        modifiers: Vec<String>,
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
            event_type: EVENT_SCROLL,
            payload: Some(EventPayload::Scroll(ScrollEvent {
                delta_kind,
                dx,
                dy,
                x,
                y,
                modifiers,
            })),
        }
    }

    pub fn hover(
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
            event_type: EVENT_HOVER,
            payload: None,
        }
    }
    pub fn submit(
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
            event_type: EVENT_SUBMIT,
            payload: None,
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn submit_with_text(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        text: String,
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
            event_type: EVENT_SUBMIT,
            payload: Some(EventPayload::Submit { text }),
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn window_resize(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        width: f32,
        height: f32,
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
            event_type: EVENT_WINDOW_RESIZE,
            payload: Some(EventPayload::WindowResize { width, height }),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn window_activation(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        active: bool,
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
            event_type: EVENT_WINDOW_ACTIVATION,
            payload: Some(EventPayload::WindowActivation { active }),
        }
    }
    pub fn window_appearance(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        appearance: WindowAppearance,
    ) -> Self {
        Self {
            protocol: PROTOCOL_VERSION,
            message: EVENT_MESSAGE,
            surface_id,
            epoch,
            revision,
            sequence,
            node_id: 1,
            listener_id: 0,
            event_type: EVENT_WINDOW_APPEARANCE,
            payload: Some(EventPayload::WindowAppearance { appearance }),
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn layout(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
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
            event_type: EVENT_LAYOUT,
            payload: Some(EventPayload::Layout {
                x,
                y,
                width,
                height,
            }),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn drag_event(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        payload: EventPayload,
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
            event_type: EVENT_DRAG,
            payload: Some(payload),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn drag_over(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        drag_type: String,
    ) -> Self {
        Self::drag_event(
            surface_id,
            epoch,
            revision,
            sequence,
            node_id,
            listener_id,
            EventPayload::DragOver { drag_type },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn drag_drop(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        drag_type: String,
    ) -> Self {
        Self::drag_event(
            surface_id,
            epoch,
            revision,
            sequence,
            node_id,
            listener_id,
            EventPayload::DragDrop { drag_type },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn external_file_drop(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        paths: Vec<String>,
    ) -> Self {
        Self::drag_event(
            surface_id,
            epoch,
            revision,
            sequence,
            node_id,
            listener_id,
            EventPayload::ExternalFileDrop { paths },
        )
    }

    pub fn action(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        action: String,
    ) -> Self {
        Self {
            protocol: PROTOCOL_VERSION,
            message: EVENT_MESSAGE,
            surface_id,
            epoch,
            revision,
            sequence,
            node_id: 1,
            listener_id: 0,
            event_type: EVENT_ACTION,
            payload: Some(EventPayload::EventAction { action }),
        }
    }

    /// Notify the renderer that a native surface was closed. This is emitted
    /// before the host removes the surface from its registry.
    pub fn surface_closed(surface_id: u32, epoch: u32, revision: u32, sequence: u32) -> Self {
        Self {
            protocol: PROTOCOL_VERSION,
            message: EVENT_MESSAGE,
            surface_id,
            epoch,
            revision,
            sequence,
            node_id: 0,
            listener_id: 0,
            event_type: EVENT_SURFACE_CLOSED,
            payload: None,
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

    // Keep the positional wire fields explicit at this stable protocol seam.
    #[allow(clippy::too_many_arguments)]
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
        wire::encode_event(self)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, ProtocolError> {
        wire::decode_event(payload)
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

/// Write one little-endian length-prefixed frame and flush it. The flush keeps
/// each event frame visible immediately when the writer is buffered.
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
