//! Bun embedding ABI. The opaque control handle may cross threads; VM and JS
//! values never do. `run` borrows its IO table until complete VM teardown.

#[cfg(feature = "embedded-bun")]
use std::ffi::c_void;

/// The entry bytes handed to `bun_embedded_run` describe either a disk path or
/// a packaged module-graph identity, and this tag selects which: a tagged entry
/// is `[PACKAGED_ENTRY_TAG, identity…]`. No OS path can contain a NUL byte, so
/// the two forms are unambiguous, and a tagged entry is served only from the
/// executable's embedded module graph; it never falls back to the filesystem.
///
/// The identity is the graph key emitted by the packager, including the virtual
/// root: `/$bunfs/root/index.js`, or `B:/~BUN/root/index.js` on Windows.
#[cfg(feature = "embedded-bun")]
pub const PACKAGED_ENTRY_TAG: u8 = 0x00;

/// Statuses `bun_embedded_run` returns for a packaged session that could not
/// start. Exit codes are `0..=255`, so each of these is unambiguous, and the
/// session failed closed: no VM was created and no filesystem entry was read.
#[cfg(feature = "embedded-bun")]
pub mod packaged_graph_status {
    /// This executable exposes no usable embedded module graph.
    pub const UNAVAILABLE: i32 = -1;
    /// The image has graph data, but it is not a valid serialized graph.
    pub const MALFORMED: i32 = -2;
    /// The packaged entry is not a virtual module-graph path.
    pub const NOT_VIRTUAL: i32 = -3;
    /// The graph is present but holds no such entry.
    pub const MISSING: i32 = -4;
    /// The graph carries precompiled bytecode or module info.
    pub const BYTECODE: i32 = -5;
    /// The graph embeds a native library that would be extracted at runtime.
    pub const NATIVE_LIBRARY: i32 = -6;
}

#[cfg(feature = "embedded-bun")]
#[repr(C)]
pub struct BunIoCallbacks {
    pub context: *mut c_void,
    /// 1 = accepted/writable, 0 = accepted/wait for drain, -1 = failed.
    pub write_commit: unsafe extern "C" fn(*mut c_void, *const u8, usize) -> i32,
    /// Number of bytes, 0 = empty, usize::MAX = closed.
    pub read_event: unsafe extern "C" fn(*mut c_void, *mut u8, usize) -> usize,
}

/// The application's declared completion result, as it crosses the embedding
/// ABI.
///
/// This is not the VM's exit status. `bun_embedded_run` reports a byte, which is
/// what an OS process exit code can carry; an application that reports a real
/// process result — a Windows UAC cancellation is 1223 — declares it through the
/// embedded bridge's `complete` instead, and the host reads the full 32-bit
/// value here. The embedded runtime mirrors this layout in its own
/// `EmbeddedResult`, because that module is compiled inside the pinned Bun tree
/// and cannot depend on this crate. The `Bun` prefix mirrors `BunIoCallbacks`,
/// so the host crate may keep its own `EmbeddedResult` name for the typed view.
#[cfg(feature = "embedded-bun")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BunEmbeddedResult {
    /// 1 when the application declared a completion, 0 otherwise.
    pub present: u32,
    /// The declared code; 0 when absent.
    pub code: u32,
}

#[cfg(feature = "embedded-bun")]
unsafe extern "C" {
    pub fn bun_embedded_create() -> *mut c_void;
    /// Call only on the process-scoped engine owner thread, once per control.
    /// Entry bytes, IO table and context must remain valid until return.
    ///
    /// `entry` is a disk path, or [`PACKAGED_ENTRY_TAG`] followed by a bundled
    /// module-graph identity. The return value is the VM's exit code (`0..=255`),
    /// 2 for malformed input, or one of [`packaged_graph_status`]'s negative
    /// statuses when a packaged entry could not be served from the graph.
    pub fn bun_embedded_run(
        control: *mut c_void,
        entry: *const u8,
        len: usize,
        io: *const BunIoCallbacks,
    ) -> i32;
    /// Reads the application's declared completion into `out`; returns 0 on
    /// success and 2 for a null argument.
    pub fn bun_embedded_result(control: *mut c_void, out: *mut BunEmbeddedResult) -> i32;
    /// Thread-safe notifications: bit 0 = input, bit 1 = output capacity.
    pub fn bun_embedded_wake(control: *mut c_void, flags: u32);
    /// Thread-safe, including before `run` has published its VM handle.
    pub fn bun_embedded_terminate(control: *mut c_void);
    /// Call only after `run` and all concurrent control operations return.
    pub fn bun_embedded_destroy(control: *mut c_void);
}
