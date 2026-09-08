use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use gpui::{
    AnyElement, App, AppContext, Bounds, Element, ElementId, Entity, GlobalElementId,
    InspectorElementId, InteractiveElement, IntoElement, LayoutId, MouseButton, ParentElement,
    Pixels, Render, SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgba,
};

use crate::protocol::{EVENT_POINTER_DOWN, EVENT_POINTER_UP, Event, KeyAction};
use crate::transport::send_event_or_exit;
use crate::tree::{
    KIND_EXTENSION, KIND_ICON, KIND_IMAGE, KIND_PRESSABLE, KIND_RAW_TEXT, KIND_TEXT,
    KIND_TEXT_INPUT, KIND_VIEW, KIND_VIRTUAL_LIST, StoredNode,
};

use super::SolidRoot;
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
mod border;
mod drag;
mod icon;
mod image;
mod overlay;
mod presentation;
pub(super) use presentation::presentation;
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
    entity: Entity<SolidRoot>,
    node_id: u32,
    namespace: bool,
    observe: bool,
    route: Option<super::extensions::ExtensionEventSink>,
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
        self.namespace
            .then_some(ElementId::Integer(self.node_id as u64))
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
        if self.observe
            && [frame.0, frame.1, frame.2, frame.3]
                .into_iter()
                .all(f32::is_finite)
        {
            let entity = self.entity.clone();
            let node_id = self.node_id;
            let route = self.route.clone();
            window.on_next_frame(move |_, app| {
                if route.as_ref().is_none_or(|route| route.is_active()) {
                    entity.update(app, |root, _| root.emit_layout_bounds(node_id, frame));
                }
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

pub(crate) fn measure_node(
    node: &StoredNode,
    element: AnyElement,
    entity: &Entity<SolidRoot>,
) -> AnyElement {
    if node.listener_id == 0
        || !matches!(
            node.kind,
            KIND_VIEW | KIND_PRESSABLE | KIND_TEXT | KIND_IMAGE | KIND_ICON | KIND_EXTENSION
        )
    {
        return element;
    }
    MeasuredElement {
        element,
        entity: entity.clone(),
        node_id: node.id,
        namespace: false,
        observe: true,
        route: None,
    }
    .into_any()
}

/// Native styled elements already own their layout box. Supply node identity
/// and measurement without introducing a second box around native flex/grid items.
pub(crate) fn scope_native_element(
    node_id: u32,
    observe: bool,
    element: AnyElement,
    entity: &Entity<SolidRoot>,
    route: super::extensions::ExtensionEventSink,
) -> AnyElement {
    MeasuredElement {
        element,
        entity: entity.clone(),
        node_id,
        namespace: true,
        observe,
        route: Some(route),
    }
    .into_any()
}

pub(crate) fn apply_style_to_extension<E: gpui::Styled>(
    element: E,
    style: Option<&crate::protocol::Style>,
) -> E {
    style::apply_style(element, style)
}

impl SolidRoot {
    pub(super) fn render_node(&self, node: &StoredNode, entity: &Entity<Self>) -> AnyElement {
        let element = self.render_node_content(node, entity);
        match self.style_for_node(node) {
            Some(style) if border::has_edge_colors(style) => {
                border::BorderElement::new(element, style).into_any()
            }
            _ => element,
        }
    }

    fn render_node_content(&self, node: &StoredNode, entity: &Entity<Self>) -> AnyElement {
        let style = self.style_for_node(node);
        if node.kind == KIND_EXTENSION {
            return super::extensions::render(self, node, entity, style);
        }
        if node.kind == KIND_TEXT_INPUT {
            return text_input::render_text_input(self, node, entity, style);
        }
        if node.kind == KIND_IMAGE {
            return image::render(node, entity, style);
        }
        if node.kind == KIND_ICON {
            return icon::render(node, entity, style);
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
            let element = text_input::render_rich_text(self, node, style);
            return measure_node(node, element, entity);
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
            if let Some(text) = node.text_content.as_ref().filter(|text| !text.is_empty()) {
                element = element.child(SharedString::new(Arc::clone(text)));
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
            && (node.kind == KIND_VIEW || node.listener_id != 0)
        {
            let focus = self
                .focus_handles
                .get(&node.id)
                .cloned()
                .expect("focusable node focus handle is reconciled before render");
            element = element.focusable().tab_stop(true).track_focus(&focus);
            if node.listener_id != 0 {
                let runtime = Arc::clone(&self.runtime);
                let sequence = Arc::clone(&self.next_sequence);
                let surface_id = self.store.surface_id();
                let epoch = self.store.epoch();
                let revision = self.store.revision();
                let node_id = node.id;
                let listener_id = node.listener_id;
                let node_kind = node.kind;
                element = element.on_key_down(move |event, window, app| {
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
                            send_event_or_exit(runtime.as_ref(), "text run press event", event);
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
        }
        if node.kind == KIND_PRESSABLE {
            element =
                element.on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default());
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
                send_event_or_exit(runtime.as_ref(), "press event", event);
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
                send_event_or_exit(runtime.as_ref(), "hover event", event);
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
