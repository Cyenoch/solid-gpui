// @reference: https://d3js.org/d3-shape/radial-line

use std::f32::consts::PI;

use gpui::{
    Background, BorderStyle, Bounds, Hsla, PaintQuad, Path, PathBuilder, Pixels, Point, Window,
    point, px, quad, size,
};

use crate::plot::{PathCache, ShapeKey};

const HALF_PI: f32 = PI / 2.;

/// A radial line generator, like `d3.lineRadial`.
///
/// Points are placed around the center of the plot bounds. The `angle`
/// accessor returns the angle in radians, with 0 at 12 o'clock and positive
/// angles proceeding clockwise. The `radius` accessor returns the distance
/// (in pixels) from the center.
///
/// Call [`RadialLine::closed`] to connect the last point back to the first
/// (like `d3.curveLinearClosed`), and [`RadialLine::fill`] to fill the
/// enclosed polygon, e.g. for radar charts.
///
/// Unlike [`Line`](super::Line), the accessors also receive the datum index,
/// matching d3's `(d, i)` accessor form, since radial charts typically derive
/// the angle from the index (e.g. `i * TAU / n`).
#[allow(clippy::type_complexity)]
pub struct RadialLine<T> {
    data: Vec<T>,
    angle: Box<dyn Fn(&T, usize) -> Option<f32>>,
    radius: Box<dyn Fn(&T, usize) -> Option<f32>>,
    closed: bool,
    fill: Option<Background>,
    stroke: Background,
    stroke_width: Pixels,
    dot: bool,
    dot_size: Pixels,
    dot_fill_color: Hsla,
    dot_stroke_color: Option<Hsla>,
}

impl<T> Default for RadialLine<T> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            angle: Box::new(|_, _| None),
            radius: Box::new(|_, _| None),
            closed: false,
            fill: None,
            stroke: Default::default(),
            stroke_width: px(1.),
            dot: false,
            dot_size: px(4.),
            dot_fill_color: gpui::transparent_black(),
            dot_stroke_color: None,
        }
    }
}

impl<T> RadialLine<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the data of the RadialLine.
    pub fn data<I>(mut self, data: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        self.data = data.into_iter().collect();
        self
    }

    /// Set the angle accessor of the RadialLine.
    ///
    /// The accessor is called with the datum and its index, and returns the
    /// angle in radians, with 0 at 12 o'clock and positive angles proceeding
    /// clockwise.
    pub fn angle<F>(mut self, angle: F) -> Self
    where
        F: Fn(&T, usize) -> Option<f32> + 'static,
    {
        self.angle = Box::new(angle);
        self
    }

    /// Set the radius accessor of the RadialLine.
    ///
    /// The accessor is called with the datum and its index, and returns the
    /// distance (in pixels) from the center.
    pub fn radius<F>(mut self, radius: F) -> Self
    where
        F: Fn(&T, usize) -> Option<f32> + 'static,
    {
        self.radius = Box::new(radius);
        self
    }

    /// Connect the last point back to the first, like `d3.curveLinearClosed`.
    pub fn closed(mut self) -> Self {
        self.closed = true;
        self
    }

    /// Set the fill color of the polygon enclosed by the RadialLine.
    ///
    /// The fill path is always closed, regardless of [`RadialLine::closed`].
    pub fn fill(mut self, fill: impl Into<Background>) -> Self {
        self.fill = Some(fill.into());
        self
    }

    /// Set the stroke color of the RadialLine.
    pub fn stroke(mut self, stroke: impl Into<Background>) -> Self {
        self.stroke = stroke.into();
        self
    }

    /// Set the stroke width of the RadialLine.
    pub fn stroke_width(mut self, stroke_width: impl Into<Pixels>) -> Self {
        self.stroke_width = stroke_width.into();
        self
    }

    /// Show dots on the RadialLine.
    pub fn dot(mut self) -> Self {
        self.dot = true;
        self
    }

    /// Set the size of the dots on the RadialLine.
    pub fn dot_size(mut self, dot_size: impl Into<Pixels>) -> Self {
        self.dot_size = dot_size.into();
        self
    }

    /// Set the fill color of the dots on the RadialLine.
    pub fn dot_fill_color(mut self, dot_fill_color: impl Into<Hsla>) -> Self {
        self.dot_fill_color = dot_fill_color.into();
        self
    }

    /// Set the stroke color of the dots on the RadialLine.
    pub fn dot_stroke_color(mut self, dot_stroke_color: impl Into<Hsla>) -> Self {
        self.dot_stroke_color = Some(dot_stroke_color.into());
        self
    }

    /// Paint a dot on the RadialLine.
    fn paint_dot(&self, dot: Point<Pixels>) -> PaintQuad {
        quad(
            gpui::bounds(dot, size(self.dot_size, self.dot_size)),
            self.dot_size / 2.,
            self.dot_fill_color,
            px(1.),
            self.dot_stroke_color.unwrap_or(self.dot_fill_color),
            BorderStyle::default(),
        )
    }

    /// Resolve the data to points around `center`, and the dot quads for
    /// them.
    fn points_at(&self, center: Point<Pixels>) -> (Vec<Point<Pixels>>, Vec<PaintQuad>) {
        let mut paint_dots = vec![];

        let points = self
            .data
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                let angle = (self.angle)(v, i)? - HALF_PI;
                let radius = (self.radius)(v, i)?;

                let p = point(
                    px(center.x.as_f32() + radius * angle.cos()),
                    px(center.y.as_f32() + radius * angle.sin()),
                );

                if self.dot {
                    let dot_radius = self.dot_size / 2.;
                    paint_dots.push(self.paint_dot(point(p.x - dot_radius, p.y - dot_radius)));
                }

                Some(p)
            })
            .collect();

        (points, paint_dots)
    }

    #[cfg(test)]
    /// Resolve the data to points around the center of the bounds.
    fn points(&self, bounds: &Bounds<Pixels>) -> Vec<Point<Pixels>> {
        let center_x = bounds.origin.x.as_f32() + bounds.size.width.as_f32() / 2.;
        let center_y = bounds.origin.y.as_f32() + bounds.size.height.as_f32() / 2.;

        self.points_at(point(px(center_x), px(center_y))).0
    }

    /// The fill and stroke paths through `points`. The fill is built whenever
    /// [`Self::fill`] is set, and is always closed.
    fn paths(&self, points: &[Point<Pixels>]) -> (Option<Path<Pixels>>, Option<Path<Pixels>>) {
        if points.is_empty() {
            return (None, None);
        }

        let fill_path = self.fill.and_then(|_| {
            if points.len() < 3 {
                return None;
            }

            let mut builder = PathBuilder::fill();
            builder.add_polygon(points, true);
            builder.build().ok()
        });

        let mut builder = PathBuilder::stroke(self.stroke_width);
        builder.move_to(points[0]);
        for p in &points[1..] {
            builder.line_to(*p);
        }
        if self.closed && points.len() > 2 {
            builder.close();
        }

        (fill_path, builder.build().ok())
    }

    /// The fill and stroke paths and the dot quads for the bounds.
    fn path(
        &self,
        bounds: &Bounds<Pixels>,
    ) -> (Option<Path<Pixels>>, Option<Path<Pixels>>, Vec<PaintQuad>) {
        let center_x = bounds.origin.x.as_f32() + bounds.size.width.as_f32() / 2.;
        let center_y = bounds.origin.y.as_f32() + bounds.size.height.as_f32() / 2.;
        let (points, dots) = self.points_at(point(px(center_x), px(center_y)));
        let (fill_path, stroke_path) = self.paths(&points);

        (fill_path, stroke_path, dots)
    }

    /// The stroke/fill shape key: the cache purpose (the fill and the stroke
    /// are separate keys), the closed flag, whether a fill is built at all,
    /// the stroke width, and every resolved point (center-relative). The
    /// center itself moves the shape, but does not change it, so a resized
    /// plot keeps the key.
    pub(crate) fn shape_key(&self, purpose: &'static str, points: &[Point<Pixels>]) -> u64 {
        let mut key = ShapeKey::new((
            purpose,
            self.closed,
            self.fill.is_some(),
            self.stroke_width.as_f32().to_bits(),
        ));
        for p in points {
            key.point(*p);
        }
        key.finish()
    }

    /// Paint the RadialLine, reusing the fill and stroke tessellated by an
    /// earlier paint while the resolved points, stroke width and closed flag
    /// are unchanged. The paths are built around a zero center and moved to
    /// this frame's center; the dots are quads at this frame's center.
    ///
    /// `fill` and `stroke` are the two caches this line keeps together, as
    /// [`Area::paint_cached`](super::Area::paint_cached) does.
    pub fn paint_cached(
        &self,
        bounds: &Bounds<Pixels>,
        fill: &mut PathCache,
        stroke: &mut PathCache,
        window: &mut Window,
    ) {
        let (points, dots) = self.points_at(Point::default());
        let center = point(
            px(bounds.origin.x.as_f32() + bounds.size.width.as_f32() / 2.),
            px(bounds.origin.y.as_f32() + bounds.size.height.as_f32() / 2.),
        );

        // One miss builds both paths; the stroke cache takes the stroke from
        // the stash instead of building again.
        let mut stroke_path = None;
        let fill_path = fill.get(self.shape_key("radial/fill", &points), center, || {
            let (fill, stroke) = self.paths(&points);
            stroke_path = Some(stroke);
            fill
        });
        let stroke_path = stroke.get(self.shape_key("radial/stroke", &points), center, || {
            stroke_path.take().unwrap_or_else(|| self.paths(&points).1)
        });

        if let (Some(path), Some(fill)) = (fill_path, self.fill) {
            window.paint_path(path, fill);
        }
        if let Some(path) = stroke_path {
            window.paint_path(path, self.stroke);
        }
        for mut dot in dots {
            dot.bounds.origin = dot.bounds.origin + center;
            window.paint_quad(dot);
        }
    }

    /// Paint the RadialLine.
    pub fn paint(&self, bounds: &Bounds<Pixels>, window: &mut Window) {
        let (fill_path, stroke_path, dots) = self.path(bounds);

        if let (Some(path), Some(fill)) = (fill_path, self.fill) {
            window.paint_path(path, fill);
        }
        if let Some(path) = stroke_path {
            window.paint_path(path, self.stroke);
        }
        for dot in dots {
            window.paint_quad(dot);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use super::*;

    use gpui::{Bounds, point, px};

    #[test]
    fn test_radial_line_points() {
        let data = vec![1., 1., 1., 1.];
        let line = RadialLine::new()
            .data(data)
            .angle(|_, i| Some(i as f32 * TAU / 4.))
            .radius(|v, _| Some(*v * 10.));

        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(100.), px(100.)));
        let points = line.points(&bounds);

        // 4 points around the center (50, 50), at 12, 3, 6 and 9 o'clock.
        assert_eq!(points.len(), 4);
        let expected = [(50., 40.), (60., 50.), (50., 60.), (40., 50.)];
        for (p, (x, y)) in points.iter().zip(expected) {
            assert!((p.x.as_f32() - x).abs() < 1e-4);
            assert!((p.y.as_f32() - y).abs() < 1e-4);
        }
    }

    #[test]
    fn test_radial_line_path() {
        let data = vec![1., 2., 3.];
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(100.), px(100.)));

        let line = RadialLine::new()
            .data(data.clone())
            .angle(|_, i| Some(i as f32 * TAU / 3.))
            .radius(|v, _| Some(*v * 10.));

        let (fill_path, stroke_path, dots) = line.path(&bounds);
        assert!(fill_path.is_none());
        assert!(stroke_path.is_some());
        assert!(dots.is_empty());

        let line = RadialLine::new()
            .data(data)
            .angle(|_, i| Some(i as f32 * TAU / 3.))
            .radius(|v, _| Some(*v * 10.))
            .closed()
            .fill(gpui::black())
            .dot();

        let (fill_path, stroke_path, dots) = line.path(&bounds);
        assert!(fill_path.is_some());
        assert!(stroke_path.is_some());
        assert_eq!(dots.len(), 3);
    }

    #[test]
    fn cached_radial_reuses_fill_and_stroke_across_centers() {
        let line = RadialLine::new()
            .data(vec![1., 2., 3.])
            .angle(|_, i| Some(i as f32 * TAU / 3.))
            .radius(|v, _| Some(*v * 10.))
            .fill(gpui::black());
        let (points, _) = line.points_at(point(px(0.), px(0.)));

        let mut cache = PathCache::default();
        let mut builds = 0;
        let key = line.shape_key("radial/fill", &points);
        for center in [point(px(50.), px(40.)), point(px(120.), px(75.))] {
            let served = cache
                .get(key, center, || {
                    builds += 1;
                    line.paths(&points).0
                })
                .unwrap();
            // The served fill is the radial polygon moved to the center.
            let fresh = line.paths(&points).0.unwrap();
            for (s, d) in served.vertices.iter().zip(fresh.vertices.iter()) {
                assert_eq!(
                    s.xy_position.x.as_f32(),
                    d.xy_position.x.as_f32() + center.x.as_f32()
                );
                assert_eq!(
                    s.xy_position.y.as_f32(),
                    d.xy_position.y.as_f32() + center.y.as_f32()
                );
            }
        }
        assert_eq!(builds, 1, "unchanged geometry must reuse the cached fill");

        let stroke_key = line.shape_key("radial/stroke", &points);
        for center in [point(px(50.), px(40.)), point(px(120.), px(75.))] {
            let served = cache
                .get(stroke_key, center, || {
                    builds += 1;
                    line.paths(&points).1
                })
                .unwrap();
            let fresh = line.paths(&points).1.unwrap();
            for (s, d) in served.vertices.iter().zip(fresh.vertices.iter()) {
                assert_eq!(
                    s.xy_position.x.as_f32(),
                    d.xy_position.x.as_f32() + center.x.as_f32()
                );
                assert_eq!(
                    s.xy_position.y.as_f32(),
                    d.xy_position.y.as_f32() + center.y.as_f32()
                );
            }
        }
        assert_eq!(builds, 2, "unchanged geometry must reuse the cached stroke");
    }

    #[test]
    fn cached_radial_fill_presence_toggles_rebuild() {
        // The reported bug: a slot warmed by a fill-less line (which builds
        // no fill) was served to a fill-bearing line with the same geometry,
        // so the fill was never painted. The fill's presence is part of the
        // key, so each direction must re-tessellate when it changes.
        let fillless = RadialLine::new()
            .data(vec![1., 2., 3.])
            .angle(|_, i| Some(i as f32 * TAU / 3.))
            .radius(|v, _| Some(*v * 10.));
        let filled = RadialLine::new()
            .data(vec![1., 2., 3.])
            .angle(|_, i| Some(i as f32 * TAU / 3.))
            .radius(|v, _| Some(*v * 10.))
            .fill(gpui::black());
        let (points, _) = fillless.points_at(point(px(0.), px(0.)));

        let mut cache = PathCache::default();
        let mut builds = 0;
        let none = cache.get(
            fillless.shape_key("radial/fill", &points),
            point(px(0.), px(0.)),
            || {
                builds += 1;
                fillless.paths(&points).0
            },
        );
        assert_eq!(builds, 1);
        assert!(none.is_none(), "the fill-less line builds no fill");

        let served = cache
            .get(
                filled.shape_key("radial/fill", &points),
                point(px(0.), px(0.)),
                || {
                    builds += 1;
                    filled.paths(&points).0
                },
            )
            .unwrap();
        assert_eq!(
            builds, 2,
            "None -> Some(fill) must re-tessellate, not serve the cached None"
        );
        let fresh = filled.paths(&points).0.unwrap();
        let got: Vec<(f32, f32)> = served
            .vertices
            .iter()
            .map(|v| (v.xy_position.x.as_f32(), v.xy_position.y.as_f32()))
            .collect();
        let want: Vec<(f32, f32)> = fresh
            .vertices
            .iter()
            .map(|v| (v.xy_position.x.as_f32(), v.xy_position.y.as_f32()))
            .collect();
        assert_eq!(got, want);

        // And back: the fill-less shape re-tessellates once the fill-bearing
        // one replaced the slot, and again serves no fill.
        let none_again = cache.get(
            fillless.shape_key("radial/fill", &points),
            point(px(0.), px(0.)),
            || {
                builds += 1;
                fillless.paths(&points).0
            },
        );
        assert_eq!(builds, 3);
        assert!(none_again.is_none());
    }

    #[test]
    fn cached_radial_invalidation_on_radius_width_and_closed() {
        let base = RadialLine::new()
            .data(vec![1., 2., 3.])
            .angle(|_, i| Some(i as f32 * TAU / 3.))
            .radius(|v, _| Some(*v * 10.))
            .fill(gpui::black());
        let (points, _) = base.points_at(point(px(0.), px(0.)));
        let original = base.paths(&points).0.unwrap();
        let stream = |path: &gpui::Path<Pixels>| -> Vec<(f32, f32)> {
            path.vertices
                .iter()
                .map(|v| (v.xy_position.x.as_f32(), v.xy_position.y.as_f32()))
                .collect()
        };

        // Each changed shape is stable in its own right (builds once, then
        // reuses) and its geometry actually differs from the original's.
        let scaled = RadialLine::new()
            .data(vec![1., 2., 3.])
            .angle(|_, i| Some(i as f32 * TAU / 3.))
            .radius(|v, _| Some(*v * 20.))
            .fill(gpui::black());
        let (scaled_points, _) = scaled.points_at(point(px(0.), px(0.)));
        let mut cache = PathCache::default();
        let mut builds = 0;
        for _ in 0..2 {
            cache
                .get(
                    scaled.shape_key("radial/fill", &scaled_points),
                    point(px(0.), px(0.)),
                    || {
                        builds += 1;
                        scaled.paths(&scaled_points).0
                    },
                )
                .unwrap();
        }
        assert_eq!(builds, 1);
        assert_ne!(
            stream(&original),
            stream(&scaled.paths(&scaled_points).0.unwrap())
        );

        let wide = RadialLine::new()
            .data(vec![1., 2., 3.])
            .angle(|_, i| Some(i as f32 * TAU / 3.))
            .radius(|v, _| Some(*v * 10.))
            .stroke_width(2.);
        let mut cache = PathCache::default();
        let mut builds = 0;
        for _ in 0..2 {
            cache
                .get(
                    wide.shape_key("radial/stroke", &points),
                    point(px(0.), px(0.)),
                    || {
                        builds += 1;
                        wide.paths(&points).1
                    },
                )
                .unwrap();
        }
        assert_eq!(builds, 1, "the wider stroke builds once and then reuses");

        let closed = RadialLine::new()
            .data(vec![1., 2., 3.])
            .angle(|_, i| Some(i as f32 * TAU / 3.))
            .radius(|v, _| Some(*v * 10.))
            .closed();
        let mut cache = PathCache::default();
        let mut builds = 0;
        for _ in 0..2 {
            cache
                .get(
                    closed.shape_key("radial/stroke", &points),
                    point(px(0.), px(0.)),
                    || {
                        builds += 1;
                        closed.paths(&points).1
                    },
                )
                .unwrap();
        }
        assert_eq!(builds, 1, "the closed stroke builds once and then reuses");
    }

    #[test]
    fn cached_radial_geometry_at_any_center_is_the_zero_center_translation() {
        let line = RadialLine::new()
            .data(vec![1., 2., 3., 4.])
            .angle(|_, i| Some(i as f32 * TAU / 4.))
            .radius(|v, _| Some(*v * 10.))
            .closed()
            .fill(gpui::black());

        // Resolving around another center preserves the same shape, within
        // f32 rounding caused by the order of translation and tessellation.
        let local = line.points_at(point(px(0.), px(0.))).0;
        let at_small = line.points_at(point(px(50.), px(40.))).0;
        let at_large = line.points_at(point(px(120.), px(75.))).0;
        for (small, large) in at_small.iter().zip(&at_large) {
            let expected = point(
                small.x + (px(120.) - px(50.)),
                small.y + (px(75.) - px(40.)),
            );
            // Composing offsets changes floating-point operation order.
            assert!((large.x - expected.x).abs() < px(1e-3));
            assert!((large.y - expected.y).abs() < px(1e-3));
        }

        // Compare every translated vertex, not merely key equality.
        let (center_x, center_y) = (120., 75.);
        let (fill_zero, stroke_zero) = line.paths(&local);
        let (fill_direct, stroke_direct) = line.paths(&at_large);

        let moved_vertices = |path: &gpui::Path<Pixels>| -> Vec<(f32, f32)> {
            path.vertices
                .iter()
                .map(|v| {
                    (
                        v.xy_position.x.as_f32() + center_x,
                        v.xy_position.y.as_f32() + center_y,
                    )
                })
                .collect()
        };
        let placed_vertices = |path: &gpui::Path<Pixels>| -> Vec<(f32, f32)> {
            path.vertices
                .iter()
                .map(|v| (v.xy_position.x.as_f32(), v.xy_position.y.as_f32()))
                .collect()
        };

        for (zero, direct) in [
            (fill_zero.as_ref(), fill_direct.as_ref()),
            (stroke_zero.as_ref(), stroke_direct.as_ref()),
        ] {
            let zero_path = zero.expect("the fixture must build local geometry");
            let direct_path = direct.expect("the fixture must build translated geometry");
            let expected = moved_vertices(zero_path);
            let actual = placed_vertices(direct_path);
            assert_eq!(actual.len(), expected.len());
            for ((x, y), (expected_x, expected_y)) in actual.into_iter().zip(expected) {
                assert!((x - expected_x).abs() < 1e-4);
                assert!((y - expected_y).abs() < 1e-4);
            }
        }
    }
}
