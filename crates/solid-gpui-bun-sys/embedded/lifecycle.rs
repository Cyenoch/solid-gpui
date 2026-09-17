//! VM session ownership on the process-scoped Bun thread.

use core::marker::PhantomData;
use core::ptr::NonNull;
use std::rc::Rc;
use std::sync::LazyLock;

use bun_jsc::JSGlobalObject;
use bun_jsc::virtual_machine::{InitOptions, Options, VirtualMachine};

unsafe extern "C" {
    // Installed by bun_bin without its CLI main. Process facilities initialize
    // once; thread-local stack configuration runs on the owner at each session.
    safe fn bun_embedded_initialize_process();
    // Like Bun's Worker: the API lock belongs to the whole VM lifetime. A
    // borrowed RAII lock must not survive the final VM destruction.
    safe fn JSC__VM__getAPILock(vm: &bun_jsc::VM);
}

/// The entry a session evaluates.
///
/// `Disk` is the canonicalized application path from the ABI's untagged form:
/// the resolver reads it from the filesystem. `Packaged` is a bundled-graph
/// identity from the tagged form (`runtime::PACKAGED_ENTRY_TAG`): it is only
/// ever answered from the executable's embedded module graph, so no disk path
/// can stand in for it.
#[derive(Clone, Copy)]
pub(super) enum SessionEntry<'a> {
    Disk(&'a [u8]),
    Packaged(&'a [u8]),
}

/// Why a bundled module graph cannot serve a packaged session.
#[derive(Clone, Copy, Debug)]
pub(super) enum GraphFailure {
    /// The executable image carries no module graph at all.
    Unavailable,
    /// The image has graph data, but it is not a valid serialized graph.
    Malformed,
    /// The requested identity is not a virtual graph path. Resolving it would
    /// fall back to the filesystem, which a packaged session must never do.
    NotVirtual,
    /// The graph is present but holds no such entry.
    Missing,
    /// The graph carries precompiled bytecode or module info. JSC mutates
    /// those buffers in place, which pins the graph to one runtime version and
    /// one payload lifetime.
    Bytecode,
    /// The graph embeds a native library. Bun loads those by extracting the
    /// bytes to a temporary file, which the single-file product forbids.
    NativeLibrary,
}

impl GraphFailure {
    /// The negative status `bun_embedded_run` reports. Exit codes are 0..=255,
    /// so every one of these is unambiguous, and none is zero.
    pub(super) fn status(self) -> i32 {
        match self {
            Self::Unavailable => -1,
            Self::Malformed => -2,
            Self::NotVirtual => -3,
            Self::Missing => -4,
            Self::Bytecode => -5,
            Self::NativeLibrary => -6,
        }
    }

    /// One line for the host's console; the specific file, when known, is
    /// printed when the graph is rejected.
    pub(super) fn describe(self) -> &'static str {
        match self {
            Self::Unavailable => "the executable exposes no usable bundled module graph",
            Self::Malformed => "the bundled module graph is not a valid serialized graph",
            Self::NotVirtual => "the packaged entry is not a virtual module-graph path",
            Self::Missing => "the bundled module graph has no such entry",
            Self::Bytecode => "the bundled module graph carries precompiled bytecode",
            Self::NativeLibrary => "the bundled module graph embeds a native library",
        }
    }
}

/// Why a session never started.
pub(super) enum SessionFailure {
    Graph(GraphFailure),
    Bun(crate::Error),
}

impl From<bun_core::Error> for SessionFailure {
    fn from(error: bun_core::Error) -> Self {
        Self::Bun(error.into())
    }
}

impl From<bun_jsc::CrateError> for SessionFailure {
    fn from(error: bun_jsc::CrateError) -> Self {
        Self::Bun(error.into())
    }
}

/// The process's bundled module graph. Parsed once, including its failure: the
/// executable image cannot change under a running process, and re-parsing per
/// session would rebuild every file entry.
static GRAPH: LazyLock<Result<&'static dyn bun_resolver::StandaloneModuleGraph, GraphFailure>> =
    LazyLock::new(load_graph);

/// Adopt the executable's bundled module graph and confirm it can serve
/// `identity` without touching the filesystem.
///
/// Packaged sessions call this before any VM exists, because `VirtualMachine`
/// stores the graph in a process singleton and hands it to the resolver, which
/// reads it while modules load.
pub(super) fn admit(
    identity: &[u8],
) -> Result<&'static dyn bun_resolver::StandaloneModuleGraph, GraphFailure> {
    let graph = *GRAPH;
    let graph = graph?;
    // The resolver decides "embedded module" with this exact predicate, so a
    // false here means the import would be resolved against the filesystem:
    // report it as a packaging error rather than let that happen. `find` also
    // refuses non-virtual names, but the split keeps the diagnosis specific.
    if !bun_options_types::standalone_path::is_bun_standalone_file_path(identity) {
        return Err(GraphFailure::NotVirtual);
    }
    if graph.find(identity).is_none() {
        return Err(GraphFailure::Missing);
    }
    Ok(graph)
}

fn load_graph() -> Result<&'static dyn bun_resolver::StandaloneModuleGraph, GraphFailure> {
    let Some(pointer) =
        bun_standalone_graph::Graph::from_executable().map_err(|_| GraphFailure::Malformed)?
    else {
        return Err(GraphFailure::Unavailable);
    };
    // SAFETY: `from_executable` stores the graph in the process-lifetime
    // singleton and returns its non-null address. No VM exists yet, and the
    // graph's own accessors hand out `*mut` from the same `UnsafeCell`.
    let graph: &'static bun_standalone_graph::Graph = unsafe { &*pointer };
    reject_unsupported(graph)?;
    Ok(graph)
}

/// Both rejections are packaging errors, not runtime conditions: a graph that
/// carries them cannot be served by this embedding without writing files or
/// adopting a bytecode format, so it is refused before a VM exists. The
/// offending file is named on stderr for the packager.
fn reject_unsupported(graph: &bun_standalone_graph::Graph) -> Result<(), GraphFailure> {
    for file in graph.files.values() {
        if !file.bytecode.is_empty() || !file.module_info.is_empty() {
            eprintln!(
                "embedded Bun graph entry '{}' carries precompiled bytecode",
                String::from_utf8_lossy(file.name)
            );
            return Err(GraphFailure::Bytecode);
        }
        if file.loader == bun_ast::Loader::Napi || is_native_library(file.name) {
            eprintln!(
                "embedded Bun graph entry '{}' is a native library, which this embedding \
                 cannot extract at runtime",
                String::from_utf8_lossy(file.name)
            );
            return Err(GraphFailure::NativeLibrary);
        }
    }
    Ok(())
}

/// `bun:ffi` and `process.dlopen` both materialize an embedded library to a
/// temporary file before the loader sees it; `Loader::Napi` covers `.node`,
/// and the shared-library extensions cover the FFI path, which looks a graph
/// entry up by name.
fn is_native_library(name: &[u8]) -> bool {
    const NATIVE_EXTENSIONS: [&[u8]; 4] = [b".node", b".so", b".dylib", b".dll"];
    let extension = bun_paths::extension(name);
    NATIVE_EXTENSIONS
        .iter()
        .any(|native| extension.eq_ignore_ascii_case(native))
}

/// The graph retains one heap-allocated `Blob` per embedded file for the
/// process, and that `Blob` records the JS global which first asked for it.
/// Every session destroys its global, so a retained blob must be rebound to
/// the new session's global before any JS can reach it.
///
/// `File.wtf_string` needs no such treatment: `BunString__fromBytes` copies
/// into a FastMalloc-backed `WTF::StringImpl`, and JSC teardown destroys only
/// the VM heap — WTF itself is initialized process-wide by `JSCInitialize`'s
/// `call_once`. The source map cache holds plain parsed data.
fn rebind_cached_blobs(global: &'static JSGlobalObject) {
    let Some(graph) = bun_standalone_graph::Graph::get() else {
        return;
    };
    // SAFETY: `get()` returns the process-lifetime singleton's address from its
    // `UnsafeCell`. This runs on the owner thread after the VM was created and
    // before it evaluates anything, so the graph is not being read elsewhere.
    for file in unsafe { (*graph).files.values_mut() } {
        let Some(blob) = file.cached_blob else {
            continue;
        };
        // SAFETY: `cached_blob` is only ever set by
        // `api::standalone_graph_jsc::FileJsc::file_blob`, which heap-allocates
        // a `webcore::Blob` and pins its store so it is never freed.
        let blob = unsafe { blob.cast::<crate::webcore::Blob>().as_ref() };
        blob.global_this.set(core::ptr::from_ref(global));
    }
}

pub(super) struct EmbeddedVm {
    pointer: Option<NonNull<VirtualMachine>>,
    owner: std::thread::ThreadId,
    exit_handled: bool,
    _thread_bound: PhantomData<Rc<()>>,
}

impl EmbeddedVm {
    pub(super) fn new(entry: SessionEntry<'_>) -> Result<Self, SessionFailure> {
        bun_embedded_initialize_process();
        // Runtime services such as Bun.serve and console formatting read the
        // process options through the canonical CLI context. Install its
        // default options and process-owned log before a VM or Worker exists.
        crate::cli::command::initialize_embedded_context();
        // Adopt the bundled graph first: `init` stores it in the process
        // singleton, and the resolver answers virtual specifiers from it while
        // modules load.
        let graph = match entry {
            SessionEntry::Disk(_) => None,
            SessionEntry::Packaged(identity) => {
                Some(admit(identity).map_err(SessionFailure::Graph)?)
            }
        };
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
        let mut transform_options = bun_options_types::schema::api::TransformOptions::default();
        transform_options.conditions = vec![b"browser".to_vec().into_boxed_slice()];
        let pointer = match graph {
            None => VirtualMachine::init(InitOptions {
                transform_options,
                is_main_thread: true,
                ..Default::default()
            }),
            // The compiled-executable entry point: it is the only path that
            // copies the graph into the resolver and skips tsconfig/package
            // discovery on disk, and it matches how a `bun build --compile`
            // binary evaluates its own bundle.
            Some(graph) => VirtualMachine::init_with_module_graph(Options {
                args: transform_options,
                graph: Some(graph),
                is_main_thread: true,
                ..Default::default()
            }),
        }?;
        let pointer = NonNull::new(pointer).expect("Bun VM initialization returned null");
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
        let session = Self {
            pointer: Some(pointer),
            owner: std::thread::current().id(),
            exit_handled: false,
            _thread_bound: PhantomData,
        };
        // The CLI normally loads inherited variables before runtime startup.
        // This is independent of optional .env-file autoloading; Workers inherit
        // this map too. Keep VM ownership established if allocation fails.
        unsafe { (*pointer.as_ptr()).transpiler.env_mut() }
            .load_process()
            .map_err(|error| SessionFailure::Bun(error.into()))?;
        if graph.is_some() {
            // `init_with_module_graph` wires the graph but, unlike a compiled
            // binary's boot, does not apply the graph's own runtime flags
            // (tsconfig/package.json/bunfig/.env autoload decisions). Do it
            // before any module resolves, exactly as `RunCommand::boot_standalone`
            // does.
            let vm = unsafe { &mut *pointer.as_ptr() };
            if let Some(graph) = bun_standalone_graph::Graph::get() {
                // SAFETY: the singleton pointer's data is live for the process
                // and this path reads only the graph's serialized flags.
                let graph = unsafe { &*graph };
                crate::run_main::apply_standalone_runtime_flags(&mut vm.transpiler, graph);
            }
            rebind_cached_blobs(vm.global());
        }
        Ok(session)
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
