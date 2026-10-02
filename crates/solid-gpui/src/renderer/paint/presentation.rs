use super::*;
use crate::renderer::PopupInput;
use gpui::{DispatchPhase, KeyDownEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent};

pub(in crate::renderer) fn presentation(
    element: AnyElement,
    input: Option<PopupInput>,
    entity: Entity<SolidRoot>,
) -> AnyElement {
    PresentationElement {
        element,
        input,
        entity,
    }
    .into_any()
}

struct PresentationElement {
    element: AnyElement,
    input: Option<PopupInput>,
    entity: Entity<SolidRoot>,
}

impl IntoElement for PresentationElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for PresentationElement {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.element.request_layout(window, cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        if let Some(focus) = self.entity.read(cx).document_selection_focus() {
            window.set_focus_handle(&focus, cx);
        }
        self.element.prepaint(window, cx);
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let entity = self.entity.downgrade();
        let epoch = self.entity.read(cx).store.epoch();
        let key_entity = self.entity.downgrade();
        window.on_key_event(move |event: &KeyDownEvent, phase, window, cx| {
            if phase != DispatchPhase::Bubble || event.is_held {
                return;
            }
            let modifiers = event.keystroke.modifiers;
            if !modifiers.shift
                && !modifiers.alt
                && (modifiers.platform || modifiers.control)
                && event.keystroke.key.eq_ignore_ascii_case("c")
            {
                let text = key_entity
                    .update(cx, |root, app| {
                        // Native controls keep clipboard precedence. This root-level
                        // route exists for a virtualized selection whose focus owner
                        // has left the mounted tree.
                        let surface_focus = root
                            .document_selection_focus()
                            .is_some_and(|focus| focus.is_focused(window));
                        if root.store.epoch() != epoch
                            || (!surface_focus && window.focused(app).is_some())
                        {
                            return None;
                        }
                        root.text_selection_snapshot()
                            .ok()
                            .map(|snapshot| snapshot.text)
                            .filter(|text| !text.is_empty())
                    })
                    .ok()
                    .flatten();
                if let Some(text) = text {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                    cx.stop_propagation();
                }
            }
        });
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase == DispatchPhase::Bubble && event.dragging() {
                let _ = entity.update(cx, |root, cx| {
                    if root.store.epoch() == epoch {
                        root.drag_document_selection(event.position, cx);
                        if root.autoscroll_document_selection(event.position, cx)
                            && let Some(generation) = root.schedule_document_scroll()
                        {
                            continue_selection_scroll(entity.clone(), epoch, generation, window);
                        }
                    }
                });
            }
        });
        let entity = self.entity.downgrade();
        window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
            if phase == DispatchPhase::Bubble {
                let _ = entity.update(cx, |root, _| {
                    if root.store.epoch() == epoch {
                        root.end_document_selection();
                    }
                });
            }
        });
        if let Some(input) = self.input.clone() {
            let mouse_input = input.clone();
            window.on_mouse_event(move |event: &MouseDownEvent, phase, _, cx| {
                if phase == DispatchPhase::Capture {
                    mouse_input(Some(event.position), cx);
                }
            });
            window.on_key_event(move |event: &KeyDownEvent, phase, _, cx| {
                if phase == DispatchPhase::Bubble && event.keystroke.key == "escape" {
                    input(None, cx);
                }
            });
        }
        self.element.paint(window, cx);
    }
}

fn continue_selection_scroll(
    entity: gpui::WeakEntity<SolidRoot>,
    epoch: u32,
    generation: u32,
    window: &mut Window,
) {
    window.on_next_frame(move |window, app| {
        let repeat = entity
            .update(app, |root, cx| {
                if root.store.epoch() != epoch || !root.document_selection_dragging() {
                    return false;
                }
                root.advance_document_scroll(generation, cx)
                    && root.schedule_document_scroll().is_some()
            })
            .unwrap_or(false);
        if repeat {
            continue_selection_scroll(entity, epoch, generation, window);
        }
    });
}
