//! Host-owned Tokio execution; no process-global runtime or borrowed enter guard.

use futures::future::BoxFuture;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, OnceLock},
    task::{Context, Poll},
};
use tokio::{
    runtime::{Builder, Runtime},
    sync::{OwnedSemaphorePermit, Semaphore},
    task::{JoinError, JoinHandle},
};

const MAX_IN_FLIGHT: usize = 128;
type Reply = Result<Vec<u8>, String>;

/// The registered modules own this executor. Initialization is lazy so merely
/// describing bindings does not start runtime threads. Composition shares one
/// executor and one admission limit across every module and surface.
pub(super) struct NativeExecutor {
    runtime: OnceLock<Result<Runtime, String>>,
    permits: Arc<Semaphore>,
}

impl Default for NativeExecutor {
    fn default() -> Self {
        Self {
            runtime: OnceLock::new(),
            permits: Arc::new(Semaphore::new(MAX_IN_FLIGHT)),
        }
    }
}

impl NativeExecutor {
    fn admit(&self) -> Result<(&Runtime, OwnedSemaphorePermit), String> {
        let permit = Arc::clone(&self.permits)
            .try_acquire_owned()
            .map_err(|_| "native executor capacity exceeded".to_owned())?;
        let runtime = self.runtime.get_or_init(|| {
            let workers = std::thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1)
                .clamp(1, 4);
            Builder::new_multi_thread()
                .worker_threads(workers)
                .max_blocking_threads(32)
                .thread_name("solid-gpui-native")
                .enable_all()
                .build()
                .map_err(|error| format!("could not start native executor: {error}"))
        });
        Ok((runtime.as_ref().map_err(Clone::clone)?, permit))
    }

    pub fn asynchronous<F>(&self, future: F) -> BoxFuture<'static, Reply>
    where
        F: Future<Output = Reply> + Send + 'static,
    {
        match self.admit() {
            Ok((runtime, permit)) => Box::pin(AbortOnDrop {
                join: runtime.spawn(async move {
                    // The actual task owns admission, not its observer. Dropping
                    // the observer aborts this future and eventually drops this permit.
                    let _permit = permit;
                    future.await
                }),
            }),
            Err(error) => Box::pin(std::future::ready(Err(error))),
        }
    }

    pub fn blocking<F>(&self, function: F) -> BoxFuture<'static, Reply>
    where
        F: FnOnce() -> Reply + Send + 'static,
    {
        match self.admit() {
            Ok((runtime, permit)) => Box::pin(AbortOnDrop {
                join: runtime.spawn_blocking(move || {
                    // Tokio cannot stop a blocking closure after it starts. Keep
                    // its permit until it returns, even if its result was cancelled.
                    let _permit = permit;
                    function()
                }),
            }),
            Err(error) => Box::pin(std::future::ready(Err(error))),
        }
    }
}

impl Drop for NativeExecutor {
    fn drop(&mut self) {
        if let Some(Ok(runtime)) = self.runtime.take() {
            // The final owner may be dropped on GPUI foreground or inside an
            // async task. Never wait there for workers or blocking commands.
            runtime.shutdown_background();
        }
    }
}

/// Dropping a plain Tokio JoinHandle detaches its task. This wrapper instead
/// connects cancellation of the GPUI request future to Tokio task abortion.
struct AbortOnDrop {
    join: JoinHandle<Reply>,
}

impl Future for AbortOnDrop {
    type Output = Reply;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Reply> {
        Pin::new(&mut self.join)
            .poll(cx)
            .map(|result| result.unwrap_or_else(|error| Err(join_error(error))))
    }
}

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.join.abort();
    }
}

fn join_error(error: JoinError) -> String {
    if error.is_cancelled() {
        return "native command was cancelled".into();
    }
    if error.is_panic() {
        let payload = error.into_panic();
        let message = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| payload.downcast_ref::<&str>().copied())
            .unwrap_or("non-string panic");
        return format!("native command panicked: {message}");
    }
    format!("native command failed: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::mpsc,
        time::{Duration, Instant},
    };

    struct DropSignal(mpsc::Sender<()>);
    impl Drop for DropSignal {
        fn drop(&mut self) {
            let _ = self.0.send(());
        }
    }

    #[test]
    fn cancellation_aborts_tasks_and_releases_global_admission() {
        let executor = NativeExecutor::default();
        let (started_tx, started_rx) = mpsc::channel();
        let (dropped_tx, dropped_rx) = mpsc::channel();
        let mut calls = Vec::new();
        for _ in 0..MAX_IN_FLIGHT {
            let started = started_tx.clone();
            let dropped = dropped_tx.clone();
            calls.push(executor.asynchronous(async move {
                let _drop = DropSignal(dropped);
                started.send(()).unwrap();
                std::future::pending::<Reply>().await
            }));
        }
        for _ in 0..MAX_IN_FLIGHT {
            started_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        }
        let overflow = futures::executor::block_on(executor.asynchronous(async { Ok(vec![]) }));
        assert_eq!(overflow.unwrap_err(), "native executor capacity exceeded");
        drop(calls);
        for _ in 0..MAX_IN_FLIGHT {
            dropped_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        while executor.permits.available_permits() != MAX_IN_FLIGHT {
            assert!(
                Instant::now() < deadline,
                "aborted tasks must release admission"
            );
            std::thread::yield_now();
        }
        assert!(futures::executor::block_on(executor.asynchronous(async { Ok(vec![]) })).is_ok());
    }

    #[test]
    fn cancelled_blocking_work_keeps_its_permit_until_it_exits() {
        let executor = NativeExecutor {
            runtime: OnceLock::new(),
            permits: Arc::new(Semaphore::new(1)),
        };
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let call = executor.blocking(move || {
            started_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            Ok(vec![])
        });
        started_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        drop(call);
        assert_eq!(
            futures::executor::block_on(executor.asynchronous(async { Ok(vec![]) })).unwrap_err(),
            "native executor capacity exceeded"
        );
        release_tx.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while executor.permits.available_permits() != 1 {
            assert!(
                Instant::now() < deadline,
                "finished blocking task must release admission"
            );
            std::thread::yield_now();
        }
    }

    #[test]
    fn blocking_commands_do_not_occupy_the_async_worker() {
        let executor = NativeExecutor {
            runtime: OnceLock::from(Ok(Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .unwrap())),
            permits: Arc::new(Semaphore::new(2)),
        };
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let blocking = executor.blocking(move || {
            started_tx.send(()).unwrap();
            release_rx
                .recv_timeout(Duration::from_secs(3))
                .map_err(|error| error.to_string())?;
            Ok(vec![])
        });
        started_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        let asynchronous = executor.asynchronous(async move {
            tokio::time::sleep(Duration::from_millis(1)).await;
            release_tx.send(()).map_err(|error| error.to_string())?;
            Ok(vec![])
        });
        assert!(futures::executor::block_on(asynchronous).is_ok());
        assert!(futures::executor::block_on(blocking).is_ok());
    }

    #[test]
    fn dropping_owner_shuts_down_pending_async_work_without_waiting() {
        let executor = NativeExecutor::default();
        let (started_tx, started_rx) = mpsc::channel();
        let (dropped_tx, dropped_rx) = mpsc::channel();
        let call = executor.asynchronous(async move {
            let _drop = DropSignal(dropped_tx);
            started_tx.send(()).unwrap();
            std::future::pending::<Reply>().await
        });
        started_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        drop(executor);
        dropped_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        assert_eq!(
            futures::executor::block_on(call).unwrap_err(),
            "native command was cancelled"
        );
    }
}
