//! Opt-in observations recorded by the production element paint path.
use super::SolidRoot;
use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Window,
};
use serde::Serialize;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PaintedNode {
    pub id: u32,
    pub parent_id: u32,
    pub kind: u32,
    pub listener_id: u32,
    pub label: Option<String>,
    /// Text supplied to this painted native element; custom extension internals
    /// are opaque and are not inferred from their DTOs.
    pub text: Option<String>,
    pub input: Option<PaintedInput>,
    pub scroll_offset: Option<f32>,
    pub selected_text: Option<String>,
    pub bounds: PaintedBounds,
}

#[derive(Clone, Copy, Serialize)]
pub(crate) struct PaintedBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PaintedInput {
    pub value: String,
    pub selection_start: usize,
    pub selection_end: usize,
    pub reversed: bool,
    pub edit_seq: u32,
    pub focused: bool,
    pub marked_start: Option<usize>,
    pub marked_end: Option<usize>,
}

pub(crate) type Observations = Rc<RefCell<BTreeMap<u32, PaintedNode>>>;

impl SolidRoot {
    pub(crate) fn enable_acceptance(&mut self) {
        self.acceptance = Some(Default::default());
    }
    pub(crate) fn acceptance_nodes(&self) -> Vec<PaintedNode> {
        self.acceptance
            .as_ref()
            .map(|nodes| nodes.borrow().values().cloned().collect())
            .unwrap_or_default()
    }
    pub(crate) fn acceptance_has_focus(&self, window: &Window, cx: &App) -> bool {
        window.focused(cx).is_some()
    }
}

pub(super) fn observe(
    root: &SolidRoot,
    node: &crate::StoredNode,
    element: AnyElement,
) -> AnyElement {
    let Some(observations) = root.acceptance.as_ref() else {
        return element;
    };
    let text = match node.kind {
        crate::KIND_TEXT_INPUT => root.input_states.get(&node.id).map(|state| {
            if state.text.is_empty() {
                match node.host_properties.as_ref() {
                    Some(crate::HostProperties::TextInput(input)) => {
                        input.placeholder.clone().unwrap_or_default()
                    }
                    _ => String::new(),
                }
            } else {
                state.text.clone()
            }
        }),
        crate::KIND_TEXT => node.text_content.as_deref().map(str::to_owned),
        crate::KIND_RAW_TEXT => node.text.as_deref().map(str::to_owned),
        _ => None,
    };
    ObservationElement {
        element,
        observations: observations.clone(),
        node: PaintedNode {
            id: node.id,
            parent_id: node.parent_id,
            kind: node.kind,
            listener_id: node.listener_id,
            label: node.accessibility.as_ref().and_then(|a| a.label.clone()),
            text,
            input: root.input_states.get(&node.id).map(|input| PaintedInput {
                value: input.text.clone(),
                selection_start: input.selection.start,
                selection_end: input.selection.end,
                reversed: input.selection_reversed,
                edit_seq: input.edit_seq,
                focused: input.focused,
                marked_start: input.marked.as_ref().map(|range| range.start),
                marked_end: input.marked.as_ref().map(|range| range.end),
            }),
            scroll_offset: root
                .virtual_lists
                .get(&node.id)
                .map(|state| -state.scroll_px_offset_for_scrollbar().y.as_f32()),
            selected_text: root.selected_selectable_text(node.id),
            bounds: PaintedBounds {
                x: 0.,
                y: 0.,
                width: 0.,
                height: 0.,
            },
        },
    }
    .into_any()
}

struct ObservationElement {
    element: AnyElement,
    observations: Observations,
    node: PaintedNode,
}
impl IntoElement for ObservationElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for ObservationElement {
    type RequestLayoutState = ();
    type PrepaintState = Bounds<Pixels>;
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
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Bounds<Pixels> {
        let clipped = bounds.intersect(&window.content_mask().bounds);
        self.element.prepaint(window, cx);
        clipped
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        clipped: &mut Bounds<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.element.paint(window, cx);
        let bounds = PaintedBounds {
            x: clipped.origin.x.into(),
            y: clipped.origin.y.into(),
            width: clipped.size.width.into(),
            height: clipped.size.height.into(),
        };
        if bounds.width > 0. && bounds.height > 0. {
            let mut node = self.node.clone();
            node.bounds = bounds;
            self.observations.borrow_mut().insert(node.id, node);
        }
    }
}
