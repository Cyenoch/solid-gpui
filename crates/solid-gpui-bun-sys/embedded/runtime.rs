//! Bun's event loop runs on the host's dedicated owner thread. Cross-thread
//! callers own only a weak VM door and byte-queue notifications, never JSC
//! values. This file is compiled inside the pinned Bun source tree.

mod lifecycle;

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use bun_event_loop::ConcurrentTask::ConcurrentTask;
use bun_event_loop::ManagedTask::ManagedTask;
use bun_event_loop::{JsPoster, Posted};
use bun_jsc::js_function::CreateJSFunctionOptions;
use bun_jsc::virtual_machine::VirtualMachine;
use bun_jsc::{CallFrame, JSFunction, JSGlobalObject, JSValue, JsResult, Strong, VmHandle};

const INPUT: u32 = 1;
const OUTPUT_DRAIN: u32 = 2;
const WAKE_FLAGS: u32 = INPUT | OUTPUT_DRAIN;
const CREATED: u8 = 0;
const RUNNING: u8 = 1;
const FINISHED: u8 = 2;
const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024 + 4;
const TURN_FRAMES: usize = 64;
const TURN_BYTES: usize = 1024 * 1024;

/// Borrowed for the whole `bun_embedded_run` call. Callbacks execute only on
/// the Bun thread, must return promptly, and must not unwind or reenter JS.
#[repr(C)]
pub struct EmbeddedIoCallbacks {
    pub context: *mut c_void,
    /// 1: accepted and writable; 0: accepted but the producer must wait for a
    /// drain notification; -1: failed/closed and the frame was not accepted.
    pub write_commit: unsafe extern "C" fn(*mut c_void, *const u8, usize) -> i32,
    /// 0: empty; usize::MAX: EOF; otherwise copies exactly one complete frame
    /// into `out`. Never return a length greater than the supplied capacity.
    pub read_event: unsafe extern "C" fn(*mut c_void, *mut u8, usize) -> usize,
}

#[derive(Clone)]
struct Target {
    vm: VmHandle,
    poster: JsPoster,
}

struct ControlState {
    flags: AtomicU32,
    scheduled: AtomicBool,
    terminated: AtomicBool,
    phase: AtomicU8,
    // The mutex protects publication only. Clone the weak target and release
    // the guard before posting/terminating; teardown can race either action.
    target: Mutex<Option<Target>>,
}

/// Opaque C ABI owner. Its allocation must outlive every in-progress ABI call;
/// the host waits for `run` and concurrent controls before `destroy`. Queued tasks retain an
/// independent Arc, so dropping a handle never frees a task's payload.
pub struct RuntimeControl {
    state: Arc<ControlState>,
}

impl ControlState {
    fn target(&self) -> Option<Target> {
        self.target
            .lock()
            .expect("embedded target mutex poisoned")
            .clone()
    }

    fn notify(self: &Arc<Self>, flags: u32) {
        if self.phase.load(Ordering::Acquire) == FINISHED {
            return;
        }
        self.flags.fetch_or(flags & WAKE_FLAGS, Ordering::AcqRel);
        self.schedule(false);
    }

    fn task(self: &Arc<Self>) -> bun_event_loop::Task {
        let payload = bun_core::heap::into_raw(Box::new(Arc::clone(self)));
        // On execution deliver() takes the payload box; on refusal/teardown
        // ManagedTask::new_owned drops it without calling JavaScript.
        ManagedTask::new_owned(payload, deliver)
    }

    fn schedule(self: &Arc<Self>, after_yield: bool) {
        if self.flags.load(Ordering::Acquire) == 0 || self.phase.load(Ordering::Acquire) != RUNNING
        {
            return;
        }
        let Some(target) = self.target() else {
            // Before publication flags remain pending. Registration signals
            // once again after the bootstrap's callback is GC-rooted.
            return;
        };
        if self.scheduled.swap(true, Ordering::AcqRel) {
            return;
        }
        let task = self.task();
        if after_yield {
            // Called only by deliver() on the owning JS thread. A normal
            // enqueue would self-refill the current drain forever, starving
            // I/O and timers. Bun promotes this queue after the next poll.
            VirtualMachine::get()
                .as_mut()
                .event_loop_mut()
                .enqueue_task_after_yield(task);
        } else if let Posted::Refused(task) = target.poster.post(ConcurrentTask::create(task)) {
            // The weak door guarantees ownership is returned after closure.
            // There are no VM-affine objects in this payload to drop here.
            unsafe { ConcurrentTask::release_refused(task) };
            self.scheduled.store(false, Ordering::Release);
        }
    }
}

struct JsState {
    control: Arc<ControlState>,
    callbacks: *const EmbeddedIoCallbacks,
    deliver: Option<Strong>,
    buffer: Vec<u8>,
    host_ref: bool,
    output_ref: bool,
    input_open: bool,
    input_active: bool,
}

thread_local! {
    static JS_STATE: RefCell<Option<JsState>> = const { RefCell::new(None) };
}

/// The outer run call borrows the callback table until after VM teardown.
/// Only the VM thread accesses this pointer; no JS call occurs while borrowed.
fn io_callbacks() -> Option<*const EmbeddedIoCallbacks> {
    JS_STATE.with(|slot| slot.borrow().as_ref().map(|s| s.callbacks))
}

#[bun_jsc::host_fn]
fn write_commit(global: &JSGlobalObject, frame: &CallFrame) -> JsResult<JSValue> {
    let Some(value) = frame.arguments().first().copied() else {
        return Err(
            global.throw_invalid_arguments(format_args!("native write requires a Uint8Array"))
        );
    };
    let Some(buffer) = value.as_array_buffer(global) else {
        return Err(
            global.throw_invalid_arguments(format_args!("native write requires a Uint8Array"))
        );
    };
    let Some(io) = io_callbacks() else {
        return Err(global.throw(format_args!("native transport is closed")));
    };
    let bytes = buffer.byte_slice();
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(
            global.throw_invalid_arguments(format_args!("native frame exceeds the byte limit"))
        );
    }
    // SAFETY: callbacks/bytes are borrowed for this synchronous invocation.
    // The host never blocks this thread while waiting for channel capacity.
    let accepted = unsafe { ((*io).write_commit)((*io).context, bytes.as_ptr(), bytes.len()) };
    if accepted == 0 {
        let acquire = JS_STATE.with(|slot| {
            let mut slot = slot.borrow_mut();
            slot.as_mut()
                .is_some_and(|state| !std::mem::replace(&mut state.output_ref, true))
        });
        if acquire {
            global.bun_vm().event_loop_shared().ref_keep_alive();
        }
    }
    match accepted {
        0 | 1 => Ok(JSValue::js_boolean(accepted == 1)),
        _ => Err(global.throw(format_args!("native commit queue is closed or full"))),
    }
}

#[bun_jsc::host_fn]
fn register_delivery(global: &JSGlobalObject, frame: &CallFrame) -> JsResult<JSValue> {
    let Some(callback) = frame
        .arguments()
        .first()
        .copied()
        .filter(|v| v.is_callable())
    else {
        return Err(
            global.throw_invalid_arguments(format_args!("native delivery requires a function"))
        );
    };
    let control = JS_STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        let state = state.as_mut()?;
        if state.deliver.is_some() {
            return None;
        }
        state.deliver = Some(Strong::create(callback, global));
        Some(Arc::clone(&state.control))
    });
    let Some(control) = control else {
        return Err(global.throw(format_args!(
            "native delivery is already registered or closed"
        )));
    };
    // Data may have arrived before the wrapper registered this callback.
    control.schedule(false);
    Ok(JSValue::UNDEFINED)
}

#[bun_jsc::host_fn]
fn activate_input(global: &JSGlobalObject, _frame: &CallFrame) -> JsResult<JSValue> {
    let control = JS_STATE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let state = slot.as_mut()?;
        state.input_active = true;
        Some(Arc::clone(&state.control))
    });
    let Some(control) = control else {
        return Err(global.throw(format_args!("native input is closed")));
    };
    control.notify(INPUT);
    Ok(JSValue::UNDEFINED)
}

fn callback() -> Option<JSValue> {
    JS_STATE.with(|slot| slot.borrow().as_ref()?.deliver.as_ref().map(Strong::get))
}

enum InputFrame {
    Empty,
    End,
    Data(Vec<u8>),
    Oversized,
}

fn read_frame() -> InputFrame {
    JS_STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        let Some(state) = state.as_mut().filter(|s| s.input_open) else {
            return InputFrame::Empty;
        };
        let io = state.callbacks;
        // SAFETY: run owns io and the reusable output buffer. The host callback
        // copies bytes and cannot call JS, so this short mutable borrow ends
        // before any event listener executes.
        let len = unsafe {
            ((*io).read_event)((*io).context, state.buffer.as_mut_ptr(), state.buffer.len())
        };
        match len {
            0 => InputFrame::Empty,
            usize::MAX => {
                state.input_open = false;
                InputFrame::End
            }
            len if len > state.buffer.len() => InputFrame::Oversized,
            len => InputFrame::Data(state.buffer[..len].to_vec()),
        }
    })
}

fn release_host_ref() {
    let release = JS_STATE.with(|slot| {
        let mut slot = slot.borrow_mut();
        slot.as_mut()
            .is_some_and(|state| std::mem::replace(&mut state.host_ref, false))
    });
    if release {
        VirtualMachine::get().event_loop_shared().unref_keep_alive();
    }
}

fn release_output_ref() {
    let release = JS_STATE.with(|slot| {
        let mut slot = slot.borrow_mut();
        slot.as_mut()
            .is_some_and(|state| std::mem::replace(&mut state.output_ref, false))
    });
    if release {
        VirtualMachine::get().event_loop_shared().unref_keep_alive();
    }
}

fn deliver_turn(control: &Arc<ControlState>) -> JsResult<bool> {
    let Some(deliver) = callback() else {
        // Bootstrap registration, not a timer, will reschedule these flags.
        return Ok(false);
    };
    // Do not retain a whole-VM shared reference across JS: a listener can
    // reenter a host function that mutates this same VM through its TLS.
    let vm = std::ptr::from_ref(VirtualMachine::get()).cast_mut();
    if !unsafe { (*vm).script_allowed() } {
        return Ok(false);
    }
    let global = unsafe { (*vm).global() };
    let flags = control.flags.swap(0, Ordering::AcqRel);
    if flags & OUTPUT_DRAIN != 0 {
        // Clear the old write's reference before invoking listeners. A drain
        // listener can refill stdout and acquire a fresh reference. This also
        // keeps a final backpressured response alive after input has reached EOF.
        release_output_ref();
        deliver
            .call(
                global,
                JSValue::UNDEFINED,
                &[JSValue::js_number_from_int32(2)],
            )
            .map_err(|error| {
                control.flags.fetch_or(flags & INPUT, Ordering::Release);
                error
            })?;
    }
    if flags & INPUT == 0 {
        return Ok(true);
    }
    if !JS_STATE.with(|slot| slot.borrow().as_ref().is_some_and(|s| s.input_active)) {
        // import() can yield before the app constructs its StdioTransport.
        // Keep frames in the bounded host queue until the first data listener
        // exists. Its newListener hook activates input without polling.
        control.flags.fetch_or(INPUT, Ordering::Release);
        return Ok(false);
    }
    let mut bytes = 0;
    for _ in 0..TURN_FRAMES {
        if !unsafe { (*vm).script_allowed() } {
            return Ok(false);
        }
        match read_frame() {
            InputFrame::Empty => return Ok(true),
            InputFrame::End => {
                release_host_ref();
                deliver.call(
                    global,
                    JSValue::UNDEFINED,
                    &[JSValue::js_number_from_int32(3)],
                )?;
                return Ok(true);
            }
            InputFrame::Oversized => {
                return Err(global.throw(format_args!("native read exceeded the frame buffer")));
            }
            InputFrame::Data(frame) => {
                bytes += frame.len();
                JSValue::create_buffer_from_box(global, frame.into_boxed_slice())
                    .and_then(|value| {
                        deliver.call(
                            global,
                            JSValue::UNDEFINED,
                            &[JSValue::js_number_from_int32(1), value],
                        )
                    })
                    .map_err(|error| {
                        // This event is consumed, but unread frames remain
                        // queued if uncaughtException handles an allocation or
                        // listener error.
                        control.flags.fetch_or(INPUT, Ordering::Release);
                        error
                    })?;
                // A single legal frame can be larger than the turn byte
                // budget; deliver it atomically and yield immediately after.
                if bytes >= TURN_BYTES {
                    break;
                }
            }
        }
    }
    control.flags.fetch_or(INPUT, Ordering::Release);
    Ok(true)
}

fn deliver(payload: *mut Arc<ControlState>) -> JsResult<()> {
    // SAFETY: this is the owned box passed to ManagedTask::new_owned. On this
    // execution path the callback, not ManagedTask, owns dropping the payload.
    let control = unsafe { bun_core::heap::take(payload) };
    let result = deliver_turn(&control);
    // Producers publish flags before their AcqRel scheduled RMW. This must
    // also be an AcqRel RMW, not a release store: when a producer saw the old
    // true bit, acquiring its write makes our following flags read see its
    // publication. Otherwise that producer sees false and posts the task.
    // A release-store/acquire-load pair alone admits the store-buffering
    // execution where both sides miss the other's publication and no task runs.
    control.scheduled.swap(false, Ordering::AcqRel);
    if !matches!(result, Ok(false)) {
        control.schedule(true);
    }
    result.map(|_| ())
}

fn install_globals(vm: *mut VirtualMachine) {
    // SAFETY: this is the current owner-thread VM, before running the entry.
    let global = unsafe { (*vm).global() };
    let target = global.to_js_value();
    let functions: [(&'static str, bun_jsc::JSHostFn, u32); 3] = [
        ("__solid_gpui_write", __jsc_host_write_commit, 1),
        ("__solid_gpui_register", __jsc_host_register_delivery, 1),
        ("__solid_gpui_activate_input", __jsc_host_activate_input, 0),
    ];
    for (name, function, arity) in functions {
        target.put(
            global,
            name,
            JSFunction::create(
                global,
                // JSFunction copies the owned Rust name through the Bun string.
                bun_core::String::static_(name.as_bytes()),
                function,
                arity,
                CreateJSFunctionOptions::default(),
            ),
        );
    }
}

fn wrapper_source(entry: &[u8]) -> Box<[u8]> {
    // Rust's quoted UTF-8 string escapes are valid modern JS string escapes
    // (including \u{...}). Reject invalid UTF-8 at the ABI boundary.
    let entry = std::str::from_utf8(entry).expect("validated UTF-8 entry");
    let quoted = format!("{:?}", entry);
    format!(
        r#"import {{ EventEmitter }} from 'node:events';
const input = new EventEmitter();
input.readable = true;
input.destroyed = false;
input.on('newListener', kind => {{
  if (kind === 'data') globalThis.__solid_gpui_activate_input();
}});
const output = new EventEmitter();
output.writable = true;
output.destroyed = false;
output.writableNeedDrain = false;
output.write = frame => {{
  if (!(frame instanceof Uint8Array)) throw new TypeError('native stdout requires Uint8Array');
  if (!output.writable) throw new Error('native stdout is closed');
  const writable = globalThis.__solid_gpui_write(frame);
  if (!writable) output.writableNeedDrain = true;
  return writable;
}};
Object.defineProperty(process, 'stdin', {{ value: input, configurable: true }});
Object.defineProperty(process, 'stdout', {{ value: output, configurable: true }});
globalThis.__solid_gpui_register((kind, frame) => {{
  if (kind === 1) {{
    if (input.readable) input.emit('data', frame);
  }} else if (kind === 2) {{
    if (output.writable && output.writableNeedDrain) {{
      output.writableNeedDrain = false;
      output.emit('drain');
    }}
  }} else if (kind === 3 && input.readable) {{
    input.readable = false;
    input.destroyed = true;
    try {{ input.emit('end'); }} finally {{ input.emit('close'); }}
  }}
}});
await import({quoted});
"#
    )
    .into_bytes()
    .into_boxed_slice()
}

fn run(control: &RuntimeControl, entry: &[u8], io: &EmbeddedIoCallbacks) -> crate::Result<u8> {
    if control.state.terminated.load(Ordering::Acquire) {
        return Ok(0);
    }
    let mut session = lifecycle::EmbeddedVm::new()?;
    let vm = session.as_ptr();
    // Initialize the one JS-thread owner before publishing a cross-thread
    // target. Host input keeps the loop alive without fabricating a timer.
    unsafe { (*vm).event_loop_shared().ref_keep_alive() };
    JS_STATE.with(|slot| {
        let previous = slot.replace(Some(JsState {
            control: Arc::clone(&control.state),
            callbacks: io,
            deliver: None,
            buffer: vec![0; MAX_FRAME_BYTES],
            host_ref: true,
            output_ref: false,
            input_open: true,
            input_active: false,
        }));
        assert!(
            previous.is_none(),
            "nested embedded session on one JS thread"
        );
    });
    install_globals(vm);
    let target = Target {
        vm: unsafe { (*vm).handle() },
        poster: unsafe { (*vm).js_poster() },
    };
    *control
        .state
        .target
        .lock()
        .expect("embedded target mutex poisoned") = Some(target.clone());
    if control.state.terminated.load(Ordering::Acquire) {
        target.vm.request_termination();
    }

    let wrapper = wrapper_source(entry);
    let entry_path = b"/[eval]";
    // Raw accesses avoid carrying an exclusive VM reference across JS, which
    // can reenter any host function and obtain the same VM through TLS.
    unsafe {
        (*vm).module_loader.eval_source = Some(Box::new(bun_ast::Source::init_path_string(
            entry_path,
            &wrapper[..],
        )));
        (*vm).set_main(entry_path);
        (*vm).load_extra_env_and_source_code_printer();
    }
    let evaluation = if unsafe { (*vm).script_allowed() } {
        unsafe { (*vm).load_entry_point(entry_path).map(Some) }
    } else {
        Ok(None)
    };
    // load_entry_point returns Ok even if the module Promise rejected, and
    // may return pending when a termination interrupts top-level await.
    let entry_promise = evaluation
        .as_ref()
        .ok()
        .and_then(|p| *p)
        .map(|promise| unsafe { Strong::create(JSValue::from_cell(promise), (*vm).global()) });
    let mut entry_rejected = false;
    if let Ok(Some(promise)) = &evaluation {
        if unsafe { (**promise).status() } == bun_jsc::js_promise::Status::Rejected {
            entry_rejected = true;
            unsafe {
                let error = (**promise).result((*vm).jsc_vm());
                (**promise).set_handled();
                // A termination trap can settle the entry as rejected too;
                // preserve process.exit's explicit code / host cancellation.
                if !error.is_termination_exception() {
                    (*vm).print_error_like_object_to_console(error);
                    (*vm).exit_handler.exit_code = 1;
                }
            }
        }
    }
    if evaluation.is_ok() && !entry_rejected {
        while unsafe { (*vm).script_allowed() && (*vm).is_event_loop_alive() } {
            unsafe { (*vm).tick() };
            if unsafe { (*vm).script_allowed() } {
                unsafe { (*vm).auto_tick_active() };
            }
        }
        if unsafe { (*vm).script_allowed() } {
            // Bun's on_before_exit owns the natural-exit loop: it drives any
            // timer/I/O scheduled by beforeExit, dispatches beforeExit again
            // after that work drains, and repeats until idle or terminated.
            unsafe { (*vm).on_before_exit() };
        }
    }
    // Natural exit listeners may still synchronously use stdout. Run them
    // while the bridge callback table and roots are alive, then revoke them
    // before destroying the heap. Forced termination forbids these listeners.
    session.on_exit();
    let stopped = !unsafe { (*vm).script_allowed() };
    // Revoke callback access before teardown can release queued tasks. The
    // weak target's already-cloned posts are safely accepted/released by Bun's
    // shutdown gate, or returned to their producer after closure.
    control
        .state
        .target
        .lock()
        .expect("embedded target mutex poisoned")
        .take();
    release_host_ref();
    release_output_ref();
    JS_STATE.with(|slot| drop(slot.take()));
    drop(entry_promise);
    let code = session.finish();
    drop(wrapper);
    // Some loader paths report JSError when a VM stop unwinds module loading.
    // Once stopped, its exit handler already owns the terminal status.
    if !stopped {
        evaluation?;
    }
    Ok(code)
}

#[unsafe(no_mangle)]
pub extern "C" fn bun_embedded_create() -> *mut RuntimeControl {
    Box::into_raw(Box::new(RuntimeControl {
        state: Arc::new(ControlState {
            flags: AtomicU32::new(0),
            scheduled: AtomicBool::new(false),
            terminated: AtomicBool::new(false),
            phase: AtomicU8::new(CREATED),
            target: Mutex::new(None),
        }),
    }))
}

/// # Safety
/// `control` is a live create() result, `entry` is readable for `len` bytes,
/// and `io` plus its context remain valid until this call returns. Call only
/// once per control, on the process's dedicated embedded Bun owner thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bun_embedded_run(
    control: *const RuntimeControl,
    entry: *const u8,
    len: usize,
    io: *const EmbeddedIoCallbacks,
) -> i32 {
    if control.is_null() || entry.is_null() || len == 0 || io.is_null() {
        return 2;
    }
    let control = unsafe { &*control };
    let entry = unsafe { std::slice::from_raw_parts(entry, len) };
    if std::str::from_utf8(entry).is_err()
        || control
            .state
            .phase
            .compare_exchange(CREATED, RUNNING, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
    {
        return 2;
    }
    let result = run(control, entry, unsafe { &*io });
    control.state.phase.store(FINISHED, Ordering::Release);
    control.state.flags.store(0, Ordering::Release);
    control.state.scheduled.store(false, Ordering::Release);
    match result {
        Ok(code) => i32::from(code),
        Err(error) => {
            eprintln!("embedded Bun failed: {error}");
            1
        }
    }
}

/// # Safety
/// `control` remains allocated until this call completes. May race run/stop.
/// Notify INPUT only after publishing its frame or EOF to the host queue.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bun_embedded_wake(control: *const RuntimeControl, flags: u32) {
    if let Some(control) = unsafe { control.as_ref() } {
        control.state.notify(flags);
    }
}

/// # Safety
/// `control` remains allocated until this call completes. May be called before
/// readiness or after run returns; a weak handle never dereferences a dead VM.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bun_embedded_terminate(control: *const RuntimeControl) {
    if let Some(control) = unsafe { control.as_ref() } {
        control.state.terminated.store(true, Ordering::Release);
        if let Some(target) = control.state.target() {
            target.vm.request_termination();
        }
    }
}

/// # Safety
/// `control` came from create(), has not been destroyed, and no ABI call can
/// still access its allocation. In particular the host has joined run().
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bun_embedded_destroy(control: *mut RuntimeControl) {
    if !control.is_null() {
        unsafe { drop(Box::from_raw(control)) };
    }
}
