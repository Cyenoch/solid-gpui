// Generated from protocol.bop; do not edit.
#![allow(dead_code)]

pub const BODY_SNAPSHOT: u32 = 1;
pub const BODY_EVENT: u32 = 2;
pub const BODY_PATCH: u32 = 3;
pub const BODY_COMMAND: u32 = 4;

pub const HOST_PROPERTIES_TEXT_INPUT_PROPERTIES: u32 = 1;
pub const HOST_PROPERTIES_VIRTUAL_LIST_PROPERTIES: u32 = 2;
pub const HOST_PROPERTIES_IMAGE_PROPERTIES: u32 = 3;
pub const HOST_PROPERTIES_DRAG_PROPERTIES: u32 = 4;
pub const HOST_PROPERTIES_EXTENSION_PROPERTIES: u32 = 5;
pub const HOST_PROPERTIES_ICON_PROPERTIES: u32 = 6;

pub const PATCH_OPERATION_PATCH_CREATE: u32 = 1;
pub const PATCH_OPERATION_PATCH_UPDATE: u32 = 2;
pub const PATCH_OPERATION_PATCH_MOVE: u32 = 3;
pub const PATCH_OPERATION_PATCH_DELETE: u32 = 4;

pub const MENU_ITEM_MENU_SEPARATOR: u32 = 1;
pub const MENU_ITEM_MENU_ACTION: u32 = 2;
pub const MENU_ITEM_MENU_SUBMENU: u32 = 3;

pub const COMMAND_PAYLOAD_U32_PAIR_COMMAND: u32 = 1;
pub const COMMAND_PAYLOAD_FLOAT_COMMAND: u32 = 2;
pub const COMMAND_PAYLOAD_TEXT_COMMAND: u32 = 3;
pub const COMMAND_PAYLOAD_STRING_PAIR_COMMAND: u32 = 4;
pub const COMMAND_PAYLOAD_OPEN_SURFACE_COMMAND: u32 = 5;
pub const COMMAND_PAYLOAD_FILE_DIALOG_OPEN_COMMAND: u32 = 6;
pub const COMMAND_PAYLOAD_NOTIFICATION_COMMAND: u32 = 7;
pub const COMMAND_PAYLOAD_MENUS_COMMAND: u32 = 8;
pub const COMMAND_PAYLOAD_KEYBINDINGS_COMMAND: u32 = 9;
pub const COMMAND_PAYLOAD_CLIPBOARD_IMAGE_COMMAND: u32 = 10;
pub const COMMAND_PAYLOAD_CLOSE_RESOLUTION_COMMAND: u32 = 11;
pub const COMMAND_PAYLOAD_INVOKE_NATIVE_COMMAND: u32 = 12;
pub const COMMAND_PAYLOAD_CANCEL_NATIVE_COMMAND: u32 = 13;
pub const COMMAND_PAYLOAD_CONFIGURE_APPLICATION_COMMAND: u32 = 14;

pub const COMMAND_VALUE_NUMBER_VALUE: u32 = 1;
pub const COMMAND_VALUE_PAIR_VALUE: u32 = 2;
pub const COMMAND_VALUE_BOOL_VALUE: u32 = 3;
pub const COMMAND_VALUE_TEXT_VALUE: u32 = 4;
pub const COMMAND_VALUE_PATHS_VALUE: u32 = 5;
pub const COMMAND_VALUE_FILE_TEXT_VALUE: u32 = 6;
pub const COMMAND_VALUE_IMAGE_VALUE: u32 = 7;
pub const COMMAND_VALUE_BOUNDS_VALUE: u32 = 8;
pub const COMMAND_VALUE_WINDOW_STATE_VALUE: u32 = 9;
pub const COMMAND_VALUE_SCROLL_OFFSET_VALUE: u32 = 10;
pub const COMMAND_VALUE_BYTES_VALUE: u32 = 11;

pub const EVENT_PAYLOAD_TEXT_INPUT_EVENT_DATA: u32 = 1;
pub const EVENT_PAYLOAD_COMMAND_RESULT: u32 = 2;
pub const EVENT_PAYLOAD_VISIBLE_RANGE_EVENT: u32 = 3;
pub const EVENT_PAYLOAD_ANIMATION_COMPLETE_EVENT: u32 = 4;
pub const EVENT_PAYLOAD_KEY_EVENT: u32 = 5;
pub const EVENT_PAYLOAD_POINTER_EVENT: u32 = 6;
pub const EVENT_PAYLOAD_POINTER_MOVE_EVENT: u32 = 7;
pub const EVENT_PAYLOAD_SCROLL_EVENT: u32 = 8;
pub const EVENT_PAYLOAD_SUBMIT_EVENT: u32 = 9;
pub const EVENT_PAYLOAD_WINDOW_RESIZE_EVENT: u32 = 10;
pub const EVENT_PAYLOAD_WINDOW_ACTIVATION_EVENT: u32 = 11;
pub const EVENT_PAYLOAD_ACTION_EVENT: u32 = 12;
pub const EVENT_PAYLOAD_WINDOW_APPEARANCE_EVENT: u32 = 13;
pub const EVENT_PAYLOAD_LAYOUT_EVENT: u32 = 14;
pub const EVENT_PAYLOAD_DRAG_OVER_EVENT: u32 = 15;
pub const EVENT_PAYLOAD_DRAG_DROP_EVENT: u32 = 16;
pub const EVENT_PAYLOAD_EXTERNAL_FILE_DROP_EVENT: u32 = 17;
pub const EVENT_PAYLOAD_NOTIFICATION_RESPONSE_EVENT: u32 = 18;
pub const EVENT_PAYLOAD_POINTER_DOWN_OUTSIDE_EVENT: u32 = 19;
pub const EVENT_PAYLOAD_CLOSE_REQUESTED_EVENT: u32 = 20;
pub const EVENT_PAYLOAD_EXTENSION_EVENT: u32 = 21;
pub const EVENT_PAYLOAD_APPLICATION_ACTIVATION_EVENT: u32 = 22;

pub const NODE_KIND_UNSPECIFIED: u32 = 0;
pub const NODE_KIND_VIEW: u32 = 1;
pub const NODE_KIND_TEXT: u32 = 2;
pub const NODE_KIND_PRESSABLE: u32 = 3;
pub const NODE_KIND_RAW_TEXT: u32 = 4;
pub const NODE_KIND_TEXT_INPUT: u32 = 5;
pub const NODE_KIND_VIRTUAL_LIST: u32 = 6;
pub const NODE_KIND_IMAGE: u32 = 7;
pub const NODE_KIND_EXTENSION: u32 = 8;
pub const NODE_KIND_ICON: u32 = 9;

pub const EVENT_UNKNOWN: u32 = 0;
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
pub const EVENT_NOTIFICATION_RESPONSE: u32 = 21;
pub const EVENT_POINTER_DOWN_OUTSIDE: u32 = 22;
pub const EVENT_CLOSE_REQUESTED: u32 = 23;
pub const EVENT_EXTENSION: u32 = 24;
pub const EVENT_APPLICATION_ACTIVATION: u32 = 25;

pub const COMMAND_UNKNOWN: u32 = 0;
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
pub const COMMAND_SET_KEYBINDINGS: u32 = 22;
pub const COMMAND_SET_CLOSE_POLICY: u32 = 23;
pub const COMMAND_RESOLVE_CLOSE_REQUEST: u32 = 24;
pub const COMMAND_READ_TEXT_FILE: u32 = 25;
pub const COMMAND_WRITE_TEXT_FILE: u32 = 26;
pub const COMMAND_CLIPBOARD_WRITE_IMAGE: u32 = 27;
pub const COMMAND_CLIPBOARD_READ_IMAGE: u32 = 28;
pub const COMMAND_LOAD_FONT: u32 = 29;
pub const COMMAND_MINIMIZE_WINDOW: u32 = 30;
pub const COMMAND_GET_WINDOW_BOUNDS: u32 = 31;
pub const COMMAND_GET_WINDOW_STATE: u32 = 32;
pub const COMMAND_ACTIVATE_WINDOW: u32 = 33;
pub const COMMAND_GET_SCROLL_OFFSET: u32 = 34;
pub const COMMAND_SCROLL_TO_OFFSET: u32 = 35;
pub const COMMAND_INVOKE_NATIVE: u32 = 36;
pub const COMMAND_CANCEL_NATIVE: u32 = 37;
pub const COMMAND_CONFIGURE_APPLICATION: u32 = 38;

pub(crate) const fn node_kind(value: u32) -> Option<super::generated::NodeKind> {
    match value {
        0 => Some(super::generated::NodeKind::Unspecified),
        1 => Some(super::generated::NodeKind::View),
        2 => Some(super::generated::NodeKind::Text),
        3 => Some(super::generated::NodeKind::Pressable),
        4 => Some(super::generated::NodeKind::RawText),
        5 => Some(super::generated::NodeKind::TextInput),
        6 => Some(super::generated::NodeKind::VirtualList),
        7 => Some(super::generated::NodeKind::Image),
        8 => Some(super::generated::NodeKind::Extension),
        9 => Some(super::generated::NodeKind::Icon),
        _ => None,
    }
}

pub(crate) const fn window_appearance(value: u32) -> Option<super::generated::WindowAppearance> {
    match value {
        0 => Some(super::generated::WindowAppearance::Unspecified),
        1 => Some(super::generated::WindowAppearance::Light),
        2 => Some(super::generated::WindowAppearance::Dark),
        _ => None,
    }
}

pub(crate) const fn event_kind(value: u32) -> Option<super::generated::EventKind> {
    match value {
        0 => Some(super::generated::EventKind::Unknown),
        1 => Some(super::generated::EventKind::Press),
        2 => Some(super::generated::EventKind::Change),
        3 => Some(super::generated::EventKind::Selection),
        4 => Some(super::generated::EventKind::Focus),
        5 => Some(super::generated::EventKind::Blur),
        6 => Some(super::generated::EventKind::CommandResult),
        7 => Some(super::generated::EventKind::VisibleRange),
        8 => Some(super::generated::EventKind::AnimationComplete),
        9 => Some(super::generated::EventKind::Key),
        10 => Some(super::generated::EventKind::Pointer),
        11 => Some(super::generated::EventKind::Hover),
        12 => Some(super::generated::EventKind::Scroll),
        13 => Some(super::generated::EventKind::Submit),
        14 => Some(super::generated::EventKind::WindowResize),
        15 => Some(super::generated::EventKind::WindowActivation),
        16 => Some(super::generated::EventKind::SurfaceClosed),
        17 => Some(super::generated::EventKind::Action),
        18 => Some(super::generated::EventKind::WindowAppearance),
        19 => Some(super::generated::EventKind::Layout),
        20 => Some(super::generated::EventKind::Drag),
        21 => Some(super::generated::EventKind::NotificationResponse),
        22 => Some(super::generated::EventKind::PointerDownOutside),
        23 => Some(super::generated::EventKind::CloseRequested),
        24 => Some(super::generated::EventKind::Extension),
        25 => Some(super::generated::EventKind::ApplicationActivation),
        _ => None,
    }
}

pub(crate) const fn command_kind(value: u32) -> Option<super::generated::CommandKind> {
    match value {
        0 => Some(super::generated::CommandKind::Unknown),
        1 => Some(super::generated::CommandKind::Focus),
        2 => Some(super::generated::CommandKind::Blur),
        3 => Some(super::generated::CommandKind::SetSelection),
        4 => Some(super::generated::CommandKind::ScrollToIndex),
        5 => Some(super::generated::CommandKind::ScrollToEnd),
        6 => Some(super::generated::CommandKind::SetTitle),
        7 => Some(super::generated::CommandKind::ResizeWindow),
        8 => Some(super::generated::CommandKind::ZoomWindow),
        9 => Some(super::generated::CommandKind::ToggleFullscreen),
        10 => Some(super::generated::CommandKind::OpenUrl),
        11 => Some(super::generated::CommandKind::FocusNext),
        12 => Some(super::generated::CommandKind::FocusPrev),
        13 => Some(super::generated::CommandKind::GetWindowSize),
        14 => Some(super::generated::CommandKind::GetFocus),
        15 => Some(super::generated::CommandKind::ClipboardWrite),
        16 => Some(super::generated::CommandKind::ClipboardRead),
        17 => Some(super::generated::CommandKind::OpenSurface),
        18 => Some(super::generated::CommandKind::FileDialogOpen),
        19 => Some(super::generated::CommandKind::FileDialogSave),
        20 => Some(super::generated::CommandKind::ShowNotification),
        21 => Some(super::generated::CommandKind::SetMenus),
        22 => Some(super::generated::CommandKind::SetKeybindings),
        23 => Some(super::generated::CommandKind::SetClosePolicy),
        24 => Some(super::generated::CommandKind::ResolveCloseRequest),
        25 => Some(super::generated::CommandKind::ReadTextFile),
        26 => Some(super::generated::CommandKind::WriteTextFile),
        27 => Some(super::generated::CommandKind::ClipboardWriteImage),
        28 => Some(super::generated::CommandKind::ClipboardReadImage),
        29 => Some(super::generated::CommandKind::LoadFont),
        30 => Some(super::generated::CommandKind::MinimizeWindow),
        31 => Some(super::generated::CommandKind::GetWindowBounds),
        32 => Some(super::generated::CommandKind::GetWindowState),
        33 => Some(super::generated::CommandKind::ActivateWindow),
        34 => Some(super::generated::CommandKind::GetScrollOffset),
        35 => Some(super::generated::CommandKind::ScrollToOffset),
        36 => Some(super::generated::CommandKind::InvokeNative),
        37 => Some(super::generated::CommandKind::CancelNative),
        38 => Some(super::generated::CommandKind::ConfigureApplication),
        _ => None,
    }
}

pub const COMMAND_KINDS: &[u32] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38,
];
