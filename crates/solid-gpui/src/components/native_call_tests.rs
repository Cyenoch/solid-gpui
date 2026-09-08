use super::*;
use gpui::TestAppContext;
use solid_gpui::protocol::{
    Command, CommandKind, CommandMeta, CommandOperation, CommandValue, EventPayload,
};
use solid_gpui::{InMemoryAdapter, Node, Snapshot};

#[gpui::test]
fn foreground_theme_calls_share_contract_validation_and_update_native_base(
    cx: &mut TestAppContext,
) {
    let runtime = InMemoryAdapter::new();
    let module = super::super::theme::native_module();
    let set_theme = module.command_id("setTheme").unwrap();
    let set_motion = module.command_id("setMotionPreference").unwrap();
    let set_application = module.command_id("setApplicationTheme").unwrap();
    let id = module.id();
    let digest = module.digest();
    let mut profile = ComponentHost::new(vec![module]);
    let extensions = profile.extension_registry();
    let (window, root) = cx.update(|app| {
        crate::motion::initialize_for_test(app);
        profile.initialize(app);
        profile
            .open_window(WindowOptions::default(), runtime.clone(), extensions, app)
            .unwrap()
    });
    let snapshot = Snapshot::new(1, 1, 0, 1, vec![Node::new(1, 0, 0, solid_gpui::KIND_VIEW)]);
    window
        .update(cx, |_, window, cx| {
            root.update(cx, |root, cx| {
                root.apply_decoded_message_in_window(
                    crate::protocol::DecodedMessage::Snapshot(snapshot),
                    window,
                    cx,
                )
            })
            .unwrap();
        })
        .unwrap();
    for (request_id, input, success, dark, function_id) in [
        (1, "\"dark\"", true, true, set_theme),
        (2, "\"unknown\"", false, true, set_theme),
        (3, "\"light\"", true, false, set_theme),
        (
            4,
            r##"{"colors":{"primary":"#D4688C","buttonPrimaryHover":"#E07B9E","foreground":"#ECEAF1"},"fontSize":14,"radius":6,"inputBackground":"#1B1A20","components":{"button":{"height":32,"fontSize":14}}}"##,
            true,
            false,
            set_application,
        ),
        (
            5,
            r##"{"colors":{"primary":"invalid"}}"##,
            false,
            false,
            set_application,
        ),
        (6, "\"dark\"", true, true, set_theme),
    ] {
        let command = Command::new(
            CommandMeta {
                surface_id: 1,
                epoch: 1,
                after_revision: 1,
                request_id,
                node_id: 1,
            },
            CommandOperation::InvokeNative {
                module_id: id,
                module_digest: digest,
                function_id,
                args: input.as_bytes().to_vec(),
            },
        );
        window
            .update(cx, |_, window, cx| {
                root.update(cx, |root, cx| {
                    root.apply_decoded_message_in_window(
                        crate::protocol::DecodedMessage::Command(command),
                        window,
                        cx,
                    )
                })
                .unwrap();
                window.draw(cx).clear(cx);
                assert_eq!(gpui_component::Theme::global(cx).is_dark(), dark);
                if request_id >= 4 {
                    let theme = gpui_component::Theme::global(cx);
                    assert_eq!(theme.primary, gpui::rgb(0xd4688c).into());
                    assert_eq!(theme.input_background(), gpui::rgb(0x1b1a20).into());
                    assert_eq!(theme.component_metrics.button.height, Some(gpui::px(32.)));
                    assert_eq!(
                        theme.tokens.button_primary_hover.color,
                        gpui::rgb(0xe07b9e).into()
                    );
                    assert_eq!(
                        gpui_base::Theme::global(cx).tokens.colors.primary,
                        theme.primary
                    );
                }
                assert_eq!(
                    matches!(
                        gpui_base::Theme::global(cx).appearance,
                        gpui_base::ThemeAppearance::Dark
                    ),
                    dark
                );
            })
            .unwrap();
        let mut result = None;
        while let Some(event) = runtime.take_event().unwrap() {
            if let EventPayload::CommandResult(reply) = event.payload {
                assert!(result.replace(reply).is_none());
            }
        }
        let reply = result.expect("foreground command replies without waiting for a worker");
        assert_eq!(reply.request_id, request_id);
        assert_eq!(reply.success, success);
    }
    for enabled in [true, false] {
        let command = Command::new(
            CommandMeta {
                surface_id: 1,
                epoch: 1,
                after_revision: 1,
                request_id: 4,
                node_id: 1,
            },
            CommandOperation::InvokeNative {
                module_id: id,
                module_digest: digest,
                function_id: set_motion,
                args: if enabled {
                    br#""reduced""#.to_vec()
                } else {
                    br#""full""#.to_vec()
                },
            },
        );
        window
            .update(cx, |_, window, cx| {
                root.update(cx, |root, cx| {
                    root.apply_decoded_message_in_window(
                        crate::protocol::DecodedMessage::Command(command),
                        window,
                        cx,
                    )
                })
                .unwrap();
                assert_eq!(cx.reduce_motion(), enabled);
            })
            .unwrap();
        let mut replied = false;
        while let Some(event) = runtime.take_event().unwrap() {
            if let EventPayload::CommandResult(reply) = event.payload {
                assert!(reply.success);
                let Some(CommandValue::Bytes(bytes)) = reply.value else {
                    panic!("motion state reply")
                };
                let state: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(state["reduced"], enabled);
                assert_eq!(state["mode"], if enabled { "reduced" } else { "full" });
                replied = true;
            }
        }
        assert!(replied);
    }
    let module = profile
        .extension_registry()
        .native_module(id, digest)
        .unwrap();
    assert!(
        module
            .invoke(set_theme, br#""dark""#)
            .unwrap_err()
            .contains("foreground")
    );
}

#[gpui::test]
fn provider_dispatches_native_calls_and_correlates_errors(cx: &mut TestAppContext) {
    cx.background_executor.allow_parking();
    let runtime = InMemoryAdapter::new();
    let module = fixture::native_module();
    let id = module.id();
    let digest = module.digest();
    let mut profile = ComponentHost::new(vec![module]);
    let extensions = profile.extension_registry();
    let (window, root) = cx.update(|app| {
        profile.initialize(app);
        profile
            .open_window(WindowOptions::default(), runtime.clone(), extensions, app)
            .unwrap()
    });
    let snapshot = Snapshot::new(1, 1, 0, 1, vec![Node::new(1, 0, 0, solid_gpui::KIND_VIEW)]);
    root.update(cx, |root, cx| {
        root.apply_payload(&snapshot.encode().unwrap(), cx)
    })
    .unwrap();
    for request_id in 1..=5 {
        let mut module_digest = digest;
        if request_id == 2 {
            module_digest[0] ^= 1;
        }
        let command = Command::new(
            CommandMeta {
                surface_id: 1,
                epoch: 1,
                after_revision: 1,
                request_id,
                node_id: 1,
            },
            CommandOperation::InvokeNative {
                module_id: id,
                module_digest,
                function_id: if request_id == 4 { u32::MAX } else { 1 },
                args: if request_id == 3 {
                    br#"{"name":" ","readiness":true,"builds":3}"#.to_vec()
                } else {
                    br#"{"name":"Solid Workspace","readiness":true,"builds":3}"#.to_vec()
                },
            },
        );
        root.update(cx, |root, cx| {
            root.apply_payload(&command.encode().unwrap(), cx)
        })
        .unwrap();
    }
    window
        .update(cx, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    let mut results = std::collections::BTreeMap::new();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    // Real Tokio tasks complete outside GPUI's deterministic executor. Pump
    // GPUI after each wake until the observable command replies arrive.
    while results.len() < 5 && std::time::Instant::now() < deadline {
        cx.run_until_parked();
        while let Some(event) = runtime.take_event().unwrap() {
            if let EventPayload::CommandResult(result) = event.payload {
                assert_eq!(result.command, CommandKind::InvokeNative);
                assert_eq!(result.node_id, 1);
                assert!(
                    results.insert(result.request_id, result).is_none(),
                    "one completion per call"
                );
            }
        }
        if results.len() < 5 {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    assert_eq!(results.len(), 5);
    for id in [1, 5] {
        let result = &results[&id];
        assert!(result.success, "{result:?}");
        let Some(CommandValue::Bytes(bytes)) = &result.value else {
            panic!("missing native bytes")
        };
        let json = std::str::from_utf8(bytes).unwrap();
        assert_eq!(json, "\"solid-workspace:true:3\"");
    }
    for (id, expected) in [
        (2, "not registered"),
        (3, "must not be blank"),
        (4, "unknown native function"),
    ] {
        let result = &results[&id];
        assert!(!result.success);
        assert!(result.value.is_none());
        assert!(result.error.as_ref().unwrap().contains(expected));
    }
}

#[crate::native_module(name = "host-call-test")]
mod fixture {
    #[command]
    async fn analyze_workspace(
        name: String,
        readiness: bool,
        builds: u32,
    ) -> Result<String, String> {
        if name.trim().is_empty() {
            return Err("name must not be blank".into());
        }
        Ok(format!(
            "{}:{readiness}:{builds}",
            name.to_lowercase().replace(' ', "-")
        ))
    }
}
