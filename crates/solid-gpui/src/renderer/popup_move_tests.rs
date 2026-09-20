//! Passive owner-move popup reconciliation scheduling.
//!
//! While a popup observer is installed, window bounds changes must keep
//! reconciling the popup without repainting the whole root for position-only
//! moves. These tests drive the real bounds-observer path and count actual
//! `Render::render` invocations through the profile stage samples instead of
//! asserting on internal flags.
use super::*;
use crate::protocol::{Node, Snapshot};
use crate::tree::KIND_VIEW;
use gpui::{AnyWindowHandle, App, AppContext as _, TestAppContext, WindowHandle, point, px, size};
use std::cell::RefCell;

fn anchor_snapshot() -> Snapshot {
    let mut anchor = Node::new(2, 1, 0, KIND_VIEW);
    anchor.style = Some(Style {
        width: Some(120.0),
        height: Some(32.0),
        ..Default::default()
    });
    Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), anchor])
}

fn open_anchor_window(
    cx: &mut TestAppContext,
) -> (WindowHandle<SolidRoot>, gpui::Entity<SolidRoot>) {
    let runtime = crate::transport::InMemoryAdapter::new();
    let window = cx.open_window(size(px(800.0), px(600.0)), {
        let runtime = runtime.clone();
        move |_, _| SolidRoot::new(runtime)
    });
    let root = window.root(cx).expect("popup probe root");
    let payload = anchor_snapshot().encode().expect("encode anchor fixture");
    root.update(cx, |root, cx| root.apply_payload(&payload, cx))
        .expect("apply anchor fixture");
    // Register the anchor so the next paint records its rendered bounds.
    root.update(cx, |root, _| {
        root.popup_anchors.insert(2);
    });
    cx.simulate_window_move(
        window.into(),
        point(px(0.0), px(0.0)),
        point(px(-12.0), px(-24.0)),
    );
    (window, root)
}

type PopupProbe = (
    Rc<Cell<u32>>,
    Rc<RefCell<Vec<Option<gpui::Bounds<gpui::Pixels>>>>>,
);

/// Install a popup observer that records every reconcile with the anchor
/// bounds it would consume. A reconcile observing missing bounds is the
/// wrong-close failure mode this suite guards against.
fn install_probe(root: &gpui::Entity<SolidRoot>, cx: &mut TestAppContext) -> PopupProbe {
    let calls = Rc::new(Cell::new(0u32));
    let anchors = Rc::new(RefCell::new(Vec::new()));
    let probe_calls = calls.clone();
    let probe_anchors = anchors.clone();
    let entity = root.downgrade();
    root.update(cx, |root, _| {
        root.popup_observer = Some(Rc::new(move |app: &mut App| {
            probe_calls.set(probe_calls.get() + 1);
            let _ = entity.update(app, |root, _| {
                probe_anchors.borrow_mut().push(root.popup_anchor(2));
            });
        }));
    });
    (calls, anchors)
}

fn draw_once(window: &WindowHandle<SolidRoot>, cx: &mut TestAppContext) {
    cx.update_window(AnyWindowHandle::from(*window), |_, window, cx| {
        window.draw(cx).clear(cx);
    })
    .expect("draw popup probe window");
}

#[gpui::test]
fn owner_move_without_painted_anchors_falls_back_to_the_render_path(cx: &mut TestAppContext) {
    let (window, root) = open_anchor_window(cx);
    draw_once(&window, cx);
    let painted_anchor = root
        .read_with(cx, |root, _| root.popup_anchor(2))
        .expect("painted anchor bounds");
    let (calls, anchors) = install_probe(&root, cx);
    let _ = profile::take();
    // Model the popup-opened-before-first-paint state: no painted frame.
    root.update(cx, |root, _| {
        root.last_painted_viewport = None;
        root.last_painted_revision = None;
        root.rendered_bounds.borrow_mut().clear();
    });

    cx.simulate_window_move(
        window.into(),
        point(px(40.0), px(30.0)),
        point(px(-12.0), px(-24.0)),
    );
    cx.run_until_parked();

    let samples = profile::take();
    assert_eq!(
        samples.count[profile::Stage::Render as usize],
        1,
        "a move without painted anchors must fall back to one render-path reconcile"
    );
    assert_eq!(calls.get(), 1, "the render path must still reconcile");
    assert_eq!(
        anchors.borrow().as_slice(),
        &[Some(painted_anchor)],
        "the fallback reconcile must see repainted anchor bounds, never missing ones"
    );
}
