pub(super) use crate::InMemoryAdapter;
pub(super) use crate::protocol::UPDATE_FOCUSABLE;
pub(super) use std::process::Command as ProcessCommand;
pub(super) use std::sync::Arc;
pub(super) use std::time::{Duration, Instant};

pub(super) use crate::{
    AccessibilityProperties, AlignItemsCode, AlignSelfCode, COMMAND_ACTIVATE_WINDOW, COMMAND_FOCUS,
    COMMAND_GET_WINDOW_SIZE, Command, CommandKind, CommandMeta, CommandOperation, CommandResult,
    CursorCode, DragProperties, EVENT_CHANGE, EVENT_POINTER_DOWN, Easing, Event, EventMeta,
    EventPayload, FlexDirectionCode, FontStyleCode, FontWeightCode, HostProperties, IconProperties,
    ImageProperties, JustifyContentCode, KIND_ICON, KIND_IMAGE, KIND_PRESSABLE, KIND_RAW_TEXT,
    KIND_TEXT, KIND_TEXT_INPUT, KIND_VIEW, KIND_VIRTUAL_LIST, Node, NodeStore, OverflowCode,
    POINTER_BUTTON_LEFT, Patch, PatchOperation, PatchStats, PointerEvent, PositionCode,
    ProcessAdapter, RuntimeAdapter, RuntimeStatus, Snapshot, StoredNode, Style, TRANSITION_HEIGHT,
    TRANSITION_OPACITY, TRANSITION_WIDTH, TextAlignCode, TextDecorationCode, TextInputEvent,
    TextInputProperties, TextOverflowCode, Transition, TreeError, UPDATE_ACCESSIBILITY,
    UPDATE_LISTENER, UPDATE_POINTER_MOVE, UPDATE_PROPERTIES, UPDATE_STYLE, UPDATE_TEXT,
    UPDATE_TOOLTIP, VirtualListProperties, send_event_or_exit,
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
