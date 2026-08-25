use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, ElementInputHandler, Entity, GlobalElementId,
    ImageSource, InspectorElementId, InteractiveElement, IntoElement, LayoutId, MouseButton,
    ObjectFit, ParentElement, Pixels, SharedString, StatefulInteractiveElement, Styled,
    StyledImage, Window, div, img, px, rgba, uniform_list,
};

use crate::protocol::{
    EVENT_POINTER_DOWN, EVENT_POINTER_UP, Event, HostProperties, KeyAction, Style,
};
use crate::transport::send_event_or_exit;
use crate::tree::{
    KIND_IMAGE, KIND_PRESSABLE, KIND_RAW_TEXT, KIND_TEXT, KIND_TEXT_INPUT, KIND_VIEW,
    KIND_VIRTUAL_LIST, StoredNode,
};

use super::ReactRoot;
use super::committed_child_index;
use super::events::{emit_key_event, emit_pointer_event, emit_scroll_event};
struct MeasuredElement {
    element: AnyElement,
    entity: Entity<ReactRoot>,
    node_id: u32,
}

impl IntoElement for MeasuredElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for MeasuredElement {
    type RequestLayoutState = ();
    type PrepaintState = Option<gpui::FocusHandle>;

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
        (self.element.request_layout(window, cx), ())
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
        let frame = (
            f32::from(bounds.origin.x),
            f32::from(bounds.origin.y),
            f32::from(bounds.size.width),
            f32::from(bounds.size.height),
        );
        if [frame.0, frame.1, frame.2, frame.3]
            .into_iter()
            .all(f32::is_finite)
        {
            let entity = self.entity.clone();
            let node_id = self.node_id;
            window.on_next_frame(move |_, app| {
                entity.update(app, |root, _| root.emit_layout_bounds(node_id, frame));
            });
        }
        self.element.prepaint(window, cx)
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.element.paint(window, cx);
    }
}

fn measure_node(node: &StoredNode, element: AnyElement, entity: &Entity<ReactRoot>) -> AnyElement {
    if node.listener_id == 0
        || !matches!(
            node.kind,
            KIND_VIEW | KIND_PRESSABLE | KIND_TEXT | KIND_IMAGE
        )
    {
        return element;
    }
    MeasuredElement {
        element,
        entity: entity.clone(),
        node_id: node.id,
    }
    .into_any()
}

impl ReactRoot {
    pub(super) fn render_node(&self, node: &StoredNode, entity: &Entity<Self>) -> AnyElement {
        let style = self.style_for_node(node);
        if node.kind == KIND_TEXT_INPUT {
            let Some(HostProperties::TextInput(input)) = node.host_properties.as_ref() else {
                return div().id(ElementId::Integer(node.id as u64)).into_any();
            };
            let focus = self
                .focus_handles
                .get(&node.id)
                .cloned()
                .expect("TextInput focus handle is reconciled before render");
            let input_entity = entity.clone();
            let input_focus = focus.clone();
            let input_id = node.id;
            let mut input_element = apply_style(div(), style);
            input_element = input_element.on_children_prepainted(move |bounds, window, app| {
                if input_focus.is_focused(window) {
                    input_entity.update(app, |root, _| root.set_input_focus(input_id, true));
                    if let Some(bounds) = bounds.into_iter().next() {
                        window.handle_input(
                            &input_focus,
                            ElementInputHandler::new(bounds, input_entity.clone()),
                            app,
                        );
                    }
                }
            });
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
            if node.listener_id != 0 {
                let runtime = Arc::clone(&self.runtime);
                let sequence = Arc::clone(&self.next_sequence);
                let surface_id = self.store.surface_id();
                let epoch = self.store.epoch();
                let revision = self.store.revision();
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
                let runtime = Arc::clone(&self.runtime);
                let sequence = Arc::clone(&self.next_sequence);
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
            let display_text = self
                .input_states
                .get(&node.id)
                .map(|state| state.text.clone())
                .unwrap_or_else(|| input.value.clone());
            let input_element = input_element
                .child(SharedString::from(display_text))
                .id(ElementId::Integer(node.id as u64))
                .focusable()
                .track_focus(&focus);
            return apply_accessibility(input_element, node).into_any();
        }
        if node.kind == KIND_IMAGE {
            let Some(HostProperties::Image(image)) = node.host_properties.as_ref() else {
                return div().id(ElementId::Integer(node.id as u64)).into_any();
            };
            let object_fit = match image.object_fit {
                1 => ObjectFit::Fill,
                2 => ObjectFit::Contain,
                3 => ObjectFit::Cover,
                4 => ObjectFit::ScaleDown,
                5 => ObjectFit::None,
                _ => unreachable!(),
            };
            let image_element =
                img(ImageSource::from(PathBuf::from(&image.source))).object_fit(object_fit);
            return measure_node(node, apply_style(image_element, style).into_any(), entity);
        }
        if node.kind == KIND_VIRTUAL_LIST {
            let Some(HostProperties::VirtualList(list)) = node.host_properties.as_ref() else {
                return div().id(ElementId::Integer(node.id as u64)).into_any();
            };
            let handle = self
                .virtual_handles
                .get(&node.id)
                .cloned()
                .expect("VirtualList scroll handle is reconciled before render");
            let list_id = node.id;
            let item_count = list.item_count as usize;
            let committed_start = list.range_start;
            let committed_end = list.range_end;
            let estimated = list.estimated_item_size;
            let overscan = list.overscan;
            let pending = Rc::clone(&self.pending_visible_ranges);
            let list_entity = entity.clone();
            let mut list_element = uniform_list(
                ElementId::Integer(list_id as u64),
                item_count,
                move |range, window, app| {
                    let start = range.start as u32;
                    let end = range.end as u32;
                    let report_start = start.saturating_sub(overscan);
                    let report_end = end.saturating_add(overscan).min(item_count as u32);
                    pending
                        .borrow_mut()
                        .insert(list_id, (report_start, report_end));
                    let pending_for_frame = Rc::clone(&pending);
                    let entity_for_frame = list_entity.clone();
                    window.on_next_frame(move |_, app| {
                        let range = pending_for_frame.borrow_mut().remove(&list_id);
                        if let Some((start, end)) = range {
                            entity_for_frame.update(app, |root, _| {
                                root.emit_visible_range(list_id, start, end)
                            });
                        }
                    });
                    list_entity.update(app, |root, _cx| {
                        range
                            .map(|absolute_index| {
                                let child = committed_child_index(
                                    absolute_index as u32,
                                    committed_start,
                                    committed_end,
                                )
                                .and_then(|offset| root.store.get_child_at(list_id, offset))
                                .cloned();
                                let row_id = ((list_id as u64) << 32) | absolute_index as u64;
                                let mut row = div().id(ElementId::Integer(row_id)).h(px(estimated));
                                if let Some(child) = child {
                                    row = row.child(root.render_node(&child, &list_entity));
                                }
                                row.into_any()
                            })
                            .collect::<Vec<_>>()
                    })
                },
            )
            .track_scroll(&handle);
            list_element = apply_style(list_element, style);
            return list_element.into_any();
        }
        let mut element = div().id(ElementId::Integer(node.id as u64));
        if node.id == 1 {
            element = element.size_full().flex().flex_col();
        }
        element = apply_style(element, style);
        if node.kind == KIND_RAW_TEXT || node.kind == KIND_TEXT {
            element = apply_text_style(element, style);
        }
        if node.kind == KIND_RAW_TEXT {
            return element
                .child(
                    node.text
                        .as_ref()
                        .map(|text| SharedString::new(Arc::clone(text)))
                        .unwrap_or_default(),
                )
                .into_any();
        }
        if node.kind == KIND_TEXT {
            if let Some(text) = node.text_content.as_ref() {
                element = element.child(SharedString::new(Arc::clone(text)));
            }
        } else {
            element = element.children(
                node.children(&self.store)
                    .map(|child| self.render_node(child, entity)),
            );
        }
        element = apply_accessibility(element, node);
        if (node.kind == KIND_VIEW || node.kind == KIND_PRESSABLE)
            && node.focusable
            && node.listener_id != 0
        {
            let focus = self
                .focus_handles
                .get(&node.id)
                .cloned()
                .expect("focusable View or Pressable focus handle is reconciled before render");
            let runtime = Arc::clone(&self.runtime);
            let sequence = Arc::clone(&self.next_sequence);
            let surface_id = self.store.surface_id();
            let epoch = self.store.epoch();
            let revision = self.store.revision();
            let node_id = node.id;
            let listener_id = node.listener_id;
            element = element
                .focusable()
                .track_focus(&focus)
                .on_key_down(move |event, _, _| {
                    emit_key_event(
                        runtime.as_ref(),
                        sequence.as_ref(),
                        surface_id,
                        epoch,
                        revision,
                        node_id,
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
            let runtime = Arc::clone(&self.runtime);
            let sequence = Arc::clone(&self.next_sequence);
            element = element.on_key_up(move |event, _, _| {
                emit_key_event(
                    runtime.as_ref(),
                    sequence.as_ref(),
                    surface_id,
                    epoch,
                    revision,
                    node_id,
                    listener_id,
                    &event.keystroke.key,
                    &event.keystroke.modifiers,
                    KeyAction::Up,
                );
            });
        }
        if node.kind == KIND_PRESSABLE && node.listener_id != 0 {
            let runtime = Arc::clone(&self.runtime);
            let sequence = Arc::clone(&self.next_sequence);
            let surface_id = self.store.surface_id();
            let epoch = self.store.epoch();
            let revision = self.store.revision();
            let node_id = node.id;
            let listener_id = node.listener_id;
            element = element.on_click(move |_, _, _| {
                let event = Event::press(
                    surface_id,
                    epoch,
                    revision,
                    sequence.fetch_add(1, Ordering::Relaxed),
                    node_id,
                    listener_id,
                );
                send_event_or_exit(runtime.as_ref(), "press event", &event);
            });
        }
        if (node.kind == KIND_VIEW || node.kind == KIND_PRESSABLE) && node.listener_id != 0 {
            let surface_id = self.store.surface_id();
            let epoch = self.store.epoch();
            let revision = self.store.revision();
            let node_id = node.id;
            let listener_id = node.listener_id;
            for button in MouseButton::all() {
                let runtime = Arc::clone(&self.runtime);
                let sequence = Arc::clone(&self.next_sequence);
                element = element.on_mouse_down(button, move |event, _, _| {
                    emit_pointer_event(
                        runtime.as_ref(),
                        sequence.as_ref(),
                        surface_id,
                        epoch,
                        revision,
                        node_id,
                        listener_id,
                        EVENT_POINTER_DOWN,
                        event.button,
                        &event.modifiers,
                        event.click_count,
                    );
                });
            }
            for button in MouseButton::all() {
                let runtime = Arc::clone(&self.runtime);
                let sequence = Arc::clone(&self.next_sequence);
                element = element.on_mouse_up(button, move |event, _, _| {
                    emit_pointer_event(
                        runtime.as_ref(),
                        sequence.as_ref(),
                        surface_id,
                        epoch,
                        revision,
                        node_id,
                        listener_id,
                        EVENT_POINTER_UP,
                        event.button,
                        &event.modifiers,
                        event.click_count,
                    );
                });
            }
            let runtime = Arc::clone(&self.runtime);
            let sequence = Arc::clone(&self.next_sequence);
            element = element.on_hover(move |_, _, _| {
                let event = Event::hover(
                    surface_id,
                    epoch,
                    revision,
                    sequence.fetch_add(1, Ordering::Relaxed),
                    node_id,
                    listener_id,
                );
                send_event_or_exit(runtime.as_ref(), "hover event", &event);
            });
        }
        if node.kind == KIND_VIEW && node.listener_id != 0 {
            let runtime = Arc::clone(&self.runtime);
            let sequence = Arc::clone(&self.next_sequence);
            let surface_id = self.store.surface_id();
            let epoch = self.store.epoch();
            let revision = self.store.revision();
            let node_id = node.id;
            let listener_id = node.listener_id;
            element = element.on_scroll_wheel(move |event, _, _| {
                emit_scroll_event(
                    runtime.as_ref(),
                    sequence.as_ref(),
                    surface_id,
                    epoch,
                    revision,
                    node_id,
                    listener_id,
                    event,
                );
            });
        }
        measure_node(node, element.into_any(), entity)
    }
}

fn apply_accessibility<E: StatefulInteractiveElement>(mut element: E, node: &StoredNode) -> E {
    let Some(accessibility) = node.accessibility.as_ref() else {
        return element;
    };
    let role = match accessibility.role {
        1 => gpui::accesskit::Role::GenericContainer,
        2 => gpui::accesskit::Role::Button,
        3 => gpui::accesskit::Role::Label,
        4 => gpui::accesskit::Role::TextInput,
        5 => gpui::accesskit::Role::CheckBox,
        6 => gpui::accesskit::Role::Heading,
        _ => return element,
    };
    if role != gpui::accesskit::Role::GenericContainer {
        element = element.role(role);
    }
    element = element.accessibility_id(Arc::clone(&node.accessibility_id));
    if let Some(label) = &accessibility.label {
        element = element.aria_label(label.clone());
    }
    if let Some(description) = &accessibility.description {
        element = element.aria_description(description.clone());
    }
    if let Some(selected) = accessibility.selected {
        element = element.aria_selected(selected);
    }
    if let Some(checked) = accessibility.checked {
        element = element.aria_toggled(if checked {
            gpui::accesskit::Toggled::True
        } else {
            gpui::accesskit::Toggled::False
        });
    }
    if let Some(value) = &accessibility.value {
        element = element.aria_value(value.clone());
    }
    element
}

fn apply_style<E: Styled>(mut element: E, style: Option<&Style>) -> E {
    let Some(style) = style else { return element };
    if let Some(width) = style.width {
        element = element.w(px(width));
    }
    if let Some(height) = style.height {
        element = element.h(px(height));
    }
    if let Some(position) = style.position {
        element = if position == 1 {
            element.absolute()
        } else {
            element.relative()
        };
    }
    if let Some(left) = style.left {
        element = element.left(px(left));
    }
    if let Some(top) = style.top {
        element = element.top(px(top));
    }
    if let Some(right) = style.right {
        element = element.right(px(right));
    }
    if let Some(bottom) = style.bottom {
        element = element.bottom(px(bottom));
    }
    if let Some(direction) = style.flex_direction {
        element = match direction {
            1 => element.flex_row(),
            2 => element.flex_col(),
            _ => element,
        };
    }
    if let Some(grow) = style.flex_grow {
        element.style().flex_grow = Some(grow);
    }
    if let Some(padding) = style.padding {
        element = element.p(px(padding));
    }
    if let Some(gap) = style.gap {
        element = element.gap(px(gap));
    }
    if let Some(margin) = style.margin_top {
        element = element.mt(px(margin));
    }
    if let Some(margin) = style.margin_right {
        element = element.mr(px(margin));
    }
    if let Some(margin) = style.margin_bottom {
        element = element.mb(px(margin));
    }
    if let Some(margin) = style.margin_left {
        element = element.ml(px(margin));
    }
    if let Some(min_width) = style.min_width {
        element = element.min_w(px(min_width));
    }
    if let Some(max_width) = style.max_width {
        element = element.max_w(px(max_width));
    }
    if let Some(min_height) = style.min_height {
        element = element.min_h(px(min_height));
    }
    if let Some(max_height) = style.max_height {
        element = element.max_h(px(max_height));
    }
    if let Some(shrink) = style.flex_shrink {
        element.style().flex_shrink = Some(shrink);
    }
    if let Some(justify) = style.justify_content {
        element = match justify {
            1 => element.justify_start(),
            2 => element.justify_center(),
            3 => element.justify_end(),
            4 => element.justify_between(),
            5 => element.justify_around(),
            6 => element.justify_evenly(),
            _ => element,
        };
    }
    if let Some(align) = style.align_items {
        element = match align {
            1 => element.items_start(),
            2 => element.items_center(),
            3 => element.items_end(),
            4 => element.items_stretch(),
            5 => element.items_baseline(),
            _ => element,
        };
    }
    if let Some(align_self) = style.align_self {
        element = match align_self {
            1 => element.self_start(),
            2 => element.self_end(),
            3 => element.self_flex_start(),
            4 => element.self_flex_end(),
            5 => element.self_center(),
            6 => element.self_baseline(),
            7 => element.self_stretch(),
            _ => element,
        };
    }
    if let Some(radius) = style.border_radius {
        element = element.rounded(px(radius));
    }
    if let Some(width) = style.border_width {
        element = element.border(px(width));
    }
    if let Some(color) = style.border_color_rgba {
        element = element.border_color(rgba(color));
    }
    if let Some(overflow) = style.overflow {
        element = match overflow {
            1 => {
                element.style().overflow.x = Some(gpui::Overflow::Visible);
                element.style().overflow.y = Some(gpui::Overflow::Visible);
                element
            }
            2 => {
                element.style().overflow.x = Some(gpui::Overflow::Hidden);
                element.style().overflow.y = Some(gpui::Overflow::Hidden);
                element
            }
            3 => {
                element.style().overflow.x = Some(gpui::Overflow::Scroll);
                element.style().overflow.y = Some(gpui::Overflow::Scroll);
                element
            }
            _ => element,
        };
    }
    if let Some(background) = style.background_rgba {
        element = element.bg(rgba(background));
    }
    if let Some(color) = style.color_rgba {
        element = element.text_color(rgba(color));
    }
    if let Some(opacity) = style.opacity {
        element = element.opacity(opacity);
    }
    element
}

fn apply_text_style<E: Styled>(mut element: E, style: Option<&Style>) -> E {
    let Some(style) = style else { return element };
    if let Some(font_style) = style.font_style {
        element = match font_style {
            0 => element.not_italic(),
            1 => element.italic(),
            _ => element,
        };
    }
    if let Some(text_decoration) = style.text_decoration {
        let text_style = element.text_style();
        text_style.underline = None;
        text_style.strikethrough = None;
        element = match text_decoration {
            0 => element.text_decoration_none(),
            1 => element.underline(),
            2 => element.line_through(),
            _ => element,
        };
    }
    if let Some(line_height) = style.line_height {
        element = element.line_height(px(line_height));
    }
    if let Some(line_clamp) = style.line_clamp {
        element.text_style().line_clamp = Some(line_clamp as usize);
        if style.overflow.is_none() {
            element.style().overflow.x = Some(gpui::Overflow::Hidden);
            element.style().overflow.y = Some(gpui::Overflow::Hidden);
        }
    }
    if let Some(text_overflow) = style.text_overflow {
        element = match text_overflow {
            1 => {
                element.text_style().text_overflow = None;
                element
            }
            2 => element.text_ellipsis(),
            _ => element,
        };
    }
    if let Some(size) = style.font_size {
        element = element.text_size(px(size));
    }
    if let Some(weight) = style.font_weight {
        element = element.font_weight(match weight {
            400 => gpui::FontWeight::NORMAL,
            500 => gpui::FontWeight::MEDIUM,
            600 => gpui::FontWeight::SEMIBOLD,
            700 => gpui::FontWeight::BOLD,
            900 => gpui::FontWeight::BLACK,
            _ => gpui::FontWeight::NORMAL,
        });
    }
    element
}
