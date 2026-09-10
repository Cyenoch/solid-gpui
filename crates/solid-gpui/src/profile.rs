//! Optional stage attribution on the actual host path, including failed commits.

#[derive(Clone, Copy, Debug)]
pub(crate) enum Stage {
    Commit,
    #[cfg(any(test, feature = "host"))]
    Queue,
    Decode,
    Dependencies,
    Tree,
    Validate,
    Reconcile,
    Extensions,
    Render,
}

pub(crate) struct Span {
    #[cfg(any(test, feature = "frame-profile"))]
    stage: Stage,
    #[cfg(any(test, feature = "frame-profile"))]
    started: web_time::Instant,
}

impl Span {
    /// Finish a measured stage at an explicit boundary, including in uninstrumented builds.
    #[inline]
    pub(crate) fn finish(self) {}
}

pub(crate) fn span(stage: Stage) -> Span {
    #[cfg(not(any(test, feature = "frame-profile")))]
    let _ = stage;
    Span {
        #[cfg(any(test, feature = "frame-profile"))]
        stage,
        #[cfg(any(test, feature = "frame-profile"))]
        started: web_time::Instant::now(),
    }
}

#[cfg(any(test, feature = "frame-profile"))]
mod enabled {
    use super::{Span, Stage};
    use std::{cell::RefCell, time::Duration};
    use web_time::Instant;

    #[derive(Clone, Debug)]
    pub(crate) struct Samples {
        pub(crate) elapsed: [Duration; 9],
        pub(crate) count: [usize; 9],
        started: Instant,
    }

    impl Default for Samples {
        fn default() -> Self {
            Self {
                elapsed: [Duration::ZERO; 9],
                count: [0; 9],
                started: Instant::now(),
            }
        }
    }

    impl Samples {
        pub(crate) fn milliseconds(&self, stage: Stage) -> f64 {
            self.elapsed[stage as usize].as_secs_f64() * 1000.0
        }
    }

    thread_local! {
        static SAMPLES: RefCell<Samples> = RefCell::new(Samples::default());
    }

    #[cfg(test)]
    pub(crate) fn take() -> Samples {
        SAMPLES.with(|samples| std::mem::take(&mut *samples.borrow_mut()))
    }

    impl Drop for Span {
        fn drop(&mut self) {
            SAMPLES.with(|samples| {
                let mut samples = samples.borrow_mut();
                samples.elapsed[self.stage as usize] += self.started.elapsed();
                samples.count[self.stage as usize] += 1;
                // Aggregate diagnostics instead of logging on every native event.
                if cfg!(all(feature = "frame-profile", not(test)))
                    && matches!(self.stage, Stage::Commit | Stage::Render)
                    && samples.started.elapsed() >= Duration::from_millis(500)
                {
                    eprintln!(
                        "solid_commit_stages: interval_ms={} commit={:.3}ms queue={:.3}ms decode={:.3}ms dependencies={:.3}ms tree={:.3}ms validate={:.3}ms reconcile={:.3}ms extensions={:.3}ms render={:.3}ms counts={:?}",
                        samples.started.elapsed().as_millis(),
                        samples.milliseconds(Stage::Commit),
                        samples.milliseconds(Stage::Queue),
                        samples.milliseconds(Stage::Decode),
                        samples.milliseconds(Stage::Dependencies),
                        samples.milliseconds(Stage::Tree),
                        samples.milliseconds(Stage::Validate),
                        samples.milliseconds(Stage::Reconcile),
                        samples.milliseconds(Stage::Extensions),
                        samples.milliseconds(Stage::Render),
                        samples.count,
                    );
                    *samples = Samples::default();
                }
            });
        }
    }
}

#[cfg(test)]
pub(crate) use enabled::take;
