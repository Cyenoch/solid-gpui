//! Edge-colored borders retain the child's layout box and paint four bounded paths.
use crate::protocol::Style;
use gpui::*;

pub(super) fn has_edge_colors(style: &Style) -> bool {
    [
        style.border_top_color,
        style.border_right_color,
        style.border_bottom_color,
        style.border_left_color,
    ]
    .iter()
    .any(Option::is_some)
}
pub(super) struct BorderElement {
    child: AnyElement,
    widths: [f32; 4],
    radii: [f32; 4],
    colors: [u32; 4],
    opacity: f32,
}
impl BorderElement {
    pub(super) fn new(child: AnyElement, style: &Style) -> Self {
        Self {
            child,
            widths: [
                style.border_top_width,
                style.border_right_width,
                style.border_bottom_width,
                style.border_left_width,
            ]
            .map(|v| v.or(style.border_width).unwrap_or(0.)),
            radii: [
                style.border_top_left_radius,
                style.border_top_right_radius,
                style.border_bottom_right_radius,
                style.border_bottom_left_radius,
            ]
            .map(|v| v.or(style.border_radius).unwrap_or(0.)),
            colors: [
                style.border_top_color,
                style.border_right_color,
                style.border_bottom_color,
                style.border_left_color,
            ]
            .map(|v| v.or(style.border_color_rgba).unwrap_or(0)),
            opacity: style.opacity.unwrap_or(1.),
        }
    }
}
impl IntoElement for BorderElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for BorderElement {
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
        (self.child.request_layout(window, cx), ())
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
        self.child.prepaint(window, cx);
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
        let w = f32::from(bounds.size.width);
        let h = f32::from(bounds.size.height);
        let radii = self.radii.map(|r| r.min(w / 2.).min(h / 2.));
        let [top, right, bottom, left] = self.widths;
        let inner_w = (w - left - right).max(0.);
        let inner_h = (h - top - bottom).max(0.);
        let corners = |inner: bool| -> [(f32, f32, f32, f32); 4] {
            let (x, y, w, h) = if inner {
                (left, top, inner_w, inner_h)
            } else {
                (0., 0., w, h)
            };
            std::array::from_fn(|i| {
                let rx = if inner {
                    (radii[i] - if i == 0 || i == 3 { left } else { right })
                        .max(0.)
                        .min(w / 2.)
                } else {
                    radii[i]
                };
                let ry = if inner {
                    (radii[i] - if i < 2 { top } else { bottom })
                        .max(0.)
                        .min(h / 2.)
                } else {
                    radii[i]
                };
                (
                    x + if i == 0 || i == 3 { rx } else { w - rx },
                    y + if i < 2 { ry } else { h - ry },
                    rx,
                    ry,
                )
            })
        };
        let outer = corners(false);
        let inner = corners(true);
        for side in 0..4 {
            if self.widths[side] == 0. {
                continue;
            }
            let next = (side + 1) % 4;
            let start = 225. + side as f32 * 90.;
            let mut path = PathBuilder::fill();
            let point_at = |c: (f32, f32, f32, f32), degrees: f32| {
                let angle = degrees.to_radians();
                point(
                    bounds.origin.x + px(c.0 + c.2 * angle.cos()),
                    bounds.origin.y + px(c.1 + c.3 * angle.sin()),
                )
            };
            let arc = |path: &mut PathBuilder, c: (f32, f32, f32, f32), angle: f32, sweep: bool| {
                let to = point_at(c, angle);
                if c.2 == 0. || c.3 == 0. {
                    path.line_to(to);
                } else {
                    path.arc_to(point(px(c.2), px(c.3)), px(0.), false, sweep, to);
                }
            };
            path.move_to(point_at(outer[side], start));
            arc(&mut path, outer[side], start + 45., true);
            path.line_to(point_at(outer[next], start + 45.));
            arc(&mut path, outer[next], start + 90., true);
            path.line_to(point_at(inner[next], start + 90.));
            arc(&mut path, inner[next], start + 45., false);
            path.line_to(point_at(inner[side], start + 45.));
            arc(&mut path, inner[side], start, false);
            path.close();
            let mut color: Hsla = rgba(self.colors[side]).into();
            color.a *= self.opacity;
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        }
    }
}
