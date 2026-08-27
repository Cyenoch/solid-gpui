#![allow(unused_imports)]
pub(super) use crate::InMemoryAdapter;
pub(super) use crate::protocol::{EVENT_LAYOUT, KeyAction};
pub(super) use std::io::Cursor;
pub(super) use std::process::Command as ProcessCommand;
pub(super) use std::sync::Arc;
pub(super) use std::time::{Duration, Instant};

pub(super) use crate::{
    AccessibilityProperties, COMMAND_BLUR, COMMAND_CLIPBOARD_READ, COMMAND_CLIPBOARD_WRITE,
    COMMAND_FILE_DIALOG_OPEN, COMMAND_FILE_DIALOG_SAVE, COMMAND_FOCUS, COMMAND_FOCUS_NEXT,
    COMMAND_FOCUS_PREV, COMMAND_GET_FOCUS, COMMAND_GET_WINDOW_SIZE, COMMAND_MESSAGE,
    COMMAND_OPEN_SURFACE, COMMAND_OPEN_URL, COMMAND_RESIZE_WINDOW, COMMAND_SCROLL_TO_END,
    COMMAND_SCROLL_TO_INDEX, COMMAND_SET_MENUS, COMMAND_SET_SELECTION, COMMAND_SET_TITLE,
    COMMAND_SHOW_NOTIFICATION, COMMAND_TOGGLE_FULLSCREEN, COMMAND_ZOOM_WINDOW, Command,
    CommandResult, CommandValue, DragProperties, EVENT_ACTION, EVENT_ANIMATION_COMPLETE,
    EVENT_BLUR, EVENT_CHANGE, EVENT_COMMAND_RESULT, EVENT_DRAG, EVENT_FOCUS, EVENT_HOVER,
    EVENT_KEY, EVENT_KEY_DOWN, EVENT_KEY_REPEAT, EVENT_KEY_UP, EVENT_MESSAGE, EVENT_POINTER,
    EVENT_POINTER_DOWN, EVENT_POINTER_DOWN_OUTSIDE, EVENT_POINTER_UP, EVENT_PRESS, EVENT_SCROLL,
    EVENT_SELECTION, EVENT_SUBMIT, EVENT_SURFACE_CLOSED, EVENT_VISIBLE_RANGE,
    EVENT_WINDOW_ACTIVATION, EVENT_WINDOW_APPEARANCE, EVENT_WINDOW_RESIZE, Easing, Event,
    EventPayload, HostProperties, ImageProperties, KIND_IMAGE, KIND_PRESSABLE, KIND_RAW_TEXT,
    KIND_TEXT, KIND_TEXT_INPUT, KIND_VIEW, KIND_VIRTUAL_LIST, MAX_CLIPBOARD_TEXT_BYTES,
    MAX_FRAME_LENGTH, MenuAction, MenuDefinition, MenuItemDefinition, Node, NodeStore,
    PATCH_MESSAGE, POINTER_BUTTON_BACK, POINTER_BUTTON_FORWARD, POINTER_BUTTON_LEFT,
    POINTER_BUTTON_MIDDLE, POINTER_BUTTON_RIGHT, PROTOCOL_VERSION, Patch, PatchOperation,
    PatchStats, PointerEvent, ProcessAdapter, ProtocolError, ReactRoot, RenderError,
    RuntimeAdapter, RuntimeStatus, SCROLL_DELTA_LINES, SCROLL_DELTA_PIXELS, ScrollEvent, Snapshot,
    StoredNode, Style, TRANSITION_BACKGROUND_COLOR, TRANSITION_HEIGHT, TRANSITION_OPACITY,
    TRANSITION_WIDTH, TextInputEvent, TextInputProperties, Transition, TreeError,
    UPDATE_ACCESSIBILITY, UPDATE_LISTENER, UPDATE_PROPERTIES, UPDATE_STYLE, UPDATE_TEXT,
    VirtualListProperties, WindowAppearance, fatal_runtime_failure, read_frame, send_event_or_exit,
    write_frame,
};

pub(super) fn root_snapshot(revision: u32, nodes: Vec<Node>) -> Snapshot {
    Snapshot::new(7, 3, revision.saturating_sub(1), revision, nodes)
}

pub(super) fn synthetic_root(revision: u32) -> Snapshot {
    root_snapshot(revision, vec![Node::new(1, 0, 0, KIND_VIEW)])
}

pub(super) fn view_node(id: u32, parent_id: u32, index: u32) -> Node {
    Node::new(id, parent_id, index, KIND_VIEW)
}
