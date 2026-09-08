use std::io::{self, Read, Write};
use std::sync::Arc;

use thiserror::Error;

#[path = "protocol/generated/protocol.rs"]
mod generated;
#[path = "protocol/generated/schema_facts.rs"]
pub(crate) mod generated_facts;
mod guard;
mod wire;

pub const PROTOCOL_VERSION: u32 = 5;
pub const SNAPSHOT_MESSAGE: u32 = generated_facts::BODY_SNAPSHOT;
pub const EVENT_MESSAGE: u32 = generated_facts::BODY_EVENT;
pub const PATCH_MESSAGE: u32 = generated_facts::BODY_PATCH;
pub const COMMAND_MESSAGE: u32 = generated_facts::BODY_COMMAND;

pub const EVENT_PRESS: u32 = generated_facts::EVENT_PRESS;
pub const EVENT_CHANGE: u32 = generated_facts::EVENT_CHANGE;
pub const EVENT_SELECTION: u32 = generated_facts::EVENT_SELECTION;
pub const EVENT_FOCUS: u32 = generated_facts::EVENT_FOCUS;
pub const EVENT_BLUR: u32 = generated_facts::EVENT_BLUR;
pub const EVENT_COMMAND_RESULT: u32 = generated_facts::EVENT_COMMAND_RESULT;
pub const EVENT_VISIBLE_RANGE: u32 = generated_facts::EVENT_VISIBLE_RANGE;
pub const EVENT_ANIMATION_COMPLETE: u32 = generated_facts::EVENT_ANIMATION_COMPLETE;
pub const EVENT_KEY: u32 = generated_facts::EVENT_KEY;
pub const EVENT_POINTER: u32 = generated_facts::EVENT_POINTER;
pub const EVENT_HOVER: u32 = generated_facts::EVENT_HOVER;
pub const EVENT_SCROLL: u32 = generated_facts::EVENT_SCROLL;
pub const EVENT_SUBMIT: u32 = generated_facts::EVENT_SUBMIT;
pub const EVENT_WINDOW_RESIZE: u32 = generated_facts::EVENT_WINDOW_RESIZE;
pub const EVENT_WINDOW_ACTIVATION: u32 = generated_facts::EVENT_WINDOW_ACTIVATION;
pub const EVENT_SURFACE_CLOSED: u32 = generated_facts::EVENT_SURFACE_CLOSED;
pub const EVENT_ACTION: u32 = generated_facts::EVENT_ACTION;
pub const EVENT_WINDOW_APPEARANCE: u32 = generated_facts::EVENT_WINDOW_APPEARANCE;
pub const EVENT_LAYOUT: u32 = generated_facts::EVENT_LAYOUT;
pub const EVENT_DRAG: u32 = generated_facts::EVENT_DRAG;
pub const EVENT_NOTIFICATION_RESPONSE: u32 = generated_facts::EVENT_NOTIFICATION_RESPONSE;
pub const EVENT_POINTER_DOWN_OUTSIDE: u32 = generated_facts::EVENT_POINTER_DOWN_OUTSIDE;
pub const EVENT_CLOSE_REQUESTED: u32 = generated_facts::EVENT_CLOSE_REQUESTED;

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

pub const COMMAND_FOCUS: u32 = generated_facts::COMMAND_FOCUS;
pub const COMMAND_BLUR: u32 = generated_facts::COMMAND_BLUR;
pub const COMMAND_SET_SELECTION: u32 = generated_facts::COMMAND_SET_SELECTION;
pub const COMMAND_SCROLL_TO_INDEX: u32 = generated_facts::COMMAND_SCROLL_TO_INDEX;
pub const COMMAND_SCROLL_TO_END: u32 = generated_facts::COMMAND_SCROLL_TO_END;
pub const COMMAND_GET_SCROLL_OFFSET: u32 = generated_facts::COMMAND_GET_SCROLL_OFFSET;
pub const COMMAND_SCROLL_TO_OFFSET: u32 = generated_facts::COMMAND_SCROLL_TO_OFFSET;
pub const COMMAND_SET_TITLE: u32 = generated_facts::COMMAND_SET_TITLE;
pub const COMMAND_RESIZE_WINDOW: u32 = generated_facts::COMMAND_RESIZE_WINDOW;
pub const COMMAND_ZOOM_WINDOW: u32 = generated_facts::COMMAND_ZOOM_WINDOW;
pub const COMMAND_TOGGLE_FULLSCREEN: u32 = generated_facts::COMMAND_TOGGLE_FULLSCREEN;
pub const COMMAND_OPEN_URL: u32 = generated_facts::COMMAND_OPEN_URL;
pub const COMMAND_FOCUS_NEXT: u32 = generated_facts::COMMAND_FOCUS_NEXT;
pub const COMMAND_FOCUS_PREV: u32 = generated_facts::COMMAND_FOCUS_PREV;
pub const COMMAND_GET_WINDOW_SIZE: u32 = generated_facts::COMMAND_GET_WINDOW_SIZE;
pub const COMMAND_GET_FOCUS: u32 = generated_facts::COMMAND_GET_FOCUS;
pub const COMMAND_CLIPBOARD_WRITE: u32 = generated_facts::COMMAND_CLIPBOARD_WRITE;
pub const COMMAND_CLIPBOARD_READ: u32 = generated_facts::COMMAND_CLIPBOARD_READ;
pub const COMMAND_OPEN_SURFACE: u32 = generated_facts::COMMAND_OPEN_SURFACE;
pub const COMMAND_FILE_DIALOG_OPEN: u32 = generated_facts::COMMAND_FILE_DIALOG_OPEN;
pub const COMMAND_FILE_DIALOG_SAVE: u32 = generated_facts::COMMAND_FILE_DIALOG_SAVE;
pub const COMMAND_SHOW_NOTIFICATION: u32 = generated_facts::COMMAND_SHOW_NOTIFICATION;
pub const COMMAND_SET_MENUS: u32 = generated_facts::COMMAND_SET_MENUS;
pub const COMMAND_SET_KEYBINDINGS: u32 = generated_facts::COMMAND_SET_KEYBINDINGS;
pub const COMMAND_SET_CLOSE_POLICY: u32 = generated_facts::COMMAND_SET_CLOSE_POLICY;
pub const COMMAND_RESOLVE_CLOSE_REQUEST: u32 = generated_facts::COMMAND_RESOLVE_CLOSE_REQUEST;
pub const COMMAND_READ_TEXT_FILE: u32 = generated_facts::COMMAND_READ_TEXT_FILE;
pub const COMMAND_WRITE_TEXT_FILE: u32 = generated_facts::COMMAND_WRITE_TEXT_FILE;
pub const COMMAND_CLIPBOARD_WRITE_IMAGE: u32 = generated_facts::COMMAND_CLIPBOARD_WRITE_IMAGE;
pub const COMMAND_CLIPBOARD_READ_IMAGE: u32 = generated_facts::COMMAND_CLIPBOARD_READ_IMAGE;
pub const COMMAND_LOAD_FONT: u32 = generated_facts::COMMAND_LOAD_FONT;
pub const COMMAND_MINIMIZE_WINDOW: u32 = generated_facts::COMMAND_MINIMIZE_WINDOW;
pub const COMMAND_GET_WINDOW_BOUNDS: u32 = generated_facts::COMMAND_GET_WINDOW_BOUNDS;
pub const COMMAND_GET_WINDOW_STATE: u32 = generated_facts::COMMAND_GET_WINDOW_STATE;
pub const COMMAND_ACTIVATE_WINDOW: u32 = generated_facts::COMMAND_ACTIVATE_WINDOW;
pub const COMMAND_INVOKE_NATIVE: u32 = generated_facts::COMMAND_INVOKE_NATIVE;
pub const MAX_NATIVE_CALL_BYTES: usize = 1 << 20;
pub const MAX_IMAGE_SOURCE_BYTES: usize = 1 << 20;
pub const MAX_WINDOW_DIMENSION: u32 = 16_384;
pub const MAX_CLIPBOARD_TEXT_BYTES: usize = 1 << 20;
/// File and clipboard-image payloads leave 1 KiB for the complete Bebop command/frame envelope.
pub const MAX_FILE_WRITE_BYTES: usize = MAX_FRAME_LENGTH - 1024;
pub const MAX_FILE_READ_BYTES: usize = MAX_FRAME_LENGTH - 1024;
pub const MAX_CLIPBOARD_IMAGE_BYTES: usize = MAX_FRAME_LENGTH - 1024;
pub const UPDATE_STYLE: u32 = 1;
pub const UPDATE_TEXT: u32 = 2;
pub const UPDATE_LISTENER: u32 = 4;
pub const UPDATE_PROPERTIES: u32 = 8;
pub const UPDATE_ACCESSIBILITY: u32 = 16;
pub const UPDATE_FOCUSABLE: u32 = 32;
pub const UPDATE_SELECTABLE: u32 = 64;
pub const UPDATE_TOOLTIP: u32 = 128;
pub const UPDATE_POINTER_MOVE: u32 = 256;
pub const MAX_FRAME_LENGTH: usize = 16 * 1024 * 1024;
pub const MAX_EXTENSION_FIELDS: usize = 256;
pub const MAX_EXTENSION_EVENTS: usize = 256;
pub const MAX_EXTENSION_TEXT_BYTES: usize = 1 << 20;
pub const MAX_EXTENSION_VALUE_BYTES: usize = 1 << 20;

pub const TRANSITION_OPACITY: u32 = 1;
pub const TRANSITION_BACKGROUND_COLOR: u32 = 2;
pub const TRANSITION_WIDTH: u32 = 4;
pub const TRANSITION_HEIGHT: u32 = 8;

/// A complete immutable renderer commit. It is encoded as
/// `Envelope{protocolVersion:5, body: Snapshot}`.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
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
/// `Envelope{protocolVersion:5, body: Patch}`.
#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
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
        selectable: bool,
        tooltip: Option<String>,
        accepts_pointer_move: bool,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationActionDefinition {
    pub id: String,
    pub label: String,
}
#[derive(Debug, Clone, PartialEq)]
pub struct MenuDefinition {
    pub title: String,
    pub items: Vec<MenuItemDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuItemDefinition {
    Separator,
    Action {
        name: String,
        disabled: bool,
        checked: bool,
    },
    Submenu(MenuDefinition),
}
#[derive(Clone, Debug, PartialEq, gpui::Action)]
#[action(namespace = solid_gpui, no_json)]
pub struct MenuAction {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeybindingDefinition {
    pub keystrokes: String,
    pub action_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowOpenOptions {
    pub kind: Option<u32>,
    pub resizable: Option<bool>,
    pub min_size: Option<(u32, u32)>,
}

macro_rules! schema_kind {
    (
        $facts_fn:ident,
        $semantic:ident,
        $generated:ident {
            $( $variant:ident => $wire:ident ),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $semantic {
            $( $variant ),+
        }

        impl TryFrom<u32> for $semantic {
            type Error = ();

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                let value = generated_facts::$facts_fn(value).ok_or(())?;
                match value {
                    $( generated::$generated::$wire => Ok(Self::$variant), )+
                    _ => Err(()),
                }
            }
        }

        impl From<$semantic> for u32 {
            fn from(value: $semantic) -> Self {
                match value {
                    $( $semantic::$variant => u8::from(generated::$generated::$wire) as u32, )+
                }
            }
        }
    };
}

schema_kind!(node_kind, NodeKind, NodeKind {
    View => View,
    Text => Text,
    Pressable => Pressable,
    RawText => RawText,
    TextInput => TextInput,
    VirtualList => VirtualList,
    Image => Image,
    Extension => Extension,
    Icon => Icon,
});
schema_kind!(event_kind, EventKind, EventKind {
    Press => Press,
    Change => Change,
    Selection => Selection,
    Focus => Focus,
    Blur => Blur,
    CommandResult => CommandResult,
    VisibleRange => VisibleRange,
    AnimationComplete => AnimationComplete,
    Key => Key,
    Pointer => Pointer,
    Hover => Hover,
    Scroll => Scroll,
    Submit => Submit,
    WindowResize => WindowResize,
    WindowActivation => WindowActivation,
    SurfaceClosed => SurfaceClosed,
    Action => Action,
    WindowAppearance => WindowAppearance,
    Layout => Layout,
    Drag => Drag,
    NotificationResponse => NotificationResponse,
    PointerDownOutside => PointerDownOutside,
    CloseRequested => CloseRequested,
    Extension => Extension,
    ApplicationActivation => ApplicationActivation,
});

#[derive(Debug, Clone, PartialEq)]
pub enum ExtensionValue {
    Bool(bool),
    Int32(i32),
    U32(u32),
    F32(f32),
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtensionField {
    pub id: u32,
    pub value: ExtensionValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtensionProperties {
    pub provider_id: [u8; 16],
    pub catalog_digest: [u8; 32],
    pub entry_id: u32,
    pub entry_version: u32,
    pub fields: Vec<ExtensionField>,
    pub event_ids: Arc<[u32]>,
}
schema_kind!(command_kind, CommandKind, CommandKind {
    Focus => Focus,
    Blur => Blur,
    SetSelection => SetSelection,
    ScrollToIndex => ScrollToIndex,
    ScrollToEnd => ScrollToEnd,
    SetTitle => SetTitle,
    ResizeWindow => ResizeWindow,
    ZoomWindow => ZoomWindow,
    ToggleFullscreen => ToggleFullscreen,
    OpenUrl => OpenUrl,
    FocusNext => FocusNext,
    FocusPrev => FocusPrev,
    GetWindowSize => GetWindowSize,
    GetFocus => GetFocus,
    ClipboardWrite => ClipboardWrite,
    ClipboardRead => ClipboardRead,
    OpenSurface => OpenSurface,
    FileDialogOpen => FileDialogOpen,
    FileDialogSave => FileDialogSave,
    ShowNotification => ShowNotification,
    SetMenus => SetMenus,
    SetKeybindings => SetKeybindings,
    SetClosePolicy => SetClosePolicy,
    ResolveCloseRequest => ResolveCloseRequest,
    ReadTextFile => ReadTextFile,
    WriteTextFile => WriteTextFile,
    ClipboardWriteImage => ClipboardWriteImage,
    ClipboardReadImage => ClipboardReadImage,
    LoadFont => LoadFont,
    MinimizeWindow => MinimizeWindow,
    GetWindowBounds => GetWindowBounds,
    GetWindowState => GetWindowState,
    ActivateWindow => ActivateWindow,
    GetScrollOffset => GetScrollOffset,
    ScrollToOffset => ScrollToOffset,
    InvokeNative => InvokeNative,
    CancelNative => CancelNative,
    ConfigureApplication => ConfigureApplication,
});

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub meta: CommandMeta,
    pub operation: CommandOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandMeta {
    pub surface_id: u32,
    pub epoch: u32,
    pub after_revision: u32,
    pub request_id: u32,
    pub node_id: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandOperation {
    Focus,
    Blur,
    SetSelection {
        start: u32,
        end: u32,
    },
    ScrollToIndex {
        index: u32,
        alignment: u32,
    },
    ScrollToEnd,
    SetTitle {
        title: String,
    },
    ResizeWindow {
        width: u32,
        height: u32,
    },
    ZoomWindow,
    ToggleFullscreen,
    OpenUrl {
        url: String,
    },
    FocusNext,
    FocusPrev,
    GetWindowSize,
    GetFocus,
    ClipboardWrite {
        text: String,
    },
    ClipboardRead,
    OpenSurface {
        title: String,
        width: u32,
        height: u32,
        options: Option<WindowOpenOptions>,
    },
    FileDialogOpen {
        title: String,
        directories: bool,
        multiple: bool,
    },
    FileDialogSave {
        default_name: String,
    },
    ShowNotification {
        title: String,
        body: String,
        actions: Option<Vec<NotificationActionDefinition>>,
    },
    SetMenus {
        menus: Vec<MenuDefinition>,
    },
    SetKeybindings {
        bindings: Vec<KeybindingDefinition>,
    },
    SetClosePolicy {
        policy: String,
    },
    ResolveCloseRequest {
        request_id: u32,
        allow: bool,
    },
    ReadTextFile {
        path: String,
    },
    WriteTextFile {
        path: String,
        content: String,
    },
    ClipboardWriteImage {
        image: ClipboardImage,
    },
    ClipboardReadImage,
    LoadFont {
        path: String,
    },
    MinimizeWindow,
    GetWindowBounds,
    GetWindowState,
    ActivateWindow,
    GetScrollOffset,
    ScrollToOffset {
        offset: f32,
    },
    ConfigureApplication {
        keep_alive: bool,
        quit: bool,
        acknowledged_sequence: u32,
    },
    CancelNative {
        request_id: u32,
    },
    InvokeNative {
        module_id: [u8; 16],
        module_digest: [u8; 32],
        function_id: u32,
        args: Vec<u8>,
    },
}

impl CommandOperation {
    pub const fn kind(&self) -> CommandKind {
        match self {
            Self::Focus => CommandKind::Focus,
            Self::Blur => CommandKind::Blur,
            Self::SetSelection { .. } => CommandKind::SetSelection,
            Self::ScrollToIndex { .. } => CommandKind::ScrollToIndex,
            Self::ScrollToEnd => CommandKind::ScrollToEnd,
            Self::SetTitle { .. } => CommandKind::SetTitle,
            Self::ResizeWindow { .. } => CommandKind::ResizeWindow,
            Self::ZoomWindow => CommandKind::ZoomWindow,
            Self::ToggleFullscreen => CommandKind::ToggleFullscreen,
            Self::OpenUrl { .. } => CommandKind::OpenUrl,
            Self::FocusNext => CommandKind::FocusNext,
            Self::FocusPrev => CommandKind::FocusPrev,
            Self::GetWindowSize => CommandKind::GetWindowSize,
            Self::GetFocus => CommandKind::GetFocus,
            Self::ClipboardWrite { .. } => CommandKind::ClipboardWrite,
            Self::ClipboardRead => CommandKind::ClipboardRead,
            Self::OpenSurface { .. } => CommandKind::OpenSurface,
            Self::FileDialogOpen { .. } => CommandKind::FileDialogOpen,
            Self::FileDialogSave { .. } => CommandKind::FileDialogSave,
            Self::ShowNotification { .. } => CommandKind::ShowNotification,
            Self::SetMenus { .. } => CommandKind::SetMenus,
            Self::SetKeybindings { .. } => CommandKind::SetKeybindings,
            Self::SetClosePolicy { .. } => CommandKind::SetClosePolicy,
            Self::ResolveCloseRequest { .. } => CommandKind::ResolveCloseRequest,
            Self::ReadTextFile { .. } => CommandKind::ReadTextFile,
            Self::WriteTextFile { .. } => CommandKind::WriteTextFile,
            Self::ClipboardWriteImage { .. } => CommandKind::ClipboardWriteImage,
            Self::ClipboardReadImage => CommandKind::ClipboardReadImage,
            Self::LoadFont { .. } => CommandKind::LoadFont,
            Self::MinimizeWindow => CommandKind::MinimizeWindow,
            Self::GetWindowBounds => CommandKind::GetWindowBounds,
            Self::GetWindowState => CommandKind::GetWindowState,
            Self::ActivateWindow => CommandKind::ActivateWindow,
            Self::GetScrollOffset => CommandKind::GetScrollOffset,
            Self::ScrollToOffset { .. } => CommandKind::ScrollToOffset,
            Self::InvokeNative { .. } => CommandKind::InvokeNative,
            Self::ConfigureApplication { .. } => CommandKind::ConfigureApplication,
            Self::CancelNative { .. } => CommandKind::CancelNative,
        }
    }
}

/// A bounded encoded image exchanged through the native clipboard.
///
/// Format codes are `1=png`, `2=jpeg`, `3=gif`, and `4=svg`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardImage {
    pub format: u32,
    pub bytes: Vec<u8>,
}

/// Optional typed data returned by a command. The tag is part of the wire
/// contract: `1=number` (a u32 count or identifier), `2=window-size pair`,
/// `3=boolean`, `4=clipboard/path text` (also the metadata family returned by
/// LoadFont), `5=selected paths`, `6=file text`, `7=clipboard image`,
/// `8=window bounds`, `9=window state`, `10=VirtualList logical scroll
/// offset in pixels`, and `11=native invocation bytes`.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandValue {
    Number(u32),
    Pair((f32, f32)),
    Bool(bool),
    Text(String),
    Paths(Vec<String>),
    FileText(String),
    Image(ClipboardImage),
    Bounds((f32, f32, f32, f32)),
    WindowState((bool, bool)),
    ScrollOffset(f32),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandResult {
    pub request_id: u32,
    pub command: CommandKind,
    pub node_id: u32,
    pub success: bool,
    pub error: Option<String>,
    /// `None` indicates that the command has no typed return value.
    pub value: Option<CommandValue>,
}

impl Command {
    pub fn new(meta: CommandMeta, operation: CommandOperation) -> Self {
        Self { meta, operation }
    }

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
    pub selectable: bool,
    pub tooltip: Option<String>,
    pub accepts_pointer_move: bool,
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
            selectable: false,
            tooltip: None,
            accepts_pointer_move: false,
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
    pub expanded: Option<bool>,
    pub level: Option<u32>,
    pub live: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HostProperties {
    TextInput(TextInputProperties),
    VirtualList(VirtualListProperties),
    Image(ImageProperties),
    Drag(DragProperties),
    Extension(ExtensionProperties),
    Icon(IconProperties),
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
    pub selection_reversed: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ImageProperties {
    pub source: String,
    pub object_fit: u32,
    pub fallback_source: Option<String>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct IconProperties {
    pub name: String,
    pub size: f32,
    pub color_rgba: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DragProperties {
    pub drag_type: Option<String>,
    pub export_files: Option<Vec<String>>,
    pub accepts_drag_over: bool,
    pub accepts_drop: bool,
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
impl TryFrom<u32> for Easing {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Linear),
            1 => Ok(Self::EaseIn),
            2 => Ok(Self::EaseOut),
            3 => Ok(Self::EaseInOut),
            _ => Err(()),
        }
    }
}

impl From<Easing> for u32 {
    fn from(value: Easing) -> Self {
        match value {
            Easing::Linear => 0,
            Easing::EaseIn => 1,
            Easing::EaseOut => 2,
            Easing::EaseInOut => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    pub duration_ms: u32,
    pub delay_ms: u32,
    pub easing: Easing,
    pub properties: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub color_rgba: u32,
    pub inset: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Style {
    pub border_top_color: Option<u32>,
    pub border_right_color: Option<u32>,
    pub border_bottom_color: Option<u32>,
    pub border_left_color: Option<u32>,

    pub linear_gradient: Option<LinearGradient>,
    pub padding_top: Option<f32>,
    pub padding_right: Option<f32>,
    pub padding_bottom: Option<f32>,
    pub padding_left: Option<f32>,
    pub border_top_width: Option<f32>,
    pub border_right_width: Option<f32>,
    pub border_bottom_width: Option<f32>,
    pub border_left_width: Option<f32>,
    pub border_top_left_radius: Option<f32>,
    pub border_top_right_radius: Option<f32>,
    pub border_bottom_right_radius: Option<f32>,
    pub border_bottom_left_radius: Option<f32>,
    pub width_percent: Option<f32>,
    pub height_percent: Option<f32>,
    pub flex_wrap: Option<FlexWrapCode>,
    pub grid_columns: Option<u32>,
    pub grid_rows: Option<u32>,
    pub grid_column_span: Option<u32>,
    pub grid_row_span: Option<u32>,

    pub width: Option<f32>,
    pub height: Option<f32>,
    pub flex_direction: Option<FlexDirectionCode>,
    pub flex_grow: Option<f32>,
    pub padding: Option<f32>,
    pub gap: Option<f32>,
    pub justify_content: Option<JustifyContentCode>,
    pub align_items: Option<AlignItemsCode>,
    pub border_radius: Option<f32>,
    pub border_width: Option<f32>,
    pub border_color_rgba: Option<u32>,
    pub font_size: Option<f32>,
    pub font_weight: Option<FontWeightCode>,
    pub background_rgba: Option<u32>,
    pub color_rgba: Option<u32>,
    pub opacity: Option<f32>,
    pub transition: Option<Transition>,
    pub overflow: Option<OverflowCode>,
    pub line_clamp: Option<u32>,
    pub text_overflow: Option<TextOverflowCode>,
    pub margin_top: Option<f32>,
    pub margin_right: Option<f32>,
    pub margin_bottom: Option<f32>,
    pub margin_left: Option<f32>,
    pub font_style: Option<FontStyleCode>,
    pub text_decoration: Option<TextDecorationCode>,
    pub line_height: Option<f32>,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub align_self: Option<AlignSelfCode>,
    pub position: Option<PositionCode>,
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub cursor: Option<CursorCode>,
    pub text_align: Option<TextAlignCode>,
    pub box_shadows: Option<Vec<BoxShadow>>,
    pub font_family: Option<String>,
}

macro_rules! closed_code {
    ($name:ident { $($variant:ident = $value:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($variant = $value),+
        }

        impl TryFrom<u32> for $name {
            type Error = ();

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(()),
                }
            }
        }

        impl From<$name> for u32 {
            fn from(value: $name) -> Self {
                value as u32
            }
        }
    };
}

closed_code!(FlexDirectionCode {
    Row = 1,
    Column = 2,
    RowReverse = 3,
    ColumnReverse = 4,
});
closed_code!(JustifyContentCode {
    FlexStart = 1,
    Center = 2,
    FlexEnd = 3,
    SpaceBetween = 4,
    SpaceAround = 5,
    SpaceEvenly = 6,
});
closed_code!(AlignItemsCode {
    FlexStart = 1,
    Center = 2,
    FlexEnd = 3,
    Stretch = 4,
    Baseline = 5,
});
closed_code!(FontWeightCode {
    Normal = 400,
    Medium = 500,
    Semibold = 600,
    Bold = 700,
    Heavy = 900,
});
closed_code!(OverflowCode {
    Visible = 1,
    Hidden = 2,
    Scroll = 3,
});
closed_code!(TextOverflowCode {
    Clip = 1,
    Ellipsis = 2,
});
closed_code!(FontStyleCode {
    Normal = 0,
    Italic = 1,
});
closed_code!(TextDecorationCode {
    None = 0,
    Underline = 1,
    LineThrough = 2,
});
closed_code!(AlignSelfCode {
    Start = 1,
    End = 2,
    FlexStart = 3,
    FlexEnd = 4,
    Center = 5,
    Baseline = 6,
    Stretch = 7,
});
closed_code!(PositionCode {
    Relative = 0,
    Absolute = 1,
    Overlay = 2,
});
closed_code!(CursorCode {
    Default = 0,
    Text = 1,
    Pointer = 2,
    Grab = 3,
    Grabbing = 4,
    NotAllowed = 5,
    ContextMenu = 6,
    Crosshair = 7,
    VerticalText = 8,
    Alias = 9,
    Copy = 10,
    NoDrop = 11,
    Move = 12,
    EwResize = 13,
    NsResize = 14,
    NeswResize = 15,
    NwseResize = 16,
    ColResize = 17,
    RowResize = 18,
});
closed_code!(TextAlignCode {
    Left = 1,
    Center = 2,
    Right = 3,
});
closed_code!(ObjectFitCode {
    Fill = 1,
    Contain = 2,
    Cover = 3,
    ScaleDown = 4,
    None = 5,
});
closed_code!(AccessibilityRoleCode {
    Unspecified = 0,
    Generic = 1,
    Button = 2,
    Label = 3,
    TextInput = 4,
    CheckBox = 5,
    Heading = 6,
    Link = 7,
    Status = 8,
    Alert = 9,
    Group = 10,
    List = 11,
    ListItem = 12,
    Dialog = 13,
});
closed_code!(ClipboardImageFormatCode {
    Png = 1,
    Jpeg = 2,
    Gif = 3,
    Svg = 4,
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputEvent {
    pub text: String,
    pub selection_start: u32,
    pub selection_end: u32,
    pub marked_start: Option<u32>,
    pub marked_end: Option<u32>,
    pub edit_seq: u32,
    pub reversed: bool,
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
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointerMoveEvent {
    pub x: f32,
    pub y: f32,
    pub modifiers: Vec<String>,
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
    ApplicationActivation {
        target_surface_id: u32,
        reason: String,
        urls: Vec<String>,
    },
    Press,
    TextInputChange(TextInputEvent),
    TextInputSelection(TextInputEvent),
    Focus,
    FocusTextInput(TextInputEvent),
    Blur,
    BlurTextInput(TextInputEvent),
    Hover,
    SurfaceClosed,
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
    PointerMove(PointerMoveEvent),
    Scroll(ScrollEvent),
    Submit {
        text: String,
    },
    WindowResize {
        width: f32,
        height: f32,
        scale_factor: f32,
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
    NotificationResponse(NotificationResponseEvent),
    PointerDownOutside {
        x: f32,
        y: f32,
    },
    CloseRequested {
        request_id: u32,
    },
    Extension {
        event_id: u32,
        fields: Vec<ExtensionField>,
    },
}

impl EventPayload {
    pub const fn event_kind(&self) -> EventKind {
        match self {
            Self::Press => EventKind::Press,
            Self::TextInputChange(_) => EventKind::Change,
            Self::TextInputSelection(_) => EventKind::Selection,
            Self::Focus | Self::FocusTextInput(_) => EventKind::Focus,
            Self::Blur | Self::BlurTextInput(_) => EventKind::Blur,
            Self::Hover => EventKind::Hover,
            Self::SurfaceClosed => EventKind::SurfaceClosed,
            Self::CommandResult(_) => EventKind::CommandResult,
            Self::VisibleRange { .. } => EventKind::VisibleRange,
            Self::AnimationComplete { .. } => EventKind::AnimationComplete,
            Self::Key(_) => EventKind::Key,
            Self::Pointer(_) | Self::PointerMove(_) => EventKind::Pointer,
            Self::Scroll(_) => EventKind::Scroll,
            Self::Submit { .. } => EventKind::Submit,
            Self::WindowResize { .. } => EventKind::WindowResize,
            Self::WindowActivation { .. } => EventKind::WindowActivation,
            Self::EventAction { .. } => EventKind::Action,
            Self::WindowAppearance { .. } => EventKind::WindowAppearance,
            Self::Layout { .. } => EventKind::Layout,
            Self::DragOver { .. } | Self::DragDrop { .. } | Self::ExternalFileDrop { .. } => {
                EventKind::Drag
            }
            Self::NotificationResponse(_) => EventKind::NotificationResponse,
            Self::PointerDownOutside { .. } => EventKind::PointerDownOutside,
            Self::CloseRequested { .. } => EventKind::CloseRequested,
            Self::Extension { .. } => EventKind::Extension,
            Self::ApplicationActivation { .. } => EventKind::ApplicationActivation,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationResponseEvent {
    pub tag: String,
    pub action_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventMeta {
    pub surface_id: u32,
    pub epoch: u32,
    pub revision: u32,
    pub sequence: u32,
    pub node_id: u32,
    pub listener_id: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub meta: EventMeta,
    pub payload: EventPayload,
}

impl Event {
    pub fn new(meta: EventMeta, payload: EventPayload) -> Self {
        Self { meta, payload }
    }
    pub const fn event_kind(&self) -> EventKind {
        self.payload.event_kind()
    }

    pub fn press(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
    ) -> Self {
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            EventPayload::Press,
        )
    }

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
        let payload = match event_type {
            EVENT_CHANGE => EventPayload::TextInputChange(payload),
            EVENT_SELECTION => EventPayload::TextInputSelection(payload),
            EVENT_FOCUS => EventPayload::FocusTextInput(payload),
            EVENT_BLUR => EventPayload::BlurTextInput(payload),
            _ => panic!("invalid text input event kind {event_type}"),
        };
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            payload,
        )
    }

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
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            EventPayload::Key(KeyEvent {
                key,
                modifiers,
                action,
            }),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn pointer(
        _event_type: u32,
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
        x: f32,
        y: f32,
    ) -> Self {
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            EventPayload::Pointer(PointerEvent {
                button,
                modifiers,
                action,
                click_count,
                x,
                y,
            }),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn pointer_move(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        x: f32,
        y: f32,
        modifiers: Vec<String>,
    ) -> Self {
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            EventPayload::PointerMove(PointerMoveEvent { x, y, modifiers }),
        )
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
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            EventPayload::Scroll(ScrollEvent {
                delta_kind,
                dx,
                dy,
                x,
                y,
                modifiers,
            }),
        )
    }

    pub fn hover(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
    ) -> Self {
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            EventPayload::Hover,
        )
    }

    pub fn focus(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        focused: bool,
    ) -> Self {
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            if focused {
                EventPayload::Focus
            } else {
                EventPayload::Blur
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn pointer_down_outside(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        x: f32,
        y: f32,
    ) -> Self {
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            EventPayload::PointerDownOutside { x, y },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn submit(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        text: String,
    ) -> Self {
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            EventPayload::Submit { text },
        )
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
        Self::window_resize_with_scale(
            surface_id,
            epoch,
            revision,
            sequence,
            node_id,
            listener_id,
            width,
            height,
            1.0,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn window_resize_with_scale(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        width: f32,
        height: f32,
        scale_factor: f32,
    ) -> Self {
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            EventPayload::WindowResize {
                width,
                height,
                scale_factor,
            },
        )
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
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            EventPayload::WindowActivation { active },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn extension(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        node_id: u32,
        listener_id: u32,
        event_id: u32,
        fields: Vec<ExtensionField>,
    ) -> Self {
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            EventPayload::Extension { event_id, fields },
        )
    }

    pub fn window_appearance(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        appearance: WindowAppearance,
    ) -> Self {
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id: 1,
                listener_id: 0,
            },
            EventPayload::WindowAppearance { appearance },
        )
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
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            EventPayload::Layout {
                x,
                y,
                width,
                height,
            },
        )
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
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            payload,
        )
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
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id: 1,
                listener_id: 0,
            },
            EventPayload::EventAction { action },
        )
    }

    pub fn notification_response(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        tag: String,
        action_id: Option<String>,
    ) -> Self {
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id: 1,
                listener_id: 0,
            },
            EventPayload::NotificationResponse(NotificationResponseEvent { tag, action_id }),
        )
    }

    pub fn close_requested(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        request_id: u32,
    ) -> Self {
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id: 1,
                listener_id: 0,
            },
            EventPayload::CloseRequested { request_id },
        )
    }

    pub fn surface_closed(surface_id: u32, epoch: u32, revision: u32, sequence: u32) -> Self {
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id: 0,
                listener_id: 0,
            },
            EventPayload::SurfaceClosed,
        )
    }

    pub fn command_result(
        surface_id: u32,
        epoch: u32,
        revision: u32,
        sequence: u32,
        result: CommandResult,
    ) -> Self {
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id: result.node_id,
                listener_id: 0,
            },
            EventPayload::CommandResult(result),
        )
    }

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
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            EventPayload::VisibleRange { start, end },
        )
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
        Self::new(
            EventMeta {
                surface_id,
                epoch,
                revision,
                sequence,
                node_id,
                listener_id,
            },
            EventPayload::AnimationComplete { generation },
        )
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        wire::encode_event(self)
    }

    pub(crate) fn encode_frame_into(&self, output: &mut Vec<u8>) -> Result<(), ProtocolError> {
        wire::encode_event_frame(self, output)
    }

    pub(crate) fn frame_size(&self) -> Result<usize, ProtocolError> {
        wire::event_frame_size(self)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, ProtocolError> {
        wire::decode_event(payload)
    }
}
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("invalid Bebop payload: {0}")]
    Decode(#[source] bebop::DeserializeError),
    #[error("bounded Bebop decode rejected: {0}")]
    BoundedDecode(String),
    #[error("could not encode Bebop payload: {0}")]
    Encode(#[source] bebop::SerializeError),
    #[error("transport I/O error: {0}")]
    Io(#[source] io::Error),
    #[error("Bebop payload has {0} trailing bytes")]
    TrailingBytes(usize),
    #[error(
        "protocol version mismatch: renderer speaks protocol v{received}; this host binary speaks protocol v{expected} — update the host binary / pin @solid-gpui/core to a v{expected} release"
    )]
    UnsupportedProtocol { received: u32, expected: u32 },
    #[error("unexpected message type {0}")]
    WrongMessageType(u32),
    #[error("unknown event type {0}")]
    UnknownEvent(u32),
    #[error("semantic protocol error at {path}: {source}")]
    Semantic {
        path: String,
        #[source]
        source: Box<ProtocolError>,
    },
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

impl ProtocolError {
    pub(crate) fn at(self, path: impl Into<String>) -> Self {
        Self::Semantic {
            path: path.into(),
            source: Box::new(self),
        }
    }
}

pub enum DecodedMessage {
    Snapshot(Snapshot),
    Patch(Patch),
    Command(Command),
}

pub fn decode_message(payload: &[u8]) -> Result<DecodedMessage, ProtocolError> {
    wire::decode_message(payload)
}

pub(crate) type PayloadClassification = (u32, Option<u32>, Option<u32>, Option<u32>, Option<bool>);
pub(crate) fn classify_payload(payload: &[u8]) -> Result<PayloadClassification, ProtocolError> {
    wire::classify_payload(payload)
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
    let length = payload.len() as u32;
    writer
        .write_all(&length.to_le_bytes())
        .and_then(|_| writer.write_all(payload))
        .and_then(|_| writer.flush())
        .map_err(ProtocolError::Io)
}

closed_code!(FlexWrapCode { NoWrap = 0, Wrap = 1, WrapReverse = 2 });

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearGradient {
    pub angle: f32,
    pub start_color: u32,
    pub start_position: f32,
    pub end_color: u32,
    pub end_position: f32,
}
impl LinearGradient {
    pub(crate) fn is_valid(&self) -> bool {
        self.angle.is_finite()
            && (0.0..=360.0).contains(&self.angle)
            && self.start_position.is_finite()
            && self.end_position.is_finite()
            && (0.0..=1.0).contains(&self.start_position)
            && (0.0..=1.0).contains(&self.end_position)
            && self.start_position < self.end_position
    }
}
