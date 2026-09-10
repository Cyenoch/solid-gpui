use super::*;
use crate::native::NativeModule;
use crate::protocol::{CommandOperation, EventPayload, Node, Snapshot};
use crate::transport::InMemoryAdapter;
use futures::{channel::oneshot, future::BoxFuture};
use gpui::{AppContext, Entity, TestAppContext, WindowHandle};
use std::collections::VecDeque;
use std::sync::{Mutex, atomic::AtomicUsize};

const MODULE_ID: [u8; 16] = [41; 16];
const MODULE_DIGEST: [u8; 32] = [42; 32];

#[derive(Default)]
struct Calls {
    started: AtomicUsize,
    active: AtomicUsize,
    cancelled: AtomicUsize,
    completed: AtomicUsize,
    replies: Mutex<VecDeque<oneshot::Sender<Vec<u8>>>>,
}
struct PendingCall {
    calls: Arc<Calls>,
    completed: bool,
}
impl Drop for PendingCall {
    fn drop(&mut self) {
        self.calls.active.fetch_sub(1, Ordering::SeqCst);
        if !self.completed {
            self.calls.cancelled.fetch_add(1, Ordering::SeqCst);
        }
    }
}
struct Module(Arc<Calls>);
impl NativeModule for Module {
    fn module_id(&self) -> [u8; 16] {
        MODULE_ID
    }
    fn module_digest(&self) -> [u8; 32] {
        MODULE_DIGEST
    }
    fn invoke(&self, _: u32, _: &[u8]) -> Result<Vec<u8>, String> {
        panic!("the background dispatcher must await invoke_async")
    }
    fn invoke_async(&self, _: u32, _: Vec<u8>) -> BoxFuture<'_, Result<Vec<u8>, String>> {
        Box::pin(async move {
            self.0.started.fetch_add(1, Ordering::SeqCst);
            self.0.active.fetch_add(1, Ordering::SeqCst);
            let mut guard = PendingCall {
                calls: self.0.clone(),
                completed: false,
            };
            let (sender, receiver) = oneshot::channel();
            self.0.replies.lock().unwrap().push_back(sender);
            let result = receiver
                .await
                .map_err(|_| "test reply channel closed".to_owned());
            guard.completed = true;
            self.0.completed.fetch_add(1, Ordering::SeqCst);
            result
        })
    }
}
struct Registry(Arc<Module>);
impl ExtensionRegistry for Registry {
    fn resolve(
        &self,
        p: [u8; 16],
        d: [u8; 32],
        e: u32,
        v: u32,
    ) -> Result<&dyn ExtensionAdapter, ExtensionError> {
        NoExtensions.resolve(p, d, e, v)
    }
    fn native_module(&self, id: [u8; 16], digest: [u8; 32]) -> Option<Arc<dyn NativeModule>> {
        (id == MODULE_ID && digest == MODULE_DIGEST)
            .then(|| self.0.clone() as Arc<dyn NativeModule>)
    }
}
struct Canvas;
impl Render for Canvas {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full()
    }
}
struct Fixture {
    root: Entity<SolidRoot>,
    window: WindowHandle<Canvas>,
    runtime: Arc<InMemoryAdapter>,
    calls: Arc<Calls>,
}
impl Fixture {
    fn new(cx: &mut TestAppContext) -> Self {
        let calls = Arc::new(Calls::default());
        let runtime = InMemoryAdapter::new();
        let root = cx.new(|_| {
            SolidRoot::with_extensions(
                runtime.clone(),
                Rc::new(Registry(Arc::new(Module(calls.clone())))),
            )
        });
        let window = cx.open_window(gpui::size(px(200.), px(100.)), |_, _| Canvas);
        let fixture = Self {
            root,
            window,
            runtime,
            calls,
        };
        fixture.epoch(cx, 1);
        fixture
    }
    fn apply(&self, cx: &mut TestAppContext, message: DecodedMessage) {
        cx.update_window(self.window.into(), |_, window, cx| {
            self.root.update(cx, |root, cx| {
                root.apply_decoded_message_in_window(message, window, cx)
                    .unwrap();
                root.process_commands(window, cx);
            })
        })
        .unwrap();
    }
    fn epoch(&self, cx: &mut TestAppContext, epoch: u32) {
        self.apply(
            cx,
            DecodedMessage::Snapshot(Snapshot::new(
                1,
                epoch,
                0,
                1,
                vec![Node::new(1, 0, 0, KIND_VIEW)],
            )),
        );
    }
    fn call(&self, cx: &mut TestAppContext, epoch: u32, request_id: u32) {
        self.apply(
            cx,
            DecodedMessage::Command(Command::new(
                CommandMeta {
                    surface_id: 1,
                    epoch,
                    after_revision: 1,
                    request_id,
                    node_id: 1,
                },
                CommandOperation::InvokeNative {
                    module_id: MODULE_ID,
                    module_digest: MODULE_DIGEST,
                    function_id: 1,
                    args: vec![],
                },
            )),
        );
        cx.run_until_parked();
    }
    fn reply(&self) -> oneshot::Sender<Vec<u8>> {
        self.calls
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .expect("native call started")
    }
}

#[gpui::test]
fn epoch_replacement_cancels_old_native_future_without_touching_reused_request_id(
    cx: &mut TestAppContext,
) {
    let fixture = Fixture::new(cx);
    fixture.call(cx, 1, 7);
    let old_reply = fixture.reply();
    assert_eq!(fixture.calls.active.load(Ordering::SeqCst), 1);
    fixture.epoch(cx, 2);
    cx.run_until_parked();
    assert_eq!(fixture.calls.cancelled.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.calls.active.load(Ordering::SeqCst), 0);
    fixture.call(cx, 2, 7);
    let new_reply = fixture.reply();
    assert_eq!(fixture.calls.active.load(Ordering::SeqCst), 1);
    assert!(
        old_reply.send(b"old".to_vec()).is_err(),
        "retired call must release its receiver"
    );
    cx.run_until_parked();
    assert_eq!(
        fixture.calls.active.load(Ordering::SeqCst),
        1,
        "old completion cannot remove the replacement call"
    );
    new_reply.send(b"new".to_vec()).unwrap();
    cx.run_until_parked();
    assert_eq!(fixture.calls.active.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.calls.completed.load(Ordering::SeqCst), 1);
    let mut results = Vec::new();
    while let Some(event) = fixture.runtime.take_event().unwrap() {
        if let EventPayload::CommandResult(result) = event.payload {
            results.push((event.meta.epoch, result));
        }
    }
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 2);
    assert_eq!(results[0].1.request_id, 7);
    assert_eq!(
        results[0].1.value,
        Some(CommandValue::Bytes(b"new".to_vec()))
    );
}

#[gpui::test]
fn closing_surface_and_dropping_root_cancel_pending_native_futures(cx: &mut TestAppContext) {
    let closed = Fixture::new(cx);
    closed.call(cx, 1, 1);
    let closed_reply = closed.reply();
    closed.root.update(cx, |root, _| root.emit_surface_closed());
    cx.run_until_parked();
    assert_eq!(closed.calls.cancelled.load(Ordering::SeqCst), 1);
    assert_eq!(closed.calls.active.load(Ordering::SeqCst), 0);
    assert!(closed_reply.send(vec![]).is_err());
    assert!(
        closed.root.downgrade().upgrade().is_some(),
        "surface closure must cancel while external handles still own the root"
    );
    assert!(matches!(
        closed.runtime.take_event().unwrap().unwrap().payload,
        EventPayload::SurfaceClosed
    ));
    assert!(closed.runtime.take_event().unwrap().is_none());
    cx.update(|_| drop(closed));

    let fixture = Fixture::new(cx);
    fixture.call(cx, 1, 1);
    let reply = fixture.reply();
    let weak = fixture.root.downgrade();
    let calls = fixture.calls.clone();
    let runtime = fixture.runtime.clone();
    // GPUI releases entities at the end of an App update, as in host teardown.
    cx.update(|_| drop(fixture));
    cx.run_until_parked();
    assert!(weak.upgrade().is_none());
    assert_eq!(calls.cancelled.load(Ordering::SeqCst), 1);
    assert_eq!(calls.active.load(Ordering::SeqCst), 0);
    assert!(reply.send(vec![]).is_err());
    assert!(runtime.take_event().unwrap().is_none());
}

#[gpui::test]
fn cancellation_releases_only_the_target_native_call(cx: &mut TestAppContext) {
    let fixture = Fixture::new(cx);
    fixture.call(cx, 1, 7);
    let cancelled = fixture.reply();
    fixture.call(cx, 1, 8);
    let adjacent = fixture.reply();
    for request_id in [7, 7, 999] {
        fixture.apply(
            cx,
            DecodedMessage::Command(Command::new(
                CommandMeta {
                    surface_id: 1,
                    epoch: 1,
                    after_revision: 1,
                    request_id: 20,
                    node_id: 1,
                },
                CommandOperation::CancelNative { request_id },
            )),
        );
        cx.run_until_parked();
    }
    assert_eq!(fixture.calls.cancelled.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.calls.active.load(Ordering::SeqCst), 1);
    assert!(cancelled.send(b"late".to_vec()).is_err());
    adjacent.send(b"adjacent".to_vec()).unwrap();
    cx.run_until_parked();
    assert_eq!(fixture.calls.completed.load(Ordering::SeqCst), 1);
    let mut replies = Vec::new();
    while let Some(event) = fixture.runtime.take_event().unwrap() {
        if let EventPayload::CommandResult(result) = event.payload
            && result.command == CommandKind::InvokeNative
        {
            replies.push(result.request_id);
        }
    }
    assert_eq!(replies, [8]);
}
