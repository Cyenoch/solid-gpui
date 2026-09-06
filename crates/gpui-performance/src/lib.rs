//! Live window draw-rate monitor for any GPUI application.
//! Passive active-draw cadence: no timer or monitor-triggered redraws.
//! Idle gaps start a new measurement segment and are not plotted.
use gpui::{
    Context, IntoElement, ParentElement, PathBuilder, Render, Styled, Window, canvas, div, point,
    px, rgba,
};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

const SAMPLE_INTERVAL: Duration = Duration::from_millis(250);
const HISTORY: Duration = Duration::from_secs(15);

// Without a platform demand signal, a gap of 500 ms ends an active burst.
const IDLE_GAP: Duration = Duration::from_millis(500);

struct Samples {
    since: Instant,
    last_draw: Option<Instant>,
    intervals: u64,
    segment: u64,
    values: VecDeque<(Instant, f64, u64)>,
}

impl Samples {
    fn new(now: Instant) -> Self {
        Self {
            since: now,
            last_draw: None,
            intervals: 0,
            segment: 0,
            values: VecDeque::new(),
        }
    }

    fn record(&mut self, now: Instant) {
        while self
            .values
            .front()
            .is_some_and(|(time, _, _)| now.duration_since(*time) > HISTORY)
        {
            self.values.pop_front();
        }
        let previous = self.last_draw.replace(now);
        if previous.is_none_or(|last| now.duration_since(last) >= IDLE_GAP) {
            self.since = now;
            self.intervals = 0;
            self.segment += 1;
            return;
        }
        self.intervals += 1;
        let elapsed = now.duration_since(self.since);
        if elapsed >= SAMPLE_INTERVAL {
            self.values.push_back((
                now,
                self.intervals as f64 / elapsed.as_secs_f64(),
                self.segment,
            ));
            self.since = now;
            self.intervals = 0;
        }
    }
}

/// Position within the containing window-sized element.
#[derive(Clone, Copy, Debug, Default)]
pub enum MonitorCorner {
    TopLeft,
    #[default]
    TopRight,
}

/// Retained passive FPS view. Does not request frames while idle.
pub struct PerformanceMonitor {
    samples: Samples,
    corner: MonitorCorner,
}

impl PerformanceMonitor {
    /// Create once with `cx.new`, then mount the entity as a window overlay.
    /// Measures draw-to-draw cadence in active bursts, including slow frames.
    pub fn new(corner: MonitorCorner, cx: &mut Context<Self>) -> Self {
        Self {
            samples: Samples::new(cx.background_executor().now()),
            corner,
        }
    }
}

impl Render for PerformanceMonitor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.samples.record(cx.background_executor().now());
        let label = self.samples.values.back().map_or_else(
            || "FPS  —".to_owned(),
            |(_, fps, _)| format!("FPS  {fps:.0}"),
        );
        let values = self.samples.values.clone();
        let end = self.samples.last_draw.unwrap_or(self.samples.since);
        let ceiling =
            (values.iter().map(|(_, fps, _)| *fps).fold(120.0, f64::max) / 60.0).ceil() * 60.0;
        // Absolute and without listeners/occlusion: no layout participation or
        // hitbox over the controls beneath the translucent panel.
        let panel = div().absolute();
        let panel = match self.corner {
            MonitorCorner::TopLeft => panel.left(px(12.)),
            MonitorCorner::TopRight => panel.right(px(56.)),
        };
        panel
            .top(px(8.))
            .w(px(176.))
            .h(px(68.))
            .p(px(8.))
            .rounded(px(7.))
            .bg(rgba(0x10191fbb))
            .text_color(rgba(0x9debcaff))
            .text_size(px(12.))
            .child(div().h(px(18.)).child(label))
            .child(
                canvas(
                    move |_, _, _| (),
                    move |bounds, _, window, _| {
                        let position = |age: f64, fps: f64| {
                            point(
                                bounds.right()
                                    - bounds.size.width * (age / HISTORY.as_secs_f64()) as f32,
                                bounds.bottom() - bounds.size.height * (fps / ceiling) as f32,
                            )
                        };
                        let mut grid = PathBuilder::stroke(px(1.));
                        for level in [0.0, ceiling / 2.0, ceiling] {
                            grid.move_to(position(HISTORY.as_secs_f64(), level));
                            grid.line_to(position(0., level));
                        }
                        if let Ok(path) = grid.build() {
                            window.paint_path(path, rgba(0xffffff22));
                        }
                        if values.len() >= 2 {
                            let mut curve = PathBuilder::stroke(px(1.5));
                            for (index, (time, fps, segment)) in values.iter().enumerate() {
                                let point = position(end.duration_since(*time).as_secs_f64(), *fps);
                                if index == 0 || values[index - 1].2 != *segment {
                                    curve.move_to(point);
                                } else {
                                    curve.line_to(point);
                                }
                            }
                            if let Ok(path) = curve.build() {
                                window.paint_path(path, rgba(0x82e6baff));
                            }
                        }
                    },
                )
                .w_full()
                .h(px(30.)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_cadence_excludes_idle_and_keeps_slow_frames() {
        let start = Instant::now();
        let mut samples = Samples::new(start);
        for ms in (0..=500).step_by(10) {
            samples.record(start + Duration::from_millis(ms));
        }
        assert_eq!(samples.values.back().unwrap().1, 100.);
        // A real 250 ms slow frame inside a burst must not be discarded.
        samples.record(start + Duration::from_millis(750));
        assert_eq!(samples.values.back().unwrap().1, 4.);
        let previous_segment = samples.segment;
        let count = samples.values.len();
        samples.record(start + Duration::from_secs(5));
        assert_eq!(samples.values.len(), count, "idle adds no low-FPS point");
        for ms in (5020..=5260).step_by(20) {
            samples.record(start + Duration::from_millis(ms));
        }
        assert_eq!(samples.values.back().unwrap().1, 50.);
        assert_ne!(samples.values.back().unwrap().2, previous_segment);
        samples.record(start + Duration::from_secs(30));
        assert!(samples.values.is_empty(), "history remains bounded");
    }
}
