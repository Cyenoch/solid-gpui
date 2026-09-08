#[cfg(feature = "gpui-component")]
use gpui::BorrowAppContext as _;
use std::borrow::Cow;
use std::path::Path;

use gpui::{
    AppContext, ClipboardEntry, ClipboardItem, Context, Image, ImageFormat, ListOffset,
    Menu as GpuiMenu, MenuItem as GpuiMenuItem, PathPromptOptions, Point, SystemNotification,
    SystemNotificationAction, Window, px, size,
};
use skrifa::MetadataProvider;

use super::SolidRoot;
use crate::protocol::{
    ClipboardImage, CommandKind, CommandMeta, CommandOperation, CommandValue, EVENT_SELECTION,
    HostProperties, MAX_CLIPBOARD_IMAGE_BYTES, MAX_FILE_READ_BYTES, MAX_FILE_WRITE_BYTES,
    MenuAction, MenuDefinition, MenuItemDefinition,
};

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
    let face = skrifa::FontRef::from_index(bytes, 0)
        .map_err(|_| "font is malformed or unsupported".to_owned())?;
    for name in face.localized_strings(skrifa::string::StringId::TYPOGRAPHIC_FAMILY_NAME) {
        let value = name.to_string();
        if !value.is_empty() && value.chars().count() <= 64 && !value.chars().any(char::is_control)
        {
            return Ok(value);
        }
    }
    for name in face.localized_strings(skrifa::string::StringId::FAMILY_NAME) {
        let value = name.to_string();
        if !value.is_empty() && value.chars().count() <= 64 && !value.chars().any(char::is_control)
        {
            return Ok(value);
        }
    }
    Err("font has no usable family name".to_owned())
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

use crate::tree::KIND_TEXT_INPUT;

impl SolidRoot {
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
        #[cfg(feature = "gpui-component")]
        if cx.has_global::<gpui_component::GlobalState>() {
            let owned = menus
                .iter()
                .cloned()
                .map(|menu| {
                    GpuiMenu::new(menu.title)
                        .items(menu.items.into_iter().map(Self::menu_item))
                        .owned()
                })
                .collect();
            cx.update_global::<gpui_component::GlobalState, _>(|state, _| {
                state.set_app_menus(owned)
            });
        }
        cx.set_menus(menus.into_iter().map(|menu| {
            GpuiMenu::new(menu.title).items(menu.items.into_iter().map(Self::menu_item))
        }));
    }

    fn spawn_open_file_dialog(
        &self,
        meta: CommandMeta,
        title: String,
        directories: bool,
        multiple: bool,
        cx: &mut Context<Self>,
    ) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: !directories,
            directories,
            multiple,
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
                root.emit_command_ack(meta, CommandKind::FileDialogOpen, success, error, value);
            });
        })
        .detach();
    }

    fn spawn_save_file_dialog(
        &self,
        meta: CommandMeta,
        suggested_name: String,
        cx: &mut Context<Self>,
    ) {
        let suggested_name = (!suggested_name.is_empty()).then_some(suggested_name);
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
                root.emit_command_ack(meta, CommandKind::FileDialogSave, success, error, value);
            });
        })
        .detach();
    }

    fn spawn_load_font(&self, meta: CommandMeta, path: String, cx: &mut Context<Self>) {
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
        let entity = cx.weak_entity();
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
                root.emit_command_ack(meta, CommandKind::LoadFont, success, error, value);
            });
        })
        .detach();
    }

    fn spawn_text_file_command(
        &self,
        meta: CommandMeta,
        operation: CommandOperation,
        cx: &mut Context<Self>,
    ) {
        let (kind, task) = match operation {
            CommandOperation::ReadTextFile { path } => (
                CommandKind::ReadTextFile,
                cx.background_spawn(async move {
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
                }),
            ),
            CommandOperation::WriteTextFile { path, content } => (
                CommandKind::WriteTextFile,
                cx.background_spawn(async move {
                    if content.len() > MAX_FILE_WRITE_BYTES {
                        return Err("file content is too large".to_owned());
                    }
                    std::fs::write(&path, content.as_bytes()).map_err(file_error)?;
                    Ok(CommandValue::Number(content.len() as u32))
                }),
            ),
            _ => return,
        };
        let entity = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            let (success, error, value) = match task.await {
                Ok(value) => (true, None, Some(value)),
                Err(error) => (false, Some(error), None),
            };
            let _ = entity.update(cx, |root, _| {
                root.emit_command_ack(meta, kind, success, error, value);
            });
        })
        .detach();
    }

    fn process_surface_command(
        &mut self,
        meta: CommandMeta,
        operation: CommandOperation,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> (bool, Option<String>, Option<CommandValue>, bool) {
        match operation {
            CommandOperation::CancelNative { request_id } => {
                self.pending_native_calls.remove(&request_id);
                (true, None, None, false)
            }
            CommandOperation::InvokeNative {
                module_id,
                module_digest,
                function_id,
                args,
            } => {
                if args.len() > crate::native::MAX_NATIVE_CALL_BYTES {
                    return (
                        false,
                        Some("native invocation arguments exceed the byte limit".into()),
                        None,
                        false,
                    );
                }
                if let Some(module) = self
                    .extension_registry
                    .native_module(module_id, module_digest)
                {
                    if module.module_id() != module_id || module.module_digest() != module_digest {
                        return (
                            false,
                            Some("native module registry returned a different contract".into()),
                            None,
                            false,
                        );
                    }
                    if self.pending_native_calls.contains_key(&meta.request_id) {
                        return (
                            false,
                            Some("native request ID is already in flight".into()),
                            None,
                            false,
                        );
                    }
                    if let Some(result) = module.invoke_foreground(function_id, &args, window, cx) {
                        return match result {
                            Ok(bytes) if bytes.len() <= crate::native::MAX_NATIVE_CALL_BYTES => {
                                (true, None, Some(CommandValue::Bytes(bytes)), false)
                            }
                            Ok(_) => (
                                false,
                                Some("native reply exceeds the byte limit".into()),
                                None,
                                false,
                            ),
                            Err(error) if error.len() <= crate::native::MAX_NATIVE_CALL_BYTES => {
                                (false, Some(error), None, false)
                            }
                            Err(_) => (
                                false,
                                Some("native error exceeds the byte limit".into()),
                                None,
                                false,
                            ),
                        };
                    }
                }
                match self.start_native_call(meta, module_id, module_digest, function_id, args, cx)
                {
                    Ok(()) => (true, None, None, true),
                    Err(error) => (false, Some(error), None, false),
                }
            }
            CommandOperation::SetMenus { menus } => {
                self.set_menus(menus, cx);
                (true, None, None, false)
            }
            CommandOperation::ShowNotification {
                title,
                body,
                actions,
            } => {
                let actions = actions
                    .unwrap_or_default()
                    .into_iter()
                    .map(|action| SystemNotificationAction {
                        id: action.id.into(),
                        label: action.label.into(),
                    })
                    .collect();
                cx.show_system_notification(SystemNotification {
                    tag: format!("solid-gpui:{}:{}", self.store.surface_id(), meta.request_id)
                        .into(),
                    title: title.into(),
                    body: body.into(),
                    actions,
                });
                (true, None, None, false)
            }
            CommandOperation::FileDialogOpen {
                title,
                directories,
                multiple,
            } => {
                self.spawn_open_file_dialog(meta, title, directories, multiple, cx);
                (true, None, None, true)
            }
            CommandOperation::FileDialogSave { default_name } => {
                self.spawn_save_file_dialog(meta, default_name, cx);
                (true, None, None, true)
            }
            CommandOperation::LoadFont { path } => {
                self.spawn_load_font(meta, path, cx);
                (true, None, None, true)
            }
            CommandOperation::ReadTextFile { .. } | CommandOperation::WriteTextFile { .. } => {
                self.spawn_text_file_command(meta, operation, cx);
                (true, None, None, true)
            }
            CommandOperation::SetTitle { title } => {
                window.set_window_title(&title);
                (true, None, None, false)
            }
            CommandOperation::ResizeWindow { width, height } => {
                window.resize(size(px(width as f32), px(height as f32)));
                (true, None, None, false)
            }
            CommandOperation::ZoomWindow => {
                window.zoom_window();
                (true, None, None, false)
            }
            CommandOperation::ToggleFullscreen => {
                window.toggle_fullscreen();
                (true, None, None, false)
            }
            CommandOperation::OpenUrl { url } => {
                cx.open_url(&url);
                (true, None, None, false)
            }
            CommandOperation::FocusNext => {
                window.focus_next(cx);
                (true, None, None, false)
            }
            CommandOperation::FocusPrev => {
                window.focus_prev(cx);
                (true, None, None, false)
            }
            CommandOperation::ClipboardWrite { text } => {
                cx.write_to_clipboard(ClipboardItem::new_string(text));
                (true, None, None, false)
            }
            CommandOperation::ClipboardRead => {
                let value = cx
                    .read_from_clipboard()
                    .and_then(|item| item.text())
                    .map(CommandValue::Text);
                (
                    value.is_some(),
                    value.is_none().then(|| "clipboard has no text".to_owned()),
                    value,
                    false,
                )
            }
            CommandOperation::ClipboardWriteImage { image } => {
                if !cfg!(any(target_os = "macos", target_os = "windows")) {
                    return (
                        false,
                        Some("clipboard image command is unsupported on this platform".to_owned()),
                        None,
                        false,
                    );
                }
                let Some(format) = gpui_image_format(image.format) else {
                    return (
                        false,
                        Some("clipboard image format is unsupported".to_owned()),
                        None,
                        false,
                    );
                };
                let native_image = Image::from_bytes(format, image.bytes);
                cx.write_to_clipboard(ClipboardItem::new_image(&native_image));
                (true, None, None, false)
            }
            CommandOperation::ClipboardReadImage => {
                if !cfg!(any(target_os = "macos", target_os = "windows")) {
                    return (
                        false,
                        Some("clipboard image command is unsupported on this platform".to_owned()),
                        None,
                        false,
                    );
                }
                let value = cx.read_from_clipboard().and_then(|item| {
                    item.entries().iter().find_map(|entry| match entry {
                        ClipboardEntry::Image(image) => clipboard_image_value(image),
                        _ => None,
                    })
                });
                (true, None, value, false)
            }
            CommandOperation::GetWindowSize => {
                let bounds = window.bounds();
                (
                    true,
                    None,
                    Some(CommandValue::Pair((
                        f32::from(bounds.size.width),
                        f32::from(bounds.size.height),
                    ))),
                    false,
                )
            }
            CommandOperation::GetWindowBounds => {
                let bounds = window.bounds();
                (
                    true,
                    None,
                    Some(CommandValue::Bounds((
                        f32::from(bounds.origin.x),
                        f32::from(bounds.origin.y),
                        f32::from(bounds.size.width),
                        f32::from(bounds.size.height),
                    ))),
                    false,
                )
            }
            CommandOperation::GetWindowState => (
                true,
                None,
                Some(CommandValue::WindowState((
                    window.is_fullscreen(),
                    window.is_maximized(),
                ))),
                false,
            ),
            CommandOperation::MinimizeWindow => {
                window.minimize_window();
                (true, None, None, false)
            }
            CommandOperation::ActivateWindow => {
                window.activate_window();
                (true, None, None, false)
            }
            _ => (
                false,
                Some("command requires a host node".to_owned()),
                None,
                false,
            ),
        }
    }

    fn process_node_command(
        &mut self,
        meta: CommandMeta,
        operation: CommandOperation,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> (bool, Option<String>, Option<CommandValue>) {
        let node_id = meta.node_id;
        match operation {
            CommandOperation::InvokeNative {
                module_id,
                module_digest,
                function_id,
                args,
            } => {
                let limit = crate::native::MAX_NATIVE_CALL_BYTES;
                if args.len() > limit {
                    return (
                        false,
                        Some("native invocation arguments exceed the byte limit".to_owned()),
                        None,
                    );
                }
                if self.extension_dirty.contains(&node_id) {
                    return (
                        false,
                        Some("native component requires the window commit entry".to_owned()),
                        None,
                    );
                }
                let Some(mounted) = self.extension_instances.get_mut(&node_id) else {
                    return (
                        false,
                        Some("native component is not mounted".to_owned()),
                        None,
                    );
                };
                if mounted.properties.provider_id != module_id
                    || mounted.properties.catalog_digest != module_digest
                {
                    return (
                        false,
                        Some("native component contract mismatch".to_owned()),
                        None,
                    );
                }
                let Some(instance) = mounted.instance.as_mut() else {
                    return (
                        false,
                        Some("native component has no callable instance".to_owned()),
                        None,
                    );
                };
                match instance.invoke(function_id, &args, window, cx) {
                    Ok(bytes) if bytes.len() <= limit => {
                        (true, None, Some(CommandValue::Bytes(bytes)))
                    }
                    Ok(_) => (
                        false,
                        Some("native invocation result exceeds the byte limit".to_owned()),
                        None,
                    ),
                    Err(error) if error.len() <= limit => (false, Some(error), None),
                    Err(_) => (
                        false,
                        Some("native invocation error exceeds the byte limit".to_owned()),
                        None,
                    ),
                }
            }
            CommandOperation::GetFocus => self
                .focus_handles
                .get(&node_id)
                .map(|handle| {
                    (
                        true,
                        None,
                        Some(CommandValue::Bool(handle.is_focused(window))),
                    )
                })
                .unwrap_or((false, Some("node is not focusable".to_owned()), None)),
            CommandOperation::Focus | CommandOperation::Blur => {
                let Some(node) = self.store.get(node_id) else {
                    return (false, Some("host node is missing".to_owned()), None);
                };
                if !node.focusable && node.kind != KIND_TEXT_INPUT {
                    return (false, Some("node is not focusable".to_owned()), None);
                }
                let Some(handle) = self.focus_handles.get(&node_id) else {
                    return (false, Some("node is not mounted".to_owned()), None);
                };
                if matches!(operation, CommandOperation::Focus) {
                    window.focus(handle, cx);
                } else {
                    window.blur(cx);
                }
                if node.kind == KIND_TEXT_INPUT {
                    self.set_input_focus(node_id, matches!(operation, CommandOperation::Focus));
                }
                (true, None, None)
            }
            CommandOperation::SetSelection { start, end } => {
                let length = self
                    .input_states
                    .get(&node_id)
                    .map(|state| state.text.encode_utf16().count())
                    .unwrap_or(0);
                if end as usize > length || start > end {
                    return (
                        false,
                        Some("selection is outside UTF-16 text range".to_owned()),
                        None,
                    );
                }
                if let Some(state) = self.input_states.get_mut(&node_id) {
                    state.set_selection(start as usize..end as usize);
                    self.emit_input_event(node_id, EVENT_SELECTION);
                    (true, None, None)
                } else {
                    (false, Some("TextInput is not mounted".to_owned()), None)
                }
            }
            CommandOperation::ScrollToIndex { .. }
            | CommandOperation::ScrollToEnd
            | CommandOperation::GetScrollOffset
            | CommandOperation::ScrollToOffset { .. } => {
                let requested_index = match &operation {
                    CommandOperation::ScrollToIndex { index, .. } => Some(*index),
                    _ => None,
                };
                let list = self.store.get(node_id).and_then(|node| {
                    node.host_properties
                        .as_ref()
                        .and_then(|properties| match properties {
                            HostProperties::VirtualList(list) => Some(list),
                            _ => None,
                        })
                });
                let Some(list) = list else {
                    return (
                        false,
                        Some("VirtualList properties are missing".to_owned()),
                        None,
                    );
                };
                let Some(state) = self.virtual_lists.get(&node_id) else {
                    return (false, Some("VirtualList is not mounted".to_owned()), None);
                };
                if let Some(index) = requested_index {
                    if index >= list.item_count {
                        return (
                            false,
                            Some(format!(
                                "VirtualList index is out of range for node {node_id}; use an index from 0 through {} - 1",
                                list.item_count
                            )),
                            None,
                        );
                    }
                    state.scroll_to(ListOffset {
                        item_ix: index as usize,
                        offset_in_item: px(0.0),
                    });
                    window.refresh();
                    return (true, None, None);
                }
                match operation {
                    CommandOperation::ScrollToEnd => {
                        state.scroll_to_end();
                        window.refresh();
                        (true, None, None)
                    }
                    CommandOperation::GetScrollOffset => {
                        let offset = -state.scroll_px_offset_for_scrollbar().y.as_f32();
                        if offset.is_finite() && offset >= 0.0 {
                            (true, None, Some(CommandValue::ScrollOffset(offset)))
                        } else {
                            (
                                false,
                                Some("VirtualList scroll offset is invalid".to_owned()),
                                None,
                            )
                        }
                    }
                    CommandOperation::ScrollToOffset { offset } => {
                        state.set_offset_from_scrollbar(Point::new(px(0.0), px(-offset)));
                        window.refresh();
                        (true, None, None)
                    }
                    _ => unreachable!(),
                }
            }
            CommandOperation::OpenPopup { .. } | CommandOperation::ClosePopup { .. } => (
                false,
                Some("SystemPopover requires a desktop host with native popup support".to_owned()),
                None,
            ),
            _ => (
                false,
                Some("command is not supported by this node".to_owned()),
                None,
            ),
        }
    }

    pub(super) fn process_commands(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for command in std::mem::take(&mut self.commands) {
            let meta = command.meta;
            let kind = command.operation.kind();
            if meta.surface_id != self.store.surface_id() || meta.epoch != self.store.epoch() {
                self.emit_command_ack(
                    meta,
                    kind,
                    false,
                    Some("surface or epoch mismatch".to_owned()),
                    None,
                );
                continue;
            }
            if meta.after_revision != self.store.revision() {
                self.emit_command_ack(
                    meta,
                    kind,
                    false,
                    Some("command revision is stale".to_owned()),
                    None,
                );
                continue;
            }
            let root_operation = (meta.node_id == 1
                && matches!(command.operation, CommandOperation::InvokeNative { .. }))
                || matches!(
                    command.operation,
                    CommandOperation::CancelNative { .. }
                        | CommandOperation::SetMenus { .. }
                        | CommandOperation::ShowNotification { .. }
                        | CommandOperation::FileDialogOpen { .. }
                        | CommandOperation::FileDialogSave { .. }
                        | CommandOperation::LoadFont { .. }
                        | CommandOperation::ReadTextFile { .. }
                        | CommandOperation::WriteTextFile { .. }
                        | CommandOperation::SetTitle { .. }
                        | CommandOperation::ResizeWindow { .. }
                        | CommandOperation::ZoomWindow
                        | CommandOperation::ToggleFullscreen
                        | CommandOperation::OpenUrl { .. }
                        | CommandOperation::FocusNext
                        | CommandOperation::FocusPrev
                        | CommandOperation::ClipboardWrite { .. }
                        | CommandOperation::ClipboardRead
                        | CommandOperation::ClipboardWriteImage { .. }
                        | CommandOperation::ClipboardReadImage
                        | CommandOperation::GetWindowSize
                        | CommandOperation::GetWindowBounds
                        | CommandOperation::GetWindowState
                        | CommandOperation::MinimizeWindow
                        | CommandOperation::ActivateWindow
                );
            if root_operation && meta.node_id != 1 {
                self.emit_command_ack(
                    meta,
                    kind,
                    false,
                    Some("command requires the root container".to_owned()),
                    None,
                );
                continue;
            }
            let (success, error, value) = if root_operation {
                let (success, error, value, deferred) =
                    self.process_surface_command(meta, command.operation, window, cx);
                if deferred {
                    continue;
                }
                (success, error, value)
            } else {
                self.process_node_command(meta, command.operation, window, cx)
            };
            self.emit_command_ack(meta, kind, success, error, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_action_states_map_to_gpui_menu_item_flags() {
        let item = SolidRoot::menu_item(MenuItemDefinition::Action {
            name: "open".to_owned(),
            disabled: true,
            checked: true,
        });
        assert!(item.is_disabled());
        assert!(item.is_checked());

        let item = SolidRoot::menu_item(MenuItemDefinition::Action {
            name: "other".to_owned(),
            disabled: false,
            checked: false,
        });
        assert!(!item.is_disabled());
        assert!(!item.is_checked());
    }
    #[test]
    fn font_family_extracts_tuffy_family() {
        let bytes = include_bytes!("../../tests/host-fixtures/tuffy.ttf");
        assert_eq!(font_family(bytes), Ok("Tuffy".to_owned()));
    }
    #[test]
    fn font_family_rejects_malformed_bytes_without_panicking() {
        assert_eq!(
            font_family(b"not a font"),
            Err("font is malformed or unsupported".to_owned())
        );
    }
}
