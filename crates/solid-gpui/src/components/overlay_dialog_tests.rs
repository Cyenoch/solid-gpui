//! Dialog popup geometry, asserted against a really-drawn frame.
//!
//! These tests live beside the contract file instead of inside it: the catalog
//! digest hashes that file's text, so a test-only edit there would force every
//! consumer to regenerate its bindings.
use gpui::{
    AppContext, Bounds, Context, Entity, IntoElement, ParentElement, Pixels, Render, Styled,
    TestAppContext, VisualTestContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use gpui_component::Root;

use crate::HostProperties;
use crate::components::host::ComponentHost;
use crate::host::HostProfile;
use crate::protocol::{ExtensionField, ExtensionProperties, ExtensionValue};
use crate::{
    DecodedMessage, InMemoryAdapter, KIND_EXTENSION, KIND_VIEW, Node, Patch, PatchOperation,
    Snapshot, SolidRoot, Style, UPDATE_PROPERTIES, UPDATE_STYLE,
};
use std::sync::Arc;

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

/// The catalog `Dialog`, driven through the extension contract the way an app drives it.
fn dialog_properties(show_footer: bool) -> HostProperties {
    let props =
        format!(r#"{{"open":true,"title":"Confirm","width":320,"showFooter":{show_footer}}}"#);
    HostProperties::Extension(ExtensionProperties {
        provider_id: super::native_module().id(),
        catalog_digest: super::native_module().digest(),
        entry_id: super::native_module()
            .component_id("Dialog")
            .expect("Dialog is in the catalog"),
        entry_version: 1,
        fields: vec![ExtensionField {
            id: 1,
            value: ExtensionValue::Bytes(props.into_bytes()),
        }],
        event_ids: Arc::from([]),
    })
}

/// One open `Dialog` in a real provider window: one group View per declared slot
/// plus the default content group, with a body that sizes the popup to its content.
fn open_dialog(
    cx: &mut TestAppContext,
    footer: Option<f32>,
) -> (VisualTestContext, Entity<SolidRoot>) {
    let mut profile = ComponentHost::default();
    let runtime = InMemoryAdapter::new();
    let extensions = profile.extension_registry();
    let (window, root) = cx.update(|app| {
        profile.initialize(app);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(560.), px(400.)),
                app,
            ))),
            ..Default::default()
        };
        profile
            .open_window(options, runtime.clone(), extensions, app)
            .expect("provider window opens")
    });
    let mut visual = VisualTestContext::from_window(window, cx);
    let mut dialog = Node::new(2, 1, 0, KIND_EXTENSION);
    dialog.host_properties = Some(dialog_properties(false));
    let mut body = Node::new(6, 3, 0, KIND_VIEW);
    body.style = Some(Style {
        height: Some(120.),
        ..Default::default()
    });
    let mut nodes = vec![
        Node::new(1, 0, 0, KIND_VIEW),
        dialog,
        Node::new(3, 2, 0, KIND_VIEW),
        Node::new(4, 2, 1, KIND_VIEW),
        Node::new(5, 2, 2, KIND_VIEW),
        body,
    ];
    if let Some(height) = footer {
        let mut custom = Node::new(7, 5, 0, KIND_VIEW);
        custom.style = Some(Style {
            height: Some(height),
            ..Default::default()
        });
        nodes.push(custom);
    }
    apply(
        &mut visual,
        &root,
        DecodedMessage::Snapshot(Snapshot::new(1, 1, 0, 1, nodes)),
    );
    (visual, root)
}

fn apply(visual: &mut VisualTestContext, root: &Entity<SolidRoot>, message: DecodedMessage) {
    visual.update(|window, cx| {
        root.update(cx, |root, cx| {
            root.apply_decoded_message_in_window(message, window, cx)
                .expect("the message applies")
        })
    });
}

/// Publish a new `showFooter` value on the open dialog and measure the frame it draws.
fn show_footer_height(
    visual: &mut VisualTestContext,
    root: &Entity<SolidRoot>,
    revision: u32,
    show_footer: bool,
) -> Pixels {
    apply(
        visual,
        root,
        DecodedMessage::Patch(Patch::new(
            1,
            1,
            revision - 1,
            revision,
            vec![PatchOperation::Update {
                id: 2,
                mask: UPDATE_PROPERTIES,
                style: None,
                text: None,
                listener_id: 0,
                host_properties: Some(dialog_properties(show_footer)),
                accessibility: None,
                focusable: false,
                selectable: false,
                tooltip: None,
                accepts_pointer_move: false,
            }],
        )),
    );
    popup_height(visual)
}

fn popup_height(visual: &mut VisualTestContext) -> Pixels {
    visual.run_until_parked();
    visual.update(|window, cx| window.draw(cx).clear(cx));
    visual
        .debug_bounds(DIALOG_POPUP_SELECTOR)
        .expect("the dialog popup paints")
        .size
        .height
}

/// `showFooter: false` drops the native OK footer and keeps a custom footer
/// slot, which keeps replacing the default footer while `showFooter` is true.
#[gpui::test]
fn a_dialog_drops_its_default_footer_without_dropping_a_custom_one(cx: &mut TestAppContext) {
    let (mut plain, plain_root) = open_dialog(cx, None);
    let suppressed = popup_height(&mut plain);
    let with_default = show_footer_height(&mut plain, &plain_root, 2, true);
    assert!(
        with_default > suppressed,
        "the native footer must occupy the popup until `showFooter` removes it: \
         {with_default:?} vs {suppressed:?}"
    );

    let (mut slot, slot_root) = open_dialog(cx, Some(60.));
    let custom = popup_height(&mut slot);
    assert!(
        custom > suppressed,
        "a custom footer slot must still paint while the default footer is disabled: \
         {custom:?} vs {suppressed:?}"
    );
    let custom_with_default = show_footer_height(&mut slot, &slot_root, 2, true);
    assert_eq!(
        custom_with_default, custom,
        "a custom footer slot replaces the default footer instead of joining it"
    );
}
