#![cfg(feature = "host")]
use solid_gpui::{
    HostProperties, Node, Snapshot,
    native::{encode_native_request, text::native_module},
    protocol::{ExtensionField, ExtensionProperties, ExtensionValue},
};

#[test]
fn extracted_application_checks_use_full_native_component_admission() {
    let module = native_module();
    let valid = encode_native_request(module.build_digest(), &serde_json::json!({})).unwrap();
    let stale = encode_native_request([0; 32], &serde_json::json!({})).unwrap();
    let invalid =
        encode_native_request(module.build_digest(), &serde_json::json!({"unknown": true}))
            .unwrap();
    let directory = tempfile::tempdir().unwrap();
    for (index, (props, accepted)) in [
        (valid, true),
        (stale, false),
        (b"{}".to_vec(), false),
        (invalid, false),
    ]
    .into_iter()
    .enumerate()
    {
        let mut node = Node::new(2, 1, 0, solid_gpui::KIND_EXTENSION);
        node.host_properties = Some(HostProperties::Extension(ExtensionProperties {
            provider_id: module.id(),
            catalog_digest: module.digest(),
            entry_id: module.component_id("TextSelectionObserver").unwrap(),
            entry_version: 1,
            fields: vec![ExtensionField {
                id: 1,
                value: ExtensionValue::Bytes(props),
            }],
            event_ids: std::sync::Arc::from([]),
        }));
        let bytes = Snapshot::new(
            1,
            1,
            0,
            1,
            vec![Node::new(1, 0, 0, solid_gpui::KIND_VIEW), node],
        )
        .encode()
        .unwrap();
        let mut frame = Vec::new();
        solid_gpui::protocol::write_frame(&mut frame, &bytes).unwrap();
        let script = directory.path().join(format!("app-{index}.js"));
        std::fs::write(
            &script,
            format!("process.stdout.write(new Uint8Array({frame:?})); process.stdin.resume();"),
        )
        .unwrap();
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_solid-gpui-host"))
            .args(["--check-app", "bun"])
            .arg(script)
            .output()
            .unwrap();
        assert_eq!(
            output.status.success(),
            accepted,
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
