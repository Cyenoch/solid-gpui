use super::*;
use gpui::{Bounds, KeyBinding, TestAppContext, WindowBounds, px, size};
use solid_gpui::protocol::{ExtensionField, ExtensionProperties, ExtensionValue};
use solid_gpui::{HostProperties, InMemoryAdapter, Node, Snapshot};
use std::sync::Arc;

fn provider_button_snapshot() -> Snapshot {
    Snapshot::new(
        1,
        1,
        0,
        1,
        vec![
            Node::new(1, 0, 0, solid_gpui::KIND_VIEW),
            {
                let mut button = Node::new(2, 1, 0, solid_gpui::KIND_EXTENSION);
                button.host_properties = Some(HostProperties::Extension(ExtensionProperties {
                    provider_id: super::super::native_module().id(),
                    catalog_digest: super::super::native_module().digest(),
                    entry_id: 1,
                    entry_version: 1,
                    fields: vec![ExtensionField {
                        id: 1,
                        value: ExtensionValue::Bytes(br#"{"label":"Provider button"}"#.to_vec()),
                    }],
                    event_ids: Arc::from([]),
                }));
                button
            },
            Node::new(3, 2, 0, solid_gpui::KIND_TEXT),
            {
                let mut text = Node::new(4, 3, 0, solid_gpui::KIND_RAW_TEXT);
                text.text = Some("Child".to_owned());
                text
            },
        ],
    )
}

#[gpui::test]
fn profile_wraps_solid_root_with_provider_root_and_renders_extension(cx: &mut TestAppContext) {
    let mut profile = ComponentHost::default();
    let runtime = InMemoryAdapter::new();
    let extensions = profile.extension_registry();
    let (window, solid_root) = cx.update(|app| {
        profile.initialize(app);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(480.0), px(320.0)),
                app,
            ))),
            ..Default::default()
        };
        profile
            .open_window(options, runtime.clone(), extensions, app)
            .expect("provider window opens")
    });

    assert!(window.downcast::<Root>().is_some());
    window
        .update(cx, |_, window, cx| {
            solid_root.update(cx, |root, cx| {
                root.apply_decoded_message_in_window(
                    crate::protocol::decode_message(&provider_button_snapshot().encode().unwrap())
                        .unwrap(),
                    window,
                    cx,
                )
                .unwrap();
            })
        })
        .unwrap();
    window
        .update(cx, |_, window, cx| window.draw(cx).clear(cx))
        .expect("provider Root draws SolidRoot content");
    cx.refresh().expect("provider refresh succeeds");
    window
        .update(cx, |_, window, cx| window.draw(cx).clear(cx))
        .expect("provider Root redraws after refresh");
    solid_root.read_with(cx, |root, _| {
        assert_eq!(root.store().surface_id(), 1);
        assert_eq!(root.store().revision(), 1);
        assert!(root.store().get(2).is_some());
    });
}

#[test]
fn profile_rejects_system_notifications_but_supports_keybindings() {
    let profile = ComponentHost::default();
    assert!(!profile.capabilities().notification_responses);
    assert!(profile.capabilities().set_keybindings);
    assert_eq!(
        profile.rejected_command_reason(solid_gpui::CommandKind::ShowNotification),
        Some(
            "ShowNotification is unavailable in the gpui-component host because gpui-component owns the global system notification callback"
        )
    );
    assert_eq!(
        profile.rejected_command_reason(solid_gpui::CommandKind::SetKeybindings),
        None
    );
}

#[gpui::test]
fn provider_keybindings_restore_baseline_for_replacement_and_close(cx: &mut TestAppContext) {
    let profile = ComponentHost::default();
    let baseline = [KeyBinding::new(
        "cmd-b",
        solid_gpui::MenuAction {
            name: "base".into(),
        },
        None,
    )];
    let first = vec![KeyBinding::new(
        "cmd-1",
        solid_gpui::MenuAction {
            name: "first".into(),
        },
        None,
    )];
    let replacement = vec![KeyBinding::new(
        "cmd-2",
        solid_gpui::MenuAction {
            name: "replacement".into(),
        },
        None,
    )];
    cx.update(|app| profile.restore_keybindings(&baseline, first, app));
    cx.update(|app| {
        let key_bindings = app.key_bindings();
        let bindings = key_bindings.borrow();
        assert_eq!(bindings.bindings().count(), 2);
    });
    cx.update(|app| profile.restore_keybindings(&baseline, replacement, app));
    cx.update(|app| {
        let key_bindings = app.key_bindings();
        let bindings = key_bindings.borrow();
        assert_eq!(bindings.bindings().count(), 2);
    });
    cx.update(|app| profile.restore_keybindings(&baseline, Vec::new(), app));
    cx.update(|app| {
        let key_bindings = app.key_bindings();
        let bindings = key_bindings.borrow();
        assert_eq!(bindings.bindings().count(), 1);
    });
}
