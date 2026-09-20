use gpui::{div, px, size, AppContext, Context, IntoElement, Render, RequestFrameOptions, Subscription, TestAppContext, Window};
use std::{cell::Cell, rc::Rc};

struct Probe {
    renders: Rc<Cell<usize>>,
    _bounds: Subscription,
}
impl Render for Probe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.renders.set(self.renders.get() + 1);
        div()
    }
}

#[test]
fn investigate_bounds_invalidation() {
    let mut cx = TestAppContext::single();
    let renders = Rc::new(Cell::new(0));
    let observations = Rc::new(Cell::new(0));
    let notify = Rc::new(Cell::new(false));
    let window = cx.open_window(size(px(800.), px(600.)), {
        let renders = renders.clone();
        let observations = observations.clone();
        let notify = notify.clone();
        move |window, cx| Probe {
            renders,
            _bounds: cx.observe_window_bounds(window, move |_, _, cx| {
                observations.set(observations.get() + 1);
                if notify.get() { cx.notify(); }
            }),
        }
    });
    let mut platform = cx.test_window(window.into());
    platform.simulate_frame_request(RequestFrameOptions::default());
    let mut last = renders.get();
    let baseline = last;
    for _ in 0..60 {
        platform.simulate_frame_request(RequestFrameOptions { require_presentation: true, force_render: false });
    }
    println!("clean_present_requests=60 root_renders={}", renders.get() - last);
    assert_eq!(renders.get(), baseline, "presentation must not require rebuilding a clean tree");
    last = renders.get();
    for _ in 0..60 {
        cx.update_window(window.into(), |_, window, cx| window.bounds_changed(cx)).unwrap();
        platform.simulate_frame_request(RequestFrameOptions::default());
    }
    println!("unchanged_bounds_callbacks=60 root_renders={} observations={}", renders.get() - last, observations.get());
    assert_eq!(observations.get(), 60);
    let unchanged_renders = renders.get() - last;
    last = renders.get();
    for _ in 0..60 { platform.simulate_resize(size(px(800.), px(600.))); }
    platform.simulate_frame_request(RequestFrameOptions::default());
    println!("coalesced_same_size_callbacks=60 root_renders={}", renders.get() - last);
    last = renders.get();
    platform.simulate_resize(size(px(640.), px(480.)));
    platform.simulate_frame_request(RequestFrameOptions::default());
    println!("actual_resize=1 root_renders={}", renders.get() - last);
    assert!(renders.get() > last);
    last = renders.get();
    platform.simulate_scale_factor_change(1.25);
    platform.simulate_frame_request(RequestFrameOptions::default());
    println!("scale_change=1 root_renders={}", renders.get() - last);
    assert!(renders.get() > last);
    last = renders.get();
    notify.set(true);
    cx.update_window(window.into(), |_, window, cx| window.bounds_changed(cx)).unwrap();
    platform.simulate_frame_request(RequestFrameOptions::default());
    println!("notifying_observer=1 root_renders={}", renders.get() - last);
    assert!(renders.get() > last);
    last = renders.get();
    platform.simulate_frame_request(RequestFrameOptions { require_presentation: true, force_render: true });
    println!("forced_recovery=1 root_renders={}", renders.get() - last);
    assert!(renders.get() > last);
    if std::env::var_os("BOUNDS_PROBE_EXPECT_CLEAN").is_some() {
        assert_eq!(unchanged_renders, 0, "unchanged bounds must not rebuild a static tree");
    }
}
