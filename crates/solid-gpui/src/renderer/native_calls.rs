use crate::native::MAX_NATIVE_CALL_BYTES;
use gpui::{AppContext, Context};

use super::SolidRoot;
use crate::protocol::{CommandKind, CommandMeta, CommandValue};

const MAX_PENDING_NATIVE_CALLS: usize = 32;

impl SolidRoot {
    pub(super) fn start_native_call(
        &mut self,
        meta: CommandMeta,
        module_id: [u8; 16],
        module_digest: [u8; 32],
        function_id: u32,
        args: Vec<u8>,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if self.pending_native_calls.len() >= MAX_PENDING_NATIVE_CALLS {
            return Err("native invocation capacity exceeded".to_owned());
        }
        let module = self
            .extension_registry
            .native_module(module_id, module_digest)
            .ok_or_else(|| "native module is not registered with this contract".to_owned())?;
        if module.module_id() != module_id || module.module_digest() != module_digest {
            return Err("native module registry returned a different contract".to_owned());
        }
        if args.len() > MAX_NATIVE_CALL_BYTES {
            return Err("native invocation arguments exceed the byte limit".to_owned());
        }
        if self.pending_native_calls.contains_key(&meta.request_id) {
            return Err("native request ID is already in flight".into());
        }
        let task = cx.background_spawn(async move {
            match module.invoke_async(function_id, args).await {
                Ok(bytes) if bytes.len() <= MAX_NATIVE_CALL_BYTES => Ok(bytes),
                Ok(_) => Err("native invocation result exceeds the byte limit".to_owned()),
                Err(error) if error.len() <= MAX_NATIVE_CALL_BYTES => Err(error),
                Err(_) => Err("native invocation error exceeds the byte limit".to_owned()),
            }
        });
        let entity = cx.weak_entity();
        let task = cx.spawn(async move |_, cx| {
            let result = task.await;
            let _ = entity.update(cx, |root, _| {
                // Closing or replacing a surface must never deliver an old call
                // into a new root's request-ID namespace. Later revisions in the
                // same epoch are valid: ordinary app state can change while Rust runs.
                if meta.surface_id != root.store.surface_id() || meta.epoch != root.store.epoch() {
                    return;
                }
                root.pending_native_calls.remove(&meta.request_id);
                let (success, error, value) = match result {
                    Ok(bytes) => (true, None, Some(CommandValue::Bytes(bytes))),
                    Err(error) => (false, Some(error), None),
                };
                root.emit_command_ack(meta, CommandKind::InvokeNative, success, error, value);
            });
        });
        self.pending_native_calls.insert(meta.request_id, task);
        Ok(())
    }
}
