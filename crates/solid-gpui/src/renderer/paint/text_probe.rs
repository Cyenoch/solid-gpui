//! Test-platform paint observation; never part of production construction.
use super::*;

pub(super) struct PaintedText {
    pub(super) element: AnyElement,
    pub(super) node_id: u32,
    pub(super) text: String,
    pub(super) painted: Rc<RefCell<HashMap<u32, String>>>,
}

impl IntoElement for PaintedText {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for PaintedText {
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
        self.element.paint(window, cx);
        self.painted
            .borrow_mut()
            .insert(self.node_id, self.text.clone());
    }
}
