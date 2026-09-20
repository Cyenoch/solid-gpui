// @reference: https://d3js.org/d3-shape/arc

use std::{f32::consts::PI, fmt::Debug};

use gpui::{Bounds, Hsla, Path, PathBuilder, Pixels, Point, Window, point, px};

use crate::plot::{PathCache, ShapeKey};

const EPSILON: f32 = 1e-12;
const HALF_PI: f32 = PI / 2.;

pub struct ArcData<'a, T> {
    pub data: &'a T,
    pub index: usize,
    pub value: f32,
    pub start_angle: f32,
    pub end_angle: f32,
    pub pad_angle: f32,
}

impl<T> Debug for ArcData<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ArcData {{ index: {}, value: {}, start_angle: {}, end_angle: {}, pad_angle: {} }}",
            self.index, self.value, self.start_angle, self.end_angle, self.pad_angle
        )
    }
}

pub struct Arc {
    inner_radius: f32,
    outer_radius: f32,
}

impl Default for Arc {
    fn default() -> Self {
        Self {
            inner_radius: 0.,
            outer_radius: 0.,
        }
    }
}

impl Arc {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the inner radius of the Arc.
    pub fn inner_radius(mut self, inner_radius: f32) -> Self {
        self.inner_radius = inner_radius;
        self
    }

    /// Set the outer radius of the Arc.
    pub fn outer_radius(mut self, outer_radius: f32) -> Self {
        self.outer_radius = outer_radius;
        self
    }

    /// Get the centroid of the Arc.
    pub fn centroid<T>(&self, arc: &ArcData<T>) -> Point<f32> {
        let start_angle = arc.start_angle - HALF_PI;
        let end_angle = arc.end_angle - HALF_PI;
        let r = (self.inner_radius + self.outer_radius) / 2.;
        let a = (start_angle + end_angle) / 2.;

        point(r * a.cos(), r * a.sin())
    }

    fn path<T>(
        &self,
        arc: &ArcData<T>,
        inner_radius: Option<f32>,
        outer_radius: Option<f32>,
        center: Point<Pixels>,
    ) -> Option<Path<Pixels>> {
        let start_angle = arc.start_angle - HALF_PI;
        let end_angle = arc.end_angle - HALF_PI;
        let da = end_angle - start_angle;
        let pad_angle = if da >= PI {
            // Leave some pad angle for full circle.
            // If not, the path start and end will be the same point.
            0.0001
        } else {
            arc.pad_angle
        };
        let r0 = inner_radius.unwrap_or(self.inner_radius).max(0.);
        let r1 = outer_radius.unwrap_or(self.outer_radius).max(0.);

        let center_x = center.x.as_f32();
        let center_y = center.y.as_f32();

        // Angle difference.
        if r1 < EPSILON || da.abs() < EPSILON {
            return None;
        }

        // Handle pad angle.
        let (a0_outer, a1_outer, a0_inner, a1_inner) = if r0 > EPSILON && pad_angle > 0.0 {
            let pad_width = r1 * pad_angle;
            let pad_angle_outer = pad_width / r1;
            let mut pad_angle_inner = pad_width / r0;
            let max_inner_pad = da * 0.8;
            if pad_angle_inner > max_inner_pad {
                pad_angle_inner = max_inner_pad;
            }
            (
                start_angle + pad_angle_outer * 0.5,
                end_angle - pad_angle_outer * 0.5,
                start_angle + pad_angle_inner * 0.5,
                end_angle - pad_angle_inner * 0.5,
            )
        } else {
            let pad = pad_angle * 0.5;
            (
                start_angle + pad,
                end_angle - pad,
                start_angle + pad,
                end_angle - pad,
            )
        };

        let da_outer = a1_outer - a0_outer;
        if da_outer <= 0. {
            return None;
        }

        // Calculate the start and end points of the outer arc.
        let x01 = center_x + r1 * a0_outer.cos();
        let y01 = center_y + r1 * a0_outer.sin();
        let x11 = center_x + r1 * a1_outer.cos();
        let y11 = center_y + r1 * a1_outer.sin();

        let mut builder = PathBuilder::fill();

        // Move to the start point of the outer arc.
        builder.move_to(point(px(x01), px(y01)));

        // Draw the outer arc.
        let large_arc = (a1_outer - a0_outer).abs() > PI;
        builder.arc_to(
            point(px(r1), px(r1)),
            px(0.),
            large_arc,
            true,
            point(px(x11), px(y11)),
        );

        if r0 > EPSILON {
            // End point of the inner arc.
            let x10 = center_x + r0 * a1_inner.cos();
            let y10 = center_y + r0 * a1_inner.sin();
            builder.line_to(point(px(x10), px(y10)));

            // Draw the inner arc.
            let x00 = center_x + r0 * a0_inner.cos();
            let y00 = center_y + r0 * a0_inner.sin();
            let large_arc_inner = (a1_inner - a0_inner).abs() > PI;
            builder.arc_to(
                point(px(r0), px(r0)),
                px(0.),
                large_arc_inner,
                false,
                point(px(x00), px(y00)),
            );
        } else {
            // If there is no inner radius, draw a line to the center.
            builder.line_to(point(px(center_x), px(center_y)));
        }

        builder.build().ok()
    }

    /// The center of `bounds`, which arc paths are built around.
    fn center(bounds: &Bounds<Pixels>) -> Point<Pixels> {
        point(
            px(bounds.origin.x.as_f32() + bounds.size.width.as_f32() / 2.),
            px(bounds.origin.y.as_f32() + bounds.size.height.as_f32() / 2.),
        )
    }

    /// The shape key for an arc slice: the cache domain tag, the angles and
    /// the resolved radii. Everything else the path depends on is derived
    /// from these.
    fn shape_key<T>(
        &self,
        arc: &ArcData<T>,
        inner_radius: Option<f32>,
        outer_radius: Option<f32>,
    ) -> u64 {
        let r0 = inner_radius.unwrap_or(self.inner_radius).max(0.);
        let r1 = outer_radius.unwrap_or(self.outer_radius).max(0.);
        ShapeKey::new((
            "arc/fill",
            arc.start_angle.to_bits(),
            arc.end_angle.to_bits(),
            arc.pad_angle.to_bits(),
            r0.to_bits(),
            r1.to_bits(),
        ))
        .finish()
    }

    /// Whether the cursor at `position` (relative to the bounds origin) is on
    /// this arc's slice: within its angles and between `inner_radius` and
    /// `outer_radius` (this arc's own radii when `None`).
    pub fn contains<T>(
        &self,
        arc: &ArcData<T>,
        position: Point<f32>,
        inner_radius: Option<f32>,
        outer_radius: Option<f32>,
        bounds: &Bounds<Pixels>,
    ) -> bool {
        let dx = position.x - bounds.size.width.as_f32() / 2.;
        let dy = position.y - bounds.size.height.as_f32() / 2.;
        let radius = dx.hypot(dy);
        let r0 = inner_radius.unwrap_or(self.inner_radius).max(0.);
        let r1 = outer_radius.unwrap_or(self.outer_radius).max(0.);
        if radius < r0 || radius > r1 {
            return false;
        }

        // Screen angle -> pie angle (0 at 12 o'clock, clockwise), in [0, TAU).
        let angle = (dy.atan2(dx) + HALF_PI).rem_euclid(2. * PI);
        (arc.start_angle..arc.end_angle).contains(&angle)
    }

    /// Paint the Arc, reusing the path tessellated by an earlier paint while
    /// the angles and radii are unchanged. The path is built around a zero
    /// center and moved to this frame's center, so a pie that moves keeps
    /// every slice cached; changed angles or radii rebuild by key.
    #[allow(clippy::too_many_arguments)]

    pub fn paint_cached<T>(
        &self,
        arc: &ArcData<T>,
        color: impl Into<Hsla>,
        inner_radius: Option<f32>,
        outer_radius: Option<f32>,
        bounds: &Bounds<Pixels>,
        cache: &mut PathCache,
        window: &mut Window,
    ) {
        let center = Self::center(bounds);
        if let Some(path) = cache.get(
            self.shape_key(arc, inner_radius, outer_radius),
            center,
            || self.path(arc, inner_radius, outer_radius, point(px(0.), px(0.))),
        ) {
            window.paint_path(path, color.into());
        }
    }

    /// Paint the Arc.
    pub fn paint<T>(
        &self,
        arc: &ArcData<T>,
        color: impl Into<Hsla>,
        inner_radius: Option<f32>,
        outer_radius: Option<f32>,
        bounds: &Bounds<Pixels>,
        window: &mut Window,
    ) {
        let path = self.path(arc, inner_radius, outer_radius, Self::center(bounds));
        if let Some(path) = path {
            window.paint_path(path, color.into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_default() {
        let arc = Arc::default();
        assert_eq!(arc.inner_radius, 0.);
        assert_eq!(arc.outer_radius, 0.);
    }

    #[test]
    fn test_arc_builder() {
        let arc = Arc::new().inner_radius(10.).outer_radius(20.);

        assert_eq!(arc.inner_radius, 10.);
        assert_eq!(arc.outer_radius, 20.);
    }

    #[test]
    fn test_arc_centroid() {
        let arc = Arc::new().inner_radius(10.).outer_radius(20.);

        let arc_data = ArcData {
            data: &(),
            index: 0,
            value: 1.,
            start_angle: 0.,
            end_angle: PI,
            pad_angle: 0.,
        };

        let centroid = arc.centroid(&arc_data);
        let expected_radius = (10. + 20.) / 2.;
        let expected_angle = (0. + PI - 2. * HALF_PI) / 2.;

        assert_eq!(centroid.x, expected_radius * expected_angle.cos());
        assert_eq!(centroid.y, expected_radius * expected_angle.sin());
    }

    #[test]
    fn cached_shape_key_follows_angles_and_radii() {
        let arc = Arc::new().inner_radius(10.).outer_radius(20.);
        let slice = |start: f32, end: f32| ArcData {
            data: &(),
            index: 0,
            value: 1.,
            start_angle: start,
            end_angle: end,
            pad_angle: 0.,
        };

        let key = arc.shape_key(&slice(0., 1.), None, None);

        // The same slice keeps the key, whoever owns the datum.
        let other = ArcData {
            data: &7,
            index: 3,
            value: 99.,
            start_angle: 0.,
            end_angle: 1.,
            pad_angle: 0.,
        };
        assert_eq!(key, arc.shape_key(&other, None, None));

        // Explicit radii equal to the configured ones resolve to the same
        // shape, so they keep the key too.
        assert_eq!(key, arc.shape_key(&slice(0., 1.), Some(10.), Some(20.)));

        // Changed angles or radii re-tessellate.
        assert_ne!(key, arc.shape_key(&slice(0., 1.1), None, None));
        assert_ne!(key, arc.shape_key(&slice(0.1, 1.), None, None));
        assert_ne!(key, arc.shape_key(&slice(0., 1.), Some(12.), None));
        assert_ne!(key, arc.shape_key(&slice(0., 1.), None, Some(25.)));
    }

    #[test]
    fn cached_arc_builds_at_zero_center_and_translates() {
        let arc = Arc::new().inner_radius(5.).outer_radius(20.);
        let slice = ArcData {
            data: &(),
            index: 0,
            value: 1.,
            start_angle: 0.,
            end_angle: 2.,
            pad_angle: 0.02,
        };

        let local = arc.path(&slice, None, None, point(px(0.), px(0.))).unwrap();
        let center_x = 120.;
        let center_y = 75.;
        let direct = arc
            .path(&slice, None, None, point(px(center_x), px(center_y)))
            .unwrap();

        assert_eq!(local.vertices.len(), direct.vertices.len());
        for (a, b) in local.vertices.iter().zip(&direct.vertices) {
            // Tessellating translated inputs changes f32 operation order.
            assert!((a.xy_position.x.as_f32() + center_x - b.xy_position.x.as_f32()).abs() < 1e-4);
            assert!((a.xy_position.y.as_f32() + center_y - b.xy_position.y.as_f32()).abs() < 1e-4);
            assert_eq!(a.st_position, b.st_position);
        }
        assert_eq!(
            (direct.bounds.origin.x.as_f32() - local.bounds.origin.x.as_f32()),
            center_x
        );
        assert!((local.bounds.size.width - direct.bounds.size.width).abs() < px(1e-4));
        assert!((local.bounds.size.height - direct.bounds.size.height).abs() < px(1e-4));
    }

    #[test]
    fn test_arc_contains() {
        use gpui::{point, px, size};

        // A 100x100 plot: center (50, 50). The right half, 12 to 6 o'clock.
        let arc = Arc::new().inner_radius(10.).outer_radius(40.);
        let right_half = ArcData {
            data: &(),
            index: 0,
            value: 1.,
            start_angle: 0.,
            end_angle: PI,
            pad_angle: 0.,
        };
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(100.), px(100.)));

        // 3 o'clock, between the radii.
        assert!(arc.contains(&right_half, point(80., 50.), None, None, &bounds));
        // 9 o'clock is the other half.
        assert!(!arc.contains(&right_half, point(20., 50.), None, None, &bounds));
        // Inside the hole and past the rim.
        assert!(!arc.contains(&right_half, point(55., 50.), None, None, &bounds));
        assert!(!arc.contains(&right_half, point(95., 50.), None, None, &bounds));
        // A wider outer radius reaches the same point.
        assert!(arc.contains(&right_half, point(95., 50.), None, Some(50.), &bounds));
        // 12 o'clock is the start of this arc, 6 o'clock the start of the next.
        assert!(arc.contains(&right_half, point(50., 20.), None, None, &bounds));
        assert!(!arc.contains(&right_half, point(50., 80.), None, None, &bounds));
    }
}
