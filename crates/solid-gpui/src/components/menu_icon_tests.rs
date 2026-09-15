//! Menu item icon size, asserted against a really-drawn frame.
//!
//! These tests live beside the contract file instead of inside it: the catalog
//! digest hashes `menus.rs`, so a test-only edit there would force every
//! consumer to regenerate its bindings.
use crate::components::host::ComponentHost;
use crate::host::HostProfile;
use crate::protocol::{ExtensionField, ExtensionProperties, ExtensionValue};
use crate::{HostProperties, InMemoryAdapter, Node, Snapshot};
use gpui::{
    Bounds, Pixels, TestAppContext, VisualTestContext, WindowBounds, WindowOptions, px, size,
};
use std::sync::Arc;

/// The vendored popup registers its leading item icon under this id.
const MENU_ICON_SELECTOR: &str = "menu-icon";
/// The size step it draws that icon at (`Size::Medium`).
const MENU_ICON_SIZE: Pixels = px(16.);
/// One menu with one icon item, exactly as the application sends it.
const POPUP_MENU_PROPS: &str =
    r#"{"menu":{"items":[{"kind":"item","id":"copy","label":"复制链接","icon":"lucide:copy"}]}}"#;

fn popup_menu_snapshot() -> Snapshot {
    let mut menu = Node::new(2, 1, 0, crate::KIND_EXTENSION);
    menu.host_properties = Some(HostProperties::Extension(ExtensionProperties {
        provider_id: super::native_module().id(),
        catalog_digest: super::native_module().digest(),
        entry_id: super::native_module()
            .component_id("PopupMenu")
            .expect("PopupMenu is in the catalog"),
        entry_version: 1,
        fields: vec![ExtensionField {
            id: 1,
            value: ExtensionValue::Bytes(POPUP_MENU_PROPS.as_bytes().to_vec()),
        }],
        event_ids: Arc::from([]),
    }));
    Snapshot::new(1, 1, 0, 1, vec![Node::new(1, 0, 0, crate::KIND_VIEW), menu])
}

/// A menu label is `text_sm` (14 px); its leading icon must share that scale,
/// which it did not while the popup drew the icon one size step below it.
#[gpui::test]
fn a_menu_item_icon_paints_at_the_label_size(cx: &mut TestAppContext) {
    let mut profile = ComponentHost::default();
    let runtime = InMemoryAdapter::new();
    let extensions = profile.extension_registry();
    let (window, solid_root) = cx.update(|app| {
        profile.initialize(app);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(480.), px(320.)),
                app,
            ))),
            ..Default::default()
        };
        profile
            .open_window(options, runtime.clone(), extensions, app)
            .expect("provider window opens")
    });
    let mut visual = VisualTestContext::from_window(window, cx);
    visual.update(|window, cx| {
        let message =
            crate::protocol::decode_message(&popup_menu_snapshot().encode().unwrap()).unwrap();
        solid_root.update(cx, |root, cx| {
            root.apply_decoded_message_in_window(message, window, cx)
                .unwrap()
        });
    });
    visual.run_until_parked();
    visual.update(|window, cx| window.draw(cx).clear(cx));

    let icon = visual
        .debug_bounds(MENU_ICON_SELECTOR)
        .expect("the menu item icon paints");
    assert_eq!(icon.size.width, MENU_ICON_SIZE, "{icon:?}");
    assert_eq!(icon.size.height, MENU_ICON_SIZE, "{icon:?}");
}
