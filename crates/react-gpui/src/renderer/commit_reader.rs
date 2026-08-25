use std::sync::Arc;
use std::thread;

use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use gpui::{App, Entity};

use crate::protocol::ProtocolError;
use crate::transport::{RuntimeAdapter, RuntimeStatus, fatal_runtime_failure};

use super::ReactRoot;

enum CommitReaderMessage {
    Payload(Vec<u8>),
    Terminated(RuntimeStatus),
    Transport(ProtocolError),
}

impl ReactRoot {
    /// Start a dedicated blocking frame reader and forward each complete frame
    /// to the GPUI foreground executor. The transport thread never touches App,
    /// Window, Entity, or NodeStore state. The channel is bounded; a stalled
    /// foreground executor applies backpressure to the renderer process.
    ///
    /// Terminal transport and commit-validation failures are delivered through
    /// the same channel. The foreground task logs them, shuts down the runtime,
    /// and exits nonzero; explicit runtime shutdown is not treated as a failure.
    pub fn start_commit_reader(entity: Entity<Self>, runtime: Arc<dyn RuntimeAdapter>, cx: &App) {
        let (mut sender, mut receiver) = mpsc::channel::<CommitReaderMessage>(32);
        let runtime_for_reader = Arc::clone(&runtime);
        thread::Builder::new()
            .name("react-gpui-commit-reader".into())
            .spawn(move || {
                loop {
                    match runtime_for_reader.recv_commit() {
                        Ok(Some(payload)) => {
                            runtime_for_reader.tap_inbound_payload(&payload);
                            if futures::executor::block_on(
                                sender.send(CommitReaderMessage::Payload(payload)),
                            )
                            .is_err()
                            {
                                break;
                            }
                        }
                        Ok(None) => {
                            let status = match runtime_for_reader.status() {
                                RuntimeStatus::Running => RuntimeStatus::Exited {
                                    code: None,
                                    signal: None,
                                },
                                status => status,
                            };
                            let _ = futures::executor::block_on(
                                sender.send(CommitReaderMessage::Terminated(status)),
                            );
                            break;
                        }
                        Err(error) => {
                            let _ = futures::executor::block_on(
                                sender.send(CommitReaderMessage::Transport(error)),
                            );
                            break;
                        }
                    }
                }
            })
            .expect("failed to start React GPUI commit reader");

        cx.spawn(async move |cx| {
            while let Some(message) = receiver.next().await {
                match message {
                    CommitReaderMessage::Payload(payload) => {
                        let result = entity.update(cx, |root, cx| root.apply_payload(&payload, cx));
                        if let Err(error) = result {
                            fatal_runtime_failure(
                                runtime.as_ref(),
                                "rejected renderer commit",
                                error,
                            );
                        }
                    }
                    CommitReaderMessage::Terminated(status) => {
                        if status.is_failure() {
                            fatal_runtime_failure(
                                runtime.as_ref(),
                                "renderer runtime terminated unexpectedly",
                                status,
                            );
                        }
                        break;
                    }
                    CommitReaderMessage::Transport(error) => {
                        fatal_runtime_failure(runtime.as_ref(), "renderer transport error", error);
                    }
                }
            }
        })
        .detach();
    }
}
