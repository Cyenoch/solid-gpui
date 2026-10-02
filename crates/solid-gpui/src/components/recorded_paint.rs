//! Bounded retained drawing. Coordinates and clips are in logical drawing units.
use super::primitives::Color;
use crate::native::{ComponentDefinition, Event, NativeChildren, NativeView};
use gpui::{
    App, Bounds, ContentMask, Context, Hsla, IntoElement, ParentElement, Path, Pixels, Render,
    SharedString, Styled, TextAlign, TextRun, Window, point, px, size,
};
use std::{cell::Cell, rc::Rc, sync::Arc};

pub(super) const MAX_COMMANDS: usize = 1024;
pub(super) const MAX_POINTS: usize = 4096;
pub(super) const MAX_VERTICES: usize = 32768;
pub(super) const MAX_TEXT_BYTES: usize = 16384;

#[crate::native_type]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintPoint {
    pub x: f32,
    pub y: f32,
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PaintColor {
    Solid { color: Color },
    Foreground,
    Background,
    Accent,
    Border,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PaintCommand {
    Quad {
        bounds: PaintRect,
        color: PaintColor,
    },
    Path {
        points: Vec<PaintPoint>,
        closed: bool,
        stroke: Option<PaintColor>,
        #[serde(rename = "strokeWidth")]
        stroke_width: f32,
        fill: Option<PaintColor>,
    },
    Text {
        origin: PaintPoint,
        text: String,
        #[serde(rename = "fontSize")]
        font_size: f32,
        color: PaintColor,
    },
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PaintFit {
    #[default]
    Contain,
    None,
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PaintTransform {
    pub scale: f32,
    pub translate_x: f32,
    pub translate_y: f32,
}
impl Default for PaintTransform {
    fn default() -> Self {
        Self {
            scale: 1.,
            translate_x: 0.,
            translate_y: 0.,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PaintRecording {
    pub width: f32,
    pub height: f32,
    pub commands: Vec<PaintCommand>,
    #[serde(default)]
    pub fit: PaintFit,
    #[serde(default)]
    pub transform: PaintTransform,
    #[serde(default)]
    pub clip: Option<PaintRect>,
}
impl PaintRecording {
    fn validate(&self) -> Result<(), String> {
        dimension(self.width)?;
        dimension(self.height)?;
        if !(0.01..=16.).contains(&self.transform.scale) {
            return Err("paint scale must be between 0.01 and 16".into());
        }
        coordinate(self.transform.translate_x)?;
        coordinate(self.transform.translate_y)?;
        if let Some(clip) = self.clip {
            clip.validate()?;
        }
        if self.commands.len() > MAX_COMMANDS {
            return Err(format!("paint recording exceeds {MAX_COMMANDS} commands"));
        }
        let (mut points, mut vertices, mut text_bytes) = (0, 0, 0);
        for command in &self.commands {
            match command {
                PaintCommand::Quad { bounds, .. } => bounds.validate()?,
                PaintCommand::Path {
                    points: path,
                    closed,
                    stroke,
                    stroke_width,
                    fill,
                } => {
                    if path.len() < 2
                        || (fill.is_some() && (!closed || path.len() < 3))
                        || (stroke.is_none() && fill.is_none())
                    {
                        return Err(
                            "paint paths need two points and a stroke, or a closed convex fill"
                                .into(),
                        );
                    }
                    points += path.len();
                    if points > MAX_POINTS {
                        return Err("paint recording exceeds 4096 path points".into());
                    }
                    for p in path {
                        p.validate()?;
                    }
                    if stroke.is_some() {
                        if !(0.25..=128.).contains(stroke_width) {
                            return Err("paint strokeWidth must be between 0.25 and 128".into());
                        }
                        vertices += 6 * (path.len() - 1 + usize::from(*closed));
                    }
                    if fill.is_some() {
                        validate_convex(path)?;
                        vertices += 3 * (path.len() - 2);
                    }
                    if vertices > MAX_VERTICES {
                        return Err("paint recording exceeds 32768 vertices".into());
                    }
                }
                PaintCommand::Text {
                    origin,
                    text,
                    font_size,
                    ..
                } => {
                    origin.validate()?;
                    if !(1.0..=256.).contains(font_size)
                        || font_size * self.transform.scale > 256.
                        || text.len() > 2048
                        || text.chars().any(char::is_control)
                    {
                        return Err("paint text needs a fontSize from 1 to 256 and at most 2048 single-line UTF-8 bytes".into());
                    }
                    text_bytes += text.len();
                    if text_bytes > MAX_TEXT_BYTES {
                        return Err("paint recording exceeds 16384 text bytes".into());
                    }
                }
            }
        }
        Ok(())
    }
    fn prepare(&self) -> Vec<PreparedCommand> {
        let mut commands = Vec::with_capacity(self.commands.len());
        for command in &self.commands {
            match command {
                PaintCommand::Quad { bounds, color } => {
                    commands.push(PreparedCommand::Quad(*bounds, color.clone()))
                }
                PaintCommand::Text {
                    origin,
                    text,
                    font_size,
                    color,
                } => commands.push(PreparedCommand::Text(
                    *origin,
                    text.clone().into(),
                    *font_size,
                    color.clone(),
                )),
                PaintCommand::Path {
                    points,
                    closed,
                    stroke,
                    stroke_width,
                    fill,
                } => {
                    if let Some(color) = fill {
                        let mut path = Path::new(points[0].native());
                        for edge in points[1..].windows(2) {
                            triangle(
                                &mut path,
                                points[0].native(),
                                edge[0].native(),
                                edge[1].native(),
                            );
                        }
                        commands.push(PreparedCommand::Path(path, color.clone()));
                    }
                    if let Some(color) = stroke {
                        let mut path = Path::new(points[0].native());
                        let count = points.len() - 1 + usize::from(*closed);
                        for i in 0..count {
                            let a = points[i];
                            let b = points[(i + 1) % points.len()];
                            let dx = b.x - a.x;
                            let dy = b.y - a.y;
                            let len = dx.hypot(dy);
                            if len == 0. {
                                continue;
                            }
                            let nx = -dy / len * stroke_width / 2.;
                            let ny = dx / len * stroke_width / 2.;
                            let a1 = point(px(a.x + nx), px(a.y + ny));
                            let a2 = point(px(a.x - nx), px(a.y - ny));
                            let b1 = point(px(b.x + nx), px(b.y + ny));
                            let b2 = point(px(b.x - nx), px(b.y - ny));
                            triangle(&mut path, a1, a2, b1);
                            triangle(&mut path, a2, b2, b1);
                        }
                        commands.push(PreparedCommand::Path(path, color.clone()));
                    }
                }
            }
        }
        commands
    }
}

fn coordinate(value: f32) -> Result<(), String> {
    if value.is_finite() && value.abs() <= 16384. {
        Ok(())
    } else {
        Err("paint coordinates must be finite and within 16384".into())
    }
}
pub(super) fn dimension(value: f32) -> Result<(), String> {
    if value.is_finite() && (1.0..=16384.).contains(&value) {
        Ok(())
    } else {
        Err("paint dimensions must be between 1 and 16384 logical pixels".into())
    }
}
impl PaintPoint {
    fn validate(self) -> Result<(), String> {
        coordinate(self.x)?;
        coordinate(self.y)
    }
    fn native(self) -> gpui::Point<Pixels> {
        point(px(self.x), px(self.y))
    }
}
impl PaintRect {
    fn validate(self) -> Result<(), String> {
        coordinate(self.x)?;
        coordinate(self.y)?;
        for length in [self.width, self.height] {
            if !length.is_finite() || !(0.0..=16384.).contains(&length) {
                return Err("paint rectangle lengths must be between 0 and 16384".into());
            }
        }
        Ok(())
    }
}
fn validate_convex(points: &[PaintPoint]) -> Result<(), String> {
    // Requiring a consistent turn plus exactly one winding excludes self-intersecting
    // stars while keeping admission and fan triangulation linear in point count.
    let mut sign = 0.;
    let mut turning = 0.;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        let c = points[(i + 2) % points.len()];
        let (dx, dy) = ((b.x - a.x) as f64, (b.y - a.y) as f64);
        let (ex, ey) = ((c.x - b.x) as f64, (c.y - b.y) as f64);
        let cross = dx * ey - dy * ex;
        if cross == 0. || (sign != 0. && cross.signum() != sign) {
            return Err("filled paint paths must be strictly convex polygons".into());
        }
        sign = cross.signum();
        turning += cross.atan2(dx * ex + dy * ey);
    }
    if (turning.abs() - std::f64::consts::TAU).abs() > 0.0001 {
        return Err("filled paint paths must have one convex winding".into());
    }
    Ok(())
}
fn triangle(
    path: &mut Path<Pixels>,
    a: gpui::Point<Pixels>,
    b: gpui::Point<Pixels>,
    c: gpui::Point<Pixels>,
) {
    path.push_triangle((a, b, c), (point(0., 1.), point(0., 1.), point(0., 1.)));
}
impl PaintColor {
    fn resolve(&self, cx: &App) -> Hsla {
        let theme = gpui_component::Theme::global(cx);
        match self {
            Self::Solid { color } => color.native(),
            Self::Foreground => theme.foreground,
            Self::Background => theme.background,
            Self::Accent => theme.primary,
            Self::Border => theme.border,
        }
    }
}
enum PreparedCommand {
    Quad(PaintRect, PaintColor),
    Path(Path<Pixels>, PaintColor),
    Text(PaintPoint, SharedString, f32, PaintColor),
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PaintViewport {
    pub width: f32,
    pub height: f32,
    pub scale_factor: f32,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecordedPaintProps {
    pub recording: PaintRecording,
    #[serde(default = "default_height")]
    pub viewport_height: f32,
}
pub(super) fn default_height() -> f32 {
    240.
}
struct RecordedPaint {
    props: RecordedPaintProps,
    prepared: Arc<Vec<PreparedCommand>>,
    viewport: Rc<Cell<Option<PaintViewport>>>,
    event: Event<PaintViewport>,
}
impl NativeView for RecordedPaint {
    type Props = RecordedPaintProps;
    type Event = PaintViewport;
    fn event_name() -> &'static str {
        "viewport"
    }
    fn validate_props(props: &Self::Props) -> Result<(), String> {
        dimension(props.viewport_height)?;
        props.recording.validate()
    }
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        _: NativeChildren,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Self {
        let prepared = Arc::new(props.recording.prepare());
        Self {
            props,
            prepared,
            viewport: Rc::new(Cell::new(None)),
            event,
        }
    }
    fn update(&mut self, props: Self::Props, _: &mut Window, _: &mut Context<Self>) {
        if props.recording != self.props.recording {
            self.prepared = Arc::new(props.recording.prepare());
        }
        self.props = props;
    }
}
impl Render for RecordedPaint {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let prepared = self.prepared.clone();
        let width = self.props.recording.width;
        let height = self.props.recording.height;
        let fit = self.props.recording.fit;
        let transform = self.props.recording.transform;
        let clip = self.props.recording.clip;
        let viewport = self.viewport.clone();
        let event = self.event.clone();
        gpui::div()
            .w_full()
            .h(px(self.props.viewport_height))
            .overflow_hidden()
            .child(
                gpui::canvas(
                    move |bounds, window, _| {
                        let value = PaintViewport {
                            width: bounds.size.width.as_f32(),
                            height: bounds.size.height.as_f32(),
                            scale_factor: window.scale_factor(),
                        };
                        if viewport.replace(Some(value)) != Some(value) {
                            event.emit(value);
                        }
                    },
                    move |bounds, (), window, cx| {
                        let fit_scale = match fit {
                            PaintFit::Contain => (bounds.size.width.as_f32() / width)
                                .min(bounds.size.height.as_f32() / height)
                                .min(1.),
                            PaintFit::None => 1.,
                        };
                        if fit_scale <= 0. {
                            return;
                        }
                        let scale = fit_scale * transform.scale;
                        let origin = bounds.origin
                            + point(
                                px((bounds.size.width.as_f32() - width * fit_scale) / 2.
                                    + transform.translate_x * fit_scale),
                                px((bounds.size.height.as_f32() - height * fit_scale) / 2.
                                    + transform.translate_y * fit_scale),
                            );
                        let at = |p: PaintPoint| origin + point(px(p.x * scale), px(p.y * scale));
                        let rect = |r: PaintRect| {
                            Bounds::new(
                                at(PaintPoint { x: r.x, y: r.y }),
                                size(px(r.width * scale), px(r.height * scale)),
                            )
                        };
                        let mask = ContentMask {
                            bounds: clip.map(rect).unwrap_or(bounds).intersect(&bounds),
                        };
                        window.with_content_mask(Some(mask), |window| {
                            for command in prepared.iter() {
                                match command {
                                    PreparedCommand::Quad(r, color) => {
                                        window.paint_quad(gpui::fill(rect(*r), color.resolve(cx)))
                                    }
                                    PreparedCommand::Path(path, color) => {
                                        let mut path = path.clone();
                                        path.bounds = Bounds::new(
                                            origin + path.bounds.origin * scale,
                                            path.bounds.size.map(|p| p * scale),
                                        );
                                        for v in &mut path.vertices {
                                            v.xy_position = origin + v.xy_position * scale;
                                        }
                                        window.paint_path(path, color.resolve(cx));
                                    }
                                    PreparedCommand::Text(p, text, font_size, color) => {
                                        let font_size = px(font_size * scale);
                                        let run = TextRun {
                                            len: text.len(),
                                            font: window.text_style().font(),
                                            color: color.resolve(cx),
                                            background_color: None,
                                            underline: None,
                                            strikethrough: None,
                                        };
                                        let line = window.text_system().shape_line(
                                            text.clone(),
                                            font_size,
                                            &[run],
                                            None,
                                        );
                                        if let Err(error) = line.paint(
                                            at(*p),
                                            font_size * 1.2,
                                            TextAlign::Left,
                                            None,
                                            window,
                                            cx,
                                        ) {
                                            eprintln!("RecordedPaint text paint failed: {error}");
                                        }
                                    }
                                }
                            }
                        });
                    },
                )
                .size_full(),
            )
    }
}
pub(super) fn definition() -> ComponentDefinition {
    ComponentDefinition::view::<RecordedPaint>("RecordedPaint")
        .with_contract(include_str!("recorded_paint.rs"))
}

#[cfg(test)]
mod tests {
    use super::super::test_support::Fixture;
    use super::*;
    use gpui::{AppContext, TestAppContext};

    #[gpui::test]
    fn paint_media_replay_updates_native_geometry_clip_theme_resize_and_dpi(
        cx: &mut TestAppContext,
    ) {
        let props: RecordedPaintProps = crate::native::decode_json(br#"{
            "viewportHeight":100,"recording":{"width":100,"height":100,"fit":"none",
            "transform":{"scale":2,"translateX":10,"translateY":5},
            "clip":{"x":0,"y":0,"width":20,"height":20},
            "commands":[{"kind":"quad","bounds":{"x":0,"y":0,"width":30,"height":30},"color":{"kind":"accent"}}]}}
        "#).unwrap();
        RecordedPaint::validate_props(&props).unwrap();
        let fixture = Fixture::<RecordedPaint>::new(props, cx);
        cx.update_window(fixture.window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
            let quad = window
                .painted_quads()
                .into_iter()
                .find(|q| q.bounds.size.width.0 == 120.)
                .unwrap();
            assert_eq!(quad.bounds.origin.x.0, 420.);
            assert_eq!(quad.bounds.origin.y.0, 10.);
            assert_eq!(quad.content_mask.bounds.size.width.0, 80.);
            assert_eq!(
                quad.background,
                gpui_component::Theme::global(cx).primary.into()
            );
        })
        .unwrap();
        cx.simulate_window_scale_factor_change(fixture.window.into(), 1.);
        cx.simulate_window_resize(fixture.window.into(), size(px(300.), px(200.)));
        cx.update_window(fixture.window.into(), |_, window, cx| {
            gpui_component::Theme::change(gpui_component::ThemeMode::Dark, Some(window), cx);
            window.draw(cx).clear(cx);
            let quad = window
                .painted_quads()
                .into_iter()
                .find(|q| q.bounds.size.width.0 == 60.)
                .unwrap();
            assert_eq!(quad.bounds.origin.x.0, 110.);
            assert_eq!(quad.content_mask.bounds.size.width.0, 40.);
            assert_eq!(
                quad.background,
                gpui_component::Theme::global(cx).primary.into()
            );
        })
        .unwrap();
        fixture.update(cx, |view, window, cx| {
            let mut props = view.props.clone();
            props.recording.commands.clear();
            view.update(props, window, cx);
            cx.notify();
        });
        cx.update_window(fixture.window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
            assert!(
                !window
                    .painted_quads()
                    .iter()
                    .any(|q| q.bounds.size.width.0 == 60.)
            );
        })
        .unwrap();
    }
    #[test]
    fn recording_admission_rejects_nonfinite_geometry_and_aggregate_work() {
        let mut recording = PaintRecording {
            width: 320.,
            height: 180.,
            commands: vec![],
            fit: PaintFit::Contain,
            transform: PaintTransform::default(),
            clip: None,
        };
        recording.commands.push(PaintCommand::Text {
            origin: PaintPoint { x: 0., y: f32::NAN },
            text: "Diagram".into(),
            font_size: 14.,
            color: PaintColor::Foreground,
        });
        assert!(recording.validate().is_err());
        recording.commands.clear();
        recording.commands = vec![
            PaintCommand::Text {
                origin: PaintPoint { x: 0., y: 0. },
                text: "x".repeat(1024),
                font_size: 14.,
                color: PaintColor::Foreground,
            };
            17
        ];
        assert!(recording.validate().is_err());
        recording.commands = vec![
            PaintCommand::Quad {
                bounds: PaintRect {
                    x: 0.,
                    y: 0.,
                    width: 10.,
                    height: 10.
                },
                color: PaintColor::Accent,
            };
            MAX_COMMANDS + 1
        ];
        assert!(recording.validate().is_err());
        recording.commands = vec![PaintCommand::Path {
            points: vec![
                PaintPoint { x: 0., y: 0. },
                PaintPoint { x: 100., y: 0. },
                PaintPoint { x: 100., y: 100. },
                PaintPoint { x: 50., y: 25. },
                PaintPoint { x: 0., y: 100. },
            ],
            closed: true,
            stroke: None,
            stroke_width: 0.,
            fill: Some(PaintColor::Accent),
        }];
        assert!(
            recording.validate().is_err(),
            "concave fan fills would paint outside the intended polygon"
        );
        recording.commands = vec![PaintCommand::Path {
            points: vec![
                PaintPoint { x: 0., y: 0. },
                PaintPoint { x: 100., y: 0. },
                PaintPoint { x: 100., y: 100. },
                PaintPoint { x: 0., y: 100. },
            ],
            closed: true,
            stroke: None,
            stroke_width: 0.,
            fill: Some(PaintColor::Accent),
        }];
        recording.validate().unwrap();
    }
}
