use super::*;
use gpui::TestAppContext;
use solid_gpui::protocol::{
    Command, CommandKind, CommandMeta, CommandOperation, CommandValue, EventPayload,
};
use solid_gpui::{InMemoryAdapter, Node, Snapshot};

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
