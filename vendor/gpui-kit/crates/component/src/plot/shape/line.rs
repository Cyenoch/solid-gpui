// @reference: https://d3js.org/d3-shape/line

use gpui::{
    Background, BorderStyle, Bounds, Hsla, PaintQuad, Path, PathBuilder, Pixels, Point, Window, px,
    quad, size,
};

use crate::plot::{PathCache, ShapeKey, StrokeStyle, origin_point};

#[allow(clippy::type_complexity)]
pub struct Line<T> {
    data: Vec<T>,
    x: Box<dyn Fn(&T) -> Option<f32>>,
    y: Box<dyn Fn(&T) -> Option<f32>>,
    stroke: Background,
    stroke_width: Pixels,
    stroke_style: StrokeStyle,
    dot: bool,
    dot_size: Pixels,
    dot_fill_color: Hsla,
    dot_stroke_color: Option<Hsla>,
}

impl<T> Default for Line<T> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            x: Box::new(|_| None),
            y: Box::new(|_| None),
            stroke: Default::default(),
            stroke_width: px(1.),
            stroke_style: Default::default(),
            dot: false,
            dot_size: px(4.),
            dot_fill_color: gpui::transparent_black(),
            dot_stroke_color: None,
        }
    }
}

impl<T> Line<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the data of the Line.
    pub fn data<I>(mut self, data: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        self.data = data.into_iter().collect();
        self
    }

    /// Set the x of the Line.
    pub fn x<F>(mut self, x: F) -> Self
    where
        F: Fn(&T) -> Option<f32> + 'static,
    {
        self.x = Box::new(x);
        self
    }

    /// Set the y of the Line.
    pub fn y<F>(mut self, y: F) -> Self
    where
        F: Fn(&T) -> Option<f32> + 'static,
    {
        self.y = Box::new(y);
        self
    }

    /// Set the stroke color of the Line.
    pub fn stroke(mut self, stroke: impl Into<Background>) -> Self {
        self.stroke = stroke.into();
        self
    }

    /// Set the stroke width of the Line.
    pub fn stroke_width(mut self, stroke_width: impl Into<Pixels>) -> Self {
        self.stroke_width = stroke_width.into();
        self
    }

    /// Set the stroke style of the Line.
    pub fn stroke_style(mut self, stroke_style: StrokeStyle) -> Self {
        self.stroke_style = stroke_style;
        self
    }

    /// Show dots on the Line.
    pub fn dot(mut self) -> Self {
        self.dot = true;
        self
    }

    /// Set the size of the dots on the Line.
    pub fn dot_size(mut self, dot_size: impl Into<Pixels>) -> Self {
        self.dot_size = dot_size.into();
        self
    }

    /// Set the fill color of the dots on the Line.
    pub fn dot_fill_color(mut self, dot_fill_color: impl Into<Hsla>) -> Self {
        self.dot_fill_color = dot_fill_color.into();
        self
    }

    /// Set the stroke color of the dots on the Line.
    pub fn dot_stroke_color(mut self, dot_stroke_color: impl Into<Hsla>) -> Self {
        self.dot_stroke_color = Some(dot_stroke_color.into());
        self
    }

    /// Paint the dots on the Line.
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

    /// The projected points relative to `origin`, and the dot quads to paint.
    fn dots(&self, origin: Point<Pixels>) -> (Vec<Point<Pixels>>, Vec<PaintQuad>) {
        let mut dots = vec![];
        let mut paint_dots = vec![];

        for v in self.data.iter() {
            let x_tick = (self.x)(v);
            let y_tick = (self.y)(v);

            if let (Some(x), Some(y)) = (x_tick, y_tick) {
                let pos = origin_point(px(x), px(y), origin);

                if self.dot {
                    let dot_radius = self.dot_size.as_f32() / 2.;
                    let dot_pos = origin_point(px(x - dot_radius), px(y - dot_radius), origin);
                    paint_dots.push(self.paint_dot(dot_pos));
                }

                dots.push(pos);
            }
        }

        (dots, paint_dots)
    }

    /// The stroke through `dots`.
    fn build_path(&self, dots: &[Point<Pixels>]) -> Option<Path<Pixels>> {
        let mut builder = PathBuilder::stroke(self.stroke_width);

        if dots.is_empty() {
            return None;
        }

        if dots.len() == 1 {
            builder.move_to(dots[0]);
            return builder.build().ok();
        }

        match self.stroke_style {
            StrokeStyle::Natural => {
                builder.move_to(dots[0]);
                let n = dots.len();
                for i in 0..n - 1 {
                    let p0 = if i == 0 { dots[0] } else { dots[i - 1] };
                    let p1 = dots[i];
                    let p2 = dots[i + 1];
                    let p3 = if i + 2 < n { dots[i + 2] } else { dots[n - 1] };

                    // Catmull-Rom to Bezier
                    let c1 = Point::new(p1.x + (p2.x - p0.x) / 6.0, p1.y + (p2.y - p0.y) / 6.0);
                    let c2 = Point::new(p2.x - (p3.x - p1.x) / 6.0, p2.y - (p3.y - p1.y) / 6.0);

                    builder.cubic_bezier_to(p2, c1, c2);
                }
            }
            StrokeStyle::Linear => {
                builder.move_to(dots[0]);
                for p in &dots[1..] {
                    builder.line_to(*p);
                }
            }
            StrokeStyle::StepAfter => {
                builder.move_to(dots[0]);
                for (i, p) in dots.windows(2).enumerate() {
                    builder.line_to(Point::new(p[1].x, p[0].y));
                    // Don't draw the vertical line for the last point
                    if i < dots.len() - 2 {
                        builder.line_to(p[1]);
                    }
                }
            }
        }

        builder.build().ok()
    }

    fn path(&self, bounds: &Bounds<Pixels>) -> (Option<Path<Pixels>>, Vec<PaintQuad>) {
        let (dots, paint_dots) = self.dots(bounds.origin);
        (self.build_path(&dots), paint_dots)
    }

    /// Paint the Line, reusing the stroke tessellated by an earlier paint
    /// while the projected points, stroke width and curve style are unchanged.
    ///
    /// Use this from a [`Plot`](crate::plot::Plot) that keeps a
    /// [`PathCache`] per line: the plot repaints on every frame it is on
    /// screen, and tessellating the stroke is most of what a line costs.
    pub fn paint_cached(
        &self,
        bounds: &Bounds<Pixels>,
        cache: &mut PathCache,
        window: &mut Window,
    ) {
        let (dots, paint_dots) = self.dots(Point::default());
        let key = self.shape_key(&dots);
        if let Some(path) = cache.get(key, bounds.origin, || self.build_path(&dots)) {
            window.paint_path(path, self.stroke);
        }
        // Dots are quads: cheap, and positioned at this frame's origin.
        for dot in paint_dots {
            let mut dot = dot;
            dot.bounds.origin = dot.bounds.origin + bounds.origin;
            window.paint_quad(dot);
        }
    }

    /// The stroke shape key for `dots` (origin-relative): the cache domain
    /// tag, the curve style, the stroke width, and every point. Colors and
    /// dots are not part of it: they don't change the tessellation, and are
    /// applied per paint.
    pub(crate) fn shape_key(&self, dots: &[Point<Pixels>]) -> u64 {
        let mut key = ShapeKey::new((
            "line/stroke",
            self.stroke_style,
            self.stroke_width.as_f32().to_bits(),
        ));
        for dot in dots {
            key.point(*dot);
        }
        key.finish()
    }

    /// Paint the Line.
    pub fn paint(&self, bounds: &Bounds<Pixels>, window: &mut Window) {
        let (path, dots) = self.path(bounds);
        if let Some(path) = path {
            window.paint_path(path, self.stroke);
        }
        for dot in dots {
            window.paint_quad(dot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::plot::shape::Area;
    use gpui::{Bounds, point, px, size};

    #[test]
    fn test_line_path() {
        let data = vec![1., 2., 3.];
        let line = Line::new()
            .data(data.clone())
            .x(|v| Some(*v))
            .y(|v| Some(*v * 2.));

        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(100.), px(100.)));
        let (path, dots) = line.path(&bounds);

        assert!(path.is_some());
        assert!(dots.is_empty());

        let line_with_dots = Line::new()
            .data(data)
            .x(|v| Some(*v))
            .y(|v| Some(*v * 2.))
            .dot();

        let (_, dots) = line_with_dots.path(&bounds);
        assert_eq!(dots.len(), 3);
    }

    #[test]
    fn cached_line_reuses_only_while_the_stroke_shape_is_unchanged() {
        let line = Line::new()
            .data(vec![1., 2., 3.])
            .x(|v| Some(*v))
            .y(|v| Some(*v * 2.))
            .stroke_style(StrokeStyle::Linear)
            .stroke_width(2.);
        let (dots, _) = line.dots(point(px(0.), px(0.)));
        let key = line.shape_key(&dots);

        let mut cache = PathCache::default();
        let mut builds = 0;
        for origin in [point(px(0.), px(0.)), point(px(40.), px(25.))] {
            let path = cache
                .get(key, origin, || {
                    builds += 1;
                    line.build_path(&dots)
                })
                .unwrap();
            // The served stroke is the line's own geometry moved to this
            // origin: every vertex is exactly a fresh build's vertex plus the
            // origin.
            let fresh = line.build_path(&dots).unwrap();
            for (served, direct) in path.vertices.iter().zip(fresh.vertices.iter()) {
                assert_eq!(
                    served.xy_position.x.as_f32(),
                    direct.xy_position.x.as_f32() + origin.x.as_f32()
                );
                assert_eq!(
                    served.xy_position.y.as_f32(),
                    direct.xy_position.y.as_f32() + origin.y.as_f32()
                );
                assert_eq!(served.st_position, direct.st_position);
            }
        }
        assert_eq!(builds, 1, "unchanged geometry must reuse the cached stroke");

        // Changed data re-tessellates and produces different geometry.
        let changed = Line::new()
            .data(vec![1., 2., 4.])
            .x(|v| Some(*v))
            .y(|v| Some(*v * 2.))
            .stroke_style(StrokeStyle::Linear)
            .stroke_width(2.);
        let (changed_dots, _) = changed.dots(point(px(0.), px(0.)));
        let changed_stroke = changed.build_path(&changed_dots).unwrap();
        let original_stroke = line.build_path(&dots).unwrap();
        let stream = |path: &gpui::Path<Pixels>| -> Vec<(f32, f32)> {
            path.vertices
                .iter()
                .map(|v| (v.xy_position.x.as_f32(), v.xy_position.y.as_f32()))
                .collect()
        };
        assert_ne!(
            stream(&original_stroke),
            stream(&changed_stroke),
            "the changed data must produce a different stroke"
        );
        cache
            .get(
                changed.shape_key(&changed_dots),
                point(px(0.), px(0.)),
                || {
                    builds += 1;
                    changed.build_path(&changed_dots)
                },
            )
            .unwrap();
        assert_eq!(builds, 2);

        // A different curve style re-tessellates...
        let natural = Line::new()
            .data(vec![1., 2., 3.])
            .x(|v| Some(*v))
            .y(|v| Some(*v * 2.))
            .stroke_style(StrokeStyle::Natural)
            .stroke_width(2.);
        cache
            .get(natural.shape_key(&dots), point(px(0.), px(0.)), || {
                builds += 1;
                natural.build_path(&dots)
            })
            .unwrap();
        assert_eq!(builds, 3);

        // ...as does a different stroke width.
        let thick = Line::new()
            .data(vec![1., 2., 3.])
            .x(|v| Some(*v))
            .y(|v| Some(*v * 2.))
            .stroke_style(StrokeStyle::Linear)
            .stroke_width(3.);
        cache
            .get(thick.shape_key(&dots), point(px(0.), px(0.)), || {
                builds += 1;
                thick.build_path(&dots)
            })
            .unwrap();
        assert_eq!(builds, 4);

        // Colors and dots don't change the tessellation. The slot currently
        // holds the thick stroke, so the original geometry misses once on its
        // way back in, then the recolored, dotted line is served by that
        // entry (colors and dots are applied per paint).
        cache
            .get(key, point(px(0.), px(0.)), || {
                builds += 1;
                line.build_path(&dots)
            })
            .unwrap();
        assert_eq!(
            builds, 5,
            "the original key missed while the thick stroke held the slot"
        );
        cache
            .get(key, point(px(0.), px(0.)), || {
                builds += 1;
                line.build_path(&dots)
            })
            .unwrap();
        assert_eq!(builds, 5, "the second request must be a hit");
        let recolored = Line::new()
            .data(vec![1., 2., 3.])
            .x(|v| Some(*v))
            .y(|v| Some(*v * 2.))
            .stroke_style(StrokeStyle::Linear)
            .stroke_width(2.)
            .stroke(gpui::red())
            .dot()
            .dot_size(8.);
        assert_eq!(key, recolored.shape_key(&dots));
        let served = cache
            .get(recolored.shape_key(&dots), point(px(0.), px(0.)), || {
                builds += 1;
                recolored.build_path(&dots)
            })
            .unwrap();
        assert_eq!(builds, 5, "a color-only change must not rebuild");
        // The served stroke is exactly the recolored line's own geometry at
        // this origin.
        let fresh = recolored.build_path(&dots).unwrap();
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

        // Moving the chart is origin movement: the key is computed from
        // origin-relative points, so resolving the same data at two origins
        // yields points that differ only by the origin offset.
        let (at_origin, _) = line.dots(point(px(100.), px(50.)));
        let (elsewhere, _) = line.dots(point(px(320.), px(90.)));
        for (b, a) in elsewhere.iter().zip(&at_origin) {
            assert_eq!(
                *b,
                *a + (point(px(320.), px(90.)) - point(px(100.), px(50.)))
            );
        }
    }

    #[test]
    fn a_shared_slot_warmed_by_another_shape_kind_serves_only_its_own_key() {
        // NativePlot hands slot 2i to whichever path shape sits at index i,
        // so a slot can be warm from an Area fill and then be asked for a
        // Line stroke. Verify that retaining the slot never retains the wrong
        // primitive's geometry when purpose and data change together.
        let area = Area::new()
            .data(vec![(0., 10.), (20., 20.)])
            .x(|d: &(f32, f32)| Some(d.0))
            .y0(30.)
            .y1(|d: &(f32, f32)| Some(d.1))
            .stroke_style(StrokeStyle::Linear)
            .fill(gpui::black());
        let line = Line::new()
            .data(vec![(0., 30.), (0., 10.), (20., 20.)])
            .x(|d: &(f32, f32)| Some(d.0))
            .y(|d: &(f32, f32)| Some(d.1))
            .stroke_style(StrokeStyle::Linear)
            .stroke_width(2.);
        let (dots, _) = line.dots(point(px(0.), px(0.)));

        let mut slot = PathCache::default();
        let mut builds = 0;
        let warm_origin = point(px(11.), px(7.));
        let area_key = area.shape_key("area/fill");
        let area_fill = slot
            .get(area_key, warm_origin, || {
                builds += 1;
                area.path(&Bounds::new(point(px(0.), px(0.)), size(px(30.), px(40.))))
                    .0
            })
            .unwrap();
        assert_eq!(builds, 1);
        // The warmed fill carries the area's baseline corners (y0 = 30) at
        // the warm origin: (first_x, 30) + (11, 7).
        assert!(area_fill.vertices.iter().any(|v| {
            v.xy_position.x.as_f32() == warm_origin.x.as_f32()
                && v.xy_position.y.as_f32() == 30. + warm_origin.y.as_f32()
        }));

        let line_key = line.shape_key(&dots);
        assert_ne!(
            line_key, area_key,
            "different primitive purposes must have distinct cache keys"
        );
        let served = slot.get(line_key, warm_origin, || {
            builds += 1;
            line.build_path(&dots)
        });
        assert_eq!(
            builds, 2,
            "the line must rebuild, not serve the warmed area fill"
        );

        // The warmed area is already translated to the requested origin.
        let aliased_serve: Vec<(f32, f32)> = area_fill
            .vertices
            .iter()
            .map(|v| (v.xy_position.x.as_f32(), v.xy_position.y.as_f32()))
            .collect();
        let served = served.expect("the line fixture must build a stroke");
        let got: Vec<(f32, f32)> = served
            .vertices
            .iter()
            .map(|v| (v.xy_position.x.as_f32(), v.xy_position.y.as_f32()))
            .collect();
        assert_ne!(
            got, aliased_serve,
            "the line must not serve the warmed area fill"
        );
        let fresh = line
            .build_path(&dots)
            .expect("the uncached line must build a stroke");
        let want: Vec<(f32, f32)> = fresh
            .vertices
            .iter()
            .map(|v| {
                (
                    v.xy_position.x.as_f32() + warm_origin.x.as_f32(),
                    v.xy_position.y.as_f32() + warm_origin.y.as_f32(),
                )
            })
            .collect();
        assert_eq!(got, want);
    }
}
