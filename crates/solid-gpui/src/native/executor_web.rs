//! Browser command futures are polled by GPUI; blocking work has no web executor.
use super::NativeCallContext;
use futures::future::BoxFuture;
use std::{future::Future, sync::Arc};
use tokio::sync::Semaphore;

pub(super) struct NativeExecutor {
    permits: Arc<Semaphore>,
}
impl Default for NativeExecutor {
    fn default() -> Self {
        Self {
            permits: Arc::new(Semaphore::new(128)),
        }
    }
}
impl NativeExecutor {
    pub fn asynchronous<F, Fut>(&self, function: F) -> BoxFuture<'static, Result<Vec<u8>, String>>
    where
        F: FnOnce(NativeCallContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<Vec<u8>, String>> + Send + 'static,
    {
        let permit = self.permits.clone().try_acquire_owned();
        let context = NativeCallContext::default();
        let cancellation = CancelOnDrop {
            context: context.clone(),
            completed: false,
        };
        Box::pin(async move {
            let mut cancellation = cancellation;
            let _permit = permit.map_err(|_| "native executor capacity exceeded".to_owned())?;
            let result = function(context).await;
            cancellation.completed = true;
            result
        })
    }
    pub fn blocking<F>(&self, _function: F) -> BoxFuture<'static, Result<Vec<u8>, String>>
    where
        F: FnOnce(NativeCallContext) -> Result<Vec<u8>, String> + Send + 'static,
    {
        Box::pin(async { Err("blocking native commands are unavailable in the browser".into()) })
    }
}

struct CancelOnDrop {
    context: NativeCallContext,
    completed: bool,
}
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        if !self.completed {
            self.context.cancel();
        }
    }
}
