//! Retained native plots: compile data on commits, paint and hit-test in GPUI.
use super::primitives::Color;
use crate::{
    ExtensionChildSummary,
    native::{ComponentDefinition, Event, NativeChildren, NativeView, TS},
};
use gpui::{
    AnyElement, App, Background, Bounds, Context, ElementId, IntoElement, Pixels, Point, Render,
    SharedString, Window, linear_color_stop, linear_gradient, px,
};
use gpui_component::{
    chart,
    plot::{
        IntoPlot, Plot,
        shape::{BarAlignment, Sankey, SankeyAlign, SankeyLink, SankeyValueScale},
        tooltip::TooltipState,
    },
};
use std::{cell::RefCell, rc::Rc};
const MAX_POINTS: usize = 16_384;
const MAX_SERIES: usize = 32;
fn yes() -> bool {
    true
}
fn one() -> usize {
    1
}
fn five() -> usize {
    5
}
fn ten() -> f32 {
    10.
}
fn number(v: f64, name: &str) -> Result<(), String> {
    if v.is_finite() {
        Ok(())
    } else {
        Err(format!("{name} must be finite"))
    }
}
fn length(v: f32, name: &str) -> Result<(), String> {
    if v.is_finite() && (0.0..=1_000_000.).contains(&v) {
        Ok(())
    } else {
        Err(format!("{name} must be between 0 and 1000000"))
    }
}
fn ratio(v: f32, name: &str) -> Result<(), String> {
    if v.is_finite() && (0.0..=1.).contains(&v) {
        Ok(())
    } else {
        Err(format!("{name} must be between 0 and 1"))
    }
}
fn count(n: usize, max: usize, name: &str) -> Result<(), String> {
    if (1..=max).contains(&n) {
        Ok(())
    } else {
        Err(format!("{name} must be between 1 and {max}"))
    }
}
fn data_len(n: usize) -> Result<(), String> {
    if n <= MAX_POINTS {
        Ok(())
    } else {
        Err(format!("a plot supports at most {MAX_POINTS} data points"))
    }
}
fn numeric_range(values: impl Iterator<Item = f64>) -> Result<(), String> {
    let (mut low, mut high) = (0.0f64, 0.0f64);
    for v in values {
        number(v, "value")?;
        low = low.min(v);
        high = high.max(v);
    }
    number(high - low, "value range")
}

#[crate::native_type]
#[derive(Clone)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PlotFill {
    Solid {
        color: Color,
    },
    Linear {
        angle: f32,
        from: Color,
        to: Color,
        #[serde(default)]
        start: f32,
        #[serde(default = "full")]
        end: f32,
    },
}
fn full() -> f32 {
    1.
}
impl PlotFill {
    pub(super) fn validate(&self) -> Result<(), String> {
        match self {
            Self::Solid { .. } => Ok(()),
            Self::Linear {
                angle, start, end, ..
            } => {
                number(*angle as f64, "gradient angle")?;
                ratio(*start, "gradient start")?;
                ratio(*end, "gradient end")?;
                if start > end {
                    Err("gradient start must not exceed end".into())
                } else {
                    Ok(())
                }
            }
        }
    }
    pub(super) fn native(&self) -> Background {
        match self {
            Self::Solid { color } => color.native().into(),
            Self::Linear {
                angle,
                from,
                to,
                start,
                end,
            } => linear_gradient(
                angle.rem_euclid(360.),
                linear_color_stop(from.native(), *start),
                linear_color_stop(to.native(), *end),
            ),
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Default)]
#[serde(rename_all = "camelCase")]
pub enum PlotCurve {
    #[default]
    Natural,
    Linear,
    StepAfter,
}
impl From<PlotCurve> for gpui_component::plot::StrokeStyle {
    fn from(v: PlotCurve) -> Self {
        match v {
            PlotCurve::Natural => Self::Natural,
            PlotCurve::Linear => Self::Linear,
            PlotCurve::StepAfter => Self::StepAfter,
        }
    }
}
#[crate::native_type]
#[derive(Clone)]
pub struct ChartPoint {
    pub label: String,
    pub value: f64,
}
pub(super) struct PointDatum {
    label: SharedString,
    value: f64,
}
impl From<ChartPoint> for PointDatum {
    fn from(v: ChartPoint) -> Self {
        Self {
            label: v.label.into(),
            value: v.value,
        }
    }
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct LineChartProps {
    pub data: Vec<ChartPoint>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub stroke: Option<Color>,
    #[serde(default)]
    pub curve: PlotCurve,
    #[serde(default)]
    pub dot: bool,
    #[serde(default = "one")]
    pub tick_margin: usize,
    #[serde(default = "yes")]
    pub x_axis: bool,
    #[serde(default = "yes")]
    pub grid: bool,
    #[serde(default = "yes")]
    pub interactive: bool,
}
#[crate::native_type]
#[derive(Clone)]
pub struct ChartValues {
    pub label: String,
    pub values: Vec<f64>,
}
pub(super) struct ValuesDatum {
    label: SharedString,
    values: Vec<f64>,
}
impl From<ChartValues> for ValuesDatum {
    fn from(v: ChartValues) -> Self {
        Self {
            label: v.label.into(),
            values: v.values,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Default)]
#[serde(default)]
pub struct ChartSeries {
    pub name: Option<String>,
    pub stroke: Option<Color>,
    pub fill: Option<PlotFill>,
    pub curve: PlotCurve,
}
fn validate_series<'a>(
    series_count: usize,
    fills: impl Iterator<Item = &'a Option<PlotFill>>,
    rows: impl Iterator<Item = &'a Vec<f64>>,
    signed: bool,
) -> Result<(), String> {
    count(series_count, MAX_SERIES, "series count")?;
    for fill in fills.flatten() {
        fill.validate()?;
    }
    let mut points = 0usize;
    for values in rows {
        points += values.len();
        data_len(points)?;
        if values.len() != series_count {
            return Err("every data row must contain exactly one value per series".into());
        }
        numeric_range(values.iter().copied())?;
        if !signed && values.iter().any(|v| *v < 0.) {
            return Err("radar values must be nonnegative".into());
        }
    }
    Ok(())
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct AreaChartProps {
    pub data: Vec<ChartValues>,
    pub series: Vec<ChartSeries>,
    #[serde(default = "one")]
    pub tick_margin: usize,
    #[serde(default = "yes")]
    pub x_axis: bool,
    #[serde(default = "yes")]
    pub grid: bool,
    #[serde(default = "yes")]
    pub interactive: bool,
}
#[crate::native_type]
#[derive(Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChartAlignment {
    #[default]
    Bottom,
    Top,
    Left,
    Right,
}
impl From<ChartAlignment> for BarAlignment {
    fn from(v: ChartAlignment) -> Self {
        match v {
            ChartAlignment::Bottom => Self::Bottom,
            ChartAlignment::Top => Self::Top,
            ChartAlignment::Left => Self::Left,
            ChartAlignment::Right => Self::Right,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct PlotCorners {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}
impl PlotCorners {
    pub(super) fn validate(&self) -> Result<(), String> {
        for v in [
            self.top_left,
            self.top_right,
            self.bottom_left,
            self.bottom_right,
        ] {
            length(v, "corner radius")?;
        }
        Ok(())
    }
    pub(super) fn native(&self) -> gpui::Corners<Pixels> {
        gpui::Corners {
            top_left: px(self.top_left),
            top_right: px(self.top_right),
            bottom_left: px(self.bottom_left),
            bottom_right: px(self.bottom_right),
        }
    }
}
#[crate::native_type]
#[derive(Clone)]
pub struct BarChartDatum {
    pub label: String,
    pub value: f64,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub fill: Option<PlotFill>,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct BarGradient {
    pub from: Color,
    pub to: Color,
    #[serde(default)]
    pub chart_range: bool,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct BarChartProps {
    pub data: Vec<BarChartDatum>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub fill: Option<PlotFill>,
    #[serde(default)]
    pub gradient: Option<BarGradient>,
    #[serde(default)]
    pub labels: bool,
    #[serde(default)]
    pub alignment: ChartAlignment,
    #[serde(default)]
    pub corner_radii: PlotCorners,
    #[serde(default = "one")]
    pub tick_margin: usize,
    #[serde(default = "yes")]
    pub label_axis: bool,
    #[serde(default)]
    pub value_axis: bool,
    #[serde(default = "five")]
    pub value_tick_count: usize,
    #[serde(default = "yes")]
    pub grid: bool,
    #[serde(default = "yes")]
    pub interactive: bool,
}
pub(super) struct BarDatum {
    label: SharedString,
    value: f64,
    text: SharedString,
    fill: Option<Background>,
}
#[crate::native_type]
#[derive(Clone)]
pub struct Candlestick {
    pub label: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct CandlestickChartProps {
    pub data: Vec<Candlestick>,
    #[serde(default = "one")]
    pub tick_margin: usize,
    #[serde(default = "body_ratio")]
    pub body_width_ratio: f32,
    #[serde(default = "yes")]
    pub x_axis: bool,
    #[serde(default = "yes")]
    pub grid: bool,
}
fn body_ratio() -> f32 {
    0.7
}
pub(super) struct CandleDatum {
    label: SharedString,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct PieSlice {
    pub value: f32,
    pub color: Color,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub label_line_color: Option<Color>,
    #[serde(default)]
    pub inner_radius: Option<f32>,
    #[serde(default)]
    pub outer_radius: Option<f32>,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct PieChartProps {
    pub data: Vec<PieSlice>,
    #[serde(default)]
    pub inner_radius: f32,
    #[serde(default)]
    pub outer_radius: f32,
    #[serde(default)]
    pub pad_angle: f32,
    #[serde(default)]
    pub labels: bool,
    #[serde(default)]
    pub label_color: Option<Color>,
    #[serde(default = "ten")]
    pub label_gap: f32,
}
pub(super) struct PieDatum {
    value: f32,
    color: gpui::Hsla,
    label: SharedString,
    line: gpui::Hsla,
    inner: Option<f32>,
    outer: Option<f32>,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct RadarDatum {
    pub label: String,
    pub values: Vec<f64>,
    #[serde(default)]
    pub label_slot: Option<usize>,
}
pub(super) struct NativeRadarDatum {
    label: SharedString,
    values: Vec<f64>,
    slot: Option<crate::native::NativeSlot>,
}
#[crate::native_type]
#[derive(Clone, Default)]
#[serde(default)]
pub struct RadarSeries {
    pub name: Option<String>,
    pub stroke: Option<Color>,
    pub fill: Option<PlotFill>,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct RadarChartProps {
    pub data: Vec<RadarDatum>,
    pub series: Vec<RadarSeries>,
    #[serde(default)]
    pub max_value: Option<f64>,
    #[serde(default)]
    pub outer_radius: f32,
    #[serde(default = "yes")]
    pub grid: bool,
    #[serde(default = "five")]
    pub grid_levels: usize,
    #[serde(default)]
    pub dot: bool,
    #[serde(default = "yes")]
    pub interactive: bool,
    #[serde(default)]
    pub label_color: Option<Color>,
    #[serde(default = "ten")]
    pub label_gap: f32,
}
#[crate::native_type]
#[derive(Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum FlowAlignment {
    Left,
    Right,
    Center,
    #[default]
    Justify,
}
impl From<FlowAlignment> for SankeyAlign {
    fn from(v: FlowAlignment) -> Self {
        match v {
            FlowAlignment::Left => Self::Left,
            FlowAlignment::Right => Self::Right,
            FlowAlignment::Center => Self::Center,
            FlowAlignment::Justify => Self::Justify,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum FlowScale {
    #[default]
    Linear,
    Sqrt,
}
impl From<FlowScale> for SankeyValueScale {
    fn from(v: FlowScale) -> Self {
        match v {
            FlowScale::Linear => Self::Linear,
            FlowScale::Sqrt => Self::Sqrt,
        }
    }
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlowLabel {
    pub text: String,
    #[serde(default)]
    pub color: Option<Color>,
    #[serde(default)]
    pub font_size: Option<f32>,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlowNode {
    pub label: String,
    pub color: Color,
    #[serde(default)]
    pub value_label: Option<String>,
    #[serde(default)]
    pub labels: Option<Vec<FlowLabel>>,
}
#[crate::native_type]
#[derive(Clone)]
pub struct FlowLink {
    pub source: usize,
    pub target: usize,
    pub value: f64,
}
impl From<&FlowLink> for SankeyLink {
    fn from(v: &FlowLink) -> Self {
        Self::new(v.source, v.target, v.value)
    }
}
fn node_width() -> f32 {
    16.
}
fn node_padding() -> f32 {
    12.
}
fn iterations() -> usize {
    6
}
fn link_opacity() -> f32 {
    0.35
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct SankeyChartProps {
    pub nodes: Vec<FlowNode>,
    pub links: Vec<FlowLink>,
    #[serde(default = "node_width")]
    pub node_width: f32,
    #[serde(default = "node_padding")]
    pub node_padding: f32,
    #[serde(default)]
    pub alignment: FlowAlignment,
    #[serde(default = "iterations")]
    pub iterations: usize,
    #[serde(default)]
    pub value_scale: FlowScale,
    #[serde(default)]
    pub node_corner_radius: f32,
    #[serde(default = "link_opacity")]
    pub link_opacity: f32,
    #[serde(default)]
    pub min_link_width: f32,
    #[serde(default = "ten")]
    pub label_gap: f32,
    #[serde(default)]
    pub value_labels: bool,
}
pub(super) struct NativeFlowNode {
    label: SharedString,
    color: gpui::Hsla,
    value_label: Option<SharedString>,
    labels: Option<Vec<chart::SankeyLabel>>,
}

pub(super) trait ChartProps: serde::de::DeserializeOwned + TS + 'static {
    type Native: Plot + 'static;
    fn validate(&self) -> Result<(), String>;
    fn validate_children(&self, c: &ExtensionChildSummary) -> Result<(), String> {
        if c.content_count() == 0 {
            Ok(())
        } else {
            Err("this chart does not accept children".into())
        }
    }
    fn build(self, children: &NativeChildren) -> Self::Native;
    fn children() -> bool {
        false
    }
}
struct Chart<P: ChartProps> {
    plot: Rc<RefCell<P::Native>>,
    children: NativeChildren,
}
impl<P: ChartProps> NativeView for Chart<P> {
    type Props = P;
    type Event = ();
    fn emits_primary_event() -> bool {
        false
    }
    fn accepts_children() -> bool {
        P::children()
    }
    fn validate_props(p: &P) -> Result<(), String> {
        p.validate()
    }
    fn validate_children(p: &P, c: &ExtensionChildSummary) -> Result<(), String> {
        p.validate_children(c)
    }
    fn mount(
        p: P,
        _: Event<()>,
        children: NativeChildren,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Self {
        Self {
            plot: Rc::new(RefCell::new(p.build(&children))),
            children,
        }
    }
    fn update(&mut self, p: P, _: &mut Window, cx: &mut Context<Self>) {
        *self.plot.borrow_mut() = p.build(&self.children);
        cx.notify();
    }
}
impl<P: ChartProps> Render for Chart<P> {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        SharedPlot(self.plot.clone())
    }
}
#[derive(IntoPlot)]
struct SharedPlot<T: Plot + 'static>(Rc<RefCell<T>>);
impl<T: Plot + 'static> Plot for SharedPlot<T> {
    fn prepaint(&mut self, b: Bounds<Pixels>, w: &mut Window, c: &mut App) -> Vec<AnyElement> {
        self.0.borrow_mut().prepaint(b, w, c)
    }
    fn paint(&mut self, b: Bounds<Pixels>, w: &mut Window, c: &mut App) {
        self.0.borrow_mut().paint(b, w, c);
    }
    fn id(&self) -> Option<ElementId> {
        self.0.borrow().id()
    }
    fn tooltip_state(&self, p: Point<Pixels>, b: Bounds<Pixels>, c: &App) -> Option<TooltipState> {
        self.0.borrow().tooltip_state(p, b, c)
    }
    fn tooltip(
        &self,
        s: &TooltipState,
        p: Point<Pixels>,
        b: Bounds<Pixels>,
        w: &mut Window,
        c: &mut App,
    ) -> Option<AnyElement> {
        self.0.borrow().tooltip(s, p, b, w, c)
    }
}
impl ChartProps for LineChartProps {
    type Native = chart::LineChart<PointDatum, SharedString, f64>;
    fn validate(&self) -> Result<(), String> {
        data_len(self.data.len())?;
        count(self.tick_margin, MAX_POINTS, "tickMargin")?;
        numeric_range(self.data.iter().map(|d| d.value))
    }
    fn build(self, _: &NativeChildren) -> Self::Native {
        let mut c = chart::LineChart::new(self.data.into_iter().map(PointDatum::from))
            .x(|d: &PointDatum| d.label.clone())
            .y(|d: &PointDatum| d.value)
            .tick_margin(self.tick_margin)
            .x_axis(self.x_axis)
            .grid(self.grid);
        if self.interactive {
            c = c.id("chart");
        }
        if let Some(v) = self.name {
            c = c.name(v);
        }
        if let Some(v) = self.stroke {
            c = c.stroke(v.native());
        }
        if self.dot {
            c = c.dot();
        }
        match self.curve {
            PlotCurve::Natural => c.natural(),
            PlotCurve::Linear => c.linear(),
            PlotCurve::StepAfter => c.step_after(),
        }
    }
}
impl ChartProps for AreaChartProps {
    type Native = chart::AreaChart<ValuesDatum, SharedString, f64>;
    fn validate(&self) -> Result<(), String> {
        data_len(self.data.len())?;
        count(self.tick_margin, MAX_POINTS, "tickMargin")?;
        validate_series(
            self.series.len(),
            self.series.iter().map(|s| &s.fill),
            self.data.iter().map(|d| &d.values),
            true,
        )?;
        numeric_range(self.data.iter().flat_map(|d| d.values.iter().copied()))
    }
    fn build(self, _: &NativeChildren) -> Self::Native {
        let mut c = chart::AreaChart::new(self.data.into_iter().map(ValuesDatum::from))
            .x(|d: &ValuesDatum| d.label.clone())
            .tick_margin(self.tick_margin)
            .x_axis(self.x_axis)
            .grid(self.grid);
        for (i, s) in self.series.into_iter().enumerate() {
            c = c.y(move |d: &ValuesDatum| d.values[i]);
            if let Some(v) = s.name {
                c = c.name(v);
            }
            if let Some(v) = s.stroke {
                c = c.stroke(v.native());
            }
            if let Some(v) = s.fill {
                c = c.fill(v.native());
            }
            c = match s.curve {
                PlotCurve::Natural => c.natural(),
                PlotCurve::Linear => c.linear(),
                PlotCurve::StepAfter => c.step_after(),
            };
        }
        if self.interactive {
            c = c.id("chart");
        }
        c
    }
}
impl ChartProps for BarChartProps {
    type Native = chart::BarChart<BarDatum, SharedString, f64>;
    fn validate(&self) -> Result<(), String> {
        data_len(self.data.len())?;
        count(self.tick_margin, MAX_POINTS, "tickMargin")?;
        count(self.value_tick_count, 128, "valueTickCount")?;
        self.corner_radii.validate()?;
        if (self.fill.is_some() || self.data.iter().any(|d| d.fill.is_some()))
            && self.gradient.is_some()
        {
            return Err("choose fill or gradient".into());
        }
        if let Some(f) = &self.fill {
            f.validate()?;
        }
        for f in self.data.iter().filter_map(|d| d.fill.as_ref()) {
            f.validate()?;
        }
        numeric_range(self.data.iter().map(|d| d.value))?;
        if self.data.iter().any(|d| !(d.value as f32).is_finite()) {
            return Err("bar values must fit f32 gradient coordinates".into());
        }
        Ok(())
    }
    fn build(self, _: &NativeChildren) -> Self::Native {
        let mut c = chart::BarChart::new(self.data.into_iter().map(|d| BarDatum {
            text: d.text.unwrap_or_else(|| d.value.to_string()).into(),
            label: d.label.into(),
            value: d.value,
            fill: d.fill.as_ref().map(PlotFill::native),
        }))
        .band(|d: &BarDatum| d.label.clone())
        .value(|d: &BarDatum| d.value)
        .alignment(self.alignment.into())
        .corner_radii(self.corner_radii.native())
        .tick_margin(self.tick_margin)
        .label_axis(self.label_axis)
        .value_axis(self.value_axis)
        .value_tick_count(self.value_tick_count)
        .grid(self.grid);
        let fill = self.fill.as_ref().map(PlotFill::native);
        c = c.fill_optional(move |d: &BarDatum, _, _, _| d.fill.or(fill));
        if let Some(g) = self.gradient {
            let (from, to) = (g.from.native(), g.to.native());
            c = c.fill_gradient(move |_, range, map| {
                let (start, end) = if g.chart_range {
                    (map(*range.start()), map(*range.end()))
                } else {
                    (0., 1.)
                };
                [linear_color_stop(from, start), linear_color_stop(to, end)]
            });
        }
        if self.labels {
            c = c.label(|d: &BarDatum| d.text.clone());
        }
        if let Some(v) = self.name {
            c = c.name(v);
        }
        if self.interactive {
            c = c.id("chart");
        }
        c
    }
}
impl ChartProps for CandlestickChartProps {
    type Native = chart::CandlestickChart<CandleDatum, SharedString, f64>;
    fn validate(&self) -> Result<(), String> {
        data_len(self.data.len())?;
        count(self.tick_margin, MAX_POINTS, "tickMargin")?;
        ratio(self.body_width_ratio, "bodyWidthRatio")?;
        for d in &self.data {
            numeric_range([d.low, d.open, d.close, d.high].into_iter())?;
            if d.low > d.open.min(d.close) || d.high < d.open.max(d.close) {
                return Err("OHLC requires low <= open/close <= high".into());
            }
        }
        numeric_range(self.data.iter().flat_map(|d| [d.low, d.high]))
    }
    fn build(self, _: &NativeChildren) -> Self::Native {
        chart::CandlestickChart::new(self.data.into_iter().map(|d| CandleDatum {
            label: d.label.into(),
            open: d.open,
            high: d.high,
            low: d.low,
            close: d.close,
        }))
        .x(|d: &CandleDatum| d.label.clone())
        .open(|d: &CandleDatum| d.open)
        .high(|d: &CandleDatum| d.high)
        .low(|d: &CandleDatum| d.low)
        .close(|d: &CandleDatum| d.close)
        .tick_margin(self.tick_margin)
        .body_width_ratio(self.body_width_ratio)
        .x_axis(self.x_axis)
        .grid(self.grid)
    }
}
impl ChartProps for PieChartProps {
    type Native = chart::PieChart<PieDatum>;
    fn validate(&self) -> Result<(), String> {
        data_len(self.data.len())?;
        length(self.inner_radius, "innerRadius")?;
        length(self.outer_radius, "outerRadius")?;
        length(self.label_gap, "labelGap")?;
        if self.pad_angle < 0. || self.pad_angle > std::f32::consts::TAU {
            return Err("padAngle must be in 0..2π".into());
        }
        let mut sum = 0f32;
        for d in &self.data {
            if d.value < 0. || !d.value.is_finite() {
                return Err("pie values must be finite and nonnegative".into());
            }
            sum += d.value;
            let inner = d.inner_radius.unwrap_or(self.inner_radius);
            let outer = d.outer_radius.unwrap_or(self.outer_radius);
            length(inner, "slice innerRadius")?;
            length(outer, "slice outerRadius")?;
            if outer != 0. && inner > outer {
                return Err("innerRadius must not exceed outerRadius".into());
            }
        }
        if !sum.is_finite() {
            return Err("pie sum must fit f32".into());
        }
        Ok(())
    }
    fn build(self, _: &NativeChildren) -> Self::Native {
        let (inner, outer) = (self.inner_radius, self.outer_radius);
        let mut c = chart::PieChart::new(self.data.into_iter().map(|d| PieDatum {
            value: d.value,
            color: d.color.native(),
            label: d.label.into(),
            line: d.label_line_color.as_ref().unwrap_or(&d.color).native(),
            inner: d.inner_radius,
            outer: d.outer_radius,
        }))
        .value(|d: &PieDatum| d.value)
        .color(|d: &PieDatum| d.color)
        .inner_radius(inner)
        .outer_radius(outer)
        .inner_radius_fn(move |a| a.data.inner.unwrap_or(inner))
        .outer_radius_fn(move |a| a.data.outer.unwrap_or(outer))
        .pad_angle(self.pad_angle)
        .label_gap(self.label_gap);
        if self.labels {
            c = c
                .label(|d: &PieDatum| d.label.clone())
                .label_line_color(|d: &PieDatum| d.line);
        }
        if let Some(v) = self.label_color {
            c = c.label_color(v.native());
        }
        c
    }
}
impl ChartProps for RadarChartProps {
    type Native = chart::RadarChart<NativeRadarDatum, f64>;
    fn children() -> bool {
        true
    }
    fn validate(&self) -> Result<(), String> {
        if self.data.len() > 128 {
            return Err("radar supports at most 128 axes".into());
        }
        validate_series(
            self.series.len(),
            self.series.iter().map(|s| &s.fill),
            self.data.iter().map(|d| &d.values),
            false,
        )?;
        length(self.outer_radius, "outerRadius")?;
        length(self.label_gap, "labelGap")?;
        count(self.grid_levels, 128, "gridLevels")?;
        if let Some(v) = self.max_value {
            number(v, "maxValue")?;
            if v <= 0. {
                return Err("maxValue must be positive".into());
            }
        }
        Ok(())
    }
    fn validate_children(&self, c: &ExtensionChildSummary) -> Result<(), String> {
        if self
            .data
            .iter()
            .any(|d| d.label_slot.is_some_and(|i| i >= c.content_count()))
        {
            Err("labelSlot must address a committed child".into())
        } else {
            Ok(())
        }
    }
    fn build(self, children: &NativeChildren) -> Self::Native {
        let content = children.content();
        let mut c = chart::RadarChart::new(self.data.into_iter().map(|d| NativeRadarDatum {
            label: d.label.into(),
            values: d.values,
            slot: d.label_slot.map(|i| content.item(i)),
        }))
        .label(|d: &NativeRadarDatum| match &d.slot {
            Some(slot) => chart::RadarLabel::Element(slot.clone().into_any_element()),
            None => chart::RadarLabel::Text(d.label.clone()),
        })
        .outer_radius(self.outer_radius)
        .grid(self.grid)
        .grid_levels(self.grid_levels)
        .label_gap(self.label_gap);
        for (i, s) in self.series.into_iter().enumerate() {
            c = c.value(move |d: &NativeRadarDatum| d.values[i]);
            if let Some(v) = s.name {
                c = c.name(v);
            }
            if let Some(v) = s.stroke {
                c = c.stroke(v.native());
            }
            if let Some(v) = s.fill {
                c = c.fill(v.native());
            }
        }
        if let Some(v) = self.max_value {
            c = c.max_value(v);
        }
        if let Some(v) = self.label_color {
            c = c.label_color(v.native());
        }
        if self.dot {
            c = c.dot();
        }
        if self.interactive {
            c = c.id("chart");
        }
        c
    }
}
impl ChartProps for SankeyChartProps {
    type Native = chart::SankeyChart<NativeFlowNode>;
    fn validate(&self) -> Result<(), String> {
        if self.nodes.len() > 512 || self.links.len() > 4096 {
            return Err("Sankey supports at most 512 nodes and 4096 links".into());
        }
        if self.iterations > 32 {
            return Err("iterations must not exceed 32".into());
        }
        for (v, name) in [
            (self.node_width, "nodeWidth"),
            (self.node_padding, "nodePadding"),
            (self.node_corner_radius, "nodeCornerRadius"),
            (self.min_link_width, "minLinkWidth"),
            (self.label_gap, "labelGap"),
        ] {
            length(v, name)?;
        }
        ratio(self.link_opacity, "linkOpacity")?;
        let mut sum = 0f64;
        for link in &self.links {
            number(link.value, "link value")?;
            if link.value < 0. {
                return Err("link values must be nonnegative".into());
            }
            sum += link.value;
        }
        number(sum, "flow sum")?;
        let links = self.links.iter().map(SankeyLink::from).collect::<Vec<_>>();
        Sankey::new()
            .topology(self.nodes.len(), &links)
            .map_err(|e| e.to_string())?;
        for node in &self.nodes {
            if let Some(labels) = &node.labels {
                if labels.len() > 32 {
                    return Err("a node supports at most 32 labels".into());
                }
                for label in labels {
                    if let Some(size) = label.font_size
                        && !(1.0..=256.).contains(&size)
                    {
                        return Err("fontSize must be between 1 and 256".into());
                    }
                }
            }
        }
        Ok(())
    }
    fn build(self, _: &NativeChildren) -> Self::Native {
        let custom = self.nodes.iter().any(|n| n.labels.is_some());
        let nodes = self.nodes.into_iter().map(|n| NativeFlowNode {
            label: n.label.into(),
            color: n.color.native(),
            value_label: n.value_label.map(Into::into),
            labels: n.labels.map(|v| {
                v.into_iter()
                    .map(|l| {
                        let mut label = chart::SankeyLabel::new(l.text);
                        if let Some(v) = l.color {
                            label = label.color(v.native());
                        }
                        if let Some(v) = l.font_size {
                            label = label.font_size(v);
                        }
                        label
                    })
                    .collect()
            }),
        });
        let mut c = chart::SankeyChart::new(nodes, self.links.iter().map(SankeyLink::from))
            .node_width(self.node_width)
            .node_padding(self.node_padding)
            .node_align(self.alignment.into())
            .iterations(self.iterations)
            .value_scale(self.value_scale.into())
            .node_corner_radius(px(self.node_corner_radius))
            .node_color(|d: &NativeFlowNode| d.color)
            .node_label(|d: &NativeFlowNode| d.label.clone())
            .link_opacity(self.link_opacity)
            .min_link_width(self.min_link_width)
            .label_gap(self.label_gap);
        if self.value_labels {
            c = c.value_label(|d: &NativeFlowNode, v| {
                d.value_label
                    .clone()
                    .unwrap_or_else(|| v.to_string().into())
            });
        }
        if custom {
            let value_labels = self.value_labels;
            c = c.labels(move |d: &NativeFlowNode, value| {
                d.labels.clone().unwrap_or_else(|| {
                    let mut labels = vec![chart::SankeyLabel::new(d.label.clone())];
                    if value_labels {
                        labels.push(chart::SankeyLabel::new(
                            d.value_label
                                .clone()
                                .unwrap_or_else(|| value.to_string().into()),
                        ));
                    }
                    labels
                })
            });
        }
        c
    }
}
pub(super) fn definition<P: ChartProps>(name: &'static str) -> ComponentDefinition {
    ComponentDefinition::view::<Chart<P>>(name)
}
pub(super) fn definitions() -> Vec<ComponentDefinition> {
    vec![
        ComponentDefinition::view::<Chart<LineChartProps>>("LineChart"),
        ComponentDefinition::view::<Chart<AreaChartProps>>("AreaChart"),
        ComponentDefinition::view::<Chart<BarChartProps>>("BarChart"),
        ComponentDefinition::view::<Chart<CandlestickChartProps>>("CandlestickChart"),
        ComponentDefinition::view::<Chart<PieChartProps>>("PieChart"),
        ComponentDefinition::view::<Chart<RadarChartProps>>("RadarChart"),
        ComponentDefinition::view::<Chart<SankeyChartProps>>("SankeyChart"),
    ]
    .into_iter()
    .map(|d| d.with_contract(include_str!("charts.rs")))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_support::Fixture;
    use gpui::{AppContext, TestAppContext, point, size};
    fn props<P: serde::de::DeserializeOwned>(json: &str) -> P {
        crate::native::decode_json(json.as_bytes()).unwrap()
    }
    fn draw<P: ChartProps>(json: &str, cx: &mut TestAppContext) {
        let p: P = props(json);
        p.validate().unwrap();
        let fixture = Fixture::<Chart<P>>::new(p, cx);
        cx.update_window(fixture.window.into(), |_, window, cx| {
            window.draw(cx).clear(cx)
        })
        .unwrap();
    }
    #[gpui::test]
    fn low_level_plot_draws_native_shapes_and_validates_work(cx: &mut TestAppContext) {
        use crate::components::plot::PlotProps;
        draw::<PlotProps>(
            r##"{"primitives":[
            {"kind":"axis","x":180,"y":20,"xLabels":[{"text":"A","tick":40,"color":"#ffffff"}],"stroke":"#777777"},
            {"kind":"grid","x":[40,100],"y":[30,70],"stroke":"#777777","dash":[4,4]},
            {"kind":"labels","items":[{"text":"Title","x":20,"y":10,"color":"#ffffff","fontWeight":700}]},
            {"kind":"line","points":[{"x":20,"y":50},{"x":null,"y":null},{"x":70,"y":90}],"stroke":{"kind":"solid","color":"#ff0000"},"dots":{"fill":"#ffffff"}},
            {"kind":"area","points":[{"x":20,"y":90},{"x":100,"y":130}],"baseline":160,"stroke":{"kind":"solid","color":"#00ff00"},"fill":{"kind":"solid","color":"#003300"}},
            {"kind":"bar","rows":[{"cross":150,"base":160,"value":60,"fill":{"kind":"solid","color":"#0000ff"},"labels":[{"text":"Bar","x":0,"y":-12,"color":"#ffffff"}]}],"bandWidth":30},
            {"kind":"radialLine","points":[{"angle":0,"radius":50},{"angle":2,"radius":50},{"angle":4,"radius":50}],"closed":true,"stroke":{"kind":"solid","color":"#ffff00"}},
            {"kind":"arc","startAngle":0,"endAngle":2,"innerRadius":15,"outerRadius":35,"fill":"#ff00ff"}
        ]}"##,
            cx,
        );
        let bad: PlotProps = props(
            r##"{"primitives":[{"kind":"axis","xLabels":[{"text":"hidden","tick":1,"color":"#ffffff"}],"stroke":"#ffffff"}]}"##,
        );
        assert!(bad.validate().unwrap_err().contains("coordinate"));
        let bad: PlotProps =
            props(r##"{"primitives":[{"kind":"grid","stroke":"#ffffff","dash":[0]}]}"##);
        assert!(bad.validate().unwrap_err().contains("dash"));
    }
    #[gpui::test]
    fn charts_draw_and_hover_tracks_each_duplicate_category(cx: &mut TestAppContext) {
        draw::<AreaChartProps>(
            r##"{"data":[{"label":"A","values":[1,3]},{"label":"B","values":[2,4]}],"series":[{}, {"name":"Second","stroke":"#ff0000"}]}"##,
            cx,
        );
        draw::<BarChartProps>(
            r##"{"data":[{"label":"A","value":1},{"label":"B","value":3}],"gradient":{"from":"#ff0000","to":"#0000ff","chartRange":true},"labels":true,"valueAxis":true}"##,
            cx,
        );
        draw::<CandlestickChartProps>(
            r#"{"data":[{"label":"A","open":2,"high":5,"low":1,"close":4}]}"#,
            cx,
        );
        draw::<PieChartProps>(
            r##"{"data":[{"value":1,"color":"#ff0000","label":"A"},{"value":2,"color":"#0000ff","label":"B","outerRadius":100}],"labels":true}"##,
            cx,
        );
        draw::<RadarChartProps>(
            r##"{"data":[{"label":"A","values":[2,3]},{"label":"B","values":[4,2]},{"label":"C","values":[3,5]}],"series":[{}, {"stroke":"#ff0000"}],"dot":true}"##,
            cx,
        );
        draw::<SankeyChartProps>(
            r##"{"nodes":[{"label":"A","color":"#ff0000"},{"label":"B","color":"#0000ff"}],"links":[{"source":0,"target":1,"value":2}],"valueLabels":true}"##,
            cx,
        );
        let fixture = Fixture::<Chart<LineChartProps>>::new(
            props(r#"{"data":[{"label":"same","value":1},{"label":"same","value":3}],"dot":true}"#),
            cx,
        );
        cx.update_window(fixture.window.into(), |_, window, cx| {
            window.draw(cx).clear(cx)
        })
        .unwrap();
        fixture.update(cx, |view, _, cx| {
            let b = Bounds {
                origin: point(px(0.), px(0.)),
                size: size(px(400.), px(300.)),
            };
            let plot = view.plot.borrow();
            let hit = plot
                .tooltip_state(point(px(395.), px(100.)), b, cx)
                .unwrap();
            assert_eq!(hit.index, 1);
            assert_eq!(hit.cross_line.x, px(400.));
        });
    }
    #[test]
    fn reject_undefined_or_unbounded_chart_work() {
        assert!(
            props::<LineChartProps>(r#"{"data":[],"tickMargin":0}"#)
                .validate()
                .is_err()
        );
        assert!(
            props::<AreaChartProps>(r#"{"data":[{"label":"a","values":[1]}],"series":[{},{}]}"#)
                .validate()
                .is_err()
        );
        assert!(
            props::<CandlestickChartProps>(
                r#"{"data":[{"label":"a","open":4,"high":3,"low":1,"close":2}]}"#
            )
            .validate()
            .is_err()
        );
        assert!(props::<SankeyChartProps>(r##"{"nodes":[{"label":"a","color":"#ff0000"}],"links":[{"source":0,"target":0,"value":1}]}"##).validate().is_err());
        assert!(
            props::<SankeyChartProps>(r#"{"nodes":[],"links":[],"iterations":1000000}"#)
                .validate()
                .is_err()
        );
    }
}
