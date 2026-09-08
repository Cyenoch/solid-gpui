use std::rc::Rc;

use gpui::{
    AnchoredPositionMode, AnyElement, App, Bounds, DispatchPhase, Element, ElementId, Entity,
    GlobalElementId, InspectorElementId, IntoElement, LayoutId, MouseDownEvent, ParentElement,
    Pixels, Window, anchored, deferred, point, px,
};

use crate::protocol::{PositionCode, Style};
use crate::tree::StoredNode;

use super::super::SolidRoot;
use super::RenderedBounds;

struct RenderedBoundsElement {
    element: AnyElement,
    node_id: u32,
    rendered_bounds: RenderedBounds,
    clip: bool,
}

impl IntoElement for RenderedBoundsElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for RenderedBoundsElement {
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
        let bounds = if self.clip {
            bounds.intersect(&window.content_mask().bounds)
        } else {
            bounds
        };
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
            self.rendered_bounds
                .borrow_mut()
                .insert(self.node_id, frame);
        }
        self.element.prepaint(window, cx);
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

struct OverlayElement {
    element: AnyElement,
    entity: Entity<SolidRoot>,
    node_id: u32,
    listener_id: u32,
    anchor_id: Option<u32>,
    rendered_bounds: RenderedBounds,
}

impl IntoElement for OverlayElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for OverlayElement {
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
            self.rendered_bounds
                .borrow_mut()
                .insert(self.node_id, frame);
        }
        self.element.prepaint(window, cx);
    }
    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let overlay_bounds = (
            f32::from(bounds.origin.x),
            f32::from(bounds.origin.y),
            f32::from(bounds.size.width),
            f32::from(bounds.size.height),
        );
        let anchor_id = self.anchor_id;
        let rendered_bounds = Rc::clone(&self.rendered_bounds);
        let entity = self.entity.clone();
        let node_id = self.node_id;
        let listener_id = self.listener_id;
        window.on_mouse_event(move |event: &MouseDownEvent, phase, _window, app| {
            if phase != DispatchPhase::Capture {
                return;
            }
            let inside_overlay = point_in_bounds(event.position, overlay_bounds);
            let inside_anchor = anchor_id
                .and_then(|id| rendered_bounds.borrow().get(&id).copied())
                .is_some_and(|anchor_bounds| point_in_bounds(event.position, anchor_bounds));
            if inside_overlay || inside_anchor {
                return;
            }
            entity.update(app, |root, _| {
                root.emit_pointer_down_outside(
                    node_id,
                    listener_id,
                    f32::from(event.position.x),
                    f32::from(event.position.y),
                );
            });
        });
        self.element.paint(window, cx);
    }
}

fn point_in_bounds(point: gpui::Point<Pixels>, bounds: (f32, f32, f32, f32)) -> bool {
    let x = f32::from(point.x);
    let y = f32::from(point.y);
    x >= bounds.0 && y >= bounds.1 && x <= bounds.0 + bounds.2 && y <= bounds.1 + bounds.3
}

pub(super) fn apply(
    root: &SolidRoot,
    element: AnyElement,
    node: &StoredNode,
    style: Option<&Style>,
    entity: &Entity<SolidRoot>,
) -> AnyElement {
    let captures_anchor_bounds = matches!(
        node.kind,
        crate::tree::KIND_VIEW | crate::tree::KIND_PRESSABLE
    ) && node.children(&root.store).any(|child| {
        child.style.as_ref().and_then(|style| style.position) == Some(PositionCode::Overlay)
    });
    let element =
        if let Some(style) = style.filter(|style| style.position == Some(PositionCode::Overlay)) {
            let element = if node.listener_id != 0 {
                OverlayElement {
                    element,
                    entity: entity.clone(),
                    node_id: node.id,
                    listener_id: node.listener_id,
                    anchor_id: (node.parent_id != 0).then_some(node.parent_id),
                    rendered_bounds: Rc::clone(&root.rendered_bounds),
                }
                .into_any()
            } else {
                element
            };
            deferred(
                anchored()
                    .position_mode(AnchoredPositionMode::Local)
                    .offset(point(
                        px(style.left.unwrap_or(0.0)),
                        px(style.top.unwrap_or(0.0)),
                    ))
                    .child(element),
            )
            .with_priority(1)
            .into_any()
        } else {
            element
        };
    if captures_anchor_bounds || root.popup_anchors.contains(&node.id) {
        RenderedBoundsElement {
            element,
            node_id: node.id,
            rendered_bounds: Rc::clone(&root.rendered_bounds),
            clip: root.popup_anchors.contains(&node.id),
        }
        .into_any()
    } else {
        element
    }
}
