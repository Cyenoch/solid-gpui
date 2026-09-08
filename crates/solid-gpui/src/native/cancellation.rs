use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::Notify;

/// Invocation-scoped cancellation for cooperative blocking work and child tasks.
/// The host sets it when the invocation is cancelled or its Surface is disposed.
#[derive(Clone, Default)]
pub struct NativeCallContext {
    state: Arc<CancellationState>,
}

#[derive(Default)]
struct CancellationState {
    cancelled: AtomicBool,
    changed: Notify,
}

impl NativeCallContext {
    pub fn is_cancelled(&self) -> bool {
        self.state.cancelled.load(Ordering::Acquire)
    }

    pub fn check_cancelled(&self) -> Result<(), String> {
        if self.is_cancelled() {
            Err("native command was cancelled".into())
        } else {
            Ok(())
        }
    }

    pub async fn cancelled(&self) {
        let notified = self.state.changed.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        if !self.is_cancelled() {
            notified.await;
        }
    }

    pub(super) fn cancel(&self) {
        self.state.cancelled.store(true, Ordering::Release);
        self.state.changed.notify_waiters();
    }
}
