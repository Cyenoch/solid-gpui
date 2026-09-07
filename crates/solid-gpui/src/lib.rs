extern crate self as solid_gpui;
pub use gpui;
#[cfg(feature = "gpui-component")]
pub mod components;
#[cfg(feature = "host")]
pub mod host;
pub mod native;
pub mod runtime;
#[cfg(feature = "quickjs")]
pub use runtime::quickjs::{QuickJsAdapter, QuickJsError};
pub use solid_gpui_macros::{component, native_module, native_type};
pub mod protocol;
mod protocol_tap;
pub mod renderer;
pub mod transport;
pub mod tree;
pub use protocol::{
    AccessibilityProperties, AlignItemsCode, AlignSelfCode, BoxShadow, COMMAND_ACTIVATE_WINDOW,
    COMMAND_BLUR, COMMAND_CLIPBOARD_READ, COMMAND_CLIPBOARD_READ_IMAGE, COMMAND_CLIPBOARD_WRITE,
    COMMAND_CLIPBOARD_WRITE_IMAGE, COMMAND_FILE_DIALOG_OPEN, COMMAND_FILE_DIALOG_SAVE,
    COMMAND_FOCUS, COMMAND_FOCUS_NEXT, COMMAND_FOCUS_PREV, COMMAND_GET_FOCUS,
    COMMAND_GET_SCROLL_OFFSET, COMMAND_GET_WINDOW_BOUNDS, COMMAND_GET_WINDOW_SIZE,
    COMMAND_GET_WINDOW_STATE, COMMAND_INVOKE_NATIVE, COMMAND_LOAD_FONT, COMMAND_MESSAGE,
    COMMAND_MINIMIZE_WINDOW, COMMAND_OPEN_SURFACE, COMMAND_OPEN_URL, COMMAND_READ_TEXT_FILE,
    COMMAND_RESIZE_WINDOW, COMMAND_RESOLVE_CLOSE_REQUEST, COMMAND_SCROLL_TO_END,
    COMMAND_SCROLL_TO_INDEX, COMMAND_SCROLL_TO_OFFSET, COMMAND_SET_CLOSE_POLICY,
    COMMAND_SET_KEYBINDINGS, COMMAND_SET_MENUS, COMMAND_SET_SELECTION, COMMAND_SET_TITLE,
    COMMAND_SHOW_NOTIFICATION, COMMAND_TOGGLE_FULLSCREEN, COMMAND_WRITE_TEXT_FILE,
    COMMAND_ZOOM_WINDOW, ClipboardImage, Command, CommandKind, CommandMeta, CommandOperation,
    CommandResult, CommandValue, CursorCode, DecodedMessage, DragProperties, EVENT_ACTION,
    EVENT_ANIMATION_COMPLETE, EVENT_BLUR, EVENT_CHANGE, EVENT_CLOSE_REQUESTED,
    EVENT_COMMAND_RESULT, EVENT_DRAG, EVENT_FOCUS, EVENT_HOVER, EVENT_KEY, EVENT_KEY_DOWN,
    EVENT_KEY_REPEAT, EVENT_KEY_UP, EVENT_MESSAGE, EVENT_NOTIFICATION_RESPONSE, EVENT_POINTER,
    EVENT_POINTER_DOWN, EVENT_POINTER_DOWN_OUTSIDE, EVENT_POINTER_UP, EVENT_PRESS, EVENT_SCROLL,
    EVENT_SELECTION, EVENT_SUBMIT, EVENT_SURFACE_CLOSED, EVENT_VISIBLE_RANGE,
    EVENT_WINDOW_ACTIVATION, EVENT_WINDOW_APPEARANCE, EVENT_WINDOW_RESIZE, Easing, Event,
    EventKind, EventMeta, EventPayload, FlexDirectionCode, FontStyleCode, FontWeightCode,
    HostProperties, IconProperties, ImageProperties, JustifyContentCode, KeybindingDefinition,
    MAX_CLIPBOARD_IMAGE_BYTES, MAX_CLIPBOARD_TEXT_BYTES, MAX_FILE_READ_BYTES, MAX_FILE_WRITE_BYTES,
    MAX_FRAME_LENGTH, MAX_NATIVE_CALL_BYTES, MenuAction, MenuDefinition, MenuItemDefinition, Node,
    NotificationActionDefinition, NotificationResponseEvent, ObjectFitCode, OverflowCode,
    PATCH_MESSAGE, POINTER_BUTTON_BACK, POINTER_BUTTON_FORWARD, POINTER_BUTTON_LEFT,
    POINTER_BUTTON_MIDDLE, POINTER_BUTTON_RIGHT, PROTOCOL_VERSION, Patch, PatchOperation,
    PointerEvent, PointerMoveEvent, PositionCode, ProtocolError, SCROLL_DELTA_PIXELS, ScrollEvent,
    Snapshot, Style, TRANSITION_BACKGROUND_COLOR, TRANSITION_HEIGHT, TRANSITION_OPACITY,
    TRANSITION_WIDTH, TextAlignCode, TextDecorationCode, TextInputEvent, TextInputProperties,
    TextOverflowCode, Transition, UPDATE_ACCESSIBILITY, UPDATE_FOCUSABLE, UPDATE_LISTENER,
    UPDATE_POINTER_MOVE, UPDATE_PROPERTIES, UPDATE_SELECTABLE, UPDATE_STYLE, UPDATE_TEXT,
    UPDATE_TOOLTIP, VirtualListProperties, WindowAppearance, WindowOpenOptions, decode_message,
    read_frame, write_frame,
};
pub use renderer::{
    ExtensionAdapter, ExtensionChildIterator, ExtensionChildSummary, ExtensionChildren,
    ExtensionContent, ExtensionError, ExtensionEventSink, ExtensionInstance, ExtensionRegistry,
    ExtensionRenderContext, NoExtensions, RenderError, SolidRoot,
};
pub use transport::{
    InMemoryAdapter, ProcessAdapter, RuntimeAdapter, RuntimeStatus, fatal_runtime_failure,
    send_event_or_exit,
};
pub use tree::{
    KIND_EXTENSION, KIND_ICON, KIND_IMAGE, KIND_PRESSABLE, KIND_RAW_TEXT, KIND_TEXT,
    KIND_TEXT_INPUT, KIND_VIEW, KIND_VIRTUAL_LIST, NodeStore, PatchStats, StoredNode, TreeError,
};
#[cfg(test)]
mod tests;

#[cfg(feature = "host")]
pub use host::{run, run_application, run_application_with_profile};

pub mod icons;
