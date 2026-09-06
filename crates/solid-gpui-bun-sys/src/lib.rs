//! Bun embedding ABI. The opaque control handle may cross threads; VM and JS
//! values never do. `run` borrows its IO table until complete VM teardown.

#[cfg(feature = "embedded-bun")]
use std::ffi::c_void;

#[cfg(feature = "embedded-bun")]
#[repr(C)]
pub struct BunIoCallbacks {
    pub context: *mut c_void,
    /// 1 = accepted/writable, 0 = accepted/wait for drain, -1 = failed.
    pub write_commit: unsafe extern "C" fn(*mut c_void, *const u8, usize) -> i32,
    /// Number of bytes, 0 = empty, usize::MAX = closed.
    pub read_event: unsafe extern "C" fn(*mut c_void, *mut u8, usize) -> usize,
}

#[cfg(feature = "embedded-bun")]
unsafe extern "C" {
    pub fn bun_embedded_create() -> *mut c_void;
    /// Call only on the process-scoped engine owner thread, once per control.
    /// Entry bytes, IO table and context must remain valid until return.
    pub fn bun_embedded_run(
        control: *mut c_void,
        entry: *const u8,
        len: usize,
        io: *const BunIoCallbacks,
    ) -> i32;
    /// Thread-safe notifications: bit 0 = input, bit 1 = output capacity.
    pub fn bun_embedded_wake(control: *mut c_void, flags: u32);
    /// Thread-safe, including before `run` has published its VM handle.
    pub fn bun_embedded_terminate(control: *mut c_void);
    /// Call only after `run` and all concurrent control operations return.
    pub fn bun_embedded_destroy(control: *mut c_void);
}
