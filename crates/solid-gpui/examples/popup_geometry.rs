//! Native regression probe: cargo run -p solid-gpui --example popup_geometry.
//! Run on macOS with each monitor configuration that needs qualification.
#[cfg(target_os = "macos")]
fn main() {
    use gpui::{popup::*, *};
    struct Blank;
    impl Render for Blank {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }
    gpui_platform::application().run(|cx| {
        for display in cx.displays() {
            let parent = cx
                .open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                            point(px(100.0), px(100.0)),
                            size(px(400.0), px(300.0)),
                        ))),
                        display_id: Some(display.id()),
                        show: false,
                        focus: false,
                        ..Default::default()
                    },
                    |_, cx| cx.new(|_| Blank),
                )
                .unwrap();
            let anchor = Bounds::new(point(px(50.0), px(50.0)), size(px(100.0), px(32.0)));
            let popup = cx
                .open_window(
                    WindowOptions {
                        titlebar: None,
                        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                            Point::default(),
                            size(px(200.0), px(120.0)),
                        ))),
                        kind: WindowKind::AnchoredPopup(PopupOptions {
                            parent: parent.into(),
                            anchor_rect: anchor,
                            anchor: PopupAnchor::BottomLeft,
                            gravity: PopupGravity::BottomRight,
                            constraint_adjustment: PopupConstraintAdjustment::all(),
                            offset: point(px(0.0), px(8.0)),
                            grab: false,
                        }),
                        show: false,
                        focus: false,
                        ..Default::default()
                    },
                    |_, cx| cx.new(|_| Blank),
                )
                .unwrap();
            let owner = parent.update(cx, |_, window, _| window.bounds()).unwrap();
            let initial = popup.update(cx, |_, window, _| window.bounds()).unwrap();
            popup
                .update(cx, |_, window, _| window.reposition_popup(anchor))
                .unwrap()
                .unwrap();
            let repeated = popup.update(cx, |_, window, _| window.bounds()).unwrap();
            eprintln!(
                "display={:?} scale={} owner={owner:?} initial={initial:?} repeated={repeated:?}",
                display.id(),
                parent.update(cx, |_, w, _| w.scale_factor()).unwrap()
            );
            // Initial placement and an identical runtime anchor update must agree.
            // This catches window initialization overwriting the resolved position.
            if initial != repeated {
                eprintln!("FAIL: first popup placement differs from runtime placement");
                std::process::exit(1);
            }
            let expected_x = owner.origin.x + anchor.origin.x;
            if (initial.origin.x - expected_x).abs() > px(1.0) {
                eprintln!("FAIL: popup is detached from its parent-local anchor");
                std::process::exit(1);
            }
            popup
                .update(cx, |_, window, _| window.remove_window())
                .unwrap();
            parent
                .update(cx, |_, window, _| window.remove_window())
                .unwrap();
        }
        eprintln!("PASS: native initial placement on all attached displays");
        cx.quit();
    });
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This AppKit geometry probe requires macOS.");
}
