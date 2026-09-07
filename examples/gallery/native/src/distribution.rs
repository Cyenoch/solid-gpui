#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args == ["--version"] {
        println!("solid-gpui-gallery {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let check = args == ["--check-bundle"];
    if !check && !args.is_empty() {
        eprintln!("Usage: solid-gpui-gallery [--version | --check-bundle]");
        std::process::exit(2);
    }
    let runtime = solid_gpui::QuickJsAdapter::from_source(
        "gallery.js",
        include_bytes!(concat!(env!("OUT_DIR"), "/gallery.js")).to_vec(),
    )
    .unwrap_or_else(|error| {
        eprintln!("Gallery startup failed: {error}");
        std::process::exit(1);
    });
    if check {
        check_bundle(runtime.as_ref());
    } else {
        solid_gpui::run_application(gallery_host::native_module(), runtime);
    }
}

fn check_bundle(runtime: &solid_gpui::QuickJsAdapter) {
    use solid_gpui::{
        DecodedMessage, ExtensionRegistry, HostProperties, NodeStore, RuntimeAdapter,
        native::NativeModules, runtime::quickjs::CommitPoll,
    };

    // Router startup first publishes a small loading tree. Wait for the real
    // Gallery, so an empty bootstrap cannot hide stale application contracts.
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut tree = NodeStore::empty();
        let sdk = solid_gpui::components::native_module();
        let gallery = gallery_host::native_module();
        let sdk_id = sdk.id();
        let gallery_id = gallery.id();
        let modules = NativeModules::new(vec![sdk, gallery]);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            let remaining = deadline
                .checked_duration_since(std::time::Instant::now())
                .ok_or("Gallery did not produce its native content within 15 seconds")?;
            let CommitPoll::Commit(bytes) = runtime.recv_commit_timeout(remaining)? else {
                return Err(
                    "Gallery ended or timed out before producing its native content".into(),
                );
            };
            match solid_gpui::decode_message(&bytes)? {
                DecodedMessage::Snapshot(snapshot) => tree.apply_snapshot(snapshot)?,
                DecodedMessage::Patch(patch) => tree.apply_patch(patch)?,
                DecodedMessage::Command(command) => {
                    if command.meta.node_id != 1
                        || command.meta.surface_id != tree.surface_id()
                        || command.meta.epoch != tree.epoch()
                        || command.meta.after_revision != tree.revision()
                    {
                        return Err("Gallery issued an invalid startup root command".into());
                    }
                    // Window services have no implementation in this headless
                    // check, and Gallery composition does not await their replies.
                    continue;
                }
            }
            let mut sdk_present = false;
            let mut gallery_present = false;
            for node in tree.iter() {
                let Some(HostProperties::Extension(properties)) = &node.host_properties else {
                    continue;
                };
                if modules
                    .resolve(
                        properties.provider_id,
                        properties.catalog_digest,
                        properties.entry_id,
                        properties.entry_version,
                    )
                    .is_none()
                {
                    return Err(
                        format!("Gallery node {} has a stale native contract", node.id).into(),
                    );
                }
                sdk_present |= properties.provider_id == sdk_id;
                gallery_present |= properties.provider_id == gallery_id;
            }
            if sdk_present && gallery_present {
                return Ok(());
            }
        }
    })();
    let shutdown = runtime.shutdown();
    if result.is_err() || shutdown.is_err() {
        eprintln!("Gallery bundle check failed: commit={result:?}, shutdown={shutdown:?}");
        std::process::exit(1);
    }
    println!("Gallery bundle tree and native contracts verified; runtime shut down cleanly");
}
