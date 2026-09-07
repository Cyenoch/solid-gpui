//! A lightweight UI runtime for prebundled ES modules. One worker owns every
//! QuickJS value; GPUI and the worker exchange only bounded owned protocol bytes.

use std::{
    cell::Cell,
    collections::VecDeque,
    io,
    path::Path,
    rc::Rc,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use rquickjs::{
    CatchResultExt, Context, Ctx, Exception, Function, Module, Object, Runtime, TypedArray, Value,
};
use thiserror::Error;

use crate::{
    Event, MAX_FRAME_LENGTH, ProtocolError, RuntimeAdapter, RuntimeStatus, transport::ProtocolTap,
};

const MAX_QUEUED_FRAMES: usize = 4096;
const MAX_QUEUED_BYTES: usize = MAX_FRAME_LENGTH + 4;
const MAX_VM_BYTES: usize = 256 * 1024 * 1024;
// Gallery's nested Solid components/effects exceed QuickJS's smaller defaults.
// Keep native callback and teardown headroom outside the bounded JS stack.
const MAX_JS_STACK_BYTES: usize = 2 * 1024 * 1024;
const WORKER_STACK_BYTES: usize = 4 * MAX_JS_STACK_BYTES;

#[derive(Default)]
struct Frames {
    values: VecDeque<Vec<u8>>,
    bytes: usize,
}

impl Frames {
    fn push(&mut self, frame: Vec<u8>) -> Result<(), &'static str> {
        if self.values.len() >= MAX_QUEUED_FRAMES
            || frame.len() > MAX_QUEUED_BYTES.saturating_sub(self.bytes)
        {
            return Err("QuickJS transport queue capacity exceeded");
        }
        self.bytes += frame.len();
        self.values.push_back(frame);
        Ok(())
    }

    fn pop(&mut self) -> Option<Vec<u8>> {
        let value = self.values.pop_front()?;
        self.bytes -= value.len();
        Some(value)
    }
}

#[derive(Default)]
struct Queues {
    events: Frames,
    commits: Frames,
    failure: Option<String>,
    shutdown: bool,
    finished: bool,
}

#[derive(Default)]
struct Shared {
    queues: Mutex<Queues>,
    changed: Condvar,
    stopped: AtomicBool,
}

impl Shared {
    fn fail(&self, message: String) {
        self.queues.lock().unwrap().failure.get_or_insert(message);
        self.stopped.store(true, Ordering::Release);
        self.changed.notify_all();
    }
    fn request_shutdown(&self) {
        self.queues.lock().unwrap().shutdown = true;
        self.stopped.store(true, Ordering::Release);
        self.changed.notify_all();
    }
}

#[derive(Debug, Error)]
pub enum QuickJsError {
    #[error("could not read QuickJS entry: {0}")]
    Entry(#[source] io::Error),
    #[error("could not start QuickJS worker: {0}")]
    Thread(#[source] io::Error),
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommitPoll {
    Commit(Vec<u8>),
    Timeout,
    Ended,
}

pub struct QuickJsAdapter {
    shared: Arc<Shared>,
    worker: Mutex<Option<JoinHandle<()>>>,
    tap: ProtocolTap,
}

impl QuickJsAdapter {
    /// Evaluate a bundled standard ES module on a dedicated worker. No package,
    /// TypeScript, Node, or Bun loader is installed in this VM.
    pub fn start(entry: impl AsRef<Path>) -> Result<Arc<Self>, QuickJsError> {
        let source = std::fs::read(entry.as_ref()).map_err(QuickJsError::Entry)?;
        let name = entry.as_ref().to_string_lossy().into_owned();
        let tap = ProtocolTap::from_env();
        let worker_tap = tap.clone();
        let shared = Arc::new(Shared::default());
        let worker_shared = Arc::clone(&shared);
        let worker = thread::Builder::new()
            .name("solid-gpui-quickjs".into())
            .stack_size(WORKER_STACK_BYTES)
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run(&worker_shared, &worker_tap, name, source)
                }));
                match result {
                    Ok(Err(error)) => worker_shared.fail(error),
                    Err(_) => worker_shared.fail("QuickJS worker panicked".into()),
                    Ok(Ok(())) => {}
                }
                // run has returned and destroyed the Context/Runtime before a
                // waiter can observe completion or join the worker.
                worker_shared.queues.lock().unwrap().finished = true;
                worker_shared.changed.notify_all();
            })
            .map_err(QuickJsError::Thread)?;
        Ok(Arc::new(Self {
            shared,
            worker: Mutex::new(Some(worker)),
            tap,
        }))
    }

    pub fn recv_commit_timeout(&self, timeout: Duration) -> Result<CommitPoll, ProtocolError> {
        self.receive(Some(timeout))
    }

    fn receive(&self, timeout: Option<Duration>) -> Result<CommitPoll, ProtocolError> {
        let queue = self.shared.queues.lock().unwrap();
        let waiting = |queue: &mut Queues| {
            queue.commits.values.is_empty()
                && !queue.finished
                && !queue.shutdown
                && queue.failure.is_none()
        };
        let mut queue = match timeout {
            Some(timeout) => {
                self.shared
                    .changed
                    .wait_timeout_while(queue, timeout, waiting)
                    .unwrap()
                    .0
            }
            None => self.shared.changed.wait_while(queue, waiting).unwrap(),
        };
        if let Some(payload) = queue.commits.pop() {
            return Ok(CommitPoll::Commit(payload));
        }
        if let Some(error) = &queue.failure {
            return Err(ProtocolError::Io(io::Error::other(error.clone())));
        }
        Ok(if queue.finished || queue.shutdown {
            CommitPoll::Ended
        } else {
            CommitPoll::Timeout
        })
    }
}

impl RuntimeAdapter for QuickJsAdapter {
    fn recv_commit(&self) -> Result<Option<Vec<u8>>, ProtocolError> {
        match self.receive(None)? {
            CommitPoll::Commit(payload) => Ok(Some(payload)),
            CommitPoll::Ended => Ok(None),
            CommitPoll::Timeout => unreachable!("unbounded receive cannot time out"),
        }
    }

    fn send_event(&self, event: Event) -> Result<(), ProtocolError> {
        let mut frame = Vec::new();
        event.encode_frame_into(&mut frame)?;
        let mut queue = self.shared.queues.lock().unwrap();
        if queue.shutdown || queue.finished || queue.failure.is_some() {
            return Err(ProtocolError::Io(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "QuickJS transport is closed",
            )));
        }
        if let Err(message) = queue.events.push(frame) {
            queue.failure = Some(message.into());
            self.shared.stopped.store(true, Ordering::Release);
            self.shared.changed.notify_all();
            return Err(ProtocolError::Io(io::Error::new(
                io::ErrorKind::WouldBlock,
                message,
            )));
        }
        self.shared.changed.notify_all();
        Ok(())
    }

    fn request_shutdown(&self) -> Result<(), ProtocolError> {
        self.shared.request_shutdown();
        Ok(())
    }

    /// Request interruption and join only from a background executor. Concurrent
    /// shutdown callers share the join; dropping the UI owner never waits.
    fn shutdown(&self) -> Result<(), ProtocolError> {
        self.request_shutdown()?;
        if let Some(worker) = self.worker.lock().unwrap().take() {
            worker
                .join()
                .map_err(|_| ProtocolError::Io(io::Error::other("QuickJS worker panicked")))?;
        }
        Ok(())
    }

    fn status(&self) -> RuntimeStatus {
        let queue = self.shared.queues.lock().unwrap();
        if queue.failure.is_some() {
            RuntimeStatus::Failed
        } else if queue.shutdown {
            RuntimeStatus::Shutdown
        } else if queue.finished {
            RuntimeStatus::Exited {
                code: Some(0),
                signal: None,
            }
        } else {
            RuntimeStatus::Running
        }
    }

    fn tap_inbound_payload(&self, payload: &[u8]) {
        self.tap.record_inbound_payload(payload);
    }
}

impl Drop for QuickJsAdapter {
    fn drop(&mut self) {
        self.shared.request_shutdown();
    }
}

fn run(
    shared: &Arc<Shared>,
    tap: &ProtocolTap,
    name: String,
    source: Vec<u8>,
) -> Result<(), String> {
    let runtime = Runtime::new().map_err(|error| error.to_string())?;
    runtime.set_memory_limit(MAX_VM_BYTES);
    runtime.set_max_stack_size(MAX_JS_STACK_BYTES);
    let termination_deadline = Rc::new(Cell::new(None::<Instant>));
    let interrupt_deadline = Rc::clone(&termination_deadline);
    let interrupt_shared = Arc::clone(shared);
    runtime.set_interrupt_handler(Some(Box::new(move || {
        interrupt_deadline.get().map_or_else(
            || interrupt_shared.stopped.load(Ordering::Acquire),
            |deadline| Instant::now() >= deadline,
        )
    })));
    let rejection_shared = Arc::clone(shared);
    runtime.set_host_promise_rejection_tracker(Some(Box::new(
        move |ctx, promise, reason, handled| {
            let result = ctx
                .globals()
                .get::<_, Function>("__solidGpuiTrackRejection")
                .and_then(|track| track.call::<_, ()>((promise, reason, handled)));
            if let Err(error) = result.catch(&ctx) {
                rejection_shared.fail(error.to_string());
            }
        },
    )));
    let context = Context::full(&runtime).map_err(|error| error.to_string())?;
    context.with(|ctx| {
        let started = Instant::now();
        let submit_shared = Arc::clone(shared);
        let submit = Function::new(
            ctx.clone(),
            move |ctx: Ctx, frame: TypedArray<u8>| -> rquickjs::Result<()> {
                let bytes = frame
                    .as_bytes()
                    .ok_or_else(|| Exception::throw_type(&ctx, "submitted frame is detached"))?;
                if bytes.len() < 4
                    || bytes.len() - 4 > MAX_FRAME_LENGTH
                    || u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize
                        != bytes.len() - 4
                {
                    let message = "QuickJS submit requires one complete bounded protocol frame";
                    submit_shared.fail(message.into());
                    return Err(Exception::throw_range(&ctx, message));
                }
                // Copy outside the shared lock: a maximum-sized commit must not
                // keep GPUI's event sender waiting while its bytes are copied.
                let payload = bytes[4..].to_vec();
                let mut queue = submit_shared.queues.lock().unwrap();
                if queue.shutdown || queue.failure.is_some() {
                    return Err(Exception::throw_message(
                        &ctx,
                        "QuickJS transport is closed",
                    ));
                }
                if let Err(message) = queue.commits.push(payload) {
                    queue.failure = Some(message.into());
                    submit_shared.stopped.store(true, Ordering::Release);
                    submit_shared.changed.notify_all();
                    return Err(Exception::throw_range(&ctx, message));
                }
                submit_shared.changed.notify_all();
                Ok(())
            },
        )
        .catch(&ctx)
        .map_err(|error| error.to_string())?;
        let now = Function::new(ctx.clone(), move || {
            started.elapsed().as_secs_f64() * 1000.0
        })
        .catch(&ctx)
        .map_err(|error| error.to_string())?;
        let log = Function::new(ctx.clone(), |message: String| {
            eprintln!("quickjs: {message}")
        })
        .catch(&ctx)
        .map_err(|error| error.to_string())?;
        let decode = Function::new(
            ctx.clone(),
            |ctx: Ctx, bytes: TypedArray<u8>, fatal: bool| -> rquickjs::Result<String> {
                let bytes = bytes
                    .as_bytes()
                    .ok_or_else(|| Exception::throw_type(&ctx, "UTF-8 input is detached"))?;
                if fatal {
                    std::str::from_utf8(bytes)
                        .map(str::to_owned)
                        .map_err(|_| Exception::throw_type(&ctx, "invalid UTF-8 input"))
                } else {
                    Ok(String::from_utf8_lossy(bytes).into_owned())
                }
            },
        )
        .catch(&ctx)
        .map_err(|error| error.to_string())?;
        let factory: Function = ctx
            .eval(include_str!("quickjs-bootstrap.js"))
            .catch(&ctx)
            .map_err(|error| error.to_string())?;
        let bridge: Object = factory
            .call((submit, now, log, decode))
            .catch(&ctx)
            .map_err(|error| error.to_string())?;
        let track: Function = bridge
            .get("trackRejection")
            .catch(&ctx)
            .map_err(|error| error.to_string())?;
        ctx.globals()
            .set("__solidGpuiTrackRejection", track)
            .catch(&ctx)
            .map_err(|error| error.to_string())?;
        let result = drive(&ctx, shared, tap, &bridge, name, source);
        let shutdown = shared.queues.lock().unwrap().shutdown;
        if let Err(error) = &result
            && !shutdown
        {
            shared.fail(error.clone());
        }
        // Let transport owners observe termination, but never allow their cleanup
        // callbacks to defeat host cancellation with another infinite JS loop.
        termination_deadline.set(Some(Instant::now() + Duration::from_millis(10)));
        // Queue failures can stop the VM between jobs or interrupt a callback.
        // Report their original cause instead of the resulting interruption.
        let message = shared
            .queues
            .lock()
            .unwrap()
            .failure
            .clone()
            .unwrap_or_else(|| "QuickJS runtime shut down".into());
        if let Ok(terminate) = bridge.get::<_, Function>("terminate") {
            let _ = terminate.call::<_, ()>((message,)).catch(&ctx);
        }
        if shared.queues.lock().unwrap().shutdown {
            Ok(())
        } else {
            result
        }
    })
}

fn drive<'js>(
    ctx: &Ctx<'js>,
    shared: &Shared,
    tap: &ProtocolTap,
    bridge: &Object<'js>,
    name: String,
    source: Vec<u8>,
) -> Result<(), String> {
    let dispatch: Function = bridge
        .get("dispatch")
        .catch(ctx)
        .map_err(|error| error.to_string())?;
    let subscribed: Function = bridge
        .get("subscribed")
        .catch(ctx)
        .map_err(|error| error.to_string())?;
    let run_timer: Function = bridge
        .get("runTimer")
        .catch(ctx)
        .map_err(|error| error.to_string())?;
    let next_timer: Function = bridge
        .get("nextTimer")
        .catch(ctx)
        .map_err(|error| error.to_string())?;
    let check_rejections: Function = bridge
        .get("checkRejections")
        .catch(ctx)
        .map_err(|error| error.to_string())?;
    let entry = Module::evaluate(ctx.clone(), name, source)
        .catch(ctx)
        .map_err(|error| error.to_string())?;
    let mut prefer_timer = false;
    loop {
        // Each native event/timer gets a complete microtask checkpoint. Checking
        // cancellation between jobs also interrupts self-replenishing job queues.
        while !shared.stopped.load(Ordering::Acquire) && ctx.execute_pending_job() {
            if ctx.has_exception() {
                return Err(
                    rquickjs::CaughtError::from_error(ctx, rquickjs::Error::Exception).to_string(),
                );
            }
        }
        if shared.stopped.load(Ordering::Acquire) {
            return Ok(());
        }
        if let Some(result) = entry.result::<Value>() {
            result.catch(ctx).map_err(|error| error.to_string())?;
        }
        check_rejections
            .call::<_, ()>(())
            .catch(ctx)
            .map_err(|error| error.to_string())?;
        if prefer_timer
            && run_timer
                .call::<_, bool>(())
                .catch(ctx)
                .map_err(|error| error.to_string())?
        {
            prefer_timer = false;
            continue;
        }
        let can_dispatch: bool = subscribed
            .call(())
            .catch(ctx)
            .map_err(|error| error.to_string())?;
        let event = if can_dispatch {
            shared.queues.lock().unwrap().events.pop()
        } else {
            None
        };
        if let Some(event) = event {
            // Optional diagnostic file writes stay off the UI and queue lock.
            tap.record_outbound_frame(&event);
            // Allocate in QuickJS so retained event buffers count toward the VM
            // memory limit, rather than attaching unaccounted external Vec storage.
            let frame = TypedArray::new_copy(ctx.clone(), &event)
                .catch(ctx)
                .map_err(|error| error.to_string())?;
            dispatch
                .call::<_, ()>((frame,))
                .catch(ctx)
                .map_err(|error| error.to_string())?;
            prefer_timer = true;
            // Due timers run on the next turn after this event's microtasks.
        } else if !run_timer
            .call::<_, bool>(())
            .catch(ctx)
            .map_err(|error| error.to_string())?
        {
            let delay: f64 = next_timer
                .call(())
                .catch(ctx)
                .map_err(|error| error.to_string())?;
            let queue = shared.queues.lock().unwrap();
            if shared.stopped.load(Ordering::Acquire)
                || (can_dispatch && !queue.events.values.is_empty())
            {
                continue;
            }
            if delay < 0.0 {
                drop(shared.changed.wait(queue).unwrap());
            } else {
                drop(
                    shared
                        .changed
                        .wait_timeout(queue, Duration::from_secs_f64(delay / 1000.0))
                        .unwrap(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{KIND_PRESSABLE, NodeStore, Patch, Snapshot};
    use std::{path::PathBuf, process::Command, sync::atomic::AtomicU64};

    struct Entry(PathBuf);
    impl Entry {
        fn new(source: &str) -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let dir = std::env::temp_dir().join(format!(
                "solid-gpui-quickjs-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&dir).unwrap();
            let path = dir.join("entry.mjs");
            std::fs::write(&path, source).unwrap();
            Self(path)
        }
        fn start(&self) -> Arc<QuickJsAdapter> {
            QuickJsAdapter::start(&self.0).unwrap()
        }
        fn bundle(fixture: &str) -> Self {
            let entry = Self::new("");
            let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            let output = Command::new("bun")
                .current_dir(root)
                .args([
                    "packages/solid-gpui/src/vite/build.ts",
                    "--runtime",
                    "quickjs",
                    fixture,
                ])
                .arg(&entry.0)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            entry
        }
    }
    impl Drop for Entry {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(self.0.parent().unwrap());
        }
    }
    fn receive(runtime: &QuickJsAdapter) -> Vec<u8> {
        let CommitPoll::Commit(payload) =
            runtime.recv_commit_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("expected QuickJS commit");
        };
        payload
    }

    #[test]
    fn real_solid_bundle_round_trips_snapshot_press_and_patch() {
        let entry = Entry::bundle("fixtures/quickjs-counter.tsx");
        let runtime = entry.start();
        let snapshot = Snapshot::decode(&receive(&runtime)).unwrap();
        let button = snapshot
            .nodes
            .iter()
            .find(|node| node.kind == KIND_PRESSABLE)
            .unwrap();
        runtime
            .send_event(Event::press(
                snapshot.surface_id,
                snapshot.epoch,
                snapshot.revision,
                1,
                button.id,
                button.listener_id,
            ))
            .unwrap();
        let patch = Patch::decode(&receive(&runtime)).unwrap();
        let mut tree = NodeStore::empty();
        tree.apply_snapshot(snapshot).unwrap();
        tree.apply_patch(patch).unwrap();
        assert!(
            tree.iter()
                .any(|node| node.text.as_deref() == Some("Count: 1 — 🌍"))
        );
        runtime.shutdown().unwrap();
        assert_eq!(runtime.status(), RuntimeStatus::Shutdown);
        assert_eq!(runtime.recv_commit().unwrap(), None);
    }

    #[test]
    fn native_router_platform_and_gallery_routes_run_in_real_vm() {
        let entry = Entry::bundle("fixtures/quickjs-router.tsx");
        let runtime = entry.start();
        // The fixture publishes only after its actual router, cancellation,
        // redirect, error rendering, and every Gallery route have succeeded.
        let snapshot = Snapshot::decode(&receive(&runtime)).unwrap();
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|node| node.text.as_deref() == Some("Router: passed"))
        );
        runtime.shutdown().unwrap();
        assert_eq!(runtime.status(), RuntimeStatus::Shutdown);
    }

    #[test]
    fn microtasks_utf8_and_cancellable_timers_use_real_worker() {
        let entry = Entry::new(
            r#"
            const encoder = new TextEncoder();
            const frame = text => {
                const bytes = encoder.encode(text), result = new Uint8Array(4 + bytes.length);
                new DataView(result.buffer).setUint32(0, bytes.length, true); result.set(bytes, 4);
                __solidGpuiHost.submit(result);
            };
            const assert = (condition) => { if (!condition) throw new Error('primitive assertion failed'); };
            assert(new TextDecoder().decode(encoder.encode('a🌍\ud800')) === 'a🌍�');
            const into = encoder.encodeInto('🌍a', new Uint8Array(4));
            assert(into.read === 2 && into.written === 4);
            const decoder = new TextDecoder('utf8', { fatal: true });
            assert(decoder.decode(new Uint8Array([0xf0, 0x9f]), { stream: true }) === '');
            assert(decoder.decode(new Uint8Array([0x8c, 0x8d])) === '🌍');
            let rejected = false;
            try { decoder.decode(new Uint8Array([0xff])); } catch { rejected = true; }
            assert(rejected);
            assert(decoder.decode(new Uint8Array([65]), { stream: true }) === 'A');
            try { decoder.decode(new Uint8Array([0xff]), { stream: true }); } catch {}
            assert(decoder.decode(new Uint8Array([0xef, 0xbb, 0xbf, 66])) === '\uFEFFB');
            assert(decoder.decode(new Uint8Array([0xef, 0xbb, 0xbf, 66])) === 'B');
            const cancelled = setTimeout(() => { throw new Error('cancelled timeout fired'); }, 0);
            clearTimeout(cancelled);
            const handled = Promise.reject(new Error('handled')); handled.catch(() => {});
            queueMicrotask(() => frame('microtask'));
            setTimeout(() => { frame('timeout'); queueMicrotask(() => frame('after timeout')); }, 0);
            const interval = setInterval(() => { clearInterval(interval); frame('interval'); }, 2);
        "#,
        );
        let runtime = entry.start();
        for expected in ["microtask", "timeout", "after timeout", "interval"] {
            assert_eq!(String::from_utf8(receive(&runtime)).unwrap(), expected);
        }
        assert_eq!(
            runtime
                .recv_commit_timeout(Duration::from_millis(10))
                .unwrap(),
            CommitPoll::Timeout
        );
        runtime.shutdown().unwrap();
    }

    #[test]
    fn shutdown_interrupts_busy_javascript_and_recursive_microtasks() {
        for work in [
            "while (true) {}",
            "const again = () => queueMicrotask(again); again();",
        ] {
            let entry = Entry::new(&format!(
                "__solidGpuiHost.submit(new Uint8Array([1,0,0,0,1])); {work}"
            ));
            let runtime = entry.start();
            assert_eq!(receive(&runtime), [1]);
            runtime.request_shutdown().unwrap();
            let (sender, receiver) = std::sync::mpsc::channel();
            let joined = Arc::clone(&runtime);
            thread::spawn(move || sender.send(joined.shutdown()).unwrap());
            receiver
                .recv_timeout(Duration::from_secs(3))
                .expect("QuickJS shutdown must interrupt execution")
                .unwrap();
            assert_eq!(runtime.status(), RuntimeStatus::Shutdown);
        }
    }

    #[test]
    fn malformed_frames_exceptions_and_queue_overflow_fail_explicitly() {
        for source in [
            "throw new Error('entry rejected');",
            "Promise.reject(new Error('unhandled rejection'));",
            "__solidGpuiHost.submit(new Uint8Array([10,0,0,0,1]));",
            "try { for (let n = 0; n < 4097; n++) __solidGpuiHost.submit(new Uint8Array([1,0,0,0,1])); } catch {}",
            "const frame = new Uint8Array(4 + 8388608); new DataView(frame.buffer).setUint32(0, 8388608, true); for (let n = 0; n < 3; n++) __solidGpuiHost.submit(frame);",
        ] {
            let entry = Entry::new(source);
            let runtime = entry.start();
            let deadline = Instant::now() + Duration::from_secs(5);
            while runtime.status() == RuntimeStatus::Running && Instant::now() < deadline {
                thread::yield_now();
            }
            assert_eq!(runtime.status(), RuntimeStatus::Failed, "{source}");
            loop {
                match runtime.recv_commit_timeout(Duration::from_secs(1)) {
                    Ok(CommitPoll::Commit(_)) => {}
                    Err(_) => break,
                    other => panic!("expected explicit transport failure: {other:?}"),
                }
            }
            runtime.shutdown().unwrap();
        }
    }

    #[test]
    fn startup_events_wait_for_subscription_and_outbound_pressure_never_waits() {
        let entry = Entry::new(
            r#"
            await new Promise(resolve => setTimeout(resolve, 2));
            const old = __solidGpuiHost.subscribe(() => { throw new Error('retired subscription called'); }, () => {});
            let rejected = false;
            try { __solidGpuiHost.subscribe(() => {}, () => {}); } catch { rejected = true; }
            if (!rejected) throw new Error('second transport was admitted');
            old();
            __solidGpuiHost.subscribe(frame => __solidGpuiHost.submit(frame), () => { while (true) {} });
            old();
        "#,
        );
        let runtime = entry.start();
        let event = Event::press(7, 3, 1, 1, 2, 2);
        runtime.send_event(event.clone()).unwrap();
        assert_eq!(Event::decode(&receive(&runtime)).unwrap(), event);
        let (sender, receiver) = std::sync::mpsc::channel();
        thread::spawn(move || sender.send(runtime.shutdown()).unwrap());
        receiver
            .recv_timeout(Duration::from_secs(3))
            .expect("termination callbacks cannot block shutdown")
            .unwrap();

        let entry = Entry::new("");
        let runtime = entry.start();
        for _ in 0..MAX_QUEUED_FRAMES {
            runtime.send_event(event.clone()).unwrap();
        }
        let error = runtime.send_event(event).unwrap_err();
        assert!(error.to_string().contains("capacity exceeded"));
        assert_eq!(runtime.status(), RuntimeStatus::Failed);
        runtime.shutdown().unwrap();
    }
}
