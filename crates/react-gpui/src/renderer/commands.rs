use std::borrow::Cow;
use std::path::Path;
use std::sync::atomic::Ordering;

use gpui::{
    AppContext, ClipboardEntry, ClipboardItem, Context, Image, ImageFormat, ListOffset,
    Menu as GpuiMenu, MenuItem as GpuiMenuItem, PathPromptOptions, Point, SystemNotification,
    SystemNotificationAction, Window, px, size,
};

use super::ReactRoot;
use crate::protocol::{
    COMMAND_ACTIVATE_WINDOW, COMMAND_BLUR, COMMAND_CLIPBOARD_READ, COMMAND_CLIPBOARD_READ_IMAGE,
    COMMAND_CLIPBOARD_WRITE, COMMAND_CLIPBOARD_WRITE_IMAGE, COMMAND_FILE_DIALOG_OPEN,
    COMMAND_FILE_DIALOG_SAVE, COMMAND_FOCUS, COMMAND_FOCUS_NEXT, COMMAND_FOCUS_PREV,
    COMMAND_GET_FOCUS, COMMAND_GET_SCROLL_OFFSET, COMMAND_GET_WINDOW_BOUNDS,
    COMMAND_GET_WINDOW_SIZE, COMMAND_GET_WINDOW_STATE, COMMAND_LOAD_FONT, COMMAND_MINIMIZE_WINDOW,
    COMMAND_OPEN_URL, COMMAND_READ_TEXT_FILE, COMMAND_RESIZE_WINDOW, COMMAND_SCROLL_TO_END,
    COMMAND_SCROLL_TO_INDEX, COMMAND_SCROLL_TO_OFFSET, COMMAND_SET_MENUS, COMMAND_SET_SELECTION,
    COMMAND_SET_TITLE, COMMAND_SHOW_NOTIFICATION, COMMAND_TOGGLE_FULLSCREEN,
    COMMAND_WRITE_TEXT_FILE, COMMAND_ZOOM_WINDOW, ClipboardImage, Command, CommandResult,
    CommandValue, EVENT_SELECTION, Event, HostProperties, MAX_CLIPBOARD_IMAGE_BYTES,
    MAX_CLIPBOARD_TEXT_BYTES, MAX_FILE_READ_BYTES, MAX_FILE_WRITE_BYTES, MAX_WINDOW_DIMENSION,
    MenuAction, MenuDefinition, MenuItemDefinition,
};
use crate::transport::send_event_or_exit;

fn file_error(error: std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::NotFound => "file not found".to_owned(),
        std::io::ErrorKind::PermissionDenied => "permission denied".to_owned(),
        std::io::ErrorKind::IsADirectory => "path is a directory".to_owned(),
        _ => {
            let detail = error.to_string();
            if detail.len() <= 256 {
                format!("file operation failed: {detail}")
            } else {
                "file operation failed".to_owned()
            }
        }
    }
}

fn font_family(bytes: &[u8]) -> Result<String, String> {
    let face = ttf_parser::Face::parse(bytes, 0)
        .map_err(|_| "font is malformed or unsupported".to_owned())?;
    let mut family = None;
    for name in face.names() {
        if name.name_id == ttf_parser::name_id::TYPOGRAPHIC_FAMILY {
            if let Some(value) = name.to_string()
                && !value.is_empty()
                && value.chars().count() <= 64
                && !value.chars().any(char::is_control)
            {
                return Ok(value);
            }
        } else if name.name_id == ttf_parser::name_id::FAMILY
            && family.is_none()
            && let Some(value) = name.to_string()
            && !value.is_empty()
            && value.chars().count() <= 64
            && !value.chars().any(char::is_control)
        {
            family = Some(value);
        }
    }
    family.ok_or_else(|| "font has no usable family name".to_owned())
}
fn gpui_image_format(format: u32) -> Option<ImageFormat> {
    match format {
        1 => Some(ImageFormat::Png),
        2 => Some(ImageFormat::Jpeg),
        3 => Some(ImageFormat::Gif),
        4 => Some(ImageFormat::Svg),
        _ => None,
    }
}

fn clipboard_image_value(image: &Image) -> Option<CommandValue> {
    let format = match image.format() {
        ImageFormat::Png => 1,
        ImageFormat::Jpeg => 2,
        ImageFormat::Gif => 3,
        ImageFormat::Svg => 4,
        _ => return None,
    };
    if image.bytes().is_empty() || image.bytes().len() > MAX_CLIPBOARD_IMAGE_BYTES {
        return None;
    }
    Some(CommandValue::Image(ClipboardImage {
        format,
        bytes: image.bytes().to_vec(),
    }))
}

use crate::tree::{KIND_PRESSABLE, KIND_TEXT_INPUT, KIND_VIEW, KIND_VIRTUAL_LIST};

impl ReactRoot {
    fn menu_item(item: MenuItemDefinition) -> GpuiMenuItem {
        match item {
            MenuItemDefinition::Separator => GpuiMenuItem::separator(),
            MenuItemDefinition::Action {
                name,
                disabled,
                checked,
            } => GpuiMenuItem::action(name.clone(), MenuAction { name })
                .checked(checked)
                .disabled(disabled),
            MenuItemDefinition::Submenu(menu) => GpuiMenuItem::submenu(
                GpuiMenu::new(menu.title).items(menu.items.into_iter().map(Self::menu_item)),
            ),
        }
    }

    fn set_menus(&self, menus: Vec<MenuDefinition>, cx: &mut Context<Self>) {
        cx.set_menus(menus.into_iter().map(|menu| {
            GpuiMenu::new(menu.title).items(menu.items.into_iter().map(Self::menu_item))
        }));
    }

    fn spawn_open_file_dialog(&self, command: Command, cx: &mut Context<Self>) {
        let Some((directories, multiple)) = command.payload else {
            return;
        };
        let title = command.title.clone().unwrap_or_default();
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: directories == 0,
            directories: directories == 1,
            multiple: multiple == 1,
            prompt: Some(title.into()),
        });
        let entity = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            let (success, error, value) = match receiver.await {
                Ok(Ok(None)) => (true, None, None),
                Ok(Ok(Some(paths))) if !paths.is_empty() => {
                    let mut values = Vec::with_capacity(paths.len());
                    let mut conversion_error = None;
                    for path in paths {
                        match path.into_os_string().into_string() {
                            Ok(path) if !path.is_empty() => values.push(path),
                            Ok(_) => {
                                conversion_error = Some("selected path is empty".to_owned());
                                break;
                            }
                            Err(_) => {
                                conversion_error =
                                    Some("selected path is not valid UTF-8".to_owned());
                                break;
                            }
                        }
                    }
                    if let Some(error) = conversion_error {
                        (false, Some(error), None)
                    } else {
                        (true, None, Some(CommandValue::Paths(values)))
                    }
                }
                Ok(Ok(Some(_))) => (
                    false,
                    Some("file dialog returned no paths".to_owned()),
                    None,
                ),
                Ok(Err(error)) => (false, Some(format!("file dialog failed: {error}")), None),
                Err(_) => (
                    false,
                    Some("file dialog response channel closed".to_owned()),
                    None,
                ),
            };
            let _ = entity.update(cx, |root, _| {
                root.emit_command_ack(
                    command.request_id,
                    command.kind,
                    command.node_id,
                    success,
                    error,
                    value,
                );
            });
        })
        .detach();
    }

    fn spawn_save_file_dialog(&self, command: Command, cx: &mut Context<Self>) {
        let suggested_name = command.title.clone().filter(|name| !name.is_empty());
        let receiver = cx.prompt_for_new_path(Path::new(""), suggested_name.as_deref());
        let entity = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            let (success, error, value) = match receiver.await {
                Ok(Ok(None)) => (true, None, None),
                Ok(Ok(Some(path))) => match path.into_os_string().into_string() {
                    Ok(path) if !path.is_empty() => (true, None, Some(CommandValue::Text(path))),
                    Ok(_) => (false, Some("selected path is empty".to_owned()), None),
                    Err(_) => (
                        false,
                        Some("selected path is not valid UTF-8".to_owned()),
                        None,
                    ),
                },
                Ok(Err(error)) => (false, Some(format!("file dialog failed: {error}")), None),
                Err(_) => (
                    false,
                    Some("file dialog response channel closed".to_owned()),
                    None,
                ),
            };
            let _ = entity.update(cx, |root, _| {
                root.emit_command_ack(
                    command.request_id,
                    command.kind,
                    command.node_id,
                    success,
                    error,
                    value,
                );
            });
        })
        .detach();
    }

    fn spawn_load_font(&self, command: Command, cx: &mut Context<Self>) {
        let path = command.title.clone().unwrap_or_default();
        let kind = command.kind;
        let request_id = command.request_id;
        let node_id = command.node_id;
        let entity = cx.weak_entity();
        let task = cx.background_spawn(async move {
            let metadata = std::fs::metadata(&path).map_err(file_error)?;
            if !metadata.is_file() {
                return Err("path is a directory".to_owned());
            }
            if metadata.len() > MAX_FILE_READ_BYTES as u64 {
                return Err("font file is too large".to_owned());
            }
            let bytes = std::fs::read(&path).map_err(file_error)?;
            if bytes.len() > MAX_FILE_READ_BYTES {
                return Err("font file is too large".to_owned());
            }
            let family = font_family(&bytes)?;
            Ok((bytes, family))
        });
        cx.spawn(async move |_, cx| {
            let (success, error, value) = match task.await {
                Ok((bytes, family)) => {
                    match cx.update(|app| app.text_system().add_fonts(vec![Cow::Owned(bytes)])) {
                        Ok(()) => {
                            cx.refresh();
                            (true, None, Some(CommandValue::Text(family)))
                        }
                        Err(error) => (
                            false,
                            Some(format!("font registration failed: {error}")),
                            None,
                        ),
                    }
                }
                Err(error) => (false, Some(error), None),
            };
            let _ = entity.update(cx, |root, _| {
                root.emit_command_ack(request_id, kind, node_id, success, error, value);
            });
        })
        .detach();
    }

    fn spawn_text_file_command(&self, command: Command, cx: &mut Context<Self>) {
        let path = command.title.clone().unwrap_or_default();
        let content = command.body.clone();
        let kind = command.kind;
        let request_id = command.request_id;
        let node_id = command.node_id;
        let entity = cx.weak_entity();
        let task = cx.background_spawn(async move {
            match kind {
                COMMAND_READ_TEXT_FILE => {
                    let metadata = std::fs::metadata(&path).map_err(file_error)?;
                    if !metadata.is_file() {
                        return Err("path is a directory".to_owned());
                    }
                    if metadata.len() > MAX_FILE_READ_BYTES as u64 {
                        return Err("file is too large".to_owned());
                    }
                    let bytes = std::fs::read(&path).map_err(file_error)?;
                    if bytes.len() > MAX_FILE_READ_BYTES {
                        return Err("file is too large".to_owned());
                    }
                    String::from_utf8(bytes)
                        .map(CommandValue::FileText)
                        .map_err(|_| "file is not valid UTF-8".to_owned())
                }
                COMMAND_WRITE_TEXT_FILE => {
                    let content = content.ok_or_else(|| "file content is required".to_owned())?;
                    if content.len() > MAX_FILE_WRITE_BYTES {
                        return Err("file content is too large".to_owned());
                    }
                    std::fs::write(&path, content.as_bytes()).map_err(file_error)?;
                    Ok(CommandValue::Number(content.len() as f32))
                }
                _ => Err("unknown text file command".to_owned()),
            }
        });
        cx.spawn(async move |_, cx| {
            let result = task.await;
            let (success, error, value) = match result {
                Ok(value) => (true, None, Some(value)),
                Err(error) => (false, Some(error), None),
            };
            let _ = entity.update(cx, |root, _| {
                root.emit_command_ack(request_id, kind, node_id, success, error, value);
            });
        })
        .detach();
    }

    pub(super) fn process_commands(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut refresh = false;
        for command in std::mem::take(&mut self.commands) {
            let mut success = true;
            let mut error = None;
            let mut value = None;
            if command.surface_id != self.store.surface_id() || command.epoch != self.store.epoch()
            {
                success = false;
                error = Some("surface or epoch mismatch".to_string());
            } else if command.after_revision != self.store.revision() {
                success = false;
                error = Some("command revision is stale".to_string());
            } else if command.kind == COMMAND_SET_MENUS {
                if command.node_id != 1
                    || command.payload.is_some()
                    || command.title.is_some()
                    || command.body.is_some()
                {
                    success = false;
                    error = Some("setMenus payload is invalid".to_owned());
                } else if let Some(menus) = command.menus.clone() {
                    self.set_menus(menus, cx);
                } else {
                    success = false;
                    error = Some("setMenus payload is required".to_owned());
                }
            } else if command.kind == COMMAND_SHOW_NOTIFICATION {
                if command.node_id != 1 {
                    success = false;
                    error = Some("showNotification requires the root container".to_owned());
                } else if command.payload.is_some()
                    || command.title.is_none()
                    || command.body.is_none()
                {
                    success = false;
                    error = Some("showNotification payload is invalid".to_owned());
                } else if command
                    .actions
                    .as_ref()
                    .is_some_and(|actions| actions.len() > 3)
                {
                    success = false;
                    error = Some("showNotification supports at most three actions".to_owned());
                } else {
                    let title = command.title.as_deref().unwrap_or_default();
                    let body = command.body.as_deref().unwrap_or_default();
                    let actions = command
                        .actions
                        .unwrap_or_default()
                        .into_iter()
                        .map(|action| SystemNotificationAction {
                            id: action.id.into(),
                            label: action.label.into(),
                        })
                        .collect();
                    cx.show_system_notification(SystemNotification {
                        tag: format!(
                            "react-gpui:{}:{}",
                            self.store.surface_id(),
                            command.request_id
                        )
                        .into(),
                        title: title.into(),
                        body: body.into(),
                        actions,
                    });
                }
            } else if matches!(
                command.kind,
                COMMAND_FILE_DIALOG_OPEN | COMMAND_FILE_DIALOG_SAVE
            ) {
                if command.node_id != 1 {
                    success = false;
                    error = Some("file dialog command requires the root container".to_owned());
                } else if command.kind == COMMAND_FILE_DIALOG_OPEN {
                    if command.title.is_some()
                        && command.payload.is_some_and(|(directories, multiple)| {
                            directories <= 1 && multiple <= 1
                        })
                    {
                        self.spawn_open_file_dialog(command, cx);
                        continue;
                    }
                    success = false;
                    error = Some("file dialog open payload is invalid".to_owned());
                } else if command.title.is_some() && command.payload.is_none() {
                    self.spawn_save_file_dialog(command, cx);
                    continue;
                } else {
                    success = false;
                    error = Some("file dialog save payload is invalid".to_owned());
                }
            } else if matches!(
                command.kind,
                COMMAND_LOAD_FONT | COMMAND_READ_TEXT_FILE | COMMAND_WRITE_TEXT_FILE
            ) {
                if command.node_id != 1 {
                    success = false;
                    error =
                        Some("font and text file commands require the root container".to_owned());
                } else if command.kind == COMMAND_LOAD_FONT {
                    self.spawn_load_font(command, cx);
                    continue;
                } else {
                    self.spawn_text_file_command(command, cx);
                    continue;
                }
            } else if command.kind == COMMAND_SET_TITLE {
                if command.node_id != 1 {
                    success = false;
                    error = Some("setTitle requires the root container".to_string());
                } else if let Some(title) = command.title.as_deref() {
                    window.set_window_title(title);
                } else {
                    success = false;
                    error = Some("setTitle payload is required".to_string());
                }
            } else if matches!(
                command.kind,
                COMMAND_RESIZE_WINDOW
                    | COMMAND_ZOOM_WINDOW
                    | COMMAND_TOGGLE_FULLSCREEN
                    | COMMAND_OPEN_URL
                    | COMMAND_FOCUS_NEXT
                    | COMMAND_FOCUS_PREV
                    | COMMAND_GET_WINDOW_SIZE
                    | COMMAND_MINIMIZE_WINDOW
                    | COMMAND_GET_WINDOW_BOUNDS
                    | COMMAND_GET_WINDOW_STATE
                    | COMMAND_ACTIVATE_WINDOW
                    | COMMAND_CLIPBOARD_WRITE
                    | COMMAND_CLIPBOARD_READ
                    | COMMAND_CLIPBOARD_WRITE_IMAGE
                    | COMMAND_CLIPBOARD_READ_IMAGE
            ) {
                if command.node_id != 1 {
                    success = false;
                    error = Some("surface command requires the root container".to_string());
                } else {
                    match command.kind {
                        COMMAND_CLIPBOARD_WRITE => {
                            if command.payload.is_some() {
                                success = false;
                                error = Some(
                                    "clipboard write does not accept a pair payload".to_string(),
                                );
                            } else if let Some(text) = command.title.as_deref() {
                                if text.len() > MAX_CLIPBOARD_TEXT_BYTES {
                                    success = false;
                                    error = Some(
                                        "clipboard text exceeds the supported size".to_string(),
                                    );
                                } else {
                                    cx.write_to_clipboard(ClipboardItem::new_string(
                                        text.to_owned(),
                                    ));
                                }
                            } else {
                                success = false;
                                error = Some("clipboard text payload is required".to_string());
                            }
                        }
                        COMMAND_CLIPBOARD_READ => {
                            if command.payload.is_some() || command.title.is_some() {
                                success = false;
                                error =
                                    Some("clipboard read does not accept a payload".to_string());
                            } else if let Some(item) = cx.read_from_clipboard() {
                                let has_string = item
                                    .entries()
                                    .iter()
                                    .any(|entry| matches!(entry, ClipboardEntry::String(_)));
                                if let Some(text) =
                                    item.text().or_else(|| has_string.then(String::new))
                                {
                                    if text.len() > MAX_CLIPBOARD_TEXT_BYTES {
                                        success = false;
                                        error = Some(
                                            "clipboard text exceeds the supported size".to_string(),
                                        );
                                    } else {
                                        value = Some(CommandValue::Text(text));
                                    }
                                } else {
                                    success = false;
                                    error = Some(
                                        "clipboard read failed: clipboard has no text content; copy text to the clipboard before calling getClipboardText"
                                            .to_string(),
                                    );
                                }
                            } else {
                                success = false;
                                error = Some(
                                    "clipboard read failed: clipboard has no text content; copy text to the clipboard before calling getClipboardText"
                                        .to_string(),
                                );
                            }
                        }
                        COMMAND_CLIPBOARD_WRITE_IMAGE => {
                            if !cfg!(any(target_os = "macos", target_os = "windows")) {
                                success = false;
                                error = Some(
                                    "clipboard image command is unsupported on this platform; use text clipboard commands or run on macOS/Windows"
                                        .to_owned(),
                                );
                            } else if command.payload.is_some() || command.title.is_some() {
                                success = false;
                                error = Some("clipboard image payload is invalid".to_owned());
                            } else if let Some(image) = command.image.as_ref() {
                                if image.bytes.is_empty()
                                    || image.bytes.len() > MAX_CLIPBOARD_IMAGE_BYTES
                                {
                                    success = false;
                                    error = Some("clipboard image bytes are invalid".to_owned());
                                } else if let Some(format) = gpui_image_format(image.format) {
                                    let native_image =
                                        Image::from_bytes(format, image.bytes.clone());
                                    cx.write_to_clipboard(ClipboardItem::new_image(&native_image));
                                } else {
                                    success = false;
                                    error =
                                        Some("clipboard image format is unsupported".to_owned());
                                }
                            } else {
                                success = false;
                                error = Some("clipboard image payload is required".to_owned());
                            }
                        }
                        COMMAND_CLIPBOARD_READ_IMAGE => {
                            if command.payload.is_some() || command.title.is_some() {
                                success = false;
                                error = Some(
                                    "clipboard image read does not accept a payload".to_owned(),
                                );
                            } else if !cfg!(any(target_os = "macos", target_os = "windows")) {
                                success = false;
                                error = Some(
                                    "clipboard image command is unsupported on this platform; use text clipboard commands or run on macOS/Windows"
                                        .to_owned(),
                                );
                            } else if let Some(item) = cx.read_from_clipboard()
                                && let Some(image) = item.entries().iter().find_map(|entry| {
                                    if let ClipboardEntry::Image(image) = entry {
                                        Some(image)
                                    } else {
                                        None
                                    }
                                })
                            {
                                if let Some(image) = clipboard_image_value(image) {
                                    value = Some(image);
                                } else {
                                    success = false;
                                    error = Some(
                                        "clipboard image is unsupported or too large".to_owned(),
                                    );
                                }
                            }
                        }
                        COMMAND_GET_WINDOW_SIZE => {
                            if command.payload.is_some() || command.title.is_some() {
                                success = false;
                                error = Some("getWindowSize does not accept a payload".to_string());
                            } else {
                                let bounds = window.bounds();
                                value = Some(CommandValue::Pair((
                                    f32::from(bounds.size.width),
                                    f32::from(bounds.size.height),
                                )));
                            }
                        }
                        COMMAND_GET_WINDOW_BOUNDS => {
                            if command.payload.is_some() || command.title.is_some() {
                                success = false;
                                error =
                                    Some("getWindowBounds does not accept a payload".to_string());
                            } else {
                                let bounds = window.bounds();
                                value = Some(CommandValue::Bounds((
                                    f32::from(bounds.origin.x),
                                    f32::from(bounds.origin.y),
                                    f32::from(bounds.size.width),
                                    f32::from(bounds.size.height),
                                )));
                            }
                        }
                        COMMAND_GET_WINDOW_STATE => {
                            if command.payload.is_some() || command.title.is_some() {
                                success = false;
                                error =
                                    Some("getWindowState does not accept a payload".to_string());
                            } else {
                                value = Some(CommandValue::WindowState((
                                    window.is_fullscreen(),
                                    window.is_maximized(),
                                )));
                            }
                        }
                        COMMAND_MINIMIZE_WINDOW => {
                            if command.payload.is_some() || command.title.is_some() {
                                success = false;
                                error =
                                    Some("minimizeWindow does not accept a payload".to_string());
                            } else {
                                window.minimize_window();
                            }
                        }
                        COMMAND_ACTIVATE_WINDOW => {
                            if command.payload.is_some() || command.title.is_some() {
                                success = false;
                                error =
                                    Some("activateWindow does not accept a payload".to_string());
                            } else {
                                window.activate_window();
                            }
                        }
                        COMMAND_RESIZE_WINDOW => {
                            if let Some((width, height)) = command.payload {
                                if width == 0
                                    || width > MAX_WINDOW_DIMENSION
                                    || height == 0
                                    || height > MAX_WINDOW_DIMENSION
                                {
                                    success = false;
                                    error = Some(
                                        "window size is outside the supported range".to_string(),
                                    );
                                } else {
                                    window.resize(size(px(width as f32), px(height as f32)));
                                }
                            } else {
                                success = false;
                                error = Some("resize payload is required".to_string());
                            }
                        }
                        COMMAND_ZOOM_WINDOW => {
                            if command.payload.is_some() || command.title.is_some() {
                                success = false;
                                error = Some("zoom does not accept a payload".to_string());
                            } else {
                                window.zoom_window();
                            }
                        }
                        COMMAND_TOGGLE_FULLSCREEN => {
                            if command.payload.is_some() || command.title.is_some() {
                                success = false;
                                error =
                                    Some("toggleFullscreen does not accept a payload".to_string());
                            } else {
                                window.toggle_fullscreen();
                            }
                        }
                        COMMAND_OPEN_URL => {
                            if let Some(url) = command.title.as_deref() {
                                cx.open_url(url);
                            } else {
                                success = false;
                                error = Some("openUrl payload is required".to_string());
                            }
                        }
                        COMMAND_FOCUS_NEXT => {
                            if command.payload.is_some() || command.title.is_some() {
                                success = false;
                                error = Some("focusNext does not accept a payload".to_string());
                            } else {
                                window.focus_next(cx);
                            }
                        }
                        COMMAND_FOCUS_PREV => {
                            if command.payload.is_some() || command.title.is_some() {
                                success = false;
                                error = Some("focusPrev does not accept a payload".to_string());
                            } else {
                                window.focus_prev(cx);
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            } else if command.kind == COMMAND_GET_FOCUS {
                if command.payload.is_some() || command.title.is_some() {
                    success = false;
                    error = Some("getFocus does not accept a payload".to_string());
                } else if let Some(handle) = self.focus_handles.get(&command.node_id) {
                    value = Some(CommandValue::Bool(handle.is_focused(window)));
                } else {
                    success = false;
                    error = Some("node is not focusable".to_string());
                }
            } else {
                match self.store.get(command.node_id) {
                    Some(node) if node.kind == KIND_VIEW => {
                        if !node.focusable {
                            success = false;
                            error = Some("View is not focusable".to_string());
                        } else {
                            match command.kind {
                                COMMAND_FOCUS => {
                                    if let Some(handle) = self.focus_handles.get(&command.node_id) {
                                        window.focus(handle, cx);
                                    } else {
                                        success = false;
                                        error = Some("View is not mounted".to_string());
                                    }
                                }
                                COMMAND_BLUR => {
                                    window.blur();
                                }
                                _ => {
                                    success = false;
                                    error = Some("unknown View command".to_string());
                                }
                            }
                        }
                    }
                    Some(node) if node.kind == KIND_PRESSABLE => {
                        if !node.focusable {
                            success = false;
                            error = Some("Pressable is not focusable".to_string());
                        } else {
                            match command.kind {
                                COMMAND_FOCUS => {
                                    if let Some(handle) = self.focus_handles.get(&command.node_id) {
                                        window.focus(handle, cx);
                                    } else {
                                        success = false;
                                        error = Some("Pressable is not mounted".to_string());
                                    }
                                }
                                COMMAND_BLUR => {
                                    window.blur();
                                }
                                _ => {
                                    success = false;
                                    error = Some("unknown Pressable command".to_string());
                                }
                            }
                        }
                    }
                    Some(node) if node.kind == KIND_TEXT_INPUT => {
                        let disabled = matches!(node.host_properties.as_ref(), Some(HostProperties::TextInput(input)) if input.disabled);
                        if disabled {
                            success = false;
                            error = Some("TextInput is disabled".to_string());
                        } else {
                            match command.kind {
                                COMMAND_FOCUS => {
                                    if let Some(handle) = self.focus_handles.get(&command.node_id) {
                                        window.focus(handle, cx);
                                        self.set_input_focus(command.node_id, true);
                                    } else {
                                        success = false;
                                        error = Some("TextInput is not mounted".to_string());
                                    }
                                }
                                COMMAND_BLUR => {
                                    window.blur();
                                    self.set_input_focus(command.node_id, false);
                                }
                                COMMAND_SET_SELECTION => {
                                    if let Some((start, end)) = command.payload {
                                        let length = self
                                            .input_states
                                            .get(&command.node_id)
                                            .map(|state| state.text.encode_utf16().count())
                                            .unwrap_or(0);
                                        if start > end || end as usize > length {
                                            success = false;
                                            error = Some(
                                                "selection is outside UTF-16 text range"
                                                    .to_string(),
                                            );
                                        } else if let Some(state) =
                                            self.input_states.get_mut(&command.node_id)
                                        {
                                            state.set_selection(start as usize..end as usize);
                                            self.emit_input_event(command.node_id, EVENT_SELECTION);
                                        }
                                    } else {
                                        success = false;
                                        error = Some("selection payload is required".to_string());
                                    }
                                }
                                _ => {
                                    success = false;
                                    error = Some("unknown TextInput command".to_string());
                                }
                            }
                        }
                    }
                    Some(node) if node.kind == KIND_VIRTUAL_LIST => {
                        let list =
                            node.host_properties
                                .as_ref()
                                .and_then(|properties| match properties {
                                    HostProperties::VirtualList(list) => Some(list),
                                    _ => None,
                                });
                        let handle = self.virtual_lists.get(&command.node_id);
                        match (list, handle) {
                            (Some(list), Some(state)) => match command.kind {
                                COMMAND_SCROLL_TO_INDEX => {
                                    if let Some((index, _)) = command.payload {
                                        if index >= list.item_count {
                                            success = false;
                                            error = Some(format!(
                                                "VirtualList index is out of range for node {}; use an index from 0 through {} - 1",
                                                command.node_id, list.item_count
                                            ));
                                        } else {
                                            state.scroll_to(ListOffset {
                                                item_ix: index as usize,
                                                offset_in_item: px(0.0),
                                            });
                                            refresh = true;
                                        }
                                    } else {
                                        success = false;
                                        error =
                                            Some("scroll index payload is required".to_string());
                                    }
                                }
                                COMMAND_SCROLL_TO_END => {
                                    state.scroll_to_end();
                                    refresh = true;
                                }
                                COMMAND_GET_SCROLL_OFFSET => {
                                    if command.payload.is_some() || command.scroll_offset.is_some()
                                    {
                                        success = false;
                                        error = Some(
                                            "getScrollOffset does not accept a payload".to_string(),
                                        );
                                    } else {
                                        let offset =
                                            -state.scroll_px_offset_for_scrollbar().y.as_f32();
                                        if offset.is_finite() && offset >= 0.0 {
                                            value = Some(CommandValue::ScrollOffset(offset));
                                        } else {
                                            success = false;
                                            error = Some(
                                                "VirtualList scroll offset is invalid".to_string(),
                                            );
                                        }
                                    }
                                }
                                COMMAND_SCROLL_TO_OFFSET => {
                                    if let Some(offset) = command.scroll_offset {
                                        if offset.is_finite() && offset >= 0.0 {
                                            state.set_offset_from_scrollbar(Point::new(
                                                px(0.0),
                                                px(-offset),
                                            ));
                                            refresh = true;
                                        } else {
                                            success = false;
                                            error = Some(
                                                "scroll offset must be finite and non-negative".to_string(),
                                            );
                                        }
                                    } else {
                                        success = false;
                                        error =
                                            Some("scroll offset payload is required".to_string());
                                    }
                                }
                                _ => {
                                    success = false;
                                    error = Some("unknown VirtualList command".to_string());
                                }
                            },
                            (None, _) => {
                                success = false;
                                error = Some("VirtualList properties are missing".to_string());
                            }
                            (_, None) => {
                                success = false;
                                error = Some("VirtualList is not mounted".to_string());
                            }
                        }
                    }
                    Some(_) => {
                        success = false;
                        error = Some("node does not support commands".to_string());
                    }
                    None => {
                        success = false;
                        error = Some("host node is missing".to_string());
                    }
                }
            }
            let result = Event::command_result(
                self.store.surface_id(),
                self.store.epoch(),
                self.store.revision(),
                self.next_sequence.fetch_add(1, Ordering::Relaxed),
                CommandResult {
                    request_id: command.request_id,
                    command: command.kind,
                    node_id: command.node_id,
                    success,
                    error,
                    value,
                },
            );
            if !send_event_or_exit(self.runtime.as_ref(), "CommandResult event", &result) {
                break;
            }
        }
        if refresh {
            window.refresh();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_action_states_map_to_gpui_menu_item_flags() {
        let item = ReactRoot::menu_item(MenuItemDefinition::Action {
            name: "open".to_owned(),
            disabled: true,
            checked: true,
        });
        assert!(item.is_disabled());
        assert!(item.is_checked());

        let item = ReactRoot::menu_item(MenuItemDefinition::Action {
            name: "other".to_owned(),
            disabled: false,
            checked: false,
        });
        assert!(!item.is_disabled());
        assert!(!item.is_checked());
    }
}
