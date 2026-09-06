//! Opt-in native measurements. No timer, redraw, or per-input logging is added.
use gpui::profiler::{FrameDurationSnapshot, InputLatencySnapshot};
use gpui::{Pixels, Size, Window};
use std::time::{Duration, Instant};

pub(super) struct FrameProfile {
    since: Instant,
    size: Size<Pixels>,
    frames: FrameDurationSnapshot,
    inputs: InputLatencySnapshot,
}

impl FrameProfile {
    pub(super) fn new(window: &Window) -> Self {
        Self {
            since: Instant::now(),
            size: window.viewport_size(),
            frames: window.frame_duration_snapshot(),
            inputs: window.input_latency_snapshot(),
        }
    }

    pub(super) fn record(&mut self, window: &Window) {
        // Discard intervals spanning a resize: narrow and wide frames must not
        // be mixed into the same histogram. Read before draw, so the current
        // unfinished frame is excluded from all metrics consistently.
        if self.size != window.viewport_size() {
            *self = Self::new(window);
            return;
        }
        if self.since.elapsed() < Duration::from_secs(1) {
            return;
        }
        let next = Self::new(window);
        let mut frames = next.frames.clone();
        let mut inputs = next.inputs.clone();
        frames
            .draw_duration_histogram
            .subtract(&self.frames.draw_duration_histogram)
            .expect("monotonic draw samples");
        frames
            .dirty_to_present_histogram
            .subtract(&self.frames.dirty_to_present_histogram)
            .expect("monotonic presentation samples");
        inputs
            .latency_histogram
            .subtract(&self.inputs.latency_histogram)
            .expect("monotonic input samples");
        if !frames.draw_duration_histogram.is_empty() {
            eprintln!(
                "[solid-gpui-frame-profile] viewport={}x{} interval_ms={} draws={} draw_p50_ms={:.3} draw_p95_ms={:.3} invalidation_to_present_p95_ms={:.3} inputs={} input_to_present_p95_ms={:.3} active={}",
                f32::from(self.size.width),
                f32::from(self.size.height),
                self.since.elapsed().as_millis(),
                frames.draw_duration_histogram.len(),
                frames.draw_duration_histogram.value_at_quantile(0.5) as f64 / 1e6,
                frames.draw_duration_histogram.value_at_quantile(0.95) as f64 / 1e6,
                frames.dirty_to_present_histogram.value_at_quantile(0.95) as f64 / 1e6,
                inputs.latency_histogram.len(),
                inputs.latency_histogram.value_at_quantile(0.95) as f64 / 1e6,
                window.is_window_active(),
            );
        }
        *self = next;
    }
}
