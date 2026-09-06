//! Low-level native plot shapes. Coordinates are logical pixels; angles are radians.
use super::{
    charts::{self, ChartAlignment, ChartProps, PlotCorners, PlotCurve, PlotFill},
    primitives::Color,
};
use crate::native::{ComponentDefinition, NativeChildren};
use gpui::{
    App, Bounds, FontWeight, ParentElement, Pixels, Point, TextAlign, Window, point, px, size,
};
use gpui_component::plot::{
    AxisLabelSide, AxisText, Grid, IntoPlot, Plot, PlotAxis,
    label::{PlotLabel, Text},
    shape::{Arc, ArcData, Area, Bar, Line, RadialLine},
    tooltip::{CrossLine, Dot, Tooltip},
};

pub(super) const MAX_ITEMS: usize = 16_384;
pub(super) fn items(n: usize) -> Result<(), String> {
    if n <= MAX_ITEMS {
        Ok(())
    } else {
        Err(format!("plot work exceeds {MAX_ITEMS} items"))
    }
}
pub(super) fn coordinate(v: f32) -> Result<(), String> {
    if v.is_finite() && (-1_000_000.0..=1_000_000.0).contains(&v) {
        Ok(())
    } else {
        Err("plot coordinates must be finite and between -1000000 and 1000000".into())
    }
}
pub(super) fn length(v: f32) -> Result<(), String> {
    coordinate(v)?;
    if v >= 0. {
        Ok(())
    } else {
        Err("plot lengths must be nonnegative".into())
    }
}
fn font_size(v: f32) -> Result<(), String> {
    if (1.0..=256.).contains(&v) {
        Ok(())
    } else {
        Err("plot font size must be between 1 and 256".into())
    }
}
fn one() -> f32 {
    1.
}
fn four() -> f32 {
    4.
}
fn six() -> f32 {
    6.
}
fn ten() -> f32 {
    10.
}
fn weight() -> f32 {
    400.
}
fn yes() -> bool {
    true
}
#[crate::native_type]
#[derive(Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum PlotTextAlign {
    #[default]
    Left,
    Center,
    Right,
}
impl From<PlotTextAlign> for TextAlign {
    fn from(v: PlotTextAlign) -> Self {
        match v {
            PlotTextAlign::Left => Self::Left,
            PlotTextAlign::Center => Self::Center,
            PlotTextAlign::Right => Self::Right,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum PlotLabelSide {
    Start,
    #[default]
    End,
}
impl From<PlotLabelSide> for AxisLabelSide {
    fn from(v: PlotLabelSide) -> Self {
        match v {
            PlotLabelSide::Start => Self::Start,
            PlotLabelSide::End => Self::End,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy)]
pub struct PlotPosition {
    pub x: f32,
    pub y: f32,
}
impl PlotPosition {
    fn validate(&self) -> Result<(), String> {
        coordinate(self.x)?;
        coordinate(self.y)
    }
    fn native(self) -> Point<Pixels> {
        point(px(self.x), px(self.y))
    }
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlotText {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub color: Color,
    #[serde(default = "ten")]
    pub font_size: f32,
    #[serde(default = "weight")]
    pub font_weight: f32,
    #[serde(default)]
    pub align: PlotTextAlign,
}
impl PlotText {
    fn validate(&self) -> Result<(), String> {
        coordinate(self.x)?;
        coordinate(self.y)?;
        font_size(self.font_size)?;
        if !(1.0..=1000.).contains(&self.font_weight) {
            return Err("fontWeight must be between 1 and 1000".into());
        }
        Ok(())
    }
    fn native(&self, origin: Point<Pixels>) -> Text {
        Text::new(
            self.text.clone(),
            origin + point(px(self.x), px(self.y)),
            self.color.native(),
        )
        .font_size(px(self.font_size))
        .font_weight(FontWeight(self.font_weight))
        .align(self.align.into())
    }
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlotAxisText {
    pub text: String,
    pub tick: f32,
    pub color: Color,
    #[serde(default = "ten")]
    pub font_size: f32,
    #[serde(default)]
    pub align: PlotTextAlign,
}
impl PlotAxisText {
    fn validate(&self) -> Result<(), String> {
        coordinate(self.tick)?;
        font_size(self.font_size)
    }
    fn native(self) -> AxisText {
        AxisText::new(self.text, px(self.tick), self.color.native())
            .font_size(px(self.font_size))
            .align(self.align.into())
    }
}
/// Null coordinates omit a point, using the native shape's connection semantics.
#[crate::native_type]
#[derive(Clone)]
pub struct PlotPoint {
    pub x: Option<f32>,
    pub y: Option<f32>,
}
#[crate::native_type]
#[derive(Clone)]
pub struct PlotRadialPoint {
    pub angle: Option<f32>,
    pub radius: Option<f32>,
}
#[crate::native_type]
#[derive(Clone)]
pub struct PlotBarRow {
    pub cross: Option<f32>,
    pub base: f32,
    pub value: Option<f32>,
    pub fill: PlotFill,
    #[serde(default)]
    pub labels: Vec<PlotText>,
}
#[crate::native_type]
#[derive(Clone)]
pub struct PlotDots {
    #[serde(default = "four")]
    pub size: f32,
    pub fill: Color,
    #[serde(default)]
    pub stroke: Option<Color>,
}
impl PlotDots {
    fn validate(&self) -> Result<(), String> {
        length(self.size)
    }
}
#[crate::native_type]
#[derive(Clone)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PlotPrimitive {
    Axis {
        x: Option<f32>,
        y: Option<f32>,
        #[serde(default = "yes")]
        x_axis: bool,
        #[serde(default = "yes")]
        y_axis: bool,
        #[serde(default)]
        x_labels: Vec<PlotAxisText>,
        #[serde(default)]
        y_labels: Vec<PlotAxisText>,
        #[serde(default)]
        x_label_side: PlotLabelSide,
        #[serde(default)]
        y_label_side: PlotLabelSide,
        stroke: Color,
    },
    Grid {
        #[serde(default)]
        x: Vec<f32>,
        #[serde(default)]
        y: Vec<f32>,
        stroke: Color,
        #[serde(default)]
        dash: Vec<f32>,
    },
    Labels {
        items: Vec<PlotText>,
    },
    Line {
        points: Vec<PlotPoint>,
        stroke: PlotFill,
        #[serde(default = "one")]
        stroke_width: f32,
        #[serde(default)]
        curve: PlotCurve,
        #[serde(default)]
        dots: Option<PlotDots>,
    },
    Area {
        points: Vec<PlotPoint>,
        baseline: f32,
        fill: PlotFill,
        stroke: PlotFill,
        #[serde(default)]
        curve: PlotCurve,
    },
    Bar {
        rows: Vec<PlotBarRow>,
        #[serde(default)]
        alignment: ChartAlignment,
        band_width: f32,
        #[serde(default)]
        corner_radii: PlotCorners,
    },
    RadialLine {
        points: Vec<PlotRadialPoint>,
        #[serde(default)]
        closed: bool,
        #[serde(default)]
        fill: Option<PlotFill>,
        stroke: PlotFill,
        #[serde(default = "one")]
        stroke_width: f32,
        #[serde(default)]
        dots: Option<PlotDots>,
    },
    Arc {
        start_angle: f32,
        end_angle: f32,
        #[serde(default)]
        pad_angle: f32,
        #[serde(default)]
        inner_radius: f32,
        outer_radius: f32,
        fill: Color,
    },
}
fn validate_points(points: &[PlotPoint]) -> Result<(), String> {
    for p in points {
        for v in [p.x, p.y].into_iter().flatten() {
            coordinate(v)?;
        }
    }
    Ok(())
}
impl PlotPrimitive {
    fn validate(&self) -> Result<usize, String> {
        Ok(match self {
            Self::Axis {
                x,
                y,
                x_labels,
                y_labels,
                ..
            } => {
                for v in [*x, *y].into_iter().flatten() {
                    coordinate(v)?;
                }
                if (x.is_none() && !x_labels.is_empty()) || (y.is_none() && !y_labels.is_empty()) {
                    return Err("axis labels require their axis coordinate".into());
                }
                for t in x_labels.iter().chain(y_labels) {
                    t.validate()?;
                }
                x_labels.len() + y_labels.len() + 2
            }
            Self::Grid { x, y, dash, .. } => {
                for v in x.iter().chain(y) {
                    coordinate(*v)?;
                }
                if dash.len() > 32 {
                    return Err("dash supports at most 32 lengths".into());
                }
                for d in dash {
                    if !d.is_finite() || !(1.0..=1_000_000.).contains(d) {
                        return Err("dash lengths must be between 1 and 1000000".into());
                    }
                }
                x.len() + y.len()
            }
            Self::Labels { items } => {
                for t in items {
                    t.validate()?;
                }
                items.len()
            }
            Self::Line {
                points,
                stroke,
                stroke_width,
                dots,
                ..
            } => {
                validate_points(points)?;
                stroke.validate()?;
                length(*stroke_width)?;
                if let Some(d) = dots {
                    d.validate()?;
                }
                points.len()
            }
            Self::Area {
                points,
                baseline,
                fill,
                stroke,
                ..
            } => {
                validate_points(points)?;
                coordinate(*baseline)?;
                fill.validate()?;
                stroke.validate()?;
                points.len()
            }
            Self::Bar {
                rows,
                band_width,
                corner_radii,
                ..
            } => {
                length(*band_width)?;
                corner_radii.validate()?;
                let mut n = rows.len();
                for r in rows {
                    for v in [r.cross, Some(r.base), r.value].into_iter().flatten() {
                        coordinate(v)?;
                    }
                    r.fill.validate()?;
                    n += r.labels.len();
                    for t in &r.labels {
                        t.validate()?;
                    }
                }
                n
            }
            Self::RadialLine {
                points,
                fill,
                stroke,
                stroke_width,
                dots,
                ..
            } => {
                for p in points {
                    if let Some(v) = p.angle {
                        coordinate(v)?;
                    }
                    if let Some(v) = p.radius {
                        length(v)?;
                    }
                }
                if let Some(v) = fill {
                    v.validate()?;
                }
                stroke.validate()?;
                length(*stroke_width)?;
                if let Some(d) = dots {
                    d.validate()?;
                }
                points.len()
            }
            Self::Arc {
                start_angle,
                end_angle,
                pad_angle,
                inner_radius,
                outer_radius,
                ..
            } => {
                coordinate(*start_angle)?;
                coordinate(*end_angle)?;
                length(*pad_angle)?;
                length(*inner_radius)?;
                length(*outer_radius)?;
                if inner_radius > outer_radius {
                    return Err("innerRadius must not exceed outerRadius".into());
                }
                1
            }
        })
    }
    fn build(self) -> NativePrimitive {
        match self {
            Self::Axis {
                x,
                y,
                x_axis,
                y_axis,
                x_labels,
                y_labels,
                x_label_side,
                y_label_side,
                stroke,
            } => {
                let mut a = PlotAxis::new()
                    .stroke(stroke.native())
                    .x_axis(x_axis)
                    .y_axis(y_axis)
                    .x_label_side(x_label_side.into())
                    .y_label_side(y_label_side.into());
                if let Some(v) = x {
                    a = a.x(px(v));
                }
                if let Some(v) = y {
                    a = a.y(px(v));
                }
                NativePrimitive::Axis(
                    a.x_label(x_labels.into_iter().map(PlotAxisText::native))
                        .y_label(y_labels.into_iter().map(PlotAxisText::native)),
                )
            }
            Self::Grid { x, y, stroke, dash } => {
                let g = Grid::new()
                    .x(x.into_iter().map(px).collect::<Vec<_>>())
                    .y(y.into_iter().map(px).collect::<Vec<_>>())
                    .stroke(stroke.native());
                NativePrimitive::Grid(if dash.is_empty() {
                    g
                } else {
                    g.dash_array(&dash.into_iter().map(px).collect::<Vec<_>>())
                })
            }
            Self::Labels { items } => NativePrimitive::Labels(PlotLabel::new(
                items.iter().map(|t| t.native(Point::default())).collect(),
            )),
            Self::Line {
                points,
                stroke,
                stroke_width,
                curve,
                dots,
            } => {
                let mut s = Line::new()
                    .data(points)
                    .x(|p: &PlotPoint| p.x)
                    .y(|p| p.y)
                    .stroke(stroke.native())
                    .stroke_width(px(stroke_width))
                    .stroke_style(curve.into());
                if let Some(d) = dots {
                    s = s.dot().dot_size(px(d.size)).dot_fill_color(d.fill.native());
                    if let Some(v) = d.stroke {
                        s = s.dot_stroke_color(v.native());
                    }
                }
                NativePrimitive::Line(s)
            }
            Self::Area {
                points,
                baseline,
                fill,
                stroke,
                curve,
            } => NativePrimitive::Area(
                Area::new()
                    .data(points)
                    .x(|p: &PlotPoint| p.x)
                    .y0(baseline)
                    .y1(|p| p.y)
                    .fill(fill.native())
                    .stroke(stroke.native())
                    .stroke_style(curve.into()),
            ),
            Self::Bar {
                rows,
                alignment,
                band_width,
                corner_radii,
            } => NativePrimitive::Bar(
                Bar::new()
                    .data(rows.into_iter().map(|r| {
                        NativeBarRow {
                            cross: r.cross,
                            base: r.base,
                            value: r.value,
                            fill: r.fill.native(),
                            labels: r
                                .labels
                                .iter()
                                .map(|t| t.native(Point::default()))
                                .collect(),
                        }
                    }))
                    .alignment(alignment.into())
                    .cross(|r: &NativeBarRow| r.cross)
                    .base(|r| r.base)
                    .value(|r| r.value)
                    .band_width(band_width)
                    .fill(|r, _, _| r.fill)
                    .label(|r, origin| {
                        r.labels
                            .iter()
                            .map(|t| Text {
                                text: t.text.clone(),
                                origin: origin + t.origin,
                                color: t.color,
                                font_size: t.font_size,
                                font_weight: t.font_weight,
                                align: t.align,
                            })
                            .collect()
                    })
                    .corner_radii(corner_radii.native()),
            ),
            Self::RadialLine {
                points,
                closed,
                fill,
                stroke,
                stroke_width,
                dots,
            } => {
                let mut s = RadialLine::new()
                    .data(points)
                    .angle(|p: &PlotRadialPoint, _| p.angle)
                    .radius(|p, _| p.radius)
                    .stroke(stroke.native())
                    .stroke_width(px(stroke_width));
                if closed {
                    s = s.closed();
                }
                if let Some(v) = fill {
                    s = s.fill(v.native());
                }
                if let Some(d) = dots {
                    s = s.dot().dot_size(px(d.size)).dot_fill_color(d.fill.native());
                    if let Some(v) = d.stroke {
                        s = s.dot_stroke_color(v.native());
                    }
                }
                NativePrimitive::RadialLine(s)
            }
            Self::Arc {
                start_angle,
                end_angle,
                pad_angle,
                inner_radius,
                outer_radius,
                fill,
            } => NativePrimitive::Arc {
                shape: Arc::new()
                    .inner_radius(inner_radius)
                    .outer_radius(outer_radius),
                start_angle,
                end_angle,
                pad_angle,
                fill: fill.native(),
            },
        }
    }
}
struct NativeBarRow {
    cross: Option<f32>,
    base: f32,
    value: Option<f32>,
    fill: gpui::Background,
    labels: Vec<Text>,
}
enum NativePrimitive {
    Axis(PlotAxis),
    Grid(Grid),
    Labels(PlotLabel),
    Line(Line<PlotPoint>),
    Area(Area<PlotPoint>),
    Bar(Bar<NativeBarRow>),
    RadialLine(RadialLine<PlotRadialPoint>),
    Arc {
        shape: Arc,
        start_angle: f32,
        end_angle: f32,
        pad_angle: f32,
        fill: gpui::Hsla,
    },
}
#[crate::native_type]
pub struct PlotProps {
    pub primitives: Vec<PlotPrimitive>,
}
#[derive(IntoPlot)]
pub(super) struct NativePlot {
    shapes: Vec<NativePrimitive>,
}
impl ChartProps for PlotProps {
    type Native = NativePlot;
    fn validate(&self) -> Result<(), String> {
        if self.primitives.len() > 256 {
            return Err("a Plot supports at most 256 primitives".into());
        }
        let mut count = 0;
        for p in &self.primitives {
            count += p.validate()?;
            items(count)?;
        }
        Ok(())
    }
    fn build(self, _: &NativeChildren) -> NativePlot {
        NativePlot {
            shapes: self
                .primitives
                .into_iter()
                .map(PlotPrimitive::build)
                .collect(),
        }
    }
}
impl Plot for NativePlot {
    fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        for p in &self.shapes {
            match p {
                NativePrimitive::Axis(v) => v.paint(&bounds, window, cx),
                NativePrimitive::Grid(v) => v.paint(&bounds, window),
                NativePrimitive::Labels(v) => v.paint(&bounds, window, cx),
                NativePrimitive::Line(v) => v.paint(&bounds, window),
                NativePrimitive::Area(v) => v.paint(&bounds, window),
                NativePrimitive::Bar(v) => v.paint(&bounds, window, cx),
                NativePrimitive::RadialLine(v) => v.paint(&bounds, window),
                NativePrimitive::Arc {
                    shape,
                    start_angle,
                    end_angle,
                    pad_angle,
                    fill,
                } => shape.paint(
                    &ArcData {
                        data: &(),
                        index: 0,
                        value: 0.,
                        start_angle: *start_angle,
                        end_angle: *end_angle,
                        pad_angle: *pad_angle,
                    },
                    *fill,
                    None,
                    None,
                    &bounds,
                    window,
                ),
            }
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum PlotCrossDirection {
    #[default]
    Vertical,
    Horizontal,
    Both,
}
#[crate::native_type]
#[derive(Clone, Copy)]
pub struct PlotSpan {
    pub start: f32,
    pub length: f32,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlotCrossLineProps {
    pub point: PlotPosition,
    #[serde(default)]
    pub direction: PlotCrossDirection,
    #[serde(default)]
    pub band_thickness: Option<f32>,
    #[serde(default)]
    pub vertical_span: Option<PlotSpan>,
    #[serde(default)]
    pub horizontal_span: Option<PlotSpan>,
}
impl PlotCrossLineProps {
    fn validate(&self) -> Result<(), String> {
        self.point.validate()?;
        if let Some(v) = self.band_thickness {
            length(v)?;
        }
        for v in [self.vertical_span, self.horizontal_span]
            .into_iter()
            .flatten()
        {
            coordinate(v.start)?;
            length(v.length)?;
        }
        Ok(())
    }
    fn native(&self) -> CrossLine {
        let mut c = CrossLine::new(self.point.native());
        c = match self.direction {
            PlotCrossDirection::Vertical => c,
            PlotCrossDirection::Horizontal => c.horizontal(),
            PlotCrossDirection::Both => c.both(),
        };
        if let Some(v) = self.band_thickness {
            c = c.band(px(v));
        }
        if let Some(v) = self.vertical_span {
            c = c.span(v.start, v.length);
        }
        if let Some(v) = self.horizontal_span {
            c = c.h_span(v.start, v.length);
        }
        c
    }
}
#[crate::native_type]
#[derive(Clone)]
pub struct PlotDotProps {
    pub point: PlotPosition,
    #[serde(default = "six")]
    pub size: f32,
    pub fill: Color,
    pub stroke: Color,
}
impl PlotDotProps {
    fn validate(&self) -> Result<(), String> {
        self.point.validate()?;
        length(self.size)
    }
    fn native(&self) -> Dot {
        Dot::new(self.point.native())
            .size(px(self.size))
            .fill(self.fill.native())
            .stroke(self.stroke.native())
    }
}
#[crate::native_type]
#[derive(Clone)]
pub struct PlotTooltipRow {
    pub color: Color,
    pub label: String,
    pub value: String,
}
#[crate::native_type]
#[derive(Clone)]
pub struct PlotSize {
    pub width: f32,
    pub height: f32,
}
#[crate::native_type]
#[serde(rename_all = "camelCase")]
pub struct PlotTooltipProps {
    pub cursor: PlotPosition,
    pub within: PlotSize,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub gap: f32,
    #[serde(default = "yes")]
    pub appearance: bool,
    #[serde(default)]
    pub rows: Vec<PlotTooltipRow>,
    #[serde(default)]
    pub cross_line: Option<PlotCrossLineProps>,
    #[serde(default)]
    pub dots: Vec<PlotDotProps>,
}
pub(super) fn definitions() -> Vec<ComponentDefinition> {
    vec![
        charts::definition::<PlotProps>("Plot"),
        ComponentDefinition::element("PlotCrossLine", vec![], |p: &PlotCrossLineProps, _| {
            p.native()
        })
        .with_children(false)
        .with_validation::<PlotCrossLineProps>(|p, _| p.validate()),
        ComponentDefinition::element("PlotDot", vec![], |p: &PlotDotProps, _| p.native())
            .with_children(false)
            .with_validation::<PlotDotProps>(|p, _| p.validate()),
        ComponentDefinition::styled_element("PlotTooltip", vec![], |p: &PlotTooltipProps, cx| {
            let mut t = Tooltip::new(
                p.cursor.native(),
                size(px(p.within.width), px(p.within.height)),
            )
            .gap(px(p.gap))
            .appearance(p.appearance)
            .dots(p.dots.iter().map(PlotDotProps::native));
            if let Some(v) = &p.title {
                t = t.title(v.clone());
            }
            if let Some(v) = &p.cross_line {
                t = t.cross_line(v.native());
            }
            for r in &p.rows {
                t = t.row(r.color.native(), r.label.clone(), r.value.clone());
            }
            t.children(cx.children())
        })
        .with_validation::<PlotTooltipProps>(|p, c| {
            p.cursor.validate()?;
            length(p.within.width)?;
            length(p.within.height)?;
            length(p.gap)?;
            items(p.rows.len() + p.dots.len())?;
            if c.content_count() > 0 && (p.title.is_some() || !p.rows.is_empty()) {
                return Err(
                    "PlotTooltip uses either structured title/rows or custom children".into(),
                );
            }
            if let Some(v) = &p.cross_line {
                v.validate()?;
            }
            for d in &p.dots {
                d.validate()?;
            }
            Ok(())
        }),
    ]
    .into_iter()
    .map(|d| d.with_contract(include_str!("plot.rs")))
    .collect()
}
