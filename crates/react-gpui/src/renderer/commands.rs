use std::path::Path;
use std::sync::atomic::Ordering;

use gpui::{
    ClipboardEntry, ClipboardItem, Context, ListOffset, Menu as GpuiMenu, MenuItem as GpuiMenuItem,
    PathPromptOptions, SystemNotification, SystemNotificationAction, Window, px, size,
};

use super::ReactRoot;
use crate::protocol::{
    COMMAND_BLUR, COMMAND_CLIPBOARD_READ, COMMAND_CLIPBOARD_WRITE, COMMAND_FILE_DIALOG_OPEN,
    COMMAND_FILE_DIALOG_SAVE, COMMAND_FOCUS, COMMAND_FOCUS_NEXT, COMMAND_FOCUS_PREV,
    COMMAND_GET_FOCUS, COMMAND_GET_WINDOW_SIZE, COMMAND_OPEN_URL, COMMAND_RESIZE_WINDOW,
    COMMAND_SCROLL_TO_END, COMMAND_SCROLL_TO_INDEX, COMMAND_SET_MENUS, COMMAND_SET_SELECTION,
    COMMAND_SET_TITLE, COMMAND_SHOW_NOTIFICATION, COMMAND_TOGGLE_FULLSCREEN, COMMAND_ZOOM_WINDOW,
    Command, CommandResult, CommandValue, EVENT_SELECTION, Event, HostProperties,
    MAX_CLIPBOARD_TEXT_BYTES, MAX_WINDOW_DIMENSION, MenuAction, MenuDefinition, MenuItemDefinition,
};
use crate::transport::send_event_or_exit;
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
                    | COMMAND_CLIPBOARD_WRITE
                    | COMMAND_CLIPBOARD_READ
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
                                    error = Some("clipboard has no text content".to_string());
                                }
                            } else {
                                success = false;
                                error = Some("clipboard has no text content".to_string());
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
                                            error = Some(
                                                "VirtualList index is out of range".to_string(),
                                            );
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
