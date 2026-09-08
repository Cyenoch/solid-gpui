import {
  BODY_TAGS,
  COMMAND_KIND_CODES,
  COMMAND_KINDS as SCHEMA_COMMAND_KINDS,
  EVENT_KIND_CODES,
  EVENT_PAYLOAD_TAGS,
} from "./generated/schema-facts";

export const PROTOCOL_VERSION = 5 as const;

export const SNAPSHOT_KIND = BODY_TAGS.Snapshot;
export const EVENT_KIND = BODY_TAGS.Event;
export const PATCH_KIND = BODY_TAGS.Patch;
export const COMMAND_KIND = BODY_TAGS.Command;

export const MAX_FRAME_SIZE = 16 * 1024 * 1024;
export const MAX_CLIPBOARD_TEXT_BYTES = 1 << 20;
export const MAX_CLIPBOARD_IMAGE_BYTES = MAX_FRAME_SIZE - 1024;
export const MAX_FILE_WRITE_BYTES = MAX_FRAME_SIZE - 1024;
export const MAX_FILE_READ_BYTES = MAX_FRAME_SIZE - 1024;
export const MAX_NATIVE_CALL_BYTES = 1_048_576;
export const MAX_IMAGE_SOURCE_BYTES = 1 << 20;
export const EVENT_EXTENSION = EVENT_KIND_CODES.Extension;
export const MAX_EXTENSION_FIELDS = 256;
export const MAX_EXTENSION_EVENTS = 256;
export const MAX_EXTENSION_TEXT_BYTES = 1 << 20;
export const MAX_EXTENSION_BYTES = 1 << 20;

export const EVENT_PRESS = EVENT_KIND_CODES.Press;
export const EVENT_CHANGE = EVENT_KIND_CODES.Change;
export const EVENT_SELECTION = EVENT_KIND_CODES.Selection;
export const EVENT_FOCUS = EVENT_KIND_CODES.Focus;
export const EVENT_BLUR = EVENT_KIND_CODES.Blur;
export const EVENT_COMMAND_RESULT = EVENT_KIND_CODES.CommandResult;
export const EVENT_VISIBLE_RANGE = EVENT_KIND_CODES.VisibleRange;
export const EVENT_ANIMATION_COMPLETE = EVENT_KIND_CODES.AnimationComplete;
export const EVENT_KEY = EVENT_KIND_CODES.Key;
export const EVENT_POINTER = EVENT_KIND_CODES.Pointer;
export const EVENT_HOVER = EVENT_KIND_CODES.Hover;
export const EVENT_SCROLL = EVENT_KIND_CODES.Scroll;
export const EVENT_SUBMIT = EVENT_KIND_CODES.Submit;
export const EVENT_WINDOW_RESIZE = EVENT_KIND_CODES.WindowResize;
export const EVENT_WINDOW_ACTIVATION = EVENT_KIND_CODES.WindowActivation;
export const EVENT_SURFACE_CLOSED = EVENT_KIND_CODES.SurfaceClosed;
export const EVENT_ACTION = EVENT_KIND_CODES.Action;
export const EVENT_WINDOW_APPEARANCE = EVENT_KIND_CODES.WindowAppearance;
export const EVENT_LAYOUT = EVENT_KIND_CODES.Layout;
export const EVENT_DRAG = EVENT_KIND_CODES.Drag;
export const EVENT_NOTIFICATION_RESPONSE = EVENT_KIND_CODES.NotificationResponse;
export const EVENT_POINTER_DOWN_OUTSIDE = EVENT_KIND_CODES.PointerDownOutside;
export const EVENT_CLOSE_REQUESTED = EVENT_KIND_CODES.CloseRequested;

export const DRAG_OVER = EVENT_PAYLOAD_TAGS.DragOverEvent;
export const DRAG_DROP = EVENT_PAYLOAD_TAGS.DragDropEvent;
export const DRAG_EXTERNAL_FILE_DROP = EVENT_PAYLOAD_TAGS.ExternalFileDropEvent;

export const EVENT_POINTER_DOWN = 1 as const;
export const EVENT_POINTER_UP = 2 as const;
export const POINTER_BUTTON_LEFT = 1 as const;
export const POINTER_BUTTON_RIGHT = 2 as const;
export const POINTER_BUTTON_MIDDLE = 3 as const;
export const POINTER_BUTTON_BACK = 4 as const;
export const POINTER_BUTTON_FORWARD = 5 as const;

export const SCROLL_DELTA_PIXELS = 1 as const;
export const SCROLL_DELTA_LINES = 2 as const;
export const EVENT_KEY_DOWN = 1 as const;
export const EVENT_KEY_REPEAT = 2 as const;
export const EVENT_KEY_UP = 3 as const;

export const COMMAND_FOCUS = COMMAND_KIND_CODES.Focus;
export const COMMAND_BLUR = COMMAND_KIND_CODES.Blur;
export const COMMAND_SET_SELECTION = COMMAND_KIND_CODES.SetSelection;
export const COMMAND_SCROLL_TO_INDEX = COMMAND_KIND_CODES.ScrollToIndex;
export const COMMAND_SCROLL_TO_END = COMMAND_KIND_CODES.ScrollToEnd;
export const COMMAND_GET_SCROLL_OFFSET = COMMAND_KIND_CODES.GetScrollOffset;
export const COMMAND_SCROLL_TO_OFFSET = COMMAND_KIND_CODES.ScrollToOffset;
export const COMMAND_SET_TITLE = COMMAND_KIND_CODES.SetTitle;
export const COMMAND_RESIZE_WINDOW = COMMAND_KIND_CODES.ResizeWindow;
export const COMMAND_ZOOM_WINDOW = COMMAND_KIND_CODES.ZoomWindow;
export const COMMAND_TOGGLE_FULLSCREEN = COMMAND_KIND_CODES.ToggleFullscreen;
export const COMMAND_OPEN_URL = COMMAND_KIND_CODES.OpenUrl;
export const COMMAND_FOCUS_NEXT = COMMAND_KIND_CODES.FocusNext;
export const COMMAND_FOCUS_PREV = COMMAND_KIND_CODES.FocusPrev;
export const COMMAND_GET_WINDOW_SIZE = COMMAND_KIND_CODES.GetWindowSize;
export const COMMAND_GET_FOCUS = COMMAND_KIND_CODES.GetFocus;
export const COMMAND_CLIPBOARD_WRITE = COMMAND_KIND_CODES.ClipboardWrite;
export const COMMAND_CLIPBOARD_READ = COMMAND_KIND_CODES.ClipboardRead;
export const COMMAND_OPEN_SURFACE = COMMAND_KIND_CODES.OpenSurface;
export const COMMAND_FILE_DIALOG_OPEN = COMMAND_KIND_CODES.FileDialogOpen;
export const COMMAND_FILE_DIALOG_SAVE = COMMAND_KIND_CODES.FileDialogSave;
export const COMMAND_SHOW_NOTIFICATION = COMMAND_KIND_CODES.ShowNotification;
export const COMMAND_SET_MENUS = COMMAND_KIND_CODES.SetMenus;
export const COMMAND_SET_KEYBINDINGS = COMMAND_KIND_CODES.SetKeybindings;
export const COMMAND_SET_CLOSE_POLICY = COMMAND_KIND_CODES.SetClosePolicy;
export const COMMAND_RESOLVE_CLOSE_REQUEST = COMMAND_KIND_CODES.ResolveCloseRequest;
export const COMMAND_READ_TEXT_FILE = COMMAND_KIND_CODES.ReadTextFile;
export const COMMAND_WRITE_TEXT_FILE = COMMAND_KIND_CODES.WriteTextFile;
export const COMMAND_CLIPBOARD_WRITE_IMAGE = COMMAND_KIND_CODES.ClipboardWriteImage;
export const COMMAND_CLIPBOARD_READ_IMAGE = COMMAND_KIND_CODES.ClipboardReadImage;
export const COMMAND_LOAD_FONT = COMMAND_KIND_CODES.LoadFont;
export const COMMAND_MINIMIZE_WINDOW = COMMAND_KIND_CODES.MinimizeWindow;
export const COMMAND_GET_WINDOW_BOUNDS = COMMAND_KIND_CODES.GetWindowBounds;
export const COMMAND_GET_WINDOW_STATE = COMMAND_KIND_CODES.GetWindowState;
export const COMMAND_ACTIVATE_WINDOW = COMMAND_KIND_CODES.ActivateWindow;
export const COMMAND_INVOKE_NATIVE = COMMAND_KIND_CODES.InvokeNative;
export const COMMAND_CANCEL_NATIVE = COMMAND_KIND_CODES.CancelNative;

export const CLIPBOARD_IMAGE_FORMAT_PNG = 1 as const;
export const CLIPBOARD_IMAGE_FORMAT_JPEG = 2 as const;
export const CLIPBOARD_IMAGE_FORMAT_GIF = 3 as const;
export const CLIPBOARD_IMAGE_FORMAT_SVG = 4 as const;
export type ClipboardImageFormat = "png" | "jpeg" | "gif" | "svg";

export const IMAGE_OBJECT_FIT_SCALE_DOWN = 4 as const;
export const IMAGE_OBJECT_FIT_NONE = 5 as const;

export const UPDATE_STYLE = 1 as const;
export const UPDATE_TEXT = 2 as const;
export const UPDATE_LISTENER = 4 as const;
export const UPDATE_PROPERTIES = 8 as const;
export const UPDATE_ACCESSIBILITY = 16 as const;
export const UPDATE_FOCUSABLE = 32 as const;
export const UPDATE_SELECTABLE = 64 as const;
export const UPDATE_TOOLTIP = 128 as const;
export const UPDATE_POINTER_MOVE = 256 as const;
export const COMMAND_KINDS = SCHEMA_COMMAND_KINDS;

export const KEY_MODIFIER_NAMES = {
  cmd: true,
  ctrl: true,
  alt: true,
  shift: true,
  function: true,
} as const;

export const EVENT_APPLICATION_ACTIVATION = EVENT_KIND_CODES.ApplicationActivation;
export const COMMAND_CONFIGURE_APPLICATION = COMMAND_KIND_CODES.ConfigureApplication;
