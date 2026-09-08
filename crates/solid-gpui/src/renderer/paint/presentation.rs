use super::*;
use crate::renderer::PopupInput;
use gpui::{DispatchPhase, KeyDownEvent, MouseDownEvent};

pub(in crate::renderer) fn presentation(
    element: AnyElement,
    input: Option<PopupInput>,
) -> AnyElement {
    match input {
        Some(input) => PresentationElement { element, input }.into_any(),
        None => element,
    }
}

struct PresentationElement {
    element: AnyElement,
    input: PopupInput,
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
        let input = self.input.clone();
        window.on_mouse_event(move |event: &MouseDownEvent, phase, _, cx| {
            if phase == DispatchPhase::Capture {
                input(Some(event.position), cx);
            }
        });
        let input = self.input.clone();
        window.on_key_event(move |event: &KeyDownEvent, phase, _, cx| {
            if phase == DispatchPhase::Bubble && event.keystroke.key == "escape" {
                input(None, cx);
            }
        });
        self.element.paint(window, cx);
    }
}
