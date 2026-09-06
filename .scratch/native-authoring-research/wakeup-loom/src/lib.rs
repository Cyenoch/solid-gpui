#[cfg(test)]
mod tests {
    use loom::sync::Arc;
    use loom::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    struct State {
        flags: AtomicU32,
        scheduled: AtomicBool,
    }

    fn model(acquire_clear: bool) {
        loom::model(move || {
            // The JS task has consumed prior flags and is finishing an empty
            // drain. A host producer concurrently publishes one new frame.
            let state = Arc::new(State {
                flags: AtomicU32::new(0),
                scheduled: AtomicBool::new(true),
            });
            let producer_state = Arc::clone(&state);
            let producer = loom::thread::spawn(move || {
                producer_state.flags.fetch_or(1, Ordering::AcqRel);
                !producer_state.scheduled.swap(true, Ordering::AcqRel)
            });

            if acquire_clear {
                state.scheduled.swap(false, Ordering::AcqRel);
            } else {
                state.scheduled.store(false, Ordering::Release);
            }
            let consumer_posts = state.flags.load(Ordering::Acquire) != 0
                && !state.scheduled.swap(true, Ordering::AcqRel);
            let producer_posts = producer.join().unwrap();
            assert!(producer_posts || consumer_posts, "pending frame has no scheduled task");
        });
    }

    #[test]
    #[should_panic(expected = "pending frame has no scheduled task")]
    fn release_store_has_a_lost_wakeup_execution() {
        model(false);
    }

    #[test]
    fn acquiring_clear_covers_every_explored_execution() {
        model(true);
    }
}
