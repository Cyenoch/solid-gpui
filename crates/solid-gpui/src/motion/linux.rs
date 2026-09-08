use super::MotionSource;
use ashpd::desktop::settings::{APPEARANCE_NAMESPACE, REDUCED_MOTION_KEY, ReducedMotion, Settings};
use futures::StreamExt;
use tokio::{
    runtime::{Builder, Runtime},
    sync::watch,
    task::JoinHandle,
};
pub(super) struct Subscription {
    task: JoinHandle<()>,
    runtime: Option<Runtime>,
}
impl Drop for Subscription {
    fn drop(&mut self) {
        self.task.abort();
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}
pub(super) fn subscribe(sender: watch::Sender<MotionSource>) -> Result<Subscription, String> {
    // This host service owns its reactor, independently of optional native modules.
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .thread_name("solid-gpui-settings")
        .build()
        .map_err(|error| error.to_string())?;
    let task = runtime.spawn(async move {
        let result: Result<(), String> = async {
            let settings = Settings::new().await.map_err(|error| error.to_string())?;
            let changes = settings
                .receive_setting_changed_with_args::<ReducedMotion>(
                    APPEARANCE_NAMESPACE,
                    REDUCED_MOTION_KEY,
                )
                .await
                .map_err(|error| error.to_string())?;
            futures::pin_mut!(changes);
            let reduced = settings
                .reduced_motion()
                .await
                .map_err(|error| error.to_string())?;
            sender.send_replace(MotionSource::Available {
                reduced: reduced == ReducedMotion::ReducedMotion,
            });
            while let Some(change) = changes.next().await {
                change.map_err(|error| error.to_string())?;
                // Re-read after notification so pre-query queued events cannot restore an older value.
                let reduced = settings
                    .reduced_motion()
                    .await
                    .map_err(|error| error.to_string())?;
                sender.send_replace(MotionSource::Available {
                    reduced: reduced == ReducedMotion::ReducedMotion,
                });
            }
            Err("system motion notification stream ended".into())
        }
        .await;
        if let Err(message) = result {
            sender.send_replace(MotionSource::Unavailable { message });
        }
    });
    Ok(Subscription {
        task,
        runtime: Some(runtime),
    })
}
