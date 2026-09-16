//! The thread that runs the GPUI application loop, and how much stack it is given.
//!
//! GPUI draws a frame by recursing through the element tree
//! (`request_layout` → `prepaint` → `paint`), so the stack a frame needs grows with the
//! depth of the UI tree. macOS and Linux start the process's initial thread with 8 MiB,
//! the budget GPUI is developed against; a Windows executable gets whatever its PE
//! header asks for, which is 1 MiB for a default Rust binary, and applications with
//! deeper trees or their own unoptimized element wrappers overflow it.
//!
//! Windows also has no equivalent of AppKit's rule that windows live on the initial
//! thread: `gpui_windows::WindowsDispatcher` records the thread id when the platform is
//! constructed and calls that thread "main" (`is_main_thread`), so the host may give the
//! application a thread with an explicit stack. macOS asserts the real main thread
//! (`NSThread.isMainThread`) inside `App::new`, so every other platform runs the
//! application inline on the calling thread.
//!
//! Hosts never call this module: the public entrypoints in [`super`] hand it a closure
//! that builds the profile on the application thread, which is also why a profile only
//! has to be `Send` when it is *constructed* there.

/// Stack the host reserves for the application thread when nothing else is configured.
///
/// Windows commits thread stack pages on demand, so the reservation costs address space
/// rather than memory. The value restores the 8 MiB budget the initial thread gets on
/// macOS and Linux, doubled for element wrappers that development profiles leave
/// unoptimized (`gpui-pre`, `taffy`, and `slotmap` are pinned to `opt-level = 3`, but the
/// host's own `Element` wrappers and application code are not), and is far above the
/// 1 MiB default that overflows.
#[cfg(any(windows, test))]
const DEFAULT_APP_STACK_BYTES: usize = 16 * 1024 * 1024;

/// Byte count overriding [`DEFAULT_APP_STACK_BYTES`] on Windows.
#[cfg(windows)]
const APP_STACK_BYTES_ENV: &str = "SOLID_GPUI_APP_STACK_BYTES";

/// Run `launch` on the thread GPUI binds as the application's main thread.
///
/// On Windows this reserves [`DEFAULT_APP_STACK_BYTES`] (or `SOLID_GPUI_APP_STACK_BYTES`)
/// for the application, starting a dedicated thread when the calling thread does not
/// already own that much stack. On every other platform `launch` runs inline, which keeps
/// the application on the real main thread that AppKit and the macOS text systems
/// require. Panics inside `launch` resume on the calling thread, so the process still
/// reports them once, with the original payload.
pub(super) fn launch_on_app_thread(launch: impl FnOnce() + Send + 'static) {
    #[cfg(windows)]
    {
        let budget = resolved_stack_budget();
        if reserved_stack_bytes().is_some_and(|bytes| bytes >= budget) {
            launch();
            return;
        }
        let thread = std::thread::Builder::new()
            .name("solid-gpui-app".to_owned())
            .stack_size(budget)
            .spawn(launch)
            .unwrap_or_else(|error| {
                panic!("solid-gpui could not start the application thread with a {budget} byte stack: {error}")
            });
        if let Err(payload) = thread.join() {
            std::panic::resume_unwind(payload);
        }
    }
    #[cfg(not(windows))]
    launch();
}

/// Report the stack the application thread runs with, so a Windows host can confirm the
/// launch policy instead of inferring it from crashes.
///
/// The reservation always meets the configured budget: the entrypoints promote the
/// application to it, and Windows honors a thread's requested reservation exactly. The
/// warning below is for the budget itself being smaller than the framework default, which
/// is a deliberate measurement override and not a stack the recursive frame passes fit.
/// Stack overflow aborts without reaching the panic hook, so an unnoticed undersized stack
/// would otherwise surface as an unexplained exit during the first complex frame.
#[cfg(windows)]
pub(super) fn report_app_stack(level: super::LogLevel) {
    let Some(reserved) = reserved_stack_bytes() else {
        return;
    };
    if reserved >= DEFAULT_APP_STACK_BYTES {
        super::host_log(
            level,
            super::LogLevel::Info,
            format!("application thread stack: {reserved} bytes reserved"),
        );
        return;
    }
    super::host_log(
        level,
        super::LogLevel::Error,
        format!(
            "application thread reserves only {reserved} bytes of stack, below the \
             {DEFAULT_APP_STACK_BYTES} byte default GPUI's recursive layout and paint passes need \
             on Windows; unset {APP_STACK_BYTES_ENV} unless this budget was measured"
        ),
    );
}

/// The configured application stack budget, or a fatal diagnostic for a bad setting.
#[cfg(windows)]
fn resolved_stack_budget() -> usize {
    match parse_stack_budget(std::env::var(APP_STACK_BYTES_ENV).ok().as_deref()) {
        Ok(bytes) => bytes,
        Err(reason) => {
            eprintln!("solid-gpui-host: invalid {APP_STACK_BYTES_ENV}: {reason}");
            std::process::exit(2);
        }
    }
}

/// Parse the configured application stack budget.
///
/// An unset variable takes the default. A present value must be a positive byte count:
/// silently substituting the default would hide a mistyped limit until the application
/// overflows, which is the failure this policy exists to prevent.
#[cfg(any(windows, test))]
fn parse_stack_budget(value: Option<&str>) -> Result<usize, String> {
    let Some(value) = value else {
        return Ok(DEFAULT_APP_STACK_BYTES);
    };
    match value.trim().parse::<usize>() {
        Ok(0) => Err("expected a byte count greater than zero".to_owned()),
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(format!("{value:?} is not a byte count")),
    }
}

/// The stack Windows reserved for the calling thread.
#[cfg(windows)]
fn reserved_stack_bytes() -> Option<usize> {
    let mut low = 0usize;
    let mut high = 0usize;
    // SAFETY: the function only writes the two out-parameters.
    unsafe {
        windows::Win32::System::Threading::GetCurrentThreadStackLimits(&mut low, &mut high);
    }
    (high > low).then(|| high - low)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_budget_rejects_unusable_settings_instead_of_defaulting() {
        assert_eq!(parse_stack_budget(None), Ok(DEFAULT_APP_STACK_BYTES));
        assert_eq!(parse_stack_budget(Some("4194304")), Ok(4194304));
        assert_eq!(parse_stack_budget(Some(" 4194304 ")), Ok(4194304));
        assert!(parse_stack_budget(Some("0")).is_err());
        assert!(parse_stack_budget(Some("16MiB")).is_err());
        assert!(parse_stack_budget(Some("")).is_err());
    }
}

/// The Windows launch policy itself, exercised through the function the public entrypoints
/// call. These run on any Windows host (or a cross-built test binary), which is the only
/// place the reservation is real; off Windows the function runs inline by design.
#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;

    /// The initial thread of a default Rust binary reserves 1 MiB, which GPUI's recursive
    /// layout and paint passes overflow, so the entrypoints must hand the application a
    /// thread that reserves the configured budget instead.
    #[test]
    fn a_small_calling_thread_promotes_the_application_to_the_budget() {
        let budget = resolved_stack_budget();
        let observed = thread::Builder::new()
            .stack_size(1024 * 1024)
            .spawn(|| {
                let (sender, receiver) = mpsc::channel();
                launch_on_app_thread(move || {
                    let _ = sender.send(reserved_stack_bytes());
                });
                receiver.recv().expect("launch runs the closure")
            })
            .expect("spawn harness thread")
            .join()
            .expect("harness thread");
        let reserved = observed.expect("Windows reports thread stack limits");
        assert!(
            reserved >= budget,
            "application thread reserved {reserved} bytes, below the {budget} byte budget"
        );
    }

    /// A host that already owns a thread with enough stack (a `/STACK` override, or a host
    /// that entered from its own worker) keeps it: the policy must not add a second thread.
    #[test]
    fn an_adequate_calling_thread_stack_runs_inline() {
        let budget = resolved_stack_budget();
        let (harness, launched) = thread::Builder::new()
            .stack_size(budget * 2)
            .spawn(|| {
                let harness = thread::current().id();
                let (sender, receiver) = mpsc::channel();
                launch_on_app_thread(move || {
                    let _ = sender.send((thread::current().id(), reserved_stack_bytes()));
                });
                (harness, receiver.recv().expect("launch runs the closure"))
            })
            .expect("spawn harness thread")
            .join()
            .expect("harness thread");
        let (thread, reserved) = launched;
        assert_eq!(
            thread, harness,
            "an adequate stack must not move the application to another thread"
        );
        assert!(
            reserved.expect("Windows reports thread stack limits") >= budget,
            "the calling thread already reserves the budget"
        );
    }
}
