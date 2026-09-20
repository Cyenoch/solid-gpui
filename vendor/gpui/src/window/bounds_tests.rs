//! Regression tests for window-move invalidation (issue #2).
//!
//! Every test drives the real platform seams on the test window — the `moved`
//! and `resize` callbacks the OS would deliver — and observes root render
//! counts, painted output, and cursor resolution. A bounds notification must
//! synchronize native state and notify observers, but it must not rebuild the
//! whole view tree unless a render-relevant input actually changed.

use std::{cell::Cell, rc::Rc};

use crate::{
    AnyWindowHandle, Bounds, CursorStyle, Decorations, DisplayId, Hsla, Pixels, PlatformDisplay,
    Point, RequestFrameOptions, ScaledPixels, Subscription, TestAppContext, TestWindow, Tiling,
    Window, div, point, prelude::*, px, size,
};

const HOVER_COLOR: Hsla = Hsla {
    h: 0.72,
    s: 0.85,
    l: 0.55,
    a: 1.0,
};

const BASE_COLOR: Hsla = Hsla {
    h: 0.55,
    s: 0.1,
    l: 0.2,
    a: 1.0,
};

/// A root view counting `Render::render` invocations, optionally with a
/// 100x100 hotspot at the window's top-left that paints a hover background and
/// requests a pointing-hand cursor.
struct Probe {
    renders: Rc<Cell<usize>>,
    hotspot: bool,
    _bounds: Subscription,
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.renders.set(self.renders.get() + 1);
        let mut root = div().size_full().bg(BASE_COLOR);
        if self.hotspot {
            root = root.child(
                div()
                    .absolute()
                    .left(px(0.))
                    .top(px(0.))
                    .size(px(100.))
                    .bg(BASE_COLOR)
                    .hover(|style| style.bg(HOVER_COLOR))
                    .cursor(CursorStyle::PointingHand),
            );
        }
        root
    }
}

type ProbeWindow = (
    AnyWindowHandle,
    TestWindow,
    Rc<Cell<usize>>,
    Rc<Cell<usize>>,
    Rc<Cell<bool>>,
);

/// Opens an 800x600 window with a render-counting root and a bounds observer,
/// then delivers one initial frame like a real compositor would.
fn open_probe(cx: &mut TestAppContext, hotspot: bool) -> ProbeWindow {
    let renders = Rc::new(Cell::new(0));
    let observations = Rc::new(Cell::new(0));
    let notifies = Rc::new(Cell::new(false));
    let window = cx.open_window(size(px(800.), px(600.)), {
        let renders = renders.clone();
        let observations = observations.clone();
        let notifies = notifies.clone();
        move |window, cx| Probe {
            renders,
            hotspot,
            _bounds: cx.observe_window_bounds(window, move |_, _, cx| {
                observations.set(observations.get() + 1);
                if notifies.get() {
                    cx.notify();
                }
            }),
        }
    });
    let handle: AnyWindowHandle = window.into();
    let platform = cx.test_window(handle);
    platform.simulate_frame_request(RequestFrameOptions::default());
    assert!(
        renders.get() > 0,
        "the initial frame must be drawn before measuring deltas"
    );
    (handle, platform, renders, observations, notifies)
}

/// Whether a solid quad with the given color was painted in the last frame.
fn has_solid_quad(window: &Window, color: Hsla) -> bool {
    window.painted_quads().iter().any(|quad| {
        quad.background.tag == crate::BackgroundTag::Solid && quad.background.solid == color
    })
}

/// Whether a solid quad with the given color was painted covering the logical
/// point at the window's scale factor.
fn has_solid_quad_at(window: &Window, color: Hsla, logical: Point<Pixels>, scale: f32) -> bool {
    let scaled = point(
        ScaledPixels(logical.x.0 * scale),
        ScaledPixels(logical.y.0 * scale),
    );
    window.painted_quads().iter().any(|quad| {
        quad.background.tag == crate::BackgroundTag::Solid
            && quad.background.solid == color
            && quad.bounds.contains(&scaled)
    })
}

/// A stand-in display with a distinct id for cross-display move tests.
#[derive(Debug)]
struct SecondaryDisplay(DisplayId);

impl PlatformDisplay for SecondaryDisplay {
    fn id(&self) -> DisplayId {
        self.0
    }

    fn uuid(&self) -> anyhow::Result<uuid::Uuid> {
        Ok(uuid::Uuid::from_u128(self.0.0 as u128))
    }

    fn bounds(&self) -> Bounds<Pixels> {
        Bounds::from_corners(point(px(0.), px(0.)), point(px(3840.), px(2160.)))
    }
}

#[gpui::test]
fn passive_move_keeps_observers_without_rebuilding(cx: &mut TestAppContext) {
    let (handle, platform, renders, observations, _) = open_probe(cx, false);
    let baseline = renders.get();

    // First delivered move: the pointer re-sample sits over the titlebar,
    // outside the client area. Crossing out of the client repaints once.
    platform.simulate_move(point(px(100.), px(80.)), point(px(-12.), px(-24.)));
    assert_eq!(renders.get(), baseline + 1);

    // Further delivered moves with an unchanged, out-of-client pointer must
    // not rebuild the tree.
    for i in 1..10 {
        platform.simulate_move(
            point(px(100. + i as f32 * 10.), px(80.)),
            point(px(-12.), px(-24.)),
        );
    }
    assert_eq!(
        renders.get(),
        baseline + 1,
        "passive same-display moves must not rebuild the root"
    );
    assert_eq!(
        observations.get(),
        10,
        "every delivered move must still reach bounds observers"
    );

    cx.update_window(handle, |_, window, _| {
        assert_eq!(window.bounds().origin, point(px(190.), px(80.)));
        assert_eq!(window.mouse_position(), point(px(-12.), px(-24.)));
    })
    .unwrap();
}

#[gpui::test]
fn same_size_resize_is_inert_but_real_resize_and_scale_invalidate(cx: &mut TestAppContext) {
    let (handle, mut platform, renders, _, _) = open_probe(cx, false);
    let baseline = renders.get();

    for _ in 0..5 {
        platform.simulate_resize(size(px(800.), px(600.)));
    }
    assert_eq!(
        renders.get(),
        baseline,
        "same-size resize callbacks must not rebuild the root"
    );

    platform.simulate_resize(size(px(640.), px(480.)));
    assert_eq!(renders.get(), baseline + 1, "a real resize must repaint");
    cx.update_window(handle, |_, window, _| {
        assert_eq!(window.viewport_size(), size(px(640.), px(480.)));
    })
    .unwrap();

    platform.simulate_scale_factor_change(1.25);
    assert_eq!(renders.get(), baseline + 2, "a scale change must repaint");
    cx.update_window(handle, |_, window, _| {
        assert_eq!(window.scale_factor(), 1.25);
    })
    .unwrap();
}

#[gpui::test]
fn same_size_visual_state_changes_invalidate(cx: &mut TestAppContext) {
    let (handle, mut platform, renders, _, _) = open_probe(cx, false);
    let baseline = renders.get();

    // Every transition below is delivered through the resize callback with an
    // unchanged client size, so only the visual-state comparison can explain
    // each repaint.
    platform.simulate_maximized_change(true);
    assert_eq!(renders.get(), baseline + 1);
    cx.update_window(handle, |_, window, _| assert!(window.is_maximized()))
        .unwrap();

    platform.simulate_maximized_change(false);
    assert_eq!(renders.get(), baseline + 2);

    platform.simulate_fullscreen_change(true);
    assert_eq!(renders.get(), baseline + 3);
    cx.update_window(handle, |_, window, _| assert!(window.is_fullscreen()))
        .unwrap();

    platform.simulate_fullscreen_change(false);
    assert_eq!(renders.get(), baseline + 4);

    platform.simulate_simple_fullscreen_change(true);
    assert_eq!(renders.get(), baseline + 5);
    cx.update_window(handle, |_, window, _| {
        assert!(window.is_simple_fullscreen())
    })
    .unwrap();

    platform.simulate_simple_fullscreen_change(false);
    assert_eq!(renders.get(), baseline + 6);

    let tiled = Decorations::Client {
        tiling: Tiling::tiled(),
    };
    platform.simulate_decorations_change(tiled);
    assert_eq!(renders.get(), baseline + 7);
    cx.update_window(handle, |_, window, _| {
        assert_eq!(window.window_decorations(), tiled);
    })
    .unwrap();

    platform.simulate_decorations_change(Decorations::Server);
    assert_eq!(
        renders.get(),
        baseline + 8,
        "each same-size visual-state transition must repaint exactly once"
    );
}

#[gpui::test]
fn display_change_still_invalidates(cx: &mut TestAppContext) {
    let (_handle, platform, renders, observations, _) = open_probe(cx, false);
    let baseline = renders.get();

    platform.simulate_display_change(Rc::new(SecondaryDisplay(DisplayId(2))));
    assert_eq!(
        renders.get(),
        baseline + 1,
        "a cross-display move must repaint"
    );
    assert_eq!(observations.get(), 1);

    platform.simulate_display_change(Rc::new(SecondaryDisplay(DisplayId(3))));
    assert_eq!(renders.get(), baseline + 2);
    assert_eq!(observations.get(), 2);
}

#[gpui::test]
fn bounds_observer_notify_still_invalidates(cx: &mut TestAppContext) {
    let (_handle, platform, renders, observations, notifies) = open_probe(cx, false);
    let baseline = renders.get();
    notifies.set(true);

    // The pointer never changes, so any repaint below is observer-driven.
    for i in 0..3 {
        platform.simulate_move(point(px(40. * i as f32), px(0.)), point(px(0.), px(0.)));
    }
    assert_eq!(
        renders.get(),
        baseline + 3,
        "the move gate must not swallow observer-driven invalidation"
    );
    assert_eq!(observations.get(), 3);
}

#[gpui::test]
fn moved_pointer_over_content_updates_hover_paint_and_cursor(cx: &mut TestAppContext) {
    let (handle, platform, renders, _, _) = open_probe(cx, true);
    let baseline = renders.get();

    // Park the pointer inside the viewport but off the hotspot. The window is
    // never activated: hover must track the pointer regardless of activation.
    platform.simulate_move(point(px(500.), px(0.)), point(px(700.), px(500.)));
    assert_eq!(
        renders.get(),
        baseline + 1,
        "pointer entering the client must repaint"
    );
    cx.update_window(handle, |_, window, _| {
        assert!(
            !has_solid_quad(window, HOVER_COLOR),
            "hover style must not paint away from the hotspot"
        );
        assert_eq!(window.rendered_frame.cursor_style(window), None);
    })
    .unwrap();

    // Move the window so the pointer lands on the hotspot content.
    platform.simulate_move(point(px(540.), px(0.)), point(px(40.), px(40.)));
    assert_eq!(renders.get(), baseline + 2);
    cx.update_window(handle, |_, window, _| {
        let scale = window.scale_factor();
        assert!(
            has_solid_quad_at(window, HOVER_COLOR, point(px(40.), px(40.)), scale),
            "hover background must paint under the pointer"
        );
        assert_eq!(
            window.rendered_frame.cursor_style(window),
            Some(CursorStyle::PointingHand)
        );
    })
    .unwrap();

    // Moving the pointer within the viewport keeps repainting the new hover
    // state, even though no input event was delivered.
    platform.simulate_move(point(px(580.), px(0.)), point(px(404.), px(300.)));
    assert_eq!(renders.get(), baseline + 3);
    cx.update_window(handle, |_, window, _| {
        assert!(!has_solid_quad(window, HOVER_COLOR));
        assert_eq!(window.rendered_frame.cursor_style(window), None);
    })
    .unwrap();
}

#[gpui::test]
fn moved_pointer_outside_client_never_rebuilds_even_when_active(cx: &mut TestAppContext) {
    let (handle, platform, renders, observations, _) = open_probe(cx, false);
    let baseline = renders.get();

    // Establish the pointer outside the client area, resting on the titlebar.
    // The old sample (the window corner) was inside, so this repaints once.
    platform.simulate_move(point(px(0.), px(0.)), point(px(-12.), px(-24.)));
    let settled = renders.get();
    assert_eq!(settled, baseline + 1);

    for i in 1..=5 {
        platform.simulate_move(point(px(i as f32 * 15.), px(0.)), point(px(-12.), px(-24.)));
    }
    assert_eq!(
        renders.get(),
        settled,
        "moving the window under an out-of-client pointer must not rebuild"
    );
    assert_eq!(
        observations.get(),
        6,
        "observers must still be delivered for every move"
    );

    // macOS treats an active window as hovered; activation must not
    // reintroduce per-move rebuilds.
    platform.simulate_active_status_change(true);
    let active_baseline = renders.get();
    for i in 1..=5 {
        platform.simulate_move(
            point(px(75. + i as f32 * 15.), px(0.)),
            point(px(-12.), px(-24.)),
        );
    }
    assert_eq!(
        renders.get(),
        active_baseline,
        "activation must not reintroduce per-move rebuilds"
    );
    cx.update_window(handle, |_, window, _| assert!(window.is_window_active()))
        .unwrap();
}

#[gpui::test]
fn forced_frame_request_rebuilds_clean_tree(cx: &mut TestAppContext) {
    let (_handle, platform, renders, _, _) = open_probe(cx, false);
    let baseline = renders.get();

    // A passive move leaves the tree clean.
    platform.simulate_move(point(px(60.), px(0.)), point(px(-12.), px(-24.)));
    let settled = renders.get();
    assert_eq!(settled, baseline + 1);

    // Surface recovery must still force a full rebuild of the clean tree.
    platform.simulate_frame_request(RequestFrameOptions {
        require_presentation: true,
        force_render: true,
    });
    assert_eq!(
        renders.get(),
        settled + 1,
        "forced rendering must rebuild a clean tree"
    );
}
