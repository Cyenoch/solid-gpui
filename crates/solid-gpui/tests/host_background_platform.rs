//! Native macOS background lifecycle regression (requires a desktop session).
//! This checks AppKit configuration and ownership, not compositor blur pixels.

#[cfg(target_os = "macos")]
fn main() {
    use gpui::{prelude::*, *};
    use objc2::MainThreadMarker;
    use objc2_app_kit::{
        NSApplication, NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState,
        NSVisualEffectView, NSWindow,
    };
    use std::time::Duration;

    struct Background;
    impl Render for Background {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full()
        }
    }

    fn check(native: &NSWindow, appearance: WindowBackgroundAppearance) {
        let content = native.contentView().unwrap();
        let children = content.subviews();
        let effects: Vec<_> = children
            .iter()
            .filter_map(|view| view.downcast::<NSVisualEffectView>().ok())
            .collect();
        assert_eq!(
            native.isOpaque(),
            appearance == WindowBackgroundAppearance::Opaque
        );
        if appearance == WindowBackgroundAppearance::Blurred {
            assert_eq!(
                effects.len(),
                1,
                "one owned effect view, including repeated enable"
            );
            let effect = &effects[0];
            assert_eq!(effect.material(), NSVisualEffectMaterial::Sidebar);
            assert_eq!(
                effect.blendingMode(),
                NSVisualEffectBlendingMode::BehindWindow
            );
            assert_eq!(effect.state(), NSVisualEffectState::Active);
            assert_eq!(
                effect.frame(),
                content.bounds(),
                "effect covers resized content"
            );
            assert!(
                children
                    .objectAtIndex(0)
                    .downcast::<NSVisualEffectView>()
                    .is_ok()
            );
        } else {
            assert!(
                effects.is_empty(),
                "disabling blur must remove the effect view"
            );
        }
    }

    Application::with_platform(gpui_platform::current_platform(false)).run(|cx| {
        let window = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                    point(px(100.), px(100.)), size(px(640.), px(480.)),
                ))),
                window_background: WindowBackgroundAppearance::Blurred,
                ..Default::default()
            },
            |_, cx| cx.new(|_| Background),
        ).unwrap();
        let app = NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
        let native = app.windows().objectAtIndex(0);
        check(&native, WindowBackgroundAppearance::Blurred);
        cx.spawn(async move |cx| {
            for appearance in [
                WindowBackgroundAppearance::Blurred,
                WindowBackgroundAppearance::Blurred,
                WindowBackgroundAppearance::Opaque,
                WindowBackgroundAppearance::Blurred,
                WindowBackgroundAppearance::Transparent,
                WindowBackgroundAppearance::Blurred,
            ] {
                window.update(cx, |_, window, _| window.set_background_appearance(appearance)).unwrap();
                cx.background_executor().timer(Duration::from_millis(50)).await;
                check(&native, appearance);
            }
            for (width, height) in [(800., 600.), (480., 320.), (640., 480.)] {
                window.update(cx, |_, window, _| window.resize(size(px(width), px(height)))).unwrap();
                cx.background_executor().timer(Duration::from_millis(50)).await;
                check(&native, WindowBackgroundAppearance::Blurred);
            }
            eprintln!("Native background lifecycle passed: initial blur, repeated enable, opaque/transparent transitions, resize");
            cx.update(|cx| cx.quit());
        }).detach();
    });
}

// AppKit must run on the process main thread, not a libtest worker.
#[cfg(not(target_os = "macos"))]
fn main() {}
