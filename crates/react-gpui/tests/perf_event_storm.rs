use react_gpui::{Event, MAX_FRAME_LENGTH, SCROLL_DELTA_PIXELS};
use std::hint::black_box;
use std::time::{Duration, Instant};

const EVENT_COUNT: u32 = 10_000;
const ENCODE_BUDGET: Duration = Duration::from_secs(2);

fn event(kind: &str, sequence: u32) -> Event {
    match kind {
        "scroll" => Event::scroll(
            7,
            3,
            1,
            sequence,
            2,
            11,
            SCROLL_DELTA_PIXELS,
            1.0,
            2.0,
            3.0,
            4.0,
            Vec::new(),
        ),
        "drag-over" => Event::drag_over(7, 3, 1, sequence, 2, 11, "card:42".to_owned()),
        "visible-range" => Event::visible_range(7, 3, 1, sequence, 2, 11, 12, 20),
        "layout" => Event::layout(7, 3, 1, sequence, 2, 11, 12.5, -3.25, 100.0, 48.75),
        "pointer-move" => Event::pointer_move(
            7,
            3,
            1,
            sequence,
            2,
            11,
            320.0,
            240.0,
            vec!["shift".to_owned()],
        ),
        _ => panic!("unknown event kind"),
    }
}

#[test]
fn native_event_frame_sizes_and_encode_budget() {
    for kind in [
        "scroll",
        "drag-over",
        "visible-range",
        "layout",
        "pointer-move",
    ] {
        let sample = event(kind, 1).encode().expect("encode sample event");
        let started = Instant::now();
        let mut bytes = 0usize;
        for sequence in 1..=EVENT_COUNT {
            bytes += black_box(event(kind, sequence).encode().expect("encode event")).len();
        }
        let elapsed = started.elapsed();
        let payload_bytes = sample.len();
        let frame_bytes = payload_bytes + 4;
        eprintln!(
            "perf_event_storm: rust kind={kind} payload_bytes={payload_bytes} frame_bytes={frame_bytes} events={EVENT_COUNT} payload_bytes_total={bytes} encode={:.3}ms payload_bytes/s={:.0}",
            elapsed.as_secs_f64() * 1_000.0,
            (bytes as f64 * 1_000.0) / elapsed.as_secs_f64(),
        );
        assert!(frame_bytes <= MAX_FRAME_LENGTH);
        assert!(
            elapsed < ENCODE_BUDGET,
            "{kind} encoding exceeded smoke budget"
        );
    }
}
