//! Dialog popup geometry, asserted against a really-drawn frame.
//!
//! These tests live beside the contract file instead of inside it: the catalog
//! digest hashes that file's text, so a test-only edit there would force every
//! consumer to regenerate its bindings.
use gpui::{
    AppContext, Bounds, Context, IntoElement, ParentElement, Pixels, Render, Styled,
    TestAppContext, VisualTestContext, Window, div, px, size,
};
use gpui_component::Root;

/// The vendored popup names itself in the debug-bounds map.
const DIALOG_POPUP_SELECTOR: &str = "dialog-popup";
/// The bottom gap the vendored popup keeps from the window's edge.
const DIALOG_BOTTOM_GAP: Pixels = px(24.);
/// A body no window can show in full, so the popup has to clamp.
const TALL_BODY: Pixels = px(2_000.);
/// A body shorter than the popup's own title and footer.
const SHORT_BODY: Pixels = px(40.);

struct Surface;
impl Render for Surface {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The application host renders this layer; without it an open dialog
        // never reaches the screen.
        div()
            .size_full()
            .children(Root::render_dialog_layer(window, cx))
    }
}

/// Opens a window whose first layer is the component `Root`.
fn root_window(height: f32, cx: &mut TestAppContext) -> VisualTestContext {
    cx.update(gpui_component::init);
    let handle = cx.open_window(size(px(560.), px(height)), |window, cx| {
        let surface = cx.new(|_| Surface);
        Root::new(surface, window, cx)
    });
    VisualTestContext::from_window(handle.into(), cx)
}

/// Opens one dialog with a body of `body` height and reports its popup.
fn draw_dialog(visual: &mut VisualTestContext, body: Pixels) -> Bounds<Pixels> {
    visual.update(|window, cx| {
        Root::update(window, cx, |root, window, cx| {
            root.open_dialog_owned(
                move |dialog, _, _| {
                    dialog
                        .width(px(320.))
                        .title("New download")
                        .footer(div().h(px(32.)))
                        .child(div().w_full().h(body))
                },
                |_, _, _| {},
                window,
                cx,
            )
        })
    });
    visual.run_until_parked();
    visual.update(|window, cx| window.draw(cx).clear(cx));
    visual
        .debug_bounds(DIALOG_POPUP_SELECTOR)
        .expect("the dialog popup paints")
}

#[gpui::test]
fn a_tall_dialog_body_keeps_the_footer_inside_the_window(cx: &mut TestAppContext) {
    const WINDOW_HEIGHT: Pixels = px(400.);
    let mut visual = root_window(400., cx);
    let popup = draw_dialog(&mut visual, TALL_BODY);
    assert!(
        popup.size.height < TALL_BODY,
        "a {TALL_BODY:?} body must not size the popup: {popup:?}"
    );
    assert!(
        popup.bottom() <= WINDOW_HEIGHT - DIALOG_BOTTOM_GAP,
        "the popup must leave the bottom gap: {popup:?}"
    );
    assert!(
        popup.size.height > WINDOW_HEIGHT / 2.,
        "the popup must use the room it has rather than shrink to a fixed height: {popup:?}"
    );
}

#[gpui::test]
fn a_short_dialog_body_sizes_the_popup_to_its_content(cx: &mut TestAppContext) {
    let mut compact = root_window(400., cx);
    let in_short_window = draw_dialog(&mut compact, SHORT_BODY);
    let mut roomy = root_window(900., cx);
    let in_tall_window = draw_dialog(&mut roomy, SHORT_BODY);
    assert_eq!(
        in_short_window.size.height, in_tall_window.size.height,
        "the popup must follow its content, not the window it shows in"
    );
    assert!(
        in_short_window.size.height < px(360.),
        "{in_short_window:?}"
    );
}
