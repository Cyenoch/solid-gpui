//! Headless extracted-artifact qualification using the real runtime and native catalog.
use super::{HostProfile, RuntimeMode, start_runtime};
use crate::{
    CommandOperation, CommandResult, CommandValue, DecodedMessage, Event, HostProperties, NodeStore,
};
use std::collections::HashMap;
use std::ffi::OsString;
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub(super) fn check<P: HostProfile>(profile: P, args: &[OsString]) -> Result<(), String> {
    if args.len() != 3 && args.len() != 4 {
        return Err("Usage: <host> --check-app <bun|quickjs> <bundle> [expected-text]".into());
    }
    let runtime = match args[1].to_str() {
        Some("quickjs") => start_runtime(RuntimeMode::QuickJs, &args[2..3], false)?,
        Some("bun") => start_runtime(
            RuntimeMode::Process,
            &["bun".into(), "--conditions=browser".into(), args[2].clone()],
            false,
        )?,
        _ => return Err("Application check runtime must be bun or quickjs".into()),
    };
    let reader = runtime.clone();
    let (sender, receiver) = mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        loop {
            let frame = reader.recv_commit();
            let ended = !matches!(&frame, Ok(Some(_)));
            if sender.send(frame).is_err() || ended {
                break;
            }
        }
    });
    let result = (|| {
        let registry = profile.extension_registry();
        let mut surfaces = HashMap::new();
        let mut sequences: HashMap<u32, u32> = HashMap::new();
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or("Application did not publish content within 15 seconds")?;
            let bytes = receiver
                .recv_timeout(remaining)
                .map_err(|e| e.to_string())?
                .map_err(|e| e.to_string())?
                .ok_or("Application ended before publishing content")?;
            let tree = match crate::decode_message(&bytes).map_err(|e| e.to_string())? {
                DecodedMessage::Snapshot(snapshot) => {
                    let tree = surfaces
                        .entry(snapshot.surface_id)
                        .or_insert_with(NodeStore::empty);
                    tree.apply_snapshot(snapshot).map_err(|e| e.to_string())?;
                    tree
                }
                DecodedMessage::Patch(patch) => {
                    let tree = surfaces
                        .get_mut(&patch.surface_id)
                        .ok_or("Patch precedes Snapshot")?;
                    tree.apply_patch(patch).map_err(|e| e.to_string())?;
                    tree
                }
                DecodedMessage::Command(command) => {
                    if matches!(
                        command.operation,
                        CommandOperation::ConfigureApplication { quit: false, .. }
                    ) {
                        continue;
                    }
                    let meta = command.meta;
                    let kind = command.operation.kind();
                    let tree = surfaces
                        .get(&meta.surface_id)
                        .ok_or("Command precedes Snapshot")?;
                    if meta.epoch != tree.epoch()
                        || meta.after_revision != tree.revision()
                        || meta.node_id != 1
                    {
                        return Err("Application check received an invalid root command".into());
                    }
                    let result = match command.operation {
                        CommandOperation::InvokeNative {
                            module_id,
                            module_digest,
                            function_id,
                            args,
                        } => {
                            let module = registry
                                .native_module(module_id, module_digest)
                                .ok_or("Native service contract is not registered")?;
                            let remaining = deadline
                                .checked_duration_since(Instant::now())
                                .ok_or("Application service check exceeded its deadline")?;
                            let bytes = futures::executor::block_on(async {
                                let call = module.invoke_async(function_id, args);
                                let timeout = async {
                                    let (sender, receiver) = futures::channel::oneshot::channel();
                                    std::thread::spawn(move || {
                                        std::thread::sleep(remaining);
                                        let _ = sender.send(());
                                    });
                                    let _ = receiver.await;
                                };
                                match futures::future::select(call, Box::pin(timeout)).await {
                                    futures::future::Either::Left((result, _)) => result,
                                    futures::future::Either::Right(_) => {
                                        Err("Application native service timed out".into())
                                    }
                                }
                            });
                            bytes.map(CommandValue::Bytes)
                        }
                        CommandOperation::CancelNative { .. } => continue,
                        _ => Err("Application check has no foreground window services".into()),
                    };
                    let sequence = sequences.entry(meta.surface_id).or_default();
                    *sequence += 1;
                    let (success, error, value) = match result {
                        Ok(value) => (true, None, Some(value)),
                        Err(error) => (false, Some(error), None),
                    };
                    runtime
                        .send_event(Event::command_result(
                            meta.surface_id,
                            meta.epoch,
                            tree.revision(),
                            *sequence,
                            CommandResult {
                                request_id: meta.request_id,
                                command: kind,
                                node_id: meta.node_id,
                                success,
                                error,
                                value,
                            },
                        ))
                        .map_err(|e| e.to_string())?;
                    continue;
                }
            };
            crate::SolidRoot::validate_extension_nodes(registry.as_ref(), tree, tree.iter())
                .map_err(|error| error.to_string())?;
            let mut content = false;
            for node in tree.iter() {
                content |= node.text.as_ref().is_some_and(|text| {
                    args.get(3).map_or(!text.is_empty(), |expected| {
                        text.as_ref() == expected.to_string_lossy().as_ref()
                    })
                });
                if args.len() == 3
                    && matches!(node.host_properties, Some(HostProperties::Extension(_)))
                {
                    content = true;
                }
            }
            if content {
                return Ok(());
            }
        }
    })();
    drop(receiver);
    let shutdown = runtime.shutdown().map_err(|e| e.to_string());
    let joined = worker
        .join()
        .map_err(|_| "Application check reader panicked".to_owned());
    result.and(shutdown).and(joined)?;
    println!("Application runtime, tree and native contracts verified; runtime shut down cleanly");
    Ok(())
}
