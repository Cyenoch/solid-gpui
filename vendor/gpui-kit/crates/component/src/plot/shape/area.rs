// @reference: https://d3js.org/d3-shape/area

use gpui::{Background, Bounds, Path, PathBuilder, Pixels, Point, Window, px};

use crate::plot::{PathCache, ShapeKey, StrokeStyle, origin_point};

#[allow(clippy::type_complexity)]
pub struct Area<T> {
    data: Vec<T>,
    x: Box<dyn Fn(&T) -> Option<f32>>,
    y0: Option<f32>,
    y1: Box<dyn Fn(&T) -> Option<f32>>,
    fill: Background,
    stroke: Background,
    stroke_style: StrokeStyle,
}

impl<T> Default for Area<T> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            x: Box::new(|_| None),
            y0: None,
            y1: Box::new(|_| None),
            fill: Default::default(),
            stroke: Default::default(),
            stroke_style: Default::default(),
        }
    }
}

impl<T> Area<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the data of the Area.
    pub fn data<I>(mut self, data: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        self.data = data.into_iter().collect();
        self
    }

    /// Set the x of the Area.
    pub fn x<F>(mut self, x: F) -> Self
    where
        F: Fn(&T) -> Option<f32> + 'static,
    {
        self.x = Box::new(x);
        self
    }

    /// Set the y0 of the Area.
    pub fn y0(mut self, y0: f32) -> Self {
        self.y0 = Some(y0);
        self
    }

    /// Set the y1 of the Area.
    pub fn y1<F>(mut self, y1: F) -> Self
    where
        F: Fn(&T) -> Option<f32> + 'static,
    {
        self.y1 = Box::new(y1);
        self
    }

    /// Set the fill color of the Area.
    pub fn fill(mut self, fill: impl Into<Background>) -> Self {
        self.fill = fill.into();
        self
    }

    /// Set the stroke color of the Area.
    pub fn stroke(mut self, stroke: impl Into<Background>) -> Self {
        self.stroke = stroke.into();
        self
    }

    /// Set the stroke style of the Area.
    pub fn stroke_style(mut self, stroke_style: StrokeStyle) -> Self {
        self.stroke_style = stroke_style;
        self
    }

    /// The fill and stroke paths for the bounds, built at the bounds origin.
    ///
    /// Crate-visible so cache tests can build an area's real geometry.
    pub(crate) fn path(
        &self,
        bounds: &Bounds<Pixels>,
    ) -> (Option<Path<Pixels>>, Option<Path<Pixels>>) {
        let origin = bounds.origin;
        let mut area_builder = PathBuilder::fill();
        let mut line_builder = PathBuilder::stroke(px(1.));

        let mut points = vec![];

        let mut first_x_tick = None;
        let mut last_x_tick = None;
        for (index, v) in self.data.iter().enumerate() {
            if index == 0 {
                first_x_tick = (self.x)(v);
            }
            if index == self.data.len() - 1 {
                last_x_tick = (self.x)(v);
            }
            let x_tick = (self.x)(v);
            let y_tick = (self.y1)(v);

            if let (Some(x), Some(y)) = (x_tick, y_tick) {
                let pos = origin_point(px(x), px(y), origin);

                points.push(pos);
            }
        }

        if points.is_empty() {
            return (None, None);
        }

        if points.len() == 1 {
            area_builder.move_to(points[0]);
            line_builder.move_to(points[0]);
            return (area_builder.build().ok(), line_builder.build().ok());
        }

        match self.stroke_style {
            StrokeStyle::Natural => {
                area_builder.move_to(points[0]);
                line_builder.move_to(points[0]);
                let n = points.len();
                for i in 0..n - 1 {
                    let p0 = if i == 0 { points[0] } else { points[i - 1] };
                    let p1 = points[i];
                    let p2 = points[i + 1];
                    let p3 = if i + 2 < n {
                        points[i + 2]
                    } else {
                        points[n - 1]
                    };

                    // Catmull-Rom to Bezier
                    let c1 = Point::new(p1.x + (p2.x - p0.x) / 6.0, p1.y + (p2.y - p0.y) / 6.0);
                    let c2 = Point::new(p2.x - (p3.x - p1.x) / 6.0, p2.y - (p3.y - p1.y) / 6.0);

                    area_builder.cubic_bezier_to(p2, c1, c2);
                    line_builder.cubic_bezier_to(p2, c1, c2);
                }
            }
            StrokeStyle::Linear => {
                area_builder.move_to(points[0]);
                line_builder.move_to(points[0]);
                for p in &points[1..] {
                    area_builder.line_to(*p);
                    line_builder.line_to(*p);
                }
            }
            StrokeStyle::StepAfter => {
                area_builder.move_to(points[0]);
                line_builder.move_to(points[0]);
                for (i, p) in points.windows(2).enumerate() {
                    area_builder.line_to(Point::new(p[1].x, p[0].y));
                    line_builder.line_to(Point::new(p[1].x, p[0].y));
                    // Don't draw the vertical line for the last point
                    if i < points.len() - 2 {
                        area_builder.line_to(p[1]);
                        line_builder.line_to(p[1]);
                    }
                }
            }
        }

        // Close path
        if let (Some(first), Some(last), Some(y)) = (first_x_tick, last_x_tick, self.y0) {
            area_builder.line_to(origin_point(px(last), px(y), bounds.origin));
            area_builder.line_to(origin_point(px(first), px(y), bounds.origin));
            area_builder.close();
        }

        (area_builder.build().ok(), line_builder.build().ok())
    }

    /// Paint the Area, reusing the fill and stroke tessellated by an earlier
    /// paint while the projected points, baseline and curve style are
    /// unchanged. `fill` and `line` are the two caches this area keeps
    /// together; see [`Line::paint_cached`](super::Line::paint_cached).
    pub fn paint_cached(
        &self,
        bounds: &Bounds<Pixels>,
        fill: &mut PathCache,
        line: &mut PathCache,
        window: &mut Window,
    ) {
        let local = Bounds::new(Point::default(), bounds.size);
        // One miss builds both paths; the second cache takes the stroke from
        // the stash instead of building again.
        let mut stroke_path = None;
        let fill_path = fill.get(self.shape_key("area/fill"), bounds.origin, || {
            let (area, stroke) = self.path(&local);
            stroke_path = Some(stroke);
            area
        });
        let line_path = line.get(self.shape_key("area/stroke"), bounds.origin, || {
            stroke_path.take().unwrap_or_else(|| self.path(&local).1)
        });
        if let Some(area) = fill_path {
            window.paint_path(area, self.fill);
        }
        if let Some(line) = line_path {
            window.paint_path(line, self.stroke);
        }
    }

    /// The shape key of the area under `purpose` (the fill and the stroke are
    /// cached separately): the curve style, the baseline, the datum count,
    /// and every datum's x and y with their presence — the closing baseline
    /// edge is built from the first and last datum's x even when that datum's
    /// y is null, so a null y must not drop the x from the key. Colors are
    /// not part of it: they don't change the tessellation and are applied per
    /// paint.
    pub(crate) fn shape_key(&self, purpose: &'static str) -> u64 {
        let mut key = ShapeKey::new((
            purpose,
            self.stroke_style,
            self.y0.map(f32::to_bits),
            self.data.len(),
        ));
        for v in self.data.iter() {
            let x = (self.x)(v);
            let y = (self.y1)(v);
            key.bit(x.is_some());
            if let Some(x) = x {
                key.f32(x);
            }
            key.bit(y.is_some());
            if let Some(y) = y {
                key.f32(y);
            }
        }
        key.finish()
    }

    /// Paint the Area.
    pub fn paint(&self, bounds: &Bounds<Pixels>, window: &mut Window) {
        let (area, line) = self.path(bounds);

        if let Some(area) = area {
            window.paint_path(area, self.fill);
        }
        if let Some(line) = line {
            window.paint_path(line, self.stroke);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, size};

    type Nullable = (f32, Option<f32>);

    #[test]
    fn cached_area_reuses_only_while_the_closed_shape_is_unchanged() {
        let area = Area::new()
            .data(vec![1., 2., 3.])
            .x(|v| Some(*v))
            .y0(90.)
            .y1(|v| Some(*v * 2.))
            .stroke_style(StrokeStyle::Linear)
            .fill(gpui::black())
            .stroke(gpui::blue());
        let local = Bounds::new(Point::default(), size(px(100.), px(100.)));
        let key = area.shape_key("area/fill");

        let mut cache = PathCache::default();
        let mut builds = 0;
        for origin in [point(px(0.), px(0.)), point(px(30.), px(12.))] {
            let path = cache
                .get(key, origin, || {
                    builds += 1;
                    area.path(&local).0
                })
                .unwrap();
            // The served fill is the area's own geometry moved to the origin:
            // the closing baseline edge keeps its first-datum corner at
            // (first_x, y0), translated by the origin.
            assert!(path.vertices.iter().any(|v| {
                v.xy_position.x.as_f32() == 1. + origin.x.as_f32()
                    && v.xy_position.y.as_f32() == 90. + origin.y.as_f32()
            }));
        }
        assert_eq!(builds, 1, "unchanged geometry must reuse the cached fill");

        // A different baseline or curve style re-tessellates.
        let rebased = Area::new()
            .data(vec![1., 2., 3.])
            .x(|v| Some(*v))
            .y0(80.)
            .y1(|v| Some(*v * 2.))
            .stroke_style(StrokeStyle::Linear)
            .fill(gpui::black());
        cache
            .get(
                rebased.shape_key("area/fill"),
                point(px(0.), px(0.)),
                || {
                    builds += 1;
                    rebased.path(&local).0
                },
            )
            .unwrap();
        assert_eq!(builds, 2);

        let stepped = Area::new()
            .data(vec![1., 2., 3.])
            .x(|v| Some(*v))
            .y0(90.)
            .y1(|v| Some(*v * 2.))
            .stroke_style(StrokeStyle::StepAfter)
            .fill(gpui::black());
        cache
            .get(
                stepped.shape_key("area/fill"),
                point(px(0.), px(0.)),
                || {
                    builds += 1;
                    stepped.path(&local).0
                },
            )
            .unwrap();
        assert_eq!(builds, 3);

        // Colors don't change the tessellation. The slot currently holds the
        // stepped area, so the original geometry misses once on its way back
        // in; the recolored area is then served by that entry (the color is
        // applied per paint).
        cache
            .get(key, point(px(0.), px(0.)), || {
                builds += 1;
                area.path(&local).0
            })
            .unwrap();
        assert_eq!(
            builds, 4,
            "the original key missed while the stepped area held the slot"
        );
        cache
            .get(key, point(px(0.), px(0.)), || {
                builds += 1;
                area.path(&local).0
            })
            .unwrap();
        assert_eq!(builds, 4, "the second request must be a hit");
        let recolored = Area::new()
            .data(vec![1., 2., 3.])
            .x(|v| Some(*v))
            .y0(90.)
            .y1(|v| Some(*v * 2.))
            .stroke_style(StrokeStyle::Linear)
            .fill(gpui::red())
            .stroke(gpui::green());
        assert_eq!(key, recolored.shape_key("area/fill"));
        let served = cache
            .get(
                recolored.shape_key("area/fill"),
                point(px(0.), px(0.)),
                || {
                    builds += 1;
                    recolored.path(&local).0
                },
            )
            .unwrap();
        assert_eq!(builds, 4, "a color-only change must not rebuild");
        // The served fill is exactly the recolored area's own geometry (same
        // shape, so identical vertices).
        let direct = recolored.path(&local).0.unwrap();
        let same: Vec<(f32, f32)> = direct
            .vertices
            .iter()
            .map(|v| (v.xy_position.x.as_f32(), v.xy_position.y.as_f32()))
            .collect();
        let cached: Vec<(f32, f32)> = served
            .vertices
            .iter()
            .map(|v| (v.xy_position.x.as_f32(), v.xy_position.y.as_f32()))
            .collect();
        assert_eq!(cached, same);
    }

    #[test]
    fn cached_area_key_carries_null_y_endpoints_into_the_baseline_edge() {
        let local = Bounds::new(Point::default(), size(px(100.), px(100.)));
        let build = |first_x: f32| {
            Area::new()
                .data(vec![(first_x, None), (10., Some(10.)), (20., Some(20.))])
                .x(|d: &Nullable| Some(d.0))
                .y0(30.)
                .y1(|d: &Nullable| d.1)
                .stroke_style(StrokeStyle::Linear)
                .fill(gpui::black())
        };

        // The closing baseline edge is built from the first datum's x even
        // though its y is null, so moving that x must change the built
        // geometry...
        let at_zero = build(0.).path(&local).0.unwrap();
        let at_five = build(5.).path(&local).0.unwrap();
        let has_corner = |path: &Path<Pixels>, x: f32| {
            path.vertices
                .iter()
                .any(|v| v.xy_position.x.as_f32() == x && v.xy_position.y.as_f32() == 30.)
        };
        assert!(has_corner(&at_zero, 0.));
        assert!(!has_corner(&at_zero, 5.));
        assert!(has_corner(&at_five, 5.));

        // ...and therefore re-tessellate through the cache instead of being
        // served the stale closing edge.
        let mut cache = PathCache::default();
        let mut builds = 0;
        cache
            .get(
                build(0.).shape_key("area/fill"),
                point(px(0.), px(0.)),
                || {
                    builds += 1;
                    build(0.).path(&local).0
                },
            )
            .unwrap();
        cache
            .get(
                build(5.).shape_key("area/fill"),
                point(px(0.), px(0.)),
                || {
                    builds += 1;
                    build(5.).path(&local).0
                },
            )
            .unwrap();
        assert_eq!(builds, 2, "the shifted null-y endpoint must re-tessellate");

        // Every other null-y-aware input does too: a null y that becomes a
        // value adds a curve point, and an extra null datum changes which
        // datum the baseline edge hangs off.
        let filled = Area::new()
            .data(vec![(0., Some(7.)), (10., Some(10.)), (20., Some(20.))])
            .x(|d: &Nullable| Some(d.0))
            .y0(30.)
            .y1(|d: &Nullable| d.1)
            .stroke_style(StrokeStyle::Linear)
            .fill(gpui::black());
        let extended = Area::new()
            .data(vec![
                (0., None),
                (10., Some(10.)),
                (20., Some(20.)),
                (25., None),
            ])
            .x(|d: &Nullable| Some(d.0))
            .y0(30.)
            .y1(|d: &Nullable| d.1)
            .stroke_style(StrokeStyle::Linear)
            .fill(gpui::black());
        let endpoint = build(0.);
        let keys = [
            endpoint.shape_key("area/fill"),
            filled.shape_key("area/fill"),
            extended.shape_key("area/fill"),
        ];
        assert_ne!(keys[0], keys[1]);
        assert_ne!(keys[0], keys[2]);
        assert_ne!(keys[1], keys[2]);
    }
}
