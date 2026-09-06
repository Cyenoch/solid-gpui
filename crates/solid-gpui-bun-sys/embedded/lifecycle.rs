//! VM session ownership on the process-scoped Bun thread.

use core::marker::PhantomData;
use core::ptr::NonNull;
use std::rc::Rc;

use bun_jsc::virtual_machine::{InitOptions, VirtualMachine};

unsafe extern "C" {
    // Installed by bun_bin without its CLI main. Process facilities initialize
    // once; thread-local stack configuration runs on the owner at each session.
    safe fn bun_embedded_initialize_process();
    // Like Bun's Worker: the API lock belongs to the whole VM lifetime. A
    // borrowed RAII lock must not survive the final VM destruction.
    safe fn JSC__VM__getAPILock(vm: &bun_jsc::VM);
}

pub(super) struct EmbeddedVm {
    pointer: Option<NonNull<VirtualMachine>>,
    owner: std::thread::ThreadId,
    exit_handled: bool,
    _thread_bound: PhantomData<Rc<()>>,
}

impl EmbeddedVm {
    pub(super) fn new() -> crate::Result<Self> {
        bun_embedded_initialize_process();
        // Runtime services such as Bun.serve and console formatting read the
        // process options through the canonical CLI context. Install its
        // default options and process-owned log before a VM or Worker exists.
        crate::cli::command::initialize_embedded_context();
        bun_jsc::initialize(false);
        bun_ast::initialize_store_or_reset();
        assert!(
            VirtualMachine::get_or_null().is_none(),
            "a Bun VM session is already active on its owner thread"
        );
        // SAFETY: sessions are serialized on this process-scoped owner and
        // previous teardown joined all workers and drained every VM ticket.
        // Bun's process-global resolver caches must observe filesystem changes
        // made by the host between VM lifetimes, including negative lookups.
        unsafe { bun_resolver::Resolver::begin_embedded_session() };
        // Match the process/Vite entry contract: Solid's reactive universal
        // runtime is its browser export. Bun's default node export is SSR and
        // deliberately suppresses render effects, so it cannot emit commits.
        let mut options = InitOptions {
            is_main_thread: true,
            ..Default::default()
        };
        options.transform_options.conditions = vec![b"browser".to_vec().into_boxed_slice()];
        let pointer = NonNull::new(VirtualMachine::init(options)?)
            .expect("Bun VM initialization returned null");
        // SAFETY: init returns the sole VM on this thread. No handle is
        // published until this returns; short accesses permit JSC reentrancy.
        unsafe {
            (*pointer.as_ptr()).is_embedded = true;
            VirtualMachine::set_is_main_thread_vm(true);
            JSC__VM__getAPILock((*pointer.as_ptr()).jsc_vm());
            // Main VMs normally defer creation of this singleton. A host can
            // terminate immediately after publication, so materialize it now.
            (*pointer.as_ptr()).jsc_vm().termination_exception();
        }
        Ok(Self {
            pointer: Some(pointer),
            owner: std::thread::current().id(),
            exit_handled: false,
            _thread_bound: PhantomData,
        })
    }

    pub(super) fn as_ptr(&self) -> *mut VirtualMachine {
        self.pointer
            .expect("Bun VM session is already closed")
            .as_ptr()
    }

    /// Run exit callbacks while the bridge and its JS roots are still alive.
    /// The VM stop gate suppresses user callbacks on forced termination.
    pub(super) fn on_exit(&mut self) {
        if self.exit_handled {
            return;
        }
        self.exit_handled = true;
        let pointer = self.as_ptr();
        // SAFETY: called on the owning thread after script returned. Keep all
        // borrows short because exit handlers may re-enter native functions.
        unsafe {
            assert!(
                !(*pointer).jsc_vm().is_entered(),
                "exit cleanup requires an outermost VM frame"
            );
            (*pointer).is_shutting_down = true;
            (*pointer).on_exit();
        }
    }

    /// Call after on_exit, all JS frames returned, and bridge/entry Strong roots
    /// were dropped. Natural beforeExit belongs to the run loop, before on_exit.
    pub(super) fn finish(mut self) -> u8 {
        self.destroy()
    }

    fn destroy(&mut self) -> u8 {
        assert_eq!(
            std::thread::current().id(),
            self.owner,
            "Bun VM destruction moved off its owner thread"
        );
        if self.pointer.is_none() {
            return 0;
        }
        self.on_exit();
        let pointer = self.pointer.take().expect("active Bun VM");
        // SAFETY: sole owner, outside JS, and the API lock is still held. This
        // mirrors Worker::shutdown: native cleanup always runs; the stop gate
        // prevents user exit callbacks after host-forced termination.
        unsafe {
            assert!(
                !(*pointer.as_ptr()).jsc_vm().is_entered(),
                "cannot destroy a Bun VM under a live JS frame"
            );
            VirtualMachine::teardown_embedded(pointer.as_ptr())
        }
    }
}

impl Drop for EmbeddedVm {
    fn drop(&mut self) {
        self.destroy();
    }
}
