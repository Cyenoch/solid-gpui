use gpui::{prelude::*, *};
use std::{cell::Cell, rc::Rc, time::Duration};

struct Probe {
    renders: Rc<Cell<usize>>,
    observations: Rc<Cell<usize>>,
    count: usize,
    _bounds: Subscription,
}

impl Render for Probe {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.renders.set(self.renders.get() + 1);
        div()
            .size_full()
            .flex()
            .flex_col()
            .p_4()
            .gap_3()
            .bg(rgb(0x18202c))
            .text_color(rgb(0xf0f3f8))
            .child(format!("Counter: {} | viewport: {:?}", self.count, window.viewport_size()))
            .child(
                div()
                    .id("increment")
                    .h(px(40.))
                    .p_2()
                    .bg(rgb(0x334866))
                    .hover(|style| style.bg(rgb(0x2f8d72)))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.count += 1;
                        cx.notify();
                    }))
                    .child("Increment — green while hovered"),
            )
            .child(
                div()
                    .id("rows")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .children((0..500).map(|index| {
                        div().h(px(24.)).child(format!("Retained content row {}", index + 1))
                    })),
            )
    }
}

fn main() {
    let legacy = std::env::args().any(|arg| arg == "--legacy-refresh");
    let storm = std::env::args().any(|arg| arg == "--storm");
    Application::with_platform(Rc::new(gpui_macos::MacPlatform::new(false))).run(move |cx| {
        let renders = Rc::new(Cell::new(0));
        let observations = Rc::new(Cell::new(0));
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                        point(px(220.), px(180.)),
                        size(px(800.), px(600.)),
                    ))),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Solid GPUI Window Movement Probe".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| {
                    let _bounds = cx.observe_window_bounds(window, move |this: &mut Probe, window, _| {
                        this.observations.set(this.observations.get() + 1);
                        if legacy {
                            window.refresh();
                        }
                    });
                    Probe { renders, observations, count: 0, _bounds }
                }),
            )
            .unwrap();
        cx.activate(true);
        eprintln!("READY mode={}", if legacy { "forced-refresh-reference" } else { "candidate" });
        #[cfg(target_os = "macos")]
        if storm {
            cx.spawn(async move |cx| {
                cx.background_executor().timer(Duration::from_secs(10)).await;
                let app = objc2_app_kit::NSApplication::sharedApplication(objc2::MainThreadMarker::new().unwrap());
                let native = app.windows().objectAtIndex(0);
                let origin = native.frame().origin;
                let before = window.update(cx, |root, _, _| (root.renders.get(), root.observations.get())).unwrap();
                for index in 0..120 {
                    native.setFrameOrigin(objc2_foundation::NSPoint::new(
                        origin.x + (index as f64 / 8.0).sin() * 60.0,
                        origin.y,
                    ));
                    cx.background_executor().timer(Duration::from_millis(16)).await;
                }
                native.setFrameOrigin(origin);
                cx.background_executor().timer(Duration::from_millis(250)).await;
                window.update(cx, |root, window, _| {
                    eprintln!("STORM mode={} renders={} callbacks={} origin={:?} viewport={:?}",
                        if legacy { "forced-refresh-reference" } else { "candidate" },
                        root.renders.get() - before.0,
                        root.observations.get() - before.1,
                        window.bounds().origin,
                        window.viewport_size(),
                    );
                }).unwrap();
                cx.update(|cx| cx.quit());
            }).detach();
        }
        cx.spawn(async move |cx| {
            for _ in 0..1800 {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                if window.update(cx, |root, window, _| {
                    eprintln!("SAMPLE mode={} renders={} callbacks={} counter={} origin={:?} viewport={:?} scale={}",
                        if legacy { "forced-refresh-reference" } else { "candidate" },
                        root.renders.get(), root.observations.get(), root.count,
                        window.bounds().origin, window.viewport_size(), window.scale_factor(),
                    );
                }).is_err() {
                    break;
                }
            }
            cx.update(|cx| cx.quit());
        }).detach();
    });
}
