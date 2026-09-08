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
const MAX_QUEUED_CONTROLS: usize = 4;
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

enum Control {
    Capture(std::sync::mpsc::SyncSender<Result<String, String>>),
    Resume,
    Activate,
}

#[derive(Default)]
struct Queues {
    events: Frames,
    commits: Frames,
    failure: Option<String>,
    shutdown: bool,
    finished: bool,
    controls: VecDeque<Control>,
    paused: bool,
    ready: bool,
    pressured: bool,
    staging: bool,
    drain: bool,
    wake_reader: bool,
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
        Self::from_source(name, source)
    }

    /// Start an application-owned bundle, including source embedded in its executable.
    /// The name identifies diagnostics; it is not resolved against the filesystem.
    pub fn from_source(
        name: impl Into<String>,
        source: Vec<u8>,
    ) -> Result<Arc<Self>, QuickJsError> {
        Self::launch(name.into(), source, None)
    }

    pub(super) fn launch(
        name: String,
        source: Vec<u8>,
        generation: Option<(u32, String)>,
    ) -> Result<Arc<Self>, QuickJsError> {
        let tap = ProtocolTap::from_env();
        let worker_tap = tap.clone();
        let shared = Arc::new(Shared::default());
        shared.queues.lock().unwrap().staging = generation.is_some();
        let worker_shared = Arc::clone(&shared);
        let worker = thread::Builder::new()
            .name("solid-gpui-quickjs".into())
            .stack_size(WORKER_STACK_BYTES)
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run(&worker_shared, &worker_tap, name, source, generation)
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

    pub(super) fn capture(&self) -> Result<String, String> {
        let (send, recv) = std::sync::mpsc::sync_channel(1);
        self.enqueue_control(Control::Capture(send))?;
        recv.recv_timeout(Duration::from_secs(2))
            .map_err(|_| "reload state capture timed out".to_owned())?
    }

    pub(super) fn resume(&self, activate: bool) {
        if let Err(error) = self.enqueue_control(if activate {
            Control::Activate
        } else {
            Control::Resume
        }) {
            self.shared.fail(error);
        }
    }

    fn enqueue_control(&self, control: Control) -> Result<(), String> {
        let mut queue = self.shared.queues.lock().unwrap();
        if queue.controls.len() >= MAX_QUEUED_CONTROLS {
            return Err("QuickJS reload control capacity exceeded".into());
        }
        // Activation must run before a later capture, even if both arrive
        // before the worker wakes. Never replace an unprocessed lifecycle step.
        queue.controls.push_back(control);
        self.shared.changed.notify_all();
        Ok(())
    }

    pub(super) fn wait_ready(&self, timeout: Duration) -> Result<(), String> {
        let queue = self.shared.queues.lock().unwrap();
        let (queue, _) = self
            .shared
            .changed
            .wait_timeout_while(queue, timeout, |q| {
                !q.ready && q.failure.is_none() && !q.finished && !q.shutdown
            })
            .unwrap();
        if let Some(error) = &queue.failure {
            return Err(error.clone());
        }
        if !queue.ready {
            return Err("QuickJS preparation timed out or stopped".into());
        }
        Ok(())
    }
    pub(super) fn wake_reader(&self) {
        self.shared.queues.lock().unwrap().wake_reader = true;
        self.shared.changed.notify_all();
    }
    pub(super) fn recv_interruptible(&self) -> Result<CommitPoll, ProtocolError> {
        self.receive(None)
    }

    pub(super) fn candidate_frames(&self) -> Result<Option<Vec<Vec<u8>>>, String> {
        let mut queue = self.shared.queues.lock().unwrap();
        if let Some(error) = &queue.failure {
            return Err(error.clone());
        }
        if !queue.ready {
            return Ok(None);
        }
        let mut frames = Vec::new();
        while let Some(frame) = queue.commits.pop() {
            frames.push(frame);
        }
        if queue.pressured {
            queue.pressured = false;
            queue.drain = true;
            self.shared.changed.notify_all();
        }
        Ok(Some(frames))
    }

    pub fn recv_commit_timeout(&self, timeout: Duration) -> Result<CommitPoll, ProtocolError> {
        self.receive(Some(timeout))
    }

    fn receive(&self, timeout: Option<Duration>) -> Result<CommitPoll, ProtocolError> {
        let queue = self.shared.queues.lock().unwrap();
        let waiting = |queue: &mut Queues| {
            queue.commits.values.is_empty()
                && !queue.wake_reader
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
        queue.wake_reader = false;
        if let Some(payload) = queue.commits.pop() {
            if queue.pressured
                && queue.commits.values.len() < 16
                && queue.commits.bytes < MAX_QUEUED_BYTES / 4
            {
                queue.pressured = false;
                queue.drain = true;
                self.shared.changed.notify_all();
            }
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
    generation: Option<(u32, String)>,
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
            move |ctx: Ctx, frame: TypedArray<u8>| -> rquickjs::Result<bool> {
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
                // Candidate publication is one bounded transaction: pausing
                // midway would hide its configuration from native validation.
                let writable = queue.staging
                    || (queue.commits.values.len() < 32
                        && queue.commits.bytes < MAX_QUEUED_BYTES / 2);
                queue.pressured |= !writable;
                submit_shared.changed.notify_all();
                Ok(writable)
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
        if let Some((epoch, state)) = generation {
            let settings = Object::new(ctx.clone()).map_err(|e| e.to_string())?;
            settings.set("epoch", epoch).map_err(|e| e.to_string())?;
            settings.set("state", state).map_err(|e| e.to_string())?;
            ctx.globals()
                .set("__solidGpuiGeneration", settings)
                .map_err(|e| e.to_string())?;
            ctx.eval::<(), _>(
                r#"
                let lifecycle;
                __solidGpuiGeneration.register = value => {
                    if (lifecycle) throw new Error('one mounted application per reload generation');
                    lifecycle = value;
                };
                globalThis.__solidGpuiGenerationControl = operation => {
                    if (operation === 'ready') return Boolean(lifecycle);
                    if (!lifecycle) throw new Error('reload requires mountApplication');
                    if (operation === 'capture') return lifecycle.capture();
                    if (operation === 'activate') lifecycle.activate();
                    if (operation === 'retire') lifecycle.retire();
                };
            "#,
            )
            .catch(&ctx)
            .map_err(|e| e.to_string())?;
        }
        let result = drive(
            &ctx,
            shared,
            tap,
            &bridge,
            name,
            source,
            &termination_deadline,
        );
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
    deadline: &Cell<Option<Instant>>,
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
    let generation = ctx
        .globals()
        .get::<_, Function>("__solidGpuiGenerationControl")
        .ok();
    let mut candidate = generation.is_some();
    let mut prefer_timer = false;
    loop {
        // Capture only after the active event's microtask checkpoint. Candidate
        // jobs stay suspended until native activation has acknowledged the tree.
        if !candidate && !shared.queues.lock().unwrap().paused {
            while !shared.stopped.load(Ordering::Acquire) && ctx.execute_pending_job() {
                if ctx.has_exception() {
                    return Err(
                        rquickjs::CaughtError::from_error(ctx, rquickjs::Error::Exception)
                            .to_string(),
                    );
                }
            }
        }
        let control = shared.queues.lock().unwrap().controls.pop_front();
        if let Some(control) = control {
            match control {
                Control::Capture(reply) => {
                    deadline.set(Some(Instant::now() + Duration::from_millis(500)));
                    let result = generation
                        .as_ref()
                        .ok_or_else(|| "runtime is not reloadable".to_owned())
                        .and_then(|f| {
                            f.call::<_, String>(("capture",))
                                .catch(ctx)
                                .map_err(|e| e.to_string())
                        });
                    deadline.set(None);
                    shared.queues.lock().unwrap().paused = result.is_ok();
                    let _ = reply.send(result);
                }
                Control::Resume => {
                    shared.queues.lock().unwrap().paused = false;
                }
                Control::Activate => {
                    candidate = false;
                    {
                        let mut queue = shared.queues.lock().unwrap();
                        queue.paused = false;
                        queue.staging = false;
                    }
                    generation
                        .as_ref()
                        .unwrap()
                        .call::<_, ()>(("activate",))
                        .catch(ctx)
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        if shared.stopped.load(Ordering::Acquire) {
            return Ok(());
        }
        let entry_ready = if let Some(result) = entry.result::<Value>() {
            result.catch(ctx).map_err(|error| error.to_string())?;
            true
        } else {
            false
        };
        if candidate
            && entry_ready
            && generation
                .as_ref()
                .unwrap()
                .call::<_, bool>(("ready",))
                .catch(ctx)
                .map_err(|e| e.to_string())?
        {
            let mut queue = shared.queues.lock().unwrap();
            queue.ready = true;
            queue.paused = true;
            shared.changed.notify_all();
        }
        {
            let queue = shared.queues.lock().unwrap();
            if queue.paused && !queue.shutdown {
                if queue.controls.is_empty() {
                    drop(shared.changed.wait(queue).unwrap());
                }
                continue;
            }
        }
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
        if candidate && !entry_ready && entry.result::<Value>().is_some() {
            continue;
        }
        let drain = {
            let mut queue = shared.queues.lock().unwrap();
            std::mem::take(&mut queue.drain)
        };
        if drain {
            bridge
                .get::<_, Function>("drain")
                .and_then(|f| f.call::<_, ()>(()))
                .catch(ctx)
                .map_err(|error| error.to_string())?;
            continue;
        }
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
                || !queue.controls.is_empty()
                || queue.drain
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
    fn reload_capture_reports_invalid_session_field_and_preserves_the_live_vm() {
        let entry = Entry::bundle("fixtures/quickjs-reload-state.tsx");
        let source = std::fs::read(&entry.0).unwrap();
        let runtime = QuickJsAdapter::launch(
            "reload-state.js".into(),
            source.clone(),
            Some((
                1,
                r#"{"state":[],"surfaceId":1,"open":true,"activationSequence":0}"#.into(),
            )),
        )
        .unwrap();
        runtime.wait_ready(Duration::from_secs(3)).unwrap();
        let frames = runtime.candidate_frames().unwrap().unwrap();
        let snapshot = Snapshot::decode(&frames[0]).unwrap();
        let button = snapshot
            .nodes
            .iter()
            .find(|node| node.kind == KIND_PRESSABLE)
            .unwrap();
        runtime.resume(true);
        // Observe activation before requesting capture from the live generation.
        assert_eq!(Patch::decode(&receive(&runtime)).unwrap().epoch, 1);
        let error = runtime.capture().unwrap_err();
        assert!(
            error.contains("reload state at $.state[0].session.userId: undefined is not JSON"),
            "{error}"
        );
        assert_ne!(runtime.status(), RuntimeStatus::Failed);
        runtime
            .send_event(Event::press(1, 1, 1, 1, button.id, button.listener_id))
            .unwrap();
        let patch = Patch::decode(&receive(&runtime)).unwrap();
        assert_eq!(patch.epoch, 1);
        let state = runtime.capture().unwrap();
        let decoded: serde_json::Value = serde_json::from_str(&state).unwrap();
        assert_eq!(
            decoded["state"],
            serde_json::json!([{ "session": { "userId": null } }])
        );
        let candidate =
            QuickJsAdapter::launch("restored-state.js".into(), source, Some((2, state))).unwrap();
        candidate.wait_ready(Duration::from_secs(3)).unwrap();
        let frames = candidate.candidate_frames().unwrap().unwrap();
        let restored = Snapshot::decode(&frames[0]).unwrap();
        assert_eq!(restored.epoch, 2);
        assert!(
            restored
                .nodes
                .iter()
                .any(|node| node.text.as_deref() == Some("Signed out"))
        );
        candidate.shutdown().unwrap();
        runtime.shutdown().unwrap();
    }

    #[test]
    fn immediate_capture_observes_activation() {
        for _ in 0..32 {
            let runtime = QuickJsAdapter::launch(
                "activation-order.js".into(),
                br#"
                let activated = false;
                __solidGpuiGeneration.register({
                    capture: () => JSON.stringify({ activated }),
                    activate: () => { activated = true; },
                    retire: () => {}
                });
                "#
                .to_vec(),
                Some((1, "{}".into())),
            )
            .unwrap();
            runtime.wait_ready(Duration::from_secs(3)).unwrap();
            runtime.resume(true);
            let captured = runtime.capture().unwrap();
            runtime.shutdown().unwrap();
            assert_eq!(captured, r#"{"activated":true}"#);
        }
    }

    #[test]
    fn candidate_staging_crosses_soft_watermark_and_resumes_after_activation() {
        let runtime = QuickJsAdapter::launch(
            "staged-pressure.js".into(),
            br#"
            const large = new Uint8Array(4 + 5 * 1024 * 1024);
            new DataView(large.buffer).setUint32(0, large.length - 4, true);
            const small = new Uint8Array([1, 0, 0, 0, 7]);
            __solidGpuiHost.subscribe(() => {}, () => {}, () => {});
            if (!__solidGpuiHost.submit(large)) throw new Error('partial staging');
            __solidGpuiHost.submit(small);
            __solidGpuiGeneration.register({
                capture: () => '{}',
                activate: () => __solidGpuiHost.submit(small),
                retire: () => {}
            });
            "#
            .to_vec(),
            Some((1, "{}".into())),
        )
        .unwrap();
        runtime.wait_ready(Duration::from_secs(3)).unwrap();
        let frames = runtime.candidate_frames().unwrap().unwrap();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].len(), 5 * 1024 * 1024);
        runtime.resume(true);
        assert_eq!(
            runtime.recv_commit_timeout(Duration::from_secs(3)).unwrap(),
            CommitPoll::Commit(vec![7])
        );
        runtime.shutdown().unwrap();
    }

    #[test]
    fn real_solid_bundle_round_trips_snapshot_press_and_patch() {
        let entry = Entry::bundle("fixtures/quickjs-counter.tsx");
        let source = std::fs::read(&entry.0).unwrap();
        drop(entry);
        let runtime = QuickJsAdapter::from_source("embedded-counter.js", source).unwrap();
        let snapshot = Snapshot::decode(&receive(&runtime)).unwrap();
        let ready = crate::protocol::decode_message(&receive(&runtime)).unwrap();
        assert!(matches!(
            ready,
            crate::protocol::DecodedMessage::Command(crate::protocol::Command {
                operation: crate::protocol::CommandOperation::ConfigureApplication {
                    quit: false,
                    ..
                },
                ..
            })
        ));
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
    fn native_async_boundaries_and_transitions_run_in_real_vm() {
        let entry = Entry::bundle("fixtures/quickjs-async.tsx");
        let runtime = entry.start();
        let snapshot = Snapshot::decode(&receive(&runtime)).unwrap();
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|node| node.text.as_deref() == Some("Async: passed"))
        );
        runtime.shutdown().unwrap();
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
            const old = __solidGpuiHost.subscribe(() => { throw new Error('retired subscription called'); }, () => {}, () => {});
            let rejected = false;
            try { __solidGpuiHost.subscribe(() => {}, () => {}, () => {}); } catch { rejected = true; }
            if (!rejected) throw new Error('second transport was admitted');
            old();
            __solidGpuiHost.subscribe(frame => __solidGpuiHost.submit(frame), () => { while (true) {} }, () => {});
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
