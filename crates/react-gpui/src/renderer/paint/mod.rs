use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use gpui::{
    AnyElement, App, AppContext, Bounds, Element, ElementId, Entity, GlobalElementId,
    InspectorElementId, InteractiveElement, IntoElement, LayoutId, MouseButton, ParentElement,
    Pixels, Render, SharedString, StatefulInteractiveElement, Styled, StyledText, Window, div, px,
    rgba,
};

use crate::protocol::{EVENT_POINTER_DOWN, EVENT_POINTER_UP, Event, KeyAction};
use crate::transport::send_event_or_exit;
use crate::tree::{
    KIND_IMAGE, KIND_PRESSABLE, KIND_RAW_TEXT, KIND_TEXT, KIND_TEXT_INPUT, KIND_VIEW,
    KIND_VIRTUAL_LIST, StoredNode,
};

use super::ReactRoot;
use super::events::{emit_key_event, emit_pointer_event, emit_pointer_move, emit_scroll_event};
pub(super) type RenderedBounds = Rc<RefCell<HashMap<u32, (f32, f32, f32, f32)>>>;
pub(super) type LinkAffordanceBounds = Rc<RefCell<HashMap<u32, Vec<(f32, f32, f32, f32)>>>>;
struct TooltipView {
    text: SharedString,
}

impl Render for TooltipView {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(4.0))
            .bg(rgba(0x1f2937f5))
            .text_color(rgba(0xf8fafcff))
            .text_size(px(11.0))
            .line_height(px(16.0))
            .child(self.text.clone())
    }
}

mod accessibility;
mod drag;
mod image;
mod overlay;
mod style;
mod text_input;
pub(super) use text_input::RichTextParts;
mod virtual_list;

#[cfg(test)]
pub(super) fn accessibility_role(role: u32) -> Option<gpui::accesskit::Role> {
    accessibility::accessibility_role(role)
}

#[cfg(test)]
pub(super) fn input_display_text(actual: String, placeholder: Option<&str>) -> (String, bool) {
    text_input::input_display_text(actual, placeholder)
}

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
            return text_input::render_text_input(self, node, entity, style);
        }
        if node.kind == KIND_IMAGE {
            return image::render(node, entity, style);
        }
        if node.kind == KIND_VIRTUAL_LIST {
            return virtual_list::render(self, node, entity, style);
        }

        let mut element = div().id(ElementId::Integer(node.id as u64));
        if node.id == 1 {
            element = element.size_full().flex().flex_col();
        }
        element = style::apply_style(element, style);
        if node.kind == KIND_RAW_TEXT || node.kind == KIND_TEXT {
            element = style::apply_text_style(element, style);
        }
        if node.kind == KIND_TEXT && node.selectable {
            return text_input::render_selectable(self, node, entity, style);
        }
        if node.kind == KIND_TEXT && node.has_children() {
            return text_input::render_rich_text(self, node, style);
        }

        if node.kind == KIND_RAW_TEXT {
            let element = element.child(
                node.text
                    .as_ref()
                    .map(|text| SharedString::new(Arc::clone(text)))
                    .unwrap_or_default(),
            );
            return accessibility::apply_accessibility(element, node).into_any();
        }
        if node.kind == KIND_TEXT {
            let mut text = String::new();
            let mut runs = Vec::new();
            let text_style = gpui::TextStyle::default();
            if let Some(content) = node.text_content.as_ref() {
                text.push_str(content);
                runs.push(style::text_run(&text_style, style, content.len()));
            }
            for child in node.children(&self.store) {
                if child.kind != KIND_TEXT {
                    continue;
                }
                let Some(content) = child.text_content.as_ref() else {
                    continue;
                };
                let start = text.len();
                text.push_str(content);
                runs.push(style::text_run(
                    &text_style,
                    child.style.as_ref(),
                    text.len() - start,
                ));
            }
            if !text.is_empty() {
                element = element.child(StyledText::new(SharedString::from(text)).with_runs(runs));
            }
        } else {
            element = element.children(
                node.children(&self.store)
                    .map(|child| self.render_node(child, entity)),
            );
        }
        element = accessibility::apply_accessibility(element, node);

        if matches!(node.kind, KIND_VIEW | KIND_PRESSABLE | KIND_TEXT)
            && node.focusable
            && node.listener_id != 0
        {
            let focus = self
                .focus_handles
                .get(&node.id)
                .cloned()
                .expect("focusable interactive focus handle is reconciled before render");
            let runtime = Arc::clone(&self.runtime);
            let sequence = Arc::clone(&self.next_sequence);
            let surface_id = self.store.surface_id();
            let epoch = self.store.epoch();
            let revision = self.store.revision();
            let node_id = node.id;
            let listener_id = node.listener_id;
            let node_kind = node.kind;
            element = element
                .focusable()
                .tab_stop(true)
                .track_focus(&focus)
                .on_key_down(move |event, window, app| {
                    if node_kind == KIND_TEXT {
                        if event.keystroke.key == "enter"
                            && !event.is_held
                            && event.keystroke.modifiers == gpui::Modifiers::none()
                            && focus.is_focused(window)
                        {
                            let event = Event::press(
                                surface_id,
                                epoch,
                                revision,
                                sequence.fetch_add(1, Ordering::Relaxed),
                                node_id,
                                listener_id,
                            );
                            send_event_or_exit(runtime.as_ref(), "text run press event", &event);
                            app.stop_propagation();
                        }
                    } else {
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
                    }
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
                element = element.on_mouse_down(button, move |event, window, _| {
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
                        event.position,
                        window.viewport_size(),
                    );
                });
            }
            for button in MouseButton::all() {
                let runtime = Arc::clone(&self.runtime);
                let sequence = Arc::clone(&self.next_sequence);
                element = element.on_mouse_up(button, move |event, window, _| {
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
                        event.position,
                        window.viewport_size(),
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
        if (node.kind == KIND_VIEW || node.kind == KIND_PRESSABLE)
            && node.accepts_pointer_move
            && node.listener_id != 0
        {
            let runtime = Arc::clone(&self.runtime);
            let sequence = Arc::clone(&self.next_sequence);
            let surface_id = self.store.surface_id();
            let epoch = self.store.epoch();
            let revision = self.store.revision();
            let node_id = node.id;
            let listener_id = node.listener_id;
            element = element.on_mouse_move(move |event, window, _| {
                emit_pointer_move(
                    runtime.as_ref(),
                    sequence.as_ref(),
                    surface_id,
                    epoch,
                    revision,
                    node_id,
                    listener_id,
                    event.position,
                    &event.modifiers,
                    window.viewport_size(),
                );
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
        if (node.kind == KIND_VIEW || node.kind == KIND_PRESSABLE)
            && let Some(tooltip) = node.tooltip.as_ref()
        {
            let tooltip = SharedString::new(Arc::clone(tooltip));
            element = element.tooltip(move |_, cx| {
                cx.new(|_| TooltipView {
                    text: tooltip.clone(),
                })
                .into()
            });
        }
        element = drag::apply(element, self, node);
        let element = overlay::apply(self, element.into_any(), node, style, entity);
        measure_node(node, element, entity)
    }
}
