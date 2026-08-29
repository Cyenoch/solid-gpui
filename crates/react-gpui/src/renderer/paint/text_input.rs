use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, ClipboardItem, ContentMask, Element, ElementId, ElementInputHandler,
    Entity, GlobalElementId, InspectorElementId, InteractiveElement, IntoElement, LayoutId,
    MouseButton, PaintQuad, ParentElement, Pixels, Point, SharedString, StatefulInteractiveElement,
    Styled, TextRun, Window, div, fill, hsla, px, relative, rgba, size,
};

use crate::protocol::{HostProperties, KeyAction, Style};
use crate::tree::StoredNode;

use super::super::ReactRoot;
use super::super::events::emit_key_event;
use super::accessibility::apply_accessibility;
use super::style::{apply_style, apply_text_style};
pub(super) fn text_style_to_run(text_style: &gpui::TextStyle, len: usize) -> TextRun {
    text_style.to_run(len)
}

pub(super) fn text_style_to_run_with_color(
    text_style: &gpui::TextStyle,
    len: usize,
    color: gpui::Hsla,
) -> TextRun {
    let mut run = text_style_to_run(text_style, len);
    run.color = color;
    run
}

struct TextInputElement {
    entity: Entity<ReactRoot>,
    node_id: u32,
    focus: gpui::FocusHandle,
    fallback_text: String,
    placeholder: String,
    multiline: bool,
    bounded_height: bool,
}

struct SelectableTextPrepaint {
    text: super::super::input::TextInputTextLayout,
    selection: Vec<PaintQuad>,
    content: String,
}

struct TextInputPrepaint {
    text: super::super::input::TextInputTextLayout,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
    scroll_offset: Point<Pixels>,
    content: String,
    placeholder: bool,
}

impl IntoElement for TextInputElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextInputElement {
    type RequestLayoutState = ();
    type PrepaintState = TextInputPrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let line_height = window.line_height();
        let mut style = gpui::Style::default();
        style.size.width = relative(1.).into();
        if self.bounded_height {
            style.size.height = relative(1.).into();
        } else {
            let line_count = if self.multiline {
                let root = self.entity.read(cx);
                let content = root
                    .input_states
                    .get(&self.node_id)
                    .map(|state| state.text.as_str())
                    .unwrap_or(self.fallback_text.as_str());
                content.split('\n').count().max(1)
            } else {
                1
            };
            style.size.height = (line_height * line_count as f32).into();
        }
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let root = self.entity.read(cx);
        let (content, selection, selection_reversed, marked, current_offset) = root
            .input_states
            .get(&self.node_id)
            .map(|state| {
                (
                    state.text.clone(),
                    state.selection.clone(),
                    state.selection_reversed,
                    state.marked.clone(),
                    state.scroll_offset,
                )
            })
            .unwrap_or_else(|| {
                (
                    self.fallback_text.clone(),
                    0..0,
                    false,
                    None,
                    Point::default(),
                )
            });
        let (display_text, is_placeholder) =
            input_display_text(content.clone(), Some(&self.placeholder));
        let text_style = window.text_style();
        let text_color = if is_placeholder {
            hsla(0.0, 0.0, 0.0, 0.2)
        } else {
            text_style.color
        };
        let run = text_style_to_run_with_color(&text_style, display_text.len(), text_color);
        let line_height = window.line_height();
        let font_size = text_style.font_size.to_pixels(window.rem_size());
        let text_layout = if self.multiline {
            let line_starts = super::super::input::line_starts(&display_text);
            let lines = window
                .text_system()
                .shape_text(
                    SharedString::from(display_text),
                    font_size,
                    &[run],
                    Some(bounds.size.width),
                    None,
                )
                .unwrap_or_default()
                .into_iter()
                .collect();
            super::super::input::TextInputTextLayout::Multiline {
                lines,
                line_starts,
                line_height,
            }
        } else {
            let line = window.text_system().shape_line(
                SharedString::from(display_text),
                font_size,
                &[run],
                None,
            );
            super::super::input::TextInputTextLayout::Single {
                line: Box::new(line),
                line_height,
            }
        };
        let scroll_offset = if is_placeholder {
            Point::default()
        } else {
            let target = if let Some(marked) = marked.as_ref() {
                let start = super::super::input::utf16_byte_index(&content, marked.start);
                let end = super::super::input::utf16_byte_index(&content, marked.end);
                text_layout.bounds_for_range(start..end, bounds)
            } else {
                let caret = if selection_reversed {
                    selection.start
                } else {
                    selection.end
                };
                let byte = super::super::input::utf16_byte_index(&content, caret);
                text_layout.target_bounds_for_utf8(byte, bounds)
            };
            super::super::input::TextInputTextLayout::adjust_scroll_offset(
                current_offset,
                target,
                bounds,
                text_layout.content_size(),
            )
        };
        let (cursor, selection_quad) = if is_placeholder || selection.start == selection.end {
            let byte_offset = if is_placeholder {
                0
            } else {
                super::super::input::utf16_byte_index(&content, selection.start)
            };
            let position = text_layout.position_for_utf8(byte_offset);
            (
                Some(fill(
                    Bounds::new(
                        bounds.origin + position.point - scroll_offset,
                        size(px(2.0), position.line_height),
                    ),
                    rgba(0x2d6cdfff),
                )),
                None,
            )
        } else {
            let start = super::super::input::utf16_byte_index(&content, selection.start);
            let end = super::super::input::utf16_byte_index(&content, selection.end);
            (
                None,
                Some(fill(
                    text_layout.bounds_for_range_with_offset(start..end, bounds, scroll_offset),
                    rgba(0x2d6cdf66),
                )),
            )
        };
        TextInputPrepaint {
            text: text_layout,
            cursor,
            selection: selection_quad,
            scroll_offset,
            content,
            placeholder: is_placeholder,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if self.focus.is_focused(window) {
            self.entity
                .update(cx, |root, _| root.set_input_focus(self.node_id, true));
        }
        let scroll_offset = prepaint.scroll_offset;
        self.entity.update(cx, |root, _| {
            if let Some(state) = root.input_states.get_mut(&self.node_id) {
                state.scroll_offset = scroll_offset;
            }
        });
        window.handle_input(
            &self.focus,
            ElementInputHandler::new(bounds, self.entity.clone()),
            cx,
        );
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            if let Some(selection) = prepaint.selection.take() {
                window.paint_quad(selection);
            }
            prepaint
                .text
                .paint(bounds.origin, bounds, scroll_offset, window, cx);
            if self.focus.is_focused(window)
                && let Some(cursor) = prepaint.cursor.take()
            {
                window.paint_quad(cursor);
            }
        });
        let text = std::mem::take(&mut prepaint.text);
        let content = prepaint.content.clone();
        let placeholder = prepaint.placeholder;
        let node_id = self.node_id;
        self.entity.update(cx, |root, _| {
            if placeholder {
                root.text_input_layouts.remove(&node_id);
            } else {
                root.text_input_layouts.insert(
                    node_id,
                    super::super::input::TextInputLayout {
                        text,
                        bounds,
                        scroll_offset,
                        content,
                        placeholder,
                    },
                );
            }
        });
    }
}
struct SelectableTextElement {
    entity: Entity<ReactRoot>,
    node_id: u32,
    text: String,
}

impl IntoElement for SelectableTextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SelectableTextElement {
    type RequestLayoutState = ();
    type PrepaintState = SelectableTextPrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = gpui::Style::default();
        style.size.width = relative(1.).into();
        let line_count = self.text.split('\n').count().max(1);
        style.size.height = (window.line_height() * line_count as f32).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let root = self.entity.read(cx);
        let mut selection = root
            .selectable_text_selections
            .get(&self.node_id)
            .cloned()
            .unwrap_or_default();
        selection.start = selection.start.min(self.text.len());
        selection.end = selection.end.min(self.text.len());
        let text_style = window.text_style();
        let run = text_style_to_run(&text_style, self.text.len());
        let line_height = window.line_height();
        let font_size = text_style.font_size.to_pixels(window.rem_size());
        let lines = window
            .text_system()
            .shape_text(
                SharedString::from(self.text.clone()),
                font_size,
                &[run],
                Some(bounds.size.width),
                None,
            )
            .unwrap_or_default()
            .into_iter()
            .collect();
        let text_layout = super::super::input::TextInputTextLayout::Multiline {
            lines,
            line_starts: super::super::input::line_starts(&self.text),
            line_height,
        };
        let selection = text_layout
            .selection_bounds_per_line(selection, bounds)
            .into_iter()
            .map(|bounds| fill(bounds, rgba(0x2d6cdf66)))
            .collect();
        SelectableTextPrepaint {
            text: text_layout,
            selection,
            content: self.text.clone(),
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        for selection in prepaint.selection.drain(..) {
            window.paint_quad(selection);
        }
        prepaint
            .text
            .paint(bounds.origin, bounds, Point::default(), window, cx);
        let text = std::mem::take(&mut prepaint.text);
        let content = prepaint.content.clone();
        let node_id = self.node_id;
        self.entity.update(cx, |root, _| {
            root.selectable_text_layouts.insert(
                node_id,
                super::super::input::TextInputLayout {
                    text,
                    bounds,
                    scroll_offset: Point::default(),
                    content,
                    placeholder: false,
                },
            );
        });
    }
}
pub(super) fn input_display_text(actual: String, placeholder: Option<&str>) -> (String, bool) {
    if actual.is_empty()
        && let Some(placeholder) = placeholder.filter(|value| !value.is_empty())
    {
        return (placeholder.to_owned(), true);
    }
    (actual, false)
}

pub(super) fn render_text_input(
    root: &ReactRoot,
    node: &StoredNode,
    entity: &Entity<ReactRoot>,
    style: Option<&Style>,
) -> AnyElement {
    let Some(HostProperties::TextInput(input)) = node.host_properties.as_ref() else {
        return div().id(ElementId::Integer(node.id as u64)).into_any();
    };
    let focus = root
        .focus_handles
        .get(&node.id)
        .cloned()
        .expect("TextInput focus handle is reconciled before render");
    let input_id = node.id;
    let mut input_element = apply_style(div(), style);
    let actual_text = root
        .input_states
        .get(&node.id)
        .map(|state| state.text.clone())
        .unwrap_or_else(|| input.value.clone());
    let submit_entity = entity.clone();
    let submit_focus = focus.clone();
    let input_multiline = input.multiline;
    input_element = input_element.on_key_down(move |event, window, app| {
        if !input_multiline
            && !event.is_held
            && event.keystroke.key == "enter"
            && submit_focus.is_focused(window)
        {
            submit_entity.update(app, |root, _| root.emit_submit_event(input_id));
        }
    });
    let navigation_entity = entity.clone();
    input_element = input_element.on_key_down(move |event, _, app| {
        let modifiers = event.keystroke.modifiers;
        if !modifiers.control && !modifiers.platform {
            navigation_entity.update(app, |root, cx| {
                root.handle_text_input_navigation(
                    input_id,
                    &event.keystroke.key,
                    modifiers.shift,
                    modifiers.alt,
                    cx,
                );
            });
        }
    });
    let clipboard_entity = entity.clone();
    input_element = input_element.on_key_down(move |event, _, app| {
        let modifiers = event.keystroke.modifiers;
        if event.is_held
            || modifiers.shift
            || modifiers.alt
            || (!modifiers.platform && !modifiers.control)
        {
            return;
        }
        let key = event.keystroke.key.as_str();
        if key.eq_ignore_ascii_case("c") {
            if let Some(text) = clipboard_entity.read(app).selected_text_for_copy(input_id) {
                app.write_to_clipboard(ClipboardItem::new_string(text));
                app.stop_propagation();
            }
        } else if key.eq_ignore_ascii_case("x") {
            if let Some(text) =
                clipboard_entity.update(app, |root, cx| root.cut_text_input_selection(input_id, cx))
            {
                app.write_to_clipboard(ClipboardItem::new_string(text));
                app.stop_propagation();
            }
        } else if key.eq_ignore_ascii_case("v")
            && let Some(text) = app.read_from_clipboard().and_then(|item| item.text())
            && clipboard_entity.update(app, |root, cx| root.replace_text_input(input_id, &text, cx))
        {
            app.stop_propagation();
        } else if key.eq_ignore_ascii_case("a")
            && clipboard_entity.update(app, |root, cx| root.select_all_text_input(input_id, cx))
        {
            app.stop_propagation();
        }
    });
    let history_entity = entity.clone();
    input_element = input_element.on_key_down(move |event, _, app| {
        let modifiers = event.keystroke.modifiers;
        if event.is_held || modifiers.alt || (!modifiers.platform && !modifiers.control) {
            return;
        }
        let key = event.keystroke.key.as_str();
        let undo = key.eq_ignore_ascii_case("z") && !modifiers.shift;
        let redo = (key.eq_ignore_ascii_case("z") && modifiers.shift)
            || (key.eq_ignore_ascii_case("y") && modifiers.control && !modifiers.platform);
        let changed = if undo {
            history_entity.update(app, |root, cx| root.undo_text_input(input_id, cx))
        } else if redo {
            history_entity.update(app, |root, cx| root.redo_text_input(input_id, cx))
        } else {
            false
        };
        if changed {
            app.stop_propagation();
        }
    });
    let deletion_entity = entity.clone();
    input_element = input_element.on_key_down(move |event, _, app| {
        let modifiers = event.keystroke.modifiers;
        if event.is_held || modifiers.platform || (!modifiers.control && modifiers.shift) {
            return;
        }
        let key = event.keystroke.key.as_str();
        let backward = key.eq_ignore_ascii_case("backspace");
        let forward = key.eq_ignore_ascii_case("delete");
        if backward || forward {
            let wordwise = modifiers.alt;
            if deletion_entity.update(app, |root, cx| {
                root.delete_text_input(input_id, backward, wordwise, cx)
            }) {
                app.stop_propagation();
            }
        }
    });
    if node.listener_id != 0 {
        let runtime = Arc::clone(&root.runtime);
        let sequence = Arc::clone(&root.next_sequence);
        let surface_id = root.store.surface_id();
        let epoch = root.store.epoch();
        let revision = root.store.revision();
        let listener_id = node.listener_id;
        input_element = input_element.on_key_down(move |event, _, _| {
            emit_key_event(
                runtime.as_ref(),
                sequence.as_ref(),
                surface_id,
                epoch,
                revision,
                input_id,
                listener_id,
                &event.keystroke.key,
                &event.keystroke.modifiers,
                if event.is_held {
                    KeyAction::Repeat
                } else {
                    KeyAction::Down
                },
            );
        });
        let runtime = Arc::clone(&root.runtime);
        let sequence = Arc::clone(&root.next_sequence);
        input_element = input_element.on_key_up(move |event, _, _| {
            emit_key_event(
                runtime.as_ref(),
                sequence.as_ref(),
                surface_id,
                epoch,
                revision,
                input_id,
                listener_id,
                &event.keystroke.key,
                &event.keystroke.modifiers,
                KeyAction::Up,
            );
        });
    }
    let mouse_entity = entity.clone();
    let mouse_focus = focus.clone();
    input_element = input_element.on_mouse_down(MouseButton::Left, move |event, window, app| {
        window.focus(&mouse_focus, app);
        mouse_entity.update(app, |root, cx| {
            root.begin_text_input_selection(
                input_id,
                event.position,
                event.modifiers.shift,
                event.click_count,
                window,
                cx,
            );
        });
    });
    let mouse_entity = entity.clone();
    input_element = input_element.on_mouse_move(move |event, window, app| {
        if event.dragging() {
            mouse_entity.update(app, |root, cx| {
                root.update_text_input_selection(input_id, event.position, window, cx);
            });
        }
    });
    let mouse_entity = entity.clone();
    input_element = input_element.on_mouse_up(MouseButton::Left, move |_, _, app| {
        mouse_entity.update(app, |root, _| root.end_text_input_selection(input_id));
    });
    let mouse_entity = entity.clone();
    input_element = input_element.on_mouse_up_out(MouseButton::Left, move |_, _, app| {
        mouse_entity.update(app, |root, _| root.end_text_input_selection(input_id));
    });
    let bounded_height = style
        .and_then(|style| style.height.or(style.max_height))
        .is_some();
    let input_element = input_element
        .child(TextInputElement {
            entity: entity.clone(),
            node_id: input_id,
            focus: focus.clone(),
            fallback_text: input.value.clone(),
            placeholder: input.placeholder.clone().unwrap_or_default(),
            multiline: input.multiline || actual_text.contains('\n'),
            bounded_height,
        })
        .id(ElementId::Integer(node.id as u64))
        .focusable()
        .tab_stop(!input.disabled)
        .track_focus(&focus);
    apply_accessibility(input_element, node).into_any()
}

pub(super) fn render_selectable(
    root: &ReactRoot,
    node: &StoredNode,
    entity: &Entity<ReactRoot>,
    style: Option<&Style>,
) -> AnyElement {
    let node_id = node.id;
    let text = node
        .text_content
        .as_ref()
        .map(|text| text.to_string())
        .unwrap_or_default();
    let focus = root
        .focus_handles
        .get(&node_id)
        .cloned()
        .expect("selectable Text focus handle is reconciled before render");
    let mut element = div().id(ElementId::Integer(node.id as u64));
    if node.id == 1 {
        element = element.size_full().flex().flex_col();
    }
    element = apply_style(element, style);
    element = apply_text_style(element, style);
    let mut selectable = element
        .focusable()
        .tab_stop(true)
        .track_focus(&focus)
        .cursor(gpui::CursorStyle::IBeam)
        .child(SelectableTextElement {
            entity: entity.clone(),
            node_id,
            text,
        });
    let mouse_entity = entity.clone();
    let mouse_focus = focus.clone();
    selectable = selectable.on_mouse_down(MouseButton::Left, move |event, window, app| {
        window.focus(&mouse_focus, app);
        mouse_entity.update(app, |root, cx| {
            root.begin_selectable_text_selection(node_id, event.position, cx);
        });
    });
    let mouse_entity = entity.clone();
    selectable = selectable.on_mouse_move(move |event, _, app| {
        if event.dragging() {
            mouse_entity.update(app, |root, cx| {
                root.update_selectable_text_selection(node_id, event.position, cx);
            });
        }
    });
    let mouse_entity = entity.clone();
    selectable = selectable.on_mouse_up(MouseButton::Left, move |_, _, app| {
        mouse_entity.update(app, |root, _| root.end_selectable_text_selection(node_id));
    });
    let mouse_entity = entity.clone();
    selectable = selectable.on_mouse_up_out(MouseButton::Left, move |_, _, app| {
        mouse_entity.update(app, |root, _| root.end_selectable_text_selection(node_id));
    });
    let key_entity = entity.clone();
    selectable = selectable.on_key_down(move |event, _, app| {
        let modifiers = event.keystroke.modifiers;
        if !event.is_held
            && !modifiers.shift
            && (modifiers.platform || modifiers.control)
            && event.keystroke.key.eq_ignore_ascii_case("c")
            && let Some(text) = key_entity.read(app).selected_selectable_text(node_id)
        {
            app.write_to_clipboard(ClipboardItem::new_string(text));
            app.stop_propagation();
        }
    });
    super::measure_node(
        node,
        apply_accessibility(selectable, node).into_any(),
        entity,
    )
}
